// Package pgtest wraps reearthx pgxtest with reearth-flow's migration application,
// so each test gets an isolated database with the triggers schema already applied.
package pgtest

import (
	"context"
	"os"
	"path/filepath"
	"testing"

	"github.com/jackc/pgx/v5/pgxpool"
	"github.com/reearth/reearthx/pgxx/pgxtest"
)

func init() {
	pgxtest.Env = "REEARTH_FLOW_DB_PG"
}

func Connect(t *testing.T) func(*testing.T) *pgxpool.Pool {
	t.Helper()
	base := pgxtest.Connect(t)
	if base == nil {
		return nil
	}
	return func(t *testing.T) *pgxpool.Pool {
		t.Helper()
		pool := base(t)
		if err := pgxtest.ApplyFS(context.Background(), pool, os.DirFS(migrationsDir(t))); err != nil {
			t.Fatalf("pgtest: apply migrations: %v", err)
		}
		return pool
	}
}

func migrationsDir(t *testing.T) string {
	t.Helper()
	wd, err := os.Getwd()
	if err != nil {
		t.Fatalf("pgtest: getwd: %v", err)
	}
	candidate := filepath.Join(wd, "db", "migrations")
	if _, err := os.Stat(candidate); err == nil {
		return candidate
	}
	d := wd
	for i := 0; i < 5; i++ {
		c := filepath.Join(d, "internal", "infrastructure", "postgres", "db", "migrations")
		if _, err := os.Stat(c); err == nil {
			return c
		}
		d = filepath.Dir(d)
	}
	t.Fatalf("pgtest: could not locate db/migrations from %s", wd)
	return ""
}
