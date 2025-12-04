package interactor

import (
	"context"

	gqlworkspace "github.com/reearth/reearth-accounts/server/pkg/gqlclient/workspace"
	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	accountsworkspace "github.com/reearth/reearth-accounts/server/pkg/workspace"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
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
	return i.workspaceRepo.FindByIDs(ctx, ids)
}

func (i *Workspace) FindByUser(ctx context.Context, uid accountsid.UserID) (accountsworkspace.List, error) {
	return i.workspaceRepo.FindByUser(ctx, uid)
}

func (i *Workspace) Create(ctx context.Context, name string) (*accountsworkspace.Workspace, error) {
	return i.workspaceRepo.Create(ctx, name)
}

func (i *Workspace) Update(ctx context.Context, wid accountsid.WorkspaceID, name string) (*accountsworkspace.Workspace, error) {
	return i.workspaceRepo.Update(ctx, wid, name)
}

func (i *Workspace) Delete(ctx context.Context, wid accountsid.WorkspaceID) error {
	return i.workspaceRepo.Delete(ctx, wid)
}

func (i *Workspace) AddUserMember(ctx context.Context, wid accountsid.WorkspaceID, users map[accountsid.UserID]accountsworkspace.Role) (*accountsworkspace.Workspace, error) {
	return i.workspaceRepo.AddUserMember(ctx, wid, users)
}

func (i *Workspace) UpdateUserMember(ctx context.Context, wid accountsid.WorkspaceID, uid accountsid.UserID, role accountsworkspace.Role) (*accountsworkspace.Workspace, error) {
	return i.workspaceRepo.UpdateUserMember(ctx, wid, uid, role)
}

func (i *Workspace) RemoveUserMember(ctx context.Context, wid accountsid.WorkspaceID, uid accountsid.UserID) (*accountsworkspace.Workspace, error) {
	return i.workspaceRepo.RemoveUserMember(ctx, wid, uid)
}
