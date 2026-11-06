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
	apiTriggerCallsPerDay       int
	assetSizeBytes              int64
	concurrentRenders           int
	debugConcurrentRuns         int
	debugRunsPerHour            int
	deploymentRunsPerDay        int
	jobRunTimeSeconds           int
	timeDrivenTriggerRunsPerDay int
	hourlySchedule              bool
}

// defaultFlowPlan is the dashboard's default Flow features, which every plan
// has for now.
var defaultFlowPlan = flowPlan{
	apiTriggerCallsPerDay:       3,
	assetSizeBytes:              5 << 20,
	concurrentRenders:           2,
	debugConcurrentRuns:         1,
	debugRunsPerHour:            15,
	deploymentRunsPerDay:        3,
	jobRunTimeSeconds:           30 * 60,
	timeDrivenTriggerRunsPerDay: 1,
}

// unlimitedFlowPlan sets no limit, so a test can limit only what it is about.
var unlimitedFlowPlan = flowPlan{
	apiTriggerCallsPerDay:       -1,
	assetSizeBytes:              -1,
	concurrentRenders:           -1,
	debugConcurrentRuns:         -1,
	debugRunsPerHour:            -1,
	deploymentRunsPerDay:        -1,
	jobRunTimeSeconds:           -1,
	timeDrivenTriggerRunsPerDay: -1,
	hourlySchedule:              true,
}

// with returns the plan with change applied.
func (p flowPlan) with(change func(*flowPlan)) flowPlan {
	change(&p)
	return p
}

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
	case gateway.PolicyCheckAPITriggerCallsPerDay:
		countLimit(p.apiTriggerCallsPerDay)
	case gateway.PolicyCheckAssetSize:
		res.Allowed = p.assetSizeBytes == -1 || req.Value <= p.assetSizeBytes
		res.CurrentLimit = gateway.PolicyLimitUnlimited
		if p.assetSizeBytes != -1 {
			res.CurrentLimit = strconv.FormatInt(p.assetSizeBytes>>20, 10) + "MB"
		}
	case gateway.PolicyCheckConcurrentRenderCount:
		countLimit(p.concurrentRenders)
	case gateway.PolicyCheckDebugConcurrentRunCount:
		countLimit(p.debugConcurrentRuns)
	case gateway.PolicyCheckDebugRunsPerHour:
		countLimit(p.debugRunsPerHour)
	case gateway.PolicyCheckDeploymentRunsPerDay:
		countLimit(p.deploymentRunsPerDay)
	case gateway.PolicyCheckJobRunTimeSeconds:
		countLimit(p.jobRunTimeSeconds)
	case gateway.PolicyCheckTimeDrivenTriggerRunsPerDay:
		countLimit(p.timeDrivenTriggerRunsPerDay)
	default:
		res.Allowed = true
	}
	return res, nil
}

// askedFor returns the requests for checkType, in order.
func (c *planChecker) askedFor(checkType gateway.PolicyCheckType) []gateway.PolicyCheckRequest {
	var reqs []gateway.PolicyCheckRequest
	for _, r := range c.asked {
		if r.CheckType == checkType {
			reqs = append(reqs, r)
		}
	}
	return reqs
}

func (c *planChecker) checkTypes() []gateway.PolicyCheckType {
	types := make([]gateway.PolicyCheckType, 0, len(c.asked))
	for _, r := range c.asked {
		types = append(types, r.CheckType)
	}
	return types
}

func debugRun(ws accountsid.WorkspaceID, status job.Status, startedAt time.Time) *job.Job {
	debug := true
	return job.New().NewID().Workspace(ws).Debug(&debug).Status(status).StartedAt(startedAt).MustBuild()
}

