package pg

import (
	"bytes"
	"context"
	"errors"
	"log/slog"
	"strings"
	"sync"
	"testing"
	"time"

	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgconn"
	"github.com/reearth/ygo/cluster"
)

// TestLastInstanceFlushesThenDeletes is the happy path of the election that
// safeDeleteSQL replaces ~40 lines of Lua to express: the final instance leaving
// must persist to GCS and only then drop the rows.
func TestLastInstanceFlushesThenDeletes(t *testing.T) {
	pool := newPool(t)
	fl := &countingFlusher{}
	r := startRelay(t, pool, &fakeSink{}, fl)

	r.RoomActivated(room)
	if err := r.Publish(context.Background(), cluster.Outbound{
		Room: room, Kind: cluster.KindSync, Data: []byte("state"),
	}); err != nil {
		t.Fatalf("Publish: %v", err)
	}
	eventually(t, 3*time.Second, "the write to persist", func() bool {
		return rowCount(t, pool, room) == 1
	})

	r.RoomDeactivated(room)

	if fl.count() != 1 {
		t.Errorf("FlushRoom called %d times, want 1", fl.count())
	}
	if n := rowCount(t, pool, room); n != 0 {
		t.Errorf("%d rows survived the last-instance evict, want 0", n)
	}
}

// TestEvictKeepsRowsWhileAnotherInstanceIsActive: deleting a live document's rows
// would strand the remaining instance, whose cursor is already past them, so a
// reconnecting client would replay nothing.
func TestEvictKeepsRowsWhileAnotherInstanceIsActive(t *testing.T) {
	pool := newPool(t)
	fl := &countingFlusher{}

	leaving := startRelay(t, pool, &fakeSink{}, fl)
	staying := startRelay(t, pool, &fakeSink{}, &countingFlusher{})

	leaving.RoomActivated(room)
	staying.RoomActivated(room)

	if err := leaving.Publish(context.Background(), cluster.Outbound{
		Room: room, Kind: cluster.KindSync, Data: []byte("shared"),
	}); err != nil {
		t.Fatalf("Publish: %v", err)
	}
	eventually(t, 3*time.Second, "the write to persist", func() bool {
		return rowCount(t, pool, room) == 1
	})

	leaving.RoomDeactivated(room)

	if fl.count() != 0 {
		t.Errorf("FlushRoom called %d times while a peer was active, want 0", fl.count())
	}
	if n := rowCount(t, pool, room); n != 1 {
		t.Errorf("%d rows left while a peer was active, want 1", n)
	}
}

// TestEvictKeepsRowsWhenFlushFails: the rows are the only copy of updates not yet
// in GCS. Deleting them after a failed flush loses user edits outright, so the
// relay must leave them for a reconnect to replay.
func TestEvictKeepsRowsWhenFlushFails(t *testing.T) {
	pool := newPool(t)
	fl := &countingFlusher{fail: errors.New("gcs unavailable")}
	r := startRelay(t, pool, &fakeSink{}, fl)

	r.RoomActivated(room)
	if err := r.Publish(context.Background(), cluster.Outbound{
		Room: room, Kind: cluster.KindSync, Data: []byte("unpersisted"),
	}); err != nil {
		t.Fatalf("Publish: %v", err)
	}
	eventually(t, 3*time.Second, "the write to persist", func() bool {
		return rowCount(t, pool, room) == 1
	})

	r.RoomDeactivated(room)

	if fl.count() != 1 {
		t.Errorf("FlushRoom called %d times, want 1", fl.count())
	}
	if n := rowCount(t, pool, room); n != 1 {
		t.Errorf("%d rows after a failed flush, want 1 (un-persisted updates must survive)", n)
	}
}

