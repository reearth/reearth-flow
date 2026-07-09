package redis

import (
	"context"
	"sync"
	"testing"
	"time"
)

// hookFlusher runs fn on each FlushRoom.
type hookFlusher struct {
	fn func(room string)
}

func (f *hookFlusher) FlushRoom(_ context.Context, room string) error {
	if f.fn != nil {
		f.fn(room)
	}
	return nil
}

// TestRoomDeactivatedPreservesConcurrentReactivation reproduces H3: if a client
// reconnects into the eviction window (ygo fires RoomActivated off-lock while
// RoomDeactivated is mid-evict), RoomDeactivated's final delete must not clobber
// the freshly re-installed roomState — otherwise the live room's goroutines leak
// and every future Publish silently no-ops (the relay goes dark for that room).
func TestRoomDeactivatedPreservesConcurrentReactivation(t *testing.T) {
	_, mr := newTestClient(t)
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	const room = "proj-react"

	relay := mustRelay(t, mr.Addr())
	_ = relay.Start(ctx, &fakeSink{})

	// When the last-instance evict flushes, simulate a client reconnecting into the
	// eviction window by re-activating the room (installs a fresh roomState).
	var once sync.Once
	relay.flusher = &hookFlusher{fn: func(r string) {
		once.Do(func() { relay.RoomActivated(r) })
	}}

	relay.RoomActivated(room)
	waitFor(t, 2*time.Second, func() bool { return mr.Exists(streamKey(room)) })

	relay.RoomDeactivated(room)

	relay.mu.Lock()
	_, present := relay.rooms[room]
	relay.mu.Unlock()
	if !present {
		t.Fatal("RoomDeactivated clobbered the concurrently re-activated room: relay goes dark + goroutine leak")
	}
	_ = relay.Close()
}

// TestEvictFlushesWhenRelayContextCancelled reproduces H2: on graceful shutdown
// ygo cancels the relay delivery context BEFORE closing peers, which then triggers
// RoomDeactivated. The last-instance flush must still run — using the cancelled
// context would abort every GCS write and silently lose the room's latest state
// for a coexisting Rust reader.
func TestEvictFlushesWhenRelayContextCancelled(t *testing.T) {
	_, mr := newTestClient(t)
	ctx, cancel := context.WithCancel(context.Background())
	const room = "proj-shutdown"

	relay := mustRelay(t, mr.Addr())
	flusher := &fakeFlusher{}
	relay.flusher = flusher
	_ = relay.Start(ctx, &fakeSink{})
	relay.RoomActivated(room)
	waitFor(t, 2*time.Second, func() bool { return mr.Exists(streamKey(room)) })

	// Graceful shutdown: relay delivery context cancelled, then eviction fires.
	cancel()
	relay.RoomDeactivated(room)

	if got := flusher.calls(); len(got) != 1 {
		t.Fatalf("last-instance flush skipped on shutdown (cancelled relay ctx): calls=%v", got)
	}
	if mr.Exists(streamKey(room)) {
		t.Fatal("stream not deleted after successful last-instance flush on shutdown")
	}
	_ = relay.Close()
}
