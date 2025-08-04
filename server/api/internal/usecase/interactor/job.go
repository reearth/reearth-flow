package interactor

import (
	"context"
	"fmt"
	"sync"
	"time"

	"github.com/reearth/reearth-flow/api/internal/rbac"
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

var _ interfaces.Job = &Job{}

type Job struct {
	jobRepo           repo.Job
	workspaceRepo     accountrepo.Workspace
	transaction       usecasex.Transaction
	file              gateway.File
	batch             gateway.Batch
	monitor           *monitor.Monitor
	subscriptions     *subscription.JobManager
	notifier          notification.Notifier
	permissionChecker gateway.PermissionChecker
	watchersMu        sync.Mutex
	activeWatchers    map[string]bool
	jobLocksMu        sync.RWMutex
	jobLocks          map[string]*sync.Mutex
}

type NotificationPayload struct {
	RunID        string   `json:"runId"`
	DeploymentID string   `json:"deploymentId"`
	Status       string   `json:"status"`
	Logs         []string `json:"logs"`
	Outputs      []string `json:"outputs"`
}

func NewJob(
	r *repo.Container,
	gr *gateway.Container,
	permissionChecker gateway.PermissionChecker,
) interfaces.Job {
	return &Job{
		jobRepo:           r.Job,
		workspaceRepo:     r.Workspace,
		transaction:       r.Transaction,
		file:              gr.File,
		batch:             gr.Batch,
		monitor:           monitor.NewMonitor(),
		subscriptions:     subscription.NewJobManager(),
		notifier:          notification.NewHTTPNotifier(),
		permissionChecker: permissionChecker,
		activeWatchers:    make(map[string]bool),
		jobLocks:          make(map[string]*sync.Mutex),
	}
}

func (i *Job) getJobLock(jobID string) *sync.Mutex {
	i.jobLocksMu.RLock()
	if lock, exists := i.jobLocks[jobID]; exists {
		i.jobLocksMu.RUnlock()
		return lock
	}
	i.jobLocksMu.RUnlock()

	i.jobLocksMu.Lock()
	defer i.jobLocksMu.Unlock()

	if lock, exists := i.jobLocks[jobID]; exists {
		return lock
	}

	lock := &sync.Mutex{}
	i.jobLocks[jobID] = lock
	return lock
}

func (i *Job) cleanupJobLock(jobID string) {
	i.jobLocksMu.Lock()
	defer i.jobLocksMu.Unlock()
	delete(i.jobLocks, jobID)
}

func (i *Job) checkPermission(ctx context.Context, action string) error {
	return checkPermission(ctx, i.permissionChecker, rbac.ResourceJob, action)
}

func (i *Job) Cancel(ctx context.Context, jobID id.JobID) (*job.Job, error) {
	if err := i.checkPermission(ctx, rbac.ActionAny); err != nil {
		return nil, err
	}

	jobLock := i.getJobLock(jobID.String())
	jobLock.Lock()
	defer jobLock.Unlock()

	j, err := i.jobRepo.FindByID(ctx, jobID)
	if err != nil {
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

	return j, nil
}

func (i *Job) FindByID(ctx context.Context, id id.JobID) (*job.Job, error) {
	if err := i.checkPermission(ctx, rbac.ActionAny); err != nil {
		return nil, err
	}

	return i.jobRepo.FindByID(ctx, id)
}

func (i *Job) Fetch(ctx context.Context, ids []id.JobID) ([]*job.Job, error) {
	if err := i.checkPermission(ctx, rbac.ActionAny); err != nil {
		return nil, err
	}

	return i.jobRepo.FindByIDs(ctx, ids)
}

func (i *Job) FindByWorkspace(
	ctx context.Context,
	wsID accountdomain.WorkspaceID,
	p *interfaces.PaginationParam,
) ([]*job.Job, *interfaces.PageBasedInfo, error) {
	if err := i.checkPermission(ctx, rbac.ActionAny); err != nil {
		return nil, nil, err
	}

	return i.jobRepo.FindByWorkspace(ctx, wsID, p)
}

func (i *Job) GetStatus(ctx context.Context, jobID id.JobID) (job.Status, error) {
	if err := i.checkPermission(ctx, rbac.ActionAny); err != nil {
		return "", err
	}

	j, err := i.jobRepo.FindByID(ctx, jobID)
	if err != nil {
		return "", err
	}
	return j.Status(), nil
}

func (i *Job) StartMonitoring(ctx context.Context, j *job.Job, notificationURL *string) error {
	if err := i.checkPermission(ctx, rbac.ActionAny); err != nil {
		return err
	}

	log.Debugfc(ctx, "job: starting monitoring for jobID=%s workspace=%s", j.ID(), j.Workspace())

	i.watchersMu.Lock()
	defer i.watchersMu.Unlock()

	jobKey := j.ID().String()
	if i.activeWatchers == nil {
		i.activeWatchers = make(map[string]bool)
	}

	if _, exists := i.activeWatchers[jobKey]; exists {
		log.Debugfc(ctx, "job: monitoring already active for jobID=%s", jobKey)
		if notificationURL != nil {
			if config := i.monitor.Get(jobKey); config != nil {
				config.NotificationURL = notificationURL
			}
		}
		return nil
	}

	i.activeWatchers[jobKey] = true

	monitorCtx, cancel := context.WithCancel(context.Background())

	i.monitor.Register(jobKey, &monitor.Config{
		Cancel:          cancel,
		NotificationURL: notificationURL,
	})

	go i.runMonitoringLoop(monitorCtx, j)

	return nil
}

func (i *Job) runMonitoringLoop(ctx context.Context, j *job.Job) {
	jobID := j.ID().String()
	log.Infof("Starting pure event-driven monitoring for job ID %s (polling disabled)", jobID)

	defer func() {
		i.watchersMu.Lock()
		delete(i.activeWatchers, jobID)
		i.watchersMu.Unlock()
		i.monitor.Remove(jobID)
		i.cleanupJobLock(jobID)
		log.Infof("Event-driven monitoring cleanup completed for job ID %s", jobID)
	}()

	maxDuration := 24 * time.Hour

	select {
	case <-ctx.Done():
		log.Infof("Event-driven monitoring stopped by context cancellation for job ID %s", jobID)
		return
	case <-time.After(maxDuration):
		log.Warnf("Exceeded maximum monitoring duration for job ID %s (pure event-driven mode)", jobID)

		if currentJob, err := i.jobRepo.FindByID(context.Background(), j.ID()); err == nil {
			if currentJob.Status() == job.StatusCompleted ||
				currentJob.Status() == job.StatusFailed ||
				currentJob.Status() == job.StatusCancelled {
				log.Infof("Job ID %s is in terminal state %s, monitoring complete",
					jobID, currentJob.Status())
			} else {
				log.Warnf("Job ID %s still in non-terminal state %s after timeout - possible event system failure",
					jobID, currentJob.Status())
			}
		}
		return
	}
}

func (i *Job) handleJobCompletion(ctx context.Context, j *job.Job) error {
	if j == nil {
		return fmt.Errorf("job cannot be nil")
	}

	if err := i.checkPermission(ctx, rbac.ActionAny); err != nil {
		return err
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

	workerLogURL := i.file.GetJobWorkerLogURL(jobID)
	if workerLogURL != "" {
		j.SetWorkerLogsURL(workerLogURL)
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

func (i *Job) sendCompletionNotification(
	ctx context.Context,
	j *job.Job,
	notificationURL string,
) error {
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

	workerLogExists, err := i.file.CheckJobWorkerLogExists(ctx, jobID)
	if err != nil {
		log.Warnfc(ctx, "job: failed to check worker log existence for jobID=%s: %v", jobID, err)
	} else if workerLogExists {
		logs = append(logs, j.WorkerLogsURL())
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

func (i *Job) UpdateJobStatusFromEvent(jobID id.JobID, status job.Status) error {
	ctx := context.Background()

	j, err := i.jobRepo.FindByID(ctx, jobID)
	if err != nil {
		return fmt.Errorf("failed to find job: %w", err)
	}

	if j.Status() == status {
		return nil
	}

	log.Infof("Updating job %s status from %s to %s via event", jobID, j.Status(), status)

	tx, err := i.transaction.Begin(ctx)
	if err != nil {
		return fmt.Errorf("failed to begin transaction: %w", err)
	}

	defer func() {
		if err2 := tx.End(ctx); err2 != nil {
			log.Errorfc(ctx, "transaction end failed: %v", err2)
		}
	}()

	j.SetStatus(status)

	if status == job.StatusCompleted || status == job.StatusFailed || status == job.StatusCancelled {
		now := time.Now()
		j.SetCompletedAt(&now)
	}

	if err := i.jobRepo.Save(ctx, j); err != nil {
		return fmt.Errorf("failed to save job: %w", err)
	}

	tx.Commit()

	i.subscriptions.Notify(j.ID().String(), j.Status())

	if status == job.StatusCompleted || status == job.StatusFailed || status == job.StatusCancelled {
		if err := i.handleJobCompletion(ctx, j); err != nil {
			log.Errorfc(ctx, "job: completion handling failed: %v", err)
		}
	}

	return nil
}

func (i *Job) Subscribe(ctx context.Context, jobID id.JobID) (chan job.Status, error) {
	if err := i.checkPermission(ctx, rbac.ActionAny); err != nil {
		return nil, err
	}

	ch := i.subscriptions.Subscribe(jobID.String())

	go func() {
		j, err := i.FindByID(context.Background(), jobID)
		if err == nil {
			i.subscriptions.Notify(jobID.String(), j.Status())
		}
	}()

	i.startMonitoringIfNeeded(jobID)

	return ch, nil
}

func (i *Job) startMonitoringIfNeeded(jobID id.JobID) {
	j, err := i.jobRepo.FindByID(context.Background(), jobID)
	if err != nil {
		log.Errorfc(context.Background(), "job: failed to find job for monitoring: %v", err)
		return
	}

	status := j.Status()
	if status == job.StatusCompleted || status == job.StatusFailed ||
		status == job.StatusCancelled {
		return
	}

	if err := i.StartMonitoring(context.Background(), j, nil); err != nil {
		log.Errorfc(context.Background(), "job: failed to start monitoring: %v", err)
	}
}

func (i *Job) Unsubscribe(jobID id.JobID, ch chan job.Status) {
	i.subscriptions.Unsubscribe(jobID.String(), ch)

	if i.subscriptions.CountSubscribers(jobID.String()) == 0 {
		i.watchersMu.Lock()
		delete(i.activeWatchers, jobID.String())
		i.watchersMu.Unlock()
	}
}
