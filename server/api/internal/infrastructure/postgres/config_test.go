package postgres_test

import (
	"context"
	"testing"

	"github.com/reearth/reearth-flow/api/internal/infrastructure/postgres"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/postgres/pgtest"
	"github.com/reearth/reearth-flow/api/pkg/config"
	"github.com/reearth/reearthx/pgxx"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func newConfigRepo(t *testing.T) *postgres.Config {
	t.Helper()
	connect := pgtest.Connect(t)
	if connect == nil {
		t.Skip("no postgres connection")
	}
	pool := connect(t)
	lock := postgres.NewLock(pool)
	return postgres.NewConfig(pgxx.NewClient(pool), lock)
}

func TestConfig_LockAndLoad_EmptyDB(t *testing.T) {
	r := newConfigRepo(t)
	ctx := context.Background()

	cfg, err := r.LockAndLoad(ctx)
	require.NoError(t, err)
	require.NotNil(t, cfg)
	assert.Equal(t, int64(0), cfg.Migration)
	assert.Nil(t, cfg.Auth)

	// Release the lock so subsequent calls can proceed.
	require.NoError(t, r.Unlock(ctx))
}

func TestConfig_Save_And_Reload(t *testing.T) {
	connect := pgtest.Connect(t)
	if connect == nil {
		t.Skip("no postgres connection")
	}
	pool := connect(t)
	ctx := context.Background()

	// Write via first repo instance.
	r1 := postgres.NewConfig(pgxx.NewClient(pool), postgres.NewLock(pool))
	cfg1, err := r1.LockAndLoad(ctx)
	require.NoError(t, err)
	cfg1.Migration = 5
	cfg1.Auth = &config.Auth{Cert: "c", Key: "k"}
	require.NoError(t, r1.SaveAndUnlock(ctx, cfg1))

	// Read back via a second repo instance.
	r2 := postgres.NewConfig(pgxx.NewClient(pool), postgres.NewLock(pool))
	cfg2, err := r2.LockAndLoad(ctx)
	require.NoError(t, err)
	require.NotNil(t, cfg2)
	assert.Equal(t, int64(5), cfg2.Migration)
	require.NotNil(t, cfg2.Auth)
	assert.Equal(t, "c", cfg2.Auth.Cert)
	assert.Equal(t, "k", cfg2.Auth.Key)
	require.NoError(t, r2.Unlock(ctx))
}

func TestConfig_SaveAuth(t *testing.T) {
	connect := pgtest.Connect(t)
	if connect == nil {
		t.Skip("no postgres connection")
	}
	pool := connect(t)
	ctx := context.Background()

	r1 := postgres.NewConfig(pgxx.NewClient(pool), postgres.NewLock(pool))
	cfg, err := r1.LockAndLoad(ctx)
	require.NoError(t, err)
	require.NoError(t, r1.SaveAndUnlock(ctx, cfg))

	// SaveAuth updates auth without touching migration.
	r2 := postgres.NewConfig(pgxx.NewClient(pool), postgres.NewLock(pool))
	require.NoError(t, r2.SaveAuth(ctx, &config.Auth{Cert: "cert2", Key: "key2"}))

	r3 := postgres.NewConfig(pgxx.NewClient(pool), postgres.NewLock(pool))
	cfg3, err := r3.LockAndLoad(ctx)
	require.NoError(t, err)
	require.NotNil(t, cfg3.Auth)
	assert.Equal(t, "cert2", cfg3.Auth.Cert)
	assert.Equal(t, "key2", cfg3.Auth.Key)
	require.NoError(t, r3.Unlock(ctx))
}

func TestConfig_SaveAndUnlock_ReleasesLock(t *testing.T) {
	connect := pgtest.Connect(t)
	if connect == nil {
		t.Skip("no postgres connection")
	}
	pool := connect(t)
	ctx := context.Background()

	r1 := postgres.NewConfig(pgxx.NewClient(pool), postgres.NewLock(pool))
	cfg, err := r1.LockAndLoad(ctx)
	require.NoError(t, err)
	cfg.Migration = 7
	require.NoError(t, r1.SaveAndUnlock(ctx, cfg))

	// Lock is released; a second repo instance should be able to LockAndLoad.
	r2 := postgres.NewConfig(pgxx.NewClient(pool), postgres.NewLock(pool))
	cfg2, err := r2.LockAndLoad(ctx)
	require.NoError(t, err)
	assert.Equal(t, int64(7), cfg2.Migration)
	require.NoError(t, r2.Unlock(ctx))
}
