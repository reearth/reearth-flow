package redis

import (
	"context"
	"strconv"
	"testing"
	"time"
)

// TestEvict_KeepsStreamWhileAnotherInstanceLoads: an instance loading the room
// is not active yet, so the last instance would delete the stream that load
// still needs; its fence keeps it.
func TestEvict_KeepsStreamWhileAnotherInstanceLoads(t *testing.T) {
	t.Parallel()
	c, mr := newTestClient(t)
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	const room = "evict-other-loads"
	relay := startedRelay(t, ctx, mr.Addr(), &fakeSink{})
	if err := c.HSet(ctx, loadsKey(room), "987654321", time.Now().Add(time.Minute).Unix()).Err(); err != nil {
		t.Fatalf("HSET: %v", err)
	}

	relay.RoomActivated(room)
	relay.RoomDeactivated(room)
	if !mr.Exists(streamKey(room)) {
		t.Error("the last instance deleted the stream while another instance was loading the room")
	}
}

// TestEvict_KeepsStreamWhileThisInstanceLoads: a load whose fence never reached
// Redis is still fenced here, and this instance keeps the stream for it.
func TestEvict_KeepsStreamWhileThisInstanceLoads(t *testing.T) {
	t.Parallel()
	c, mr := newTestClient(t)
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	const room = "evict-self-loads"
	relay := startedRelay(t, ctx, mr.Addr(), &fakeSink{})
	relay.RoomActivated(room)
	if err := relay.WaitCaughtUp(ctx, room); err != nil {
		t.Fatalf("WaitCaughtUp: %v", err)
	}
	relay.LoadStarting(ctx, room)(nil) // after the catch-up, so no pass covers it
	if err := c.HDel(ctx, loadsKey(room), strconv.FormatUint(relay.clientID, 10)).Err(); err != nil {
		t.Fatalf("HDEL: %v", err)
	}

	relay.RoomDeactivated(room)
	if !mr.Exists(streamKey(room)) {
		t.Error("the last instance deleted the stream while it was loading the room itself")
	}
}
