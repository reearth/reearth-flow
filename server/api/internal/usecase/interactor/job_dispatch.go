package interactor

import (
	"context"

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
func failJob(ctx context.Context, jobRepo repo.Job, j *job.Job) {
	j.SetStatus(job.StatusFailed)
	if err := jobRepo.Save(ctx, j); err != nil {
		log.Errorfc(ctx, "interactor: could not mark job %s failed after a submission error: %v", j.ID(), err)
	}
}
