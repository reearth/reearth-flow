package interactor

import (
	"context"
	"errors"
	"fmt"
	"sync"
	"time"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/internal/rbac"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/diagnostic"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/job"
	"github.com/reearth/reearth-flow/api/pkg/job/monitor"
	"github.com/reearth/reearth-flow/api/pkg/notification"
	"github.com/reearth/reearth-flow/api/pkg/subscription"
	"github.com/reearth/reearthx/log"
	"github.com/reearth/reearthx/rerror"
	"github.com/reearth/reearthx/usecasex"
)

var _ interfaces.Job = &Job{}

type Job struct {
	jobRepo             repo.Job
	nodeDiagnosticsRepo repo.NodeDiagnostics
	transaction         usecasex.Transactor
	file                gateway.File
	batch               gateway.Batch
	cloudRunWorker      gateway.CloudRunWorker
	redis               gateway.LiveStore
	usage               gateway.UsageCounter
	notifier            notification.Notifier
	permissionChecker   gateway.PermissionChecker
	monitor             *monitor.Monitor
	subscriptions       *subscription.JobManager
	activeWatchers      map[string]bool
	jobLocks            map[string]*sync.Mutex
	now                 func() time.Time
	sleep               func(context.Context, time.Duration) error
	jobLocksMu          sync.RWMutex
	watchersMu          sync.Mutex
	// monitorInterval is the loop's poll interval; zero means defaultMonitorInterval.
	monitorInterval time.Duration
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
		jobRepo:             r.Job,
		nodeDiagnosticsRepo: r.NodeDiagnostics,
		transaction:         r.Transaction,
		file:                gr.File,
		batch:               gr.Batch,
		cloudRunWorker:      gr.CloudRunWorker,
		redis:               gr.LiveStore,
		usage:               gr.UsageCounter,
		monitor:             monitor.NewMonitor(),
		subscriptions:       subscription.NewJobManager(),
		notifier:            notification.NewHTTPNotifier(),
		permissionChecker:   permissionChecker,
		activeWatchers:      make(map[string]bool),
		jobLocks:            make(map[string]*sync.Mutex),
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

func (i *Job) checkPermission(ctx context.Context, action string, workspaceID ...accountsid.WorkspaceID) error {
	return checkPermission(ctx, i.permissionChecker, rbac.ResourceJob, action, workspaceID...)
}

func (i *Job) Cancel(ctx context.Context, jobID id.JobID) (*job.Job, error) {
	target, err := i.jobRepo.FindByID(ctx, jobID)
	if err != nil {
		return nil, err
	}
	if target == nil {
		return nil, rerror.ErrNotFound
	}
	if err := i.checkPermission(ctx, rbac.ActionAny, target.Workspace()); err != nil {
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
		return nil, fmt.Errorf("%w: current status is %s", interfaces.ErrJobNotCancellable, j.Status())
	}

	if isCloudRunDebug(j) && i.cloudRunWorker != nil {
		if err := i.cloudRunWorker.CancelJob(ctx, j.ID()); err != nil {
			return nil, err
		}
	} else {
		if err := i.batch.CancelJob(ctx, j.GCPJobID()); err != nil {
			return nil, err
		}
	}

	j.SetStatus(job.StatusCancelled)
	// Status() follows the Batch status, so set it too or the job still reads as running.
	if j.BatchStatus() != nil {
		j.SetBatchStatus(job.StatusCancelled)
	}
	now := time.Now()
	j.SetCompletedAt(&now)

	got, saved, err := i.finalize(ctx, j, nil)
	if err != nil {
		return nil, err
	}
	if !saved {
		// It finished, possibly on another instance, while it was being cancelled.
		return nil, fmt.Errorf("%w: current status is %s", interfaces.ErrJobNotCancellable, got.Status())
	}
	return got, nil
}

const cloudRunInfraFailureGrace = 30 * time.Second

// RunCloudRunWorker executes a debug job on the Cloud Run worker; the worker
// publishes its result to Redis and the standard monitoring loop finalizes the
// job. Only an infra failure (no result will ever arrive) is finalized here.
// notStarted, when set, is called once the worker is known not to have started
// the run, so a caller can give back what the run was counted as.
//
// While every worker instance is busy the job waits for one, up to
// runDispatchPolicy.MaxWait.
func (i *Job) RunCloudRunWorker(j *job.Job, p gateway.RunJobParam, notStarted func()) {
	go func() {
		ctx := context.Background()
		defer func() {
			if r := recover(); r != nil {
				log.Errorfc(ctx, "job: cloud run worker %s panicked: %v", j.ID(), r)
			}
		}()

		status, err := i.dispatchWhenFree(ctx, j.ID(), runDispatchPolicy, func(ctx context.Context) (gateway.JobStatus, error) {
			return i.runCloudRunAttempt(ctx, p)
		})
		if errors.Is(err, errFinishedWhileWaiting) {
			if notStarted != nil {
				notStarted()
			}
			return
		}
		if err != nil {
			log.Errorfc(ctx, "job: cloud run worker %s run error: %v", j.ID(), err)
			if notStarted != nil && errors.Is(err, gateway.ErrRunNotStarted) {
				notStarted()
			}
		}
		if errors.Is(err, errPoolStayedFull) {
			i.failCloudRunJobNow(ctx, j.ID())
			return
		}
		if err != nil || status == gateway.JobStatusFailed {
			i.failCloudRunJob(ctx, j.ID())
		}
	}()
}

func (i *Job) runCloudRunAttempt(ctx context.Context, p gateway.RunJobParam) (gateway.JobStatus, error) {
	runCtx := ctx
	if p.MaxRunDuration > 0 {
		var cancel context.CancelFunc
		runCtx, cancel = context.WithTimeout(ctx, p.MaxRunDuration)
		defer cancel()
	}
	status, err := i.cloudRunWorker.RunJob(runCtx, p)
	if errors.Is(runCtx.Err(), context.DeadlineExceeded) {
		i.cancelOverrunWorker(ctx, p)
	}
	return status, err
}

func (i *Job) cancelOverrunWorker(ctx context.Context, p gateway.RunJobParam) {
	log.Infofc(ctx, "job: cloud run worker %s ran past its %s limit, cancelling it", p.JobID, p.MaxRunDuration)
	if err := i.cloudRunWorker.CancelJob(ctx, p.JobID); err != nil {
		log.Errorfc(ctx, "job: could not cancel cloud run worker %s after its limit: %v", p.JobID, err)
	}
}

// PreviewSchemaCloudRunWorker executes a preview-schema probe on the Cloud Run
// worker via its dedicated probe-schema route. Mirrors RunCloudRunWorker: the
// worker publishes its result and the standard monitoring loop finalizes the
// job; only an infra failure is finalized here.
func (i *Job) PreviewSchemaCloudRunWorker(j *job.Job, p gateway.ProbeSchemaParam) {
	go func() {
		ctx := context.Background()
		defer func() {
			if r := recover(); r != nil {
				log.Errorfc(ctx, "job: preview-schema cloud run worker %s panicked: %v", j.ID(), r)
			}
		}()

		status, err := i.dispatchWhenFree(ctx, j.ID(), probeDispatchPolicy, func(ctx context.Context) (gateway.JobStatus, error) {
			return i.cloudRunWorker.PreviewSchema(ctx, p)
		})
		if errors.Is(err, errFinishedWhileWaiting) {
			return
		}
		if err != nil {
			log.Errorfc(ctx, "job: preview-schema cloud run worker %s run error: %v", j.ID(), err)
		}
		if errors.Is(err, errPoolStayedFull) {
			i.failCloudRunJobNow(ctx, j.ID())
			return
		}
		if err != nil || status == gateway.JobStatusFailed {
			i.failCloudRunJob(ctx, j.ID())
		}
	}()
}

// failCloudRunJob marks a debug job failed when the worker call fails. A
// client-side error may still yield a result, so it waits a grace period and
// bails if the loop already finalized or a worker event arrived; it sets the
// legacy status (not batchStatus) so a late success isn't forced to FAILED.
func (i *Job) failCloudRunJob(ctx context.Context, jobID id.JobID) {
	select {
	case <-ctx.Done():
		return
	case <-time.After(cloudRunInfraFailureGrace):
	}
	i.failCloudRunJobNow(ctx, jobID)
}

func (i *Job) failCloudRunJobNow(ctx context.Context, jobID id.JobID) {
	lock := i.getJobLock(jobID.String())
	lock.Lock()
	defer lock.Unlock()

	latest, err := i.reconcileLocked(ctx, jobID)
	if err != nil {
		log.Errorfc(ctx, "job: failed to reconcile cloud run job %s: %v", jobID, err)
		return
	}
	if latest.Status().IsTerminal() {
		return
	}
	// A worker event the reconcile did not act on still means a result arrived.
	if ev, _ := i.redis.GetJobCompleteEvent(ctx, jobID); ev != nil {
		return
	}

	latest.SetStatus(job.StatusFailed)
	if _, _, err := i.finalize(ctx, latest, nil); err != nil {
		log.Errorfc(ctx, "job: failed to save cloud run job %s: %v", jobID, err)
	}
}

func (i *Job) FindByID(ctx context.Context, id id.JobID) (*job.Job, error) {
	j, err := i.jobRepo.FindByID(ctx, id)
	if err != nil {
		return nil, err
	}
	if j == nil {
		return nil, rerror.ErrNotFound
	}
	if err := i.checkPermission(ctx, rbac.ActionRead, j.Workspace()); err != nil {
		return nil, err
	}

	return j, nil
}

func (i *Job) Fetch(ctx context.Context, ids []id.JobID) ([]*job.Job, error) {
	got, err := i.jobRepo.FindByIDs(ctx, ids)
	if err != nil {
		return nil, err
	}
	jobs := alignToRequestedIDs(ids, got, nil, (*job.Job).ID)

	if err := authorizeFetchByWorkspace(ctx, i.checkPermission, rbac.ActionRead, jobs, nil, (*job.Job).Workspace); err != nil {
		return nil, err
	}

	return jobs, nil
}

func (i *Job) FindByWorkspace(
	ctx context.Context,
	wsID accountsid.WorkspaceID,
	p *interfaces.PaginationParam,
	keyword *string,
) ([]*job.Job, *interfaces.PageBasedInfo, error) {
	if err := i.checkPermission(ctx, rbac.ActionRead, wsID); err != nil {
		return nil, nil, err
	}

	return i.jobRepo.FindByWorkspace(ctx, wsID, p, keyword)
}

func (i *Job) GetStatus(ctx context.Context, jobID id.JobID) (job.Status, error) {
	j, err := i.jobRepo.FindByID(ctx, jobID)
	if err != nil {
		return "", err
	}
	if j == nil {
		return "", rerror.ErrNotFound
	}
	if err := i.checkPermission(ctx, rbac.ActionRead, j.Workspace()); err != nil {
		return "", err
	}
	return j.Status(), nil
}

// Refresh reconciles an unfinished job; a finished one is returned as is.
func (i *Job) Refresh(ctx context.Context, j *job.Job) (*job.Job, error) {
	if j.Status().IsTerminal() {
		return j, nil
	}
	return i.Reconcile(ctx, j.ID())
}

func (i *Job) StartMonitoring(ctx context.Context, j *job.Job) error {
	log.Debugfc(ctx, "job: starting monitoring for jobID=%s workspace=%s", j.ID(), j.Workspace())

	i.watchersMu.Lock()
	defer i.watchersMu.Unlock()

	jobKey := j.ID().String()
	if i.activeWatchers == nil {
		i.activeWatchers = make(map[string]bool)
	}

	if _, exists := i.activeWatchers[jobKey]; exists {
		log.Debugfc(ctx, "job: monitoring already active for jobID=%s", jobKey)
		return nil
	}

	i.activeWatchers[jobKey] = true

	monitorCtx, cancel := context.WithCancel(context.WithoutCancel(ctx))

	i.monitor.Register(jobKey, &monitor.Config{
		Cancel: cancel,
	})

	go i.runMonitoringLoop(monitorCtx, j)

	return nil
}

const defaultMonitorInterval = 5 * time.Second

func (i *Job) runMonitoringLoop(ctx context.Context, j *job.Job) {
	interval := i.monitorInterval
	if interval <= 0 {
		interval = defaultMonitorInterval
	}
	ticker := time.NewTicker(interval)
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

			// Reconcile notifies subscribers of a finished job, even one finished elsewhere.
			currentJob, err := i.Reconcile(ctx, j.ID())
			if err != nil {
				log.Errorfc(ctx, "job: status check failed for job ID %s: %v", jobID, err)
				continue
			}
			if status := currentJob.Status(); status.IsTerminal() {
				log.Infof("Job ID %s in terminal state %s, stopping monitoring", jobID, status)
				return
			}
		}
	}
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

	// A preview-schema job's only output is its SchemaReport; surface it via
	// outputURLs (no dedicated Job field).
	if j.Mode() == job.ModePreviewSchema {
		if schemaURL := i.file.GetJobPreviewSchemaURL(jobID); schemaURL != "" {
			j.SetOutputURLs([]string{schemaURL})
		}
	}

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
	j, err := i.jobRepo.FindByID(ctx, jobID)
	if err != nil {
		return nil, err
	}
	if j == nil {
		return nil, rerror.ErrNotFound
	}
	if err := i.checkPermission(ctx, rbac.ActionRead, j.Workspace()); err != nil {
		return nil, err
	}

	ch := i.subscriptions.Subscribe(jobID.String())

	go i.startMonitoringIfNeeded(context.WithoutCancel(ctx), jobID)

	return ch, nil
}

