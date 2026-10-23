// Package accountsdb reads users and workspaces straight from the accounts
// database, for callers that name a user but hold none of that user's tokens,
// such as the internal API. The accounts GraphQL API answers only for the user
// whose token it is given. Everything here is read-only.
package accountsdb

import (
	"context"
	"errors"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	accountsrole "github.com/reearth/reearth-accounts/server/pkg/role"
	accountsuser "github.com/reearth/reearth-accounts/server/pkg/user"
	accountsworkspace "github.com/reearth/reearth-accounts/server/pkg/workspace"
	"github.com/reearth/reearthx/account/accountdomain/user"
	"github.com/reearth/reearthx/account/accountdomain/workspace"
	"github.com/reearth/reearthx/account/accountusecase/accountrepo"
	"github.com/reearth/reearthx/rerror"
)

// FindUser returns the user with the reearth-accounts ID, or
// rerror.ErrNotFound when there is none.
func FindUser(ctx context.Context, users accountrepo.User, userID string) (*accountsuser.User, error) {
	uid, err := user.IDFrom(userID)
	if err != nil {
		return nil, rerror.ErrNotFound
	}
	u, err := users.FindByID(ctx, uid)
	if err != nil {
		return nil, err
	}
	if u == nil {
		return nil, rerror.ErrNotFound
	}
	id, err := accountsid.UserIDFrom(u.ID().String())
	if err != nil {
		return nil, err
	}
	return accountsuser.New().
		ID(id).
		Name(u.Name()).
		Alias(u.Alias()).
		Email(u.Email()).
		Build()
}

func toWorkspace(w *workspace.Workspace) (*accountsworkspace.Workspace, error) {
	wid, err := accountsid.WorkspaceIDFrom(w.ID().String())
	if err != nil {
		return nil, err
	}
	members := map[accountsid.UserID]accountsworkspace.Member{}
	if w.Members() != nil {
		for uid, m := range w.Members().Users() {
			id, err := accountsid.UserIDFrom(uid.String())
			if err != nil {
				return nil, err
			}
			members[id] = accountsworkspace.Member{
				Role:     accountsrole.RoleType(m.Role),
				Disabled: m.Disabled,
				Host:     m.Host,
			}
		}
	}
	return accountsworkspace.New().
		ID(wid).
		Name(w.Name()).
		Alias(w.Alias()).
		Email(w.Email()).
		Personal(w.IsPersonal()).
		Members(members).
		Build()
}

var errReadOnly = errors.New("accountsdb: workspaces are read-only here")
