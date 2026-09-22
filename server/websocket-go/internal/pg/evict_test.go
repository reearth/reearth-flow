package pg

import (
	"context"
	"errors"
	"testing"
	"time"

	"github.com/jackc/pgx/v5"
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
