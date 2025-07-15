package gql

import (
	"context"

	"github.com/reearth/reearth-flow/api/internal/adapter/gql/gqlmodel"
)

type deploymentResolver struct{ *Resolver }

func (r *deploymentResolver) Project(ctx context.Context, obj *gqlmodel.Deployment) (*gqlmodel.Project, error) {
	if obj.ProjectID == nil {
		return nil, nil
	}
	return dataloaders(ctx).Project.Load(*obj.ProjectID)
}

func (r *deploymentResolver) Workspace(ctx context.Context, obj *gqlmodel.Deployment) (*gqlmodel.Workspace, error) {
	return dataloaders(ctx).Workspace.Load(obj.WorkspaceID)
}
