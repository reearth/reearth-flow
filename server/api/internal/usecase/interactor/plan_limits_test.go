package interactor

import (
	"context"
	"errors"
	"strconv"
	"testing"
	"time"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/memory"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/job"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// flowPlan holds a plan's Flow features as the dashboard stores them. -1 means
// unlimited.
type flowPlan struct {
	hourlySchedule        bool
	concurrentJobs        int
	apiTriggerCallsPerDay int
}

// defaultFlowPlan is the dashboard's default Flow features, which every plan
// has for now.
var defaultFlowPlan = flowPlan{concurrentJobs: 1, apiTriggerCallsPerDay: 25}

// planChecker answers the flow_* checks the way the dashboard does: a count is
// allowed when it is at most the plan's limit. Workspaces not in plans have
// defaultFlowPlan.
type planChecker struct {
	plans map[accountsid.WorkspaceID]flowPlan
	err   error
	asked []gateway.PolicyCheckRequest
}

func (c *planChecker) CheckPolicy(_ context.Context, req gateway.PolicyCheckRequest) (*gateway.PolicyCheckResponse, error) {
	c.asked = append(c.asked, req)
	if c.err != nil {
		return nil, c.err
	}
	p, ok := c.plans[req.WorkspaceID]
	if !ok {
		p = defaultFlowPlan
	}
	res := &gateway.PolicyCheckResponse{CheckType: req.CheckType, Value: req.Value}
	countLimit := func(limit int) {
		res.Allowed = limit == -1 || req.Value <= int64(limit)
		res.CurrentLimit = gateway.PolicyLimitUnlimited
		if limit != -1 {
			res.CurrentLimit = strconv.Itoa(limit)
		}
	}
	switch req.CheckType {
	case gateway.PolicyCheckHourlySchedule:
		res.Allowed = p.hourlySchedule
	case gateway.PolicyCheckConcurrentJobCount:
		countLimit(p.concurrentJobs)
	case gateway.PolicyCheckAPITriggerCallsPerDay:
		countLimit(p.apiTriggerCallsPerDay)
	default:
		res.Allowed = true
	}
	return res, nil
}

func (c *planChecker) checkTypes() []gateway.PolicyCheckType {
	types := make([]gateway.PolicyCheckType, 0, len(c.asked))
	for _, r := range c.asked {
		types = append(types, r.CheckType)
	}
	return types
}

func TestPlanLimits_EnsureCanRun(t *testing.T) {
	ctx := context.Background()
	now := time.Date(2026, 10, 19, 12, 0, 0, 0, time.UTC)
	ws := accountsid.NewWorkspaceID()

	running := func(startedAt time.Time) *job.Job {
		return job.New().NewID().Workspace(ws).Status(job.StatusRunning).StartedAt(startedAt).MustBuild()
	}

	tests := []struct {
		checker     *planChecker
		wantErr     error
		name        string
		jobs        []*job.Job
		jobLifetime time.Duration
		wantLimit   int
		wantValue   int64
	}{
		{
			name:      "nothing running may run, and the slot is reserved",
			checker:   &planChecker{},
			wantLimit: 1,
			wantValue: 1,
		},
		{
			name:      "a job running is refused",
			checker:   &planChecker{},
			jobs:      []*job.Job{running(now.Add(-time.Hour))},
			wantErr:   interfaces.ErrJobAlreadyRunning,
			wantValue: 2,
		},
		{
			name:      "a job older than the default lifetime no longer blocks",
			checker:   &planChecker{},
			jobs:      []*job.Job{running(now.Add(-gateway.DefaultJobLifetime - time.Minute))},
			wantLimit: 1,
			wantValue: 1,
		},
		{
			name:        "a longer worker lifetime keeps a long job counting",
			checker:     &planChecker{},
			jobs:        []*job.Job{running(now.Add(-8 * time.Hour))},
			jobLifetime: gateway.JobLifetimeFor(12*time.Hour, 1),
			wantErr:     interfaces.ErrJobAlreadyRunning,
			wantValue:   2,
		},
		{
			name:        "a job older than the configured lifetime no longer blocks",
			checker:     &planChecker{},
			jobs:        []*job.Job{running(now.Add(-14 * time.Hour))},
			jobLifetime: gateway.JobLifetimeFor(12*time.Hour, 1),
			wantLimit:   1,
			wantValue:   1,
		},
		{
			name:      "a plan with a higher limit allows more jobs at once",
			checker:   &planChecker{plans: map[accountsid.WorkspaceID]flowPlan{ws: {concurrentJobs: 2}}},
			jobs:      []*job.Job{running(now)},
			wantLimit: 2,
			wantValue: 2,
		},
		{
			name:      "an unlimited plan reserves no slot",
			checker:   &planChecker{plans: map[accountsid.WorkspaceID]flowPlan{ws: {concurrentJobs: -1}}},
			jobs:      []*job.Job{running(now)},
			wantValue: 2,
		},
		{
			name:      "a policy service failure allows the run",
			checker:   &planChecker{err: errors.New("policy: check returned status 503")},
			jobs:      []*job.Job{running(now)},
			wantValue: 2,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			jobRepo := memory.NewJob()
			for _, j := range tt.jobs {
				require.NoError(t, jobRepo.Save(ctx, j))
			}
			l := planLimits{
				policyChecker: tt.checker,
				jobRepo:       jobRepo,
				jobLifetime:   tt.jobLifetime,
				now:           func() time.Time { return now },
			}

			a, err := l.ensureCanRun(ctx, ws)

			if tt.wantErr != nil {
				assert.ErrorIs(t, err, tt.wantErr)
			} else {
				assert.NoError(t, err)
			}
			assert.Equal(t, tt.wantLimit, a.limit)
			assert.Equal(t, tt.wantLimit > 0, a.reserve)
			require.Len(t, tt.checker.asked, 1)
			assert.Equal(t, gateway.PolicyCheckConcurrentJobCount, tt.checker.asked[0].CheckType)
			assert.Equal(t, tt.wantValue, tt.checker.asked[0].Value, "the count includes the new job")
		})
	}
}

