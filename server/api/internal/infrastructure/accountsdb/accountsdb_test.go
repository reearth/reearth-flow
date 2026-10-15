package accountsdb

import (
	"context"
	"testing"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	accountsrole "github.com/reearth/reearth-accounts/server/pkg/role"
	"github.com/reearth/reearthx/account/accountdomain/user"
	"github.com/reearth/reearthx/account/accountdomain/workspace"
	"github.com/reearth/reearthx/account/accountinfrastructure/accountmemory"
	"github.com/reearth/reearthx/rerror"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestFindUser(t *testing.T) {
	ctx := context.Background()
	users := accountmemory.NewUser()
	u := user.New().NewID().Name("kim").Alias("kim").Email("kim@example.com").Workspace(workspace.NewID()).MustBuild()
	require.NoError(t, users.Save(ctx, u))

	got, err := FindUser(ctx, users, u.ID().String())
	require.NoError(t, err)
	assert.Equal(t, u.ID().String(), got.ID().String())
	assert.Equal(t, "kim", got.Name())
	assert.Equal(t, "kim@example.com", got.Email())

	_, err = FindUser(ctx, users, accountsid.NewUserID().String())
	assert.ErrorIs(t, err, rerror.ErrNotFound)
	_, err = FindUser(ctx, users, "not-an-id")
	assert.ErrorIs(t, err, rerror.ErrNotFound)
}

func TestWorkspaces(t *testing.T) {
	ctx := context.Background()
	repo := accountmemory.NewWorkspace()
	uid := user.NewID()
	ws := workspace.New().NewID().Name("team").Alias("team").
		Members(map[user.ID]workspace.Member{uid: {Role: workspace.RoleWriter, Disabled: true}}).
		MustBuild()
	require.NoError(t, repo.Save(ctx, ws))
	w := NewWorkspaces(repo)

	got, err := w.FindByID(ctx, ws.ID().String())
	require.NoError(t, err)
	assert.Equal(t, ws.ID().String(), got.ID().String())
	assert.Equal(t, "team", got.Alias())
	m := got.Members().User(accountsid.MustUserID(uid.String()))
	require.NotNil(t, m)
	assert.Equal(t, accountsrole.RoleWriter, m.Role)
	assert.True(t, m.Disabled)

	byAlias, err := w.FindByAlias(ctx, "team")
	require.NoError(t, err)
	assert.Equal(t, got.ID(), byAlias.ID())

	list, err := w.FindByUser(ctx, uid.String())
	require.NoError(t, err)
	assert.Len(t, list, 1)

	_, err = w.FindByID(ctx, "not-an-id")
	assert.ErrorIs(t, err, rerror.ErrNotFound)

	assert.ErrorIs(t, w.DeleteWorkspace(ctx, ws.ID().String()), errReadOnly)
}
