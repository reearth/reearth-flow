# Postgres Support Golden Path (Trigger Pilot) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add PostgreSQL as an alternative persistence backend for reearth-flow behind the existing `repo.Container` interfaces, proven end-to-end on the `Trigger` entity, with a reusable SQL/transaction package (`pgxx`) in reearthx and an Atlas + sqlc golden-path recipe.

**Architecture:** Phase 0 builds a generic, driver-level `pgxx` package in the reearthx repo (a pgx/v5-backed `usecasex.Transaction`, an executor-from-context helper, error mapping, and an env-gated test helper). Phase 1 consumes it in reearth-flow to implement a Postgres `repo.Trigger` adapter — schema via Atlas (declarative `schema.hcl` → versioned SQL migrations), typed queries via sqlc (pgx/v5), with integration tests asserting parity with the Mongo repo. The running app stays on Mongo; `DB_DRIVER` selects the backend at boot, and `postgres` mode is only fully bootable once all entities are ported (per the design's A1 model).

**Tech Stack:** Go 1.24+ (server) / Go 1.26 (reearthx), `github.com/jackc/pgx/v5` + `pgxpool`, `ariga.io/atlas` CLI, `sqlc-dev/sqlc` (pgx/v5), `github.com/reearth/reearthx/usecasex`, Postgres 16.

**Spec:** `docs/superpowers/specs/2026-06-09-postgres-support-design.md`

---

## Conventions used in this plan

- **reearthx repo:** `/Users/dexter/active/reearthx` (module `github.com/reearth/reearthx`, branch off `main`).
- **flow server:** `/Users/dexter/active/reearth-flow/server/api` (module `github.com/reearth/reearth-flow/api`).
- All `git commit` messages use conventional-commit prefixes and **no AI attribution**.
- Postgres integration tests are **env-gated**: they read a Postgres admin URI from an env var and **skip** when it is absent (mirroring `mongotest`). Local runs: `export REEARTH_FLOW_DB_PG="postgres://reearth:reearth@localhost:5432/postgres?sslmode=disable"`. For reearthx's own tests the var is `REEARTH_DB_PG`.

---

## File Structure

### Phase 0 — reearthx (`/Users/dexter/active/reearthx`)

- Create: `pgxx/pgxx.go` — `DBTX` interface, context key, `Executor(ctx, db) DBTX`.
- Create: `pgxx/transaction.go` — `Transaction` + `Tx` implementing `usecasex.Transaction`.
- Create: `pgxx/errors.go` — `IsSerializationError`, `WrapError`.
- Create: `pgxx/pgxx_test.go` — unit test for `Executor` (no DB needed).
- Create: `pgxx/transaction_test.go` — integration test (env-gated) for commit/rollback/retry-error.
- Create: `pgxx/pgxtest/pgxtest.go` — env-gated test connector (unique DB per test).
- Modify: `go.mod` / `go.sum` — add `github.com/jackc/pgx/v5`.

### Phase 1 — reearth-flow (`/Users/dexter/active/reearth-flow/server/api`)

- Modify: `go.mod` — add pgx/v5 + temporary `replace` for local reearthx.
- Create: `internal/infrastructure/postgres/db/atlas.hcl` — Atlas project config.
- Create: `internal/infrastructure/postgres/db/schema.hcl` — declarative schema (source of truth).
- Create: `internal/infrastructure/postgres/db/migrations/*.sql` — generated, versioned (Atlas).
- Create: `internal/infrastructure/postgres/sqlc.yaml` — sqlc config (pgx/v5).
- Create: `internal/infrastructure/postgres/query/trigger.sql` — sqlc query annotations.
- Create: `internal/infrastructure/postgres/gen/*.go` — sqlc-generated (do not hand-edit).
- Create: `internal/infrastructure/postgres/variables.go` — `variables` JSONB <-> `[]variable.Variable`.
- Create: `internal/infrastructure/postgres/trigger.go` — `repo.Trigger` adapter.
- Create: `internal/infrastructure/postgres/trigger_test.go` — parity integration tests.
- Create: `internal/infrastructure/postgres/container.go` — `postgres.New(...)` + unported-repo guard.
- Create: `internal/infrastructure/postgres/pgtest/pgtest.go` — env-gated connector that applies migrations.
- Create: `internal/infrastructure/postgres/README.md` — golden-path recipe.
- Modify: `internal/app/config/config.go` — add `DB_Driver` and `DB_PG` config fields.
- Modify: `internal/app/repo.go` — driver selection at boot.
- Modify: `docker-compose.yml` — add `reearth-flow-postgres` service.
- Modify: `Makefile` — add `pg-up`, `sqlc`, `atlas-diff`, `atlas-lint` targets.
- Modify: `.github/workflows/ci_api.yml` — Postgres service + env var + sqlc/atlas drift check.

---

# Phase 0 — reearthx `pgxx` package

### Task 0.0: Pre-flight — Go toolchain alignment

**Why:** reearthx `main` declares `go 1.26.2`; flow declares `go 1.24.10`. A `replace` pointing flow at local reearthx will fail to build if the required module needs a newer Go toolchain than flow uses.

- [ ] **Step 1: Check both Go directives**

Run:
```bash
grep '^go ' /Users/dexter/active/reearthx/go.mod
grep '^go ' /Users/dexter/active/reearth-flow/server/api/go.mod
```
Expected: reearthx `go 1.26.2`, flow `go 1.24.10`.

- [ ] **Step 2: Decide and record the alignment strategy**

If reearthx `main` requires a newer Go than flow's toolchain, choose ONE and note it in the plan's PR description:
- **(a) Preferred:** create the `pgxx` branch from the reearthx commit flow already consumes (`v0.0.0-20251202081949-5abca579aec6`) so the `go` directive matches what flow can build:
  ```bash
  cd /Users/dexter/active/reearthx
  git fetch origin
  git checkout -b feat/pgxx 5abca579aec6
  grep '^go ' go.mod   # confirm <= 1.24.x
  ```
- **(b) Alternative:** branch from `main` and bump flow's toolchain to match (`go 1.26` in `server/api/go.mod`) — larger blast radius; only if the team already intends to upgrade.

Default to **(a)**. The rest of Phase 0 assumes you are on a `feat/pgxx` branch in reearthx whose `go` directive flow can build.

- [ ] **Step 3: Confirm clean working tree on the chosen branch**

Run: `cd /Users/dexter/active/reearthx && git status --short && git branch --show-current`
Expected: branch `feat/pgxx`, clean tree.

### Task 0.1: Add pgx dependency

**Files:**
- Modify: `/Users/dexter/active/reearthx/go.mod`

- [ ] **Step 1: Add pgx/v5**

Run:
```bash
cd /Users/dexter/active/reearthx
go get github.com/jackc/pgx/v5@v5.7.2
```
Expected: `go.mod` gains `github.com/jackc/pgx/v5 v5.7.2`.

- [ ] **Step 2: Verify it tidies**

Run: `cd /Users/dexter/active/reearthx && go mod tidy && go build ./...`
Expected: no errors.

- [ ] **Step 3: Commit**

```bash
cd /Users/dexter/active/reearthx
git add go.mod go.sum
git commit -m "chore(pgxx): add jackc/pgx/v5 dependency"
```

### Task 0.2: Executor + context helper (unit-testable, no DB)

**Files:**
- Create: `/Users/dexter/active/reearthx/pgxx/pgxx.go`
- Create: `/Users/dexter/active/reearthx/pgxx/pgxx_test.go`

- [ ] **Step 1: Write the failing test**

`pgxx/pgxx_test.go`:
```go
package pgxx_test

import (
	"context"
	"testing"

	"github.com/jackc/pgx/v5/pgxpool"
	"github.com/reearth/reearthx/pgxx"
	"github.com/stretchr/testify/assert"
)

func TestExecutor_ReturnsPoolWhenNoTx(t *testing.T) {
	var pool *pgxpool.Pool // nil pool is fine; we only check identity, not use
	got := pgxx.Executor(context.Background(), pool)
	// With no tx in context, Executor returns the pool it was given.
	assert.Equal(t, pgxx.DBTX(pool), got)
}

func TestExecutor_ReturnsTxFromContext(t *testing.T) {
	ctx := pgxx.ContextWithTx(context.Background(), fakeTx{})
	got := pgxx.Executor(ctx, (*pgxpool.Pool)(nil))
	_, isFake := got.(fakeTx)
	assert.True(t, isFake, "Executor must return the tx stored in context")
}

// fakeTx satisfies pgx.Tx minimally enough to be stored/retrieved.
type fakeTx struct{ pgxNoopTx }
```

`pgx.Tx` is a large interface; to keep the test small we embed a no-op stub. Add this stub to the test file:
```go
import "github.com/jackc/pgx/v5"

// pgxNoopTx embeds pgx.Tx so fakeTx satisfies it without implementing every method.
type pgxNoopTx struct{ pgx.Tx }
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd /Users/dexter/active/reearthx && go test ./pgxx/ -run TestExecutor -v`
Expected: FAIL — `undefined: pgxx.Executor`, `pgxx.DBTX`, `pgxx.ContextWithTx`.

- [ ] **Step 3: Write the implementation**

`pgxx/pgxx.go`:
```go
// Package pgxx provides reusable PostgreSQL building blocks for reearth services:
// a pgx-backed usecasex.Transaction, an executor-from-context helper that lets
// repositories transparently participate in a transaction, and error helpers.
package pgxx

import (
	"context"

	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgconn"
)

// DBTX is the minimal query surface shared by *pgxpool.Pool and pgx.Tx.
// It matches the interface sqlc generates for the pgx/v5 driver, so a value of
// this type is assignable to a generated package's DBTX parameter.
type DBTX interface {
	Exec(context.Context, string, ...any) (pgconn.CommandTag, error)
	Query(context.Context, string, ...any) (pgx.Rows, error)
	QueryRow(context.Context, string, ...any) pgx.Row
}

type txKey struct{}

// ContextWithTx returns a copy of ctx carrying tx. Used by Transaction.Begin;
// exported so tests (and advanced callers) can inject a transaction.
func ContextWithTx(ctx context.Context, tx pgx.Tx) context.Context {
	return context.WithValue(ctx, txKey{}, tx)
}

func txFromContext(ctx context.Context) (pgx.Tx, bool) {
	tx, ok := ctx.Value(txKey{}).(pgx.Tx)
	return tx, ok
}

// Executor returns the active transaction stored in ctx if present, otherwise
// the supplied db (typically a *pgxpool.Pool). Repositories build their sqlc
// Queries with Executor(ctx, pool) so writes inside a usecasex.Transaction run
// on the transaction's connection automatically.
func Executor(ctx context.Context, db DBTX) DBTX {
	if tx, ok := txFromContext(ctx); ok {
		return tx
	}
	return db
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cd /Users/dexter/active/reearthx && go test ./pgxx/ -run TestExecutor -v`
Expected: PASS (both sub-tests).

- [ ] **Step 5: Commit**

```bash
cd /Users/dexter/active/reearthx
git add pgxx/pgxx.go pgxx/pgxx_test.go
git commit -m "feat(pgxx): add DBTX and executor-from-context helper"
```

### Task 0.3: Error helpers

**Files:**
- Create: `/Users/dexter/active/reearthx/pgxx/errors.go`
- Create: `/Users/dexter/active/reearthx/pgxx/errors_test.go`

- [ ] **Step 1: Write the failing test**

`pgxx/errors_test.go`:
```go
package pgxx_test

import (
	"errors"
	"testing"

	"github.com/jackc/pgx/v5/pgconn"
	"github.com/reearth/reearthx/pgxx"
	"github.com/reearth/reearthx/usecasex"
	"github.com/stretchr/testify/assert"
)

func TestIsSerializationError(t *testing.T) {
	assert.True(t, pgxx.IsSerializationError(&pgconn.PgError{Code: "40001"}))
	assert.True(t, pgxx.IsSerializationError(&pgconn.PgError{Code: "40P01"}))
	assert.False(t, pgxx.IsSerializationError(&pgconn.PgError{Code: "23505"}))
	assert.False(t, pgxx.IsSerializationError(errors.New("nope")))
	assert.False(t, pgxx.IsSerializationError(nil))
}

func TestWrapError_SerializationBecomesRetryable(t *testing.T) {
	err := pgxx.WrapError(&pgconn.PgError{Code: "40001"})
	assert.True(t, errors.Is(err, usecasex.ErrTransaction))
}

func TestWrapError_PassesThroughOthers(t *testing.T) {
	orig := errors.New("boom")
	assert.Equal(t, orig, pgxx.WrapError(orig))
	assert.Nil(t, pgxx.WrapError(nil))
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd /Users/dexter/active/reearthx && go test ./pgxx/ -run "TestIsSerializationError|TestWrapError" -v`
Expected: FAIL — `undefined: pgxx.IsSerializationError`, `pgxx.WrapError`.

- [ ] **Step 3: Write the implementation**

`pgxx/errors.go`:
```go
package pgxx

import (
	"errors"

	"github.com/jackc/pgx/v5/pgconn"
	"github.com/reearth/reearthx/usecasex"
)

// IsSerializationError reports whether err is a Postgres serialization failure
// (40001) or deadlock (40P01) — the cases worth retrying.
func IsSerializationError(err error) bool {
	var pgErr *pgconn.PgError
	if errors.As(err, &pgErr) {
		return pgErr.Code == "40001" || pgErr.Code == "40P01"
	}
	return false
}

// WrapError maps a Postgres serialization failure to usecasex.ErrTransaction so
// the existing usecasex.DoTransaction retry loop picks it up. Other errors
// (and nil) pass through unchanged.
func WrapError(err error) error {
	if err == nil {
		return nil
	}
	if IsSerializationError(err) {
		return errors.Join(usecasex.ErrTransaction, err)
	}
	return err
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cd /Users/dexter/active/reearthx && go test ./pgxx/ -run "TestIsSerializationError|TestWrapError" -v`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
cd /Users/dexter/active/reearthx
git add pgxx/errors.go pgxx/errors_test.go
git commit -m "feat(pgxx): map serialization failures to usecasex.ErrTransaction"
```

### Task 0.4: pgx-backed `usecasex.Transaction`

**Files:**
- Create: `/Users/dexter/active/reearthx/pgxx/transaction.go`

- [ ] **Step 1: Write the implementation**

(The integration test in Task 0.6 exercises this against a real DB; this task is a compile-time contract check.)

`pgxx/transaction.go`:
```go
package pgxx

import (
	"context"

	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgxpool"
	"github.com/reearth/reearthx/usecasex"
	"go.uber.org/atomic"
)

// Transaction is a pgx-backed implementation of usecasex.Transaction.
type Transaction struct {
	pool *pgxpool.Pool
}

var _ usecasex.Transaction = (*Transaction)(nil)

func NewTransaction(pool *pgxpool.Pool) *Transaction {
	return &Transaction{pool: pool}
}

// Begin opens a pgx transaction and returns a Tx whose Context() carries it, so
// repositories using Executor(ctx, pool) run on the transaction's connection.
func (t *Transaction) Begin(ctx context.Context) (usecasex.Tx, error) {
	pgtx, err := t.pool.Begin(ctx)
	if err != nil {
		return nil, err
	}
	return &Tx{tx: pgtx, ctx: ContextWithTx(ctx, pgtx)}, nil
}

type Tx struct {
	tx        pgx.Tx
	ctx       context.Context
	committed atomic.Bool
}

var _ usecasex.Tx = (*Tx)(nil)

func (t *Tx) Context() context.Context { return t.ctx }

func (t *Tx) Commit() { t.committed.Store(true) }

func (t *Tx) IsCommitted() bool { return t.committed.Load() }

// End commits if Commit() was called, otherwise rolls back. A rollback after a
// successful commit is a no-op in pgx, so the post-commit path is safe.
func (t *Tx) End(ctx context.Context) error {
	if t.committed.Load() {
		return t.tx.Commit(ctx)
	}
	return t.tx.Rollback(ctx)
}
```

- [ ] **Step 2: Verify it compiles and satisfies the interface**

Run: `cd /Users/dexter/active/reearthx && go build ./pgxx/`
Expected: no errors (the `var _ usecasex.Transaction` / `usecasex.Tx` assertions confirm conformance).

- [ ] **Step 3: Commit**

```bash
cd /Users/dexter/active/reearthx
git add pgxx/transaction.go
git commit -m "feat(pgxx): add pgx-backed usecasex.Transaction"
```

### Task 0.5: Env-gated test connector

**Files:**
- Create: `/Users/dexter/active/reearthx/pgxx/pgxtest/pgxtest.go`

- [ ] **Step 1: Write the implementation**

`pgxx/pgxtest/pgxtest.go`:
```go
// Package pgxtest provides an env-gated Postgres connection for tests, mirroring
// reearthx/mongox/mongotest: tests skip when no DB URI is configured, and each
// call creates an isolated, uniquely-named database that is dropped on cleanup.
package pgxtest

import (
	"context"
	"os"
	"strings"
	"testing"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5/pgxpool"
)

// Env is the environment variable holding an admin Postgres URI
// (e.g. postgres://user:pass@localhost:5432/postgres?sslmode=disable).
var Env = "REEARTH_DB_PG"

// Connect returns a factory that yields an isolated *pgxpool.Pool per call.
// It t.Skip()s when Env is unset, matching mongotest semantics.
func Connect(t *testing.T) func(*testing.T) *pgxpool.Pool {
	t.Helper()

	adminURI := os.Getenv(Env)
	if adminURI == "" {
		t.Skipf("pgxtest: %s not set; skipping Postgres integration test", Env)
		return nil
	}

	ctx := context.Background()
	admin, err := pgxpool.New(ctx, adminURI)
	if err != nil {
		t.Fatalf("pgxtest: connect admin: %v", err)
	}

	return func(t *testing.T) *pgxpool.Pool {
		t.Helper()

		dbName := "reearth_test_" + strings.ReplaceAll(uuid.NewString(), "-", "")
		if _, err := admin.Exec(ctx, "CREATE DATABASE "+dbName); err != nil {
			t.Fatalf("pgxtest: create database: %v", err)
		}
		t.Cleanup(func() {
			_, _ = admin.Exec(ctx, "DROP DATABASE IF EXISTS "+dbName+" WITH (FORCE)")
		})

		pool, err := pgxpool.New(ctx, replaceDBName(adminURI, dbName))
		if err != nil {
			t.Fatalf("pgxtest: connect test db: %v", err)
		}
		t.Cleanup(pool.Close)
		return pool
	}
}

// replaceDBName swaps the path component (database name) of a Postgres URI.
func replaceDBName(uri, dbName string) string {
	// uri form: scheme://user:pass@host:port/dbname?query
	q := ""
	if i := strings.IndexByte(uri, '?'); i >= 0 {
		q = uri[i:]
		uri = uri[:i]
	}
	if i := strings.LastIndexByte(uri, '/'); i >= 0 {
		uri = uri[:i]
	}
	return uri + "/" + dbName + q
}
```

- [ ] **Step 2: Add the uuid dependency if missing, build**

Run:
```bash
cd /Users/dexter/active/reearthx
go get github.com/google/uuid
go build ./pgxx/...
```
Expected: builds (uuid is already an indirect dep of reearthx via mongotest).

- [ ] **Step 3: Commit**

```bash
cd /Users/dexter/active/reearthx
git add pgxx/pgxtest/pgxtest.go go.mod go.sum
git commit -m "feat(pgxx): add env-gated pgxtest connector"
```

### Task 0.6: Transaction integration test (env-gated)

**Files:**
- Create: `/Users/dexter/active/reearthx/pgxx/transaction_test.go`

- [ ] **Step 1: Write the test**

`pgxx/transaction_test.go`:
```go
package pgxx_test

import (
	"context"
	"testing"

	"github.com/reearth/reearthx/pgxx"
	"github.com/reearth/reearthx/pgxx/pgxtest"
	"github.com/reearth/reearthx/usecasex"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func setupScratch(t *testing.T) (context.Context, usecasex.Transaction, queryFn) {
	pool := pgxtest.Connect(t)(t)
	ctx := context.Background()
	_, err := pool.Exec(ctx, `CREATE TABLE items (id text PRIMARY KEY)`)
	require.NoError(t, err)

	count := func(ctx context.Context) int {
		var n int
		err := pgxx.Executor(ctx, pool).QueryRow(ctx, `SELECT count(*) FROM items`).Scan(&n)
		require.NoError(t, err)
		return n
	}
	insert := func(ctx context.Context, id string) error {
		_, err := pgxx.Executor(ctx, pool).Exec(ctx, `INSERT INTO items (id) VALUES ($1)`, id)
		return err
	}
	return ctx, pgxx.NewTransaction(pool), queryFn{count: count, insert: insert}
}

type queryFn struct {
	count  func(context.Context) int
	insert func(context.Context, string) error
}

func TestTransaction_CommitsOnCommit(t *testing.T) {
	ctx, tr, q := setupScratch(t)

	tx, err := tr.Begin(ctx)
	require.NoError(t, err)
	require.NoError(t, q.insert(tx.Context(), "a"))
	tx.Commit()
	require.NoError(t, tx.End(tx.Context()))

	assert.Equal(t, 1, q.count(ctx))
}

func TestTransaction_RollsBackWithoutCommit(t *testing.T) {
	ctx, tr, q := setupScratch(t)

	tx, err := tr.Begin(ctx)
	require.NoError(t, err)
	require.NoError(t, q.insert(tx.Context(), "a"))
	// no Commit()
	require.NoError(t, tx.End(tx.Context()))

	assert.Equal(t, 0, q.count(ctx))
}

func TestTransaction_DoTransactionCommits(t *testing.T) {
	ctx, tr, q := setupScratch(t)

	err := usecasex.DoTransaction(ctx, tr, 1, func(ctx context.Context) error {
		return q.insert(ctx, "x")
	})
	require.NoError(t, err)
	assert.Equal(t, 1, q.count(ctx))
}
```

- [ ] **Step 2: Run it WITHOUT a DB to confirm it skips**

Run: `cd /Users/dexter/active/reearthx && go test ./pgxx/ -run TestTransaction -v`
Expected: SKIP (`pgxtest: REEARTH_DB_PG not set`).

- [ ] **Step 3: Run it WITH a DB to confirm it passes**

Run:
```bash
docker run --rm -d --name pgxx-test -e POSTGRES_USER=reearth -e POSTGRES_PASSWORD=reearth -p 5433:5432 postgres:16-alpine
sleep 3
cd /Users/dexter/active/reearthx
REEARTH_DB_PG="postgres://reearth:reearth@localhost:5433/postgres?sslmode=disable" go test ./pgxx/ -run TestTransaction -v
docker rm -f pgxx-test
```
Expected: PASS (3 tests).

- [ ] **Step 4: Commit**

```bash
cd /Users/dexter/active/reearthx
git add pgxx/transaction_test.go
git commit -m "test(pgxx): integration tests for pgx transaction commit/rollback"
```

### Task 0.7: Push reearthx branch

- [ ] **Step 1: Push and note the commit SHA**

Run:
```bash
cd /Users/dexter/active/reearthx
git push -u origin feat/pgxx
git rev-parse HEAD
```
Expected: branch pushed; record the SHA — Phase 1 uses a local `replace` during development and this SHA for the eventual `go.mod` bump.

---

# Phase 1 — reearth-flow Trigger pilot

All paths below are under `/Users/dexter/active/reearth-flow/server/api` unless noted. Work on the existing `feat/postgres-support` branch (where the spec was committed).

### Task 1.1: Wire flow to local reearthx + add pgx

**Files:**
- Modify: `go.mod`

- [ ] **Step 1: Add a temporary replace directive + pgx**

Run:
```bash
cd /Users/dexter/active/reearth-flow/server/api
go mod edit -replace github.com/reearth/reearthx=/Users/dexter/active/reearthx
go get github.com/jackc/pgx/v5@v5.7.2
go mod tidy
```
Expected: `go.mod` has a `replace github.com/reearth/reearthx => /Users/dexter/active/reearthx` line and requires `github.com/jackc/pgx/v5`.

- [ ] **Step 2: Verify the pgxx package resolves**

Run: `cd /Users/dexter/active/reearth-flow/server/api && go list github.com/reearth/reearthx/pgxx`
Expected: prints the package path (no error). If it errors with a Go version mismatch, revisit Task 0.0 step 2.

- [ ] **Step 3: Commit**

```bash
cd /Users/dexter/active/reearth-flow
git add server/api/go.mod server/api/go.sum
git commit -m "chore(server): add pgx/v5 and local reearthx replace for pgxx (temporary)"
```

> NOTE: the `replace` is temporary. The final landing PR removes it and bumps the reearthx version to the Task 0.7 SHA.

### Task 1.2: Atlas schema + first migration

**Files:**
- Create: `internal/infrastructure/postgres/db/atlas.hcl`
- Create: `internal/infrastructure/postgres/db/schema.hcl`
- Create: `internal/infrastructure/postgres/db/migrations/` (generated)

- [ ] **Step 1: Install Atlas (if needed)**

Run: `atlas version || curl -sSf https://atlasgo.sh | sh`
Expected: `atlas version vX.Y.Z`.

- [ ] **Step 2: Write the declarative schema**

`internal/infrastructure/postgres/db/schema.hcl`:
```hcl
schema "public" {}

table "triggers" {
  schema = schema.public
  column "id"             { type = text }
  column "workspace_id"   { type = text }
  column "deployment_id"  { type = text }
  column "description"     { type = text, default = "" }
  column "event_source"   { type = text }
  column "time_interval"  { type = text, null = true }
  column "auth_token"     { type = text, null = true }
  column "enabled"        { type = boolean, default = false }
  column "last_triggered" { type = timestamptz, null = true }
  column "variables"      { type = jsonb, null = true }
  column "created_at"     { type = timestamptz }
  column "updated_at"     { type = timestamptz }
  primary_key { columns = [column.id] }
  index "triggers_workspace_id_idx"  { columns = [column.workspace_id] }
  index "triggers_deployment_id_idx" { columns = [column.deployment_id] }
}
```

- [ ] **Step 3: Write the Atlas project config**

`internal/infrastructure/postgres/db/atlas.hcl`:
```hcl
env "local" {
  src = "file://schema.hcl"
  dev = "docker://postgres/16/dev?search_path=public"
  migration {
    dir = "file://migrations"
  }
  format {
    migrate {
      diff = "{{ sql . \"  \" }}"
    }
  }
}
```

- [ ] **Step 4: Generate the first migration**

Run:
```bash
cd /Users/dexter/active/reearth-flow/server/api/internal/infrastructure/postgres/db
atlas migrate diff initial_triggers --env local
```
Expected: creates `migrations/<timestamp>_initial_triggers.sql` (CREATE TABLE triggers + indexes) and `migrations/atlas.sum`.

- [ ] **Step 5: Lint the migration**

Run: `cd /Users/dexter/active/reearth-flow/server/api/internal/infrastructure/postgres/db && atlas migrate lint --env local --latest 1`
Expected: no errors.

- [ ] **Step 6: Commit**

```bash
cd /Users/dexter/active/reearth-flow
git add server/api/internal/infrastructure/postgres/db
git commit -m "feat(server/postgres): add Atlas declarative schema and initial triggers migration"
```

### Task 1.3: sqlc config + queries + generate

**Files:**
- Create: `internal/infrastructure/postgres/sqlc.yaml`
- Create: `internal/infrastructure/postgres/query/trigger.sql`
- Create: `internal/infrastructure/postgres/gen/*` (generated)

- [ ] **Step 1: Write the sqlc config**

`internal/infrastructure/postgres/sqlc.yaml`:
```yaml
version: "2"
sql:
  - engine: "postgresql"
    schema: "db/migrations"
    queries: "query"
    gen:
      go:
        package: "gen"
        out: "gen"
        sql_package: "pgx/v5"
        emit_json_tags: false
        emit_interface: false
        emit_exact_table_names: false
```

- [ ] **Step 2: Write the queries**

`internal/infrastructure/postgres/query/trigger.sql`:
```sql
-- name: GetTrigger :one
SELECT * FROM triggers WHERE id = $1;

-- name: ListTriggersByIDs :many
SELECT * FROM triggers WHERE id = ANY($1::text[]);

-- name: ListTriggersByDeployment :many
SELECT * FROM triggers WHERE deployment_id = $1;

-- name: UpsertTrigger :exec
INSERT INTO triggers (
  id, workspace_id, deployment_id, description, event_source,
  time_interval, auth_token, enabled, last_triggered, variables,
  created_at, updated_at
) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12)
ON CONFLICT (id) DO UPDATE SET
  workspace_id   = EXCLUDED.workspace_id,
  deployment_id  = EXCLUDED.deployment_id,
  description    = EXCLUDED.description,
  event_source   = EXCLUDED.event_source,
  time_interval  = EXCLUDED.time_interval,
  auth_token     = EXCLUDED.auth_token,
  enabled        = EXCLUDED.enabled,
  last_triggered = EXCLUDED.last_triggered,
  variables      = EXCLUDED.variables,
  created_at     = EXCLUDED.created_at,
  updated_at     = EXCLUDED.updated_at;

-- name: DeleteTrigger :exec
DELETE FROM triggers WHERE id = $1;
```

- [ ] **Step 3: Generate**

Run:
```bash
cd /Users/dexter/active/reearth-flow/server/api/internal/infrastructure/postgres
go tool sqlc generate || (go get -tool github.com/sqlc-dev/sqlc/cmd/sqlc@v1.27.0 && go tool sqlc generate)
```
Expected: creates `gen/db.go`, `gen/models.go`, `gen/trigger.sql.go`. Inspect `gen/models.go` — the `Trigger` struct should have `ID/WorkspaceID/DeploymentID/Description/EventSource string`, `TimeInterval/AuthToken pgtype.Text`, `Enabled bool`, `LastTriggered pgtype.Timestamptz`, `Variables []byte`, `CreatedAt/UpdatedAt time.Time`.

> If `go tool sqlc` is unavailable, install the binary: `go install github.com/sqlc-dev/sqlc/cmd/sqlc@v1.27.0` and run `sqlc generate`.

- [ ] **Step 4: Verify it builds**

Run: `cd /Users/dexter/active/reearth-flow/server/api && go build ./internal/infrastructure/postgres/gen/`
Expected: no errors.

- [ ] **Step 5: Commit**

```bash
cd /Users/dexter/active/reearth-flow
git add server/api/internal/infrastructure/postgres/sqlc.yaml \
        server/api/internal/infrastructure/postgres/query \
        server/api/internal/infrastructure/postgres/gen \
        server/api/go.mod server/api/go.sum
git commit -m "feat(server/postgres): add sqlc config, trigger queries, generated code"
```

### Task 1.4: variables JSONB mapping

**Files:**
- Create: `internal/infrastructure/postgres/variables.go`
- Create: `internal/infrastructure/postgres/variables_test.go`

- [ ] **Step 1: Write the failing test**

`internal/infrastructure/postgres/variables_test.go`:
```go
package postgres

import (
	"testing"

	"github.com/reearth/reearth-flow/api/pkg/parameter"
	"github.com/reearth/reearth-flow/api/pkg/variable"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestVariablesJSONRoundTrip(t *testing.T) {
	in := []variable.Variable{
		{Key: "a", Type: parameter.Type("text"), Value: "hello"},
		{Key: "n", Type: parameter.Type("number"), Value: float64(3)},
	}

	b, err := variablesToJSON(in)
	require.NoError(t, err)

	out, err := variablesFromJSON(b)
	require.NoError(t, err)
	assert.Equal(t, in, out)
}

func TestVariablesJSON_EmptyAndNil(t *testing.T) {
	b, err := variablesToJSON(nil)
	require.NoError(t, err)
	assert.Nil(t, b)

	out, err := variablesFromJSON(nil)
	require.NoError(t, err)
	assert.Nil(t, out)
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd /Users/dexter/active/reearth-flow/server/api && go test ./internal/infrastructure/postgres/ -run TestVariablesJSON -v`
Expected: FAIL — `undefined: variablesToJSON`, `variablesFromJSON`.

- [ ] **Step 3: Write the implementation**

`internal/infrastructure/postgres/variables.go`:
```go
package postgres

import (
	"encoding/json"

	"github.com/reearth/reearth-flow/api/pkg/parameter"
	"github.com/reearth/reearth-flow/api/pkg/variable"
)

// variableJSON is the JSONB shape for trigger.variables. Field names mirror the
// Mongo VariableDocument so data is conceptually consistent across backends.
type variableJSON struct {
	Key   string `json:"key"`
	Type  string `json:"type"`
	Value any    `json:"value"`
}

func variablesToJSON(vars []variable.Variable) ([]byte, error) {
	if len(vars) == 0 {
		return nil, nil
	}
	out := make([]variableJSON, 0, len(vars))
	for _, v := range vars {
		out = append(out, variableJSON{Key: v.Key, Type: string(v.Type), Value: v.Value})
	}
	return json.Marshal(out)
}

func variablesFromJSON(b []byte) ([]variable.Variable, error) {
	if len(b) == 0 {
		return nil, nil
	}
	var docs []variableJSON
	if err := json.Unmarshal(b, &docs); err != nil {
		return nil, err
	}
	if len(docs) == 0 {
		return nil, nil
	}
	out := make([]variable.Variable, 0, len(docs))
	for _, d := range docs {
		out = append(out, variable.Variable{Key: d.Key, Type: parameter.Type(d.Type), Value: d.Value})
	}
	return out, nil
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cd /Users/dexter/active/reearth-flow/server/api && go test ./internal/infrastructure/postgres/ -run TestVariablesJSON -v`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
cd /Users/dexter/active/reearth-flow
git add server/api/internal/infrastructure/postgres/variables.go \
        server/api/internal/infrastructure/postgres/variables_test.go
git commit -m "feat(server/postgres): add trigger variables JSONB mapping"
```

### Task 1.5: pgtest helper (applies migrations)

**Files:**
- Create: `internal/infrastructure/postgres/pgtest/pgtest.go`

- [ ] **Step 1: Write the implementation**

`internal/infrastructure/postgres/pgtest/pgtest.go`:
```go
// Package pgtest wraps reearthx pgxtest with reearth-flow's migration application,
// so each test gets an isolated database with the triggers schema already applied.
package pgtest

import (
	"context"
	"os"
	"path/filepath"
	"sort"
	"strings"
	"testing"

	"github.com/jackc/pgx/v5/pgxpool"
	"github.com/reearth/reearthx/pgxx/pgxtest"
)

func init() {
	// reuse the flow-specific env var name
	pgxtest.Env = "REEARTH_FLOW_DB_PG"
}

// Connect returns a factory yielding an isolated *pgxpool.Pool with all
// migrations applied. Skips when REEARTH_FLOW_DB_PG is unset.
func Connect(t *testing.T) func(*testing.T) *pgxpool.Pool {
	t.Helper()
	base := pgxtest.Connect(t)
	if base == nil {
		return nil
	}
	return func(t *testing.T) *pgxpool.Pool {
		pool := base(t)
		applyMigrations(t, pool)
		return pool
	}
}

func applyMigrations(t *testing.T, pool *pgxpool.Pool) {
	t.Helper()
	dir := migrationsDir(t)
	entries, err := os.ReadDir(dir)
	if err != nil {
		t.Fatalf("pgtest: read migrations dir: %v", err)
	}
	var files []string
	for _, e := range entries {
		if strings.HasSuffix(e.Name(), ".sql") {
			files = append(files, e.Name())
		}
	}
	sort.Strings(files)
	ctx := context.Background()
	for _, f := range files {
		sqlBytes, err := os.ReadFile(filepath.Join(dir, f))
		if err != nil {
			t.Fatalf("pgtest: read migration %s: %v", f, err)
		}
		if _, err := pool.Exec(ctx, string(sqlBytes)); err != nil {
			t.Fatalf("pgtest: apply migration %s: %v", f, err)
		}
	}
}

// migrationsDir resolves the migrations directory relative to this source file,
// independent of the test's working directory.
func migrationsDir(t *testing.T) string {
	t.Helper()
	// pgtest lives at internal/infrastructure/postgres/pgtest; migrations are one level up.
	wd, err := os.Getwd()
	if err != nil {
		t.Fatalf("pgtest: getwd: %v", err)
	}
	// Tests for the postgres package run with wd = .../internal/infrastructure/postgres
	candidate := filepath.Join(wd, "db", "migrations")
	if _, err := os.Stat(candidate); err == nil {
		return candidate
	}
	// Fallback: walk up to find db/migrations
	d := wd
	for i := 0; i < 5; i++ {
		c := filepath.Join(d, "internal", "infrastructure", "postgres", "db", "migrations")
		if _, err := os.Stat(c); err == nil {
			return c
		}
		d = filepath.Dir(d)
	}
	t.Fatalf("pgtest: could not locate db/migrations from %s", wd)
	return ""
}
```

> NOTE: `atlas.sum` and any non-`.sql` files are ignored. Migration files contain plain SQL (no `\` psql meta-commands), so `pool.Exec` runs them via the simple protocol, which supports multiple statements per call.

- [ ] **Step 2: Build**

Run: `cd /Users/dexter/active/reearth-flow/server/api && go build ./internal/infrastructure/postgres/pgtest/`
Expected: no errors.

- [ ] **Step 3: Commit**

```bash
cd /Users/dexter/active/reearth-flow
git add server/api/internal/infrastructure/postgres/pgtest
git commit -m "feat(server/postgres): add pgtest helper that applies migrations"
```

### Task 1.6: Trigger adapter — mapping + Save + FindByID

**Files:**
- Create: `internal/infrastructure/postgres/trigger.go`
- Create: `internal/infrastructure/postgres/trigger_test.go`

- [ ] **Step 1: Write the failing test**

`internal/infrastructure/postgres/trigger_test.go`:
```go
package postgres_test

import (
	"context"
	"testing"
	"time"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/postgres"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/postgres/pgtest"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/trigger"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestTrigger_Save_FindByID(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()

	tid := id.NewTriggerID()
	wid := accountsid.NewWorkspaceID()
	did := id.NewDeploymentID()

	tr := trigger.New().
		ID(tid).
		Workspace(wid).
		Deployment(did).
		Description("desc").
		EventSource(trigger.EventSourceTypeTimeDriven).
		TimeInterval(trigger.TimeIntervalEveryDay).
		Enabled(true).
		CreatedAt(time.Now()).
		UpdatedAt(time.Now()).
		MustBuild()

	r := postgres.NewTrigger(pool)
	require.NoError(t, r.Save(ctx, tr))

	got, err := r.FindByID(ctx, tid)
	require.NoError(t, err)
	assert.Equal(t, tid, got.ID())
	assert.Equal(t, wid, got.Workspace())
	assert.Equal(t, did, got.Deployment())
	assert.Equal(t, "desc", got.Description())
	assert.Equal(t, trigger.EventSourceTypeTimeDriven, got.EventSource())
	assert.True(t, got.Enabled())
}

func TestTrigger_FindByID_NotFound(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()

	got, err := postgres.NewTrigger(pool).FindByID(ctx, id.NewTriggerID())
	assert.Error(t, err)
	assert.Nil(t, got)
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd /Users/dexter/active/reearth-flow/server/api && go test ./internal/infrastructure/postgres/ -run TestTrigger_Save_FindByID -v`
Expected: FAIL — `undefined: postgres.NewTrigger` (or SKIP if `REEARTH_FLOW_DB_PG` unset; set it to get a real FAIL).

- [ ] **Step 3: Write the implementation**

`internal/infrastructure/postgres/trigger.go`:
```go
package postgres

import (
	"context"
	"errors"

	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgtype"
	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/postgres/gen"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/trigger"
	"github.com/reearth/reearthx/pgxx"
	"github.com/reearth/reearthx/rerror"
)

type Trigger struct {
	pool pgxx.DBTX
	f    repo.WorkspaceFilter
}

func NewTrigger(pool pgxx.DBTX) *Trigger {
	return &Trigger{pool: pool}
}

func (r *Trigger) Filtered(f repo.WorkspaceFilter) repo.Trigger {
	return &Trigger{pool: r.pool, f: r.f.Merge(f)}
}

func (r *Trigger) q(ctx context.Context) *gen.Queries {
	return gen.New(pgxx.Executor(ctx, r.pool))
}

func (r *Trigger) FindByID(ctx context.Context, tid id.TriggerID) (*trigger.Trigger, error) {
	row, err := r.q(ctx).GetTrigger(ctx, tid.String())
	if err != nil {
		if errors.Is(err, pgx.ErrNoRows) {
			return nil, rerror.ErrNotFound
		}
		return nil, rerror.ErrInternalByWithContext(ctx, pgxx.WrapError(err))
	}
	t, err := triggerFromRow(row)
	if err != nil {
		return nil, err
	}
	if !r.f.CanRead(t.Workspace()) {
		return nil, rerror.ErrNotFound
	}
	return t, nil
}

func (r *Trigger) Save(ctx context.Context, t *trigger.Trigger) error {
	if !r.f.CanWrite(t.Workspace()) {
		return repo.ErrOperationDenied
	}
	params, err := triggerToParams(t)
	if err != nil {
		return rerror.ErrInternalByWithContext(ctx, err)
	}
	if err := r.q(ctx).UpsertTrigger(ctx, params); err != nil {
		return rerror.ErrInternalByWithContext(ctx, pgxx.WrapError(err))
	}
	return nil
}

// --- mapping helpers ---

func triggerToParams(t *trigger.Trigger) (gen.UpsertTriggerParams, error) {
	vars, err := variablesToJSON(t.Variables())
	if err != nil {
		return gen.UpsertTriggerParams{}, err
	}
	p := gen.UpsertTriggerParams{
		ID:           t.ID().String(),
		WorkspaceID:  t.Workspace().String(),
		DeploymentID: t.Deployment().String(),
		Description:  t.Description(),
		EventSource:  string(t.EventSource()),
		Enabled:      t.Enabled(),
		Variables:    vars,
		CreatedAt:    t.CreatedAt(),
		UpdatedAt:    t.UpdatedAt(),
	}
	if ti := t.TimeInterval(); ti != nil {
		p.TimeInterval = pgtype.Text{String: string(*ti), Valid: true}
	}
	if at := t.AuthToken(); at != nil {
		p.AuthToken = pgtype.Text{String: *at, Valid: true}
	}
	if lt := t.LastTriggered(); lt != nil {
		p.LastTriggered = pgtype.Timestamptz{Time: *lt, Valid: true}
	}
	return p, nil
}

func triggerFromRow(row gen.Trigger) (*trigger.Trigger, error) {
	tid, err := id.TriggerIDFrom(row.ID)
	if err != nil {
		return nil, err
	}
	wid, err := accountsid.WorkspaceIDFrom(row.WorkspaceID)
	if err != nil {
		return nil, err
	}
	did, err := id.DeploymentIDFrom(row.DeploymentID)
	if err != nil {
		return nil, err
	}

	b := trigger.New().
		ID(tid).
		Workspace(wid).
		Deployment(did).
		Description(row.Description).
		EventSource(trigger.EventSourceType(row.EventSource)).
		Enabled(row.Enabled).
		CreatedAt(row.CreatedAt).
		UpdatedAt(row.UpdatedAt)

	if row.TimeInterval.Valid {
		b = b.TimeInterval(trigger.TimeInterval(row.TimeInterval.String))
	}
	if row.AuthToken.Valid {
		b = b.AuthToken(row.AuthToken.String)
	}
	if row.LastTriggered.Valid {
		b = b.LastTriggered(row.LastTriggered.Time)
	}

	vars, err := variablesFromJSON(row.Variables)
	if err != nil {
		return nil, err
	}
	if len(vars) > 0 {
		b = b.Variables(vars)
	}

	return b.Build()
}
```

> NOTE: `trigger.Build()` requires non-empty `description` and `event_source`. The test always sets them. If a future caller persists a trigger with an empty description (the Mongo `Save` allows it), `Build()` would fail on read — this matches a latent constraint in the domain builder and is out of scope here.

- [ ] **Step 4: Run test to verify it passes (with DB)**

Run:
```bash
docker run --rm -d --name flow-pg-test -e POSTGRES_USER=reearth -e POSTGRES_PASSWORD=reearth -p 5434:5432 postgres:16-alpine
sleep 3
cd /Users/dexter/active/reearth-flow/server/api
REEARTH_FLOW_DB_PG="postgres://reearth:reearth@localhost:5434/postgres?sslmode=disable" \
  go test ./internal/infrastructure/postgres/ -run "TestTrigger_Save_FindByID|TestTrigger_FindByID_NotFound" -v
docker rm -f flow-pg-test
```
Expected: PASS (2 tests).

- [ ] **Step 5: Commit**

```bash
cd /Users/dexter/active/reearth-flow
git add server/api/internal/infrastructure/postgres/trigger.go \
        server/api/internal/infrastructure/postgres/trigger_test.go
git commit -m "feat(server/postgres): trigger adapter Save + FindByID with mapping"
```

### Task 1.7: FindByIDs (input-order preservation)

**Files:**
- Modify: `internal/infrastructure/postgres/trigger.go`
- Modify: `internal/infrastructure/postgres/trigger_test.go`

- [ ] **Step 1: Add the failing test**

Append to `trigger_test.go`:
```go
func TestTrigger_FindByIDs_Order(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	r := postgres.NewTrigger(pool)

	wid := accountsid.NewWorkspaceID()
	did := id.NewDeploymentID()
	tid1 := id.NewTriggerID()
	tid2 := id.NewTriggerID()
	for _, x := range []id.TriggerID{tid1, tid2} {
		require.NoError(t, r.Save(ctx, trigger.New().ID(x).Workspace(wid).Deployment(did).
			Description("d").EventSource(trigger.EventSourceTypeTimeDriven).
			CreatedAt(time.Now()).UpdatedAt(time.Now()).MustBuild()))
	}

	// request in reverse + a missing id => order preserved, missing => nil
	missing := id.NewTriggerID()
	got, err := r.FindByIDs(ctx, id.TriggerIDList{tid2, missing, tid1})
	require.NoError(t, err)
	require.Len(t, got, 3)
	assert.Equal(t, tid2, got[0].ID())
	assert.Nil(t, got[1])
	assert.Equal(t, tid1, got[2].ID())
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cd /Users/dexter/active/reearth-flow/server/api && go test ./internal/infrastructure/postgres/ -run TestTrigger_FindByIDs_Order -v` (with `REEARTH_FLOW_DB_PG` set)
Expected: FAIL — `r.FindByIDs undefined`.

- [ ] **Step 3: Implement**

Add to `trigger.go`:
```go
func (r *Trigger) FindByIDs(ctx context.Context, ids id.TriggerIDList) ([]*trigger.Trigger, error) {
	rows, err := r.q(ctx).ListTriggersByIDs(ctx, ids.Strings())
	if err != nil {
		return nil, rerror.ErrInternalByWithContext(ctx, pgxx.WrapError(err))
	}

	byID := make(map[string]*trigger.Trigger, len(rows))
	for _, row := range rows {
		t, err := triggerFromRow(row)
		if err != nil {
			return nil, err
		}
		if r.f.CanRead(t.Workspace()) {
			byID[t.ID().String()] = t
		}
	}

	res := make([]*trigger.Trigger, 0, len(ids))
	for _, tid := range ids {
		res = append(res, byID[tid.String()]) // nil when missing/unreadable
	}
	return res, nil
}
```

- [ ] **Step 4: Run to verify it passes**

Run: same as Step 2.
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
cd /Users/dexter/active/reearth-flow
git add server/api/internal/infrastructure/postgres/trigger.go \
        server/api/internal/infrastructure/postgres/trigger_test.go
git commit -m "feat(server/postgres): trigger FindByIDs preserving input order"
```

### Task 1.8: FindByDeployment

**Files:**
- Modify: `internal/infrastructure/postgres/trigger.go`
- Modify: `internal/infrastructure/postgres/trigger_test.go`

- [ ] **Step 1: Add the failing test**

Append to `trigger_test.go`:
```go
func TestTrigger_FindByDeployment(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	r := postgres.NewTrigger(pool)

	wid := accountsid.NewWorkspaceID()
	did := id.NewDeploymentID()
	other := id.NewDeploymentID()

	require.NoError(t, r.Save(ctx, trigger.New().ID(id.NewTriggerID()).Workspace(wid).Deployment(did).
		Description("d").EventSource(trigger.EventSourceTypeTimeDriven).
		CreatedAt(time.Now()).UpdatedAt(time.Now()).MustBuild()))
	require.NoError(t, r.Save(ctx, trigger.New().ID(id.NewTriggerID()).Workspace(wid).Deployment(other).
		Description("d").EventSource(trigger.EventSourceTypeTimeDriven).
		CreatedAt(time.Now()).UpdatedAt(time.Now()).MustBuild()))

	got, err := r.FindByDeployment(ctx, did)
	require.NoError(t, err)
	assert.Len(t, got, 1)
	assert.Equal(t, did, got[0].Deployment())
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `go test ./internal/infrastructure/postgres/ -run TestTrigger_FindByDeployment -v` (DB set)
Expected: FAIL — `r.FindByDeployment undefined`.

- [ ] **Step 3: Implement**

Add to `trigger.go`:
```go
func (r *Trigger) FindByDeployment(ctx context.Context, did id.DeploymentID) ([]*trigger.Trigger, error) {
	rows, err := r.q(ctx).ListTriggersByDeployment(ctx, did.String())
	if err != nil {
		return nil, rerror.ErrInternalByWithContext(ctx, pgxx.WrapError(err))
	}
	res := make([]*trigger.Trigger, 0, len(rows))
	for _, row := range rows {
		t, err := triggerFromRow(row)
		if err != nil {
			return nil, err
		}
		if r.f.CanRead(t.Workspace()) {
			res = append(res, t)
		}
	}
	return res, nil
}
```

- [ ] **Step 4: Run to verify it passes** — Expected: PASS.

- [ ] **Step 5: Commit**

```bash
cd /Users/dexter/active/reearth-flow
git add server/api/internal/infrastructure/postgres/trigger.go \
        server/api/internal/infrastructure/postgres/trigger_test.go
git commit -m "feat(server/postgres): trigger FindByDeployment"
```

### Task 1.9: Remove (+ workspace write filter)

**Files:**
- Modify: `internal/infrastructure/postgres/trigger.go`
- Modify: `internal/infrastructure/postgres/trigger_test.go`

- [ ] **Step 1: Add the failing tests**

Append to `trigger_test.go`:
```go
func newTrig(wid accountsid.WorkspaceID, did id.DeploymentID, tid id.TriggerID) *trigger.Trigger {
	return trigger.New().ID(tid).Workspace(wid).Deployment(did).
		Description("d").EventSource(trigger.EventSourceTypeTimeDriven).
		CreatedAt(time.Now()).UpdatedAt(time.Now()).MustBuild()
}

func TestTrigger_Remove(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	r := postgres.NewTrigger(pool)

	wid := accountsid.NewWorkspaceID()
	did := id.NewDeploymentID()
	tid := id.NewTriggerID()
	require.NoError(t, r.Save(ctx, newTrig(wid, did, tid)))

	require.NoError(t, r.Remove(ctx, tid))
	got, err := r.FindByID(ctx, tid)
	assert.Error(t, err)
	assert.Nil(t, got)
}

func TestTrigger_Remove_WithWorkspaceFilter(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	base := postgres.NewTrigger(pool)

	wid1 := accountsid.NewWorkspaceID()
	wid2 := accountsid.NewWorkspaceID()
	did := id.NewDeploymentID()
	tid1 := id.NewTriggerID()
	tid2 := id.NewTriggerID()
	require.NoError(t, base.Save(ctx, newTrig(wid1, did, tid1)))
	require.NoError(t, base.Save(ctx, newTrig(wid2, did, tid2)))

	r := base.Filtered(repo.WorkspaceFilter{
		Readable: accountsid.WorkspaceIDList{wid1},
		Writable: accountsid.WorkspaceIDList{wid1},
	})

	// removing a writable-workspace trigger works
	require.NoError(t, r.Remove(ctx, tid1))
	// removing a non-writable trigger is a no-op (row remains)
	require.NoError(t, r.Remove(ctx, tid2))
	got, err := base.FindByID(ctx, tid2)
	require.NoError(t, err)
	assert.NotNil(t, got)
}
```

Add the `repo` import to the test file's import block:
```go
"github.com/reearth/reearth-flow/api/internal/usecase/repo"
```

- [ ] **Step 2: Run to verify it fails**

Run: `go test ./internal/infrastructure/postgres/ -run "TestTrigger_Remove" -v` (DB set)
Expected: FAIL — `r.Remove undefined`.

- [ ] **Step 3: Implement**

The `DeleteTrigger` sqlc query only matches by id, so the workspace write-filter needs a guarded delete. Replace the use of the generated delete with a direct, filtered statement. Add to `trigger.go`:
```go
func (r *Trigger) Remove(ctx context.Context, tid id.TriggerID) error {
	exec := pgxx.Executor(ctx, r.pool)
	if r.f.Writable == nil {
		if _, err := exec.Exec(ctx, `DELETE FROM triggers WHERE id = $1`, tid.String()); err != nil {
			return rerror.ErrInternalByWithContext(ctx, pgxx.WrapError(err))
		}
		return nil
	}
	if _, err := exec.Exec(ctx,
		`DELETE FROM triggers WHERE id = $1 AND workspace_id = ANY($2::text[])`,
		tid.String(), r.f.Writable.Strings(),
	); err != nil {
		return rerror.ErrInternalByWithContext(ctx, pgxx.WrapError(err))
	}
	return nil
}
```

> NOTE: the generated `DeleteTrigger` is intentionally left unused for now (kept for callers that have already enforced the filter). If lint flags it as unused, that's fine — it is a public method on `*gen.Queries`, not dead code.

- [ ] **Step 4: Run to verify it passes** — Expected: PASS (2 tests).

- [ ] **Step 5: Commit**

```bash
cd /Users/dexter/active/reearth-flow
git add server/api/internal/infrastructure/postgres/trigger.go \
        server/api/internal/infrastructure/postgres/trigger_test.go
git commit -m "feat(server/postgres): trigger Remove with workspace write filter"
```

### Task 1.10: FindByWorkspace (filter + keyword + pagination + ordering)

**Files:**
- Modify: `internal/infrastructure/postgres/trigger.go`
- Modify: `internal/infrastructure/postgres/trigger_test.go`

- [ ] **Step 1: Add the failing tests**

Append to `trigger_test.go`:
```go
func TestTrigger_FindByWorkspace_NoPagination(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	r := postgres.NewTrigger(pool)

	wid := accountsid.NewWorkspaceID()
	wid2 := accountsid.NewWorkspaceID()
	did := id.NewDeploymentID()
	require.NoError(t, r.Save(ctx, newTrig(wid, did, id.NewTriggerID())))
	require.NoError(t, r.Save(ctx, newTrig(wid, did, id.NewTriggerID())))
	require.NoError(t, r.Save(ctx, newTrig(wid2, did, id.NewTriggerID())))

	got, info, err := r.FindByWorkspace(ctx, wid, nil, nil)
	require.NoError(t, err)
	require.NotNil(t, info)
	assert.Len(t, got, 2)
}

func TestTrigger_FindByWorkspace_Paginated(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	r := postgres.NewTrigger(pool)

	wid := accountsid.NewWorkspaceID()
	did := id.NewDeploymentID()
	for i := 0; i < 5; i++ {
		require.NoError(t, r.Save(ctx, newTrig(wid, did, id.NewTriggerID())))
	}

	page := &interfaces.PaginationParam{Page: &interfaces.PageBasedPaginationParam{Page: 1, PageSize: 2}}
	got, info, err := r.FindByWorkspace(ctx, wid, page, nil)
	require.NoError(t, err)
	assert.Len(t, got, 2)
	assert.Equal(t, int64(5), info.TotalCount)
	assert.Equal(t, 3, info.TotalPages)
	assert.Equal(t, 1, info.CurrentPage)
}

func TestTrigger_FindByWorkspace_Keyword(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	r := postgres.NewTrigger(pool)

	wid := accountsid.NewWorkspaceID()
	did := id.NewDeploymentID()
	hay := trigger.New().ID(id.NewTriggerID()).Workspace(wid).Deployment(did).
		Description("findme please").EventSource(trigger.EventSourceTypeTimeDriven).
		CreatedAt(time.Now()).UpdatedAt(time.Now()).MustBuild()
	require.NoError(t, r.Save(ctx, hay))
	require.NoError(t, r.Save(ctx, newTrig(wid, did, id.NewTriggerID())))

	kw := "findme"
	got, _, err := r.FindByWorkspace(ctx, wid, nil, &kw)
	require.NoError(t, err)
	assert.Len(t, got, 1)
	assert.Equal(t, "findme please", got[0].Description())
}

func TestTrigger_FindByWorkspace_NotReadable(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	wid := accountsid.NewWorkspaceID()

	r := postgres.NewTrigger(pool).Filtered(repo.WorkspaceFilter{
		Readable: accountsid.WorkspaceIDList{accountsid.NewWorkspaceID()}, // not wid
	})
	got, info, err := r.FindByWorkspace(ctx, wid, nil, nil)
	require.NoError(t, err)
	assert.Empty(t, got)
	assert.NotNil(t, info)
}
```

Add to the test imports:
```go
"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
```

- [ ] **Step 2: Run to verify it fails**

Run: `go test ./internal/infrastructure/postgres/ -run TestTrigger_FindByWorkspace -v` (DB set)
Expected: FAIL — `r.FindByWorkspace undefined`.

- [ ] **Step 3: Implement (dynamic query, hand-written, whitelisted ORDER BY)**

Add to `trigger.go` (and add `"fmt"`, `"strings"` to imports):
```go
// orderByColumns maps interface order keys to safe SQL columns. Keys absent here
// (e.g. the legacy Mongo "status") are ignored, matching Mongo's no-op behavior.
var triggerOrderByColumns = map[string]string{
	"description": "description",
	"createdAt":   "created_at",
	"updatedAt":   "updated_at",
	"id":          "id",
}

func (r *Trigger) FindByWorkspace(
	ctx context.Context,
	wid accountsid.WorkspaceID,
	pagination *interfaces.PaginationParam,
	keyword *string,
) ([]*trigger.Trigger, *interfaces.PageBasedInfo, error) {
	if !r.f.CanRead(wid) {
		return nil, interfaces.NewPageBasedInfo(0, 1, 1), nil
	}

	where := []string{"workspace_id = $1"}
	args := []any{wid.String()}
	if keyword != nil && *keyword != "" {
		args = append(args, "%"+*keyword+"%")
		where = append(where, fmt.Sprintf("(description ILIKE $%d OR id ILIKE $%d)", len(args), len(args)))
	}
	whereSQL := "WHERE " + strings.Join(where, " AND ")
	exec := pgxx.Executor(ctx, r.pool)

	if pagination != nil && pagination.Page != nil {
		p := pagination.Page

		var total int64
		if err := exec.QueryRow(ctx,
			"SELECT count(*) FROM triggers "+whereSQL, args...,
		).Scan(&total); err != nil {
			return nil, nil, rerror.ErrInternalByWithContext(ctx, pgxx.WrapError(err))
		}

		orderCol := "updated_at"
		if p.OrderBy != nil {
			if c, ok := triggerOrderByColumns[*p.OrderBy]; ok {
				orderCol = c
			}
		}
		dir := "ASC"
		if p.OrderDir != nil && strings.EqualFold(*p.OrderDir, "DESC") {
			dir = "DESC"
		} else if p.OrderBy == nil {
			dir = "DESC" // default updated_at DESC
		}

		limit := p.PageSize
		offset := (p.Page - 1) * p.PageSize
		query := fmt.Sprintf(
			"SELECT * FROM triggers %s ORDER BY %s %s LIMIT $%d OFFSET $%d",
			whereSQL, orderCol, dir, len(args)+1, len(args)+2,
		)
		args = append(args, limit, offset)

		ts, err := r.queryTriggers(ctx, exec, query, args)
		if err != nil {
			return nil, nil, err
		}
		return ts, interfaces.NewPageBasedInfo(total, p.Page, p.PageSize), nil
	}

	ts, err := r.queryTriggers(ctx, exec, "SELECT * FROM triggers "+whereSQL+" ORDER BY updated_at DESC", args)
	if err != nil {
		return nil, nil, err
	}
	return ts, interfaces.NewPageBasedInfo(int64(len(ts)), 1, len(ts)), nil
}

func (r *Trigger) queryTriggers(ctx context.Context, exec pgxx.DBTX, query string, args []any) ([]*trigger.Trigger, error) {
	rows, err := exec.Query(ctx, query, args...)
	if err != nil {
		return nil, rerror.ErrInternalByWithContext(ctx, pgxx.WrapError(err))
	}
	defer rows.Close()

	genRows, err := pgx.CollectRows(rows, pgx.RowToStructByName[gen.Trigger])
	if err != nil {
		return nil, rerror.ErrInternalByWithContext(ctx, pgxx.WrapError(err))
	}
	res := make([]*trigger.Trigger, 0, len(genRows))
	for _, row := range genRows {
		t, err := triggerFromRow(row)
		if err != nil {
			return nil, err
		}
		res = append(res, t)
	}
	return res, nil
}
```

> NOTE on `RowToStructByName`: it maps columns to `gen.Trigger` fields by name (snake_case column -> Go field via sqlc's `db` tags). Confirm `gen/models.go` `Trigger` fields carry `json`/`db` mapping that pgx recognizes — sqlc pgx/v5 emits `db` struct tags by default. If `RowToStructByName` errors on a name mismatch, fall back to `pgx.RowToStructByPos` (the `SELECT *` column order matches the migration's column order) — both are valid; pick whichever the generated struct supports.

- [ ] **Step 4: Run to verify it passes** — Expected: PASS (4 tests).

- [ ] **Step 5: Commit**

```bash
cd /Users/dexter/active/reearth-flow
git add server/api/internal/infrastructure/postgres/trigger.go \
        server/api/internal/infrastructure/postgres/trigger_test.go
git commit -m "feat(server/postgres): trigger FindByWorkspace with filter, keyword, pagination"
```

### Task 1.11: Transaction parity test

**Files:**
- Modify: `internal/infrastructure/postgres/trigger_test.go`

- [ ] **Step 1: Add the test**

Append to `trigger_test.go`:
```go
func TestTrigger_Save_RollsBackInTransaction(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	r := postgres.NewTrigger(pool)
	tr := pgxx.NewTransaction(pool)

	wid := accountsid.NewWorkspaceID()
	did := id.NewDeploymentID()
	tid := id.NewTriggerID()

	tx, err := tr.Begin(ctx)
	require.NoError(t, err)
	require.NoError(t, r.Save(tx.Context(), newTrig(wid, did, tid)))
	// do NOT Commit -> rollback on End
	require.NoError(t, tx.End(tx.Context()))

	got, err := r.FindByID(ctx, tid)
	assert.Error(t, err)
	assert.Nil(t, got)
}

func TestTrigger_Save_CommitsInTransaction(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	r := postgres.NewTrigger(pool)
	tr := pgxx.NewTransaction(pool)

	wid := accountsid.NewWorkspaceID()
	did := id.NewDeploymentID()
	tid := id.NewTriggerID()

	tx, err := tr.Begin(ctx)
	require.NoError(t, err)
	require.NoError(t, r.Save(tx.Context(), newTrig(wid, did, tid)))
	tx.Commit()
	require.NoError(t, tx.End(tx.Context()))

	got, err := r.FindByID(ctx, tid)
	require.NoError(t, err)
	assert.Equal(t, tid, got.ID())
}
```

Add to test imports: `"github.com/reearth/reearthx/pgxx"`.

- [ ] **Step 2: Run to verify (with DB)**

Run: `go test ./internal/infrastructure/postgres/ -run "TestTrigger_Save_(RollsBack|Commits)InTransaction" -v` (DB set)
Expected: PASS — proves the adapter honors `usecasex.Transaction` via executor-from-context.

- [ ] **Step 3: Commit**

```bash
cd /Users/dexter/active/reearth-flow
git add server/api/internal/infrastructure/postgres/trigger_test.go
git commit -m "test(server/postgres): trigger transaction commit/rollback parity"
```

### Task 1.12: Container + boot wiring (DB_DRIVER)

**Files:**
- Create: `internal/infrastructure/postgres/container.go`
- Modify: `internal/app/config/config.go`
- Modify: `internal/app/repo.go`

- [ ] **Step 1: Inspect the current config struct**

Run: `grep -n "DB\b\|DB_Account\|DB_Users\|type Config struct" /Users/dexter/active/reearth-flow/server/api/internal/app/config/config.go`
Expected: shows `DB`, `DB_Account`, `DB_Users` fields and the `Config` struct location.

- [ ] **Step 2: Add config fields**

In `internal/app/config/config.go`, add to the `Config` struct (next to `DB`):
```go
	DB_Driver string `default:"mongo"` // "mongo" | "postgres"
	DB_PG     string // Postgres connection URI (used when DB_Driver == "postgres")
```
(Match the existing struct-tag style in that file; if it uses `envconfig`/`pp` tags, mirror them for these two fields.)

- [ ] **Step 3: Write the container constructor**

`internal/infrastructure/postgres/container.go`:
```go
package postgres

import (
	"context"
	"fmt"

	"github.com/jackc/pgx/v5/pgxpool"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearthx/account/accountusecase/accountrepo"
	"github.com/reearth/reearthx/pgxx"
)

// New builds a repo.Container backed by Postgres. During the incremental
// migration (design A1), only ported entities are implemented here; unported
// repos are left nil and guarded by mustComplete so that "postgres" mode cannot
// be booted into production until every entity is migrated.
func New(ctx context.Context, pool *pgxpool.Pool, account *accountrepo.Container) (*repo.Container, error) {
	c := &repo.Container{
		Trigger:     NewTrigger(pool),
		Transaction: pgxx.NewTransaction(pool),
		// account-owned repos still come from the account container:
		Workspace: account.Workspace,
		User:      account.User,
		Role:      account.Role,
	}
	if err := mustComplete(c); err != nil {
		return nil, err
	}
	return c, nil
}

// mustComplete fails fast if a not-yet-ported repo would be used. Extend the
// checklist as entities are migrated; remove it entirely at final cutover.
func mustComplete(c *repo.Container) error {
	missing := []string{}
	if c.Project == nil {
		missing = append(missing, "Project")
	}
	if c.Job == nil {
		missing = append(missing, "Job")
	}
	if c.Deployment == nil {
		missing = append(missing, "Deployment")
	}
	// ... (remaining unported repos)
	if len(missing) > 0 {
		return fmt.Errorf("postgres backend not yet implemented for: %v (set DB_DRIVER=mongo)", missing)
	}
	return nil
}
```

- [ ] **Step 4: Wire driver selection at boot**

In `internal/app/repo.go`, wrap the Mongo construction so Postgres is selectable. After the existing `repos, err := mongorepo.New(...)` block, add a branch. Replace lines 36–77 region's tail so it reads:
```go
	var repos *repo.Container
	switch conf.DB_Driver {
	case "postgres":
		pool, perr := pgxpool.New(ctx, conf.DB_PG)
		if perr != nil {
			log.Fatalf("postgres error: %+v\n", perr)
		}
		repos, err = postgresrepo.New(ctx, pool, accountRepos)
		if err != nil {
			log.Fatalf("Failed to init postgres: %+v\n", err)
		}
	default: // "mongo"
		repos, err = mongorepo.New(ctx, client.Database(databaseName), accountRepos, txAvailable)
		if err != nil {
			log.Fatalf("Failed to init mongo: %+v\n", err)
		}
	}
```
Add imports to `internal/app/repo.go`:
```go
	"github.com/jackc/pgx/v5/pgxpool"
	postgresrepo "github.com/reearth/reearth-flow/api/internal/infrastructure/postgres"
```

- [ ] **Step 5: Build**

Run: `cd /Users/dexter/active/reearth-flow/server/api && go build ./...`
Expected: no errors. (Booting with `DB_DRIVER=postgres` will fail fast via `mustComplete` until all repos are ported — that is intended.)

- [ ] **Step 6: Commit**

```bash
cd /Users/dexter/active/reearth-flow
git add server/api/internal/app/config/config.go \
        server/api/internal/app/repo.go \
        server/api/internal/infrastructure/postgres/container.go
git commit -m "feat(server): select DB backend via DB_DRIVER; postgres container scaffold"
```

### Task 1.13: Local dev — docker-compose + Makefile

**Files:**
- Modify: `docker-compose.yml`
- Modify: `Makefile`

- [ ] **Step 1: Add a Postgres service**

Append to `server/api/docker-compose.yml` under `services:`:
```yaml
  reearth-flow-postgres:
    image: postgres:16-alpine
    environment:
      POSTGRES_USER: reearth
      POSTGRES_PASSWORD: reearth
      POSTGRES_DB: reearth-flow
    ports:
      - 5432:5432
    volumes:
      - ./postgres:/var/lib/postgresql/data
```

- [ ] **Step 2: Add Makefile targets**

Append to `server/api/Makefile` (and add the new names to the `.PHONY` line):
```makefile
run-db-pg:
	docker compose -f ./docker-compose.yml up -d reearth-flow-postgres

PG_DIR=internal/infrastructure/postgres

sqlc:
	cd $(PG_DIR) && go tool sqlc generate

atlas-diff:
	cd $(PG_DIR)/db && atlas migrate diff $(name) --env local

atlas-lint:
	cd $(PG_DIR)/db && atlas migrate lint --env local --latest 1
```
Update the `.PHONY` line to include: `run-db-pg sqlc atlas-diff atlas-lint`.

- [ ] **Step 3: Verify compose config parses**

Run: `cd /Users/dexter/active/reearth-flow/server/api && docker compose -f ./docker-compose.yml config >/dev/null && echo OK`
Expected: `OK`.

- [ ] **Step 4: Commit**

```bash
cd /Users/dexter/active/reearth-flow
git add server/api/docker-compose.yml server/api/Makefile
git commit -m "chore(server): add postgres dev service and sqlc/atlas make targets"
```

### Task 1.14: CI — Postgres service + drift check

**Files:**
- Modify: `.github/workflows/ci_api.yml`

- [ ] **Step 1: Add a Postgres service + env var to the test job**

In `.github/workflows/ci_api.yml`, under `ci-api-test.services`, add alongside `mongo`:
```yaml
      postgres:
        image: postgres:16-alpine
        env:
          POSTGRES_USER: reearth
          POSTGRES_PASSWORD: reearth
        ports:
          - 5432:5432
        options: >-
          --health-cmd "pg_isready -U reearth"
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
```
And add to the `test` step's `env:` block:
```yaml
          REEARTH_FLOW_DB_PG: postgres://reearth:reearth@localhost:5432/postgres?sslmode=disable
```

- [ ] **Step 2: Add a generation-drift check job**

Append a new job to `ci_api.yml`:
```yaml
  ci-api-codegen-drift:
    runs-on: ubuntu-latest
    if: github.event_name != 'push' || !startsWith(github.event.head_commit.message, 'v')
    steps:
      - uses: actions/checkout@34e114876b0b11c390a56381ad16ebd13914f8d5 # v4.3.1
      - uses: actions/setup-go@40f1582b2485089dde7abd97c1529aa768e1baff # v5.6.0
        with:
          go-version-file: "server/api/go.mod"
      - name: Install Atlas
        run: curl -sSf https://atlasgo.sh | sh
      - name: sqlc generate (no drift)
        working-directory: server/api/internal/infrastructure/postgres
        run: |
          go tool sqlc generate
          git diff --exit-code gen/ || (echo "sqlc drift: run 'make sqlc' and commit"; exit 1)
      - name: atlas migrate lint
        working-directory: server/api/internal/infrastructure/postgres/db
        run: atlas migrate lint --env local --latest 1
```

- [ ] **Step 3: Validate the workflow YAML**

Run: `cd /Users/dexter/active/reearth-flow && python3 -c "import yaml,sys; yaml.safe_load(open('.github/workflows/ci_api.yml')); print('OK')"`
Expected: `OK`.

- [ ] **Step 4: Commit**

```bash
cd /Users/dexter/active/reearth-flow
git add .github/workflows/ci_api.yml
git commit -m "ci(api): add postgres service and sqlc/atlas drift checks"
```

### Task 1.15: Golden-path README

**Files:**
- Create: `internal/infrastructure/postgres/README.md`

- [ ] **Step 1: Write the recipe**

`internal/infrastructure/postgres/README.md`:
```markdown
# Postgres backend (golden path)

Adds Postgres support behind `repo.Container`. Generic SQL/transaction code lives
in `github.com/reearth/reearthx/pgxx`.

## Per-entity recipe

1. Add the table to `db/schema.hcl`.
2. Generate a migration: `make atlas-diff name=<change>` (from `server/api`).
3. Add queries to `query/<entity>.sql` with sqlc annotations.
4. Regenerate: `make sqlc`.
5. Implement the repo adapter in `<entity>.go`, obtaining the executor via
   `pgxx.Executor(ctx, pool)` so writes join any active `usecasex.Transaction`.
6. Add parity integration tests in `<entity>_test.go` using `pgtest.Connect`.
   Run locally: `make run-db-pg` then
   `REEARTH_FLOW_DB_PG=postgres://reearth:reearth@localhost:5432/postgres?sslmode=disable make test`.
7. CI runs the tests against a Postgres service and checks codegen drift.

## Transactions

`pgxx.NewTransaction(pool)` implements `usecasex.Transaction`. `repo.Container.Transaction`
is set to it in `container.go`. No use-case changes are required.

## Status

Ported: Trigger. Unported repos are guarded by `mustComplete` in `container.go`;
`DB_DRIVER=postgres` cannot boot until every entity is ported (design A1).
```

- [ ] **Step 2: Commit**

```bash
cd /Users/dexter/active/reearth-flow
git add server/api/internal/infrastructure/postgres/README.md
git commit -m "docs(server/postgres): document the per-entity golden-path recipe"
```

### Task 1.16: Full verification

- [ ] **Step 1: Lint**

Run: `cd /Users/dexter/active/reearth-flow/server/api && make lint`
Expected: passes (or only pre-existing warnings unrelated to new files).

- [ ] **Step 2: Full test run with Postgres + Mongo**

Run:
```bash
cd /Users/dexter/active/reearth-flow/server/api
docker compose -f ./docker-compose.yml up -d reearth-flow-postgres reearth-flow-mongo
sleep 4
REEARTH_FLOW_DB=mongodb://localhost \
REEARTH_FLOW_DB_PG=postgres://reearth:reearth@localhost:5432/postgres?sslmode=disable \
  go test ./internal/infrastructure/postgres/... -race -v
```
Expected: all Postgres trigger tests PASS (none skipped).

- [ ] **Step 3: Confirm Mongo path is unaffected**

Run: `cd /Users/dexter/active/reearth-flow/server/api && REEARTH_FLOW_DB=mongodb://localhost go test ./internal/infrastructure/mongo/ -run TestTrigger -v`
Expected: existing Mongo trigger tests still PASS.

### Task 1.17: Landing prep (remove replace, bump reearthx)

**Files:**
- Modify: `server/api/go.mod`

- [ ] **Step 1: Merge reearthx `feat/pgxx` and get a pseudo-version**

After the reearthx PR is merged to `main`, get the version:
```bash
cd /Users/dexter/active/reearth-flow/server/api
go mod edit -dropreplace github.com/reearth/reearthx
go get github.com/reearth/reearthx@main
go mod tidy
```
Expected: `replace` removed; reearthx pinned to the merged commit's pseudo-version.

- [ ] **Step 2: Re-run full verification (Task 1.16) against the published reearthx**

Expected: all tests PASS without the local replace.

- [ ] **Step 3: Commit**

```bash
cd /Users/dexter/active/reearth-flow
git add server/api/go.mod server/api/go.sum
git commit -m "chore(server): consume published reearthx pgxx; drop local replace"
```

---

## Self-Review (completed by plan author)

- **Spec coverage:** §3 reearthx pgxx → Tasks 0.1–0.7; §4 Trigger pilot (schema/sqlc/adapter) → Tasks 1.2–1.11; §5 golden-path recipe → Task 1.15; §6 transaction model → Tasks 0.4/1.11; §7 testing → Tasks 0.6/1.6–1.11/1.16; §9 local dev → Task 1.13; §10 CI → Task 1.14; §11 phasing → Task ordering; §12 open items: Go skew → Task 0.0, replace/bump → Tasks 1.1/1.17, account repos out of scope (only Workspace/User/Role wired from account container, not reimplemented).
- **Placeholders:** the `mustComplete` checklist intentionally lists representative unported repos with a `// ...` marker — this is a guard that grows per entity, not a code placeholder; every other step contains complete code/commands.
- **Type consistency:** `pgxx.DBTX`/`Executor`/`ContextWithTx`/`NewTransaction`/`WrapError`/`IsSerializationError` used consistently across Phase 0 and Phase 1; `triggerFromRow`/`triggerToParams`/`variablesToJSON`/`variablesFromJSON`/`queryTriggers`/`triggerOrderByColumns` names are consistent across tasks; `gen.Trigger`/`gen.UpsertTriggerParams`/`gen.New` match the sqlc config.
- **Known risk to validate during execution:** sqlc's exact Go types for nullable columns (`pgtype.Text`/`pgtype.Timestamptz`) and whether `pgx.RowToStructByName` or `...ByPos` matches the generated struct tags — verified at Task 1.3 (inspect `gen/models.go`) and Task 1.10 (note).
```
