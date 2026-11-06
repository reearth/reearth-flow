package interactor

import (
	"context"
	"errors"
	"strconv"
	"time"

	"github.com/google/uuid"
	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/job"
	"github.com/reearth/reearthx/log"
)

// planLimits enforces the Flow limits of a workspace's plan: how many debug
// runs it may have pending or running and start per hour, how often its
// deployments may be run by hand and its triggers may run per day, how many
// views it may render at once, how large its assets may be, how long a job may
// run, and whether its triggers may run every hour. The plan answers whether a
// use is allowed; Flow counts the use and sends the count.
// A failure to reach the policy service or to count allows the action and logs
// it: an outage must not stop workspaces from running.
type planLimits struct {
	policyChecker gateway.PolicyChecker
	jobRepo       repo.Job
	usage         gateway.UsageCounter
	// jobs brings a counted job's stored status up to date before a run is
	// refused for it. Without it the stored status is taken as it is.
	jobs        interfaces.Job
	now         func() time.Time
	jobLifetime time.Duration
}

func newPlanLimits(r *repo.Container, gr *gateway.Container, jobs interfaces.Job) planLimits {
	return planLimits{
		policyChecker: gr.PolicyChecker,
		jobRepo:       r.Job,
		usage:         gr.UsageCounter,
		jobs:          jobs,
		now:           time.Now,
		jobLifetime:   gr.JobLifetime,
	}
}

// check asks the plan whether value uses of checkType are allowed. It returns
// nil, meaning allowed with no limit to apply, when there is no policy checker
// or the policy service cannot be reached.
func (l planLimits) check(ctx context.Context, wid accountsid.WorkspaceID, checkType gateway.PolicyCheckType, value int64) *gateway.PolicyCheckResponse {
	if l.policyChecker == nil {
		return nil
	}
	res, err := l.policyChecker.CheckPolicy(ctx, gateway.PolicyCheckRequest{
		CheckType:   checkType,
		WorkspaceID: wid,
		Value:       value,
	})
	if err != nil {
		log.Warnfc(ctx, "plan limits: %s check failed for workspace %s, allowing it: %v", checkType, wid, err)
		return nil
	}
	return res
}

// jobAdmission is a run that one of the ensureCanRun functions allowed. Its
// save makes the decision final.
type jobAdmission struct {
	// use is the per-hour or per-day use this run was counted as, if any.
	use   *countedUse
	since time.Time
	l     planLimits
	// limit is the plan's debug runs at a time, reserved by save when reserve
	// is set. Without a limit to apply, save is a plain save.
	limit   int
	reserve bool
	// maxRun is how long the plan lets the job run; 0 leaves the worker's own
	// limit.
	maxRun time.Duration
}

// save saves the new job, reserving one of the plan's debug run slots in the
// same database operation when there is one to reserve, so two debug runs that
// both passed ensureCanRunDebug cannot both start. It must not run inside a
// transaction: see repo.Job.SaveIfRunningBelow.
func (a jobAdmission) save(ctx context.Context, j *job.Job) error {
	if !a.reserve {
		return a.l.jobRepo.Save(ctx, j)
	}
	err := a.l.jobRepo.SaveIfRunningBelow(ctx, j, a.limit, a.since)
	if errors.Is(err, interfaces.ErrJobAlreadyRunning) {
		logJobRejected(ctx, j.Workspace(), jobRejectedDebugConcurrentRuns, "limit", a.limit)
	}
	return err
}

// refund gives back the use this run was counted as. Call it when the run is
// not started after all: a run that never ran must not use up the plan's runs
// or calls.
func (a jobAdmission) refund(ctx context.Context) {
	if a.use == nil {
		return
	}
	if err := a.l.usage.Decrement(context.WithoutCancel(ctx), a.use.key, a.use.resetAt); err != nil {
		log.Warnfc(ctx, "plan limits: could not give back a counted use: %v", err)
	}
}

// ensureCanRunDebug admits a debug run: it refuses one early when the
// workspace already has as many debug runs pending or running as its plan
// allows, or has started as many this hour. Call it before any work for the
// job is done, so a refused run costs nothing, save the job with the returned
// admission, and refund it if the run is not started after all.
//
// A stored status only moves on while a monitoring loop watches the job, and
// a server restart stops the loop. So before refusing, ensureCanRunDebug asks
// the worker about the runs it counted, and one that has in fact finished stops
// counting. The refusal names the runs that are still going.
func (l planLimits) ensureCanRunDebug(ctx context.Context, wid accountsid.WorkspaceID) (jobAdmission, error) {
	a, err := l.reserveDebugSlot(ctx, wid)
	if err != nil {
		return jobAdmission{}, err
	}
	if a.use, err = l.count(ctx, wid, debugRunsPerHour); err != nil {
		return jobAdmission{}, err
	}
	a.maxRun = l.jobRunTime(ctx, wid)
	return a, nil
}

