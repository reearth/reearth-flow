package accountsdb

import (
	"context"

	gqlworkspace "github.com/reearth/reearth-accounts/server/pkg/gqlclient/workspace"
	accountsworkspace "github.com/reearth/reearth-accounts/server/pkg/workspace"
	"github.com/reearth/reearthx/account/accountdomain"
	"github.com/reearth/reearthx/account/accountdomain/user"
	"github.com/reearth/reearthx/account/accountdomain/workspace"
	"github.com/reearth/reearthx/account/accountusecase/accountrepo"
	"github.com/reearth/reearthx/idx"
	"github.com/reearth/reearthx/rerror"
)

// Workspaces serves the read methods of the accounts GraphQL workspace
// repository from the accounts database. Every write returns an error.
type Workspaces struct {
	repo accountrepo.Workspace
}

var _ gqlworkspace.WorkspaceRepo = (*Workspaces)(nil)

func NewWorkspaces(repo accountrepo.Workspace) *Workspaces {
	return &Workspaces{repo: repo}
}

func (w *Workspaces) FindByID(ctx context.Context, id string) (*accountsworkspace.Workspace, error) {
	wid, err := workspace.IDFrom(id)
	if err != nil {
		return nil, rerror.ErrNotFound
	}
	ws, err := w.repo.FindByID(ctx, wid)
	if err != nil {
		return nil, err
	}
	return toWorkspace(ws)
}

func (w *Workspaces) FindByIDs(ctx context.Context, ids []string) (accountsworkspace.List, error) {
	wids, err := idx.ListFrom[accountdomain.Workspace](ids)
	if err != nil {
		return nil, err
	}
	list, err := w.repo.FindByIDs(ctx, wids)
	if err != nil {
		return nil, err
	}
	return toWorkspaces(list)
}

func (w *Workspaces) FindByAlias(ctx context.Context, alias string) (*accountsworkspace.Workspace, error) {
	ws, err := w.repo.FindByAlias(ctx, alias)
	if err != nil {
		return nil, err
	}
	return toWorkspace(ws)
}

func (w *Workspaces) FindByUser(ctx context.Context, userID string) (accountsworkspace.List, error) {
	uid, err := user.IDFrom(userID)
	if err != nil {
		return nil, err
	}
	list, err := w.repo.FindByUser(ctx, uid)
	if err != nil {
		return nil, err
	}
	return toWorkspaces(list)
}

func (w *Workspaces) FindByUserWithPagination(context.Context, string, int64, int64) (accountsworkspace.List, int, error) {
	return nil, 0, errReadOnly
}

func (w *Workspaces) CreateWorkspace(context.Context, gqlworkspace.CreateWorkspaceInput) (*accountsworkspace.Workspace, error) {
	return nil, errReadOnly
}

func (w *Workspaces) UpdateWorkspace(context.Context, gqlworkspace.UpdateWorkspaceInput) (*accountsworkspace.Workspace, error) {
	return nil, errReadOnly
}

func (w *Workspaces) DeleteWorkspace(context.Context, string) error {
	return errReadOnly
}

func (w *Workspaces) AddUsersToWorkspace(context.Context, gqlworkspace.AddUsersToWorkspaceInput) (*accountsworkspace.Workspace, error) {
	return nil, errReadOnly
}

func (w *Workspaces) RemoveUserFromWorkspace(context.Context, string, string) (*accountsworkspace.Workspace, error) {
	return nil, errReadOnly
}

func (w *Workspaces) UpdateUserOfWorkspace(context.Context, gqlworkspace.UpdateUserOfWorkspaceInput) (*accountsworkspace.Workspace, error) {
	return nil, errReadOnly
}

func (w *Workspaces) TransferOwnership(context.Context, string, string) (*accountsworkspace.Workspace, error) {
	return nil, errReadOnly
}

func toWorkspaces(list workspace.List) (accountsworkspace.List, error) {
	res := make(accountsworkspace.List, 0, len(list))
	for _, ws := range list {
		if ws == nil {
			continue
		}
		w, err := toWorkspace(ws)
		if err != nil {
			return nil, err
		}
		res = append(res, w)
	}
	return res, nil
}
