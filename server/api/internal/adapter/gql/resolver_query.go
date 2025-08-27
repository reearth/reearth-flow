package gql

import (
	"context"
	"log"

	"github.com/reearth/reearth-flow/api/internal/adapter/gql/gqlmodel"
)

type queryResolver struct{ *Resolver }

func (r *queryResolver) Assets(ctx context.Context, workspaceID gqlmodel.ID, keyword *string, sortType *gqlmodel.AssetSortType, pagination gqlmodel.PageBasedPagination) (*gqlmodel.AssetConnection, error) {
	return loaders(ctx).Asset.FindByWorkspace(ctx, workspaceID, keyword, gqlmodel.AssetSortTypeFrom(sortType), &pagination)
}

func (r *queryResolver) Me(ctx context.Context) (*gqlmodel.Me, error) {
	// TODO: During migration, fallback to legacy getUser if FlowUser is unavailable.
	// Eventually, getUser will be unified to return FlowUser from flow package.
	u := getFlowUser(ctx)
	if u != nil {
		log.Printf("DEBUG:[queryResolver.Me] Uses FlowUser %s", u.ID())
		return gqlmodel.ToMeFromFlow(u), nil
	}
	log.Printf("WARNING:[queryResolver.Me] Fallback to legacy getUser")

	u2 := getUser(ctx)
	if u2 == nil {
		return nil, nil
	}
	return gqlmodel.ToMe(u2), nil
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

func (r *queryResolver) NodeExecution(ctx context.Context, jobID gqlmodel.ID, nodeID string) (*gqlmodel.NodeExecution, error) {
	return loaders(ctx).Node.FindByJobNodeID(ctx, jobID, nodeID)
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

func (r *queryResolver) Parameters(ctx context.Context, projectID gqlmodel.ID) ([]*gqlmodel.Parameter, error) {
	return loaders(ctx).Parameter.FindByProject(ctx, projectID)
}
