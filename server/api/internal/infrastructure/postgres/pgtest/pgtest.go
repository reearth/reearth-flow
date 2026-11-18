// Package pgtest wraps reearthx pgxtest with reearth-flow's migration application,
// so each test gets an isolated database with the triggers schema already applied.
package pgtest

import (
	"context"
	"testing"

	"github.com/jackc/pgx/v5/pgxpool"
	flowdb "github.com/reearth/reearth-flow/db"
	"github.com/reearth/reearthx/pgxx/pgxtest"
)

func init() {
	pgxtest.Env = "REEARTH_FLOW_DB_PG"
}

func Connect(t *testing.T) func(*testing.T) *pgxpool.Pool {
	t.Helper()
	// pgxtest.Connect skips the test via runtime.Goexit when REEARTH_FLOW_DB_PG is
	// unset, so it never returns nil and callers can write pgtest.Connect(t)(t).
	base := pgxtest.Connect(t)
	return func(t *testing.T) *pgxpool.Pool {
		t.Helper()
		pool := base(t)
		// The embedded migrations keep the schema independent of the test's working
		// directory.
		if err := pgxtest.ApplyFS(context.Background(), pool, flowdb.MigrationsFS); err != nil {
			t.Fatalf("pgtest: apply migrations: %v", err)
		}
		return pool
	}
}
