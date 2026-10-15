package permission

import (
	"context"
	"errors"
	"testing"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	accountsuser "github.com/reearth/reearth-accounts/server/pkg/user"
	"github.com/reearth/reearth-flow/api/internal/adapter"
	"github.com/reearth/reearth-flow/api/internal/rbac"
	xuser "github.com/reearth/reearthx/account/accountdomain/user"
	xworkspace "github.com/reearth/reearthx/account/accountdomain/workspace"
	"github.com/reearth/reearthx/rerror"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

type fakeWorkspaceFinder struct {
	ws  *xworkspace.Workspace
	err error
}

func (f *fakeWorkspaceFinder) FindByID(_ context.Context, id xworkspace.ID) (*xworkspace.Workspace, error) {
	if f.err != nil {
		return nil, f.err
	}
	if f.ws == nil || f.ws.ID() != id {
		return nil, rerror.ErrNotFound
	}
	return f.ws, nil
}

// localFixture returns a context carrying a user, and a workspace where that
// user is the given member.
func localFixture(t *testing.T, m xworkspace.Member) (context.Context, *xworkspace.Workspace) {
	t.Helper()
	u := accountsuser.New().NewID().Name("u").Email("u@example.com").MustBuild()
	uid := xuser.MustID(u.ID().String())
	ws := xworkspace.New().NewID().
		Members(map[xworkspace.UserID]xworkspace.Member{uid: m}).
		MustBuild()
	return adapter.AttachUser(context.Background(), u), ws
}

func workspaceID(t *testing.T, ws *xworkspace.Workspace) accountsid.WorkspaceID {
	t.Helper()
	return accountsid.MustWorkspaceID(ws.ID().String())
}

// TestLocalChecker_MatchesRoleRules checks every role against every rule, so
// the internal API answers exactly as the workspace-role guard on GraphQL.
func TestLocalChecker_MatchesRoleRules(t *testing.T) {
	for _, role := range []xworkspace.Role{xworkspace.RoleReader, xworkspace.RoleWriter, xworkspace.RoleMaintainer, xworkspace.RoleOwner} {
		ctx, ws := localFixture(t, xworkspace.Member{Role: role})
		c := NewLocalChecker(&fakeWorkspaceFinder{ws: ws})
		for _, r := range rbac.DefineResources() {
			for action := range r.Actions {
				got, err := c.CheckPermission(ctx, r.Resource, action, workspaceID(t, ws))
				require.NoError(t, err)
				assert.Equal(t, roleAllows(r.Resource, action, string(role)), got, "%s on %s/%s", role, r.Resource, action)
			}
		}
	}
}

func TestLocalChecker_Examples(t *testing.T) {
	ctx, ws := localFixture(t, xworkspace.Member{Role: xworkspace.RoleReader})
	c := NewLocalChecker(&fakeWorkspaceFinder{ws: ws})
	wid := workspaceID(t, ws)

	ok, err := c.CheckPermission(ctx, rbac.ResourceProject, rbac.ActionList, wid)
	require.NoError(t, err)
	assert.True(t, ok, "a reader lists projects")

	ok, err = c.CheckPermission(ctx, rbac.ResourceProject, rbac.ActionDelete, wid)
	require.NoError(t, err)
	assert.False(t, ok, "a reader cannot delete projects")

	ok, err = c.CheckPermission(ctx, rbac.ResourceDeployment, rbac.ActionAny, wid)
	require.NoError(t, err)
	assert.False(t, ok, "deployments need writer or above")
}

func TestLocalChecker_Denies(t *testing.T) {
	ctx, ws := localFixture(t, xworkspace.Member{Role: xworkspace.RoleOwner})
	wid := workspaceID(t, ws)
	c := NewLocalChecker(&fakeWorkspaceFinder{ws: ws})

	t.Run("no user", func(t *testing.T) {
		ok, err := c.CheckPermission(context.Background(), rbac.ResourceProject, rbac.ActionList, wid)
		require.NoError(t, err)
		assert.False(t, ok)
	})

	t.Run("no workspace", func(t *testing.T) {
		ok, err := c.CheckPermission(ctx, rbac.ResourceProject, rbac.ActionList)
		require.NoError(t, err)
		assert.False(t, ok)
	})

	t.Run("not a member", func(t *testing.T) {
		other, _ := localFixture(t, xworkspace.Member{Role: xworkspace.RoleOwner})
		ok, err := c.CheckPermission(other, rbac.ResourceProject, rbac.ActionList, wid)
		require.NoError(t, err)
		assert.False(t, ok)
	})

	t.Run("disabled member", func(t *testing.T) {
		ctx, ws := localFixture(t, xworkspace.Member{Role: xworkspace.RoleOwner, Disabled: true})
		ok, err := NewLocalChecker(&fakeWorkspaceFinder{ws: ws}).CheckPermission(ctx, rbac.ResourceProject, rbac.ActionList, workspaceID(t, ws))
		require.NoError(t, err)
		assert.False(t, ok)
	})

	t.Run("unknown workspace", func(t *testing.T) {
		ok, err := c.CheckPermission(ctx, rbac.ResourceProject, rbac.ActionList, accountsid.NewWorkspaceID())
		require.NoError(t, err)
		assert.False(t, ok)
	})

	t.Run("lookup error", func(t *testing.T) {
		boom := errors.New("boom")
		_, err := NewLocalChecker(&fakeWorkspaceFinder{err: boom}).CheckPermission(ctx, rbac.ResourceProject, rbac.ActionList, wid)
		assert.ErrorIs(t, err, boom)
	})
}
