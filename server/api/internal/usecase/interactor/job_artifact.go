package interactor

import (
	"context"
	"fmt"

	"github.com/reearth/reearth-flow/api/pkg/job"
	"github.com/reearth/reearthx/log"
)

func (i *Job) updateJobArtifacts(ctx context.Context, j *job.Job) error {
	jobID := j.ID().String()

	outputs, err := i.file.ListJobArtifacts(ctx, jobID)
	if err != nil {
		return fmt.Errorf("failed to list job artifacts: %w", err)
	}
	j.SetOutputURLs(outputs)

	logURL := i.file.GetJobLogURL(jobID)
	if logURL != "" {
		j.SetLogsURL(logURL)
	}

	return nil
}

func (i *Job) saveJobState(ctx context.Context, j *job.Job) error {
	tx, err := i.transaction.Begin(ctx)
	if err != nil {
		return fmt.Errorf("failed to begin transaction: %w", err)
	}

	defer func() {
		if err := tx.End(ctx); err != nil {
			log.Errorfc(ctx, "transaction end failed: %v", err)
		}
	}()

	if err := i.jobRepo.Save(ctx, j); err != nil {
		return fmt.Errorf("failed to save job: %w", err)
	}

	tx.Commit()
	return nil
}
