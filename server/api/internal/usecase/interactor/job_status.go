package interactor

import (
	"context"

	"github.com/reearth/reearth-flow/api/internal/rbac"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/pkg/job"
	"github.com/reearth/reearthx/log"
)

func (i *Job) checkJobStatus(ctx context.Context, j *job.Job) error {
	if err := i.checkPermission(ctx, rbac.ActionAny); err != nil {
		return err
	}

	status, err := i.batch.GetJobStatus(ctx, j.GCPJobID())
	if err != nil {
		return err
	}

	newStatus := job.Status(status)
	statusChanged := j.Status() != newStatus

	if status == gateway.JobStatusCompleted || status == gateway.JobStatusFailed {
		if statusChanged {
			log.Infof("job status changing to terminal state %s from %s for job ID %s",
				newStatus, j.Status(), j.ID())

			if err := i.updateJobStatus(ctx, j, newStatus); err != nil {
				return err
			}

			if err := i.handleJobCompletion(ctx, j); err != nil {
				log.Errorfc(ctx, "job: completion handling failed: %v", err)
			}

			i.monitor.Remove(j.ID().String())
		} else {
			if len(j.OutputURLs()) == 0 {
				log.Infof("job already in terminal state %s but has no outputs, checking artifacts for job ID %s",
					j.Status(), j.ID())

				if err := i.handleJobCompletion(ctx, j); err != nil {
					log.Errorfc(ctx, "job: completion handling failed: %v", err)
				}
			}
		}
	} else if statusChanged {
		log.Infof("job status changing from %s to %s for job ID %s",
			j.Status(), newStatus, j.ID())

		if err := i.updateJobStatus(ctx, j, newStatus); err != nil {
			return err
		}
	}

	return nil
}

func (i *Job) updateJobStatus(ctx context.Context, j *job.Job, status job.Status) error {
	if err := i.checkPermission(ctx, rbac.ActionAny); err != nil {
		return err
	}

	tx, err := i.transaction.Begin(ctx)
	if err != nil {
		return err
	}
	defer func() {
		if err := tx.End(ctx); err != nil {
			log.Errorfc(ctx, "transaction end failed: %v", err)
		}
	}()

	j.SetStatus(status)
	if err := i.jobRepo.Save(ctx, j); err != nil {
		return err
	}

	tx.Commit()
	i.subscriptions.Notify(j.ID().String(), j.Status())
	return nil
}
