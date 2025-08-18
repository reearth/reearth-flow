package gql

import (
	"context"
	"time"

	"github.com/reearth/reearth-flow/api/internal/adapter/gql/gqlmodel"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/job"
)

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

		if jobEntity, err := usecases(ctx).Job.FindByID(ctx, jID); err == nil {
			currentStatus := jobEntity.Status()

			select {
			case resultCh <- gqlmodel.JobStatus(currentStatus):
			case <-ctx.Done():
				return
			}

			if currentStatus == job.StatusCompleted ||
				currentStatus == job.StatusFailed ||
				currentStatus == job.StatusCancelled {

				select {
				case <-time.After(100 * time.Millisecond):
				case <-ctx.Done():
				}
				return
			}
		}

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

				if status == job.StatusCompleted ||
					status == job.StatusFailed ||
					status == job.StatusCancelled {
					return
				}
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

func (r *subscriptionResolver) NodeStatus(ctx context.Context, jobID gqlmodel.ID, nodeId string) (<-chan gqlmodel.NodeStatus, error) {
	jid, err := id.JobIDFrom(string(jobID))
	if err != nil {
		return nil, err
	}

	nodeExCh, err := usecases(ctx).NodeExecution.SubscribeToNode(ctx, jid, nodeId)
	if err != nil {
		return nil, err
	}

	resultCh := make(chan gqlmodel.NodeStatus)

	go func() {
		defer close(resultCh)
		defer usecases(ctx).NodeExecution.UnsubscribeFromNode(jid, string(nodeId), nodeExCh)

		for {
			select {
			case <-ctx.Done():
				return
			case nodeEx, ok := <-nodeExCh:
				if !ok {
					return
				}
				res := gqlmodel.NodeStatus(nodeEx.Status())
				resultCh <- res
			}
		}
	}()

	return resultCh, nil
}
