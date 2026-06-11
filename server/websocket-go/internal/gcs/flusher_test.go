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

// The Flusher satisfies the redis.Flusher seam structurally.
var _ interface {
	FlushRoom(ctx context.Context, room string) error
} = (*Flusher)(nil)
