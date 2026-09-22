package pg

import (
	"context"
	"sync"
	"testing"
	"time"

	"github.com/reearth/ygo/cluster"
)

// TestInsertLockMakesCommitOrderMatchIDOrder is the primary correctness guarantee
// the reader's `id > cursor` cursor rests on.
//
// bigserial allocates ids at INSERT and rows appear at COMMIT, so without
// serialization those two orders can disagree and a reader that advanced past a
// higher id never sees the lower one. serializeInsertSQL removes the possibility by
// holding a per-document lock across the insert transaction.
//
// The test hammers one document from many goroutines and asserts the invariant
// directly: scanning rows in id order must also be scanning them in xmin (commit)
// order. A regression here is a silently diverging document, so it is checked
// against the database's own transaction ids rather than inferred from timing.
func TestInsertLockMakesCommitOrderMatchIDOrder(t *testing.T) {
	pool := newPool(t)
	ctx := context.Background()

	const writers, each = 8, 20

	var wg sync.WaitGroup
	for w := 0; w < writers; w++ {
		wg.Go(func() {
			for i := 0; i < each; i++ {
				// Exactly what the writer does: lock first, then insert, in one
				// implicit transaction.
				tx, err := pool.Begin(ctx)
				if err != nil {
					t.Errorf("begin: %v", err)
					return
				}
				if _, err := tx.Exec(ctx, serializeInsertSQL, room); err != nil {
					t.Errorf("lock: %v", err)
					_ = tx.Rollback(ctx)
					return
				}
				if _, err := tx.Exec(ctx, appendSQL, room, kindSync, []byte("x"), int64(1)); err != nil {
					t.Errorf("insert: %v", err)
					_ = tx.Rollback(ctx)
					return
				}
				if err := tx.Commit(ctx); err != nil {
					t.Errorf("commit: %v", err)
					return
				}
			}
		})
	}
	wg.Wait()

	// xmin is the inserting transaction's id. Because each insert committed while
	// holding the document's lock, ascending id must imply ascending xmin.
	rows, err := pool.Query(ctx, `
		SELECT id, xmin::text::bigint
		  FROM ws_stream
		 WHERE doc_id = $1
		 ORDER BY id`, room)
	if err != nil {
		t.Fatalf("query: %v", err)
	}
	defer rows.Close()

	var (
		count    int
		prevID   int64
		prevXmin int64
	)
	for rows.Next() {
		var id, xmin int64
		if err := rows.Scan(&id, &xmin); err != nil {
			t.Fatalf("scan: %v", err)
		}
		if count > 0 && xmin < prevXmin {
			t.Fatalf("id %d (xmin %d) committed before id %d (xmin %d): ids are not in commit order, so the reader's cursor can skip a row",
				id, xmin, prevID, prevXmin)
		}
		prevID, prevXmin = id, xmin
		count++
	}
	if err := rows.Err(); err != nil {
		t.Fatalf("rows: %v", err)
	}
	if count != writers*each {
		t.Fatalf("persisted %d rows, want %d", count, writers*each)
	}
}

// TestNoUpdatesLostUnderConcurrentWriters is the soak the plan calls for before any
// latency number is trusted: two instances writing the same document concurrently,
// with every update accounted for at the far end.
//
// It is the end-to-end statement of the same invariant as the test above — that no
// row is skipped — but measured where it matters: at the sink.
func TestNoUpdatesLostUnderConcurrentWriters(t *testing.T) {
	pool := newPool(t)

	readerSink := &fakeSink{}
	a := startRelay(t, pool, &fakeSink{}, nil)
	b := startRelay(t, pool, &fakeSink{}, nil)
	reader := startRelay(t, pool, readerSink, nil)

	a.RoomActivated(room)
	b.RoomActivated(room)
	reader.RoomActivated(room)

	const each = 100
	var wg sync.WaitGroup
	for _, w := range []*Relay{a, b} {
		relay := w
		wg.Go(func() {
			for i := 0; i < each; i++ {
				if err := relay.Publish(context.Background(), cluster.Outbound{
					Room: room, Kind: cluster.KindSync, Data: []byte("u"),
				}); err != nil {
					t.Errorf("Publish: %v", err)
					return
				}
			}
		})
	}
	wg.Wait()

	want := 2 * each
	eventually(t, 15*time.Second, "every update to reach the reader", func() bool {
		return readerSink.count() >= want
	})
	if got := readerSink.count(); got != want {
		t.Errorf("reader saw %d updates, want exactly %d", got, want)
	}
	if d := a.DroppedWrites() + b.DroppedWrites(); d != 0 {
		t.Errorf("%d writes dropped on a full queue; the count above is not a clean result", d)
	}
}
