package interactor

import (
	"context"
	"time"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/job"
	"github.com/reearth/reearthx/log"
)

// freeTier applies gateway.FreeTierLimits to workspaces on the free plan.
// A failure to reach the policy service or to count jobs allows the action and
// logs it: an outage must not stop paying workspaces from running.
type freeTier struct {
	policyChecker gateway.PolicyChecker
	jobRepo       repo.Job
	usage         gateway.UsageCounter
	now           func() time.Time
	limits        gateway.FreeTierLimits
}

func newFreeTier(r *repo.Container, gr *gateway.Container) freeTier {
	return freeTier{
		limits:        gr.FreeTier,
		policyChecker: gr.PolicyChecker,
		jobRepo:       r.Job,
		usage:         gr.UsageCounter,
		now:           time.Now,
	}
}

// planAllowsPrivate asks the plan policy whether the workspace may have private
// projects. Without a policy checker every workspace may.
func planAllowsPrivate(ctx context.Context, checker gateway.PolicyChecker, wid accountsid.WorkspaceID) (bool, error) {
	if checker == nil {
		return true, nil
	}
	res, err := checker.CheckPolicy(ctx, gateway.PolicyCheckRequest{
		CheckType:   gateway.PolicyCheckPrivateProjectCreation,
		WorkspaceID: wid,
		Value:       1,
	})
	if err != nil {
		return false, err
	}
	return res.Allowed, nil
}

// isFree reports whether the workspace is on the free plan. The dashboard's
// free plan is the one that does not allow private projects, so Flow reads it
// from that answer until the policy service has Flow-specific checks.
func (f freeTier) isFree(ctx context.Context, wid accountsid.WorkspaceID) (bool, error) {
	allowed, err := planAllowsPrivate(ctx, f.policyChecker, wid)
	if err != nil {
		return false, err
	}
	return !allowed, nil
}

// ensureCanRun refuses a new job early when a free workspace already has as
// many pending or running jobs as the free tier allows. Call it before any work
// for the job is done, so a refused run costs nothing. It returns the limit to
// pass to saveJob, which makes the decision final: 0 means no limit applies.
func (f freeTier) ensureCanRun(ctx context.Context, wid accountsid.WorkspaceID) (int, error) {
	if f.limits.ConcurrentJobs <= 0 || !f.isFreeOrAllow(ctx, wid, "the run") {
		return 0, nil
	}
	return f.runLimit(ctx, wid)
}

// ensureCanRunAPITrigger is ensureCanRun for a call to an API trigger, which on
// a free workspace also counts toward the day's API trigger calls. Every call
// counts, even one then refused because a job is already running, so a caller
// looping on the webhook is stopped. The plan is asked once for both limits.
func (f freeTier) ensureCanRunAPITrigger(ctx context.Context, wid accountsid.WorkspaceID) (int, error) {
	countCalls := f.limits.APITriggerCallsPerDay > 0 && f.usage != nil
	if !countCalls && f.limits.ConcurrentJobs <= 0 {
		return 0, nil
	}
	if !f.isFreeOrAllow(ctx, wid, "the API trigger call") {
		return 0, nil
	}
	if countCalls {
		if err := f.countAPITriggerCall(ctx, wid); err != nil {
			return 0, err
		}
	}
	if f.limits.ConcurrentJobs <= 0 {
		return 0, nil
	}
	return f.runLimit(ctx, wid)
}

// isFreeOrAllow reports whether the workspace is free. When the plan cannot be
// read it logs and reports false, so the action is allowed.
func (f freeTier) isFreeOrAllow(ctx context.Context, wid accountsid.WorkspaceID, action string) bool {
	free, err := f.isFree(ctx, wid)
	if err != nil {
		log.Warnfc(ctx, "free tier: plan check failed for workspace %s, allowing %s: %v", wid, action, err)
		return false
	}
	return free
}

// japanTime is the zone the daily limits reset in. Japan has no daylight
// saving, so a fixed offset is exact.
var japanTime = time.FixedZone("JST", 9*60*60)

const apiTriggerCallsKeyPrefix = "flow:free-tier:api-trigger-calls:"

// countAPITriggerCall counts one API trigger call against the workspace's
// calls for the day, and refuses it once they are used up. A counter failure
// allows the call.
func (f freeTier) countAPITriggerCall(ctx context.Context, wid accountsid.WorkspaceID) error {
	now := f.now().In(japanTime)
	day := time.Date(now.Year(), now.Month(), now.Day(), 0, 0, 0, 0, japanTime)
	resetAt := day.AddDate(0, 0, 1)
	key := apiTriggerCallsKeyPrefix + wid.String() + ":" + day.Format(time.DateOnly)

	calls, err := f.usage.Increment(ctx, key, resetAt)
	if err != nil {
		log.Warnfc(ctx, "free tier: could not count API trigger calls for workspace %s, allowing the call: %v", wid, err)
		return nil
	}
	if calls > int64(f.limits.APITriggerCallsPerDay) {
		return &interfaces.APITriggerLimitError{ResetAt: resetAt}
	}
	return nil
}

// runLimit refuses a new job when the workspace already has as many pending or
// running jobs as the free tier allows, and otherwise returns the limit.
func (f freeTier) runLimit(ctx context.Context, wid accountsid.WorkspaceID) (int, error) {
	running, err := f.jobRepo.CountRunningByWorkspace(ctx, wid, f.now().Add(-f.jobLifetime()))
	if err != nil {
		log.Warnfc(ctx, "free tier: could not count running jobs for workspace %s, allowing the run: %v", wid, err)
		return 0, nil
	}
	if running >= f.limits.ConcurrentJobs {
		return 0, interfaces.ErrJobAlreadyRunning
	}
	return f.limits.ConcurrentJobs, nil
}

// saveJob saves a new job, reserving one of limit slots in the same database
// operation, so two runs that both passed ensureCanRun cannot both start. With
// limit 0 it is a plain save.
func (f freeTier) saveJob(ctx context.Context, j *job.Job, limit int) error {
	if limit <= 0 {
		return f.jobRepo.Save(ctx, j)
	}
	return f.jobRepo.SaveIfRunningBelow(ctx, j, limit, f.now().Add(-f.jobLifetime()))
}

// jobLifetime bounds how far back a pending or running job still counts toward
// the concurrency cap. It must be at least as long as the worker lets a job
// live, or a job that is still running stops counting and a second one starts.
// An older job is stuck, not running, and must not block its workspace forever.
func (f freeTier) jobLifetime() time.Duration {
	if f.limits.JobLifetime > 0 {
		return f.limits.JobLifetime
	}
	return gateway.DefaultJobLifetime
}

// hourlyAllowed returns ErrHourlyScheduleNotAllowed when the workspace is free
// and the free tier does not allow hourly schedules. Callers ask it before any
// transaction, and only when a trigger would become hourly.
func (f freeTier) hourlyAllowed(ctx context.Context, wid accountsid.WorkspaceID) error {
	if !f.limits.DisallowHourly {
		return nil
	}
	free, err := f.isFree(ctx, wid)
	if err != nil {
		log.Warnfc(ctx, "free tier: plan check failed for workspace %s, allowing the hourly schedule: %v", wid, err)
		return nil
	}
	if free {
		return interfaces.ErrHourlyScheduleNotAllowed
	}
	return nil
}
