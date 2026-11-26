package interactor

import (
	"context"
	"errors"
	"fmt"
	"sync"
	"sync/atomic"
	"testing"
	"time"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/job"
	pkglog "github.com/reearth/reearth-flow/api/pkg/log"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

type dispatchJobRepo struct {
	*mockCheckStatusJobRepo
	stored *job.Job
	saved  []job.Status
	mu     sync.Mutex
}

func newDispatchJobRepo(j *job.Job) *dispatchJobRepo {
	return &dispatchJobRepo{mockCheckStatusJobRepo: &mockCheckStatusJobRepo{}, stored: j}
}

func (r *dispatchJobRepo) FindByID(context.Context, id.JobID) (*job.Job, error) {
	r.mu.Lock()
	defer r.mu.Unlock()
	cp := *r.stored
	return &cp, nil
}

func (r *dispatchJobRepo) Save(_ context.Context, j *job.Job) error {
	r.mu.Lock()
	defer r.mu.Unlock()
	cp := *j
	r.stored = &cp
	r.saved = append(r.saved, j.Status())
	return nil
}

func (r *dispatchJobRepo) status() job.Status {
	r.mu.Lock()
	defer r.mu.Unlock()
	return r.stored.Status()
}

func (r *dispatchJobRepo) savedStatuses() []job.Status {
	r.mu.Lock()
	defer r.mu.Unlock()
	return append([]job.Status(nil), r.saved...)
}

type fakeLeases struct {
	leases map[string]map[string]time.Time
	mu     sync.Mutex
}

func newFakeLeases() *fakeLeases { return &fakeLeases{leases: map[string]map[string]time.Time{}} }

func (f *fakeLeases) AcquireLease(_ context.Context, key, member string, now, expireAt time.Time) (int64, error) {
	f.mu.Lock()
	defer f.mu.Unlock()
	held := f.leases[key]
	if held == nil {
		held = map[string]time.Time{}
		f.leases[key] = held
	}
	for m, until := range held {
		if !until.After(now) {
			delete(held, m)
		}
	}
	held[member] = expireAt
	return int64(len(held)), nil
}

func (f *fakeLeases) ReleaseLease(_ context.Context, key, member string) error {
	f.mu.Lock()
	defer f.mu.Unlock()
	delete(f.leases[key], member)
	return nil
}

func (f *fakeLeases) Increment(context.Context, string, time.Time) (int64, error) {
	panic("unused")
}
func (f *fakeLeases) Decrement(context.Context, string, time.Time) error { panic("unused") }

type dispatchClock struct {
	t      time.Time
	waited []time.Duration
	mu     sync.Mutex
}

func (c *dispatchClock) now() time.Time {
	c.mu.Lock()
	defer c.mu.Unlock()
	return c.t
}

func (c *dispatchClock) sleep(_ context.Context, d time.Duration) error {
	c.mu.Lock()
	defer c.mu.Unlock()
	c.t = c.t.Add(d)
	c.waited = append(c.waited, d)
	return nil
}

func (c *dispatchClock) waits() int {
	c.mu.Lock()
	defer c.mu.Unlock()
	return len(c.waited)
}

type fakeRunWorker struct {
	mockCloudRunWorker
	answers []error
	calls   atomic.Int32
	onCall  func(ctx context.Context, n int)
}

func (w *fakeRunWorker) answer() (gateway.JobStatus, error) {
	n := int(w.calls.Add(1))
	err := w.answers[min(n, len(w.answers))-1]
	if err != nil {
		return gateway.JobStatusFailed, err
	}
	return gateway.JobStatusCompleted, nil
}

func (w *fakeRunWorker) RunJob(ctx context.Context, _ gateway.RunJobParam) (gateway.JobStatus, error) {
	if w.onCall != nil {
		w.onCall(ctx, int(w.calls.Load())+1)
	}
	return w.answer()
}

func (w *fakeRunWorker) PreviewSchema(context.Context, gateway.ProbeSchemaParam) (gateway.JobStatus, error) {
	return w.answer()
}

var busy = fmt.Errorf("cloudrunworker: run refused (http 429): %w: %w", gateway.ErrWorkerBusy, gateway.ErrRunNotStarted)

func queuedDebugJob(t *testing.T, startedAt time.Time) *job.Job {
	t.Helper()
	debug := true
	j, err := job.New().NewID().Workspace(accountsid.NewWorkspaceID()).
		Status(job.StatusPending).StartedAt(startedAt).Debug(&debug).Build()
	require.NoError(t, err)
	return j
}

func newDispatchJob(repo *dispatchJobRepo, worker gateway.CloudRunWorker, leases *fakeLeases, clock *dispatchClock) *Job {
	i := newCheckStatusJob(nil, &mockDiagnosticsRepo{}, &mockCheckStatusRedis{})
	i.jobRepo = repo
	i.cloudRunWorker = worker
	i.usage = leases
	i.now = clock.now
	i.sleep = clock.sleep
	return i
}

func TestDispatchPolicy_Backoff(t *testing.T) {
	t.Parallel()

	p := dispatchPolicy{BaseBackoff: time.Second, MaxBackoff: 12 * time.Second}

	t.Run("never shorter than the base or longer than the cap", func(t *testing.T) {
		t.Parallel()

		for n := 1; n <= 40; n++ {
			assert.Equal(t, p.BaseBackoff, p.backoff(n, 0), "attempt %d", n)
			assert.LessOrEqual(t, p.backoff(n, 0.999999), p.MaxBackoff, "attempt %d", n)
		}
	})

	t.Run("the ceiling doubles until it reaches the cap", func(t *testing.T) {
		t.Parallel()

		ceiling := func(n int) time.Duration { return p.backoff(n, 1) }
		assert.Equal(t, 2*time.Second, ceiling(1))
		assert.Equal(t, 4*time.Second, ceiling(2))
		assert.Equal(t, 8*time.Second, ceiling(3))
		assert.Equal(t, 12*time.Second, ceiling(4))
		assert.Equal(t, 12*time.Second, ceiling(10))
	})

	t.Run("a waiting run is out of Cloud Run's queue for 2 to 10 seconds", func(t *testing.T) {
		t.Parallel()

		for _, p := range []dispatchPolicy{runDispatchPolicy, probeDispatchPolicy} {
			for n := 1; n <= 40; n++ {
				assert.Equal(t, 2*time.Second, p.backoff(n, 0), "attempt %d", n)
				assert.LessOrEqual(t, p.backoff(n, 1), 10*time.Second, "attempt %d", n)
			}
		}
	})
}

func TestJob_RunCloudRunWorker_WaitsForAFreeInstance(t *testing.T) {
	t.Parallel()

	clock := &dispatchClock{t: time.Unix(1_000_000, 0)}
	queued := queuedDebugJob(t, clock.now())
	repo := newDispatchJobRepo(queued)
	worker := &fakeRunWorker{answers: []error{busy, busy, busy, nil}}
	i := newDispatchJob(repo, worker, newFakeLeases(), clock)
	var refunds atomic.Int32

	i.RunCloudRunWorker(queued, gateway.RunJobParam{JobID: queued.ID()}, func() { refunds.Add(1) })

	require.Eventually(t, func() bool { return worker.calls.Load() == 4 }, 5*time.Second, time.Millisecond)
	assert.Never(t, func() bool { return worker.calls.Load() > 4 || refunds.Load() > 0 }, 200*time.Millisecond, 10*time.Millisecond)
	assert.NotContains(t, repo.savedStatuses(), job.StatusFailed)
	assert.Equal(t, 3, clock.waits(), "one wait after each refusal")
}

func TestJob_RunCloudRunWorker_FailsWhenThePoolStaysFull(t *testing.T) {
	t.Parallel()

	clock := &dispatchClock{t: time.Unix(1_000_000, 0)}
	queued := queuedDebugJob(t, clock.now())
	repo := newDispatchJobRepo(queued)
	worker := &fakeRunWorker{answers: []error{busy}}
	i := newDispatchJob(repo, worker, newFakeLeases(), clock)
	var refunds atomic.Int32

	i.RunCloudRunWorker(queued, gateway.RunJobParam{JobID: queued.ID()}, func() { refunds.Add(1) })

	require.Eventually(t, func() bool { return repo.status() == job.StatusFailed }, 5*time.Second, time.Millisecond)
	assert.Never(t, func() bool { return refunds.Load() > 1 }, 200*time.Millisecond, 10*time.Millisecond)
	assert.EqualValues(t, 1, refunds.Load(), "the run never started, so its count is given back once")
	assert.GreaterOrEqual(t, clock.now().Sub(queued.StartedAt()), runDispatchPolicy.MaxWait)
}

func TestJob_RunCloudRunWorker_ACancelWhileWaitingSendsNoMoreRequests(t *testing.T) {
	t.Parallel()

	clock := &dispatchClock{t: time.Unix(1_000_000, 0)}
	queued := queuedDebugJob(t, clock.now())
	repo := newDispatchJobRepo(queued)
	worker := &fakeRunWorker{answers: []error{busy, nil}}
	i := newDispatchJob(repo, worker, newFakeLeases(), clock)
	var refunds atomic.Int32
	i.sleep = func(ctx context.Context, d time.Duration) error {
		_, err := i.Cancel(ctx, queued.ID())
		assert.NoError(t, err)
		return clock.sleep(ctx, d)
	}

	i.RunCloudRunWorker(queued, gateway.RunJobParam{JobID: queued.ID()}, func() { refunds.Add(1) })

	require.Eventually(t, func() bool { return repo.status() == job.StatusCancelled }, 5*time.Second, time.Millisecond)
	assert.Never(t, func() bool { return worker.calls.Load() > 1 }, 200*time.Millisecond, 10*time.Millisecond)
	assert.Equal(t, job.StatusCancelled, repo.status())
	assert.NotContains(t, repo.savedStatuses(), job.StatusFailed)
}

func TestJob_RunCloudRunWorker_EachAttemptGetsTheWholeRunLimit(t *testing.T) {
	t.Parallel()

	clock := &dispatchClock{t: time.Unix(1_000_000, 0)}
	queued := queuedDebugJob(t, clock.now())
	repo := newDispatchJobRepo(queued)
	const limit = 100 * time.Millisecond
	var stale atomic.Int32
	worker := &fakeRunWorker{answers: []error{busy, busy, nil}}
	worker.onCall = func(ctx context.Context, n int) {
		deadline, ok := ctx.Deadline()
		if !ok || time.Until(deadline) < limit/2 {
			stale.Add(1)
		}
		time.Sleep(limit * 2 / 3)
	}
	i := newDispatchJob(repo, worker, newFakeLeases(), clock)

	i.RunCloudRunWorker(queued, gateway.RunJobParam{JobID: queued.ID(), MaxRunDuration: limit}, nil)

	require.Eventually(t, func() bool { return worker.calls.Load() == 3 }, 5*time.Second, time.Millisecond)
	assert.Zero(t, stale.Load(), "an attempt started with a limit already part spent")
}

func TestJob_PreviewSchemaCloudRunWorker_WaitsForAFreeInstance(t *testing.T) {
	t.Parallel()

	clock := &dispatchClock{t: time.Unix(1_000_000, 0)}
	queued := queuedDebugJob(t, clock.now())
	repo := newDispatchJobRepo(queued)
	worker := &fakeRunWorker{answers: []error{busy, busy, nil}}
	i := newDispatchJob(repo, worker, newFakeLeases(), clock)

	i.PreviewSchemaCloudRunWorker(queued, gateway.ProbeSchemaParam{JobID: queued.ID()})

	require.Eventually(t, func() bool { return worker.calls.Load() == 3 }, 5*time.Second, time.Millisecond)
	assert.Never(t, func() bool { return worker.calls.Load() > 3 }, 200*time.Millisecond, 10*time.Millisecond)
	assert.Empty(t, repo.savedStatuses(), "a probe stays queued until its worker logs")
}

func TestJob_PreviewSchemaCloudRunWorker_GivesUpSoonerThanARun(t *testing.T) {
	t.Parallel()

	clock := &dispatchClock{t: time.Unix(1_000_000, 0)}
	queued := queuedDebugJob(t, clock.now())
	repo := newDispatchJobRepo(queued)
	i := newDispatchJob(repo, &fakeRunWorker{answers: []error{busy}}, newFakeLeases(), clock)

	i.PreviewSchemaCloudRunWorker(queued, gateway.ProbeSchemaParam{JobID: queued.ID()})

	require.Eventually(t, func() bool { return repo.status() == job.StatusFailed }, 5*time.Second, time.Millisecond)
	waited := clock.now().Sub(queued.StartedAt())
	assert.GreaterOrEqual(t, waited, probeDispatchPolicy.MaxWait)
	assert.Less(t, waited, probeDispatchPolicy.MaxWait+probeDispatchPolicy.MaxBackoff)
}

func TestJob_Refresh_FailsAnAbandonedDispatchOnce(t *testing.T) {
	t.Parallel()

	setup := func(t *testing.T, age time.Duration) (*Job, *dispatchJobRepo, *fakeLeases, *job.Job) {
		t.Helper()
		clock := &dispatchClock{t: time.Unix(1_000_000, 0)}
		queued := queuedDebugJob(t, clock.now().Add(-age))
		repo := newDispatchJobRepo(queued)
		leases := newFakeLeases()
		return newDispatchJob(repo, &fakeRunWorker{answers: []error{nil}}, leases, clock), repo, leases, queued
	}

	t.Run("a queued job with no dispatcher is failed", func(t *testing.T) {
		t.Parallel()

		i, repo, _, queued := setup(t, 3*time.Minute)

		got, err := i.Refresh(context.Background(), queued)

		require.NoError(t, err)
		assert.Equal(t, job.StatusFailed, got.Status())
		assert.Equal(t, []job.Status{job.StatusFailed, job.StatusFailed}, repo.savedStatuses(), "the failure and its completion handling")
	})

	t.Run("refreshing it again writes nothing", func(t *testing.T) {
		t.Parallel()

		i, repo, _, queued := setup(t, 3*time.Minute)
		_, err := i.Refresh(context.Background(), queued)
		require.NoError(t, err)
		saves := len(repo.savedStatuses())

		_, err = i.Refresh(context.Background(), queued)

		require.NoError(t, err)
		assert.Len(t, repo.savedStatuses(), saves)
	})

	t.Run("two replicas noticing it fail it once", func(t *testing.T) {
		t.Parallel()

		i, repo, leases, staleCopy := setup(t, 3*time.Minute)
		other := newDispatchJob(repo, &fakeRunWorker{answers: []error{nil}}, leases, &dispatchClock{t: i.clock()})

		var wg sync.WaitGroup
		for _, replica := range []*Job{i, other} {
			wg.Add(1)
			go func() {
				defer wg.Done()
				_, err := replica.Refresh(context.Background(), staleCopy)
				assert.NoError(t, err)
			}()
		}
		wg.Wait()

		assert.Equal(t, []job.Status{job.StatusFailed, job.StatusFailed}, repo.savedStatuses())
	})

	t.Run("a job with a live dispatcher is left queued", func(t *testing.T) {
		t.Parallel()

		i, repo, _, queued := setup(t, 3*time.Minute)
		release := i.holdDispatchLease(context.Background(), queued.ID())
		defer release(false)

		got, err := i.Refresh(context.Background(), queued)

		require.NoError(t, err)
		assert.Equal(t, job.StatusPending, got.Status())
		assert.Empty(t, repo.savedStatuses())
	})

	t.Run("a job just saved is left queued for its dispatcher to take it", func(t *testing.T) {
		t.Parallel()

		i, repo, _, queued := setup(t, time.Minute)

		got, err := i.Refresh(context.Background(), queued)

		require.NoError(t, err)
		assert.Equal(t, job.StatusPending, got.Status())
		assert.Empty(t, repo.savedStatuses())
	})

	t.Run("a job whose worker has logged is running, not abandoned", func(t *testing.T) {
		t.Parallel()

		i, repo, _, queued := setup(t, 3*time.Minute)
		i.redis = &mockCheckStatusRedis{logs: []*pkglog.Log{pkglog.NewLog(queued.ID(), nil, i.clock(), pkglog.LevelInfo, "started")}}

		got, err := i.Refresh(context.Background(), queued)

		require.NoError(t, err)
		assert.Equal(t, job.StatusRunning, got.Status())
		assert.Equal(t, []job.Status{job.StatusRunning}, repo.savedStatuses())
	})

	t.Run("without a usage counter no job is read as abandoned", func(t *testing.T) {
		t.Parallel()

		i, repo, _, queued := setup(t, time.Hour)
		i.usage = nil

		got, err := i.Refresh(context.Background(), queued)

		require.NoError(t, err)
		assert.Equal(t, job.StatusPending, got.Status())
		assert.Empty(t, repo.savedStatuses())
	})
}

func TestJob_checkJobStatus_FailsAnAbandonedDispatch(t *testing.T) {
	t.Parallel()

	clock := &dispatchClock{t: time.Unix(1_000_000, 0)}
	queued := queuedDebugJob(t, clock.now().Add(-3*time.Minute))
	repo := newDispatchJobRepo(queued)
	i := newDispatchJob(repo, &fakeRunWorker{answers: []error{nil}}, newFakeLeases(), clock)

	require.NoError(t, i.checkJobStatus(context.Background(), queued))
	require.NoError(t, i.checkJobStatus(context.Background(), queued))

	assert.Equal(t, job.StatusFailed, repo.status())
	assert.Equal(t, []job.Status{job.StatusFailed, job.StatusFailed}, repo.savedStatuses(), "failed by the first check only")
}

func TestJob_RunCloudRunWorker_StaysQueuedWhileItWaits(t *testing.T) {
	t.Parallel()

	clock := &dispatchClock{t: time.Unix(1_000_000, 0)}
	queued := queuedDebugJob(t, clock.now())
	repo := newDispatchJobRepo(queued)
	var mu sync.Mutex
	var seen []job.Status
	worker := &fakeRunWorker{answers: []error{busy, busy, nil}}
	worker.onCall = func(context.Context, int) {
		mu.Lock()
		defer mu.Unlock()
		seen = append(seen, repo.status())
	}
	i := newDispatchJob(repo, worker, newFakeLeases(), clock)

	i.RunCloudRunWorker(queued, gateway.RunJobParam{JobID: queued.ID()}, nil)

	require.Eventually(t, func() bool { return worker.calls.Load() == 3 }, 5*time.Second, time.Millisecond)
	assert.Never(t, func() bool { return worker.calls.Load() > 3 }, 200*time.Millisecond, 10*time.Millisecond)
	mu.Lock()
	defer mu.Unlock()
	assert.Equal(t, []job.Status{job.StatusPending, job.StatusPending, job.StatusPending}, seen)
	assert.Empty(t, repo.savedStatuses(), "only the monitoring loop moves a queued job")
}

func notified(ch chan job.Status) []job.Status {
	var got []job.Status
	for {
		select {
		case s := <-ch:
			got = append(got, s)
		default:
			return got
		}
	}
}

func TestJob_checkJobStatus_AQueuedJobRunsFromItsFirstLog(t *testing.T) {
	t.Parallel()

	setup := func(t *testing.T, age time.Duration) (*Job, *dispatchJobRepo, *mockCheckStatusRedis, *job.Job, chan job.Status) {
		t.Helper()
		clock := &dispatchClock{t: time.Unix(1_000_000, 0)}
		queued := queuedDebugJob(t, clock.now().Add(-age))
		repo := newDispatchJobRepo(queued)
		store := &mockCheckStatusRedis{}
		i := newDispatchJob(repo, &fakeRunWorker{answers: []error{nil}}, newFakeLeases(), clock)
		i.redis = store
		return i, repo, store, queued, i.subscriptions.Subscribe(queued.ID().String())
	}
	firstLog := func(j *job.Job) []*pkglog.Log {
		return []*pkglog.Log{pkglog.NewLog(j.ID(), nil, time.Unix(1_000_000, 0), pkglog.LevelInfo, "started")}
	}

	t.Run("with no log it stays queued and nothing is notified", func(t *testing.T) {
		t.Parallel()

		i, repo, _, queued, ch := setup(t, 0)

		require.NoError(t, i.checkJobStatus(context.Background(), queued))
		require.NoError(t, i.checkJobStatus(context.Background(), queued))

		assert.Equal(t, job.StatusPending, repo.status())
		assert.Empty(t, repo.savedStatuses())
		assert.Empty(t, notified(ch))
	})

	t.Run("the first log makes it running once", func(t *testing.T) {
		t.Parallel()

		i, repo, store, queued, ch := setup(t, 0)
		require.NoError(t, i.checkJobStatus(context.Background(), queued))
		store.logs = firstLog(queued)

		require.NoError(t, i.checkJobStatus(context.Background(), queued))
		require.NoError(t, i.checkJobStatus(context.Background(), queued))

		assert.Equal(t, job.StatusRunning, repo.status())
		assert.Equal(t, []job.Status{job.StatusRunning}, repo.savedStatuses())
		assert.Equal(t, []job.Status{job.StatusRunning}, notified(ch))
		assert.Equal(t, 2, store.logsCalls, "a running job reads no logs")
	})

	t.Run("a run that finishes before any log goes straight to completed", func(t *testing.T) {
		t.Parallel()

		i, repo, store, queued, ch := setup(t, 0)
		store.event = &gateway.JobCompleteEvent{JobID: queued.ID().String(), Result: "success"}

		require.NoError(t, i.checkJobStatus(context.Background(), queued))

		assert.Equal(t, job.StatusCompleted, repo.status())
		assert.NotContains(t, repo.savedStatuses(), job.StatusRunning)
		assert.Equal(t, []job.Status{job.StatusCompleted}, notified(ch))
	})

	t.Run("a logged job with no dispatcher left is running, not abandoned", func(t *testing.T) {
		t.Parallel()

		i, repo, store, queued, _ := setup(t, 2*dispatchLeaseStartupGrace)
		store.logs = firstLog(queued)

		require.NoError(t, i.checkJobStatus(context.Background(), queued))
		require.NoError(t, i.checkJobStatus(context.Background(), queued))

		assert.Equal(t, job.StatusRunning, repo.status())
		assert.Equal(t, []job.Status{job.StatusRunning}, repo.savedStatuses())
	})

	t.Run("a log read error leaves it queued", func(t *testing.T) {
		t.Parallel()

		i, repo, store, queued, ch := setup(t, 0)
		store.logs = firstLog(queued)
		store.logsErr = fmt.Errorf("live store down")

		require.NoError(t, i.checkJobStatus(context.Background(), queued))

		assert.Equal(t, job.StatusPending, repo.status())
		assert.Empty(t, notified(ch))
	})
}

func (c *dispatchClock) advance(d time.Duration) {
	c.mu.Lock()
	defer c.mu.Unlock()
	c.t = c.t.Add(d)
}

func (f *fakeLeases) heldUntil(key, member string) time.Time {
	f.mu.Lock()
	defer f.mu.Unlock()
	return f.leases[key][member]
}

func TestJob_RunCloudRunWorker_AFinishedRunWaitsForItsResult(t *testing.T) {
	t.Parallel()

	setup := func(t *testing.T) (*Job, *dispatchJobRepo, *mockCheckStatusRedis, *dispatchClock, *job.Job) {
		t.Helper()
		clock := &dispatchClock{t: time.Unix(1_000_000, 0)}
		queued := queuedDebugJob(t, clock.now())
		repo := newDispatchJobRepo(queued)
		leases := newFakeLeases()
		i := newDispatchJob(repo, &fakeRunWorker{answers: []error{nil}}, leases, clock)

		i.RunCloudRunWorker(queued, gateway.RunJobParam{JobID: queued.ID()}, nil)
		require.Eventually(t, func() bool {
			return leases.heldUntil(dispatchLeaseKey(queued.ID()), dispatcherMember).Sub(clock.now()) >= dispatchResultWait
		}, 5*time.Second, time.Millisecond, "the dispatcher handed the job to its worker")

		store := &mockCheckStatusRedis{}
		monitor := newDispatchJob(repo, &fakeRunWorker{answers: []error{nil}}, leases, clock)
		monitor.redis = store
		return monitor, repo, store, clock, queued
	}

	t.Run("a result that arrives late completes it", func(t *testing.T) {
		t.Parallel()

		monitor, repo, store, clock, queued := setup(t)
		clock.advance(dispatchResultWait - time.Minute)

		require.NoError(t, monitor.checkJobStatus(context.Background(), queued))
		assert.Equal(t, job.StatusPending, repo.status(), "not failed as abandoned while its result is on the way")

		store.event = &gateway.JobCompleteEvent{JobID: queued.ID().String(), Result: "success"}
		require.NoError(t, monitor.checkJobStatus(context.Background(), queued))
		assert.Equal(t, job.StatusCompleted, repo.status())
		assert.NotContains(t, repo.savedStatuses(), job.StatusFailed)
	})

	t.Run("a result that never arrives fails it", func(t *testing.T) {
		t.Parallel()

		monitor, repo, _, clock, queued := setup(t)
		clock.advance(dispatchResultWait + time.Second)

		require.NoError(t, monitor.checkJobStatus(context.Background(), queued))
		assert.Equal(t, job.StatusFailed, repo.status())
	})
}

func TestJob_Refresh_LeavesAJobWithAResultToTheMonitoringLoop(t *testing.T) {
	t.Parallel()

	clock := &dispatchClock{t: time.Unix(1_000_000, 0)}
	queued := queuedDebugJob(t, clock.now().Add(-3*time.Minute))
	repo := newDispatchJobRepo(queued)
	i := newDispatchJob(repo, &fakeRunWorker{answers: []error{nil}}, newFakeLeases(), clock)
	i.redis = &mockCheckStatusRedis{event: &gateway.JobCompleteEvent{JobID: queued.ID().String(), Result: "success"}}

	got, err := i.Refresh(context.Background(), queued)

	require.NoError(t, err)
	assert.Equal(t, job.StatusPending, got.Status())
	assert.Empty(t, repo.savedStatuses())
}

type failingRenewals struct {
	*fakeLeases
	dispatcherCalls atomic.Int32
}

func (f *failingRenewals) AcquireLease(ctx context.Context, key, member string, now, expireAt time.Time) (int64, error) {
	if member == dispatcherMember && f.dispatcherCalls.Add(1) > 1 {
		return 0, errors.New("redis: i/o timeout")
	}
	return f.fakeLeases.AcquireLease(ctx, key, member, now, expireAt)
}

func TestJob_RunCloudRunWorker_ARequestInFlightOutlivesFailedLeaseRenewals(t *testing.T) {
	t.Parallel()

	clock := &dispatchClock{t: time.Unix(1_000_000, 0)}
	queued := queuedDebugJob(t, clock.now().Add(-3*time.Minute))
	repo := newDispatchJobRepo(queued)
	inFlight := make(chan struct{})
	finish := make(chan struct{})
	worker := &fakeRunWorker{answers: []error{nil}}
	worker.onCall = func(context.Context, int) {
		close(inFlight)
		<-finish
	}
	leases := &failingRenewals{fakeLeases: newFakeLeases()}
	i := newDispatchJob(repo, worker, nil, clock)
	i.usage = leases

	i.RunCloudRunWorker(queued, gateway.RunJobParam{JobID: queued.ID()}, nil)
	<-inFlight
	defer close(finish)
	clock.advance(90 * time.Second)

	other := newDispatchJob(repo, &fakeRunWorker{answers: []error{nil}}, nil, clock)
	other.usage = leases
	require.NoError(t, other.checkJobStatus(context.Background(), queued))

	assert.Equal(t, job.StatusPending, repo.status(), "renewals failed for 90 s, but the request is still in flight")
	assert.Empty(t, repo.savedStatuses())
}

type cancelOnLogRead struct {
	*mockCheckStatusRedis
	repo *dispatchJobRepo
	once sync.Once
}

func (s *cancelOnLogRead) GetLogs(ctx context.Context, since, until time.Time, jobID id.JobID) ([]*pkglog.Log, error) {
	s.once.Do(func() {
		j, _ := s.repo.FindByID(ctx, jobID)
		j.SetStatus(job.StatusCancelled)
		_ = s.repo.Save(ctx, j)
	})
	return s.mockCheckStatusRedis.GetLogs(ctx, since, until, jobID)
}

func TestJob_checkJobStatus_AFirstLogDoesNotUndoACancel(t *testing.T) {
	t.Parallel()

	clock := &dispatchClock{t: time.Unix(1_000_000, 0)}
	queued := queuedDebugJob(t, clock.now())
	repo := newDispatchJobRepo(queued)
	i := newDispatchJob(repo, &fakeRunWorker{answers: []error{nil}}, newFakeLeases(), clock)
	i.redis = &cancelOnLogRead{
		mockCheckStatusRedis: &mockCheckStatusRedis{logs: []*pkglog.Log{pkglog.NewLog(queued.ID(), nil, clock.now(), pkglog.LevelInfo, "started")}},
		repo:                 repo,
	}
	ch := i.subscriptions.Subscribe(queued.ID().String())

	require.NoError(t, i.checkJobStatus(context.Background(), queued))

	assert.Equal(t, job.StatusCancelled, repo.status())
	assert.Equal(t, []job.Status{job.StatusCancelled}, repo.savedStatuses(), "only the cancel was saved")
	assert.Empty(t, notified(ch))
}

func TestJob_RunCloudRunWorker_ACancelWhileWaitingGivesTheRunBack(t *testing.T) {
	t.Parallel()

	clock := &dispatchClock{t: time.Unix(1_000_000, 0)}
	queued := queuedDebugJob(t, clock.now())
	repo := newDispatchJobRepo(queued)
	worker := &fakeRunWorker{answers: []error{busy, nil}}
	i := newDispatchJob(repo, worker, newFakeLeases(), clock)
	i.sleep = func(ctx context.Context, d time.Duration) error {
		_, err := i.Cancel(ctx, queued.ID())
		assert.NoError(t, err)
		return clock.sleep(ctx, d)
	}
	var refunds atomic.Int32

	i.RunCloudRunWorker(queued, gateway.RunJobParam{JobID: queued.ID()}, func() { refunds.Add(1) })

	require.Eventually(t, func() bool { return refunds.Load() == 1 }, 5*time.Second, time.Millisecond, "the cancelled run never started")
	assert.Never(t, func() bool { return refunds.Load() > 1 }, 200*time.Millisecond, 10*time.Millisecond)
}

func TestJob_RunCloudRunWorker_StopsWithinOneHoldOfTheWaitLimit(t *testing.T) {
	t.Parallel()

	const cloudRunHold = 10 * time.Second
	clock := &dispatchClock{t: time.Unix(1_000_000, 0)}
	queued := queuedDebugJob(t, clock.now())
	repo := newDispatchJobRepo(queued)
	worker := &fakeRunWorker{answers: []error{busy}}
	worker.onCall = func(context.Context, int) { clock.advance(cloudRunHold) }
	i := newDispatchJob(repo, worker, newFakeLeases(), clock)
	var refunds atomic.Int32

	i.RunCloudRunWorker(queued, gateway.RunJobParam{JobID: queued.ID()}, func() { refunds.Add(1) })

	require.Eventually(t, func() bool { return repo.status() == job.StatusFailed }, 5*time.Second, time.Millisecond)
	calls := worker.calls.Load()
	assert.Never(t, func() bool { return worker.calls.Load() > calls || refunds.Load() > 1 }, 200*time.Millisecond, 10*time.Millisecond)
	assert.EqualValues(t, 1, refunds.Load())
	waited := clock.now().Sub(queued.StartedAt())
	assert.GreaterOrEqual(t, waited, runDispatchPolicy.MaxWait)
	assert.LessOrEqual(t, waited, runDispatchPolicy.MaxWait+cloudRunHold)
	assert.Equal(t, []job.Status{job.StatusFailed, job.StatusFailed}, repo.savedStatuses(), "failed once: the failure and its completion handling")
}
