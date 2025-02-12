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

func (r *subscriptionResolver) JobStatus(ctx context.Context, jobID gqlmodel.ID) (<-chan gqlmodel.JobStatus, error) {
	jID, err := id.JobIDFrom(string(jobID))
	if err != nil {
		return nil, err
	}

	statusCh, err := usecases(ctx).Job.Subscribe(ctx, jID)
	if err != nil {
		return nil, err
	}

	resultCh := make(chan gqlmodel.JobStatus)

	go func() {
		defer close(resultCh)
		defer usecases(ctx).Job.Unsubscribe(jID, statusCh)

		for {
			select {
			case <-ctx.Done():
				return
			case status, ok := <-statusCh:
				if !ok {
					return
				}
				resultCh <- gqlmodel.JobStatus(status)
			}
		}
	}()

	return resultCh, nil
}
