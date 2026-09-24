package pg

import (
	"context"
	"embed"
	"fmt"
	"io/fs"
	"sort"
	"strings"
)

//go:embed migrations/*.sql
var migrationsFS embed.FS

const migrationsDir = "migrations"

// revisionTableSQL tracks applied migrations. Hand-written and idempotent: it must
// exist before any Atlas migration can be recorded.
const revisionTableSQL = `
CREATE TABLE IF NOT EXISTS ws_schema_migrations (
  version    text PRIMARY KEY,
  applied_at timestamptz NOT NULL DEFAULT now()
)`

const (
	appliedVersionsSQL = `SELECT version FROM ws_schema_migrations`
	recordVersionSQL   = `INSERT INTO ws_schema_migrations (version) VALUES ($1)`
	// schemaExistsSQL detects a database predating revision tracking.
	schemaExistsSQL = `SELECT to_regclass('ws_stream') IS NOT NULL`
)

// Migrate applies unseen migrations in filename order. Safe to run concurrently from
// several starting instances, and at startup rather than from a job: these tables are
// unlogged scratch space, so a fresh database must work without an operator step.
//
// Authored in server/db (`atlas migrate diff --env ws`); the .sql files here are a
// committed copy — the image build context cannot reach outside this module. Refresh
// with `make sync-migrations`; CI fails on drift.
//
// Atlas emits bare CREATE TABLE, hence the version tracking. All of it runs in ONE
// transaction with the advisory lock, which releases at commit.
func Migrate(ctx context.Context, q Querier) error {
	names, err := migrationNames()
	if err != nil {
		return err
	}

	tx, err := q.Begin(ctx)
	if err != nil {
		return fmt.Errorf("pg: begin migration: %w", err)
	}
	defer func() { _ = tx.Rollback(ctx) }()

	if _, err := tx.Exec(ctx, lockMigrationSQL); err != nil {
		return fmt.Errorf("pg: take migration lock: %w", err)
	}
	if _, err := tx.Exec(ctx, revisionTableSQL); err != nil {
		return fmt.Errorf("pg: create revision table: %w", err)
	}

	applied := map[string]bool{}
	rows, err := tx.Query(ctx, appliedVersionsSQL)
	if err != nil {
		return fmt.Errorf("pg: read applied versions: %w", err)
	}
	for rows.Next() {
		var v string
		if err := rows.Scan(&v); err != nil {
			rows.Close()
			return fmt.Errorf("pg: scan applied version: %w", err)
		}
		applied[v] = true
	}
	rows.Close()
	if err := rows.Err(); err != nil {
		return fmt.Errorf("pg: read applied versions: %w", err)
	}

	// Baseline a pre-Atlas database: tables present, revisions absent. Re-running the
	// bare CREATE TABLE would fail, so record the initial migration instead. Empty
	// revision table only — a partially-migrated one is left alone.
	if len(applied) == 0 {
		var exists bool
		if err := tx.QueryRow(ctx, schemaExistsSQL).Scan(&exists); err != nil {
			return fmt.Errorf("pg: detect existing schema: %w", err)
		}
		if exists {
			if _, err := tx.Exec(ctx, recordVersionSQL, names[0]); err != nil {
				return fmt.Errorf("pg: baseline %s: %w", names[0], err)
			}
			applied[names[0]] = true
		}
	}

	for _, name := range names {
		if applied[name] {
			continue
		}
		body, err := migrationsFS.ReadFile(migrationsDir + "/" + name)
		if err != nil {
			return fmt.Errorf("pg: read %s: %w", name, err)
		}
		if _, err := tx.Exec(ctx, string(body)); err != nil {
			return fmt.Errorf("pg: apply %s: %w", name, err)
		}
		if _, err := tx.Exec(ctx, recordVersionSQL, name); err != nil {
			return fmt.Errorf("pg: record %s: %w", name, err)
		}
	}

	if err := tx.Commit(ctx); err != nil {
		return fmt.Errorf("pg: commit migration: %w", err)
	}
	return nil
}

// migrationNames lists the embedded .sql files in apply order.
func migrationNames() ([]string, error) {
	entries, err := fs.ReadDir(migrationsFS, migrationsDir)
	if err != nil {
		return nil, fmt.Errorf("pg: read migrations: %w", err)
	}
	names := make([]string, 0, len(entries))
	for _, e := range entries {
		if !e.IsDir() && strings.HasSuffix(e.Name(), ".sql") {
			names = append(names, e.Name())
		}
	}
	sort.Strings(names)
	if len(names) == 0 {
		return nil, fmt.Errorf("pg: no migrations embedded from %s", migrationsDir)
	}
	return names, nil
}
