package db

import (
	"context"
	"errors"
	"fmt"
	"io/fs"
	"sort"
	"strings"

	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgconn"
	"github.com/jackc/pgx/v5/pgxpool"
)

const (
	// revisionTableSQL tracks applied migrations. Hand-written and idempotent: it
	// must exist before any Atlas migration can be recorded.
	revisionTableSQL = `
CREATE TABLE IF NOT EXISTS schema_migrations (
  version    text PRIMARY KEY,
  applied_at timestamptz NOT NULL DEFAULT now()
)`
	lockMigrationSQL   = `SELECT pg_advisory_xact_lock(hashtext('flow_api_migrate'))`
	appliedVersionsSQL = `SELECT version FROM schema_migrations`
	recordVersionSQL   = `INSERT INTO schema_migrations (version) VALUES ($1)`
	// schemaExistsSQL detects a database seeded by Apply, before revisions were tracked.
	schemaExistsSQL = `SELECT to_regclass('public.projects') IS NOT NULL`
)

// Migrate applies the embedded migrations not yet recorded in schema_migrations,
// in filename order. It runs at API startup so a deploy carrying a new migration
// also carries the schema change. Safe to run from several starting instances:
// everything runs in one transaction holding an advisory lock.
//
// A database seeded by Apply has tables but no revisions. Which migrations it
// already holds depends on when it was seeded, so each one is tried in a
// savepoint: one that fails because its objects already exist is recorded as
// applied, and the rest run. This baseline happens once, on an empty revision
// table only.
func Migrate(ctx context.Context, pool *pgxpool.Pool) error {
	names, err := migrationNames()
	if err != nil {
		return err
	}

	tx, err := pool.Begin(ctx)
	if err != nil {
		return fmt.Errorf("db.Migrate: begin: %w", err)
	}
	defer func() { _ = tx.Rollback(ctx) }()

	if _, err := tx.Exec(ctx, lockMigrationSQL); err != nil {
		return fmt.Errorf("db.Migrate: take lock: %w", err)
	}
	if _, err := tx.Exec(ctx, revisionTableSQL); err != nil {
		return fmt.Errorf("db.Migrate: create revision table: %w", err)
	}

	applied, err := appliedVersions(ctx, tx)
	if err != nil {
		return err
	}

	baseline := false
	if len(applied) == 0 {
		if err := tx.QueryRow(ctx, schemaExistsSQL).Scan(&baseline); err != nil {
			return fmt.Errorf("db.Migrate: detect existing schema: %w", err)
		}
	}

	for _, name := range names {
		if applied[name] {
			continue
		}
		body, err := fs.ReadFile(MigrationsFS, "migrations/"+name)
		if err != nil {
			return fmt.Errorf("db.Migrate: read %s: %w", name, err)
		}
		if baseline {
			err = applyIfAbsent(ctx, tx, string(body))
		} else {
			_, err = tx.Exec(ctx, string(body))
		}
		if err != nil {
			return fmt.Errorf("db.Migrate: apply %s: %w", name, err)
		}
		if _, err := tx.Exec(ctx, recordVersionSQL, name); err != nil {
			return fmt.Errorf("db.Migrate: record %s: %w", name, err)
		}
	}

	if err := tx.Commit(ctx); err != nil {
		return fmt.Errorf("db.Migrate: commit: %w", err)
	}
	return nil
}

// applyIfAbsent runs body in a savepoint, treating "already exists" as applied.
func applyIfAbsent(ctx context.Context, tx pgx.Tx, body string) error {
	if _, err := tx.Exec(ctx, "SAVEPOINT migration"); err != nil {
		return err
	}
	if _, err := tx.Exec(ctx, body); err != nil {
		if _, rerr := tx.Exec(ctx, "ROLLBACK TO SAVEPOINT migration"); rerr != nil {
			return rerr
		}
		if !isAlreadyExists(err) {
			return err
		}
	}
	_, err := tx.Exec(ctx, "RELEASE SAVEPOINT migration")
	return err
}

func isAlreadyExists(err error) bool {
	var pgErr *pgconn.PgError
	if !errors.As(err, &pgErr) {
		return false
	}
	switch pgErr.Code {
	case "42P07", // duplicate_table (also indexes)
		"42701", // duplicate_column
		"42710": // duplicate_object
		return true
	}
	return false
}

func appliedVersions(ctx context.Context, tx pgx.Tx) (map[string]bool, error) {
	rows, err := tx.Query(ctx, appliedVersionsSQL)
	if err != nil {
		return nil, fmt.Errorf("db.Migrate: read applied versions: %w", err)
	}
	defer rows.Close()
	applied := map[string]bool{}
	for rows.Next() {
		var v string
		if err := rows.Scan(&v); err != nil {
			return nil, fmt.Errorf("db.Migrate: scan applied version: %w", err)
		}
		applied[v] = true
	}
	if err := rows.Err(); err != nil {
		return nil, fmt.Errorf("db.Migrate: read applied versions: %w", err)
	}
	return applied, nil
}

// migrationNames lists the embedded .sql files in apply order.
func migrationNames() ([]string, error) {
	entries, err := fs.ReadDir(MigrationsFS, "migrations")
	if err != nil {
		return nil, fmt.Errorf("db: read migrations: %w", err)
	}
	names := make([]string, 0, len(entries))
	for _, e := range entries {
		if !e.IsDir() && strings.HasSuffix(e.Name(), ".sql") {
			names = append(names, e.Name())
		}
	}
	sort.Strings(names)
	if len(names) == 0 {
		return nil, errors.New("db: no migrations embedded")
	}
	return names, nil
}
