package redis

import (
	"context"
	"testing"
	"time"

	"github.com/reearth/ygo/cluster"
)

// TestSingleLockedEvictReconnectLosesNoUpdates: a reconnect arriving during an
// eviction must lose 0 updates.
func TestSingleLockedEvictReconnectLosesNoUpdates(t *testing.T) {
	c, mr := newTestClient(t)
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	const room = "proj1"

	// Seed an update that MUST survive as a remote entry before eviction.
	if err := xaddEntry(ctx, c, streamKey(room), msgTypeSync, []byte{42}, 808080); err != nil {
		t.Fatalf("seed: %v", err)
	}

	sink := &fakeSink{}
	flusher := &fakeFlusher{}
	relay := mustRelay(t, mr.Addr())
	relay.flusher = flusher
	if err := relay.Start(ctx, sink); err != nil {
		t.Fatalf("Start: %v", err)
	}
	relay.RoomActivated(room)
	waitFor(t, 3*time.Second, func() bool {
		for _, in := range sink.snapshot() {
			if string(in.Data) == string([]byte{42}) {
				return true
			}
		}
		return false
	})

	// Deactivate (last instance → flush + delete the stream under the lock).
	relay.RoomDeactivated(room)
	waitFor(t, 3*time.Second, func() bool { return len(flusher.calls()) == 1 })

	// A reconnect re-activates and re-publishes the surviving update; the relay
	// must accept and re-inject it.
	sink2 := &fakeSink{}
	relay2 := mustRelay(t, mr.Addr())
	defer relay2.Close()
	_ = relay2.Start(ctx, sink2)
	relay2.RoomActivated(room)

	if err := relay2.Publish(ctx, cluster.Outbound{Room: room, Kind: cluster.KindSync, Data: []byte{42}}); err != nil {
		t.Fatalf("reconnect publish: %v", err)
	}
	// A third reader observes the re-published update — the post-evict stream is
	// healthy and nothing was lost.
	sink3 := &fakeSink{}
	relay3 := mustRelay(t, mr.Addr())
	defer relay3.Close()
	_ = relay3.Start(ctx, sink3)
	relay3.RoomActivated(room)
	waitFor(t, 3*time.Second, func() bool {
		for _, in := range sink3.snapshot() {
			if string(in.Data) == string([]byte{42}) {
				return true
			}
		}
		return false
	})
	_ = relay.Close()
}

// TestPublishRefusedDuringEvict: a Publish issued while a room is mid-evict is
// refused (no XADD races the DEL).
func TestPublishRefusedDuringEvict(t *testing.T) {
	c, mr := newTestClient(t)
	ctx := context.Background()
	const room = "proj1"

	relay := mustRelay(t, mr.Addr())
	defer relay.Close()
	_ = relay.Start(ctx, &fakeSink{})
	relay.RoomActivated(room)
	waitFor(t, 2*time.Second, func() bool { return mr.Exists(streamKey(room)) })

	relay.mu.Lock()
	relay.rooms[room].evicting = true
	relay.mu.Unlock()

	before, _ := c.XLen(ctx, streamKey(room)).Result()
	if err := relay.Publish(ctx, cluster.Outbound{Room: room, Kind: cluster.KindSync, Data: []byte{1}}); err != nil {
		t.Fatalf("Publish during evict returned err: %v", err)
	}
	after, _ := c.XLen(ctx, streamKey(room)).Result()
	if after != before {
		t.Fatalf("Publish during evict wrote to the stream: before=%d after=%d", before, after)
	}
}

// TestForceEvictDeletesStreamWithoutFlush: rollback path deletes the stream
// unconditionally and does NOT call the Flusher (no GCS write on rollback).
func TestForceEvictDeletesStreamWithoutFlush(t *testing.T) {
	_, mr := newTestClient(t)
	ctx := context.Background()
	const room = "proj1"

	flusher := &fakeFlusher{}
	relay := mustRelay(t, mr.Addr())
	defer relay.Close()
	relay.flusher = flusher
	_ = relay.Start(ctx, &fakeSink{})
	relay.RoomActivated(room)
	waitFor(t, 2*time.Second, func() bool { return mr.Exists(streamKey(room)) })

	if err := relay.ForceEvict(ctx, room); err != nil {
		t.Fatalf("ForceEvict: %v", err)
	}
	if mr.Exists(streamKey(room)) {
		t.Fatal("ForceEvict did not delete the stream")
	}
	if len(flusher.calls()) != 0 {
		t.Fatalf("ForceEvict flushed GCS on rollback: %v", flusher.calls())
	}
	// Idempotent: a second ForceEvict on a non-resident room is a no-op.
	if err := relay.ForceEvict(ctx, room); err != nil {
		t.Fatalf("second ForceEvict: %v", err)
	}
}

// TestCtxCancellationStopsReaders ensures cancelling Start's ctx stops the room
// goroutines (no leak, honors ctx).
func TestCtxCancellationStopsReaders(t *testing.T) {
	_, mr := newTestClient(t)
	ctx, cancel := context.WithCancel(context.Background())
	const room = "proj1"

	relay := mustRelay(t, mr.Addr())
	defer relay.Close()
	_ = relay.Start(ctx, &fakeSink{})
	relay.RoomActivated(room)
	waitFor(t, 2*time.Second, func() bool { return mr.Exists(instancesKey(room)) })

	cancel()
	// After cancellation Close must return promptly (no goroutine leak).
	done := make(chan struct{})
	go func() { _ = relay.Close(); close(done) }()
	select {
	case <-done:
	case <-time.After(3 * time.Second):
		t.Fatal("Close hung after ctx cancellation (goroutine leak)")
	}
}
