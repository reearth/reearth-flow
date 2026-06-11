package redis

import (
	"context"
	"sync"
	"testing"
	"time"

	"github.com/reearth/ygo/awareness"
	"github.com/reearth/ygo/cluster"
	"github.com/reearth/ygo/crdt"
)

// fakeSink records injected inbound messages. It satisfies cluster.Sink.
type fakeSink struct {
	mu       sync.Mutex
	injected []cluster.Inbound
	rooms    []string
}

func (f *fakeSink) Inject(_ context.Context, in cluster.Inbound) error {
	f.mu.Lock()
	defer f.mu.Unlock()
	cp := append([]byte(nil), in.Data...)
	f.injected = append(f.injected, cluster.Inbound{Room: in.Room, Kind: in.Kind, Data: cp})
	return nil
}
func (f *fakeSink) Rooms() []string { return f.rooms }
func (f *fakeSink) GetAwareness(string) (*awareness.Awareness, bool) {
	return nil, false
}
func (f *fakeSink) GetDoc(string) *crdt.Doc { return nil }

func (f *fakeSink) count() int {
	f.mu.Lock()
	defer f.mu.Unlock()
	return len(f.injected)
}
func (f *fakeSink) snapshot() []cluster.Inbound {
	f.mu.Lock()
	defer f.mu.Unlock()
	return append([]cluster.Inbound(nil), f.injected...)
}

// fakeFlusher records FlushRoom calls.
type fakeFlusher struct {
	mu      sync.Mutex
	flushed []string
}

func (f *fakeFlusher) FlushRoom(_ context.Context, room string) error {
	f.mu.Lock()
	defer f.mu.Unlock()
	f.flushed = append(f.flushed, room)
	return nil
}
func (f *fakeFlusher) calls() []string {
	f.mu.Lock()
	defer f.mu.Unlock()
	return append([]string(nil), f.flushed...)
}

func waitFor(t *testing.T, d time.Duration, cond func() bool) {
	t.Helper()
	deadline := time.Now().Add(d)
	for time.Now().Before(deadline) {
		if cond() {
			return
		}
		time.Sleep(5 * time.Millisecond)
	}
	t.Fatalf("condition not met within %v", d)
}

// TestTwoInstancesFanOutWithSelfFilter: A publishes a sync update; B's reader
// injects it; A does NOT re-apply its own entry. Awareness round-trips the same way.
func TestTwoInstancesFanOutWithSelfFilter(t *testing.T) {
	_, mr := newTestClient(t)
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	const room = "proj1"

	sinkA := &fakeSink{}
	sinkB := &fakeSink{}
	relayA := mustRelay(t, mr.Addr())
	relayB := mustRelay(t, mr.Addr())
	defer relayA.Close()
	defer relayB.Close()

	if err := relayA.Start(ctx, sinkA); err != nil {
		t.Fatalf("A.Start: %v", err)
	}
	if err := relayB.Start(ctx, sinkB); err != nil {
		t.Fatalf("B.Start: %v", err)
	}
	relayA.RoomActivated(room)
	relayB.RoomActivated(room)

	if err := relayA.Publish(ctx, cluster.Outbound{Room: room, Kind: cluster.KindSync, Data: []byte{1, 2, 3}}); err != nil {
		t.Fatalf("A.Publish: %v", err)
	}

	waitFor(t, 3*time.Second, func() bool { return sinkB.count() >= 1 })
	got := sinkB.snapshot()[0]
	if got.Kind != cluster.KindSync || string(got.Data) != string([]byte{1, 2, 3}) || got.Room != room {
		t.Fatalf("B injected %+v, want sync {1,2,3} for room", got)
	}

	// A must NOT re-apply its own entry (self-filter).
	time.Sleep(300 * time.Millisecond)
	for _, in := range sinkA.snapshot() {
		if string(in.Data) == string([]byte{1, 2, 3}) {
			t.Fatal("A re-applied its own update (self-filter broken)")
		}
	}

	if err := relayB.Publish(ctx, cluster.Outbound{Room: room, Kind: cluster.KindAwareness, Data: []byte{9, 9}}); err != nil {
		t.Fatalf("B.Publish awareness: %v", err)
	}
	waitFor(t, 3*time.Second, func() bool {
		for _, in := range sinkA.snapshot() {
			if in.Kind == cluster.KindAwareness && string(in.Data) == string([]byte{9, 9}) {
				return true
			}
		}
		return false
	})
}

