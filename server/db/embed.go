// Package db holds the flow Postgres schemas: the Atlas-authored migration files
// and helpers to apply them. The .sql files are the single source of truth
// (authored via `atlas migrate diff`); embed.go makes them available at runtime
// without a repo checkout.
//
// Two schemas, deliberately separate — they have different lifecycles and live in
// different databases:
//
//   - schema.hcl / migrations: the API's system-of-record tables.
//   - ws-schema.hcl / ws-migrations: websocket-go's unlogged coordination tables.
package db

import "embed"

//go:embed migrations/*.sql
var MigrationsFS embed.FS

// WSMigrationsFS holds the websocket-go coordination schema.
//
//go:embed ws-migrations/*.sql
var WSMigrationsFS embed.FS
