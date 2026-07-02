package migration

import "github.com/reearth/reearthx/usecasex/migration"

// Postgres data/code migrations, keyed by version. DDL/schema changes are
// handled by Atlas (db/migrations) + the Cloud Run apply job; this map is for
// data/code migrations that must run transactionally via pgxx.Client.
//
// To add one, append an entry: <version>: func(ctx, c DBClient) error { ... }
// using c.DB(ctx) for SQL. Migrations run in ascending version order, each in
// its own transaction, tracked by the config row's Migration field.
var migrations = migration.Migrations[DBClient]{}
