package gql

import (
	"context"
	"fmt"

	"github.com/reearth/reearth-flow/api/internal/adapter/gql/gqlmodel"
)

func (r *queryResolver) ProjectSharingInfo(ctx context.Context, projectId gqlmodel.ID) (*gqlmodel.ProjectSharingInfoPayload, error) {
	project, err := loaders(ctx).Project.DataLoader(ctx).Load(projectId)
	if err != nil {
		return nil, err
	}

	if project == nil {
		return nil, fmt.Errorf("project not found")
	}

	return &gqlmodel.ProjectSharingInfoPayload{
		ProjectID:    projectId,
		SharingToken: project.SharedToken,
	}, nil
}

func (r *queryResolver) SharedProject(ctx context.Context, token string) (*gqlmodel.SharedProjectPayload, error) {
	res, err := usecases(ctx).ProjectAccess.Fetch(ctx, token)
	if err != nil {
		return nil, err
	}

	return &gqlmodel.SharedProjectPayload{
		Project: gqlmodel.ToProject(res),
	}, nil
}
