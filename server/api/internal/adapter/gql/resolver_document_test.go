package gql

import (
	"context"
	"errors"
	"testing"

	"github.com/reearth/reearth-flow/api/internal/adapter"
	"github.com/reearth/reearth-flow/api/internal/adapter/gql/gqlmodel"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
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
