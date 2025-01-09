package gql

import (
	"context"

	"github.com/reearth/reearth-flow/api/internal/adapter/gql/gqlmodel"
	"github.com/reearth/reearth-flow/api/pkg/id"
)

func (r *Resolver) Subscription() SubscriptionResolver {
	return &subscriptionResolver{r}
}

type subscriptionResolver struct{ *Resolver }

func (r *subscriptionResolver) JobStatus(ctx context.Context, obj *gqlmodel.Subscription, jobID gqlmodel.ID) (gqlmodel.JobStatus, error) {
	loader := loaders(ctx).Job
	job, err := loader.FindByID(ctx, jobID)
	if err != nil {
		return "", err
	}

	jID, err := id.JobIDFrom(string(jobID))
	if err != nil {
		return "", err
	}

	statusCh, err := usecases(ctx).Job.Subscribe(ctx, jID, getOperator(ctx))
	if err != nil {
		return "", err
	}

	go func() {
		defer usecases(ctx).Job.Unsubscribe(jID, statusCh)
		for {
			select {
			case <-ctx.Done():
				return
			case status, ok := <-statusCh:
				if !ok {
					return
				}
				obj.JobStatus = gqlmodel.JobStatus(status)
			}
		}
	}()

	return gqlmodel.JobStatus(job.Status), nil
}
