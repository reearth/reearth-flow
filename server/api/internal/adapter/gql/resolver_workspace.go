package gql

import (
	"context"

	"github.com/reearth/reearth-flow/api/internal/adapter/gql/gqlmodel"
	"github.com/reearth/reearth-flow/api/pkg/id"
)

type workspaceResolver struct{ *Resolver }

func (r *workspaceResolver) Assets(ctx context.Context, obj *gqlmodel.Workspace, pagination *gqlmodel.Pagination) (*gqlmodel.AssetConnection, error) {
	if pagination != nil && pagination.Page != nil && pagination.PageSize != nil {
		return loaders(ctx).Asset.FindByWorkspace(ctx, obj.ID, nil, nil, &gqlmodel.PageBasedPagination{
			Page:     *pagination.Page,
			PageSize: *pagination.PageSize,
			OrderBy:  pagination.OrderBy,
			OrderDir: pagination.OrderDir,
		})
	}
	return loaders(ctx).Asset.FindByWorkspace(ctx, obj.ID, nil, nil, &gqlmodel.PageBasedPagination{
		Page:     1,
		PageSize: 30,
	})
}

func (r *workspaceResolver) AssetsPage(ctx context.Context, obj *gqlmodel.Workspace, pagination gqlmodel.PageBasedPagination) (*gqlmodel.AssetConnection, error) {
	return loaders(ctx).Asset.FindByWorkspace(ctx, obj.ID, nil, nil, &pagination)
}

func (r *workspaceResolver) Projects(ctx context.Context, obj *gqlmodel.Workspace, includeArchived *bool, pagination *gqlmodel.Pagination) (*gqlmodel.ProjectConnection, error) {
	if pagination != nil && pagination.Page != nil && pagination.PageSize != nil {
		return loaders(ctx).Project.FindByWorkspacePage(ctx, obj.ID, gqlmodel.PageBasedPagination{
			Page:     *pagination.Page,
			PageSize: *pagination.PageSize,
			OrderBy:  pagination.OrderBy,
			OrderDir: pagination.OrderDir,
		})
	}
	return nil, nil
}

func (r *workspaceResolver) ProjectsPage(ctx context.Context, obj *gqlmodel.Workspace, includeArchived *bool, pagination gqlmodel.PageBasedPagination) (*gqlmodel.ProjectConnection, error) {
	return loaders(ctx).Project.FindByWorkspacePage(ctx, obj.ID, pagination)
}

func (r *workspaceResolver) Deployments(ctx context.Context, obj *gqlmodel.Workspace, includeArchived *bool, pagination *gqlmodel.Pagination) (*gqlmodel.DeploymentConnection, error) {
	if pagination != nil && pagination.Page != nil && pagination.PageSize != nil {
		return loaders(ctx).Deployment.FindByWorkspacePage(ctx, obj.ID, gqlmodel.PageBasedPagination{
			Page:     *pagination.Page,
			PageSize: *pagination.PageSize,
			OrderBy:  pagination.OrderBy,
			OrderDir: pagination.OrderDir,
		})
	}
	return nil, nil
}

func (r *workspaceResolver) DeploymentsPage(ctx context.Context, obj *gqlmodel.Workspace, includeArchived *bool, pagination gqlmodel.PageBasedPagination) (*gqlmodel.DeploymentConnection, error) {
	return loaders(ctx).Deployment.FindByWorkspacePage(ctx, obj.ID, pagination)
}

func (r *workspaceResolver) WorkerConfig(ctx context.Context, obj *gqlmodel.Workspace) (*gqlmodel.WorkerConfig, error) {
	wid, err := gqlmodel.ToID[id.Workspace](obj.ID)
	if err != nil {
		return nil, err
	}

	cfg, err := usecases(ctx).WorkerConfig.FindByWorkspace(ctx, wid)
	if err != nil {
		return nil, err
	}

	if cfg == nil {
		return nil, nil
	}

	return gqlmodel.ToWorkerConfig(cfg), nil
}

type workspaceMemberResolver struct{ *Resolver }

func (r *workspaceMemberResolver) User(ctx context.Context, obj *gqlmodel.WorkspaceMember) (*gqlmodel.User, error) {
	return dataloaders(ctx).User.Load(obj.UserID)
}
