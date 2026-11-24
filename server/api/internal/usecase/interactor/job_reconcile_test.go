package interactor

import (
	"context"
	"sync"
	"testing"
	"time"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/memory"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/job"
	"github.com/reearth/reearth-flow/api/pkg/job/monitor"
	"github.com/reearth/reearth-flow/api/pkg/notification"
	"github.com/reearth/reearth-flow/api/pkg/subscription"
	"github.com/reearth/reearthx/usecasex"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// reconcileLiveStore holds one completion event and drops it when deleted.
type reconcileLiveStore struct {
	gateway.LiveStore
	mu    sync.Mutex
	event *gateway.JobCompleteEvent
}

func (s *reconcileLiveStore) GetJobCompleteEvent(context.Context, id.JobID) (*gateway.JobCompleteEvent, error) {
	s.mu.Lock()
	defer s.mu.Unlock()
	return s.event, nil
}

func (s *reconcileLiveStore) DeleteJobCompleteEvent(context.Context, id.JobID) error {
	s.mu.Lock()
	defer s.mu.Unlock()
	s.event = nil
	return nil
}

func (s *reconcileLiveStore) hasEvent() bool {
	s.mu.Lock()
	defer s.mu.Unlock()
	return s.event != nil
}

// reconcileBatch reports one Batch status for every job.
type reconcileBatch struct {
	gateway.Batch
	mu     sync.Mutex
	status gateway.JobStatus
}

func (b *reconcileBatch) GetJobStatus(context.Context, string) (gateway.JobStatus, error) {
	b.mu.Lock()
	defer b.mu.Unlock()
	return b.status, nil
}

func (b *reconcileBatch) CancelJob(context.Context, string) error { return nil }

func (b *reconcileBatch) set(s gateway.JobStatus) {
	b.mu.Lock()
	defer b.mu.Unlock()
	b.status = s
}

type recordingNotifier struct {
	mu   sync.Mutex
	sent []notification.Payload
	urls []string
}

func (n *recordingNotifier) Send(url string, p notification.Payload) error {
	n.mu.Lock()
	defer n.mu.Unlock()
	n.urls = append(n.urls, url)
	n.sent = append(n.sent, p)
	return nil
}

func (n *recordingNotifier) count() int {
	n.mu.Lock()
	defer n.mu.Unlock()
	return len(n.sent)
}

// newReconcileJob is one server instance sharing the repo and live store.
func newReconcileJob(jobs repo.Job, live gateway.LiveStore, batch gateway.Batch, n notification.Notifier) *Job {
	return &Job{
		jobRepo:             jobs,
		nodeDiagnosticsRepo: &mockDiagnosticsRepo{},
		transaction:         usecasex.NewTransactor(&usecasex.NopTransaction{}, 0),
		file:                &mockCheckStatusFile{},
		batch:               batch,
		redis:               live,
		monitor:             monitor.NewMonitor(),
		subscriptions:       subscription.NewJobManager(),
		notifier:            n,
		permissionChecker:   alwaysAllowPermissionChecker(),
		activeWatchers:      make(map[string]bool),
		jobLocks:            make(map[string]*sync.Mutex),
	}
}

// runningCloudRunJob is a debug run on Cloud Run: it has no Batch job.
func runningCloudRunJob(t *testing.T, jobs repo.Job) *job.Job {
	t.Helper()
	debug := true
	j := job.New().NewID().Workspace(accountsid.NewWorkspaceID()).Debug(&debug).
		Status(job.StatusRunning).StartedAt(time.Now().Add(-25 * time.Hour)).MustBuild()
	require.NoError(t, jobs.Save(context.Background(), j))
	return j
}

// runningTriggerJob is an API trigger run on Batch, with a webhook.
func runningTriggerJob(t *testing.T, jobs repo.Job, notificationURL string) *job.Job {
	t.Helper()
	did := id.NewDeploymentID()
	running := job.StatusRunning
	j := job.New().NewID().Workspace(accountsid.NewWorkspaceID()).Deployment(&did).GCPJobID("gcp-1").
		Status(job.StatusRunning).BatchStatus(&running).StartedAt(time.Now().Add(-25 * time.Hour)).
		NotificationURL(&notificationURL).MustBuild()
	require.NoError(t, jobs.Save(context.Background(), j))
	return j
}

func TestJob_Reconcile_FinalizesWithTheEventTimestamp(t *testing.T) {
	t.Parallel()
	ctx := context.Background()
	jobs := memory.NewJob()
	url := "https://example.com/hook"
	j := runningTriggerJob(t, jobs, url)
	finishedAt := time.Now().Add(-24 * time.Hour).UTC().Truncate(time.Second)
	live := &reconcileLiveStore{event: &gateway.JobCompleteEvent{Timestamp: finishedAt, JobID: j.ID().String(), Result: "success"}}
	n := &recordingNotifier{}
	i := newReconcileJob(jobs, live, &reconcileBatch{status: gateway.JobStatusCompleted}, n)

	got, err := i.Reconcile(ctx, j.ID())

	require.NoError(t, err)
	assert.Equal(t, job.StatusCompleted, got.Status())
	require.NotNil(t, got.CompletedAt())
	assert.True(t, finishedAt.Equal(*got.CompletedAt()), "completedAt %v is the worker's finish time %v", got.CompletedAt(), finishedAt)
	assert.False(t, live.hasEvent(), "the handled event is deleted")
	require.Equal(t, 1, n.count())
	assert.Equal(t, url, n.urls[0])
	assert.Equal(t, "succeeded", n.sent[0].Status)
	assert.NotNil(t, got.NotifiedAt())
}

func TestJob_Reconcile_IsIdempotent(t *testing.T) {
	t.Parallel()
	ctx := context.Background()
	jobs := memory.NewJob()
	url := "https://example.com/hook"
	j := runningTriggerJob(t, jobs, url)
	finishedAt := time.Now().Add(-time.Hour).UTC().Truncate(time.Second)
	event := &gateway.JobCompleteEvent{Timestamp: finishedAt, JobID: j.ID().String(), Result: "failed"}
	live := &reconcileLiveStore{event: event}
	n := &recordingNotifier{}
	batch := &reconcileBatch{status: gateway.JobStatusFailed}
	first := newReconcileJob(jobs, live, batch, n)
	other := newReconcileJob(jobs, live, batch, n)

	_, err := first.Reconcile(ctx, j.ID())
	require.NoError(t, err)

	// Again on the same instance.
	got, err := first.Reconcile(ctx, j.ID())
	require.NoError(t, err)
	assert.Equal(t, job.StatusFailed, got.Status())
	assert.True(t, finishedAt.Equal(*got.CompletedAt()))

	// On another instance, even with the event still there.
	live.mu.Lock()
	live.event = event
	live.mu.Unlock()
	got, err = other.Reconcile(ctx, j.ID())
	require.NoError(t, err)
	assert.Equal(t, job.StatusFailed, got.Status())
	assert.True(t, finishedAt.Equal(*got.CompletedAt()))

	assert.Equal(t, 1, n.count(), "the webhook is sent once")
}

func TestJob_Refresh_FinalizesACloudRunJob(t *testing.T) {
	t.Parallel()
	ctx := context.Background()
	jobs := memory.NewJob()
	j := runningCloudRunJob(t, jobs)
	finishedAt := time.Now().Add(-2 * time.Hour).UTC().Truncate(time.Second)
	live := &reconcileLiveStore{event: &gateway.JobCompleteEvent{Timestamp: finishedAt, JobID: j.ID().String(), Result: "success"}}
	i := newReconcileJob(jobs, live, nil, &recordingNotifier{})

	got, err := i.Refresh(ctx, j)

	require.NoError(t, err)
	assert.Equal(t, job.StatusCompleted, got.Status())
	assert.True(t, finishedAt.Equal(*got.CompletedAt()))
	stored, err := jobs.FindByID(ctx, j.ID())
	require.NoError(t, err)
	assert.Equal(t, job.StatusCompleted, stored.Status())
}

func TestJob_Refresh_LeavesARunningCloudRunJob(t *testing.T) {
	t.Parallel()
	jobs := memory.NewJob()
	j := runningCloudRunJob(t, jobs)
	i := newReconcileJob(jobs, &reconcileLiveStore{}, nil, &recordingNotifier{})

	got, err := i.Refresh(context.Background(), j)

	require.NoError(t, err)
	assert.Equal(t, job.StatusRunning, got.Status())
	assert.Nil(t, got.CompletedAt())
}

// An event that beats Batch is kept so the job still gets the worker's finish time.
func TestJob_Reconcile_KeepsTheEventUntilBatchFinishes(t *testing.T) {
	t.Parallel()
	ctx := context.Background()
	jobs := memory.NewJob()
	j := job.New().NewID().Workspace(accountsid.NewWorkspaceID()).GCPJobID("gcp-1").
		Status(job.StatusPending).StartedAt(time.Now().Add(-time.Hour)).MustBuild()
	require.NoError(t, jobs.Save(ctx, j))
	finishedAt := time.Now().Add(-30 * time.Minute).UTC().Truncate(time.Second)
	live := &reconcileLiveStore{event: &gateway.JobCompleteEvent{Timestamp: finishedAt, JobID: j.ID().String(), Result: "success"}}
	batch := &reconcileBatch{status: gateway.JobStatusRunning}
	i := newReconcileJob(jobs, live, batch, &recordingNotifier{})

	got, err := i.Reconcile(ctx, j.ID())
	require.NoError(t, err)
	assert.Equal(t, job.StatusRunning, got.Status())
	assert.True(t, live.hasEvent())

	batch.set(gateway.JobStatusCompleted)
	got, err = i.Reconcile(ctx, j.ID())
	require.NoError(t, err)
	assert.Equal(t, job.StatusCompleted, got.Status())
	assert.True(t, finishedAt.Equal(*got.CompletedAt()))
	assert.False(t, live.hasEvent())
}

func TestJob_runMonitoringLoop_NotifiesAJobFinishedElsewhere(t *testing.T) {
	t.Parallel()
	ctx := context.Background()
	jobs := memory.NewJob()
	j := runningCloudRunJob(t, jobs)
	live := &reconcileLiveStore{}
	watcher := newReconcileJob(jobs, live, nil, &recordingNotifier{})
	watcher.monitorInterval = 10 * time.Millisecond
	ch := watcher.subscriptions.Subscribe(j.ID().String())

	// Another instance finishes the job.
	live.mu.Lock()
	live.event = &gateway.JobCompleteEvent{Timestamp: time.Now(), JobID: j.ID().String(), Result: "success"}
	live.mu.Unlock()
	_, err := newReconcileJob(jobs, live, nil, &recordingNotifier{}).Reconcile(ctx, j.ID())
	require.NoError(t, err)

	done := make(chan struct{})
	go func() {
		watcher.runMonitoringLoop(ctx, j)
		close(done)
	}()

	select {
	case s := <-ch:
		assert.Equal(t, job.StatusCompleted, s)
	case <-time.After(5 * time.Second):
		t.Fatal("subscribers were not told the job finished")
	}
	select {
	case <-done:
	case <-time.After(5 * time.Second):
		t.Fatal("the loop did not stop")
	}
}

// Cancelling must set the Batch status too, since Status() follows it.
func TestJob_Cancel_ABatchJobReadsCancelled(t *testing.T) {
	t.Parallel()
	ctx := context.Background()
	jobs := memory.NewJob()
	running := job.StatusRunning
	j := job.New().NewID().Workspace(accountsid.NewWorkspaceID()).GCPJobID("gcp-1").
		Status(job.StatusRunning).BatchStatus(&running).StartedAt(time.Now()).MustBuild()
	require.NoError(t, jobs.Save(ctx, j))
	i := newReconcileJob(jobs, &reconcileLiveStore{}, &reconcileBatch{status: gateway.JobStatusRunning}, &recordingNotifier{})

	got, err := i.Cancel(ctx, j.ID())

	require.NoError(t, err)
	assert.Equal(t, job.StatusCancelled, got.Status())
	assert.NotNil(t, got.CompletedAt())
}