// ensureCanRunDeployment admits a run of a deployment by hand, counted toward
// the day's deployment runs.
func (l planLimits) ensureCanRunDeployment(ctx context.Context, wid accountsid.WorkspaceID) (jobAdmission, error) {
	return l.ensureCanRunCounted(ctx, wid, deploymentRunsPerDay)
}

// ensureCanRunAPITrigger admits a call to an API trigger, counted toward the
// day's API trigger calls.
func (l planLimits) ensureCanRunAPITrigger(ctx context.Context, wid accountsid.WorkspaceID) (jobAdmission, error) {
	return l.ensureCanRunCounted(ctx, wid, apiTriggerCallsPerDay)
}

// ensureCanRunTimeDrivenTrigger admits a scheduled run of a time-driven
// trigger, counted toward the day's time-driven trigger runs.
func (l planLimits) ensureCanRunTimeDrivenTrigger(ctx context.Context, wid accountsid.WorkspaceID) (jobAdmission, error) {
	return l.ensureCanRunCounted(ctx, wid, timeDrivenTriggerRunsPerDay)
}

// ensureCanRunCounted admits a run that the plan limits per day, not at once:
// deployment and trigger runs may run beside each other and beside a debug
// run.
func (l planLimits) ensureCanRunCounted(ctx context.Context, wid accountsid.WorkspaceID, u usageLimit) (jobAdmission, error) {
	a := jobAdmission{l: l, since: l.now().Add(-l.lifetime())}
	if l.policyChecker == nil {
		return a, nil
	}
	var err error
	if a.use, err = l.count(ctx, wid, u); err != nil {
		return jobAdmission{}, err
	}
	a.maxRun = l.jobRunTime(ctx, wid)
	return a, nil
}

// reserveDebugSlot is the at-a-time half of ensureCanRunDebug.
func (l planLimits) reserveDebugSlot(ctx context.Context, wid accountsid.WorkspaceID) (jobAdmission, error) {
	since := l.now().Add(-l.lifetime())
	none := jobAdmission{l: l, since: since}
	if l.policyChecker == nil {
		return none, nil
	}
	running, err := l.jobRepo.FindRunningByWorkspace(ctx, wid, since)
	if err != nil {
		log.Warnfc(ctx, "plan limits: could not count running debug runs for workspace %s, allowing the run: %v", wid, err)
		return none, nil
	}
	res := l.check(ctx, wid, gateway.PolicyCheckDebugConcurrentRunCount, int64(len(running))+1)
	if res == nil {
		return none, nil
	}
	if !res.Allowed {
		still := l.refresh(ctx, running)
		if len(still) < len(running) {
			res = l.check(ctx, wid, gateway.PolicyCheckDebugConcurrentRunCount, int64(len(still))+1)
			if res == nil {
				return none, nil
			}
		}
		if !res.Allowed {
			logJobRejected(ctx, wid, jobRejectedDebugConcurrentRuns, "running", len(still), "limit", res.CurrentLimit)
			return jobAdmission{}, &interfaces.JobLimitError{Running: jobIDs(still)}
		}
	}
	limit, ok := l.parseLimit(ctx, wid, res.CurrentLimit)
	return jobAdmission{l: l, since: since, limit: limit, reserve: ok}, nil
}

// refresh returns the jobs that are still pending or running once their stored
// status is up to date. A job whose status cannot be checked still counts.
func (l planLimits) refresh(ctx context.Context, running []*job.Job) []*job.Job {
	if l.jobs == nil {
		return running
	}
	still := make([]*job.Job, 0, len(running))
	for _, j := range running {
		fresh, err := l.jobs.Refresh(ctx, j)
		if err != nil {
			log.Warnfc(ctx, "plan limits: could not check job %s, it still counts as running: %v", j.ID(), err)
			still = append(still, j)
			continue
		}
		if s := fresh.Status(); s == job.StatusPending || s == job.StatusRunning {
			still = append(still, fresh)
		}
	}
	return still
}

func jobIDs(jobs []*job.Job) []id.JobID {
	ids := make([]id.JobID, len(jobs))
	for i, j := range jobs {
		ids[i] = j.ID()
	}
	return ids
}

