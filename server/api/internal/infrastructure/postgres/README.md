# Postgres backend (golden path)

Adds Postgres support behind `repo.Container`. Generic SQL/transaction code lives
in `github.com/reearth/reearthx/pgxx`.

## Per-entity recipe

1. Add the table to `db/schema.hcl`.
2. Generate a migration: `make atlas-diff name=<change>` (from `server/api`).
3. Add queries to `query/<entity>.sql` with sqlc annotations.
4. Regenerate: `make sqlc`.
5. Implement the repo adapter in `<entity>.go`, obtaining the executor via
   `pgxx.Executor(ctx, pool)` so writes join any active transaction.
6. Add parity integration tests in `<entity>_test.go` using `pgtest.Connect`.
   Run locally: `make run-db-pg` then
   `REEARTH_FLOW_DB_PG=postgres://reearth:reearth@localhost:5432/postgres?sslmode=disable make test`.
7. CI runs the tests against a Postgres service and checks codegen drift
   (`sqlc generate` is clean) and migration validity (`atlas migrate validate`).

## Transactions

Repos hold a `*pgxx.Client`. `pgxx.NewClient(pool)` backs `repo.Container.Transaction`
and provides `WithinTransaction`. Repos obtain the active executor via `client.DB(ctx)`,
which returns the ambient transaction if one is active, or the pool otherwise — so writes
automatically join any transaction started by `WithinTransaction`. Use-case code calls
`i.transaction.WithinTransaction(ctx, func(ctx context.Context) error { ... })` —
returning nil commits; returning an error rolls back.

## Tooling notes

- `sqlc` is a standalone binary (`sqlc generate`), not a `go tool`.
- `atlas migrate lint` requires Atlas Pro; CI uses `atlas migrate validate` (free).

## Status

Ported: Trigger. Unported repos are guarded by `mustComplete` in `container.go`;
`DB_DRIVER=postgres` cannot boot until every entity is ported (design A1).
