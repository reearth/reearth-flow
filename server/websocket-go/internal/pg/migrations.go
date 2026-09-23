package pg

import (
	"context"
	"embed"
	"fmt"
)

//go:embed schema.sql
var schemaFS embed.FS

// Migrate applies the coordination schema. Idempotent and safe to run concurrently
// from several starting instances. Applied at startup, not by a migration job: these
// are unlogged scratch tables, so a crash-truncated database must recover unattended.
//
// The lock and DDL share ONE explicit transaction — pg_advisory_xact_lock releases at
// commit, so a separate Exec would drop it before the schema is applied. A batch
// cannot substitute: the DDL is multi-statement.
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
