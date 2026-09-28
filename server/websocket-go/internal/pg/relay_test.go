package pg

import (
	"context"
	"errors"
	"fmt"
	"testing"
	"time"

	"github.com/reearth/ygo/cluster"
)

const room = "550e8400-e29b-41d4-a716-446655440000"

// TestPublishReachesOtherInstance is the relay's entire purpose: an update written
// by one instance must appear at another. Everything else in this file is a guard
// around this behaviour.
func TestPublishReachesOtherInstance(t *testing.T) {
	pool := newPool(t)

	writerSink, readerSink := &fakeSink{}, &fakeSink{}
	writer := startRelay(t, pool, writerSink, nil)
	reader := startRelay(t, pool, readerSink, nil)

	writer.RoomActivated(room)
	reader.RoomActivated(room)

	if err := writer.Publish(context.Background(), cluster.Outbound{
		Room: room, Kind: cluster.KindSync, Data: []byte("hello"),
	}); err != nil {
		t.Fatalf("Publish: %v", err)
	}

	eventually(t, 3*time.Second, "reader to receive the update", func() bool {
		return readerSink.count() == 1
	})
	if got := string(readerSink.payloads()[0]); got != "hello" {
		t.Errorf("payload = %q, want %q", got, "hello")
	}
}

// TestSelfWritesAreNotInjected: without the client_id check an instance would apply
// its own update back to itself, and the local observer would republish it — the
// echo loop cluster.Relay's sentinel exists to prevent.
func TestSelfWritesAreNotInjected(t *testing.T) {
	pool := newPool(t)

	sink := &fakeSink{}
	r := startRelay(t, pool, sink, nil)
	r.RoomActivated(room)

	for i := 0; i < 5; i++ {
		if err := r.Publish(context.Background(), cluster.Outbound{
			Room: room, Kind: cluster.KindSync, Data: []byte{byte(i)},
		}); err != nil {
			t.Fatalf("Publish: %v", err)
		}
	}

	// All five rows must land...
	eventually(t, 3*time.Second, "writes to be persisted", func() bool {
		return rowCount(t, pool, room) == 5
	})
	// ...and the reader must have polled past them at least once.
	time.Sleep(100 * time.Millisecond)

	if n := sink.count(); n != 0 {
		t.Errorf("injected %d self-originated updates, want 0", n)
	}
}

// TestCursorAdvancesPastSelfRows guards the subtlest carry-over from the Redis
// relay (relay.go:297): self rows are skipped on apply but must still move the
// cursor. Filtering them in SQL instead would leave the cursor stuck behind them,
// so a later remote update would be re-read forever and never delivered.
func TestCursorAdvancesPastSelfRows(t *testing.T) {
	pool := newPool(t)

	selfSink, peerSink := &fakeSink{}, &fakeSink{}
	self := startRelay(t, pool, selfSink, nil)
	peer := startRelay(t, pool, peerSink, nil)

	self.RoomActivated(room)
	peer.RoomActivated(room)

	// A burst of our own writes first: these must not dam the cursor.
	for i := 0; i < 200; i++ {
		if err := self.Publish(context.Background(), cluster.Outbound{
			Room: room, Kind: cluster.KindSync, Data: []byte(fmt.Sprintf("self-%d", i)),
		}); err != nil {
			t.Fatalf("Publish self: %v", err)
		}
	}
	eventually(t, 5*time.Second, "self writes to persist", func() bool {
		return rowCount(t, pool, room) == 200
	})

	// Then one remote update, which self must still deliver.
	if err := peer.Publish(context.Background(), cluster.Outbound{
		Room: room, Kind: cluster.KindSync, Data: []byte("from-peer"),
	}); err != nil {
		t.Fatalf("Publish peer: %v", err)
	}

	eventually(t, 5*time.Second, "remote update after a self burst", func() bool {
		for _, p := range selfSink.payloads() {
			if string(p) == "from-peer" {
				return true
			}
		}
		return false
	})
}

