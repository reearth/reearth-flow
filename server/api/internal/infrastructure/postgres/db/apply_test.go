package db_test

import (
	"context"
	"testing"

	"github.com/jackc/pgx/v5/pgxpool"
	postgresdb "github.com/reearth/reearth-flow/api/internal/infrastructure/postgres/db"
	"github.com/reearth/reearthx/pgxx/pgxtest"
	"github.com/stretchr/testify/require"
)

func init() {
	pgxtest.Env = "REEARTH_FLOW_DB_PG"
}

func TestApply_CreatesSchema(t *testing.T) {
	connect := pgxtest.Connect(t)
	pool := connect(t)

	ctx := context.Background()
	require.NoError(t, postgresdb.Apply(ctx, pool))

	requireTable(t, ctx, pool, "jobs")
	requireTable(t, ctx, pool, "triggers")

	require.Error(t, postgresdb.Apply(ctx, pool))
}

func requireTable(t *testing.T, ctx context.Context, pool *pgxpool.Pool, name string) {
	t.Helper()
	var reg *string
	require.NoError(t, pool.QueryRow(ctx, "SELECT to_regclass('public."+name+"')::text").Scan(&reg))
	require.NotNil(t, reg, "table %q should exist after Apply", name)
}
