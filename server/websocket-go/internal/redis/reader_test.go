package redis

import (
	"context"
	"os"
	"testing"
	"time"

	"github.com/reearth/ygo/cluster"
)

// TestCatchUpDrainOrdering: every sibling entry present at activation is injected
// during catch-up, in stream order, before the live reader starts.
func TestCatchUpDrainOrdering(t *testing.T) {
	c, mr := newTestClient(t)
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	const room = "proj1"

	// Three remote entries, in order, present before activation.
	for _, b := range [][]byte{{1}, {2}, {3}} {
		if err := xaddEntry(ctx, c, streamKey(room), msgTypeSync, b, 700700); err != nil {
			t.Fatalf("seed: %v", err)
		}
	}

	sink := &fakeSink{}
	relay := mustRelay(t, mr.Addr())
	defer relay.Close()
	_ = relay.Start(ctx, sink)

	// Catch-up runs on the reader goroutine (deferred off the activation callback
	// to avoid the ygo#133 re-entrant deadlock), so wait for the replay to land.
	relay.RoomActivated(room)
	waitFor(t, 3*time.Second, func() bool { return sink.count() >= 3 })

	got := sink.snapshot()
	for i, want := range [][]byte{{1}, {2}, {3}} {
		if string(got[i].Data) != string(want) {
			t.Fatalf("entry[%d] = %v, want %v (drain order broken)", i, got[i].Data, want)
		}
	}
}

// TestLiveReaderXReadBlock exercises the live XREAD BLOCK loop against real Redis,
// which miniredis does not faithfully model. Skipped unless REDIS_ADDR is set.
func TestLiveReaderXReadBlock(t *testing.T) {
	addr := os.Getenv("REDIS_ADDR")
	if addr == "" {
		t.Skip("REDIS_ADDR not set; skipping real-Redis XREAD BLOCK test")
	}
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	room := "relay-itest-" + time.Now().Format("150405.000")

	sink := &fakeSink{}
	writer := mustRelay(t, addr)
	reader := mustRelay(t, addr)
	defer writer.Close()
	defer reader.Close()
	defer func() { _ = reader.ForceEvict(context.Background(), room) }()

	_ = writer.Start(ctx, &fakeSink{})
	_ = reader.Start(ctx, sink)
	writer.RoomActivated(room)
	reader.RoomActivated(room)

	// Publish after the reader is blocked in XREAD; it must unblock and inject.
	time.Sleep(200 * time.Millisecond)
	if err := writer.Publish(ctx, cluster.Outbound{Room: room, Kind: cluster.KindSync, Data: []byte{0xab, 0xcd}}); err != nil {
		t.Fatalf("publish: %v", err)
	}
	waitFor(t, 5*time.Second, func() bool {
		for _, in := range sink.snapshot() {
			if string(in.Data) == string([]byte{0xab, 0xcd}) {
				return true
			}
		}
		return false
	})
}
