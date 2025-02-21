package gql

import (
	"context"

	"github.com/reearth/reearth-flow/api/internal/adapter/gql/gqlmodel"
	"github.com/reearth/reearth-flow/api/pkg/id"
)

func (r *mutationResolver) CancelJob(ctx context.Context, input gqlmodel.CancelJobInput) (*gqlmodel.CancelJobPayload, error) {
	jid, err := id.JobIDFrom(string(input.JobID))
	if err != nil {
		return nil, err
	}

	job, err := usecases(ctx).Job.Cancel(ctx, jid, getOperator(ctx))
	if err != nil {
		return nil, err
	}

	return &gqlmodel.CancelJobPayload{Job: gqlmodel.ToJob(job)}, nil
}