func TestPlanLimits_EnsureCanRunDebug(t *testing.T) {
	ctx := context.Background()
	now := time.Date(2026, 10, 19, 12, 0, 0, 0, time.UTC)
	ws := accountsid.NewWorkspaceID()

	running := func(startedAt time.Time) *job.Job { return debugRun(ws, job.StatusRunning, startedAt) }
	deployed := func(startedAt time.Time) *job.Job {
		debug := false
		return job.New().NewID().Workspace(ws).Debug(&debug).Status(job.StatusRunning).StartedAt(startedAt).MustBuild()
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
			name:      "a debug run running is refused",
			checker:   &planChecker{},
			jobs:      []*job.Job{running(now.Add(-time.Hour))},
			wantErr:   interfaces.ErrJobAlreadyRunning,
			wantValue: 2,
		},
		{
			name:      "a deployment or trigger run running does not count",
			checker:   &planChecker{},
			jobs:      []*job.Job{deployed(now.Add(-time.Hour))},
			wantLimit: 1,
			wantValue: 1,
		},
		{
			name:      "a debug run older than the default lifetime no longer blocks",
			checker:   &planChecker{},
			jobs:      []*job.Job{running(now.Add(-gateway.DefaultJobLifetime - time.Minute))},
			wantLimit: 1,
			wantValue: 1,
		},
		{
			name:        "a longer worker lifetime keeps a long debug run counting",
			checker:     &planChecker{},
			jobs:        []*job.Job{running(now.Add(-8 * time.Hour))},
			jobLifetime: gateway.JobLifetimeFor(12*time.Hour, 1),
			wantErr:     interfaces.ErrJobAlreadyRunning,
			wantValue:   2,
		},
		{
			name:        "a debug run older than the configured lifetime no longer blocks",
			checker:     &planChecker{},
			jobs:        []*job.Job{running(now.Add(-14 * time.Hour))},
			jobLifetime: gateway.JobLifetimeFor(12*time.Hour, 1),
			wantLimit:   1,
			wantValue:   1,
		},
		{
			name: "a plan with a higher limit allows more debug runs at once",
			checker: &planChecker{plans: map[accountsid.WorkspaceID]flowPlan{
				ws: defaultFlowPlan.with(func(p *flowPlan) { p.debugConcurrentRuns = 2 }),
			}},
			jobs:      []*job.Job{running(now)},
			wantLimit: 2,
			wantValue: 2,
		},
		{
			name:      "an unlimited plan reserves no slot",
			checker:   &planChecker{plans: map[accountsid.WorkspaceID]flowPlan{ws: unlimitedFlowPlan}},
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

			a, err := l.ensureCanRunDebug(ctx, ws)

			if tt.wantErr != nil {
				assert.ErrorIs(t, err, tt.wantErr)
			} else {
				assert.NoError(t, err)
			}
			assert.Equal(t, tt.wantLimit, a.limit)
			assert.Equal(t, tt.wantLimit > 0, a.reserve)
			asked := tt.checker.askedFor(gateway.PolicyCheckDebugConcurrentRunCount)
			require.Len(t, asked, 1)
			assert.Equal(t, tt.wantValue, asked[0].Value, "the count includes the new run")
		})
	}
}

func TestPlanLimits_NoPolicyChecker(t *testing.T) {
	ctx := context.Background()
	ws := accountsid.NewWorkspaceID()
	jobRepo := memory.NewJob()
	require.NoError(t, jobRepo.Save(ctx, debugRun(ws, job.StatusRunning, time.Now())))
	usage := &usageCounter{}
	l := planLimits{jobRepo: jobRepo, usage: usage, now: time.Now}

	a, err := l.ensureCanRunDebug(ctx, ws)
	assert.NoError(t, err)
	assert.False(t, a.reserve)
	assert.Zero(t, a.maxRun)

	for _, ensure := range []func(context.Context, accountsid.WorkspaceID) (jobAdmission, error){
		l.ensureCanRunAPITrigger, l.ensureCanRunDeployment, l.ensureCanRunTimeDrivenTrigger,
	} {
		_, err = ensure(ctx, ws)
		assert.NoError(t, err)
	}
	done, err := l.startRender(ctx, ws)
	assert.NoError(t, err)
	done()
	assert.Empty(t, usage.keys, "nothing to enforce, so nothing is counted")

	assert.NoError(t, l.hourlyAllowed(ctx, ws))
	assert.NoError(t, l.assetSizeAllowed(ctx, ws, 1<<40))
}

