package pg

import (
	"context"
	"embed"
	"fmt"
)

//go:embed schema.sql
var schemaFS embed.FS

// Migrate applies the coordination schema. It is idempotent (every statement is
// IF NOT EXISTS) and safe to run concurrently from several starting instances.
//
// The schema is applied at startup rather than by a separate migration job because
// these tables are unlogged scratch space, not system of record — a fresh database,
// or one whose tables a crash truncated, must be usable without an operator step.
//
// The lock and the DDL share ONE explicit transaction. pg_advisory_xact_lock is
// released at commit, so issuing it as a separate Exec would release it before the
// schema was applied — and on a pool the two calls can even land on different
// connections. The DDL is a multi-statement string, which rules out a batch (pgx
// batches use the extended protocol, one statement per query), so an explicit
// transaction is the only form that holds the lock for the whole apply.
func Migrate(ctx context.Context, q Querier) error {
	ddl, err := schemaFS.ReadFile("schema.sql")
	if err != nil {
		return fmt.Errorf("pg: read schema: %w", err)
	}

	tx, err := q.Begin(ctx)
	if err != nil {
		return fmt.Errorf("pg: begin migration: %w", err)
	}
	// Rollback is a no-op once Commit has succeeded.
	defer func() { _ = tx.Rollback(ctx) }()

	if _, err := tx.Exec(ctx, lockMigrationSQL); err != nil {
		return fmt.Errorf("pg: take migration lock: %w", err)
	}
	if _, err := tx.Exec(ctx, string(ddl)); err != nil {
		return fmt.Errorf("pg: apply schema: %w", err)
	}
	if err := tx.Commit(ctx); err != nil {
		return fmt.Errorf("pg: commit migration: %w", err)
	}
	return nil
}