// parseLimit reads a check's CurrentLimit as a count of debug runs to reserve,
// and reports whether there is one. Anything but "unlimited" or a non-negative
// count is logged: the plan's limit then holds only as far as reserveDebugSlot
// checks it, and debug runs started together may pass it.
func (l planLimits) parseLimit(ctx context.Context, wid accountsid.WorkspaceID, s string) (int, bool) {
	if s == gateway.PolicyLimitUnlimited {
		return 0, false
	}
	n, err := strconv.Atoi(s)
	if err != nil || n < 0 {
		log.Warnfc(ctx, "plan limits: unexpected debug run limit %q for workspace %s, no slot is reserved", s, wid)
		return 0, false
	}
	return n, true
}

// requestedJobRunTime is the run time Flow asks the plan about. Only the
// answer's CurrentLimit, the plan's own limit, is used.
const requestedJobRunTime = 6 * time.Hour

// jobRunTime is how long the workspace's plan lets a job run, or 0 when it sets
// no limit or cannot be asked.
func (l planLimits) jobRunTime(ctx context.Context, wid accountsid.WorkspaceID) time.Duration {
	res := l.check(ctx, wid, gateway.PolicyCheckJobRunTimeSeconds, int64(requestedJobRunTime/time.Second))
	if res == nil || res.CurrentLimit == gateway.PolicyLimitUnlimited {
		return 0
	}
	seconds, err := strconv.ParseInt(res.CurrentLimit, 10, 64)
	if err != nil || seconds <= 0 {
		log.Warnfc(ctx, "plan limits: unexpected job run time limit %q for workspace %s, the worker's own limit applies", res.CurrentLimit, wid)
		return 0
	}
	return time.Duration(seconds) * time.Second
}

// japanTime is the zone the daily limits reset in. Japan has no daylight
// saving, so a fixed offset is exact.
var japanTime = time.FixedZone("JST", 9*60*60)

// usageLimit is a limit on how many times something may happen per hour or
// per day.
type usageLimit struct {
	// refuse is the error a refused use returns.
	refuse func(resetAt time.Time) error
	// keyPrefix starts the counter's key; the workspace and the window follow.
	keyPrefix string
	check     gateway.PolicyCheckType
	// reason is logged when a run is refused.
	reason string
	// hourly makes the window an hour instead of a day in Japan.
	hourly bool
}

var (
	apiTriggerCallsPerDay = usageLimit{
		keyPrefix: "flow:plan-limits:api-trigger-calls:",
		check:     gateway.PolicyCheckAPITriggerCallsPerDay,
		reason:    jobRejectedAPITriggerCalls,
		refuse: func(resetAt time.Time) error {
			return &interfaces.APITriggerLimitError{ResetAt: resetAt}
		},
	}
	debugRunsPerHour = usageLimit{
		keyPrefix: "flow:plan-limits:debug-runs:",
		check:     gateway.PolicyCheckDebugRunsPerHour,
		reason:    jobRejectedDebugRunsPerHour,
		refuse: func(resetAt time.Time) error {
			return &interfaces.UsageLimitError{Err: interfaces.ErrDebugRunsPerHourLimit, ResetAt: resetAt}
		},
		hourly: true,
	}
	deploymentRunsPerDay = usageLimit{
		keyPrefix: "flow:plan-limits:deployment-runs:",
		check:     gateway.PolicyCheckDeploymentRunsPerDay,
		reason:    jobRejectedDeploymentRuns,
		refuse: func(resetAt time.Time) error {
			return &interfaces.UsageLimitError{Err: interfaces.ErrDeploymentRunsPerDayLimit, ResetAt: resetAt}
		},
	}
	timeDrivenTriggerRunsPerDay = usageLimit{
		keyPrefix: "flow:plan-limits:time-driven-trigger-runs:",
		check:     gateway.PolicyCheckTimeDrivenTriggerRunsPerDay,
		reason:    jobRejectedTimeDrivenTriggerRuns,
		refuse: func(resetAt time.Time) error {
			return &interfaces.UsageLimitError{Err: interfaces.ErrTimeDrivenTriggerRunsPerDayLimit, ResetAt: resetAt}
		},
	}
)

// window returns when the window now is in resets, and the window's name in a
// counter key.
func (u usageLimit) window(now time.Time) (resetAt time.Time, name string) {
	now = now.In(japanTime)
	if u.hourly {
		start := now.Truncate(time.Hour)
		return start.Add(time.Hour), start.Format("2006-01-02T15")
	}
	start := time.Date(now.Year(), now.Month(), now.Day(), 0, 0, 0, 0, japanTime)
	return start.AddDate(0, 0, 1), start.Format(time.DateOnly)
}

// countedUse is one use counted against a usageLimit.
type countedUse struct {
	resetAt time.Time
	key     string
}

