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

func (r *subscriptionResolver) EdgeStatus(ctx context.Context, jobID gqlmodel.ID, edgeId string) (<-chan gqlmodel.EdgeStatus, error) {
	jid, err := id.JobIDFrom(string(jobID))
	if err != nil {
		return nil, err
	}

	edgeExCh, err := usecases(ctx).EdgeExecution.SubscribeToEdge(ctx, jid, edgeId)
	if err != nil {
		return nil, err
	}

	resultCh := make(chan gqlmodel.EdgeStatus)

	go func() {
		defer close(resultCh)
		defer usecases(ctx).EdgeExecution.UnsubscribeFromEdge(jid, string(edgeId), edgeExCh)

		for {
			select {
			case <-ctx.Done():
				return
			case edgeEx, ok := <-edgeExCh:
				if !ok {
					return
				}
				res := gqlmodel.EdgeStatus(edgeEx.Status())
				resultCh <- res
			}
		}
	}()

	return resultCh, nil
}

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
				res := gqlmodel.JobStatus(status)
				resultCh <- res
			}
		}
	}()

	return resultCh, nil
}

func (r *subscriptionResolver) Logs(ctx context.Context, jobID gqlmodel.ID) (<-chan *gqlmodel.Log, error) {
	jid, err := id.JobIDFrom(string(jobID))
	if err != nil {
		return nil, err
	}

	logsCh, err := usecases(ctx).Log.Subscribe(ctx, jid)
	if err != nil {
		return nil, err
	}

	resultCh := make(chan *gqlmodel.Log)

	go func() {
		defer close(resultCh)
		defer usecases(ctx).Log.Unsubscribe(jid, logsCh)

		for {
			select {
			case <-ctx.Done():
				return
			case log, ok := <-logsCh:
				if !ok {
					return
				}
				glog := gqlmodel.ToLog(log)
				resultCh <- glog
			}
		}
	}()

	return resultCh, nil
}
