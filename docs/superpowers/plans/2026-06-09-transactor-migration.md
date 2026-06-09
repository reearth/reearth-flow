# Transactor Migration Implementation Plan

> Supersedes the `usecasex.Transaction` approach in the Postgres pilot (spec Decision F).
> REQUIRED SUB-SKILL: superpowers:subagent-driven-development.

**Goal:** Replace the `usecasex.Transaction` (Begin/Commit/End) transaction style with the Thibaut Rousseau **`Transactor.WithinTransaction(ctx, fn)`** callback across reearthx (`pgxx` + a new `usecasex.Transactor`) and ALL of reearth-flow's interactors, preserving Mongo behavior and the Postgres pilot's parity tests.

**Architecture:** A new `usecasex.Transactor` interface is the canonical abstraction. Mongo reuses its existing `usecasex.Transaction` via a `NewTransactor` bridge (delegates to `DoTransaction`, preserving retry). `pgxx` implements `Transactor` natively (begins a `pgx.Tx`, stores it in context for `Executor`/DBGetter, commits/rolls back, retries on serialization failure). Flow's `repo.Container.Transaction` becomes `usecasex.Transactor`; `Run3` and all 26 manual `Begin/Commit/End` call-sites become `WithinTransaction` closures.

**Tech stack:** Go, `jackc/pgx/v5`, reearthx `usecasex`, existing `mongox`.

---

## Phase A — reearthx (`/Users/dexter/active/reearthx`, branch `feat/pgxx`; updates PR #148)

### A1. Add `usecasex.Transactor` + bridge
Create `usecasex/transactor.go`:
```go
package usecasex

import "context"

// Transactor runs a function within a database transaction, committing on a nil
// return and rolling back on error. The fn receives a context carrying the
// transaction; repositories resolve it (e.g. via pgxx.Executor) transparently.
type Transactor interface {
	WithinTransaction(ctx context.Context, fn func(ctx context.Context) error) error
}

// NewTransactor adapts an existing Transaction (e.g. the Mongo implementation)
// into a Transactor, delegating to DoTransaction so retry-on-ErrTransaction
// behavior is preserved. retry <= 0 means no retry.
func NewTransactor(t Transaction, retry int) Transactor {
	return &transactorAdapter{t: t, retry: retry}
}

type transactorAdapter struct {
	t     Transaction
	retry int
}

func (a *transactorAdapter) WithinTransaction(ctx context.Context, fn func(ctx context.Context) error) error {
	return DoTransaction(ctx, a.t, a.retry, fn)
}
```
- [ ] Create the file. `go build ./usecasex/ && go vet ./usecasex/`.
- [ ] Unit test `usecasex/transactor_test.go`: a `NewTransactor(&NopTransaction{}, 2)` runs fn and returns its error; on a fn returning an `ErrTransaction`-wrapped error once then nil, fn runs twice (retry preserved). Run `go test ./usecasex/ -run Transactor`.
- [ ] Commit: `feat(usecasex): add Transactor (WithinTransaction) interface and bridge`.

