package interactor

import (
	"context"
	"strconv"
	"time"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/id"
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

// jobAdmission is a run that ensureCanRun allowed. Its save makes the decision
// final.
type jobAdmission struct {
	l     planLimits
	since time.Time
	// limit is the plan's concurrent jobs, reserved by save when reserve is
	// set. Without a limit to apply, save is a plain save.
	limit   int
	reserve bool
	// call is the API trigger call this run was counted as, if any.
	call *apiTriggerCall
}

// save saves the new job, reserving one of the plan's slots in the same
// database operation, so two runs that both passed ensureCanRun cannot both
// start. It must not run inside a transaction: see repo.Job.SaveIfRunningBelow.
func (a jobAdmission) save(ctx context.Context, j *job.Job) error {
	if !a.reserve {
		return a.l.jobRepo.Save(ctx, j)
	}
	return a.l.jobRepo.SaveIfRunningBelow(ctx, j, a.limit, a.since)
}

// ensureCanRun refuses a new job early when the workspace already has as many
// pending or running jobs as its plan allows. Call it before any work for the
// job is done, so a refused run costs nothing, and save the job with the
// returned admission.
//
// A stored status only moves on while a monitoring loop watches the job, and
// a server restart stops the loop. So before refusing, ensureCanRun asks the
// worker about the jobs it counted, and a job that has in fact finished stops
// counting. The refusal names the jobs that are still running.
func (l planLimits) ensureCanRun(ctx context.Context, wid accountsid.WorkspaceID) (jobAdmission, error) {
	since := l.now().Add(-l.lifetime())
	none := jobAdmission{l: l, since: since}
	if l.policyChecker == nil {
		return none, nil
	}
	running, err := l.jobRepo.FindRunningByWorkspace(ctx, wid, since)
	if err != nil {
		log.Warnfc(ctx, "plan limits: could not count running jobs for workspace %s, allowing the run: %v", wid, err)
		return none, nil
	}
	res := l.check(ctx, wid, gateway.PolicyCheckConcurrentJobCount, int64(len(running))+1)
	if res == nil {
		return none, nil
	}
	if !res.Allowed {
		still := l.refresh(ctx, running)
		if len(still) < len(running) {
			res = l.check(ctx, wid, gateway.PolicyCheckConcurrentJobCount, int64(len(still))+1)
			if res == nil {
				return none, nil
			}
		}
		if !res.Allowed {
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

// parseLimit reads a check's CurrentLimit as a count of jobs to reserve, and
// reports whether there is one. Anything but "unlimited" or a non-negative
// count is logged: the plan's limit then holds only as far as ensureCanRun
// checks it, and concurrent runs may pass it together.
func (l planLimits) parseLimit(ctx context.Context, wid accountsid.WorkspaceID, s string) (int, bool) {
	if s == gateway.PolicyLimitUnlimited {
		return 0, false
	}
	n, err := strconv.Atoi(s)
	if err != nil || n < 0 {
		log.Warnfc(ctx, "plan limits: unexpected concurrent job limit %q for workspace %s, no slot is reserved", s, wid)
		return 0, false
	}
	return n, true
}

// ensureCanRunAPITrigger is ensureCanRun for a call to an API trigger, which
// also counts toward the day's API trigger calls. A call refused because the
// workspace's jobs are running still counts, so a caller looping on the
// webhook is stopped; a call the server fails can be given back with
// refundCall. Without a usage counter the calls are not limited.
func (l planLimits) ensureCanRunAPITrigger(ctx context.Context, wid accountsid.WorkspaceID) (jobAdmission, error) {
	if l.policyChecker == nil {
		return jobAdmission{l: l, since: l.now().Add(-l.lifetime())}, nil
	}
	var call *apiTriggerCall
	if l.usage != nil {
		var err error
		if call, err = l.countAPITriggerCall(ctx, wid); err != nil {
			return jobAdmission{}, err
		}
	}
	a, err := l.ensureCanRun(ctx, wid)
	if err != nil {
		return jobAdmission{}, err
	}
	a.call = call
	return a, nil
}

// japanTime is the zone the daily limits reset in. Japan has no daylight
// saving, so a fixed offset is exact.
var japanTime = time.FixedZone("JST", 9*60*60)

const apiTriggerCallsKeyPrefix = "flow:plan-limits:api-trigger-calls:"

// apiTriggerCall is one counted API trigger call.
type apiTriggerCall struct {
	resetAt time.Time
	key     string
}

// countAPITriggerCall counts one API trigger call against the workspace's
// calls for the day, and refuses it once the plan's calls are used up. A
// counter failure allows the call, which is then not counted.
func (l planLimits) countAPITriggerCall(ctx context.Context, wid accountsid.WorkspaceID) (*apiTriggerCall, error) {
	now := l.now().In(japanTime)
	day := time.Date(now.Year(), now.Month(), now.Day(), 0, 0, 0, 0, japanTime)
	call := &apiTriggerCall{
		resetAt: day.AddDate(0, 0, 1),
		key:     apiTriggerCallsKeyPrefix + wid.String() + ":" + day.Format(time.DateOnly),
	}

	calls, err := l.usage.Increment(ctx, call.key, call.resetAt)
	if err != nil {
		log.Warnfc(ctx, "plan limits: could not count API trigger calls for workspace %s, allowing the call: %v", wid, err)
		return nil, nil
	}
	if res := l.check(ctx, wid, gateway.PolicyCheckAPITriggerCallsPerDay, calls); res != nil && !res.Allowed {
		return nil, &interfaces.APITriggerLimitError{ResetAt: call.resetAt}
	}
	return call, nil
}

// refundCall gives back the API trigger call this run was counted as. Call it
// when the server fails the run after admitting it: the failure is not the
// caller's, so it must not use up the plan's calls.
func (a jobAdmission) refundCall(ctx context.Context) {
	if a.call == nil {
		return
	}
	if err := a.l.usage.Decrement(context.WithoutCancel(ctx), a.call.key, a.call.resetAt); err != nil {
		log.Warnfc(ctx, "plan limits: could not give back an API trigger call: %v", err)
	}
}

// lifetime bounds how far back a pending or running job still counts toward
// the concurrency cap. It must be at least as long as the worker lets a job
// live, or a job that is still running stops counting and a second one starts.
// A row older than that is stale: whatever ran it has stopped, and it must not
// block its workspace forever.
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
