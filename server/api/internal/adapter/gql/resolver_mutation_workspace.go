package gql

import (
	"context"

	"github.com/reearth/reearth-flow/api/internal/adapter/gql/gqlmodel"
	"github.com/reearth/reearth-flow/api/pkg/id"
	pkgworkspace "github.com/reearth/reearth-flow/api/pkg/workspace"
)

func (r *mutationResolver) CreateWorkspace(ctx context.Context, input gqlmodel.CreateWorkspaceInput) (*gqlmodel.CreateWorkspacePayload, error) {
	res, err := usecases(ctx).Workspace.Create(ctx, input.Name)
	if err != nil {
		return nil, err
	}

	return &gqlmodel.CreateWorkspacePayload{Workspace: gqlmodel.ToWorkspace(res)}, nil
}

func (r *mutationResolver) DeleteWorkspace(ctx context.Context, input gqlmodel.DeleteWorkspaceInput) (*gqlmodel.DeleteWorkspacePayload, error) {
	tid, err := gqlmodel.ToID[id.Workspace](input.WorkspaceID)
	if err != nil {
		return nil, err
	}

	if err := usecases(ctx).Workspace.Delete(ctx, tid); err != nil {
		return nil, err
	}

	return &gqlmodel.DeleteWorkspacePayload{WorkspaceID: input.WorkspaceID}, nil
}

func (r *mutationResolver) UpdateWorkspace(ctx context.Context, input gqlmodel.UpdateWorkspaceInput) (*gqlmodel.UpdateWorkspacePayload, error) {
	tid, err := gqlmodel.ToID[id.Workspace](input.WorkspaceID)
	if err != nil {
		return nil, err
	}

	res, err := usecases(ctx).Workspace.Update(ctx, tid, input.Name)
	if err != nil {
		return nil, err
	}

	return &gqlmodel.UpdateWorkspacePayload{Workspace: gqlmodel.ToWorkspace(res)}, nil
}

func (r *mutationResolver) AddMemberToWorkspace(ctx context.Context, input gqlmodel.AddMemberToWorkspaceInput) (*gqlmodel.AddMemberToWorkspacePayload, error) {
	tid, uid, err := gqlmodel.ToID2[id.Workspace, id.User](input.WorkspaceID, input.UserID)
	if err != nil {
		return nil, err
	}

	res, err := usecases(ctx).Workspace.AddUserMember(ctx, tid, map[id.UserID]pkgworkspace.Role{uid: gqlmodel.FromRole(input.Role)})
	if err != nil {
		return nil, err
	}

	return &gqlmodel.AddMemberToWorkspacePayload{Workspace: gqlmodel.ToWorkspace(res)}, nil
}

func (r *mutationResolver) RemoveMemberFromWorkspace(ctx context.Context, input gqlmodel.RemoveMemberFromWorkspaceInput) (*gqlmodel.RemoveMemberFromWorkspacePayload, error) {
	tid, uid, err := gqlmodel.ToID2[id.Workspace, id.User](input.WorkspaceID, input.UserID)
	if err != nil {
		return nil, err
	}

	res, err := usecases(ctx).Workspace.RemoveUserMember(ctx, tid, uid)
	if err != nil {
		return nil, err
	}

	return &gqlmodel.RemoveMemberFromWorkspacePayload{Workspace: gqlmodel.ToWorkspace(res)}, nil
}

func (r *mutationResolver) UpdateMemberOfWorkspace(ctx context.Context, input gqlmodel.UpdateMemberOfWorkspaceInput) (*gqlmodel.UpdateMemberOfWorkspacePayload, error) {
	tid, uid, err := gqlmodel.ToID2[id.Workspace, id.User](input.WorkspaceID, input.UserID)
	if err != nil {
		return nil, err
	}

	res, err := usecases(ctx).Workspace.UpdateUserMember(ctx, tid, uid, gqlmodel.FromRole(input.Role))
	if err != nil {
		return nil, err
	}

	return &gqlmodel.UpdateMemberOfWorkspacePayload{Workspace: gqlmodel.ToWorkspace(res)}, nil
}

func (r *mutationResolver) UpdateWorkerConfig(ctx context.Context, input gqlmodel.UpdateWorkerConfigInput) (*gqlmodel.UpdateWorkerConfigPayload, error) {
	wid, err := gqlmodel.ToID[id.Workspace](input.WorkspaceID)
	if err != nil {
		return nil, err
	}

	res, err := usecases(ctx).WorkerConfig.Update(
		ctx,
		wid,
		input.MachineType,
		input.ComputeCPUMilli,
		input.ComputeMemoryMib,
		input.BootDiskSizeGb,
		input.TaskCount,
		input.MaxConcurrency,
		input.ThreadPoolSize,
		input.ChannelBufferSize,
		input.FeatureFlushThreshold,
		input.NodeStatusPropagationDelayMilli,
	)
	if err != nil {
		return nil, err
	}

	return &gqlmodel.UpdateWorkerConfigPayload{Config: gqlmodel.ToWorkerConfig(res)}, nil
}

func (r *mutationResolver) DeleteWorkerConfig(ctx context.Context, input gqlmodel.DeleteWorkerConfigInput) (*gqlmodel.DeleteWorkerConfigPayload, error) {
	wid, err := gqlmodel.ToID[id.Workspace](input.WorkspaceID)
	if err != nil {
		return nil, err
	}

	if err := usecases(ctx).WorkerConfig.Delete(ctx, wid); err != nil {
		return nil, err
	}

	return &gqlmodel.DeleteWorkerConfigPayload{WorkspaceID: input.WorkspaceID}, nil
}
