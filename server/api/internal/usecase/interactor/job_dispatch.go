package interactor

import (
	"context"
	"time"

	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/job"
	"github.com/reearth/reearthx/log"
)

// failJob marks an already-committed job row as failed after its cloud
// submission could not be dispatched.
//
// Submission deliberately happens after the transaction commits, so without
// this the row would sit in PENDING forever with no cloud job behind it. The
// save is best-effort: the submission error is what the caller reports, and a
// failure here only leaves the row for the operator to clean up.
//
// The dispatch failure is usually the caller's context expiring (e.g. a
// trigger's HTTP deadline), so this detaches from cancellation while keeping
// values like trace/request id, and bounds the write with its own timeout.
func failJob(ctx context.Context, jobRepo repo.Job, j *job.Job) {
	ctx, cancel := context.WithTimeout(context.WithoutCancel(ctx), 10*time.Second)
	defer cancel()

	j.SetStatus(job.StatusFailed)
	if err := jobRepo.Save(ctx, j); err != nil {
		log.Errorfc(ctx, "interactor: could not mark job %s failed after a submission error: %v", j.ID(), err)
	}
}
