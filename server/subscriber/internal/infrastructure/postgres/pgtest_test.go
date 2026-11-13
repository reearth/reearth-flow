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

// Valid IDs for the successful-read fixtures.
const (
	testWorkflowID = "5b0f6a1e-3c2d-4e8f-9a7b-1c2d3e4f5a6b"
	testNodeID1    = "7e1d2c3b-4a59-4687-b9a0-c1d2e3f4a5b6"
	testNodeID2    = "8f2e3d4c-5b6a-4798-80b1-d2e3f4a5b6c7"
)

func count(t *testing.T, pool *pgxpool.Pool, sql string, args ...any) int {
	t.Helper()
	var n int
	if err := pool.QueryRow(context.Background(), sql, args...).Scan(&n); err != nil {
		t.Fatalf("count: %v", err)
	}
	return n
}
