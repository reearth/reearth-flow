package postgres_test

import (
	"context"
	"testing"

	"github.com/reearth/reearth-flow/api/internal/infrastructure/postgres/pgtest"
	"github.com/reearth/reearthx/authserver"
	"github.com/reearth/reearthx/pgxx"
	"github.com/reearth/reearthx/rerror"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// TestAuthRequest_AtlasSchema runs the reearthx authserver.Postgres repo against
// flow's Atlas-migrated auth_requests table, without the library's own Init.
// It fails if the Atlas DDL and the reearthx repo drift apart.
func TestAuthRequest_AtlasSchema(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	r := authserver.NewPostgres(pgxx.NewClient(pool))

	id := authserver.NewRequestID()
	_, err := r.FindByID(ctx, id)
	assert.Same(t, rerror.ErrNotFound, err)

	req := authserver.NewRequest().ID(id).Code("code123").Subject("sub").MustBuild()
	require.NoError(t, r.Save(ctx, req))

	got, err := r.FindByID(ctx, id)
	require.NoError(t, err)
	assert.Equal(t, id, got.ID())

	got, err = r.FindByCode(ctx, "code123")
	require.NoError(t, err)
	assert.Equal(t, id, got.ID())

	got, err = r.FindBySubject(ctx, "sub")
	require.NoError(t, err)
	assert.Equal(t, id, got.ID())

	require.NoError(t, r.Remove(ctx, id))
	_, err = r.FindByID(ctx, id)
	assert.Same(t, rerror.ErrNotFound, err)
}
