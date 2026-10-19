package interactor

import (
	"context"
	"errors"
	"testing"
	"time"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/memory"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/pkg/job"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// planChecker answers general_private_project_creation: paid workspaces may
// have private projects, every other workspace is on the free plan.
type planChecker struct {
	paid  map[accountsid.WorkspaceID]bool
	err   error
	calls int
}

func (c *planChecker) CheckPolicy(_ context.Context, req gateway.PolicyCheckRequest) (*gateway.PolicyCheckResponse, error) {
	c.calls++
	if c.err != nil {
		return nil, c.err
	}
	return &gateway.PolicyCheckResponse{CheckType: req.CheckType, Allowed: c.paid[req.WorkspaceID], Value: req.Value}, nil
}

func TestFreeTier_EnsureCanRun(t *testing.T) {
	ctx := context.Background()
	now := time.Date(2026, 10, 19, 12, 0, 0, 0, time.UTC)
	free, paid := accountsid.NewWorkspaceID(), accountsid.NewWorkspaceID()

	running := func(ws accountsid.WorkspaceID, startedAt time.Time) *job.Job {
		return job.New().NewID().Workspace(ws).Status(job.StatusRunning).StartedAt(startedAt).MustBuild()
	}

	tests := []struct {
		checker   *planChecker
		wantErr   error
		name      string
		jobs      []*job.Job
		ws        accountsid.WorkspaceID
		limits    gateway.FreeTierLimits
		wantCalls int
	}{
		{
			name:      "no limit configured never asks the plan",
			limits:    gateway.FreeTierLimits{},
			checker:   &planChecker{},
			jobs:      []*job.Job{running(free, now)},
			ws:        free,
			wantCalls: 0,
		},
		{
			name:      "free workspace with nothing running may run",
			limits:    gateway.FreeTierLimits{ConcurrentJobs: 1},
			checker:   &planChecker{},
			ws:        free,
			wantCalls: 1,
		},
		{
			name:      "free workspace with a job running is refused",
			limits:    gateway.FreeTierLimits{ConcurrentJobs: 1},
			checker:   &planChecker{},
			jobs:      []*job.Job{running(free, now.Add(-time.Hour))},
			ws:        free,
			wantErr:   interfaces.ErrJobAlreadyRunning,
			wantCalls: 1,
		},
		{
			name:      "a job older than the default lifetime no longer blocks",
			limits:    gateway.FreeTierLimits{ConcurrentJobs: 1},
			checker:   &planChecker{},
			jobs:      []*job.Job{running(free, now.Add(-gateway.DefaultJobLifetime-time.Minute))},
			ws:        free,
			wantCalls: 1,
		},
		{
			name:      "a longer worker lifetime keeps a long job counting",
			limits:    gateway.FreeTierLimits{ConcurrentJobs: 1, JobLifetime: gateway.JobLifetimeFor(12*time.Hour, 1)},
			checker:   &planChecker{},
			jobs:      []*job.Job{running(free, now.Add(-8*time.Hour))},
			ws:        free,
			wantErr:   interfaces.ErrJobAlreadyRunning,
			wantCalls: 1,
		},
		{
			name:      "a job older than the configured lifetime no longer blocks",
			limits:    gateway.FreeTierLimits{ConcurrentJobs: 1, JobLifetime: gateway.JobLifetimeFor(12*time.Hour, 1)},
			checker:   &planChecker{},
			jobs:      []*job.Job{running(free, now.Add(-14*time.Hour))},
			ws:        free,
			wantCalls: 1,
		},
		{
			name:      "a higher limit allows more jobs at once",
			limits:    gateway.FreeTierLimits{ConcurrentJobs: 2},
			checker:   &planChecker{},
			jobs:      []*job.Job{running(free, now)},
			ws:        free,
			wantCalls: 1,
		},
		{
			name:      "paid workspace is not limited",
			limits:    gateway.FreeTierLimits{ConcurrentJobs: 1},
			checker:   &planChecker{paid: map[accountsid.WorkspaceID]bool{paid: true}},
			jobs:      []*job.Job{running(paid, now)},
			ws:        paid,
			wantCalls: 1,
		},
		{
			name:      "a policy service failure allows the run",
			limits:    gateway.FreeTierLimits{ConcurrentJobs: 1},
			checker:   &planChecker{err: errors.New("policy: check returned status 503")},
			jobs:      []*job.Job{running(free, now)},
			ws:        free,
			wantCalls: 1,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			jobRepo := memory.NewJob()
			for _, j := range tt.jobs {
				require.NoError(t, jobRepo.Save(ctx, j))
			}
			f := freeTier{
				limits:        tt.limits,
				policyChecker: tt.checker,
				jobRepo:       jobRepo,
				now:           func() time.Time { return now },
			}

			_, err := f.ensureCanRun(ctx, tt.ws)

			if tt.wantErr != nil {
				assert.ErrorIs(t, err, tt.wantErr)
			} else {
				assert.NoError(t, err)
			}
			assert.Equal(t, tt.wantCalls, tt.checker.calls)
		})
	}
}

