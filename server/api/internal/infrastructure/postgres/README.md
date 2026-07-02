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

All flow-owned repos are ported to Postgres (Trigger, Lock, Config, Parameter,
WorkerConfig, ProjectAccess, Workflow, Edge/NodeExecution, Deployment, Job,
Project, Asset, AssetUpload; AuthRequest via reearthx `authserver.Postgres`).
The interim `mustComplete` boot guard has been removed — `DB_DRIVER=postgres`
boots a complete backend. Mongo is untouched and remains the default; the
per-environment rollout is a deployment concern (design A1).

## Data migration (Mongo → Postgres)

`cmd/dbmigrate` replicates the flow-owned data. It streams each Mongo collection
through the same `mongodoc` decoder the Mongo repos use and upserts each record
via the same Postgres adapter `Save` the API uses, so it inherits both sides'
field mappings and is idempotent (re-runnable). Account repos stay on Mongo;
Config (migration bookkeeping) and AuthRequest (transient) are skipped.

```sh
REEARTH_FLOW_DB="mongodb+srv://…"  REEARTH_FLOW_DB_PG="postgres://…" \
  go run ./cmd/dbmigrate -apply-schema -db reearth-flow

# read every replicated row back through the Postgres adapters (target only)
REEARTH_FLOW_DB_PG="postgres://…" go run ./cmd/dbmigrate -verify
```

## Seeding a new Postgres (golden path)

`cmd/dbmigrate` replicates the flow-owned collections from Mongo into Postgres.
With `-apply-schema` it first applies the embedded Atlas migrations
(`internal/infrastructure/postgres/db/migrations`, embedded via `db.MigrationsFS`)
to a fresh instance, so a brand-new Cloud SQL database needs no separate schema
step. `-apply-schema` is one-shot (bare `CREATE TABLE`); reseed = drop/recreate.

For a private-IP Cloud SQL instance, run `dbmigrate` from an ephemeral, in-VPC
Cloud Run job on the flow-api image (which now ships the `dbmigrate` binary)
so the job can reach the private instance without a bastion:

```sh
gcloud run jobs create reearth-flow-dbmigrate \
  --project reearth-oss \
  --region us-central1 \
  --image us-central1-docker.pkg.dev/reearth-oss/reearth/reearth-flow-api:nightly \
  --network default \
  --subnet default \
  --vpc-egress private-ranges-only \
  --service-account reearth-flow-migration@reearth-oss.iam.gserviceaccount.com \
  --set-secrets REEARTH_FLOW_DB=reearth-flow-db:latest,REEARTH_FLOW_DB_PG=reearth-flow-db-postgres:latest \
  --command /reearth-flow/dbmigrate \
  --args=-apply-schema,-db=reearth-flow,-verify

gcloud run jobs execute reearth-flow-dbmigrate \
  --project reearth-oss \
  --region us-central1 \
  --wait
```

The service account needs `roles/secretmanager.secretAccessor` on both
`reearth-flow-db` (source Mongo) and `reearth-flow-db-postgres` (target). Clean
up the job when the seed is done, and re-run per environment by changing the
project, image, and service account inputs.