func TestPlanLimits_NoPolicyChecker(t *testing.T) {
	ctx := context.Background()
	ws := accountsid.NewWorkspaceID()
	jobRepo := memory.NewJob()
	require.NoError(t, jobRepo.Save(ctx, job.New().NewID().Workspace(ws).Status(job.StatusRunning).StartedAt(time.Now()).MustBuild()))
	usage := &usageCounter{}
	l := planLimits{jobRepo: jobRepo, usage: usage, now: time.Now}

	a, err := l.ensureCanRun(ctx, ws)
	assert.NoError(t, err)
	assert.False(t, a.reserve)

	_, err = l.ensureCanRunAPITrigger(ctx, ws)
	assert.NoError(t, err)
	assert.Empty(t, usage.keys, "nothing to enforce, so calls are not counted")

	assert.NoError(t, l.hourlyAllowed(ctx, ws))
}

// Two runs that both pass the early check must not both be saved: the slot is
// taken when the job is saved.
func TestPlanLimits_SaveJobReservesTheSlot(t *testing.T) {
	ctx := context.Background()
	ws := accountsid.NewWorkspaceID()
	l := planLimits{policyChecker: &planChecker{}, jobRepo: memory.NewJob(), now: time.Now}

	a, err := l.ensureCanRun(ctx, ws)
	require.NoError(t, err)
	b, err := l.ensureCanRun(ctx, ws)
	require.NoError(t, err)
	require.Equal(t, 1, b.limit, "both runs see an empty workspace")

	newJob := func() *job.Job {
		return job.New().NewID().Workspace(ws).Status(job.StatusPending).StartedAt(time.Now()).MustBuild()
	}
	assert.NoError(t, a.save(ctx, newJob()))
	assert.ErrorIs(t, b.save(ctx, newJob()), interfaces.ErrJobAlreadyRunning)
}

