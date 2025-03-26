package interactor

import (
	"context"

	"github.com/reearth/reearth-flow/api/internal/rbac"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/job"
	"github.com/reearth/reearthx/log"
)

func (i *Job) Subscribe(ctx context.Context, jobID id.JobID) (chan job.Status, error) {
	if err := i.checkPermission(ctx, rbac.ActionAny); err != nil {
		return nil, err
	}

	ch := i.subscriptions.Subscribe(jobID.String())

	go func() {
		j, err := i.FindByID(ctx, jobID)
		if err == nil {
			i.subscriptions.Notify(jobID.String(), j.Status())

			if j.Status() != job.StatusCompleted &&
				j.Status() != job.StatusFailed &&
				j.Status() != job.StatusCancelled {

				i.monitoringMu.RLock()
				_, isMonitoring := i.monitoringJobs[jobID.String()]
				i.monitoringMu.RUnlock()

				if !isMonitoring {
					if err := i.StartMonitoring(ctx, j, nil); err != nil {
						log.Errorf("Failed to start monitoring for job %s: %v", jobID, err)
					}
				}
			}
		}
	}()

	return ch, nil
}

func (i *Job) Unsubscribe(jobID id.JobID, ch chan job.Status) {
	i.subscriptions.Unsubscribe(jobID.String(), ch)
}
