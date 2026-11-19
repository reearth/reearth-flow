package gql

import (
	"context"
	"errors"
	"testing"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	accountsuser "github.com/reearth/reearth-accounts/server/pkg/user"
	"github.com/reearth/reearth-flow/api/internal/adapter/gql/gqlmodel"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// stubUserUsecase answers FindByIDs with canned users; other methods panic.
type stubUserUsecase struct {
	interfaces.User
	err   error
	users accountsuser.List
}

func (s *stubUserUsecase) FindByIDs(context.Context, accountsid.UserIDList) (accountsuser.List, error) {
	return s.users, s.err
}

func newLoaderTestUser(t *testing.T, name string) *accountsuser.User {
	t.Helper()

	return accountsuser.New().
		NewID().
		Name(name).
		Email(name + "@example.com").
		Workspace(accountsuser.NewWorkspaceID()).
		MustBuild()
}

func TestUserLoader_Fetch(t *testing.T) {
	t.Parallel()

	t.Run("aligns users with keys whatever order they come back in", func(t *testing.T) {
		t.Parallel()

		alice, bob := newLoaderTestUser(t, "alice"), newLoaderTestUser(t, "bob")
		loader := NewUserLoader(&stubUserUsecase{users: accountsuser.List{bob, alice}})

		got, errs := loader.Fetch(context.Background(), []gqlmodel.ID{
			gqlmodel.ID(alice.ID().String()),
			gqlmodel.ID(bob.ID().String()),
		})

		assert.Empty(t, errs)
		require.Len(t, got, 2)
		assert.Equal(t, "alice", got[0].Name)
		assert.Equal(t, "bob", got[1].Name)
	})

	t.Run("leaves nil for a user that does not resolve", func(t *testing.T) {
		t.Parallel()

		alice, bob := newLoaderTestUser(t, "alice"), newLoaderTestUser(t, "bob")
		loader := NewUserLoader(&stubUserUsecase{users: accountsuser.List{alice, bob}})

		got, errs := loader.Fetch(context.Background(), []gqlmodel.ID{
			gqlmodel.ID(alice.ID().String()),
			gqlmodel.ID(accountsid.NewUserID().String()),
			gqlmodel.ID(bob.ID().String()),
		})

		assert.Empty(t, errs)
		require.Len(t, got, 3)
		assert.Equal(t, "alice", got[0].Name)
		assert.Nil(t, got[1])
		assert.Equal(t, "bob", got[2].Name)
	})

	t.Run("returns the lookup error", func(t *testing.T) {
		t.Parallel()

		loader := NewUserLoader(&stubUserUsecase{err: errors.New("accounts down")})

		_, errs := loader.Fetch(context.Background(), []gqlmodel.ID{gqlmodel.ID(accountsid.NewUserID().String())})

		require.Len(t, errs, 1)
		assert.EqualError(t, errs[0], "accounts down")
	})
}