### A2. Rework `pgxx` to a native `Transactor` (remove the `usecasex.Transaction` impl)
Replace `pgxx/transaction.go` with `pgxx/transactor.go`:
```go
package pgxx

import (
	"context"
	"errors"

	"github.com/jackc/pgx/v5/pgxpool"
	"github.com/reearth/reearthx/usecasex"
)

// Transactor is a pgx-backed usecasex.Transactor. WithinTransaction begins a
// transaction, stores it in the context (see Executor), runs fn, and commits on
// success or rolls back on error. Serialization failures (see WrapError) are
// retried up to retries times.
type Transactor struct {
	pool    *pgxpool.Pool
	retries int
}

var _ usecasex.Transactor = (*Transactor)(nil)

// NewTransactor returns a pgx Transactor. retries is the number of extra
// attempts on serialization failure (0 = single attempt).
func NewTransactor(pool *pgxpool.Pool, retries int) *Transactor {
	return &Transactor{pool: pool, retries: retries}
}

func (t *Transactor) WithinTransaction(ctx context.Context, fn func(ctx context.Context) error) error {
	var err error
	for attempt := 0; ; attempt++ {
		err = t.runOnce(ctx, fn)
		if err == nil || !errors.Is(err, usecasex.ErrTransaction) || attempt >= t.retries {
			return err
		}
	}
}

func (t *Transactor) runOnce(ctx context.Context, fn func(ctx context.Context) error) error {
	tx, err := t.pool.Begin(ctx)
	if err != nil {
		return err
	}
	txCtx := ContextWithTx(ctx, tx)
	if err := fn(txCtx); err != nil {
		_ = tx.Rollback(context.Background())
		return err
	}
	if err := tx.Commit(context.Background()); err != nil {
		return WrapError(err)
	}
	return nil
}
```
- [ ] Delete `pgxx/transaction.go` and `pgxx/transaction_test.go`; create `pgxx/transactor.go`.
- [ ] Keep `pgxx/pgxx.go` (DBTX/Executor/ContextWithTx) and `pgxx/errors.go` unchanged.
- [ ] Replace the integration test with `pgxx/transactor_test.go` (env-gated, uses `pgxtest`): a scratch `items` table; `WithinTransaction` returning nil commits the insert; returning an error rolls it back; assert row counts. (Mirror the prior commit/rollback tests but via `WithinTransaction`.)
- [ ] `REEARTH_DB_PG=... go test ./pgxx/...` all pass; without it, integration tests skip.
- [ ] Commit: `feat(pgxx)!: implement usecasex.Transactor (WithinTransaction); drop usecasex.Transaction impl`.

### A3. Update PR #148
- [ ] `git push origin feat/pgxx`. Update the PR description to describe `Transactor`/`WithinTransaction` (replace the `usecasex.Transaction` wording). Do not merge yet (flow depends on it).

---

## Phase B — flow (`/Users/dexter/active/reearth-flow/server/api`, branch `feat/postgres-support`)

> Local `replace` still points at the reworked local reearthx, so these build immediately.

### B1. `repo.Container.Transaction` → `usecasex.Transactor`
In `internal/usecase/repo/container.go`:
- [ ] Change field: `Transaction usecasex.Transactor` (was `usecasex.Transaction`).
- [ ] `Filtered` keeps copying `Transaction: c.Transaction` (now Transactor) — unchanged.
- [ ] `AccountRepos()` currently sets `Transaction: c.Transaction` but `accountrepo.Container.Transaction` is `usecasex.Transaction`. `AccountRepos()` has NO callers (dead compat shim). Make it compile: drop the `Transaction:` line (leave nil) and add a comment: `// Transaction omitted: account container is built directly in app/repo.go; this shim is unused.`
- [ ] `go build ./internal/usecase/repo/`.

### B2. Container wiring
- Mongo (`internal/infrastructure/mongo/container.go:46`): `Transaction: usecasex.NewTransactor(client.Transaction(), 2)`. Add `usecasex` import.
- Postgres (`internal/infrastructure/postgres/container.go`): `Transaction: pgxx.NewTransactor(pool, 2)` (was `pgxx.NewTransaction(pool)`).
- [ ] `go build ./...`.

### B3. `usecase.go` Run3
In `internal/usecase/interactor/usecase.go` (`Run3`, ~line 68):
```go
	var t usecasex.Transactor
	if e.tx && r.Transaction != nil {
		t = r.Transaction
	}
	if t == nil {
		err = f(ctx)
		return
	}
	err = t.WithinTransaction(ctx, func(ctx context.Context) error {
		a, b, c, err := f(ctx)
		_ = a; _ = b; _ = c // assigned to outer vars below
		return err
	})
```
NOTE: preserve the existing closure-capture of `a,b,c`. Concretely:
```go
	var t usecasex.Transactor
	if e.tx && r.Transaction != nil {
		t = r.Transaction
	}
	run := func(ctx context.Context) error {
		a, b, c, err = f(ctx)
		return err
	}
	if t == nil {
		err = run(ctx)
		return
	}
	err = t.WithinTransaction(ctx, run)
	return
```
- [ ] Remove the now-unused `retry` const if nothing else uses it (check first; `grep retry`).
- [ ] `go build ./internal/usecase/interactor/`.

