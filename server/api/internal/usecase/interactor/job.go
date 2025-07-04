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
}

type NotificationPayload struct {
	RunID        string   `json:"runId"`
	DeploymentID string   `json:"deploymentId"`
	Status       string   `json:"status"`
	Logs         []string `json:"logs"`
	Outputs      []string `json:"outputs"`
}

func NewJob(r *repo.Container, gr *gateway.Container, permissionChecker gateway.PermissionChecker) interfaces.Job {
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
	}
}

func (i *Job) checkPermission(ctx context.Context, action string) error {
	return checkPermission(ctx, i.permissionChecker, rbac.ResourceJob, action)
}

func (i *Job) Cancel(ctx context.Context, jobID id.JobID) (*job.Job, error) {
	if err := i.checkPermission(ctx, rbac.ActionAny); err != nil {
		return nil, err
	}

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

	// Re-fetch to ensure we have the latest version
	freshJob, err := i.jobRepo.FindByID(ctx, jobID)
	if err != nil {
		return nil, err
	}

	// Check if status was already updated
	if freshJob.Status() == job.StatusCancelled {
		tx.Commit()
		return freshJob, nil
	}

	// Update with version check
	freshJob.SetStatus(job.StatusCancelled)
	now := time.Now()
	freshJob.SetCompletedAt(&now)
	freshJob.IncrementVersion()

	if err := i.jobRepo.Save(ctx, freshJob); err != nil {
		return nil, err
	}

	tx.Commit()

	if err := i.handleJobCompletion(ctx, freshJob); err != nil {
		log.Errorfc(ctx, "job: completion handling failed: %v", err)
	}

	i.subscriptions.Notify(freshJob.ID().String(), freshJob.Status())
	i.monitor.Remove(freshJob.ID().String())

	return freshJob, nil
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

func (i *Job) FindByWorkspace(ctx context.Context, wsID accountdomain.WorkspaceID, p *interfaces.PaginationParam) ([]*job.Job, *interfaces.PageBasedInfo, error) {
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

	monitorCtx, cancel := context.WithCancel(context.Background())

	i.monitor.Register(j.ID().String(), &monitor.Config{
		Cancel:          cancel,
		NotificationURL: notificationURL,
	})

	go i.runMonitoringLoop(monitorCtx, j)

	return nil
}

func (i *Job) runMonitoringLoop(ctx context.Context, j *job.Job) {
	// Start with shorter interval for active jobs
	baseInterval := 5 * time.Second
	maxInterval := 30 * time.Second
	currentInterval := baseInterval
	consecutiveErrors := 0

	ticker := time.NewTicker(currentInterval)
	defer ticker.Stop()

	jobID := j.ID().String()
	log.Infof("Starting adaptive monitoring for job ID %s", jobID)

	maxDuration := 24 * time.Hour
	startTime := time.Now()
	lastStatusChange := time.Now()

	for {
		select {
		case <-ctx.Done():
			log.Infof("Monitoring stopped by context cancellation for job ID %s", jobID)
			return
		case <-ticker.C:
			if time.Since(startTime) > maxDuration {
				log.Warnf("Exceeded maximum monitoring duration for job ID %s", jobID)
				i.monitor.Remove(jobID)
				return
			}

			currentJob, err := i.jobRepo.FindByID(ctx, j.ID())
			if err != nil {
				consecutiveErrors++
				log.Errorf("Failed to fetch current job state for job ID %s (error %d): %v",
					jobID, consecutiveErrors, err)

				// Exponential backoff on errors
				if consecutiveErrors > 3 {
					newInterval := time.Duration(float64(currentInterval) * 1.5)
					if newInterval > maxInterval {
						newInterval = maxInterval
					}
					if newInterval != currentInterval {
						currentInterval = newInterval
						ticker.Reset(currentInterval)
						log.Warnf("Increased polling interval to %v due to errors", currentInterval)
					}
				}
				continue
			}

			// Reset error counter on success
			consecutiveErrors = 0

			status := currentJob.Status()
			if status == job.StatusCompleted || status == job.StatusFailed || status == job.StatusCancelled {
				log.Infof("Job ID %s already in terminal state %s, stopping monitoring", jobID, status)
				i.monitor.Remove(jobID)
				return
			}

			// Check if status changed
			previousStatus := j.Status()
			if err := i.checkJobStatus(ctx, currentJob); err != nil {
				log.Errorfc(ctx, "job: status check failed: %v", err)
				time.Sleep(time.Second * 2)
				continue
			}

			// Adaptive polling: decrease interval on status changes, increase when stable
			if currentJob.Status() != previousStatus {
				lastStatusChange = time.Now()
				// Status changed, use shorter interval
				if currentInterval != baseInterval {
					currentInterval = baseInterval
					ticker.Reset(currentInterval)
					log.Debugf("Status changed for job %s, using faster polling interval", jobID)
				}
			} else if time.Since(lastStatusChange) > 2*time.Minute {
				// No changes for a while, slow down polling
				newInterval := time.Duration(float64(currentInterval) * 1.2)
				if newInterval > maxInterval {
					newInterval = maxInterval
				}
				if newInterval != currentInterval {
					currentInterval = newInterval
					ticker.Reset(currentInterval)
					log.Debugf("No status changes for job %s, reduced polling frequency to %v", jobID, currentInterval)
				}
			}

			// Update our reference
			*j = *currentJob
		}
	}
}

func (i *Job) checkJobStatus(ctx context.Context, j *job.Job) error {
	status, err := i.batch.GetJobStatus(ctx, j.GCPJobID())
	if err != nil {
		return fmt.Errorf("failed to get job status from batch service: %w", err)
	}

	newStatus := job.Status(status)
	statusChanged := j.Status() != newStatus

	isTerminalState := status == gateway.JobStatusCompleted || status == gateway.JobStatusFailed || status == gateway.JobStatusCancelled
	isNewTerminalState := isTerminalState && statusChanged

	if statusChanged {
		// Use optimistic locking with version check
		retries := 3
		for attempt := 1; attempt <= retries; attempt++ {
			freshJob, err := i.jobRepo.FindByID(ctx, j.ID())
			if err != nil {
				return fmt.Errorf("failed to fetch fresh job state: %w", err)
			}

			// Check if job was already updated
			if freshJob.Status() != j.Status() {
				log.Warnfc(ctx, "job: status already changed by another process for job %s (expected %s, got %s)",
					j.ID(), j.Status(), freshJob.Status())
				// Update our local reference
				*j = *freshJob
				return nil
			}

			// Check version to prevent concurrent updates
			if freshJob.Version() != j.Version() {
				log.Warnfc(ctx, "job: version mismatch for job %s (expected %d, got %d), retrying...",
					j.ID(), j.Version(), freshJob.Version())
				// Update our reference and retry
				*j = *freshJob
				if attempt < retries {
					time.Sleep(time.Duration(attempt*100) * time.Millisecond)
					continue
				}
				return fmt.Errorf("version mismatch after %d retries", retries)
			}

			// Update status with version increment
			freshJob.SetStatus(newStatus)
			freshJob.IncrementVersion()

			if err := i.updateJobWithVersion(ctx, freshJob, j.Version()); err != nil {
				if attempt < retries {
					log.Warnfc(ctx, "job: failed to update job status (attempt %d/%d): %v", attempt, retries, err)
					time.Sleep(time.Duration(attempt*100) * time.Millisecond)
					continue
				}
				return fmt.Errorf("failed to update job status after %d attempts: %w", retries, err)
			}

			// Success - update our reference
			*j = *freshJob
			break
		}
	}

	if isNewTerminalState && statusChanged {
		log.Infof("Job %s transitioning to terminal state %s, handling completion", j.ID(), newStatus)
		if err := i.handleJobCompletion(ctx, j); err != nil {
			log.Errorfc(ctx, "job: completion handling failed: %v", err)
		}
		i.monitor.Remove(j.ID().String())
	}

	return nil
}

func (i *Job) updateJobWithVersion(ctx context.Context, j *job.Job, expectedVersion int) error {
	if err := i.checkPermission(ctx, rbac.ActionAny); err != nil {
		return err
	}

	tx, err := i.transaction.Begin(ctx)
	if err != nil {
		return err
	}
	defer func() {
		if err := tx.End(ctx); err != nil {
			log.Errorfc(ctx, "transaction end failed: %v", err)
		}
	}()

	// Verify version hasn't changed during transaction
	currentJob, err := i.jobRepo.FindByID(ctx, j.ID())
	if err != nil {
		return fmt.Errorf("failed to verify job version: %w", err)
	}

	if currentJob.Version() != expectedVersion {
		return fmt.Errorf("version mismatch during update")
	}

	if err := i.jobRepo.Save(ctx, j); err != nil {
		return fmt.Errorf("failed to save job: %w", err)
	}

	tx.Commit()

	i.subscriptions.Notify(j.ID().String(), j.Status())
	log.Debugfc(ctx, "job: status updated to %s for job %s (version %d -> %d)",
		j.Status(), j.ID(), expectedVersion, j.Version())

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
	i.watchersMu.Lock()
	defer i.watchersMu.Unlock()

	jobKey := jobID.String()
	if i.activeWatchers == nil {
		i.activeWatchers = make(map[string]bool)
	}

	if _, exists := i.activeWatchers[jobKey]; exists {
		return
	}

	i.activeWatchers[jobKey] = true

	j, err := i.jobRepo.FindByID(context.Background(), jobID)
	if err != nil {
		log.Errorfc(context.Background(), "job: failed to find job for monitoring: %v", err)
		return
	}

	monitorCtx, cancel := context.WithCancel(context.Background())
	i.monitor.Register(jobKey, &monitor.Config{
		Cancel: cancel,
	})

	go i.runMonitoringLoop(monitorCtx, j)
}

func (i *Job) Unsubscribe(jobID id.JobID, ch chan job.Status) {
	i.subscriptions.Unsubscribe(jobID.String(), ch)

	if i.subscriptions.CountSubscribers(jobID.String()) == 0 {
		i.watchersMu.Lock()
		delete(i.activeWatchers, jobID.String())
		i.watchersMu.Unlock()
	}
}
