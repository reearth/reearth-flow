package gql

import (
	"context"

	"github.com/reearth/reearth-flow/api/internal/adapter/gql/gqlmodel"
)

type projectResolver struct{ *Resolver }

func (r *projectResolver) Deployment(ctx context.Context, obj *gqlmodel.Project) (*gqlmodel.Deployment, error) {
	return dataloaders(ctx).DeploymentByProject.Load(obj.ID)
}

func (r *projectResolver) Parameters(ctx context.Context, obj *gqlmodel.Project) ([]*gqlmodel.Parameter, error) {
	return dataloaders(ctx).ParametersByProject.Load(obj.ID)
}

func (r *projectResolver) Workspace(ctx context.Context, obj *gqlmodel.Project) (*gqlmodel.Workspace, error) {
	return dataloaders(ctx).Workspace.Load(obj.WorkspaceID)
}
