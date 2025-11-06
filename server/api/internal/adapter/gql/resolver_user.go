package gql

import (
	"context"

	"github.com/reearth/reearth-flow/api/internal/adapter/gql/gqlmodel"
)

type meResolver struct{ *Resolver }

func (r *meResolver) MyWorkspace(ctx context.Context, obj *gqlmodel.Me) (*gqlmodel.Workspace, error) {
	return dataloaders(ctx).Workspace.Load(obj.MyWorkspaceID)
}

// NOTE: Although Workspaces field is removed from user, we keep it in the GraphQL schema for backward compatibility with FE.
// The resolver uses WorkspaceLoader.FindByUser to fetch workspaces.
func (r *meResolver) Workspaces(ctx context.Context, obj *gqlmodel.Me) ([]*gqlmodel.Workspace, error) {
	return loaders(ctx).Workspace.FindByUser(ctx, obj.ID)
}
