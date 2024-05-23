package gql

import (
	"context"

	"github.com/reearth/reearth-flow/api/internal/adapter/gql/gqlmodel"
	"github.com/reearth/reearthx/usecasex"
)

func (r *Resolver) Workspace() WorkspaceResolver {
	return &workspaceResolver{r}
}

func (r *Resolver) WorkspaceMember() WorkspaceMemberResolver {
	return &workspaceMemberResolver{r}
}

type workspaceResolver struct{ *Resolver }

func (r *workspaceResolver) Assets(ctx context.Context, obj *gqlmodel.Workspace, first *int, last *int, after *usecasex.Cursor, before *usecasex.Cursor) (*gqlmodel.AssetConnection, error) {
	return loaders(ctx).Asset.FindByWorkspace(ctx, obj.ID, nil, nil, &gqlmodel.Pagination{
		First:  first,
		Last:   last,
		After:  after,
		Before: before,
	})
}

func (r *workspaceResolver) Projects(ctx context.Context, obj *gqlmodel.Workspace, includeArchived *bool, first *int, last *int, after *usecasex.Cursor, before *usecasex.Cursor) (*gqlmodel.ProjectConnection, error) {
	return loaders(ctx).Project.FindByWorkspace(ctx, obj.ID, first, last, before, after)
}

type workspaceMemberResolver struct{ *Resolver }

func (r *workspaceMemberResolver) User(ctx context.Context, obj *gqlmodel.WorkspaceMember) (*gqlmodel.User, error) {
	return dataloaders(ctx).User.Load(obj.UserID)
}