### B4. Interactor field type (all 8)
In each of `deployment.go, edge.go, job.go, parameter.go, project.go, projectAccess.go, trigger.go, worker_config.go`:
- [ ] Change struct field `transaction usecasex.Transaction` → `transaction usecasex.Transactor`. Constructor wiring `transaction: r.Transaction` is unchanged (now Transactor).

### B5. Canonical call-site transform (apply to all 26)
**Before:**
```go
tx, err := i.transaction.Begin(ctx)
if err != nil {
	return /* zero values, */ err
}
defer func() {
	if err := tx.End(ctx); err != nil {
		log.Errorfc(ctx, "transaction end failed: %v", err)
	}
}()

// ... body that may `return err` early ...
if err := i.someRepo.Save(ctx, x); err != nil {
	return err
}

tx.Commit()
// ... post-commit work (return value assembly, side effects) ...
return result, nil
```
**After:**
```go
var result *Thing // declare outputs the closure assigns
err := i.transaction.WithinTransaction(ctx, func(ctx context.Context) error {
	// ... same body; early `return err` rolls back; final `return nil` commits ...
	if err := i.someRepo.Save(ctx, x); err != nil {
		return err
	}
	result = x
	return nil
})
if err != nil {
	return /* zero, */ err
}
// ... post-commit work that previously followed tx.Commit() ...
return result, nil
```
Rules:
- The closure body is exactly what sat between `Begin` and `tx.Commit()`. Replace `return err` with `return err` (now rolls back). Replace `tx.Commit()` with `return nil` at the end of the transactional section.
- Anything AFTER `tx.Commit()` (Redis cleanup, building return values, logging) moves to AFTER `WithinTransaction` returns nil — it must NOT run if the tx failed, so guard on `err == nil` / return early on error.
- All repo calls inside the closure use the closure's `ctx` (which carries the tx). Use the closure parameter `ctx`, not an outer one.
- For methods that returned values produced inside the tx, declare them before the closure and assign inside.
- For `job.go`'s conditional `if statusChanged { tx := Begin... }` blocks, wrap only that block in `WithinTransaction`.

Apply per file, building + running that package's tests after each:
- [ ] edge.go (1) → `go build ./... && go test ./internal/usecase/interactor/ -run Edge` (or the package's existing tests).
- [ ] worker_config.go (2), projectAccess.go (2), job.go (3), deployment.go (4), project.go (4), parameter.go (5), trigger.go (5).
- [ ] After all: `go build ./...`, `go vet ./...`, `gofmt -l` clean.
- [ ] Commit incrementally per file or per logical group: `refactor(server): migrate <area> interactor to Transactor.WithinTransaction`.

### B6. Re-verify
- [ ] `go build ./...` clean.
- [ ] Postgres parity tests still pass: `REEARTH_FLOW_DB_PG=postgres://reearth:reearth@localhost:5434/postgres?sslmode=disable go test ./internal/infrastructure/postgres/...` (the Trigger tests now exercise `pgxx.NewTransactor`).
- [ ] Interactor unit tests pass: `go test ./internal/usecase/interactor/...` (these use `usecasex.NopTransaction` or mocks; if any wire `Transaction:` with a `usecasex.Transaction`, wrap via `usecasex.NewTransactor(&usecasex.NopTransaction{}, 0)` or adjust the test to the Transactor type).
- [ ] Update `internal/infrastructure/postgres/trigger_test.go` transaction tests (TestTrigger_Save_*InTransaction) to use `pgxx.NewTransactor(pool, 0).WithinTransaction(...)` instead of `Begin/Commit/End`.
- [ ] Update the postgres `README.md` transaction section to describe `Transactor.WithinTransaction`.

---

## Self-Review
- **Spec coverage:** Decision F + §7 → Phases A/B. Mongo preserved via bridge (A1) + B2. Pilot parity preserved (B6). Account compat (B1).
- **Risk:** the 26 call-site transform is the delicate part — semantics hinge on "post-commit work must not run on error" and "closure uses the tx-carrying ctx". B5 codifies both. Interactor tests are the safety net.
- **Type consistency:** `usecasex.Transactor`, `WithinTransaction`, `NewTransactor`, `pgxx.NewTransactor` used consistently A→B.
