package permission

import (
	"context"
	"errors"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/internal/adapter"
	"github.com/reearth/reearthx/account/accountdomain/user"
	"github.com/reearth/reearthx/account/accountdomain/workspace"
	"github.com/reearth/reearthx/rerror"
)

// WorkspaceFinder reads a workspace, with its members, from the accounts database.
type WorkspaceFinder interface {
	FindByID(context.Context, workspace.ID) (*workspace.Workspace, error)
}

// LocalChecker decides permissions from the caller's role in the workspace as
// stored in the accounts database. It serves callers that name a user but hold
// none of that user's tokens, such as the internal API: the accounts service
// can only check permissions for the user whose token it is given.
//
// It applies roleAllows, the same rules the workspace-role guard in checker
// applies after Cerbos. Anyone who is not an active member of the workspace is
// denied, as is a check that names no workspace.
type LocalChecker struct {
	workspaces WorkspaceFinder
}

func NewLocalChecker(workspaces WorkspaceFinder) *LocalChecker {
	return &LocalChecker{workspaces: workspaces}
}

func (c *LocalChecker) CheckPermission(ctx context.Context, resource, action string, workspaceID ...accountsid.WorkspaceID) (bool, error) {
	u := adapter.User(ctx)
	if u == nil || len(workspaceID) != 1 {
		return false, nil
	}

	wid, err := workspace.IDFrom(workspaceID[0].String())
	if err != nil {
		return false, nil
	}
	uid, err := user.IDFrom(u.ID().String())
	if err != nil {
		return false, nil
	}

	ws, err := c.workspaces.FindByID(ctx, wid)
	if errors.Is(err, rerror.ErrNotFound) {
		return false, nil
	}
	if err != nil {
		return false, err
	}
	if ws == nil || ws.Members() == nil {
		return false, nil
	}

	m := ws.Members().User(uid)
	if m == nil || m.Disabled {
		return false, nil
	}
	return roleAllows(resource, action, string(m.Role)), nil
}
