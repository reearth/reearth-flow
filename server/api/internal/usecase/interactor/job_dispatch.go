package interactor

import (
	"context"
	"time"

	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/job"
	"github.com/reearth/reearthx/log"
)

// failJob marks an already-committed job row as failed after its Batch
// submission could not be dispatched, so it does not sit in PENDING with no
// cloud job behind it. It detaches from cancellation, since the failure is
// usually the caller's context expiring, and bounds the write with its own timeout.
func failJob(ctx context.Context, jobRepo repo.Job, j *job.Job) {
	ctx, cancel := context.WithTimeout(context.WithoutCancel(ctx), 10*time.Second)
	defer cancel()

	j.SetStatus(job.StatusFailed)
	if err := jobRepo.Save(ctx, j); err != nil {
		log.Errorfc(ctx, "interactor: could not mark job %s failed after a submission error: %v", j.ID(), err)
	}
	// The job has no Batch job ID yet, so its backend is given, not inferred.
	logJobEventOn(ctx, jobEventSubmitFailed, j, jobBackendBatch)
}
