package interactor

import (
	"context"
	"strconv"
	"time"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/job"
	"github.com/reearth/reearthx/log"
)

// planLimits enforces the Flow limits of a workspace's plan: how many jobs it
// may have pending or running, how many calls its API triggers may receive per
// day, and whether its triggers may run every hour. The plan answers whether a
// use is allowed; Flow counts the use and sends the count.
// A failure to reach the policy service or to count allows the action and logs
// it: an outage must not stop workspaces from running.
type planLimits struct {
	policyChecker gateway.PolicyChecker
	jobRepo       repo.Job
	usage         gateway.UsageCounter
	now           func() time.Time
	jobLifetime   time.Duration
}

func newPlanLimits(r *repo.Container, gr *gateway.Container) planLimits {
	return planLimits{
		policyChecker: gr.PolicyChecker,
		jobRepo:       r.Job,
		usage:         gr.UsageCounter,
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

// ensureCanRun refuses a new job early when the workspace already has as many
// pending or running jobs as its plan allows. Call it before any work for the
// job is done, so a refused run costs nothing. It returns the limit to pass to
// saveJob, which makes the decision final: 0 means no limit applies.
func (l planLimits) ensureCanRun(ctx context.Context, wid accountsid.WorkspaceID) (int, error) {
	if l.policyChecker == nil {
		return 0, nil
	}
	running, err := l.jobRepo.CountRunningByWorkspace(ctx, wid, l.now().Add(-l.lifetime()))
	if err != nil {
		log.Warnfc(ctx, "plan limits: could not count running jobs for workspace %s, allowing the run: %v", wid, err)
		return 0, nil
	}
	res := l.check(ctx, wid, gateway.PolicyCheckConcurrentJobCount, int64(running)+1)
	if res == nil {
		return 0, nil
	}
	if !res.Allowed {
		return 0, interfaces.ErrJobAlreadyRunning
	}
	return parseLimit(res.CurrentLimit), nil
}

// parseLimit reads a check's CurrentLimit as a count. An unlimited or
// unreadable limit is 0, so saveJob reserves no slot.
func parseLimit(s string) int {
	n, err := strconv.Atoi(s)
	if err != nil || n < 0 {
		return 0
	}
	return n
}

// ensureCanRunAPITrigger is ensureCanRun for a call to an API trigger, which
// also counts toward the day's API trigger calls. Every call counts, even one
// then refused because a job is already running, so a caller looping on the
// webhook is stopped. Without a usage counter the calls are not limited.
func (l planLimits) ensureCanRunAPITrigger(ctx context.Context, wid accountsid.WorkspaceID) (int, error) {
	if l.policyChecker == nil {
		return 0, nil
	}
	if l.usage != nil {
		if err := l.countAPITriggerCall(ctx, wid); err != nil {
			return 0, err
		}
	}
	return l.ensureCanRun(ctx, wid)
}

// japanTime is the zone the daily limits reset in. Japan has no daylight
// saving, so a fixed offset is exact.
var japanTime = time.FixedZone("JST", 9*60*60)

const apiTriggerCallsKeyPrefix = "flow:plan-limits:api-trigger-calls:"

// countAPITriggerCall counts one API trigger call against the workspace's
// calls for the day, and refuses it once the plan's calls are used up. A
// counter failure allows the call.
func (l planLimits) countAPITriggerCall(ctx context.Context, wid accountsid.WorkspaceID) error {
	now := l.now().In(japanTime)
	day := time.Date(now.Year(), now.Month(), now.Day(), 0, 0, 0, 0, japanTime)
	resetAt := day.AddDate(0, 0, 1)
	key := apiTriggerCallsKeyPrefix + wid.String() + ":" + day.Format(time.DateOnly)

	calls, err := l.usage.Increment(ctx, key, resetAt)
	if err != nil {
		log.Warnfc(ctx, "plan limits: could not count API trigger calls for workspace %s, allowing the call: %v", wid, err)
		return nil
	}
	if res := l.check(ctx, wid, gateway.PolicyCheckAPITriggerCallsPerDay, calls); res != nil && !res.Allowed {
		return &interfaces.APITriggerLimitError{ResetAt: resetAt}
	}
	return nil
}

// saveJob saves a new job, reserving one of limit slots in the same database
// operation, so two runs that both passed ensureCanRun cannot both start. With
// limit 0 it is a plain save.
func (l planLimits) saveJob(ctx context.Context, j *job.Job, limit int) error {
	if limit <= 0 {
		return l.jobRepo.Save(ctx, j)
	}
	return l.jobRepo.SaveIfRunningBelow(ctx, j, limit, l.now().Add(-l.lifetime()))
}

// lifetime bounds how far back a pending or running job still counts toward
// the concurrency cap. It must be at least as long as the worker lets a job
// live, or a job that is still running stops counting and a second one starts.
// An older job is stuck, not running, and must not block its workspace forever.
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
