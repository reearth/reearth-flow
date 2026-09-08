package db

import (
	"context"
	"fmt"
	"io/fs"
	"log"
	"sort"
	"strings"

	"github.com/jackc/pgx/v5/pgxpool"
)

// Apply runs every embedded migration, in filename order, against pool.
//
// It is intentionally not idempotent: the Atlas migrations use bare
// CREATE TABLE, so Apply targets a fresh database (initial seed of a new
// instance). Recurring/incremental migration is a separate concern.
func Apply(ctx context.Context, pool *pgxpool.Pool) error {
	entries, err := fs.ReadDir(MigrationsFS, "migrations")
	if err != nil {
		return fmt.Errorf("db.Apply: read migrations: %w", err)
	}

	names := make([]string, 0, len(entries))
	for _, entry := range entries {
		if !entry.IsDir() && strings.HasSuffix(entry.Name(), ".sql") {
			names = append(names, entry.Name())
		}
	}
	sort.Strings(names)

	for _, name := range names {
		body, err := fs.ReadFile(MigrationsFS, "migrations/"+name)
		if err != nil {
			return fmt.Errorf("db.Apply: read %s: %w", name, err)
		}
		if _, err := pool.Exec(ctx, string(body)); err != nil {
			return fmt.Errorf("db.Apply: %s: %w", name, err)
		}
		log.Printf("  applied %s", name)
	}

	return nil
}
