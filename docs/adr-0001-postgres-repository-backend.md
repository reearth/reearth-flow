# ADR-0001: Postgres Repository Backend and Shared Transaction Model

- Status: Proposed
- Date: 2026-07-01

## Context

Re:Earth Flow's server API has historically used MongoDB as its primary
repository backend. That shaped both persistence code and transaction handling:
repository implementations were Mongo-specific, and use-case code relied on a
manual transaction lifecycle based on `Begin`, `Commit`, and `End`.

This branch introduces a second repository backend implemented on PostgreSQL.
The goal is not only to add SQL persistence, but to do so without splitting the
use-case layer into Mongo-specific and Postgres-specific execution paths.

That creates two architectural requirements:

1. The repository container must be able to select Mongo or Postgres at runtime
   without changing use-case behavior.
2. Transaction orchestration must use one shared abstraction across both
   backends, so use cases stay backend-agnostic.

The Postgres work also introduces schema and query tooling that does not exist
in the Mongo path today:

- Atlas for schema definition and migration validation
- sqlc for generated query bindings
- pgx/pgxx for pooled connections and transaction-aware execution

Because this is an infrastructure-level change, the decision needs durable
documentation beyond PR discussion and code comments.

## Decision

We will support a dormant-by-default PostgreSQL repository backend in the
server API and standardize transactional use-case orchestration on
`usecasex.Transactor.WithinTransaction(ctx, fn)`.

More specifically:

1. Repository backend selection happens at application wiring time through the
   server's database driver configuration.
2. Mongo remains the default backend and existing Mongo behavior is preserved.
3. Postgres implementations live alongside Mongo implementations behind the same
   repository interfaces and `repo.Container`.
4. Use cases no longer manage transaction lifecycles with explicit
   `Begin`/`Commit`/`End` calls. Instead, they execute transactional work inside
   `WithinTransaction(ctx, fn)`.
5. Repository implementations resolve the active executor from context so the
   same repository call transparently uses either the ambient transaction or the
   base database handle.
6. Postgres schema changes are managed through Atlas migrations and SQL access
   is generated through sqlc rather than handwritten ad hoc query execution.

## Consequences

### Positive

- The use-case layer keeps a single transaction shape across Mongo and
  Postgres, which reduces backend-specific branching.
- Postgres can be introduced incrementally without changing domain and use-case
  contracts.
- The callback-style transactor makes commit and rollback behavior more obvious:
  returning `nil` commits, returning an error rolls back.
- Atlas and sqlc make schema evolution and query drift more explicit and easier
  to validate in CI.
- Integration tests can assert backend parity at the repository boundary rather
  than duplicating higher-level business logic.

### Negative

- Repository implementation complexity increases because the server now carries
  two persistence backends.
- Postgres introduces new operational tooling and migration discipline that the
  Mongo-only path did not require.
- Transaction debugging shifts from explicit transaction objects in use-case
  code to context-carried execution, which requires stricter repository
  conventions.

### Neutral / Follow-on

- Data migration from Mongo to Postgres is a rollout concern, not a prerequisite
  for compiling or booting the server.
- Enabling Postgres in any given environment remains an application deployment
  decision.
- This ADR documents the backend and transaction architecture; it does not make
  a broader claim that Mongo will be removed.

## Alternatives Considered

### 1. Keep Mongo as the only backend

Rejected because it would block SQL-backed deployments and prevent the team from
using relational schema management and query tooling where they are beneficial.

### 2. Add Postgres, but keep backend-specific transaction APIs

Rejected because it would leak infrastructure differences into the use-case
layer and make parity harder to preserve. The branch already showed that a
shared transactor abstraction is the cleaner seam.

### 3. Add Postgres behind shared repositories, but keep manual
`Begin`/`Commit`/`End` transaction control

Rejected because explicit lifecycle handling duplicates boilerplate across
interactors and is easier to get wrong. The callback form centralizes commit,
rollback, and retry behavior.

## Rollout Notes

- Mongo remains the default backend after this change.
- Postgres is intended to be enabled per environment through configuration.
- Postgres-backed repositories should continue to be validated with parity-style
  integration tests against the existing repository contracts.
- Future ADRs should extend this sequence if the project later decides to make
  Postgres the default backend or retire Mongo-backed implementations.
