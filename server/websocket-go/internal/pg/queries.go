package pg

import (
	"context"
	"time"

	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgconn"
)

// Querier is the subset of pgxpool.Pool this package uses. Narrowing it lets the
// tests pass a single connection, and is satisfied by *pgxpool.Pool, *pgx.Conn and
// pgx.Tx alike.
//
// SendBatch belongs here rather than being type-asserted at the call site: the
// writer needs it on every insert, so a handle without it must fail to compile
// instead of panicking on the first update.
type Querier interface {
	Exec(ctx context.Context, sql string, args ...any) (pgconn.CommandTag, error)
	Query(ctx context.Context, sql string, args ...any) (pgx.Rows, error)
	QueryRow(ctx context.Context, sql string, args ...any) pgx.Row
	SendBatch(ctx context.Context, b *pgx.Batch) pgx.BatchResults
	// Begin is needed by Migrate, which must hold a transaction-scoped advisory
	// lock across a multi-statement DDL. A batch cannot express that: pgx batches
	// use the extended protocol, which forbids multiple statements per query.
	Begin(ctx context.Context) (pgx.Tx, error)
}

// lockMigrationSQL serialises concurrent startups so two instances cannot race the
// same CREATE TABLE (which IF NOT EXISTS does not make safe — concurrent creates can
// still collide on pg_type's unique index).
//
// Xact-scoped, so it MUST share a transaction with the DDL it protects. Issued as a
// standalone Exec on a pool it is worthless: that Exec's own implicit transaction
// commits immediately, releasing the lock before the schema is sent.
const lockMigrationSQL = `SELECT pg_advisory_xact_lock(hashtext('ws_migrate'))`

// serializeInsertSQL is what makes the reader's `id > cursor` cursor correct, and
// must stay FIRST in every write batch.
//
// bigserial allocates ids at INSERT but rows appear at COMMIT, so concurrent inserts
// can commit out of id order: a reader already past 101 would never see a late 100 —
// a silently dropped update, and a permanently diverged document. Holding a
// per-document lock across the insert transaction makes allocation order and commit
// order the same, so no timing guard is needed on the read side.
//
// NOTIFY does not remove this hazard: it reports that something committed, not that
// no lower id is still in flight.
const serializeInsertSQL = `SELECT pg_advisory_xact_lock(hashtext('wsins:' || $1::text))`

// appendSQL writes one update; the writer sends many in one pgx.Batch.
//
// created_at defaults to now() (transaction START), used by the janitor sweep and
// the latency measurement. Cursor correctness comes from serializeInsertSQL, not
// from this column.
const appendSQL = `
INSERT INTO ws_stream (doc_id, kind, data, client_id)
VALUES ($1, $2, $3, $4)`

// notifyChannel carries a doc id to every listening instance. One channel for all
// documents: per-document channels would mean LISTEN/UNLISTEN on every activation,
// to save a wakeup that costs nothing to ignore.
const notifyChannel = "ws_stream_update"

// notifySQL is the last statement of every write batch. Postgres delivers at COMMIT
// and collapses duplicates per transaction, so one per batch suffices.
//
// Sent regardless of this instance's own fan-out: a silent poll-mode writer would
// leave notify-mode peers blind to its writes, making a mixed rollout lossy.
const notifySQL = `SELECT pg_notify($1, $2)`

// readSQL fetches the next batch for one document after a cursor.
//
// Two things here are deliberate and easy to "optimise" into bugs:
//
//  1. NO client_id filter. Self-originated rows must still advance the cursor, or
//     a burst of our own writes stalls it and we re-read them forever. The caller
//     skips them in Go after taking the id. This mirrors relay.go:297.
//
//  2. The created_at guard is defence-in-depth, not the primary mechanism — that is
//     serializeInsertSQL. It defaults to zero and exists only for rows that might
//     reach ws_stream without taking the insert lock (a manual backfill, say).
//
// The trailing age column is the latency measurement. Both timestamps come from the
// DATABASE clock, so it carries no cross-instance skew. It slightly overstates,
// since created_at is the transaction's start — the honest direction to err.
const readSQL = `
SELECT id, kind, data, client_id,
       EXTRACT(EPOCH FROM (clock_timestamp() - created_at))
  FROM ws_stream
 WHERE doc_id = $1
   AND id > $2
   AND created_at < now() - make_interval(secs => $3)
 ORDER BY id
 LIMIT $4`

