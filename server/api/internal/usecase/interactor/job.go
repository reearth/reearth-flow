package interactor

import (
	"context"
	"fmt"
	"os"
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
	job := &Job{
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

	log.Debugf("[NewJob] Created Job[%p] subs[%p] mon[%p]", job, job.subscriptions, job.monitor)

	return job
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
	log.Debugfc(ctx, "[StartMonitoring] Job[%p] subs[%p] mon[%p]", i, i.subscriptions, i.monitor)

	if err := i.checkPermission(ctx, rbac.ActionAny); err != nil {
		return err
	}

	log.Debugfc(ctx, "job: starting monitoring for jobID=%s workspace=%s", j.ID(), j.Workspace())

	hostname, _ := os.Hostname()
	log.Debugfc(ctx, "[%s] StartMonitoring called for job %s", hostname, j.ID())
	log.Debugfc(ctx, "[%s] Current active watchers: %v", hostname, i.activeWatchers)

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
	hostname, _ := os.Hostname()
	log.Debugfc(ctx, "[%s] Starting monitoring loop for job %s", hostname, j.ID().String())

	ticker := time.NewTicker(5 * time.Second)
	defer ticker.Stop()

	jobID := j.ID().String()
	log.Infof("Starting continuous monitoring for job ID %s", jobID)

	defer func() {
		i.watchersMu.Lock()
		delete(i.activeWatchers, jobID)
		i.watchersMu.Unlock()
		i.monitor.Remove(jobID)
		i.cleanupJobLock(jobID)
		log.Infof("Monitoring cleanup completed for job ID %s", jobID)
	}()

	maxDuration := 24 * time.Hour
	startTime := time.Now()

	for {
		select {
		case <-ctx.Done():
			log.Infof("Monitoring stopped by context cancellation for job ID %s", jobID)
			return
		case <-ticker.C:
			if time.Since(startTime) > maxDuration {
				log.Warnf("Exceeded maximum monitoring duration for job ID %s", jobID)
				return
			}

			currentJob, err := i.jobRepo.FindByID(context.Background(), j.ID())
			if err != nil {
				log.Errorf("Failed to fetch current job state for job ID %s: %v", jobID, err)
				continue
			}

			status := currentJob.Status()
			if status == job.StatusCompleted || status == job.StatusFailed ||
				status == job.StatusCancelled {
				log.Infof(
					"Job ID %s already in terminal state %s, stopping monitoring",
					jobID,
					status,
				)
				return
			}

			if err := i.checkJobStatus(ctx, currentJob); err != nil {
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

	newStatus := job.Status(status)

	jobLock := i.getJobLock(j.ID().String())
	jobLock.Lock()
	defer jobLock.Unlock()

	currentJob, err := i.jobRepo.FindByID(ctx, j.ID())
	if err != nil {
		return fmt.Errorf("failed to fetch current job state: %w", err)
	}

	statusChanged := currentJob.Status() != newStatus
	if !statusChanged {
		return nil
	}

	if err := i.updateJobStatus(ctx, currentJob, newStatus); err != nil {
		return err
	}

	isTerminalState := newStatus == job.StatusCompleted || newStatus == job.StatusFailed ||
		newStatus == job.StatusCancelled
	if isTerminalState {
		log.Infof(
			"Job %s transitioning to terminal state %s, handling completion",
			currentJob.ID(),
			newStatus,
		)

		currentJob.SetStatus(newStatus)

		if err := i.handleJobCompletion(ctx, currentJob); err != nil {
			log.Errorfc(ctx, "job: completion handling failed: %v", err)
		}
	}

	return nil
}

func (i *Job) updateJobStatus(ctx context.Context, j *job.Job, status job.Status) error {
	log.Debugfc(ctx, "[updateJobStatus] Job[%p] subs[%p] mon[%p]", i, i.subscriptions, i.monitor)

	hostname, _ := os.Hostname()
	log.Debugfc(ctx, "[%s] Updating job %s status from %s to %s", hostname, j.ID().String(), j.Status(), status)

	if err := i.checkPermission(ctx, rbac.ActionAny); err != nil {
		return err
	}

	tx, err := i.transaction.Begin(ctx)
	if err != nil {
		return err
	}

	var txErr error
	defer func() {
		if err2 := tx.End(ctx); err2 != nil && txErr == nil {
			log.Errorfc(ctx, "transaction end failed: %v", err2)
		}
	}()

	j.SetStatus(status)
	if err := i.jobRepo.Save(ctx, j); err != nil {
		txErr = err
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

	userFacingLogURL := i.file.GetJobUserFacingLogURL(jobID)
	if userFacingLogURL != "" {
		j.SetUserFacingLogsURL(userFacingLogURL)
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

	userFacingLogExists, err := i.file.CheckJobUserFacingLogExists(ctx, jobID)
	if err != nil {
		log.Warnfc(ctx, "job: failed to check user-facing log existence for jobID=%s: %v", jobID, err)
	} else if userFacingLogExists {
		logs = append(logs, j.UserFacingLogsURL())
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

func (i *Job) Subscribe(ctx context.Context, jobID id.JobID) (chan job.Status, error) {
	log.Debugfc(ctx, "[Subscribe] Job[%p] subs[%p] mon[%p]", i, i.subscriptions, i.monitor)

	if err := i.checkPermission(ctx, rbac.ActionAny); err != nil {
		return nil, err
	}

	ch := i.subscriptions.Subscribe(jobID.String())

	hostname, _ := os.Hostname()
	log.Debugfc(ctx, "[%s] WebSocket subscription started for job %s", hostname, jobID.String())

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
	log.Debugf("[startMonitoringIfNeeded] Job[%p] subs[%p] for job %s", i, i.subscriptions, jobID)

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
