package gcs

import (
	"context"
	"testing"

	"github.com/alicebob/miniredis/v2"
	goredis "github.com/redis/go-redis/v9"
	"github.com/reearth/ygo/crdt"
)

// TestFlushRoomHoldsReadLock proves FlushRoom persists the doc state and holds
// read:lock:{room} during the critical section, releasing it afterward.
func TestFlushRoomHoldsReadLock(t *testing.T) {
	mr, err := miniredis.Run()
	if err != nil {
		t.Fatalf("miniredis: %v", err)
	}
	t.Cleanup(mr.Close)
	rc := goredis.NewClient(&goredis.Options{Addr: mr.Addr()})
	t.Cleanup(func() { _ = rc.Close() })

	client, bucket := newFakeGCS(t)
	a, _ := New(Options{Client: client, Bucket: bucket, Locker: NewNoLock()})
	const room = "proj-flush"

	doc := crdt.New(crdt.WithClientID(5))
	txt := doc.GetText("t")
	doc.Transact(func(txn *crdt.Transaction) { txt.Insert(txn, 0, "abc", nil) })
	state := crdt.EncodeStateAsUpdateV1(doc, nil)

	var lockHeldDuringFlush bool
	f := NewFlusher(FlusherOptions{
		Adapter: a,
		Redis:   rc,
		Owner:   "instance-test",
		StateOf: func(r string) []byte {
			// Observed from inside the critical section: the lock must be held here.
			if r == room {
				lockHeldDuringFlush = mr.Exists(readLockName(room))
				return state
			}
			return nil
		},
	})

	if err := f.FlushRoom(context.Background(), room); err != nil {
		t.Fatalf("FlushRoom: %v", err)
	}
	if !lockHeldDuringFlush {
		t.Fatal("read:lock was NOT held during the flush critical section")
	}
	// Lock released after the flush.
	if mr.Exists(readLockName(room)) {
		t.Fatal("read:lock was not released after FlushRoom")
	}
	// State persisted: Load reproduces the text.
	lr, err := a.Load(context.Background(), room)
	if err != nil {
		t.Fatalf("Load: %v", err)
	}
	rebuilt := crdt.New()
	if err := crdt.ApplyUpdateV1(rebuilt, lr.Update, nil); err != nil {
		t.Fatalf("ApplyUpdateV1: %v", err)
	}
	if got := rebuilt.GetText("t").ToString(); got != "abc" {
		t.Fatalf("flushed text = %q, want %q", got, "abc")
	}
}

// TestFlushRoomNoopWhenEmpty: no resident state → no version written.
func TestFlushRoomNoopWhenEmpty(t *testing.T) {
	client, bucket := newFakeGCS(t)
	a, _ := New(Options{Client: client, Bucket: bucket, Locker: NewNoLock()})
	f := NewFlusher(FlusherOptions{Adapter: a, StateOf: func(string) []byte { return nil }})
	if err := f.FlushRoom(context.Background(), "room"); err != nil {
		t.Fatalf("FlushRoom: %v", err)
	}
	metas, err := a.ListVersions(context.Background(), "room")
	if err != nil {
		t.Fatalf("ListVersions: %v", err)
	}
	if len(metas) != 0 {
		t.Fatalf("empty flush wrote %d versions, want 0", len(metas))
	}
}

