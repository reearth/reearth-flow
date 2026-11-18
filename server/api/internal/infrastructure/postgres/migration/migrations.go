package migration

import "github.com/reearth/reearthx/usecasex/migration"

// migrations are Postgres data/code migrations keyed by version, each run in its
// own transaction in ascending order. Schema changes belong in Atlas
// (db/migrations), not here.
var migrations = migration.Migrations[DBClient]{}
