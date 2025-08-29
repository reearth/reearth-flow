package gql

import (
	"context"
	"log"

	"github.com/reearth/reearth-flow/api/internal/adapter/gql/gqlmodel"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearthx/account/accountdomain"
	"github.com/reearth/reearthx/account/accountdomain/workspace"
)

// TODO: After migration, remove this logic and use the new usecase directly.
func (r *mutationResolver) CreateWorkspace(ctx context.Context, input gqlmodel.CreateWorkspaceInput) (*gqlmodel.CreateWorkspacePayload, error) {
	if usecases(ctx).TempNewWorkspace != nil {
		tempRes, err := usecases(ctx).TempNewWorkspace.Create(ctx, input.Name)
		if err != nil {
			log.Printf("WARNING:[mutationResolver.CreateWorkspaceWithTempNewUsecase] Failed to create workspace: %v", err)
		} else if tempRes == nil {
			log.Printf("DWARNINGEBUG:[mutationResolver.CreateWorkspaceWithTempNewUsecase] Created workspace is nil")
		} else {
			log.Printf("DEBUG:[mutationResolver.CreateWorkspaceWithTempNewUsecase] Created workspace with tempNewUsecase")
			return &gqlmodel.CreateWorkspacePayload{Workspace: gqlmodel.ToWorkspaceFromFlow(tempRes)}, nil
		}
	}
	log.Printf("WARNING:[mutationResolver.CreateWorkspace] Fallback to traditional usecase")

	res, err := usecases(ctx).Workspace.Create(ctx, input.Name, getUser(ctx).ID(), getAcOperator(ctx))
	if err != nil {
		return nil, err
	}

	return &gqlmodel.CreateWorkspacePayload{Workspace: gqlmodel.ToWorkspace(res)}, nil
}

func (r *mutationResolver) DeleteWorkspace(ctx context.Context, input gqlmodel.DeleteWorkspaceInput) (*gqlmodel.DeleteWorkspacePayload, error) {
	tid, err := gqlmodel.ToID[accountdomain.Workspace](input.WorkspaceID)
	if err != nil {
		return nil, err
	}

	if err := usecases(ctx).Workspace.Remove(ctx, tid, getAcOperator(ctx)); err != nil {
		return nil, err
	}

	return &gqlmodel.DeleteWorkspacePayload{WorkspaceID: input.WorkspaceID}, nil
}

func (r *mutationResolver) UpdateWorkspace(ctx context.Context, input gqlmodel.UpdateWorkspaceInput) (*gqlmodel.UpdateWorkspacePayload, error) {
	if usecases(ctx).TempNewWorkspace != nil {
		tempNewWorkspace := r.updateWorkspaceWithTempNewUsecase(ctx, input)
		if tempNewWorkspace != nil {
			log.Printf("DEBUG:[mutationResolver.updateWorkspaceWithTempNewUsecase] Updated workspace with tempNewUsecase")
			return tempNewWorkspace, nil
		}
	}
	log.Printf("WARNING:[mutationResolver.UpdateWorkspace] Fallback to traditional usecase")

	tid, err := gqlmodel.ToID[accountdomain.Workspace](input.WorkspaceID)
	if err != nil {
		return nil, err
	}

	res, err := usecases(ctx).Workspace.Update(ctx, tid, input.Name, getAcOperator(ctx))
	if err != nil {
		return nil, err
	}

	return &gqlmodel.UpdateWorkspacePayload{Workspace: gqlmodel.ToWorkspace(res)}, nil
}

func (r *mutationResolver) updateWorkspaceWithTempNewUsecase(ctx context.Context, input gqlmodel.UpdateWorkspaceInput) *gqlmodel.UpdateWorkspacePayload {
	tid, err := gqlmodel.ToID[id.Workspace](input.WorkspaceID)
	if err != nil {
		log.Printf("WARNING:[mutationResolver.updateWorkspaceWithTempNewUsecase] Failed to convert ID: %v", err)
		return nil
	}

	res, err := usecases(ctx).TempNewWorkspace.Update(ctx, tid, input.Name)
	if err != nil {
		log.Printf("WARNING:[mutationResolver.updateWorkspaceWithTempNewUsecase] Failed to update workspace: %v", err)
		return nil
	}

	return &gqlmodel.UpdateWorkspacePayload{Workspace: gqlmodel.ToWorkspaceFromFlow(res)}
}

func (r *mutationResolver) AddMemberToWorkspace(ctx context.Context, input gqlmodel.AddMemberToWorkspaceInput) (*gqlmodel.AddMemberToWorkspacePayload, error) {
	tid, uid, err := gqlmodel.ToID2[accountdomain.Workspace, accountdomain.User](input.WorkspaceID, input.UserID)
	if err != nil {
		return nil, err
	}

	res, err := usecases(ctx).Workspace.AddUserMember(ctx, tid, map[accountdomain.UserID]workspace.Role{uid: gqlmodel.FromRole(input.Role)}, getAcOperator(ctx))
	if err != nil {
		return nil, err
	}

	return &gqlmodel.AddMemberToWorkspacePayload{Workspace: gqlmodel.ToWorkspace(res)}, nil
}

func (r *mutationResolver) RemoveMemberFromWorkspace(ctx context.Context, input gqlmodel.RemoveMemberFromWorkspaceInput) (*gqlmodel.RemoveMemberFromWorkspacePayload, error) {
	tid, uid, err := gqlmodel.ToID2[accountdomain.Workspace, accountdomain.User](input.WorkspaceID, input.UserID)
	if err != nil {
		return nil, err
	}

	res, err := usecases(ctx).Workspace.RemoveUserMember(ctx, tid, uid, getAcOperator(ctx))
	if err != nil {
		return nil, err
	}

	return &gqlmodel.RemoveMemberFromWorkspacePayload{Workspace: gqlmodel.ToWorkspace(res)}, nil
}

func (r *mutationResolver) UpdateMemberOfWorkspace(ctx context.Context, input gqlmodel.UpdateMemberOfWorkspaceInput) (*gqlmodel.UpdateMemberOfWorkspacePayload, error) {
	tid, uid, err := gqlmodel.ToID2[accountdomain.Workspace, accountdomain.User](input.WorkspaceID, input.UserID)
	if err != nil {
		return nil, err
	}

	res, err := usecases(ctx).Workspace.UpdateUserMember(ctx, tid, uid, gqlmodel.FromRole(input.Role), getAcOperator(ctx))
	if err != nil {
		return nil, err
	}

	return &gqlmodel.UpdateMemberOfWorkspacePayload{Workspace: gqlmodel.ToWorkspace(res)}, nil
}
