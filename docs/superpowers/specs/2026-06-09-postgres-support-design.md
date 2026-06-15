# Postgres Support & Migration Golden Path — Design

- **Date:** 2026-06-09
- **Status:** Draft (awaiting review)
- **Component:** `server/` (Go GraphQL API) + `github.com/reearth/reearthx`
- **Author:** piyush@eukarya.io

## 1. Goal & Scope

Add **PostgreSQL** as an alternative persistence backend for the reearth-flow Go
server, behind the **existing `repo.Container` interfaces** — no changes to
domain models, use cases, or repository interfaces. Along the way, establish a
**repeatable per-entity "golden path"** (Atlas + sqlc + a transaction
abstraction) that other reearth projects following the same Clean-Architecture +
reearthx pattern can inherit.

The generic, reusable SQL machinery lives in **reearthx** (built first); flow
consumes it.

### In scope (now)

- Atlas declarative schema + versioned migrations.
- sqlc typed query generation (pgx/v5).
- A Postgres-backed `usecasex.Transaction` implementation + executor-from-context
  helper, hosted in reearthx.
- A Postgres repo adapter for the **pilot entity: `Trigger`**, implementing
  `repo.Trigger`.
- Integration tests against a real Postgres asserting parity with the Mongo repo.
- Local dev (docker-compose) + CI plumbing.

### Out of scope (now)

- Reading/migrating existing **Mongo data** into Postgres (data ETL,
  dual-write, backfill). Tracked as a separate later effort.
- Porting every entity. Only `Trigger` is implemented in Phase 1; the rest follow
  the golden path one PR at a time.
- Flipping production to Postgres.

## 2. Key Decisions

