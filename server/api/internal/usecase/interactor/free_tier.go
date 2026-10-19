package interactor

import (
	"context"
	"time"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearthx/log"
)

// freeTier applies gateway.FreeTierLimits to workspaces on the free plan.
// A failure to reach the policy service or to count jobs allows the action and
// logs it: an outage must not stop paying workspaces from running.
type freeTier struct {
	policyChecker gateway.PolicyChecker
	jobRepo       repo.Job
	now           func() time.Time
	limits        gateway.FreeTierLimits
}

func newFreeTier(r *repo.Container, gr *gateway.Container) freeTier {
	return freeTier{
		limits:        gr.FreeTier,
		policyChecker: gr.PolicyChecker,
		jobRepo:       r.Job,
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

// ensureCanRun refuses a new job when a free workspace already has as many
// pending or running jobs as the free tier allows. Call it before any work for
// the job is done, so a refused run costs nothing.
func (f freeTier) ensureCanRun(ctx context.Context, wid accountsid.WorkspaceID) error {
	if f.limits.ConcurrentJobs <= 0 {
		return nil
	}
	free, err := f.isFree(ctx, wid)
	if err != nil {
		log.Warnfc(ctx, "free tier: plan check failed for workspace %s, allowing the run: %v", wid, err)
		return nil
	}
	if !free {
		return nil
	}
	running, err := f.jobRepo.CountRunningByWorkspace(ctx, wid, f.now().Add(-f.jobLifetime()))
	if err != nil {
		log.Warnfc(ctx, "free tier: could not count running jobs for workspace %s, allowing the run: %v", wid, err)
		return nil
	}
	if running >= f.limits.ConcurrentJobs {
		return interfaces.ErrJobAlreadyRunning
	}
	return nil
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