func TestFreeTier_EnsureCanRun_NoPolicyChecker(t *testing.T) {
	ctx := context.Background()
	ws := accountsid.NewWorkspaceID()
	jobRepo := memory.NewJob()
	require.NoError(t, jobRepo.Save(ctx, job.New().NewID().Workspace(ws).Status(job.StatusRunning).StartedAt(time.Now()).MustBuild()))

	f := freeTier{limits: gateway.FreeTierLimits{ConcurrentJobs: 1}, jobRepo: jobRepo, now: time.Now}

	limit, err := f.ensureCanRun(ctx, ws)
	assert.NoError(t, err, "without a policy checker no workspace is on the free plan")
	assert.Zero(t, limit)
}

// Two runs that both pass the early check must not both be saved: the slot is
// taken when the job is saved.
func TestFreeTier_SaveJobReservesTheSlot(t *testing.T) {
	ctx := context.Background()
	ws := accountsid.NewWorkspaceID()
	f := freeTier{limits: gateway.FreeTierLimits{ConcurrentJobs: 1}, policyChecker: &planChecker{}, jobRepo: memory.NewJob(), now: time.Now}

	limitA, err := f.ensureCanRun(ctx, ws)
	require.NoError(t, err)
	limitB, err := f.ensureCanRun(ctx, ws)
	require.NoError(t, err)
	require.Equal(t, 1, limitB, "both runs see an empty workspace")

	newJob := func() *job.Job {
		return job.New().NewID().Workspace(ws).Status(job.StatusPending).StartedAt(time.Now()).MustBuild()
	}
	assert.NoError(t, f.saveJob(ctx, newJob(), limitA))
	assert.ErrorIs(t, f.saveJob(ctx, newJob(), limitB), interfaces.ErrJobAlreadyRunning)
}

func TestFreeTier_HourlyAllowed(t *testing.T) {
	ctx := context.Background()
	free, paid := accountsid.NewWorkspaceID(), accountsid.NewWorkspaceID()

	tests := []struct {
		checker   *planChecker
		wantErr   error
		name      string
		ws        accountsid.WorkspaceID
		limits    gateway.FreeTierLimits
		wantCalls int
	}{
		{name: "hourly allowed by config never asks the plan", checker: &planChecker{}, ws: free},
		{name: "free workspace is refused", limits: gateway.FreeTierLimits{DisallowHourly: true}, checker: &planChecker{}, ws: free, wantErr: interfaces.ErrHourlyScheduleNotAllowed, wantCalls: 1},
		{name: "paid workspace is allowed", limits: gateway.FreeTierLimits{DisallowHourly: true}, checker: &planChecker{paid: map[accountsid.WorkspaceID]bool{paid: true}}, ws: paid, wantCalls: 1},
		{name: "a policy service failure allows it", limits: gateway.FreeTierLimits{DisallowHourly: true}, checker: &planChecker{err: errors.New("timeout")}, ws: free, wantCalls: 1},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			f := freeTier{limits: tt.limits, policyChecker: tt.checker, jobRepo: memory.NewJob(), now: time.Now}

			err := f.hourlyAllowed(ctx, tt.ws)

			if tt.wantErr != nil {
				assert.ErrorIs(t, err, tt.wantErr)
			} else {
				assert.NoError(t, err)
			}
			assert.Equal(t, tt.wantCalls, tt.checker.calls)
		})
	}
}

// usageCounter counts in memory, like the Redis counter.
type usageCounter struct {
	err      error
	counts   map[string]int64
	expireAt time.Time
	keys     []string
}

func (c *usageCounter) Increment(_ context.Context, key string, expireAt time.Time) (int64, error) {
	if c.err != nil {
		return 0, c.err
	}
	if c.counts == nil {
		c.counts = map[string]int64{}
	}
	c.counts[key]++
	c.keys = append(c.keys, key)
	c.expireAt = expireAt
	return c.counts[key], nil
}