// Two debug runs that both pass the early check must not both be saved: the
// slot is taken when the job is saved.
func TestPlanLimits_SaveJobReservesTheSlot(t *testing.T) {
	ctx := context.Background()
	ws := accountsid.NewWorkspaceID()
	l := planLimits{policyChecker: &planChecker{}, jobRepo: memory.NewJob(), now: time.Now}

	a, err := l.ensureCanRunDebug(ctx, ws)
	require.NoError(t, err)
	b, err := l.ensureCanRunDebug(ctx, ws)
	require.NoError(t, err)
	require.Equal(t, 1, b.limit, "both runs see an empty workspace")

	assert.NoError(t, a.save(ctx, debugRun(ws, job.StatusPending, time.Now())))
	assert.ErrorIs(t, b.save(ctx, debugRun(ws, job.StatusPending, time.Now())), interfaces.ErrJobAlreadyRunning)
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
		{name: "a plan with hourly schedules is allowed", checker: &planChecker{plans: map[accountsid.WorkspaceID]flowPlan{ws: unlimitedFlowPlan}}},
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
		return &planChecker{plans: map[accountsid.WorkspaceID]flowPlan{
			ws: unlimitedFlowPlan.with(func(p *flowPlan) { p.apiTriggerCallsPerDay = n }),
		}}
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
		asked := checker.askedFor(gateway.PolicyCheckAPITriggerCallsPerDay)
		assert.Equal(t, int64(3), asked[len(asked)-1].Value, "the plan is sent the day's calls, counting this one")
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

	t.Run("a debug run in progress does not hold a call back", func(t *testing.T) {
		checker, usage := &planChecker{}, &usageCounter{}
		l := newPlanLimits(checker, usage, debugRun(ws, job.StatusRunning, now))

		a, err := l.ensureCanRunAPITrigger(ctx, ws)

		assert.NoError(t, err)
		assert.False(t, a.reserve, "trigger runs take no debug run slot")
		assert.Empty(t, checker.askedFor(gateway.PolicyCheckDebugConcurrentRunCount))
	})

	t.Run("without a counter the calls are not limited", func(t *testing.T) {
		checker := callsPerDay(1)
		l := newPlanLimits(checker, nil)

		for range 3 {
			_, err := l.ensureCanRunAPITrigger(ctx, ws)
			require.NoError(t, err)
		}
		assert.Empty(t, checker.askedFor(gateway.PolicyCheckAPITriggerCallsPerDay))
	})

	t.Run("a counter failure allows the call", func(t *testing.T) {
		l := newPlanLimits(callsPerDay(1), &usageCounter{err: errors.New("redis: connection refused")})

		_, err := l.ensureCanRunAPITrigger(ctx, ws)

		assert.NoError(t, err)
	})
}

func TestPlanLimits_CountedRuns(t *testing.T) {
	ctx := context.Background()
	jst := time.FixedZone("JST", 9*60*60)
	// 08:30 on 20 October in Japan, still 19 October in UTC.
	now := time.Date(2026, 10, 19, 23, 30, 0, 0, time.UTC)
	ws := accountsid.NewWorkspaceID()

	tests := []struct {
		resetAt   time.Time
		ensure    func(planLimits) func(context.Context, accountsid.WorkspaceID) (jobAdmission, error)
		wantErr   error
		name      string
		keyPrefix string
		window    string
		checkType gateway.PolicyCheckType
		limit     int
	}{
		{
			name: "debug runs per hour",
			ensure: func(l planLimits) func(context.Context, accountsid.WorkspaceID) (jobAdmission, error) {
				return l.ensureCanRunDebug
			},
			wantErr:   interfaces.ErrDebugRunsPerHourLimit,
			keyPrefix: "flow:plan-limits:debug-runs:",
			window:    "2026-10-20T08",
			checkType: gateway.PolicyCheckDebugRunsPerHour,
			resetAt:   time.Date(2026, 10, 20, 9, 0, 0, 0, jst),
			limit:     15,
		},
		{
			name: "deployment runs per day",
			ensure: func(l planLimits) func(context.Context, accountsid.WorkspaceID) (jobAdmission, error) {
				return l.ensureCanRunDeployment
			},
			wantErr:   interfaces.ErrDeploymentRunsPerDayLimit,
			keyPrefix: "flow:plan-limits:deployment-runs:",
			window:    "2026-10-20",
			checkType: gateway.PolicyCheckDeploymentRunsPerDay,
			resetAt:   time.Date(2026, 10, 21, 0, 0, 0, 0, jst),
			limit:     3,
		},
		{
			name: "time-driven trigger runs per day",
			ensure: func(l planLimits) func(context.Context, accountsid.WorkspaceID) (jobAdmission, error) {
				return l.ensureCanRunTimeDrivenTrigger
			},
			wantErr:   interfaces.ErrTimeDrivenTriggerRunsPerDayLimit,
			keyPrefix: "flow:plan-limits:time-driven-trigger-runs:",
			window:    "2026-10-20",
			checkType: gateway.PolicyCheckTimeDrivenTriggerRunsPerDay,
			resetAt:   time.Date(2026, 10, 21, 0, 0, 0, 0, jst),
			limit:     1,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			checker := &planChecker{plans: map[accountsid.WorkspaceID]flowPlan{
				ws: defaultFlowPlan.with(func(p *flowPlan) { p.debugConcurrentRuns = -1 }),
			}}
			usage := &usageCounter{}
			l := planLimits{policyChecker: checker, jobRepo: memory.NewJob(), usage: usage, now: func() time.Time { return now }}

			for range tt.limit {
				a, err := tt.ensure(l)(ctx, ws)
				require.NoError(t, err)
				assert.Equal(t, 30*time.Minute, a.maxRun, "the plan's run time applies")
			}
			_, err := tt.ensure(l)(ctx, ws)

			assert.ErrorIs(t, err, tt.wantErr)
			var limitErr *interfaces.UsageLimitError
			require.ErrorAs(t, err, &limitErr)
			assert.True(t, limitErr.ResetAt.Equal(tt.resetAt), "resets at %s, want %s", limitErr.ResetAt, tt.resetAt)
			assert.Equal(t, tt.keyPrefix+ws.String()+":"+tt.window, usage.keys[0])
			asked := checker.askedFor(tt.checkType)
			assert.Equal(t, int64(tt.limit+1), asked[len(asked)-1].Value, "the plan is sent the uses so far, counting this one")
		})
	}
}

