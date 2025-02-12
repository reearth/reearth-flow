package gql

import (
	"context"

	"github.com/reearth/reearth-flow/api/internal/adapter/gql/gqlmodel"
	"github.com/reearth/reearth-flow/api/pkg/id"
)

func (r *mutationResolver) ShareProject(ctx context.Context, input gqlmodel.ShareProjectInput) (*gqlmodel.ShareProjectPayload, error) {
	pid, err := gqlmodel.ToID[id.Project](input.ProjectID)
	if err != nil {
		return nil, err
	}

	sharingUrl, err := usecases(ctx).ProjectAccess.Share(ctx, pid)
	if err != nil {
		return nil, err
	}

	return &gqlmodel.ShareProjectPayload{ProjectID: input.ProjectID, SharingURL: sharingUrl}, nil
}

func (r *mutationResolver) UnshareProject(ctx context.Context, input gqlmodel.UnshareProjectInput) (*gqlmodel.UnshareProjectPayload, error) {
	pid, err := gqlmodel.ToID[id.Project](input.ProjectID)
	if err != nil {
		return nil, err
	}

	if err := usecases(ctx).ProjectAccess.Unshare(ctx, pid); err != nil {
		return nil, err
	}

	return &gqlmodel.UnshareProjectPayload{ProjectID: input.ProjectID}, nil
}
