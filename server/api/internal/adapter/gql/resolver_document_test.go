package gql

import (
	"context"
	"errors"
	"testing"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	accountsuser "github.com/reearth/reearth-accounts/server/pkg/user"
	"github.com/reearth/reearth-flow/api/internal/adapter"
	"github.com/reearth/reearth-flow/api/internal/adapter/gql/gqlmodel"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/pkg/websocket"
	"github.com/samber/lo"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// projectSnapshot serves two addressing modes through one field, which the schema
// cannot constrain to exactly one. These guards are that constraint, so they are
// asserted here rather than left to the caller.
//
// Both reject before any usecase call, so a nil resolver is enough: reaching the
// websocket client would panic and fail the test.
func TestProjectSnapshot_RejectsAmbiguousArguments(t *testing.T) {
	r := &queryResolver{}

	t.Run("both arguments", func(t *testing.T) {
		_, err := r.ProjectSnapshot(context.Background(), gqlmodel.ID("p1"), lo.ToPtr(1), lo.ToPtr(2))
		assert.ErrorContains(t, err, "not both")
	})

	t.Run("neither argument", func(t *testing.T) {
		_, err := r.ProjectSnapshot(context.Background(), gqlmodel.ID("p1"), nil, nil)
		assert.ErrorContains(t, err, "either version or snapshotNumber")
	})
}

// saveClient is a WebsocketClient whose save path is scripted; the other
// methods panic through the nil embedded interface.
type saveClient struct {
	interfaces.WebsocketClient
	flushErr, versionErr error
	// unchanged makes SaveVersion find nothing new since the last version.
	unchanged bool
	versions  []string
}

func (c *saveClient) FlushToGCS(context.Context, string) error { return c.flushErr }
func (c *saveClient) SaveVersion(_ context.Context, docID, label string) (bool, error) {
	c.versions = append(c.versions, docID+" "+label)
	return c.versionErr == nil && !c.unchanged, c.versionErr
}

func saveSnapshot(t *testing.T, c *saveClient) (bool, error) {
	t.Helper()
	ctx := adapter.AttachUsecases(context.Background(), &interfaces.Container{Websocket: c})
	return (&mutationResolver{}).SaveSnapshot(ctx, gqlmodel.ID("p1"))
}

func TestSaveSnapshot_SavesAVersion(t *testing.T) {
	t.Parallel()

	t.Run("saves a save version", func(t *testing.T) {
		t.Parallel()
		c := &saveClient{}
		ok, err := saveSnapshot(t, c)
		require.NoError(t, err)
		assert.True(t, ok)
		assert.Equal(t, []string{"p1 save"}, c.versions)
	})

	t.Run("nothing new to save answers false", func(t *testing.T) {
		t.Parallel()
		c := &saveClient{unchanged: true}
		ok, err := saveSnapshot(t, c)
		require.NoError(t, err)
		assert.False(t, ok)
		assert.Equal(t, []string{"p1 save"}, c.versions)
	})

	t.Run("another save in progress is not an error", func(t *testing.T) {
		t.Parallel()
		ok, err := saveSnapshot(t, &saveClient{versionErr: interfaces.ErrVersionSaveBusy})
		require.NoError(t, err)
		assert.True(t, ok)
	})

	t.Run("a server without versions is not an error", func(t *testing.T) {
		t.Parallel()
		ok, err := saveSnapshot(t, &saveClient{versionErr: interfaces.ErrVersionsUnsupported})
		require.NoError(t, err)
		assert.True(t, ok)
	})

	t.Run("a failed save is reported", func(t *testing.T) {
		t.Parallel()
		ok, err := saveSnapshot(t, &saveClient{versionErr: errors.New("websocket down")})
		assert.Error(t, err)
		assert.False(t, ok)
	})

	t.Run("a failed flush saves no version", func(t *testing.T) {
		t.Parallel()
		c := &saveClient{flushErr: errors.New("websocket down")}
		_, err := saveSnapshot(t, c)
		assert.Error(t, err)
		assert.Empty(t, c.versions)
	})
}

// snapshotResolverContext serves the user data loader from users through the
// real UserLoader, so ids that do not resolve behave as they do in production.
func snapshotResolverContext(t *testing.T, uc *stubUserUsecase) context.Context {
	t.Helper()
	return context.WithValue(context.Background(), contextDataloaders, &DataLoaders{
		User: NewUserLoader(uc).OrdinaryDataLoader(context.Background()),
	})
}

func TestToNamedSnapshot(t *testing.T) {
	t.Parallel()

	t.Run("carries attribution", func(t *testing.T) {
		t.Parallel()

		got := toNamedSnapshot(&websocket.SnapshotMetadata{
			ID:               4,
			Label:            "save",
			Contributors:     []string{"u1", "u2"},
			ContributorCount: 5,
			LastEditor:       "u2",
		})

		assert.Equal(t, 4, got.SnapshotNumber)
		assert.Equal(t, []gqlmodel.ID{"u1", "u2"}, got.ContributorIds)
		assert.Equal(t, 5, got.ContributorCount)
		assert.Equal(t, lo.ToPtr(gqlmodel.ID("u2")), got.LastEditorID)
	})

	t.Run("an unattributed snapshot has empty attribution", func(t *testing.T) {
		t.Parallel()

		got := toNamedSnapshot(&websocket.SnapshotMetadata{ID: 1, Label: "Before restore"})

		assert.NotNil(t, got.ContributorIds)
		assert.Empty(t, got.ContributorIds)
		assert.Zero(t, got.ContributorCount)
		assert.Nil(t, got.LastEditorID)
	})
}

func TestNamedSnapshotResolver_Contributors(t *testing.T) {
	t.Parallel()

	t.Run("leaves out contributors that are no longer users", func(t *testing.T) {
		t.Parallel()

		alice, bob := newLoaderTestUser(t, "alice"), newLoaderTestUser(t, "bob")
		ctx := snapshotResolverContext(t, &stubUserUsecase{users: accountsuser.List{bob, alice}})

		got, err := (&namedSnapshotResolver{}).Contributors(ctx, &gqlmodel.NamedSnapshot{
			ContributorIds: []gqlmodel.ID{
				gqlmodel.ID(alice.ID().String()),
				gqlmodel.ID(accountsid.NewUserID().String()),
				"not-a-user-id",
				gqlmodel.ID(bob.ID().String()),
			},
			ContributorCount: 4,
		})

		require.NoError(t, err)
		require.Len(t, got, 2)
		assert.Equal(t, "alice", got[0].Name)
		assert.Equal(t, "bob", got[1].Name)
	})

	t.Run("a failed lookup leaves the list empty without failing", func(t *testing.T) {
		t.Parallel()

		ctx := snapshotResolverContext(t, &stubUserUsecase{err: errors.New("accounts down")})

		got, err := (&namedSnapshotResolver{}).Contributors(ctx, &gqlmodel.NamedSnapshot{
			ContributorIds: []gqlmodel.ID{gqlmodel.ID(accountsid.NewUserID().String())},
		})

		require.NoError(t, err)
		assert.NotNil(t, got)
		assert.Empty(t, got)
	})

	t.Run("an unattributed snapshot looks nothing up", func(t *testing.T) {
		t.Parallel()

		// A nil User loader panics if reached.
		ctx := context.WithValue(context.Background(), contextDataloaders, &DataLoaders{})

		got, err := (&namedSnapshotResolver{}).Contributors(ctx, &gqlmodel.NamedSnapshot{ContributorIds: []gqlmodel.ID{}})

		require.NoError(t, err)
		assert.NotNil(t, got)
		assert.Empty(t, got)
	})
}

func TestNamedSnapshotResolver_LastEditor(t *testing.T) {
	t.Parallel()

	t.Run("resolves the last editor", func(t *testing.T) {
		t.Parallel()

		alice := newLoaderTestUser(t, "alice")
		ctx := snapshotResolverContext(t, &stubUserUsecase{users: accountsuser.List{alice}})

		got, err := (&namedSnapshotResolver{}).LastEditor(ctx, &gqlmodel.NamedSnapshot{
			LastEditorID: lo.ToPtr(gqlmodel.ID(alice.ID().String())),
		})

		require.NoError(t, err)
		require.NotNil(t, got)
		assert.Equal(t, "alice", got.Name)
	})

	t.Run("is nil for a user that no longer exists", func(t *testing.T) {
		t.Parallel()

		ctx := snapshotResolverContext(t, &stubUserUsecase{})

		got, err := (&namedSnapshotResolver{}).LastEditor(ctx, &gqlmodel.NamedSnapshot{
			LastEditorID: lo.ToPtr(gqlmodel.ID(accountsid.NewUserID().String())),
		})

		require.NoError(t, err)
		assert.Nil(t, got)
	})

	t.Run("is nil when the lookup fails", func(t *testing.T) {
		t.Parallel()

		ctx := snapshotResolverContext(t, &stubUserUsecase{err: errors.New("accounts down")})

		got, err := (&namedSnapshotResolver{}).LastEditor(ctx, &gqlmodel.NamedSnapshot{
			LastEditorID: lo.ToPtr(gqlmodel.ID(accountsid.NewUserID().String())),
		})

		require.NoError(t, err)
		assert.Nil(t, got)
	})

	t.Run("is nil without a last editor", func(t *testing.T) {
		t.Parallel()

		ctx := context.WithValue(context.Background(), contextDataloaders, &DataLoaders{})

		got, err := (&namedSnapshotResolver{}).LastEditor(ctx, &gqlmodel.NamedSnapshot{})

		require.NoError(t, err)
		assert.Nil(t, got)
	})
}