func TestPlanLimits_Refund(t *testing.T) {
	ctx := context.Background()
	ws := accountsid.NewWorkspaceID()
	usage := &usageCounter{}
	l := planLimits{policyChecker: &planChecker{}, jobRepo: memory.NewJob(), usage: usage, now: time.Now}

	a, err := l.ensureCanRunDeployment(ctx, ws)
	require.NoError(t, err)
	a.refund(ctx)

	require.Len(t, usage.keys, 1)
	assert.Zero(t, usage.counts[usage.keys[0]], "a run given back no longer counts")
	jobAdmission{}.refund(ctx)
}

func TestPlanLimits_JobRunTime(t *testing.T) {
	ctx := context.Background()
	ws := accountsid.NewWorkspaceID()
	runTime := func(seconds int) *planChecker {
		return &planChecker{plans: map[accountsid.WorkspaceID]flowPlan{
			ws: unlimitedFlowPlan.with(func(p *flowPlan) { p.jobRunTimeSeconds = seconds }),
		}}
	}

	tests := []struct {
		checker *planChecker
		name    string
		want    time.Duration
	}{
		{name: "the default plan stops a job after 30 minutes", checker: &planChecker{}, want: 30 * time.Minute},
		{name: "a plan's longer limit is used even when it is allowed", checker: runTime(8 * 60 * 60), want: 8 * time.Hour},
		{name: "an unlimited plan leaves the worker's limit", checker: runTime(-1)},
		{name: "a policy service failure leaves the worker's limit", checker: &planChecker{err: errors.New("timeout")}},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			l := planLimits{policyChecker: tt.checker, now: time.Now}

			assert.Equal(t, tt.want, l.jobRunTime(ctx, ws))
		})
	}

	t.Run("an unexpected limit leaves the worker's limit", func(t *testing.T) {
		for _, s := range []string{"", "0", "-5", "30m"} {
			l := planLimits{policyChecker: fixedAnswer{CurrentLimit: s}, now: time.Now}
			assert.Zero(t, l.jobRunTime(ctx, ws), s)
		}
	})
}

// fixedAnswer answers every check with the same response.
type fixedAnswer gateway.PolicyCheckResponse

func (a fixedAnswer) CheckPolicy(context.Context, gateway.PolicyCheckRequest) (*gateway.PolicyCheckResponse, error) {
	res := gateway.PolicyCheckResponse(a)
	return &res, nil
}

func TestPlanLimits_StartRender(t *testing.T) {
	ctx := context.Background()
	ws := accountsid.NewWorkspaceID()
	usage := &usageCounter{}
	checker := &planChecker{}
	l := planLimits{policyChecker: checker, usage: usage, now: time.Now}

	first, err := l.startRender(ctx, ws)
	require.NoError(t, err)
	second, err := l.startRender(ctx, ws)
	require.NoError(t, err)

	_, err = l.startRender(ctx, ws)
	assert.ErrorIs(t, err, interfaces.ErrConcurrentRenderLimit, "the default plan renders two at a time")
	key := renderSlotKeyPrefix + ws.String()
	assert.Equal(t, int64(2), usage.counts[key], "a refused render gives its slot straight back")

	first()
	third, err := l.startRender(ctx, ws)
	assert.NoError(t, err, "a finished render frees its slot")
	second()
	third()
	assert.Zero(t, usage.counts[key])
	assert.Equal(t, []int64{1, 2, 3, 2}, values(checker.askedFor(gateway.PolicyCheckConcurrentRenderCount)))

	t.Run("a counter failure allows the render", func(t *testing.T) {
		l := planLimits{policyChecker: &planChecker{}, usage: &usageCounter{err: errors.New("redis: down")}, now: time.Now}
		done, err := l.startRender(ctx, ws)
		assert.NoError(t, err)
		done()
	})
}