// TestCatchUpReplaysHistory: a room activated after rows already exist must replay
// them, or a reconnecting client silently loses every edit made while it was away.
func TestCatchUpReplaysHistory(t *testing.T) {
	pool := newPool(t)

	writer := startRelay(t, pool, &fakeSink{}, nil)
	writer.RoomActivated(room)
	for i := 0; i < 3; i++ {
		if err := writer.Publish(context.Background(), cluster.Outbound{
			Room: room, Kind: cluster.KindSync, Data: []byte{byte(i)},
		}); err != nil {
			t.Fatalf("Publish: %v", err)
		}
	}
	eventually(t, 3*time.Second, "history to persist", func() bool {
		return rowCount(t, pool, room) == 3
	})

	// A second instance joins only now.
	lateSink := &fakeSink{}
	late := startRelay(t, pool, lateSink, nil)
	late.RoomActivated(room)

	eventually(t, 3*time.Second, "history replay", func() bool {
		return lateSink.count() == 3
	})
}

// TestAwarenessRoundTrips: awareness rides the same table with a different kind, so
// a mis-encoded kind would deliver presence data as a document update.
func TestAwarenessRoundTrips(t *testing.T) {
	pool := newPool(t)

	readerSink := &fakeSink{}
	writer := startRelay(t, pool, &fakeSink{}, nil)
	reader := startRelay(t, pool, readerSink, nil)
	writer.RoomActivated(room)
	reader.RoomActivated(room)

	if err := writer.Publish(context.Background(), cluster.Outbound{
		Room: room, Kind: cluster.KindAwareness, Data: []byte("cursor"),
	}); err != nil {
		t.Fatalf("Publish: %v", err)
	}

	eventually(t, 3*time.Second, "awareness delivery", func() bool {
		return readerSink.count() == 1
	})
	readerSink.mu.Lock()
	kind := readerSink.got[0].Kind
	readerSink.mu.Unlock()
	if kind != cluster.KindAwareness {
		t.Errorf("kind = %v, want KindAwareness", kind)
	}
}

// TestPublishAfterCloseIsRefused: the cluster.Relay contract requires Publish to
// return ErrRelayClosed after Close, so the provider can distinguish a shut-down
// relay from a dropped update.
func TestPublishAfterCloseIsRefused(t *testing.T) {
	pool := newPool(t)
	r := startRelay(t, pool, &fakeSink{}, nil)
	r.RoomActivated(room)
	if err := r.Close(); err != nil {
		t.Fatalf("Close: %v", err)
	}
	err := r.Publish(context.Background(), cluster.Outbound{Room: room, Data: []byte("x")})
	if !errors.Is(err, ErrRelayClosed) {
		t.Errorf("Publish after Close = %v, want ErrRelayClosed", err)
	}
}

// TestPublishForUnknownRoomIsNoOp: the contract allows a Publish to arrive after
// RoomDeactivated (the provider's final drain runs asynchronously), and it must be
// dropped quietly rather than reviving rows for an evicted document.
func TestPublishForUnknownRoomIsNoOp(t *testing.T) {
	pool := newPool(t)
	r := startRelay(t, pool, &fakeSink{}, nil)

	if err := r.Publish(context.Background(), cluster.Outbound{
		Room: room, Kind: cluster.KindSync, Data: []byte("orphan"),
	}); err != nil {
		t.Errorf("Publish for an inactive room = %v, want nil", err)
	}
	time.Sleep(50 * time.Millisecond)
	if n := rowCount(t, pool, room); n != 0 {
		t.Errorf("wrote %d rows for an inactive room, want 0", n)
	}
}

// TestRoomActivatedIsIdempotent: ygo may re-activate a resident room, and a second
// set of goroutines per activation would double every write and leak on shutdown.
func TestRoomActivatedIsIdempotent(t *testing.T) {
	pool := newPool(t)

	peerSink := &fakeSink{}
	writer := startRelay(t, pool, &fakeSink{}, nil)
	peer := startRelay(t, pool, peerSink, nil)

	writer.RoomActivated(room)
	writer.RoomActivated(room)
	writer.RoomActivated(room)
	peer.RoomActivated(room)

	if err := writer.Publish(context.Background(), cluster.Outbound{
		Room: room, Kind: cluster.KindSync, Data: []byte("once"),
	}); err != nil {
		t.Fatalf("Publish: %v", err)
	}

	eventually(t, 3*time.Second, "the update to arrive", func() bool {
		return peerSink.count() >= 1
	})
	time.Sleep(100 * time.Millisecond)
	if n := rowCount(t, pool, room); n != 1 {
		t.Errorf("wrote %d rows after 3 activations, want 1", n)
	}
	if n := peerSink.count(); n != 1 {
		t.Errorf("delivered %d copies, want 1", n)
	}
}
