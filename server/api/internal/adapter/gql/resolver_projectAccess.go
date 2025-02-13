package gql

import (
	"context"

	"github.com/reearth/reearth-flow/api/internal/adapter/gql/gqlmodel"
)

func (r *queryResolver) SharedProject(ctx context.Context, token string) (*gqlmodel.SharedProjectPayload, error) {
	res, err := usecases(ctx).ProjectAccess.Fetch(ctx, token)
	if err != nil {
		return nil, err
	}

	return &gqlmodel.SharedProjectPayload{
		Project: gqlmodel.ToProject(res),
	}, nil
}