// TestReadLockFencesTheDelete is why ws_lock exists at all. A flush in progress on
// another instance holds the room's read lock; the election must see it and refuse
// to delete, or it would pull rows out from under an in-flight reader.
func TestReadLockFencesTheDelete(t *testing.T) {
	pool := newPool(t)
	r := startRelay(t, pool, &fakeSink{}, &countingFlusher{})

	r.RoomActivated(room)
	if err := r.Publish(context.Background(), cluster.Outbound{
		Room: room, Kind: cluster.KindSync, Data: []byte("mid-read"),
	}); err != nil {
		t.Fatalf("Publish: %v", err)
	}
	eventually(t, 3*time.Second, "the write to persist", func() bool {
		return rowCount(t, pool, room) == 1
	})

	// Another instance is mid-flush: it holds the room's read lock.
	peer := NewLocker(pool, "instance-peer")
	held, err := peer.tryAcquire(context.Background(), lockKeyFor(room), time.Minute)
	if err != nil || !held {
		t.Fatalf("tryAcquire read lock: held=%v err=%v", held, err)
	}

	r.RoomDeactivated(room)

	if n := rowCount(t, pool, room); n != 1 {
		t.Errorf("%d rows deleted while the read lock was held, want 1 kept", n)
	}
}

// TestPublishRefusedDuringEvict: an INSERT landing between the flush and the DELETE
// would be deleted unread, so Publish must be refused for the whole critical
// section rather than merely before it.
func TestPublishRefusedDuringEvict(t *testing.T) {
	pool := newPool(t)

	// A flusher that publishes from inside the critical section, which is the
	// narrow window the evicting flag exists to close.
	var r *Relay
	fl := &publishingFlusher{publish: func() {
		_ = r.Publish(context.Background(), cluster.Outbound{
			Room: room, Kind: cluster.KindSync, Data: []byte("racing"),
		})
	}}
	r = startRelay(t, pool, &fakeSink{}, fl)

	r.RoomActivated(room)
	if err := r.Publish(context.Background(), cluster.Outbound{
		Room: room, Kind: cluster.KindSync, Data: []byte("original"),
	}); err != nil {
		t.Fatalf("Publish: %v", err)
	}
	eventually(t, 3*time.Second, "the write to persist", func() bool {
		return rowCount(t, pool, room) == 1
	})

	r.RoomDeactivated(room)

	if n := rowCount(t, pool, room); n != 0 {
		t.Errorf("%d rows after evict, want 0 — a Publish during eviction was not refused", n)
	}
}

type publishingFlusher struct{ publish func() }

func (f *publishingFlusher) FlushRoom(context.Context, string) error {
	f.publish()
	return nil
}

// TestForceEvictDeletesWithoutFlush: the rollback path must not persist the state
// it is discarding, or the rolled-back version would be written back to GCS as the
// current one.
func TestForceEvictDeletesWithoutFlush(t *testing.T) {
	pool := newPool(t)
	fl := &countingFlusher{}
	r := startRelay(t, pool, &fakeSink{}, fl)

	r.RoomActivated(room)
	if err := r.Publish(context.Background(), cluster.Outbound{
		Room: room, Kind: cluster.KindSync, Data: []byte("rolled-back"),
	}); err != nil {
		t.Fatalf("Publish: %v", err)
	}
	eventually(t, 3*time.Second, "the write to persist", func() bool {
		return rowCount(t, pool, room) == 1
	})

	if err := r.ForceEvict(context.Background(), room); err != nil {
		t.Fatalf("ForceEvict: %v", err)
	}
	if fl.count() != 0 {
		t.Errorf("FlushRoom called %d times on the rollback path, want 0", fl.count())
	}
	if n := rowCount(t, pool, room); n != 0 {
		t.Errorf("%d rows after ForceEvict, want 0", n)
	}

	// Idempotent: a second call on a non-resident room must not error.
	if err := r.ForceEvict(context.Background(), room); err != nil {
		t.Errorf("second ForceEvict: %v", err)
	}
}

// TestRoomDeactivatedIsIdempotent: ygo can fire it twice across an eviction/reload
// handoff, and the second call must not re-run the election against a fresh room.
func TestRoomDeactivatedIsIdempotent(t *testing.T) {
	pool := newPool(t)
	fl := &countingFlusher{}
	r := startRelay(t, pool, &fakeSink{}, fl)

	r.RoomActivated(room)
	r.RoomDeactivated(room)
	r.RoomDeactivated(room)

	if fl.count() > 1 {
		t.Errorf("FlushRoom called %d times for one deactivation, want at most 1", fl.count())
	}
}

