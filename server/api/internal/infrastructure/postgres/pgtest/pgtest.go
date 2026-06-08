// Package pgtest wraps reearthx pgxtest with reearth-flow's migration application,
// so each test gets an isolated database with the triggers schema already applied.
package pgtest

import (
	"context"
	"os"
	"path/filepath"
	"sort"
	"strings"
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
		applyMigrations(t, pool)
		return pool
	}
}

func applyMigrations(t *testing.T, pool *pgxpool.Pool) {
	t.Helper()
	dir := migrationsDir(t)
	entries, err := os.ReadDir(dir)
	if err != nil {
		t.Fatalf("pgtest: read migrations dir: %v", err)
	}
	var files []string
	for _, e := range entries {
		if strings.HasSuffix(e.Name(), ".sql") {
			files = append(files, e.Name())
		}
	}
	sort.Strings(files)
	ctx := context.Background()
	for _, f := range files {
		sqlBytes, err := os.ReadFile(filepath.Join(dir, f))
		if err != nil {
			t.Fatalf("pgtest: read migration %s: %v", f, err)
		}
		if _, err := pool.Exec(ctx, string(sqlBytes)); err != nil {
			t.Fatalf("pgtest: apply migration %s: %v", f, err)
		}
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