// startMonitoringIfNeeded reconciles the job, sends its status, and watches it if unfinished.
func (i *Job) startMonitoringIfNeeded(ctx context.Context, jobID id.JobID) {
	j, err := i.Reconcile(ctx, jobID)
	if err != nil {
		log.Warnfc(ctx, "job: failed to reconcile job %s for a subscription: %v", jobID, err)
		if j, err = i.jobRepo.FindByID(ctx, jobID); err != nil {
			log.Errorfc(ctx, "job: failed to find job %s for monitoring: %v", jobID, err)
			return
		}
	}
	if j.Status().IsTerminal() {
		// Reconcile has sent the final status.
		return
	}
	i.subscriptions.Notify(jobID.String(), j.Status())

	if err := i.StartMonitoring(ctx, j); err != nil {
		log.Errorfc(ctx, "job: failed to start monitoring: %v", err)
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

func (i *Job) persistTerminalDiagnostics(ctx context.Context, jobID id.JobID, event *gateway.JobCompleteEvent) error {
	if event == nil {
		return nil
	}
	if len(event.FailedNodes) == 0 && len(event.AggregatedDiagnostics) == 0 && event.DroppedEventCount == nil {
		return nil
	}
	if i.nodeDiagnosticsRepo == nil {
		return nil
	}

	failedNodes, err := wireDiagnosticsToDomain(jobID, event.Timestamp, event.FailedNodes)
	if err != nil {
		return fmt.Errorf("failed to convert failed nodes: %w", err)
	}

	aggregated, err := wireDiagnosticsToDomain(jobID, event.Timestamp, event.AggregatedDiagnostics)
	if err != nil {
		return fmt.Errorf("failed to convert aggregated diagnostics: %w", err)
	}

	return i.nodeDiagnosticsRepo.SaveTerminalDiagnostics(ctx, jobID, event.WorkflowID, event.Timestamp, failedNodes, aggregated, event.DroppedEventCount)
}

func wireDiagnosticsToDomain(jobID id.JobID, timestamp time.Time, wire []gateway.WireDiagnostic) ([]*diagnostic.Diagnostic, error) {
	if len(wire) == 0 {
		return nil, nil
	}
	result := make([]*diagnostic.Diagnostic, 0, len(wire))
	for _, w := range wire {
		d, err := w.ToDomain(jobID, timestamp)
		if err != nil {
			return nil, err
		}
		result = append(result, d)
	}
	return result, nil
}
