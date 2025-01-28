package gql

import (
	"context"

	"github.com/reearth/reearth-flow/api/internal/adapter/gql/gqlmodel"
)

func (r *Resolver) Workspace() WorkspaceResolver {
	return &workspaceResolver{r}
}

func (r *Resolver) WorkspaceMember() WorkspaceMemberResolver {
	return &workspaceMemberResolver{r}
}

type workspaceResolver struct{ *Resolver }

func (r *workspaceResolver) Assets(ctx context.Context, obj *gqlmodel.Workspace, pagination *gqlmodel.Pagination) (*gqlmodel.AssetConnection, error) {
	return loaders(ctx).Asset.FindByWorkspace(ctx, obj.ID, nil, nil, pagination)
}

func (r *workspaceResolver) Projects(ctx context.Context, obj *gqlmodel.Workspace, includeArchived *bool, pagination *gqlmodel.Pagination) (*gqlmodel.ProjectConnection, error) {
	return loaders(ctx).Project.FindByWorkspace(ctx, obj.ID, pagination)
}

func (r *workspaceResolver) Deployments(ctx context.Context, obj *gqlmodel.Workspace, includeArchived *bool, pagination *gqlmodel.Pagination) (*gqlmodel.DeploymentConnection, error) {
	return loaders(ctx).Deployment.FindByWorkspace(ctx, obj.ID, pagination)
}

type workspaceMemberResolver struct{ *Resolver }

func (r *workspaceMemberResolver) User(ctx context.Context, obj *gqlmodel.WorkspaceMember) (*gqlmodel.User, error) {
	return dataloaders(ctx).User.Load(obj.UserID)
}
