package gql

import (
	"context"

	"github.com/reearth/reearth-flow/api/internal/adapter/gql/gqlmodel"
)

type triggerResolver struct{ *Resolver }

func (r *triggerResolver) Workspace(ctx context.Context, obj *gqlmodel.Trigger) (*gqlmodel.Workspace, error) {
	return dataloaders(ctx).Workspace.Load(obj.WorkspaceID)
}

func (r *triggerResolver) Deployment(ctx context.Context, obj *gqlmodel.Trigger) (*gqlmodel.Deployment, error) {
	return dataloaders(ctx).Deployment.Load(obj.DeploymentID)
}