func TestPlanLimits_HourlyAllowed(t *testing.T) {
	ctx := context.Background()
	ws := accountsid.NewWorkspaceID()

	tests := []struct {
		checker *planChecker
		wantErr error
		name    string
	}{
		{name: "a plan without hourly schedules is refused", checker: &planChecker{}, wantErr: interfaces.ErrHourlyScheduleNotAllowed},
		{name: "a plan with hourly schedules is allowed", checker: &planChecker{plans: map[accountsid.WorkspaceID]flowPlan{ws: {hourlySchedule: true}}}},
		{name: "a policy service failure allows it", checker: &planChecker{err: errors.New("timeout")}},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			l := planLimits{policyChecker: tt.checker, jobRepo: memory.NewJob(), now: time.Now}

			err := l.hourlyAllowed(ctx, ws)

			if tt.wantErr != nil {
				assert.ErrorIs(t, err, tt.wantErr)
			} else {
				assert.NoError(t, err)
			}
			assert.Equal(t, []gateway.PolicyCheckType{gateway.PolicyCheckHourlySchedule}, tt.checker.checkTypes())
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

func (c *usageCounter) Decrement(_ context.Context, key string, _ time.Time) error {
	c.counts[key]--
	return nil
}

func TestPlanLimits_EnsureCanRunAPITrigger(t *testing.T) {
	ctx := context.Background()
	jst := time.FixedZone("JST", 9*60*60)
	// 08:30 on 20 October in Japan, still 19 October in UTC.
	now := time.Date(2026, 10, 19, 23, 30, 0, 0, time.UTC)
	nextMidnight := time.Date(2026, 10, 21, 0, 0, 0, 0, jst)
	ws := accountsid.NewWorkspaceID()

	newPlanLimits := func(checker *planChecker, usage gateway.UsageCounter, jobs ...*job.Job) planLimits {
		jobRepo := memory.NewJob()
		for _, j := range jobs {
			require.NoError(t, jobRepo.Save(ctx, j))
		}
		return planLimits{policyChecker: checker, jobRepo: jobRepo, usage: usage, now: func() time.Time { return now }}
	}
	callsPerDay := func(n int) *planChecker {
		return &planChecker{plans: map[accountsid.WorkspaceID]flowPlan{ws: {concurrentJobs: -1, apiTriggerCallsPerDay: n}}}
	}

	t.Run("refuses the call after the day's limit until midnight Japan time", func(t *testing.T) {
		checker, usage := callsPerDay(2), &usageCounter{}
		l := newPlanLimits(checker, usage)

		for range 2 {
			_, err := l.ensureCanRunAPITrigger(ctx, ws)
			require.NoError(t, err)
		}
		_, err := l.ensureCanRunAPITrigger(ctx, ws)

		assert.ErrorIs(t, err, interfaces.ErrAPITriggerDailyLimit)
		var limitErr *interfaces.APITriggerLimitError
		require.ErrorAs(t, err, &limitErr)
		assert.True(t, limitErr.ResetAt.Equal(nextMidnight), "resets at %s, want %s", limitErr.ResetAt, nextMidnight)
		assert.True(t, usage.expireAt.Equal(nextMidnight), "the count expires when it resets")
		assert.Contains(t, usage.keys[0], ws.String()+":2026-10-20", "the day is Japan's, not UTC's")
		last := checker.asked[len(checker.asked)-1]
		assert.Equal(t, gateway.PolicyCheckAPITriggerCallsPerDay, last.CheckType)
		assert.Equal(t, int64(3), last.Value, "the plan is sent the day's calls, counting this one")
	})

	t.Run("a new day in Japan starts a new count", func(t *testing.T) {
		usage := &usageCounter{}
		l := newPlanLimits(callsPerDay(1), usage)

		l.now = func() time.Time { return time.Date(2026, 10, 20, 14, 59, 0, 0, time.UTC) } // 23:59 JST
		_, err := l.ensureCanRunAPITrigger(ctx, ws)
		require.NoError(t, err)
		l.now = func() time.Time { return time.Date(2026, 10, 20, 15, 0, 0, 0, time.UTC) } // 00:00 JST
		_, err = l.ensureCanRunAPITrigger(ctx, ws)

		assert.NoError(t, err)
		assert.NotEqual(t, usage.keys[0], usage.keys[1])
	})

	t.Run("a call refused because a job is running still counts", func(t *testing.T) {
		checker, usage := &planChecker{}, &usageCounter{}
		running := job.New().NewID().Workspace(ws).Status(job.StatusRunning).StartedAt(now).MustBuild()
		l := newPlanLimits(checker, usage, running)

		_, err := l.ensureCanRunAPITrigger(ctx, ws)

		assert.ErrorIs(t, err, interfaces.ErrJobAlreadyRunning)
		assert.Len(t, usage.keys, 1)
		assert.Equal(t, []gateway.PolicyCheckType{
			gateway.PolicyCheckAPITriggerCallsPerDay,
			gateway.PolicyCheckConcurrentJobCount,
		}, checker.checkTypes())
	})

	t.Run("returns the run limit when a call is allowed", func(t *testing.T) {
		l := newPlanLimits(&planChecker{}, &usageCounter{})

		a, err := l.ensureCanRunAPITrigger(ctx, ws)

		assert.NoError(t, err)
		assert.Equal(t, 1, a.limit)
	})

	t.Run("without a counter the calls are not limited", func(t *testing.T) {
		checker := callsPerDay(1)
		l := newPlanLimits(checker, nil)

		for range 3 {
			_, err := l.ensureCanRunAPITrigger(ctx, ws)
			require.NoError(t, err)
		}
		assert.NotContains(t, checker.checkTypes(), gateway.PolicyCheckAPITriggerCallsPerDay)
	})

	t.Run("a counter failure allows the call", func(t *testing.T) {
		l := newPlanLimits(callsPerDay(1), &usageCounter{err: errors.New("redis: connection refused")})

		_, err := l.ensureCanRunAPITrigger(ctx, ws)

		assert.NoError(t, err)
	})
}

// refreshingJobs answers Refresh with the status the worker reports, as the
// monitoring loop would store it.
type refreshingJobs struct {
	interfaces.Job
	repo     *memory.Job
	finished map[string]bool
	err      error
	asked    int
}

func (r *refreshingJobs) Refresh(ctx context.Context, j *job.Job) (*job.Job, error) {
	r.asked++
	if r.err != nil {
		return nil, r.err
	}
	if r.finished[j.ID().String()] {
		j.SetStatus(job.StatusCompleted)
		if err := r.repo.Save(ctx, j); err != nil {
			return nil, err
		}
	}
	return j, nil
}

// A server restart stops the monitoring loops, so a job can finish while its
// row still says running. The row must not lock the workspace out.
func TestPlanLimits_EnsureCanRunRefreshesStaleJobs(t *testing.T) {
	ctx := context.Background()
	ws := accountsid.NewWorkspaceID()
	setup := func(t *testing.T, finished bool) (planLimits, *refreshingJobs, *planChecker, *job.Job) {
		t.Helper()
		jobRepo := memory.NewJob()
		stale := job.New().NewID().Workspace(ws).Status(job.StatusRunning).StartedAt(time.Now().Add(-time.Hour)).MustBuild()
		require.NoError(t, jobRepo.Save(ctx, stale))
		jobs := &refreshingJobs{repo: jobRepo, finished: map[string]bool{stale.ID().String(): finished}}
		checker := &planChecker{}
		return planLimits{policyChecker: checker, jobRepo: jobRepo, jobs: jobs, now: time.Now}, jobs, checker, stale
	}

	t.Run("a job that has finished stops counting", func(t *testing.T) {
		l, jobs, checker, _ := setup(t, true)

		a, err := l.ensureCanRun(ctx, ws)

		require.NoError(t, err)
		assert.Equal(t, 1, jobs.asked)
		require.Len(t, checker.asked, 2)
		assert.Equal(t, int64(1), checker.asked[1].Value, "asked again without the finished job")
		assert.NoError(t, a.save(ctx, job.New().NewID().Workspace(ws).Status(job.StatusPending).StartedAt(time.Now()).MustBuild()),
			"the save counts the refreshed row too")
	})

	t.Run("a job still running is refused and named", func(t *testing.T) {
		l, _, checker, stale := setup(t, false)

		_, err := l.ensureCanRun(ctx, ws)

		assert.ErrorIs(t, err, interfaces.ErrJobAlreadyRunning)
		var limitErr *interfaces.JobLimitError
		require.ErrorAs(t, err, &limitErr)
		assert.Equal(t, []id.JobID{stale.ID()}, limitErr.Running)
		assert.Contains(t, err.Error(), stale.ID().String())
		assert.Len(t, checker.asked, 1, "nothing changed, so the plan is not asked again")
	})

	t.Run("a job that cannot be checked still counts", func(t *testing.T) {
		l, jobs, _, _ := setup(t, true)
		jobs.err = errors.New("batch: unavailable")

		_, err := l.ensureCanRun(ctx, ws)

		assert.ErrorIs(t, err, interfaces.ErrJobAlreadyRunning)
	})

	t.Run("jobs are only checked when the plan refuses", func(t *testing.T) {
		l, jobs, _, _ := setup(t, true)
		l.policyChecker = &planChecker{plans: map[accountsid.WorkspaceID]flowPlan{ws: {concurrentJobs: 2}}}

		_, err := l.ensureCanRun(ctx, ws)

		require.NoError(t, err)
		assert.Zero(t, jobs.asked)
	})
}

func TestPlanLimits_UnexpectedLimitReservesNoSlot(t *testing.T) {
	ctx := context.Background()
	ws := accountsid.NewWorkspaceID()
	for _, s := range []string{"", "-1", "1 job", "1.0"} {
		limit, ok := planLimits{}.parseLimit(ctx, ws, s)
		assert.False(t, ok, s)
		assert.Zero(t, limit, s)
	}
	limit, ok := planLimits{}.parseLimit(ctx, ws, "2")
	assert.True(t, ok)
	assert.Equal(t, 2, limit)
	_, ok = planLimits{}.parseLimit(ctx, ws, gateway.PolicyLimitUnlimited)
	assert.False(t, ok)
}
