package interactor

import (
	"context"
	"fmt"
	"sync"
	"time"

	"github.com/reearth/reearth-flow/api/internal/usecase"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/job"
	"github.com/reearth/reearth-flow/api/pkg/job/monitor"
	"github.com/reearth/reearth-flow/api/pkg/notification"
	"github.com/reearth/reearth-flow/api/pkg/subscription"
	"github.com/reearth/reearthx/account/accountdomain"
	"github.com/reearth/reearthx/account/accountusecase/accountrepo"
	"github.com/reearth/reearthx/log"
	"github.com/reearth/reearthx/usecasex"
)

type Job struct {
	common
	jobRepo       repo.Job
	workspaceRepo accountrepo.Workspace
	transaction   usecasex.Transaction
	file          gateway.File
	batch         gateway.Batch
	monitor       *monitor.Monitor
	subscriptions *subscription.JobManager
	notifier      notification.Notifier
	activeWatchers    map[string]bool
	watchersMu        sync.Mutex
}

type NotificationPayload struct {
	RunID        string   `json:"runId"`
	DeploymentID string   `json:"deploymentId"`
	Status       string   `json:"status"`
	Logs         []string `json:"logs"`
	Outputs      []string `json:"outputs"`
}

func NewJob(r *repo.Container, gr *gateway.Container) interfaces.Job {
	return &Job{
		jobRepo:       r.Job,
		workspaceRepo: r.Workspace,
		transaction:   r.Transaction,
		file:          gr.File,
		batch:         gr.Batch,
		monitor:       monitor.NewMonitor(),
		subscriptions: subscription.NewJobManager(),
		notifier:      notification.NewHTTPNotifier(),
		activeWatchers:    make(map[string]bool),
		watchersMu:        sync.Mutex{},
	}
}

func (i *Job) Cancel(ctx context.Context, jobID id.JobID, operator *usecase.Operator) (*job.Job, error) {
	j, err := i.jobRepo.FindByID(ctx, jobID)
	if err != nil {
		return nil, err
	}

	if err := i.CanWriteWorkspace(j.Workspace(), operator); err != nil {
		return nil, err
	}

	if j.Status() != job.StatusPending && j.Status() != job.StatusRunning {
		return nil, fmt.Errorf("job cannot be cancelled: current status is %s", j.Status())
	}

	tx, err := i.transaction.Begin(ctx)
	if err != nil {
		return nil, err
	}
	defer func() {
		if err := tx.End(ctx); err != nil {
			log.Errorfc(ctx, "transaction end failed: %v", err)
		}
	}()

	if err := i.batch.CancelJob(ctx, j.GCPJobID()); err != nil {
		return nil, err
	}

	j.SetStatus(job.StatusCancelled)
	now := time.Now()
	j.SetCompletedAt(&now)

	if err := i.jobRepo.Save(ctx, j); err != nil {
		return nil, err
	}

	tx.Commit()

	if err := i.handleJobCompletion(ctx, j); err != nil {
		log.Errorfc(ctx, "job: completion handling failed: %v", err)
	}

	i.subscriptions.Notify(j.ID().String(), j.Status())
	i.monitor.Remove(j.ID().String())

	return j, nil
}

func (i *Job) FindByID(ctx context.Context, id id.JobID, operator *usecase.Operator) (*job.Job, error) {
	j, err := i.jobRepo.FindByID(ctx, id)
	if err != nil {
		return nil, err
	}
	if err := i.CanReadWorkspace(j.Workspace(), operator); err != nil {
		return nil, err
	}
	return j, nil
}

func (i *Job) Fetch(ctx context.Context, ids []id.JobID, operator *usecase.Operator) ([]*job.Job, error) {
	jobs, err := i.jobRepo.FindByIDs(ctx, ids)
	if err != nil {
		return nil, err
	}
	return i.filterReadableJobs(jobs, operator), nil
}

func (i *Job) FindByWorkspace(ctx context.Context, wsID accountdomain.WorkspaceID, p *interfaces.PaginationParam, operator *usecase.Operator) ([]*job.Job, *interfaces.PageBasedInfo, error) {
	if err := i.CanReadWorkspace(wsID, operator); err != nil {
		return nil, nil, err
	}
	return i.jobRepo.FindByWorkspace(ctx, wsID, p)
}

func (i *Job) GetStatus(ctx context.Context, jobID id.JobID, operator *usecase.Operator) (job.Status, error) {
	j, err := i.jobRepo.FindByID(ctx, jobID)
	if err != nil {
		return "", err
	}
	return j.Status(), nil
}

func (i *Job) StartMonitoring(ctx context.Context, j *job.Job, notificationURL *string, operator *usecase.Operator) error {
	log.Debugfc(ctx, "job: starting monitoring for jobID=%s workspace=%s", j.ID(), j.Workspace())

	monitorCtx, cancel := context.WithCancel(context.Background())

	i.monitor.Register(j.ID().String(), &monitor.Config{
		Cancel:          cancel,
		NotificationURL: notificationURL,
	})

	go i.runMonitoringLoop(monitorCtx, j)

	return nil
}

func (i *Job) runMonitoringLoop(ctx context.Context, j *job.Job) {
	ticker := time.NewTicker(10 * time.Second)
	defer ticker.Stop()

	for {
		select {
		case <-ctx.Done():
			return
		case <-ticker.C:
			if err := i.checkJobStatus(ctx, j); err != nil {
				log.Errorfc(ctx, "job: status check failed: %v", err)
			}
		}
	}
}