// count counts one use against the workspace's uses in the current window, and
// refuses it once the plan's uses are used up. A refused use still counts, so
// a caller looping on a refusal is not let through by its own retries. Without
// a usage counter, or when it fails, the use is allowed and not counted.
func (l planLimits) count(ctx context.Context, wid accountsid.WorkspaceID, u usageLimit) (*countedUse, error) {
	if l.usage == nil || l.policyChecker == nil {
		return nil, nil
	}
	resetAt, window := u.window(l.now())
	use := &countedUse{resetAt: resetAt, key: u.keyPrefix + wid.String() + ":" + window}

	n, err := l.usage.Increment(ctx, use.key, use.resetAt)
	if err != nil {
		log.Warnfc(ctx, "plan limits: could not count %s for workspace %s, allowing it: %v", u.check, wid, err)
		return nil, nil
	}
	if res := l.check(ctx, wid, u.check, n); res != nil && !res.Allowed {
		logJobRejected(ctx, wid, u.reason, "count", n, "limit", res.CurrentLimit)
		return nil, u.refuse(use.resetAt)
	}
	return use, nil
}

// renderSlotKeyPrefix starts the key holding a lease for each of a
// workspace's renders in progress.
const renderSlotKeyPrefix = "flow:plan-limits:render-leases:"

// renderWorkTimeout bounds all of a render's work once it holds a slot: the
// render itself, which renderTimeout bounds, and reading its report back.
const renderWorkTimeout = renderTimeout + time.Minute

// renderSlotLifetime is how long a render's lease lasts, which bounds how long
// a slot never given back, as when the server stops mid-render, stays taken. It
// is above renderWorkTimeout, so a render still in progress keeps its slot.
const renderSlotLifetime = renderWorkTimeout + time.Minute

// startRender takes one of the workspace's render slots, or refuses the render
// with ErrConcurrentRenderLimit when its plan's renders at a time are all
// taken. Each render holds a lease of its own, so a release can only give
// back its own slot, never one a newer render holds. Finish the render's work
// within renderWorkTimeout and call the returned function once it is done to
// give the slot back. Without a usage counter, or when it fails, the render is
// allowed.
func (l planLimits) startRender(ctx context.Context, wid accountsid.WorkspaceID) (func(), error) {
	done := func() {}
	if l.usage == nil || l.policyChecker == nil {
		return done, nil
	}
	key := renderSlotKeyPrefix + wid.String()
	lease := uuid.NewString()
	now := l.now()
	n, err := l.usage.AcquireLease(ctx, key, lease, now, now.Add(renderSlotLifetime))
	if err != nil {
		log.Warnfc(ctx, "plan limits: could not count renders for workspace %s, allowing the render: %v", wid, err)
		return done, nil
	}
	release := func() {
		if err := l.usage.ReleaseLease(context.WithoutCancel(ctx), key, lease); err != nil {
			log.Warnfc(ctx, "plan limits: could not give back a render slot for workspace %s: %v", wid, err)
		}
	}
	if res := l.check(ctx, wid, gateway.PolicyCheckConcurrentRenderCount, n); res != nil && !res.Allowed {
		release()
		log.Infofc(ctx, "plan limits: render refused for workspace %s: %d renders, limit %s", wid, n, res.CurrentLimit)
		return nil, interfaces.ErrConcurrentRenderLimit
	}
	return release, nil
}

// assetSizeAllowed returns an AssetSizeLimitError when the workspace's plan
// does not allow an asset of size bytes.
func (l planLimits) assetSizeAllowed(ctx context.Context, wid accountsid.WorkspaceID, size int64) error {
	if res := l.check(ctx, wid, gateway.PolicyCheckAssetSize, size); res != nil && !res.Allowed {
		return &interfaces.AssetSizeLimitError{Limit: res.CurrentLimit}
	}
	return nil
}

// lifetime bounds how far back a pending or running debug run still counts
// toward the plan's debug runs at a time. It must be at least as long as the
// worker lets a job live, or a run that is still going stops counting and a
// second one starts. A row older than that is stale: whatever ran it has
// stopped, and it must not block its workspace forever.
func (l planLimits) lifetime() time.Duration {
	if l.jobLifetime > 0 {
		return l.jobLifetime
	}
	return gateway.DefaultJobLifetime
}

// hourlyAllowed returns ErrHourlyScheduleNotAllowed when the workspace's plan
// does not allow hourly schedules. Callers ask it before any transaction, and
// only when a trigger would become hourly.
func (l planLimits) hourlyAllowed(ctx context.Context, wid accountsid.WorkspaceID) error {
	if res := l.check(ctx, wid, gateway.PolicyCheckHourlySchedule, 1); res != nil && !res.Allowed {
		return interfaces.ErrHourlyScheduleNotAllowed
	}
	return nil
}
