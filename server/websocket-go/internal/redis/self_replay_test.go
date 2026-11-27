package redis

import (
	"context"
	"testing"
	"time"

	goredis "github.com/redis/go-redis/v9"
	"github.com/reearth/ygo/cluster"
)

// addOwn appends an entry to room's stream as if this relay had published it.
func addOwn(t *testing.T, c *goredis.Client, r *Relay, room, msgType string, data []byte) {
	t.Helper()
	if err := c.XAdd(context.Background(), &goredis.XAddArgs{Stream: streamKey(room), Values: xaddValues(msgType, data, r.clientID)}).Err(); err != nil {
		t.Fatalf("XADD: %v", err)
	}
}

func addForeign(t *testing.T, c *goredis.Client, room string, data []byte) {
	t.Helper()
	if err := c.XAdd(context.Background(), &goredis.XAddArgs{Stream: streamKey(room), Values: xaddValues(msgTypeSync, data, 424242)}).Err(); err != nil {
		t.Fatalf("XADD: %v", err)
	}
}

func injectedKind(s *fakeSink, kind cluster.Kind, data []byte) bool {
	for _, in := range s.snapshot() {
		if in.Kind == kind && string(in.Data) == string(data) {
			return true
		}
	}
	return false
}

// TestCatchUp_AppliesOwnSyncEntries: a room loaded from storage can lack this
// instance's own earlier updates, and the stream is where it gets them.
func TestCatchUp_AppliesOwnSyncEntries(t *testing.T) {
	t.Parallel()
	c, mr := newTestClient(t)
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	const room = "own-catchup"
	sink := &fakeSink{}
	relay := startedRelay(t, ctx, mr.Addr(), sink)
	addOwn(t, c, relay, room, msgTypeSync, []byte{7})

	relay.RoomActivated(room)
	if err := relay.WaitCaughtUp(ctx, room); err != nil {
		t.Fatalf("WaitCaughtUp: %v", err)
	}
	if !injectedKind(sink, cluster.KindSync, []byte{7}) {
		t.Error("catch-up skipped this instance's own sync entry")
	}
}

// TestCatchUp_SkipsOwnAwarenessEntries: re-applying this instance's own
// presence would bring back clients that left with the room it replaces.
func TestCatchUp_SkipsOwnAwarenessEntries(t *testing.T) {
	t.Parallel()
	c, mr := newTestClient(t)
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	const room = "own-awareness"
	sink := &fakeSink{}
	relay := startedRelay(t, ctx, mr.Addr(), sink)
	addOwn(t, c, relay, room, msgTypeAwareness, []byte{5, 5})

	relay.RoomActivated(room)
	if err := relay.WaitCaughtUp(ctx, room); err != nil {
		t.Fatalf("WaitCaughtUp: %v", err)
	}
	if injectedKind(sink, cluster.KindAwareness, []byte{5, 5}) {
		t.Error("catch-up applied this instance's own awareness entry")
	}
}

// TestLiveLoop_AppliesOwnSyncInsideWindow: a retiring room's last update can
// reach the stream after its replacement's replay has read it.
func TestLiveLoop_AppliesOwnSyncInsideWindow(t *testing.T) {
	t.Parallel()
	c, mr := newTestClient(t)
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	const room = "own-window"
	sink := &fakeSink{}
	relay := startedRelay(t, ctx, mr.Addr(), sink)
	relay.RoomActivated(room)
	relay.RoomActivated(room) // a replacement activated onto the live state
	t.Cleanup(func() { relay.RoomDeactivated(room); relay.RoomDeactivated(room) })
	if err := relay.WaitCaughtUp(ctx, room); err != nil {
		t.Fatalf("WaitCaughtUp: %v", err)
	}

	addOwn(t, c, relay, room, msgTypeSync, []byte{8})
	waitFor(t, 3*time.Second, func() bool { return injectedKind(sink, cluster.KindSync, []byte{8}) })
}

// TestLiveLoop_AppliesOwnSyncAfterAPartialDeactivate: the retiring room's
// deactivation opens the window too, since its lane drains after it.
func TestLiveLoop_AppliesOwnSyncAfterAPartialDeactivate(t *testing.T) {
	t.Parallel()
	c, mr := newTestClient(t)
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	const room = "own-partial"
	sink := &fakeSink{}
	relay := startedRelay(t, ctx, mr.Addr(), sink)
	relay.RoomActivated(room)
	relay.RoomActivated(room)
	t.Cleanup(func() { relay.RoomDeactivated(room) })
	if err := relay.WaitCaughtUp(ctx, room); err != nil {
		t.Fatalf("WaitCaughtUp: %v", err)
	}
	relay.mu.Lock()
	relay.rooms[room].selfApplyUntil = time.Time{} // the activation's window has passed
	relay.mu.Unlock()

	relay.RoomDeactivated(room)
	addOwn(t, c, relay, room, msgTypeSync, []byte{11})
	waitFor(t, 3*time.Second, func() bool { return injectedKind(sink, cluster.KindSync, []byte{11}) })
}

// TestLiveLoop_SkipsOwnSyncOutsideWindow: a room with a single activation
// already holds its own updates.
func TestLiveLoop_SkipsOwnSyncOutsideWindow(t *testing.T) {
	t.Parallel()
	c, mr := newTestClient(t)
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	const room = "own-no-window"
	sink := &fakeSink{}
	relay := startedRelay(t, ctx, mr.Addr(), sink)
	relay.RoomActivated(room)
	if err := relay.WaitCaughtUp(ctx, room); err != nil {
		t.Fatalf("WaitCaughtUp: %v", err)
	}

	addOwn(t, c, relay, room, msgTypeSync, []byte{9})
	addForeign(t, c, room, []byte{10})
	waitFor(t, 3*time.Second, func() bool { return injectedKind(sink, cluster.KindSync, []byte{10}) })
	if injectedKind(sink, cluster.KindSync, []byte{9}) {
		t.Error("the live reader applied its own entry with no replacement in progress")
	}
}