func TestFreeTier_EnsureCanRunAPITrigger(t *testing.T) {
	ctx := context.Background()
	jst := time.FixedZone("JST", 9*60*60)
	// 08:30 on 20 October in Japan, still 19 October in UTC.
	now := time.Date(2026, 10, 19, 23, 30, 0, 0, time.UTC)
	nextMidnight := time.Date(2026, 10, 21, 0, 0, 0, 0, jst)
	free, paid := accountsid.NewWorkspaceID(), accountsid.NewWorkspaceID()

	newFreeTier := func(limits gateway.FreeTierLimits, checker *planChecker, usage gateway.UsageCounter, jobs ...*job.Job) freeTier {
		jobRepo := memory.NewJob()
		for _, j := range jobs {
			require.NoError(t, jobRepo.Save(ctx, j))
		}
		return freeTier{limits: limits, policyChecker: checker, jobRepo: jobRepo, usage: usage, now: func() time.Time { return now }}
	}

	t.Run("refuses the call after the day's limit until midnight Japan time", func(t *testing.T) {
		checker, usage := &planChecker{}, &usageCounter{}
		f := newFreeTier(gateway.FreeTierLimits{APITriggerCallsPerDay: 2}, checker, usage)

		for range 2 {
			_, err := f.ensureCanRunAPITrigger(ctx, free)
			require.NoError(t, err)
		}
		_, err := f.ensureCanRunAPITrigger(ctx, free)

		assert.ErrorIs(t, err, interfaces.ErrAPITriggerDailyLimit)
		var limitErr *interfaces.APITriggerLimitError
		require.ErrorAs(t, err, &limitErr)
		assert.True(t, limitErr.ResetAt.Equal(nextMidnight), "resets at %s, want %s", limitErr.ResetAt, nextMidnight)
		assert.True(t, usage.expireAt.Equal(nextMidnight), "the count expires when it resets")
		assert.Contains(t, usage.keys[0], free.String()+":2026-10-20", "the day is Japan's, not UTC's")
		assert.Equal(t, 3, checker.calls, "one plan check per call")
	})

	t.Run("a new day in Japan starts a new count", func(t *testing.T) {
		usage := &usageCounter{}
		f := newFreeTier(gateway.FreeTierLimits{APITriggerCallsPerDay: 1}, &planChecker{}, usage)

		f.now = func() time.Time { return time.Date(2026, 10, 20, 14, 59, 0, 0, time.UTC) } // 23:59 JST
		_, err := f.ensureCanRunAPITrigger(ctx, free)
		require.NoError(t, err)
		f.now = func() time.Time { return time.Date(2026, 10, 20, 15, 0, 0, 0, time.UTC) } // 00:00 JST
		_, err = f.ensureCanRunAPITrigger(ctx, free)

		assert.NoError(t, err)
		assert.NotEqual(t, usage.keys[0], usage.keys[1])
	})

	t.Run("a call refused because a job is running still counts", func(t *testing.T) {
		checker, usage := &planChecker{}, &usageCounter{}
		running := job.New().NewID().Workspace(free).Status(job.StatusRunning).StartedAt(now).MustBuild()
		f := newFreeTier(gateway.FreeTierLimits{ConcurrentJobs: 1, APITriggerCallsPerDay: 5}, checker, usage, running)

		_, err := f.ensureCanRunAPITrigger(ctx, free)

		assert.ErrorIs(t, err, interfaces.ErrJobAlreadyRunning)
		assert.Len(t, usage.keys, 1)
		assert.Equal(t, 1, checker.calls, "the plan is asked once for both limits")
	})

	t.Run("returns the run limit when a call is allowed", func(t *testing.T) {
		f := newFreeTier(gateway.FreeTierLimits{ConcurrentJobs: 1, APITriggerCallsPerDay: 5}, &planChecker{}, &usageCounter{})

		limit, err := f.ensureCanRunAPITrigger(ctx, free)

		assert.NoError(t, err)
		assert.Equal(t, 1, limit)
	})

	t.Run("a paid workspace is not counted", func(t *testing.T) {
		usage := &usageCounter{}
		f := newFreeTier(gateway.FreeTierLimits{APITriggerCallsPerDay: 1}, &planChecker{paid: map[accountsid.WorkspaceID]bool{paid: true}}, usage)

		for range 3 {
			_, err := f.ensureCanRunAPITrigger(ctx, paid)
			require.NoError(t, err)
		}
		assert.Empty(t, usage.keys)
	})

	t.Run("without a counter the limit is not enforced", func(t *testing.T) {
		checker := &planChecker{}
		f := newFreeTier(gateway.FreeTierLimits{APITriggerCallsPerDay: 1}, checker, nil)

		for range 3 {
			_, err := f.ensureCanRunAPITrigger(ctx, free)
			require.NoError(t, err)
		}
		assert.Zero(t, checker.calls, "nothing to enforce, so the plan is not asked")
	})

	t.Run("a counter failure allows the call", func(t *testing.T) {
		f := newFreeTier(gateway.FreeTierLimits{APITriggerCallsPerDay: 1}, &planChecker{}, &usageCounter{err: errors.New("redis: connection refused")})

		_, err := f.ensureCanRunAPITrigger(ctx, free)

		assert.NoError(t, err)
	})
}
