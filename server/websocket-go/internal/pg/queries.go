package pg

import (
	"context"
	"time"

	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgconn"
)

// Querier is the subset of pgxpool.Pool this package uses, so tests can pass a
// single connection. Satisfied by *pgxpool.Pool, *pgx.Conn and pgx.Tx.
type Querier interface {
	Exec(ctx context.Context, sql string, args ...any) (pgconn.CommandTag, error)
	Query(ctx context.Context, sql string, args ...any) (pgx.Rows, error)
	QueryRow(ctx context.Context, sql string, args ...any) pgx.Row
	SendBatch(ctx context.Context, b *pgx.Batch) pgx.BatchResults
	// Begin: Migrate holds an xact lock across a multi-statement DDL, which a batch
	// cannot express (extended protocol allows one statement per query).
	Begin(ctx context.Context) (pgx.Tx, error)
}

// lockMigrationSQL serialises concurrent startups; IF NOT EXISTS alone does not make
// concurrent CREATE TABLE safe. Xact-scoped, so it MUST share a transaction with the
// DDL — a standalone Exec commits and releases it before the schema is sent.
const lockMigrationSQL = `SELECT pg_advisory_xact_lock(hashtext('ws_migrate'))`

// serializeInsertSQL makes the reader's `id > cursor` correct and must stay FIRST in
// every write batch. bigserial allocates ids at INSERT but rows appear at COMMIT, so
// without it concurrent inserts commit out of id order and a reader past 101 never
// sees a late 100 — a silently dropped update. NOTIFY does not help: it reports that
// something committed, not that no lower id is in flight.
const serializeInsertSQL = `SELECT pg_advisory_xact_lock(hashtext('wsins:' || $1::text))`

// appendSQL writes one update; the writer sends many in one pgx.Batch. created_at is
// transaction START, used by the janitor and the latency measurement only.
const appendSQL = `
INSERT INTO ws_stream (doc_id, kind, data, client_id)
VALUES ($1, $2, $3, $4)`

// notifyChannel carries a doc id to every listener. One channel for all documents:
// per-document channels would mean LISTEN/UNLISTEN on every room activation.
const notifyChannel = "ws_stream_update"

// notifySQL is the last statement of every write batch; Postgres delivers at COMMIT
// and collapses duplicates, so one per batch suffices. Sent whatever this instance's
// own fan-out, or a poll-mode writer leaves notify peers blind to its writes.
const notifySQL = `SELECT pg_notify($1, $2)`

// readSQL fetches the next batch for one document after a cursor. Two things here
// look like easy optimisations and are not:
//
//  1. NO client_id filter — self-originated rows must still advance the cursor, or a
//     burst of our own writes stalls it. The caller skips them in Go.
//  2. The created_at guard is defence-in-depth for rows that reach ws_stream without
//     the insert lock (a manual backfill); it defaults to zero.
//
// The age column is the latency measurement: both timestamps are the DATABASE clock,
// so no cross-instance skew.
const readSQL = `
SELECT id, kind, data, client_id,
       EXTRACT(EPOCH FROM (clock_timestamp() - created_at))
  FROM ws_stream
 WHERE doc_id = $1
   AND id > $2
   AND created_at < now() - make_interval(secs => $3)
 ORDER BY id
 LIMIT $4`

// heartbeatSQL records this instance's liveness. Liveness is decided by seen_at,
// never by whether a row still exists — expiry is the janitor's job.
const heartbeatSQL = `
INSERT INTO ws_instance (doc_id, client_id, seen_at)
VALUES ($1, $2, now())
ON CONFLICT (doc_id, client_id) DO UPDATE SET seen_at = now()`

// activeInstancesSQL counts live instances for a document (60s window, deliberately
// shorter than the janitor's reap).
const activeInstancesSQL = `
SELECT count(*)
  FROM ws_instance
 WHERE doc_id = $1
   AND seen_at > now() - make_interval(secs => $2)`

// removeHeartbeatSQL drops this instance's row; the caller re-counts afterwards.
const removeHeartbeatSQL = `
DELETE FROM ws_instance WHERE doc_id = $1 AND client_id = $2`

// electionLockSQL is the first statement of the eviction batch, so only one instance
// elects at a time. Keep it a separate statement, not a CTE beside the DELETE: CTE
// evaluation order is not guaranteed. A batch is one transaction, so queuing it first
// is enough.
const electionLockSQL = `SELECT pg_advisory_xact_lock(hashtext('wsdoc:' || $1::text))`

// safeDeleteSQL is the last-instance election. Both guards evaluate in the DELETE's
// own snapshot. Returns rows deleted; 0 means still live (or already empty) and the
// caller must leave it alone.
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

// releaseLockSQL releases only our own lock. Without the owner predicate, a holder
// whose TTL lapsed mid-work would delete its successor's lock.
const releaseLockSQL = `DELETE FROM ws_lock WHERE key = $1 AND owner = $2`

// reapStreamSQL and reapInstancesSQL are the janitor, replacing Redis key TTLs.
const (
	reapStreamSQL    = `DELETE FROM ws_stream WHERE created_at < now() - make_interval(secs => $1)`
	reapInstancesSQL = `DELETE FROM ws_instance WHERE seen_at < now() - make_interval(secs => $1)`
	reapLocksSQL     = `DELETE FROM ws_lock WHERE expires_at < now() - make_interval(secs => $1)`
)

// secs renders a Go duration for make_interval(secs => ...). Duration.String() is
// not a valid interval literal — Go syntax, no nanosecond unit, and "m" is ambiguous
// between minutes and months.
func secs(d time.Duration) float64 { return d.Seconds() }
