package interactor

import (
	"context"
	"errors"
	"fmt"

	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/trigger"
	"github.com/reearth/reearthx/log"
)

// ReconcileScheduledJobs deletes scheduler jobs whose trigger no longer exists and
// rewrites time-driven jobs that do not send the OIDC token /execute-scheduled
// requires. Every API instance runs it on start with no user, so each step must be
// safe to repeat.
func ReconcileScheduledJobs(ctx context.Context, triggers repo.Trigger, scheduler gateway.Scheduler) error {
	if scheduler == nil {
		return nil
	}

	jobs, err := scheduler.ListScheduledJobs(ctx)
	if err != nil {
		return err
	}
	if len(jobs) == 0 {
		return nil
	}

	ids := make(id.TriggerIDList, 0, len(jobs))
	for _, j := range jobs {
		ids = append(ids, j.TriggerID)
	}
	found, err := triggers.FindByIDs(ctx, ids)
	if err != nil {
		return fmt.Errorf("failed to find the triggers of scheduled jobs: %w", err)
	}
	byID := make(map[id.TriggerID]*trigger.Trigger, len(found))
	for _, t := range found {
		if t != nil {
			byID[t.ID()] = t
		}
	}

	// Several jobs and not one trigger is more likely a database the server
	// cannot see than triggers that are all gone, so delete nothing.
	if len(byID) == 0 && len(jobs) > 1 {
		return fmt.Errorf("none of the %d scheduled jobs has a trigger; deleting none of them", len(jobs))
	}

	var errs []error
	for _, j := range jobs {
		t := byID[j.TriggerID]
		switch {
		case t == nil:
			if err := scheduler.DeleteScheduledJob(ctx, j.TriggerID); err != nil {
				errs = append(errs, fmt.Errorf("trigger %s: %w", j.TriggerID, err))
				continue
			}
			log.Infofc(ctx, "[Scheduler] deleted the job of trigger %s, which no longer exists", j.TriggerID)
		case t.EventSource() != "TIME_DRIVEN":
			// Left alone: the trigger may be mid-update, and Trigger.Update deletes
			// the job of one that stops being time-driven.
		case !j.Authenticated:
			if err := scheduler.UpdateScheduledJob(ctx, t); err != nil {
				errs = append(errs, fmt.Errorf("trigger %s: %w", j.TriggerID, err))
				continue
			}
			log.Infofc(ctx, "[Scheduler] the job of trigger %s now sends the scheduler's OIDC token", j.TriggerID)
		}
	}
	return errors.Join(errs...)
}
