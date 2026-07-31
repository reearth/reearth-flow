// Package db holds the flow Postgres schema: the Atlas-authored migration
// files and helpers to apply them. The .sql files are the single source of
// truth (authored via `atlas migrate diff`); embed.go makes them available
// at runtime to the seed/ETL tooling without a repo checkout.
package db

import "embed"

//go:embed migrations/*.sql
var MigrationsFS embed.FS
