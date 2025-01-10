package interactor

import (
	"context"
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
	jobRepo       repo.Job
	workspaceRepo accountrepo.Workspace
	transaction   usecasex.Transaction
	file          gateway.File
	batch         gateway.Batch
	monitor       *monitor.Monitor
	subscriptions *subscription.Manager
	notifier      notification.Notifier
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
		subscriptions: subscription.NewManager(),
		notifier:      notification.NewHTTPNotifier(),
	}
}

func (i *Job) FindByID(ctx context.Context, id id.JobID) (*job.Job, error) {
	j, err := i.jobRepo.FindByID(ctx, id)
	if err != nil {
		return nil, err
	}
	return j, nil
}

func (i *Job) Fetch(ctx context.Context, ids []id.JobID) ([]*job.Job, error) {
	jobs, err := i.jobRepo.FindByIDs(ctx, ids)
	if err != nil {
		return nil, err
	}
	return jobs, nil
}

func (i *Job) FindByWorkspace(ctx context.Context, wsID accountdomain.WorkspaceID, p *interfaces.PaginationParam) ([]*job.Job, *interfaces.PageBasedInfo, error) {
	return i.jobRepo.FindByWorkspace(ctx, wsID, p)
}

func (i *Job) GetStatus(ctx context.Context, jobID id.JobID) (job.Status, error) {
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
	config := i.monitor.Get(j.ID().String())
	if config == nil || config.NotificationURL == nil {
		return nil
	}

	outputs, err := i.file.ListJobArtifacts(ctx, j.ID().String())
	if err != nil {
		return err
	}

	logURL := i.file.GetJobLogURL(j.ID().String())
	var logs []string

	if exists, _ := i.file.CheckJobLogExists(ctx, j.ID().String()); exists {
		logs = append(logs, logURL)
	}

	status := "failed"
	if j.Status() == job.StatusCompleted {
		status = "succeeded"
	}

	payload := notification.Payload{
		RunID:        j.ID().String(),
		DeploymentID: j.Deployment().String(),
		Status:       status,
		Logs:         logs,
		Outputs:      outputs,
	}

	log.Debugfc(ctx, "job: sending notification payload for jobID=%s: %+v", j.ID(), payload)

	return i.notifier.Send(*config.NotificationURL, payload)
}

func (i *Job) Subscribe(ctx context.Context, jobID id.JobID) (chan job.Status, error) {
	j, err := i.FindByID(ctx, jobID)
	if err != nil {
		return nil, err
	}

	ch := i.subscriptions.Subscribe(jobID.String())
	ch <- j.Status()
	return ch, nil
}

func (i *Job) Unsubscribe(jobID id.JobID, ch chan job.Status) {
	i.subscriptions.Unsubscribe(jobID.String(), ch)
}
