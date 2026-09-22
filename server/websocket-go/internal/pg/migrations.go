package pg

import (
	"context"
	"embed"
	"fmt"
)

//go:embed schema.sql
var schemaFS embed.FS

// Migrate applies the coordination schema. It is idempotent (every statement is
// IF NOT EXISTS) and safe to run concurrently from several starting instances:
// the advisory lock serialises them so two CREATE TABLE races cannot both fire.
//
// The schema is applied at startup rather than by a separate migration job because
// these tables are unlogged scratch space, not system of record — a fresh database,
// or one whose tables a crash truncated, must be usable without an operator step.
func Migrate(ctx context.Context, q Querier) error {
	ddl, err := schemaFS.ReadFile("schema.sql")
	if err != nil {
		return fmt.Errorf("pg: read schema: %w", err)
	}
	if _, err := q.Exec(ctx, lockMigrationSQL); err != nil {
		return fmt.Errorf("pg: take migration lock: %w", err)
	}
	if _, err := q.Exec(ctx, string(ddl)); err != nil {
		return fmt.Errorf("pg: apply schema: %w", err)
	}
	return nil
}