func values(reqs []gateway.PolicyCheckRequest) []int64 {
	vs := make([]int64, len(reqs))
	for i, r := range reqs {
		vs[i] = r.Value
	}
	return vs
}

func TestPlanLimits_AssetSizeAllowed(t *testing.T) {
	ctx := context.Background()
	ws := accountsid.NewWorkspaceID()
	l := planLimits{policyChecker: &planChecker{}, now: time.Now}

	assert.NoError(t, l.assetSizeAllowed(ctx, ws, 5<<20))
	err := l.assetSizeAllowed(ctx, ws, 5<<20+1)
	assert.ErrorIs(t, err, interfaces.ErrAssetTooLarge)
	assert.ErrorContains(t, err, "5MB")

	unlimited := planLimits{policyChecker: &planChecker{plans: map[accountsid.WorkspaceID]flowPlan{ws: unlimitedFlowPlan}}, now: time.Now}
	assert.NoError(t, unlimited.assetSizeAllowed(ctx, ws, 1<<40))
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

// A server restart stops the monitoring loops, so a debug run can finish while
// its row still says running. The row must not lock the workspace out.
func TestPlanLimits_EnsureCanRunDebugRefreshesStaleRuns(t *testing.T) {
	ctx := context.Background()
	ws := accountsid.NewWorkspaceID()
	setup := func(t *testing.T, finished bool) (planLimits, *refreshingJobs, *planChecker, *job.Job) {
		t.Helper()
		jobRepo := memory.NewJob()
		stale := debugRun(ws, job.StatusRunning, time.Now().Add(-time.Hour))
		require.NoError(t, jobRepo.Save(ctx, stale))
		jobs := &refreshingJobs{repo: jobRepo, finished: map[string]bool{stale.ID().String(): finished}}
		checker := &planChecker{}
		return planLimits{policyChecker: checker, jobRepo: jobRepo, jobs: jobs, now: time.Now}, jobs, checker, stale
	}

	t.Run("a run that has finished stops counting", func(t *testing.T) {
		l, jobs, checker, _ := setup(t, true)

		a, err := l.ensureCanRunDebug(ctx, ws)

		require.NoError(t, err)
		assert.Equal(t, 1, jobs.asked)
		asked := checker.askedFor(gateway.PolicyCheckDebugConcurrentRunCount)
		require.Len(t, asked, 2)
		assert.Equal(t, int64(1), asked[1].Value, "asked again without the finished run")
		assert.NoError(t, a.save(ctx, debugRun(ws, job.StatusPending, time.Now())), "the save counts the refreshed row too")
	})

	t.Run("a run still going is refused and named", func(t *testing.T) {
		l, _, checker, stale := setup(t, false)

		_, err := l.ensureCanRunDebug(ctx, ws)

		assert.ErrorIs(t, err, interfaces.ErrJobAlreadyRunning)
		var limitErr *interfaces.JobLimitError
		require.ErrorAs(t, err, &limitErr)
		assert.Equal(t, []id.JobID{stale.ID()}, limitErr.Running)
		assert.Contains(t, err.Error(), stale.ID().String())
		assert.Len(t, checker.asked, 1, "nothing changed, so the plan is not asked again")
	})

	t.Run("a run that cannot be checked still counts", func(t *testing.T) {
		l, jobs, _, _ := setup(t, true)
		jobs.err = errors.New("batch: unavailable")

		_, err := l.ensureCanRunDebug(ctx, ws)

		assert.ErrorIs(t, err, interfaces.ErrJobAlreadyRunning)
	})

	t.Run("runs are only checked when the plan refuses", func(t *testing.T) {
		l, jobs, _, _ := setup(t, true)
		l.policyChecker = &planChecker{plans: map[accountsid.WorkspaceID]flowPlan{
			ws: defaultFlowPlan.with(func(p *flowPlan) { p.debugConcurrentRuns = 2 }),
		}}

		_, err := l.ensureCanRunDebug(ctx, ws)

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
