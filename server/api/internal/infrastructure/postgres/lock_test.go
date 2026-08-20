package postgres_test

import (
	"context"
	"testing"

	"github.com/reearth/reearth-flow/api/internal/infrastructure/postgres"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/postgres/pgtest"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestLock_LockUnlock(t *testing.T) {
	connect := pgtest.Connect(t)
	if connect == nil {
		t.Skip("no postgres connection")
	}
	pool := connect(t)
	ctx := context.Background()
	r := postgres.NewLock(pool)

	// First lock succeeds.
	require.NoError(t, r.Lock(ctx, "config"))

	// Second lock on same name returns ErrAlreadyLocked.
	err := r.Lock(ctx, "config")
	assert.ErrorIs(t, err, repo.ErrAlreadyLocked)

	// Unlock succeeds.
	require.NoError(t, r.Unlock(ctx, "config"))

	// Unlock again returns ErrNotLocked.
	err = r.Unlock(ctx, "config")
	assert.ErrorIs(t, err, repo.ErrNotLocked)
}
