package gql

import (
	"context"

	"github.com/reearth/reearth-flow/api/internal/adapter/gql/gqlmodel"
)

func (r *Resolver) Deployment() DeploymentResolver {
	return &deploymentResolver{r}
}

type deploymentResolver struct{ *Resolver }

func (r *deploymentResolver) Project(ctx context.Context, obj *gqlmodel.Deployment) (*gqlmodel.Project, error) {
	return dataloaders(ctx).Project.Load(obj.ProjectID)
}

func (r *deploymentResolver) Workspace(ctx context.Context, obj *gqlmodel.Deployment) (*gqlmodel.Workspace, error) {
	return dataloaders(ctx).Workspace.Load(obj.WorkspaceID)
}
