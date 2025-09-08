package interactor

import (
	"context"

	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/workspace"
)

type Workspace struct {
	workspaceRepo workspace.Repo
}

func NewWorkspace(workspaceRepo workspace.Repo) interfaces.Workspace {
	return &Workspace{
		workspaceRepo: workspaceRepo,
	}
}

func (i *Workspace) FindByIDs(ctx context.Context, ids id.WorkspaceIDList) (workspace.List, error) {
	return i.workspaceRepo.FindByIDs(ctx, ids)
}

func (i *Workspace) FindByUser(ctx context.Context, uid id.UserID) (workspace.List, error) {
	return i.workspaceRepo.FindByUser(ctx, uid)
}

func (i *Workspace) Create(ctx context.Context, name string) (*workspace.Workspace, error) {
	return i.workspaceRepo.Create(ctx, name)
}

func (i *Workspace) Update(ctx context.Context, wid id.WorkspaceID, name string) (*workspace.Workspace, error) {
	return i.workspaceRepo.Update(ctx, wid, name)
}

func (i *Workspace) Delete(ctx context.Context, wid id.WorkspaceID) error {
	return i.workspaceRepo.Delete(ctx, wid)
}

func (i *Workspace) AddUserMember(ctx context.Context, wid id.WorkspaceID, users map[id.UserID]workspace.Role) (*workspace.Workspace, error) {
	return i.workspaceRepo.AddUserMember(ctx, wid, users)
}

func (i *Workspace) UpdateUserMember(ctx context.Context, wid id.WorkspaceID, uid id.UserID, role workspace.Role) (*workspace.Workspace, error) {
	return i.workspaceRepo.UpdateUserMember(ctx, wid, uid, role)
}

func (i *Workspace) RemoveUserMember(ctx context.Context, wid id.WorkspaceID, uid id.UserID) (*workspace.Workspace, error) {
	return i.workspaceRepo.RemoveUserMember(ctx, wid, uid)
}
