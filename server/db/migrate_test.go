package db_test

import (
	"context"
	"io/fs"
	"sort"
	"testing"

	"github.com/jackc/pgx/v5/pgxpool"
	postgresdb "github.com/reearth/reearth-flow/db"
	"github.com/reearth/reearthx/pgxx/pgxtest"
	"github.com/stretchr/testify/require"
)

func TestMigrate_FreshDatabase(t *testing.T) {
	connect := pgxtest.Connect(t)
	pool := connect(t)
	ctx := context.Background()

	require.NoError(t, postgresdb.Migrate(ctx, pool))
	requireTable(t, ctx, pool, "projects")
	requireColumn(t, ctx, pool, "projects", "alias")
	require.Equal(t, len(migrationFiles(t)), countRevisions(t, ctx, pool))

	// A second start applies nothing.
	require.NoError(t, postgresdb.Migrate(ctx, pool))
	require.Equal(t, len(migrationFiles(t)), countRevisions(t, ctx, pool))
}

// A database seeded by Apply before the project metadata migration existed: the
// state that made checkProjectAlias fail with `column "alias" does not exist`.
func TestMigrate_BaselinesUntrackedDatabase(t *testing.T) {
	connect := pgxtest.Connect(t)
	pool := connect(t)
	ctx := context.Background()

	for _, f := range migrationFiles(t) {
		if f >= "migrations/20261014020624_add_project_metadata.sql" {
			break
		}
		b, err := fs.ReadFile(postgresdb.MigrationsFS, f)
		require.NoError(t, err)
		_, err = pool.Exec(ctx, string(b))
		require.NoError(t, err, f)
	}
	_, err := pool.Exec(ctx, `INSERT INTO projects (id, workspace_id, updated_at) VALUES ('p1', 'w1', now())`)
	require.NoError(t, err)

	require.NoError(t, postgresdb.Migrate(ctx, pool))
	requireColumn(t, ctx, pool, "projects", "alias")
	require.Equal(t, len(migrationFiles(t)), countRevisions(t, ctx, pool))

	var alias string
	require.NoError(t, pool.QueryRow(ctx, `SELECT alias FROM projects WHERE id = 'p1'`).Scan(&alias))
	require.Equal(t, "p-p1", alias)
}

func migrationFiles(t *testing.T) []string {
	t.Helper()
	files, err := fs.Glob(postgresdb.MigrationsFS, "migrations/*.sql")
	require.NoError(t, err)
	sort.Strings(files)
	return files
}

func countRevisions(t *testing.T, ctx context.Context, pool *pgxpool.Pool) int {
	t.Helper()
	var n int
	require.NoError(t, pool.QueryRow(ctx, `SELECT count(*) FROM schema_migrations`).Scan(&n))
	return n
}

func requireColumn(t *testing.T, ctx context.Context, pool *pgxpool.Pool, table, column string) {
	t.Helper()
	var exists bool
	require.NoError(t, pool.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM information_schema.columns
		WHERE table_schema = 'public' AND table_name = $1 AND column_name = $2)`, table, column).Scan(&exists))
	require.True(t, exists, "column %s.%s should exist", table, column)
}