| # | Decision | Choice |
|---|----------|--------|
| A | Coexistence & transaction model | **A1 — global driver switch, pilot validated by tests.** `DB_DRIVER` selects the whole backend at boot. Code migrates entity-by-entity, each proven by integration tests vs Mongo parity. Prod stays Mongo until the full set (incl. reearthx account repos) is ported, then a single cutover. |
| B | Migration tooling | **Atlas**, declarative `schema.hcl` as single source of truth → `atlas migrate diff` generates versioned SQL → same migrations feed sqlc. CI runs `atlas migrate lint`. |
| C | Query codegen + driver | **sqlc** with **pgx/v5**. Transaction wiring via executor-from-context (Thibaut Rousseau pattern). **Superseded — see Decision F.** |
| F | Transaction abstraction (revised 2026-06-09) | Adopt the blog's **`Transactor.WithinTransaction(ctx, fn)` callback** as the canonical abstraction, **not** `usecasex.Transaction` (Begin/Commit/End). New `usecasex.Transactor` interface; Mongo reuses its existing `usecasex.Transaction` via a `NewTransactor` bridge (DoTransaction-backed); **all** flow interactors + `repo.Container.Transaction` migrate to it (full adoption). The account package keeps `usecasex.Transaction` via a compat bridge. |
| G | Generic pg lib shape + tooling (revised 2026-06-10) | `reearthx/pgxx` provides a **`Client`** (mirroring `mongox.Client`): holds the pool, `DB(ctx)` executor-from-context, and a **composing** `WithinTransaction` (reuses an ambient tx in ctx so nested calls compose). `Client` implements `usecasex.Transactor`. **Atlas is retained** for migrations (declarative schema → versioned SQL; preferred for cross-DB). sqlc uses `emit_interface` + timestamptz→`time.Time` overrides for clean Go types. `reearth-accounts` is the reference for the *account domain* (its own repo, golang-migrate today) and is converged onto the shared lib **separately by the owner — out of scope here**. No account/asset domain repos are built in this effort. |
| H | Migration & cutover (added 2026-06-16) | **Schema apply at runtime = `atlas migrate apply` via a GCP Cloud Run Job** (mirrors `reearth-accounts`' `run-migration.yml`: `gcloud run jobs update <job> --image …` then `execute --wait`). Image strategy = **Option A**: a dedicated `Dockerfile.migrate` `FROM arigaio/atlas` with `db/migrations/` copied in (keeps the app image migration-free). The Cloud Run Job terraform lives in **reearth-infrastructure** and is wired **after PR #148 merges**. **Layer 3 (Mongo→Postgres data ETL)** is NOT an Atlas job — planned as an interface-driven backfill (read via Mongo repos, `Save` via PG repos) or dual-write; the maintenance-window-vs-zero-downtime choice is still open. Prod cutover = global `DB_DRIVER` flip per env, Mongo kept hot for instant rollback, gated on account+authserver also on PG (separate stream). |
| D | Where reusable code lives | **reearthx-first.** Generic SQL/tx package built and released in reearthx (Phase 0), then consumed by flow. |
| E | Pilot entity | **`Trigger`** — exercises pagination, workspace filtering, find-by-FK, CRUD, and transactional writes. |

### Why A1 (the consequential one)

The Trigger interactor wraps `triggerRepo.Save` **and** `jobRepo.Save` inside a
single `usecasex.Transaction` (e.g. executing a trigger creates a Job and updates
the trigger atomically — `internal/usecase/interactor/trigger.go:143–251`). A
single transaction cannot span Mongo and Postgres. Therefore we cannot flip
individual entities to Postgres in production while their transactional partners
stay on Mongo without either expanding the pilot or sacrificing atomicity. A1
keeps transaction semantics honest: the running app uses exactly one backend, and
the per-entity work is proven by tests until the whole set is ready for a single
cutover. The "golden path" is the **repeatable code + test recipe**, not a
per-entity production toggle.

## 3. Current Architecture (context)

- Clean Architecture + DDD. Repo interfaces in `internal/usecase/repo`,
  implementations in `internal/infrastructure/{mongo,memory}`.
- `repo.Container` aggregates ~14 flow-owned repos (Asset, AssetUpload, Config,
  WorkerConfig, Deployment, EdgeExecution, Job, NodeExecution, Parameter, Project,
  ProjectAccess, Lock, Trigger, Workflow) plus `AuthRequest`.
- `User`, `Workspace`, `Role`, `Permittable`, and `Transaction` come from
  `github.com/reearth/reearthx/account` + `usecasex`.
- `usecasex.Transaction` is **context-carrying**: `Begin(ctx) → Tx`, and
  `Tx.Context()` propagates the session/tx. `DoTransaction` provides a retry loop
  keyed on `usecasex.ErrTransaction`.
- Boot wiring lives in `internal/app/repo.go:initReposAndGateways`.
- **reearthx currently has zero SQL/Postgres/sqlc/Atlas code** — Mongo + memory +
  fs only. The reusable SQL layer is net-new.
- Go 1.24.10.

## 4. reearthx Phase 0 — Reusable SQL Package

A new package in reearthx named **`pgxx`** (chosen to avoid clashing with
`github.com/jmoiron/sqlx`). Provides:

1. **`usecasex.Transaction` implementation** backed by `pgxpool.Pool`:
   - `Begin(ctx)` opens a `pgx.Tx` and returns a `Tx` whose `Context()` carries the
     `pgx.Tx`.
   - `Commit()` / `End(ctx)` / `IsCommitted()` follow the existing semantics
     (commit only if `Commit()` was called, else rollback on `End`).
   - Postgres serialization/deadlock failures (SQLSTATE `40001`, `40P01`) are
     mapped to `usecasex.ErrTransaction` so the existing `DoTransaction` retry loop
     works unchanged.
2. **`Executor(ctx, pool) DBTX`** helper — returns the `pgx.Tx` stored in the
   context if a transaction is active, otherwise the pool. This is the core of the
   Thibaut Rousseau "transaction in context" pattern. `DBTX` is the minimal
   query interface satisfied by both `pgx.Tx` and `pgxpool.Pool` (the same shape
   sqlc's pgx output expects).
3. **Page-based pagination helper** producing `LIMIT/OFFSET` + a `COUNT(*)` total,
   shaped to fill `interfaces.PageBasedInfo` (cursor-based helper can follow later).
4. **Atlas migration runner** — applies embedded versioned migrations at boot or
   via CLI; shared sqlc + Atlas config conventions other projects copy.
5. **Observability** — `otelpgx` tracing to mirror the existing `otelmongo`.

Released and tagged; flow bumps `go.mod` to consume it. **The
`usecasex.Transaction` interface is not modified** — Postgres is simply a second
implementation alongside the Mongo one.

## 5. Flow Phase 1 — Trigger Pilot

### Layout

```
server/api/internal/infrastructure/postgres/
  db/
    schema.hcl              # Atlas declarative schema (source of truth)
    migrations/*.sql        # generated, versioned (atlas migrate diff)
    atlas.hcl               # Atlas project/env config
  query/
    trigger.sql             # sqlc query annotations
  gen/                      # sqlc-generated types + queries (pgx/v5)
  sqlc.yaml
  trigger.go                # repo.Trigger adapter
  container.go              # postgres.New(...) wiring
```

### Trigger table

IDs are reearth ULID strings (not UUIDs) → stored as `text`. `variables` →
`jsonb`. Mirrors the persisted Mongo `TriggerDocument` fields.

```sql
CREATE TABLE triggers (
  id            text PRIMARY KEY,
  workspace_id  text NOT NULL,
  deployment_id text NOT NULL,
  description   text NOT NULL DEFAULT '',
  event_source  text NOT NULL,
  time_interval text,
  auth_token    text,
  enabled       boolean NOT NULL DEFAULT false,
  last_triggered timestamptz,
  variables     jsonb,
  created_at    timestamptz NOT NULL,
  updated_at    timestamptz NOT NULL
);
CREATE INDEX triggers_workspace_id_idx  ON triggers (workspace_id);
CREATE INDEX triggers_deployment_id_idx ON triggers (deployment_id);
```

### Repo adapter behavior (parity with Mongo)

`trigger.go` implements `repo.Trigger`:

- `Filtered(WorkspaceFilter)` → applied as SQL predicates (`workspace_id IN (...)`
  for read/write filtering, matching the Mongo `f.CanRead/CanWrite` semantics).
- `FindByID` / `FindByIDs` — `FindByIDs` returns results in input order with `nil`
  placeholders for misses (mirrors `filterTriggers`).
- `FindByWorkspace(ctx, workspaceID, *PaginationParam, *keyword)`:
  - keyword → `ILIKE` on `description` and `id`.
  - page pagination → `LIMIT/OFFSET` + `COUNT(*)`, `ORDER BY` mapped column
    (`description`, `created_at`, `updated_at`, `id`; the Mongo code's `status`
    sort key maps to a non-existent field and is treated as an ignored no-op),
    default `updated_at DESC`.
  - returns `*interfaces.PageBasedInfo` via `NewPageBasedInfo(total, page, size)`.
- `FindByDeployment` → `WHERE deployment_id = $1`.
- `Save` → upsert (`INSERT ... ON CONFLICT (id) DO UPDATE`), guarded by
  `f.CanWrite`.
- `Remove` → delete guarded by write filter.

All queries obtain their executor via `Executor(ctx, pool)`, so they
automatically participate in an active `usecasex.Transaction`.

### Boot wiring (`internal/app/repo.go`)

Add `DB_DRIVER` (default `mongo`). When `postgres`: build a `pgxpool`, run Atlas
migrations, construct `postgres.New(...)`. Under A1, `postgres.New` can only return
a complete `repo.Container` once all entities are ported; during Phase 1 the
Trigger PG repo is exercised by integration tests rather than by booting the app in
`postgres` mode. Not-yet-ported repos surface a clear "not implemented for postgres"
error to prevent accidental partial production use.

## 6. The Golden-Path Recipe (repeatable per entity)

1. Add the entity's table to `db/schema.hcl`.
2. `atlas migrate diff <name>` → new versioned SQL migration.
3. Write `query/<entity>.sql` with sqlc annotations.
4. `sqlc generate`.
5. Implement the repo adapter mapping domain ↔ rows, using `Executor(ctx, pool)`.
6. Write an integration test (dockertest) asserting parity with the Mongo repo.
7. CI: `atlas migrate lint` + `sqlc` drift check.

This recipe — plus the reearthx `pgxx` package — is what other projects copy.

## 7. Transaction Model (revised 2026-06-09 — Decision F)

The canonical abstraction is the Thibaut Rousseau **`Transactor.WithinTransaction(ctx, fn)`
callback**, NOT `usecasex.Transaction` (Begin/Commit/End):

```go
// reearthx/usecasex
type Transactor interface {
    WithinTransaction(ctx context.Context, fn func(ctx context.Context) error) error
}
```

- **Mongo (and Nop):** reuse the existing `usecasex.Transaction` via a bridge
  `usecasex.NewTransactor(t Transaction, retry int)` whose `WithinTransaction`
  delegates to `usecasex.DoTransaction` — so no hand-written Mongo impl and the
  existing retry-on-`ErrTransaction` behavior is preserved.
- **Postgres:** `pgxx` implements `Transactor` natively — begins a `pgx.Tx`,
  carries it in the context (`Executor(ctx, pool)` / DBGetter resolves it in
  repos), runs `fn`, commits on success / rolls back on error, retrying on
  serialization failure (`WrapError`). `pgxx` no longer implements
  `usecasex.Transaction`.
- **Flow (full adoption):** `repo.Container.Transaction` becomes
  `usecasex.Transactor`; `usecase.go`'s `Run3` and all 26 manual
  `Begin/Commit/End` call-sites across 8 interactors become
  `transaction.WithinTransaction(ctx, func(ctx) error { … })` closures
  (semantics preserved: commit on nil return, rollback on error/panic).
- **Account package:** continues to consume `usecasex.Transaction`; flow keeps a
  `usecasex.Transaction` reference for the `AccountRepos()` compat bridge.

## 8. Error Handling

- `pgx.ErrNoRows` → `rerror.ErrNotFound`.
- Other DB errors → `rerror.ErrInternalByWithContext(ctx, err)`.
- SQLSTATE `40001`/`40P01` → `usecasex.ErrTransaction` (triggers retry).

## 9. Testing Strategy

- **Integration tests** against a real Postgres via **dockertest** (parallel to
  reearthx `mongotest`). A parity suite mirrors the existing Mongo Trigger tests:
  CRUD, `FindByIDs` ordering, workspace filtering, pagination, keyword search,
  find-by-deployment, transactional Save/rollback.
- **Unit tests** for domain ↔ row mapping (incl. nullable fields and `variables`
  JSONB round-trip).
- **CI**: spin a Postgres service, apply Atlas migrations, assert `sqlc generate`
  and `atlas migrate diff` produce no uncommitted drift, run `go test` with race.

## 10. Local Dev & CI

- docker-compose: add a `postgres` service.
- Makefile targets: `pg-up`, `sqlc`, `atlas-diff`, `atlas-lint`.
- CI drift check ensures generated code + migrations are committed.

## 11. Phasing

- **Phase 0 (reearthx):** generic `pgxx` package + PG `Transaction`; release; bump
  `go.mod` in flow.
- **Phase 1 (flow):** Trigger pilot end-to-end (schema, migrations, sqlc, adapter,
  tests) + tooling + CI.
- **Phase 2…N (flow):** remaining entities, one PR each, following the recipe; incl.
  Postgres implementations of the reearthx account repos.
- **Final:** flip `DB_DRIVER=postgres`, remove Mongo infrastructure.

## 12. Open Items / Risks

- **reearthx package name:** resolved — **`pgxx`** (avoids the
  `github.com/jmoiron/sqlx` clash).
- **reearthx local checkout:** resolved — `/Users/dexter/active/reearthx`
  (sibling of reearth-flow; `origin git@github.com:reearth/reearthx.git`, `main`).
  Phase 0 is developed there. Flow consumes it during development via a temporary
  `replace` directive in `go.mod` (pointing at the local path), then a tagged
  reearthx release + `go.mod` version bump to land. (Flow currently pins
  `v0.0.0-20251202081949-5abca579aec6`; the `replace` sidesteps version skew while
  iterating.)
- **Account repos:** resolved — Postgres implementations of the reearthx `account`
  repos (User/Workspace/Role/Permittable) are tracked as a **separate later
  stream**. They gate the final production cutover but **not** the Trigger pilot.
- **pgxpool tuning & observability** (`otelpgx`) to match current Mongo telemetry.
- **ID column type:** `text` (ULID) chosen for fidelity with existing
  serialization; revisit only if a future entity uses true UUIDs.
```