// heartbeatSQL records this instance's liveness. Replaces HSET + EXPIRE 120; the
// expiry becomes the janitor's job because liveness is decided by seen_at, never by
// whether a row still exists.
const heartbeatSQL = `
INSERT INTO ws_instance (doc_id, client_id, seen_at)
VALUES ($1, $2, now())
ON CONFLICT (doc_id, client_id) DO UPDATE SET seen_at = now()`

// activeInstancesSQL counts live instances for a document. The interval mirrors
// Redis's activeTimeoutSecs (60s), which is deliberately shorter than the 120s key
// TTL it replaced.
const activeInstancesSQL = `
SELECT count(*)
  FROM ws_instance
 WHERE doc_id = $1
   AND seen_at > now() - make_interval(secs => $2)`

// removeHeartbeatSQL drops this instance's row. Unlike Redis's HDEL script it does
// not report whether the hash emptied — the caller re-counts, which is the same
// two-step the Redis path performs.
const removeHeartbeatSQL = `
DELETE FROM ws_instance WHERE doc_id = $1 AND client_id = $2`

// electionLockSQL is the first statement of the eviction batch. It must be its OWN
// statement, not a CTE alongside the DELETE: Postgres does not guarantee evaluation
// order between CTEs, so a lock acquired in a sibling CTE may not be held when the
// DELETE runs — which is the one thing it exists to guarantee. pgx executes batched
// statements in queue order inside one implicit transaction, so queuing this first
// is sufficient.
const electionLockSQL = `SELECT pg_advisory_xact_lock(hashtext('wsdoc:' || $1::text))`

// safeDeleteSQL is the last-instance election, replacing ~40 lines of Lua that
// existed only because Redis has no transactions. Both guards are evaluated in the
// same snapshot as the DELETE, and electionLockSQL (queued before it in the same
// batch) serialises concurrent electors.
//
// Returns rows deleted; 0 means the document was still live (or already empty) and
// the caller must leave it alone.
const safeDeleteSQL = `
WITH deleted AS (
  DELETE FROM ws_stream
   WHERE doc_id = $1
     AND NOT EXISTS (
       SELECT 1 FROM ws_lock
        WHERE key = $2 AND expires_at > now()
     )
     AND NOT EXISTS (
       SELECT 1 FROM ws_instance
        WHERE doc_id = $1
          AND seen_at > now() - make_interval(secs => $3)
     )
  RETURNING 1
)
SELECT count(*) FROM deleted`

// forceDeleteSQL is the rollback path: drop a document's rows with no flush and no
// election, so rolled-back state cannot be revived by a reconnect replaying it.
const forceDeleteSQL = `DELETE FROM ws_stream WHERE doc_id = $1`

// acquireLockSQL takes key when it is free or its holder has expired. Replaces
// SET NX PX. Returns a row only when this owner now holds it.
const acquireLockSQL = `
INSERT INTO ws_lock (key, owner, expires_at)
VALUES ($1, $2, now() + make_interval(secs => $3))
ON CONFLICT (key) DO UPDATE
   SET owner = excluded.owner, expires_at = excluded.expires_at
 WHERE ws_lock.expires_at <= now()
RETURNING 1`

// releaseLockSQL releases only our own lock. The owner predicate is what makes this
// safe: without it, a holder whose TTL lapsed mid-work would delete the lock its
// successor now legitimately holds.
const releaseLockSQL = `DELETE FROM ws_lock WHERE key = $1 AND owner = $2`

// reapStreamSQL and reapInstancesSQL are the janitor, replacing Redis key TTLs.
const (
	reapStreamSQL    = `DELETE FROM ws_stream WHERE created_at < now() - make_interval(secs => $1)`
	reapInstancesSQL = `DELETE FROM ws_instance WHERE seen_at < now() - make_interval(secs => $1)`
	reapLocksSQL     = `DELETE FROM ws_lock WHERE expires_at < now() - make_interval(secs => $1)`
)

// secs renders a Go duration for make_interval(secs => ...).
//
// Duration.String() is NOT usable as a Postgres interval literal: it emits Go
// syntax ("1ns", "6h0m0s"), and interval has no nanosecond unit while "m" is
// ambiguous between minutes and months. Passing a float of seconds and letting
// make_interval build the value sidesteps both. Sub-microsecond values round to
// zero, which is interval's resolution.
func secs(d time.Duration) float64 { return d.Seconds() }
