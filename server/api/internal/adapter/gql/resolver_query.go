package gql

import (
	"context"

	"github.com/reearth/reearth-flow/api/internal/adapter/gql/gqlmodel"
)

func (r *Resolver) Query() QueryResolver {
	return &queryResolver{r}
}

type queryResolver struct{ *Resolver }

func (r *queryResolver) Assets(ctx context.Context, workspaceID gqlmodel.ID, keyword *string, sortType *gqlmodel.AssetSortType, pagination gqlmodel.PageBasedPagination) (*gqlmodel.AssetConnection, error) {
	return loaders(ctx).Asset.FindByWorkspace(ctx, workspaceID, keyword, gqlmodel.AssetSortTypeFrom(sortType), &pagination)
}

func (r *queryResolver) Me(ctx context.Context) (*gqlmodel.Me, error) {
	u := getUser(ctx)
	if u == nil {
		return nil, nil
	}
	return gqlmodel.ToMe(u), nil
}

func (r *queryResolver) Deployments(ctx context.Context, workspaceID gqlmodel.ID, pagination gqlmodel.PageBasedPagination) (*gqlmodel.DeploymentConnection, error) {
	return loaders(ctx).Deployment.FindByWorkspacePage(ctx, workspaceID, pagination)
}

func (r *queryResolver) DeploymentByVersion(ctx context.Context, input gqlmodel.GetByVersionInput) (*gqlmodel.Deployment, error) {
	return loaders(ctx).Deployment.FindByVersion(ctx, &input)
}

func (r *queryResolver) DeploymentHead(ctx context.Context, input gqlmodel.GetHeadInput) (*gqlmodel.Deployment, error) {
	return loaders(ctx).Deployment.FindHead(ctx, &input)
}

func (r *queryResolver) DeploymentVersions(ctx context.Context, workspaceID gqlmodel.ID, projectID *gqlmodel.ID) ([]*gqlmodel.Deployment, error) {
	return loaders(ctx).Deployment.FindVersions(ctx, workspaceID, projectID)
}

func (r *queryResolver) EdgeExecution(ctx context.Context, jobID gqlmodel.ID, edgeID string) (*gqlmodel.EdgeExecution, error) {
	return loaders(ctx).Edge.FindByJobEdgeID(ctx, jobID, edgeID)
}

func (r *queryResolver) Job(ctx context.Context, id gqlmodel.ID) (*gqlmodel.Job, error) {
	return loaders(ctx).Job.FindByID(ctx, id)
}

func (r *queryResolver) Jobs(ctx context.Context, workspaceID gqlmodel.ID, pagination gqlmodel.PageBasedPagination) (*gqlmodel.JobConnection, error) {
	return loaders(ctx).Job.FindByWorkspacePage(ctx, workspaceID, pagination)
}

func (r *queryResolver) Node(ctx context.Context, i gqlmodel.ID, typeArg gqlmodel.NodeType) (gqlmodel.Node, error) {
	dataloaders := dataloaders(ctx)
	switch typeArg {
	case gqlmodel.NodeTypeAsset:
		result, err := dataloaders.Asset.Load(i)
		if result == nil {
			return nil, nil
		}
		return result, err
	case gqlmodel.NodeTypeProject:
		result, err := dataloaders.Project.Load(i)
		if result == nil {
			return nil, nil
		}
		return result, err
	case gqlmodel.NodeTypeWorkspace:
		result, err := dataloaders.Workspace.Load(i)
		if result == nil {
			return nil, nil
		}
		return result, err
	case gqlmodel.NodeTypeUser:
		result, err := dataloaders.User.Load(i)
		if result == nil {
			return nil, nil
		}
		return result, err
	}
	return nil, nil
}

func (r *queryResolver) Nodes(ctx context.Context, ids []gqlmodel.ID, typeArg gqlmodel.NodeType) ([]gqlmodel.Node, error) {
	dataloaders := dataloaders(ctx)
	switch typeArg {
	case gqlmodel.NodeTypeAsset:
		data, err := dataloaders.Asset.LoadAll(ids)
		if len(err) > 0 && err[0] != nil {
			return nil, err[0]
		}
		nodes := make([]gqlmodel.Node, len(data))
		for i := range data {
			nodes[i] = data[i]
		}
		return nodes, nil
	case gqlmodel.NodeTypeProject:
		data, err := dataloaders.Project.LoadAll(ids)
		if len(err) > 0 && err[0] != nil {
			return nil, err[0]
		}
		nodes := make([]gqlmodel.Node, len(data))
		for i := range data {
			nodes[i] = data[i]
		}
		return nodes, nil
	case gqlmodel.NodeTypeWorkspace:
		data, err := dataloaders.Workspace.LoadAll(ids)
		if len(err) > 0 && err[0] != nil {
			return nil, err[0]
		}
		nodes := make([]gqlmodel.Node, len(data))
		for i := range data {
			nodes[i] = data[i]
		}
		return nodes, nil
	case gqlmodel.NodeTypeUser:
		data, err := dataloaders.User.LoadAll(ids)
		if len(err) > 0 && err[0] != nil {
			return nil, err[0]
		}
		nodes := make([]gqlmodel.Node, len(data))
		for i := range data {
			nodes[i] = data[i]
		}
		return nodes, nil
	default:
		return nil, nil
	}
}

func (r *queryResolver) Projects(ctx context.Context, workspaceID gqlmodel.ID, includeArchived *bool, pagination gqlmodel.PageBasedPagination) (*gqlmodel.ProjectConnection, error) {
	return loaders(ctx).Project.FindByWorkspacePage(ctx, workspaceID, pagination)
}

func (r *queryResolver) SearchUser(ctx context.Context, nameOrEmail string) (*gqlmodel.User, error) {
	return loaders(ctx).User.SearchUser(ctx, nameOrEmail)
}

func (r *queryResolver) Triggers(ctx context.Context, workspaceID gqlmodel.ID, pagination gqlmodel.PageBasedPagination) (*gqlmodel.TriggerConnection, error) {
	return loaders(ctx).Trigger.FindByWorkspacePage(ctx, workspaceID, pagination)
}
