package postgres

import (
	"context"
	"testing"

	"github.com/jackc/pgx/v5/pgxpool"
	flowdb "github.com/reearth/reearth-flow/db"
	"github.com/reearth/reearthx/pgxx/pgxtest"
)

// The tests run against the Postgres named by REEARTH_FLOW_DB_PG (the same
// variable server/api's tests use) and skip when it is unset: CI supplies a
// service container, and a developer without one still gets a green go test.
func init() {
	pgxtest.Env = "REEARTH_FLOW_DB_PG"
}

// newPool returns a pool on a database private to this test with the API's
// migrations applied. Per-test databases because the janitor deletes rows
// unqualified by job_id, so it would otherwise reach into a concurrent test.
func newPool(t *testing.T) *pgxpool.Pool {
	t.Helper()
	pool := pgxtest.Connect(t)(t)
	if err := pgxtest.ApplyFS(context.Background(), pool, flowdb.MigrationsFS); err != nil {
		t.Fatalf("apply migrations: %v", err)
	}
	return pool
}

func count(t *testing.T, pool *pgxpool.Pool, sql string, args ...any) int {
	t.Helper()
	var n int
	if err := pool.QueryRow(context.Background(), sql, args...).Scan(&n); err != nil {
		t.Fatalf("count: %v", err)
	}
	return n
}