func (i *Job) checkJobStatus(ctx context.Context, j *job.Job) error {
	status, err := i.batch.GetJobStatus(ctx, j.GCPJobID())
	if err != nil {
		return err
	}

	if j.Status() != job.Status(status) {
		if err := i.updateJobStatus(ctx, j, job.Status(status)); err != nil {
			return err
		}
	}

	if status == gateway.JobStatusCompleted || status == gateway.JobStatusFailed {
		if err := i.handleJobCompletion(ctx, j); err != nil {
			log.Errorfc(ctx, "job: completion handling failed: %v", err)
		}
		i.monitor.Remove(j.ID().String())
	}

	return nil
}

func (i *Job) updateJobStatus(ctx context.Context, j *job.Job, status job.Status) error {
	tx, err := i.transaction.Begin(ctx)
	if err != nil {
		return err
	}
	defer func() {
		if err := tx.End(ctx); err != nil {
			log.Errorfc(ctx, "transaction end failed: %v", err)
		}
	}()

	j.SetStatus(status)
	if err := i.jobRepo.Save(ctx, j); err != nil {
		return err
	}

	tx.Commit()
	i.subscriptions.Notify(j.ID().String(), j.Status())
	return nil
}

func (i *Job) handleJobCompletion(ctx context.Context, j *job.Job) error {
	if j == nil {
		return fmt.Errorf("job cannot be nil")
	}

	jobID := j.ID().String()
	log.Debugfc(ctx, "job: handling completion for jobID=%s with status=%s", jobID, j.Status())

	config := i.monitor.Get(jobID)

	if err := i.updateJobArtifacts(ctx, j); err != nil {
		log.Errorfc(ctx, "job: failed to update artifacts for jobID=%s: %v", jobID, err)
	}

	if err := i.saveJobState(ctx, j); err != nil {
		return fmt.Errorf("failed to save job state: %w", err)
	}

	if config == nil || config.NotificationURL == nil || *config.NotificationURL == "" {
		return nil
	}

	if err := i.sendCompletionNotification(ctx, j, *config.NotificationURL); err != nil {
		return fmt.Errorf("failed to send notification: %w", err)
	}

	return nil
}

func (i *Job) updateJobArtifacts(ctx context.Context, j *job.Job) error {
	jobID := j.ID().String()

	outputs, err := i.file.ListJobArtifacts(ctx, jobID)
	if err != nil {
		return fmt.Errorf("failed to list job artifacts: %w", err)
	}
	j.SetOutputURLs(outputs)

	logURL := i.file.GetJobLogURL(jobID)
	if logURL != "" {
		j.SetLogsURL(logURL)
	}

	return nil
}

func (i *Job) saveJobState(ctx context.Context, j *job.Job) error {
	tx, err := i.transaction.Begin(ctx)
	if err != nil {
		return fmt.Errorf("failed to begin transaction: %w", err)
	}

	defer func() {
		if err := tx.End(ctx); err != nil {
			log.Errorfc(ctx, "transaction end failed: %v", err)
		}
	}()

	if err := i.jobRepo.Save(ctx, j); err != nil {
		return fmt.Errorf("failed to save job: %w", err)
	}

	tx.Commit()
	return nil
}

func (i *Job) sendCompletionNotification(ctx context.Context, j *job.Job, notificationURL string) error {
	jobID := j.ID().String()

	status := "failed"
	switch j.Status() {
	case job.StatusCompleted:
		status = "succeeded"
	case job.StatusCancelled:
		status = "cancelled"
	}

	var logs []string
	logExists, err := i.file.CheckJobLogExists(ctx, jobID)
	if err != nil {
		log.Warnfc(ctx, "job: failed to check log existence for jobID=%s: %v", jobID, err)
	} else if logExists {
		logs = append(logs, j.LogsURL())
	}

	payload := notification.Payload{
		RunID:        jobID,
		DeploymentID: j.Deployment().String(),
		Status:       status,
		Logs:         logs,
		Outputs:      j.OutputURLs(),
	}

	log.Debugfc(ctx, "job: sending notification for jobID=%s to URL=%s", jobID, notificationURL)

	return i.notifier.Send(notificationURL, payload)
}

func (i *Job) Subscribe(ctx context.Context, jobID id.JobID, operator *usecase.Operator) (chan job.Status, error) {
	j, err := i.FindByID(ctx, jobID, operator)
	if err != nil {
		return nil, err
	}

	ch := i.subscriptions.Subscribe(jobID.String())
	ch <- j.Status()

	i.watchersMu.Lock()
	if i.activeWatchers == nil {
		i.activeWatchers = make(map[string]bool)
	}
	i.activeWatchers[jobID.String()] = true
	i.watchersMu.Unlock()

	return ch, nil
}

func (i *Job) Unsubscribe(jobID id.JobID, ch chan job.Status) {
	i.subscriptions.Unsubscribe(jobID.String(), ch)

	if i.subscriptions.CountSubscribers(jobID.String()) == 0 {
		i.watchersMu.Lock()
		delete(i.activeWatchers, jobID.String())
		i.watchersMu.Unlock()
	}
}

func (i *Job) filterReadableJobs(jobs []*job.Job, operator *usecase.Operator) []*job.Job {
	result := make([]*job.Job, 0, len(jobs))
	for _, j := range jobs {
		if i.CanReadWorkspace(j.Workspace(), operator) == nil {
			result = append(result, j)
		}
	}
	return result
}
