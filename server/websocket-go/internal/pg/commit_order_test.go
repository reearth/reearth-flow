package pg

import (
	"context"
	"testing"
	"time"

	"github.com/jackc/pgx/v5/pgxpool"
)

// readRaw runs readSQL directly so a test can vary the lag and cursor without
// driving a whole relay.
func readRaw(t *testing.T, pool *pgxpool.Pool, doc string, after int64, lag time.Duration) []int64 {
	t.Helper()
	rows, err := pool.Query(context.Background(), readSQL, doc, after, secs(lag), readLimit)
	if err != nil {
		t.Fatalf("read: %v", err)
	}
	defer rows.Close()

	var ids []int64
	for rows.Next() {
		var (
			id       int64
			kind     int16
			data     []byte
			clientID int64
			ageSecs  float64
		)
		if err := rows.Scan(&id, &kind, &data, &clientID, &ageSecs); err != nil {
			t.Fatalf("scan: %v", err)
		}
		ids = append(ids, id)
	}
	if err := rows.Err(); err != nil {
		t.Fatalf("rows: %v", err)
	}
	return ids
}

const insertReturningID = `
INSERT INTO ws_stream (doc_id, kind, data, client_id)
VALUES ($1, 0, $2, 1)
RETURNING id`

// TestCommitOrderHazardAndGuard exercises the ReadLag escape hatch in isolation.
//
// bigserial allocates ids at INSERT time, but rows become visible at COMMIT time.
// So a transaction that takes a low id and commits late becomes visible AFTER a
// transaction that took a higher id and committed early. A reader tracking
// `id > cursor` that advanced on the higher id will never see the lower one — a
// silently dropped update, which for a CRDT relay means a permanently diverged
// document.
//
// IN PRODUCTION THIS INTERLEAVING CANNOT ARISE: every write batch takes the
// document's insert lock (serializeInsertSQL), so ids are allocated and committed
// in the same order — see TestInsertLockMakesCommitOrderMatchIDOrder. These two
// tests deliberately bypass that lock to construct the hazard, because ReadLag is
// the fallback for rows that reach ws_stream by some path that does not take it,
// and a fallback nobody exercises is a fallback nobody can trust.
func TestCommitOrderHazardAndGuard(t *testing.T) {
	pool := newPool(t)
	ctx := context.Background()

	// A starts first, so it takes the LOWER id — but it will commit last.
	txA, err := pool.Begin(ctx)
	if err != nil {
		t.Fatalf("begin A: %v", err)
	}
	defer func() { _ = txA.Rollback(ctx) }()

	var idA int64
	if err := txA.QueryRow(ctx, insertReturningID, room, []byte("A")).Scan(&idA); err != nil {
		t.Fatalf("insert A: %v", err)
	}

	// B starts second, takes the HIGHER id, and commits immediately.
	txB, err := pool.Begin(ctx)
	if err != nil {
		t.Fatalf("begin B: %v", err)
	}
	var idB int64
	if err := txB.QueryRow(ctx, insertReturningID, room, []byte("B")).Scan(&idB); err != nil {
		t.Fatalf("insert B: %v", err)
	}
	if err := txB.Commit(ctx); err != nil {
		t.Fatalf("commit B: %v", err)
	}

	if idB <= idA {
		t.Fatalf("setup invalid: idB=%d must exceed idA=%d", idB, idA)
	}

	// THE HAZARD. With no lag, the reader sees B and nothing else — A is still
	// uncommitted and therefore invisible. A real reader would set its cursor to
	// idB here, and A would never satisfy `id > cursor` again.
	unguarded := readRaw(t, pool, room, 0, 0)
	if len(unguarded) != 1 || unguarded[0] != idB {
		t.Fatalf("unguarded read = %v, want exactly [%d]; the hazard this guards against did not reproduce", unguarded, idB)
	}

	// THE GUARD. With a lag wider than A's transaction, nothing is eligible yet, so
	// the cursor cannot advance past B while A is in flight. That is the whole fix:
	// not detecting A, but declining to move on without it.
	guarded := readRaw(t, pool, room, 0, 5*time.Second)
	if len(guarded) != 0 {
		t.Errorf("guarded read = %v, want empty; the cursor must not advance while a lower id may still commit", guarded)
	}

	// A commits well inside the lag window.
	if err := txA.Commit(ctx); err != nil {
		t.Fatalf("commit A: %v", err)
	}

	// Once both rows are older than the lag, both are delivered, in id order.
	eventually(t, 3*time.Second, "both rows to become eligible", func() bool {
		return len(readRaw(t, pool, room, 0, 50*time.Millisecond)) == 2
	})
	got := readRaw(t, pool, room, 0, 50*time.Millisecond)
	if len(got) != 2 || got[0] != idA || got[1] != idB {
		t.Errorf("guarded read after commit = %v, want [%d %d]", got, idA, idB)
	}
}

// TestReadLagBoundIsTransactionDuration documents the guard's exact limit, so
// nobody tunes the lag down on the assumption that it is merely conservative.
//
// A row is missed only if its transaction outlives the lag. Here A is held open
// longer than the lag, the reader legitimately advances past B, and A is lost. That
// is the failure mode, and the reason the production default (100ms) is an order of
// magnitude above a single-statement INSERT.
func TestReadLagBoundIsTransactionDuration(t *testing.T) {
	pool := newPool(t)
	ctx := context.Background()

	lag := 50 * time.Millisecond

	txA, err := pool.Begin(ctx)
	if err != nil {
		t.Fatalf("begin A: %v", err)
	}
	defer func() { _ = txA.Rollback(ctx) }()
	var idA int64
	if err := txA.QueryRow(ctx, insertReturningID, room, []byte("A")).Scan(&idA); err != nil {
		t.Fatalf("insert A: %v", err)
	}

	txB, err := pool.Begin(ctx)
	if err != nil {
		t.Fatalf("begin B: %v", err)
	}
	var idB int64
	if err := txB.QueryRow(ctx, insertReturningID, room, []byte("B")).Scan(&idB); err != nil {
		t.Fatalf("insert B: %v", err)
	}
	if err := txB.Commit(ctx); err != nil {
		t.Fatalf("commit B: %v", err)
	}

	// Hold A open past the lag: B becomes eligible and the cursor advances.
	time.Sleep(4 * lag)
	cursor := int64(0)
	for _, id := range readRaw(t, pool, room, cursor, lag) {
		cursor = id
	}
	if cursor != idB {
		t.Fatalf("cursor = %d, want %d (B should have been read alone)", cursor, idB)
	}

	// Now A commits — too late. It is behind the cursor forever.
	if err := txA.Commit(ctx); err != nil {
		t.Fatalf("commit A: %v", err)
	}
	time.Sleep(4 * lag)
	if got := readRaw(t, pool, room, cursor, lag); len(got) != 0 {
		t.Fatalf("read after the cursor = %v, want empty", got)
	}

	// Confirm A really is in the table: it was dropped by the cursor, not the insert.
	var total int
	if err := pool.QueryRow(ctx, `SELECT count(*) FROM ws_stream WHERE doc_id = $1`, room).Scan(&total); err != nil {
		t.Fatalf("count: %v", err)
	}
	if total != 2 {
		t.Errorf("table holds %d rows, want 2; the setup did not persist both inserts", total)
	}
}
