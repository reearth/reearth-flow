package interactor

import (
	"context"

	gqlworkspace "github.com/reearth/reearth-accounts/server/pkg/gqlclient/workspace"
	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-accounts/server/pkg/role"
	accountsworkspace "github.com/reearth/reearth-accounts/server/pkg/workspace"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/samber/lo"
)

type Workspace struct {
	workspaceRepo gqlworkspace.WorkspaceRepo
}

func NewWorkspace(workspaceRepo gqlworkspace.WorkspaceRepo) interfaces.Workspace {
	return &Workspace{
		workspaceRepo: workspaceRepo,
	}
}

func (i *Workspace) FindByIDs(ctx context.Context, ids accountsid.WorkspaceIDList) (accountsworkspace.List, error) {
	return i.workspaceRepo.FindByIDs(ctx, ids.Strings())
}

func (i *Workspace) FindByUser(ctx context.Context, uid accountsid.UserID) (accountsworkspace.List, error) {
	return i.workspaceRepo.FindByUser(ctx, uid.String())
}

func (i *Workspace) Create(ctx context.Context, name string) (*accountsworkspace.Workspace, error) {
	return i.workspaceRepo.CreateWorkspace(ctx, gqlworkspace.CreateWorkspaceInput{
		Name:  name,
		Alias: name,
	})
}

func (i *Workspace) Update(ctx context.Context, wid accountsid.WorkspaceID, name string) (*accountsworkspace.Workspace, error) {
	return i.workspaceRepo.UpdateWorkspace(ctx, gqlworkspace.UpdateWorkspaceInput{
		WorkspaceID: wid.String(),
		Name:        name,
	})
}

func (i *Workspace) Delete(ctx context.Context, wid accountsid.WorkspaceID) error {
	return i.workspaceRepo.DeleteWorkspace(ctx, wid.String())
}

func (i *Workspace) AddUserMember(ctx context.Context, wid accountsid.WorkspaceID, users map[accountsid.UserID]role.RoleType) (*accountsworkspace.Workspace, error) {
	members := lo.MapToSlice(users, func(uid accountsid.UserID, r role.RoleType) gqlworkspace.MemberInput {
		return gqlworkspace.MemberInput{
			UserID: uid.String(),
			Role:   string(r),
		}
	})

	return i.workspaceRepo.AddUsersToWorkspace(ctx, gqlworkspace.AddUsersToWorkspaceInput{
		WorkspaceID: wid.String(),
		Users:       members,
	})
}

func (i *Workspace) UpdateUserMember(ctx context.Context, wid accountsid.WorkspaceID, uid accountsid.UserID, role role.RoleType) (*accountsworkspace.Workspace, error) {
	return i.workspaceRepo.UpdateUserOfWorkspace(ctx, gqlworkspace.UpdateUserOfWorkspaceInput{
		WorkspaceID: wid.String(),
		UserID:      uid.String(),
		Role:        string(role),
	})
}

func (i *Workspace) RemoveUserMember(ctx context.Context, wid accountsid.WorkspaceID, uid accountsid.UserID) (*accountsworkspace.Workspace, error) {
	return i.workspaceRepo.RemoveUserFromWorkspace(ctx, wid.String(), uid.String())
}