// TestCatchUpReplaysExistingStream verifies RoomActivated replays the stream's
// existing entries into the sink.
func TestCatchUpReplaysExistingStream(t *testing.T) {
	c, mr := newTestClient(t)
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	const room = "proj1"

	// Pre-seed a remote entry (clientId differs from the relay's).
	if err := xaddEntry(ctx, c, streamKey(room), msgTypeSync, []byte{7, 7}, 424242); err != nil {
		t.Fatalf("seed: %v", err)
	}

	sink := &fakeSink{}
	relay := mustRelay(t, mr.Addr())
	defer relay.Close()
	if err := relay.Start(ctx, sink); err != nil {
		t.Fatalf("Start: %v", err)
	}
	relay.RoomActivated(room)

	waitFor(t, 3*time.Second, func() bool {
		for _, in := range sink.snapshot() {
			if string(in.Data) == string([]byte{7, 7}) {
				return true
			}
		}
		return false
	})
}

// TestRoomActivatedWritesMarkerAndHeartbeat asserts RoomActivated sets the stream
// EXPIRE and registers a heartbeat.
func TestRoomActivatedWritesMarkerAndHeartbeat(t *testing.T) {
	_, mr := newTestClient(t)
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	const room = "proj1"

	relay := mustRelay(t, mr.Addr())
	defer relay.Close()
	_ = relay.Start(ctx, &fakeSink{})
	relay.RoomActivated(room)

	waitFor(t, 2*time.Second, func() bool { return mr.Exists(streamKey(room)) })
	if ttl := mr.TTL(streamKey(room)); ttl <= 0 {
		t.Fatalf("stream TTL not set: %v", ttl)
	}
	waitFor(t, 2*time.Second, func() bool { return mr.Exists(instancesKey(room)) })
}

// TestLastInstanceEvictsFlushesThenDeletes: the last active instance flushes GCS
// (via the injected Flusher) then deletes the stream; a non-last instance does
// neither.
func TestLastInstanceEvictsFlushesThenDeletes(t *testing.T) {
	c, mr := newTestClient(t)
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	const room = "proj1"

	t.Run("last instance flushes then deletes", func(t *testing.T) {
		flusher := &fakeFlusher{}
		relay := mustRelay(t, mr.Addr())
		relay.flusher = flusher
		_ = relay.Start(ctx, &fakeSink{})
		relay.RoomActivated(room)
		waitFor(t, 2*time.Second, func() bool { return mr.Exists(streamKey(room)) })

		relay.RoomDeactivated(room)

		waitFor(t, 3*time.Second, func() bool { return len(flusher.calls()) == 1 })
		waitFor(t, 3*time.Second, func() bool { return !mr.Exists(streamKey(room)) })
		_ = relay.Close()
	})

	t.Run("non-last instance neither flushes nor deletes", func(t *testing.T) {
		// Seed a second active heartbeat so the deactivating relay is not last.
		_ = publishEmptyMarker(ctx, c, room, 1)
		_ = updateHeartbeat(ctx, c, room, 555555)

		flusher := &fakeFlusher{}
		relay := mustRelay(t, mr.Addr())
		relay.flusher = flusher
		_ = relay.Start(ctx, &fakeSink{})
		relay.RoomActivated(room)
		waitFor(t, 2*time.Second, func() bool { return mr.Exists(instancesKey(room)) })

		relay.RoomDeactivated(room)
		time.Sleep(500 * time.Millisecond)

		if len(flusher.calls()) != 0 {
			t.Fatalf("non-last instance flushed: %v", flusher.calls())
		}
		if !mr.Exists(streamKey(room)) {
			t.Fatal("non-last instance deleted the stream")
		}
		_ = relay.Close()
	})
}

// TestPublishAfterCloseFails: Close stops everything and Publish then errors.
func TestPublishAfterCloseFails(t *testing.T) {
	_, mr := newTestClient(t)
	ctx := context.Background()
	relay := mustRelay(t, mr.Addr())
	_ = relay.Start(ctx, &fakeSink{})
	if err := relay.Close(); err != nil {
		t.Fatalf("Close: %v", err)
	}
	if err := relay.Publish(ctx, cluster.Outbound{Room: "p", Kind: cluster.KindSync, Data: []byte{1}}); err == nil {
		t.Fatal("Publish after Close returned nil, want ErrRelayClosed")
	}
}

func mustRelay(t *testing.T, addr string) *Relay {
	t.Helper()
	r, err := New(Options{Addr: addr})
	if err != nil {
		t.Fatalf("New relay: %v", err)
	}
	return r
}
