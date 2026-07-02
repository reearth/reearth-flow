package postgres_test

import (
	"context"
	"testing"

	"github.com/reearth/reearth-flow/api/internal/infrastructure/postgres"
	pgmigration "github.com/reearth/reearth-flow/api/internal/infrastructure/postgres/migration"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/postgres/pgtest"
	"github.com/reearth/reearthx/pgxx"
	"github.com/reearth/reearthx/usecasex/migration"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestPostgresMigration_Do_NoMigrations(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	client := pgxx.NewClient(pool)
	cfg := postgres.NewConfig(client, postgres.NewLock(pool))

	// Empty package migrations: a no-op that still exercises lock/current/unlock.
	require.NoError(t, pgmigration.Do(ctx, client, cfg))
}

func TestPostgresMigration_RunnerAppliesAndTracks(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	client := pgxx.NewClient(pool)
	cfg := postgres.NewConfig(client, postgres.NewLock(pool))

	ran := 0
	migs := migration.Migrations[*pgxx.Client]{
		1: func(ctx context.Context, c *pgxx.Client) error {
			ran++
			_, err := c.DB(ctx).Exec(ctx, `CREATE TABLE mig_probe (id integer)`)
			return err
		},
	}
	runner := migration.NewRunner[*pgxx.Client](client, client, pgmigration.NewConfig(cfg), migs)

	require.NoError(t, runner.Migrate(ctx))
	assert.Equal(t, 1, ran)

	// version persisted via the config row
	got, err := cfg.LockAndLoad(ctx)
	require.NoError(t, err)
	assert.Equal(t, int64(1), got.Migration)
	require.NoError(t, cfg.Unlock(ctx))

	// re-run is a no-op (already at version 1)
	require.NoError(t, runner.Migrate(ctx))
	assert.Equal(t, 1, ran)
}
