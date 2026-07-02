package postgres_test

import (
	"context"
	"testing"
	"time"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/postgres"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/postgres/pgtest"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/asset"
	"github.com/reearth/reearthx/pgxx"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func newUpload(wid accountsid.WorkspaceID, uuid string) *asset.Upload {
	return asset.NewUpload().
		UUID(uuid).
		Workspace(wid).
		FileName("test.zip").
		ContentType("application/zip").
		ContentEncoding("").
		ContentLength(4096).
		ExpiresAt(time.Now().Add(time.Hour)).
		Build()
}

func TestAssetUpload_Save_FindByID(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	r := postgres.NewAssetUpload(pgxx.NewClient(pool))

	wid := accountsid.NewWorkspaceID()
	u := newUpload(wid, "test-uuid-1")

	require.NoError(t, r.Save(ctx, u))
	got, err := r.FindByID(ctx, "test-uuid-1")
	require.NoError(t, err)
	assert.Equal(t, "test-uuid-1", got.UUID())
	assert.Equal(t, wid, got.Workspace())
	assert.Equal(t, "test.zip", got.FileName())
	assert.Equal(t, "application/zip", got.ContentType())
	assert.Equal(t, int64(4096), got.ContentLength())
}

func TestAssetUpload_FindByID_NotFound(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	got, err := postgres.NewAssetUpload(pgxx.NewClient(pool)).FindByID(context.Background(), "no-such-uuid")
	assert.Error(t, err)
	assert.Nil(t, got)
}

func TestAssetUpload_Save_CanWriteDenied(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	wid := accountsid.NewWorkspaceID()
	r := postgres.NewAssetUpload(pgxx.NewClient(pool)).Filtered(repo.WorkspaceFilter{
		Readable: accountsid.WorkspaceIDList{wid},
		Writable: accountsid.WorkspaceIDList{accountsid.NewWorkspaceID()},
	})
	u := newUpload(wid, "denied-uuid")
	err := r.Save(ctx, u)
	assert.ErrorIs(t, err, repo.ErrOperationDenied)
}

func TestAssetUpload_Filtered_ReadNoGate(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	wid := accountsid.NewWorkspaceID()
	base := postgres.NewAssetUpload(pgxx.NewClient(pool))

	u := newUpload(wid, "read-no-gate-uuid")
	require.NoError(t, base.Save(ctx, u))

	// Filter for a different workspace — FindByID has no read gate, so it should still work
	r := base.Filtered(repo.WorkspaceFilter{
		Readable: accountsid.WorkspaceIDList{accountsid.NewWorkspaceID()},
	})
	got, err := r.FindByID(ctx, "read-no-gate-uuid")
	require.NoError(t, err)
	assert.Equal(t, "read-no-gate-uuid", got.UUID())
}
