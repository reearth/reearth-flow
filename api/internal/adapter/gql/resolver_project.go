package gql

import (
	"context"

	"github.com/reearth/reearth-flow/api/internal/adapter/gql/gqlmodel"
)

func (r *Resolver) Project() ProjectResolver {
	return &projectResolver{r}
}

type projectResolver struct{ *Resolver }

func (r *projectResolver) Workspace(ctx context.Context, obj *gqlmodel.Project) (*gqlmodel.Workspace, error) {
	return dataloaders(ctx).Workspace.Load(obj.WorkspaceID)
}

func (r *projectResolver) Deployment(ctx context.Context, obj *gqlmodel.Project) (*gqlmodel.Deployment, error) {
	return loaders(ctx).Deployment.FindByProject(ctx, obj.ID)
}
