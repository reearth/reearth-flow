package gql

import (
	"context"
	"time"

	"github.com/reearth/reearth-flow/api/internal/adapter/gql/gqlmodel"
)

type jobResolver struct{ *Resolver }

func (r *jobResolver) Deployment(ctx context.Context, obj *gqlmodel.Job) (*gqlmodel.Deployment, error) {
	return dataloaders(ctx).Deployment.Load(obj.DeploymentID)
}

func (r *jobResolver) Workspace(ctx context.Context, obj *gqlmodel.Job) (*gqlmodel.Workspace, error) {
	return dataloaders(ctx).Workspace.Load(obj.WorkspaceID)
}

func (r *jobResolver) Logs(ctx context.Context, obj *gqlmodel.Job, since time.Time) ([]*gqlmodel.Log, error) {
	return loaders(ctx).Log.GetLogs(ctx, since, obj.ID)
}