// TestFlushRoomWritesCompleteDocV2 pins cross-implementation read compatibility
// with the Rust server. Rust's room load (yrs load_doc_v2) reads ONLY the doc_v2
// object: decompress brotli -> Update::decode_v2 -> apply, with NO folding of tail
// update objects. So after a last-instance flush, doc_v2 MUST hold the complete
// current state, otherwise a Rust instance cold-loading a Go-flushed room (once
// the Redis stream is gone) reconstructs an empty or stale document. This test
// reads doc_v2 the way Rust does and fails if it is missing or incomplete.
func TestFlushRoomWritesCompleteDocV2(t *testing.T) {
	client, bucket := newFakeGCS(t)
	a, _ := New(Options{Client: client, Bucket: bucket, Locker: NewNoLock()})
	ctx := context.Background()
	const room = "proj-xcompat"

	doc := crdt.New(crdt.WithClientID(9))
	txt := doc.GetText("t")
	doc.Transact(func(txn *crdt.Transaction) { txt.Insert(txn, 0, "cross-impl-state", nil) })
	state := crdt.EncodeStateAsUpdateV1(doc, nil)

	f := NewFlusher(FlusherOptions{
		Adapter: a,
		StateOf: func(r string) []byte {
			if r == room {
				return state
			}
			return nil
		},
	})
	if err := f.FlushRoom(ctx, room); err != nil {
		t.Fatalf("FlushRoom: %v", err)
	}

	// Read exactly what the Rust server reads: the doc_v2 object alone.
	raw, err := a.store.get(ctx, a.layout.DocV2Name(room))
	if err != nil {
		t.Fatalf("FlushRoom did not write doc_v2 (a Rust reader would see an empty document): %v", err)
	}
	v2, err := decompressBrotli(raw)
	if err != nil {
		t.Fatalf("doc_v2 is not valid brotli: %v", err)
	}
	got := crdt.New()
	if err := crdt.ApplyUpdateV2(got, v2, nil); err != nil {
		t.Fatalf("doc_v2 is not a valid V2 update: %v", err)
	}
	if s := got.GetText("t").ToString(); s != "cross-impl-state" {
		t.Fatalf("doc_v2-only read = %q, want %q (Rust would not see the flushed state)", s, "cross-impl-state")
	}
}

// TestFlushRoomSnapshotsFromStoreWhenDocEvicted mirrors production: ygo removes
// the live doc from GetDoc before firing RoomDeactivated, so the flusher's StateOf
// returns nil. ygo first drains the per-update persistence worker, so the updates
// are already in GCS. FlushRoom must reconstruct the full state from GCS and write
// a complete doc_v2 the Rust reader can load.
func TestFlushRoomSnapshotsFromStoreWhenDocEvicted(t *testing.T) {
	client, bucket := newFakeGCS(t)
	a, _ := New(Options{Client: client, Bucket: bucket, Locker: NewNoLock()})
	ctx := context.Background()
	const room = "proj-evicted"

	// Simulate ygo's per-update persistence (StoreUpdate -> AppendUpdate) that ran
	// while the doc was resident.
	doc := crdt.New(crdt.WithClientID(11))
	txt := doc.GetText("t")
	doc.Transact(func(txn *crdt.Transaction) { txt.Insert(txn, 0, "persisted-incrementally", nil) })
	if _, err := a.AppendUpdate(ctx, room, crdt.EncodeStateAsUpdateV1(doc, nil)); err != nil {
		t.Fatalf("AppendUpdate: %v", err)
	}

	// Live doc already torn down: StateOf returns nil, as in production.
	f := NewFlusher(FlusherOptions{Adapter: a, StateOf: func(string) []byte { return nil }})
	if err := f.FlushRoom(ctx, room); err != nil {
		t.Fatalf("FlushRoom: %v", err)
	}

	// Rust-style read: doc_v2 alone must reconstruct the full content.
	raw, err := a.store.get(ctx, a.layout.DocV2Name(room))
	if err != nil {
		t.Fatalf("FlushRoom did not write doc_v2 from the persisted GCS state (Rust would read empty): %v", err)
	}
	v2, err := decompressBrotli(raw)
	if err != nil {
		t.Fatalf("doc_v2 not valid brotli: %v", err)
	}
	got := crdt.New()
	if err := crdt.ApplyUpdateV2(got, v2, nil); err != nil {
		t.Fatalf("doc_v2 not a valid V2 update: %v", err)
	}
	if s := got.GetText("t").ToString(); s != "persisted-incrementally" {
		t.Fatalf("doc_v2-only read = %q, want %q", s, "persisted-incrementally")
	}
}

// The Flusher satisfies the redis.Flusher seam structurally.
var _ interface {
	FlushRoom(ctx context.Context, room string) error
} = (*Flusher)(nil)