// TestElectionLockIsHeldBeforeDelete pins the ordering the election depends on: the
// advisory lock must be held before the DELETE runs, or concurrent electors are not
// serialised at all.
//
// The lock previously sat in a CTE beside the DELETE. Measured on Postgres 18 that
// ordering did hold, so this is hardening rather than a bug fix — but CTE evaluation
// order is not guaranteed, and nothing would have surfaced a change in it. Queuing
// the lock as its own batched statement makes the ordering explicit, and this test
// makes a regression loud.
//
// It holds the election lock from an outside session, then runs the eviction batch
// with a short timeout. Taking the lock first means blocking and timing out with the
// rows intact; deleting without waiting means the lock is decorative.
func TestElectionLockIsHeldBeforeDelete(t *testing.T) {
	pool := newPool(t)
	ctx := context.Background()

	for i := 0; i < 3; i++ {
		if _, err := pool.Exec(ctx, appendSQL, room, kindSync, []byte("x"), int64(99)); err != nil {
			t.Fatalf("seed row: %v", err)
		}
	}

	// Hold the election lock in its own transaction, from outside the relay.
	blocker, err := pool.Begin(ctx)
	if err != nil {
		t.Fatalf("begin blocker: %v", err)
	}
	defer func() { _ = blocker.Rollback(ctx) }()
	if _, err := blocker.Exec(ctx, electionLockSQL, room); err != nil {
		t.Fatalf("blocker take lock: %v", err)
	}

	r := startRelay(t, pool, &fakeSink{}, nil)

	// No heartbeats and no read lock, so the guards would otherwise permit deletion.
	blocked, cancel := context.WithTimeout(ctx, 750*time.Millisecond)
	defer cancel()

	b := &pgx.Batch{}
	b.Queue(electionLockSQL, room)
	b.Queue(safeDeleteSQL, room, lockKeyFor(room), secs(activeTimeout))
	br := r.q.SendBatch(blocked, b)
	_, execErr := br.Exec()
	_ = br.Close()

	if execErr == nil {
		t.Fatal("eviction batch completed while another session held the election lock; the lock is not serialising electors")
	}

	if n := rowCount(t, pool, room); n != 3 {
		t.Errorf("rows = %d, want 3 left intact while the lock was held", n)
	}
}

// reactivatingQuerier fires a hook the first time it sees the force-evict heartbeat
// removal, which is exactly the window between rs.wg.Wait() and the row delete. It
// is the only way to stage the reconnect deterministically: staging it BEFORE the
// call proves nothing, because then the successor is simply the resident state and
// any implementation evicts it correctly.
type reactivatingQuerier struct {
	Querier
	once sync.Once
	on   func()
}

func (q *reactivatingQuerier) Exec(ctx context.Context, sql string, args ...any) (pgconn.CommandTag, error) {
	if sql == removeHeartbeatSQL && q.on != nil {
		q.once.Do(q.on)
	}
	return q.Querier.Exec(ctx, sql, args...)
}

// TestForceEvictConvergesWhenRoomReactivates covers a race the evicting flag alone
// does not close: the flag belongs to one roomState, not to the room name.
// RoomActivated treats an evicting state as absent and installs a FRESH one, so a
// reconnect landing mid-eviction leaves a live successor whose heartbeat and rows an
// unconditional delete then erases while its goroutines keep running.
//
// The reconnect is injected at the delete boundary, so the successor appears exactly
// where the old code could not see it.
func TestForceEvictConvergesWhenRoomReactivates(t *testing.T) {
	pool := newPool(t)
	q := &reactivatingQuerier{Querier: pool}

	r, err := New(Options{Q: q, PollEvery: 5 * time.Millisecond, Logger: testLogger(t)})
	if err != nil {
		t.Fatalf("New: %v", err)
	}
	ctx, cancel := context.WithCancel(context.Background())
	t.Cleanup(func() { cancel(); _ = r.Close() })
	if err := r.Start(ctx, &fakeSink{}); err != nil {
		t.Fatalf("Start: %v", err)
	}

	r.RoomActivated(room)
	r.mu.Lock()
	first := r.rooms[room]
	r.mu.Unlock()

	// The reconnect, landing inside ForceEvict after the goroutines have stopped.
	q.on = func() { r.RoomActivated(room) }

	if err := r.ForceEvict(context.Background(), room); err != nil {
		t.Fatalf("ForceEvict: %v", err)
	}

	r.mu.Lock()
	leftover, stillResident := r.rooms[room]
	r.mu.Unlock()
	if stillResident {
		t.Fatal("room still resident after ForceEvict: a successor was installed mid-eviction and left live with its rows deleted")
	}
	if leftover == first {
		t.Fatal("unexpected: the original state is still mapped")
	}
	if n := rowCount(t, pool, room); n != 0 {
		t.Errorf("rows = %d, want 0", n)
	}
}

// TestBatchIsOneTransaction pins the property the eviction election depends on:
// electionLockSQL and safeDeleteSQL are queued in one pgx.Batch, and the lock is
// pg_advisory_xact_lock — transaction-scoped. If a batch were NOT one transaction,
// the lock would be released before the delete and concurrent electors would not be
// serialised at all.
//
// pgx pipelines a batch with a single Sync, and Postgres treats messages between
// Syncs as one implicit transaction. That is documented, but it is subtle enough to
// have been questioned twice in review, so it is asserted here rather than trusted.
func TestBatchIsOneTransaction(t *testing.T) {
	pool := newPool(t)
	ctx := context.Background()

	b := &pgx.Batch{}
	b.Queue("SELECT pg_current_xact_id()")
	b.Queue("SELECT pg_current_xact_id()")
	br := pool.SendBatch(ctx, b)
	var first, second uint64
	if err := br.QueryRow().Scan(&first); err != nil {
		t.Fatalf("first xid: %v", err)
	}
	if err := br.QueryRow().Scan(&second); err != nil {
		t.Fatalf("second xid: %v", err)
	}
	if err := br.Close(); err != nil {
		t.Fatalf("close: %v", err)
	}
	if first != second {
		t.Fatalf("batch statements ran in different transactions (%d, %d); the election lock is released before the delete", first, second)
	}
}

// TestElectionLockSurvivesIntoTheDelete is the same property stated in the terms the
// election actually cares about: the advisory lock taken by the batch's first
// statement must still be held while its second runs.
func TestElectionLockSurvivesIntoTheDelete(t *testing.T) {
	pool := newPool(t)
	ctx := context.Background()

	b := &pgx.Batch{}
	b.Queue(electionLockSQL, room)
	b.Queue(`SELECT count(*) FROM pg_locks WHERE locktype = 'advisory' AND pid = pg_backend_pid()`)
	br := pool.SendBatch(ctx, b)
	if _, err := br.Exec(); err != nil {
		t.Fatalf("take lock: %v", err)
	}
	var held int
	if err := br.QueryRow().Scan(&held); err != nil {
		t.Fatalf("count advisory locks: %v", err)
	}
	if err := br.Close(); err != nil {
		t.Fatalf("close: %v", err)
	}
	if held != 1 {
		t.Fatalf("advisory locks held during the second statement = %d, want 1", held)
	}
}

// TestEvictionDoesNotClaimSuccessWhenNothingDeleted: safeDeleteSQL returns 0 when
// the election declines — a peer re-activated, or a flush holds the read lock. The
// log used to say "evicted room" regardless, so an operator reading it would believe
// a document had been cleaned up while its rows were still there.
func TestEvictionDoesNotClaimSuccessWhenNothingDeleted(t *testing.T) {
	pool := newPool(t)
	ctx := context.Background()

	var buf bytes.Buffer
	r := startRelayWith(t, pool, &fakeSink{}, nil, Options{
		PollEvery: 5 * time.Millisecond,
		Logger:    slog.New(slog.NewJSONHandler(&buf, &slog.HandlerOptions{Level: slog.LevelDebug})),
	})
	r.RoomActivated(room)

	// A second instance keeps the document live, so the election must decline.
	peer := startRelay(t, pool, &fakeSink{}, nil)
	peer.RoomActivated(room)

	for i := 0; i < 3; i++ {
		if _, err := pool.Exec(ctx, appendSQL, room, kindSync, []byte("x"), int64(99)); err != nil {
			t.Fatalf("seed: %v", err)
		}
	}

	r.RoomDeactivated(room)

	logged := buf.String()
	if strings.Contains(logged, "relay evicted room") {
		t.Errorf("logged an eviction while a peer was still active:\n%s", logged)
	}
	if n := rowCount(t, pool, room); n == 0 {
		t.Error("rows were deleted while a peer was still active")
	}
}
