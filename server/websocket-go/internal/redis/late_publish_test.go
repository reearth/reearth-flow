package redis

import (
	"context"
	"testing"
	"time"

	"github.com/reearth/ygo/cluster"
)

func startedRelay(t *testing.T, ctx context.Context, addr string, sink *fakeSink) *Relay {
	t.Helper()
	r := mustRelay(t, addr)
	t.Cleanup(func() { _ = r.Close() })
	if err := r.Start(ctx, sink); err != nil {
		t.Fatalf("Start: %v", err)
	}
	return r
}

func injected(s *fakeSink, data []byte) bool {
	for _, in := range s.snapshot() {
		if in.Kind == cluster.KindSync && string(in.Data) == string(data) {
			return true
		}
	}
	return false
}

// TestPublishJustBeforeDeactivateReachesHolders: an update published right
// before its room is deactivated here still reaches an instance holding the
// room. ygo publishes a room's last edits just before it deactivates it, and the
// writer used to lose a batch it had taken off the queue when the deactivation
// cancelled its context.
func TestPublishJustBeforeDeactivateReachesHolders(t *testing.T) {
	_, mr := newTestClient(t)
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	sinkB := &fakeSink{}
	relayB := startedRelay(t, ctx, mr.Addr(), sinkB)

	for i := byte(1); i <= 20; i++ {
		room := "proj-" + string('a'+rune(i))
		relayA := startedRelay(t, ctx, mr.Addr(), &fakeSink{})
		relayA.RoomActivated(room)
		relayB.RoomActivated(room)
		data := []byte{i, i, i}
		if err := relayA.Publish(ctx, cluster.Outbound{Room: room, Kind: cluster.KindSync, Data: data}); err != nil {
			t.Fatalf("Publish: %v", err)
		}
		relayA.RoomDeactivated(room)
		waitFor(t, 3*time.Second, func() bool { return injected(sinkB, data) })
	}
}

// TestPublishRacingDeactivateReachesHolders: an update Publish accepts while
// RoomDeactivated is starting must be written, not left in a queue whose writer
// has already made its final drain.
func TestPublishRacingDeactivateReachesHolders(t *testing.T) {
	_, mr := newTestClient(t)
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	const room = "proj1"
	sinkB := &fakeSink{}
	relayA := startedRelay(t, ctx, mr.Addr(), &fakeSink{})
	relayB := startedRelay(t, ctx, mr.Addr(), sinkB)
	relayA.RoomActivated(room)
	relayB.RoomActivated(room)

	deactivated := make(chan struct{})
	relayA.beforeEnqueue = func() {
		go func() {
			relayA.RoomDeactivated(room)
			close(deactivated)
		}()
		time.Sleep(100 * time.Millisecond) // let the deactivation run as far as it can
	}
	data := []byte{9, 9, 9}
	if err := relayA.Publish(ctx, cluster.Outbound{Room: room, Kind: cluster.KindSync, Data: data}); err != nil {
		t.Fatalf("Publish: %v", err)
	}
	<-deactivated
	waitFor(t, 3*time.Second, func() bool { return injected(sinkB, data) })
}

// TestPublishAfterDeactivateReachesHolders: ygo can hand a room's last update
// to Publish after RoomDeactivated. It is written late, so an instance still
// holding the room receives it.
func TestPublishAfterDeactivateReachesHolders(t *testing.T) {
	_, mr := newTestClient(t)
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	const room = "proj1"
	sinkB := &fakeSink{}
	relayA := startedRelay(t, ctx, mr.Addr(), &fakeSink{})
	relayB := startedRelay(t, ctx, mr.Addr(), sinkB)
	relayA.RoomActivated(room)
	relayB.RoomActivated(room)

	relayA.RoomDeactivated(room)
	data := []byte{4, 5, 6}
	if err := relayA.Publish(ctx, cluster.Outbound{Room: room, Kind: cluster.KindSync, Data: data}); err != nil {
		t.Fatalf("Publish: %v", err)
	}
	waitFor(t, 3*time.Second, func() bool { return injected(sinkB, data) })
	if got := relayA.LateWrites(); got != 1 {
		t.Fatalf("LateWrites = %d, want 1", got)
	}
}

// TestPublishAfterLastDeactivateLeavesNoStream: once no instance holds the room
// its stream is gone, and a late update must not create it again: the update is
// already persisted.
func TestPublishAfterLastDeactivateLeavesNoStream(t *testing.T) {
	_, mr := newTestClient(t)
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	const room = "proj1"
	relayA := startedRelay(t, ctx, mr.Addr(), &fakeSink{})
	relayA.RoomActivated(room)
	relayA.RoomDeactivated(room)
	if mr.Exists(streamKey(room)) {
		t.Fatal("the last instance left the stream in place")
	}

	if err := relayA.Publish(ctx, cluster.Outbound{Room: room, Kind: cluster.KindSync, Data: []byte{1}}); err != nil {
		t.Fatalf("Publish: %v", err)
	}
	if mr.Exists(streamKey(room)) {
		t.Fatal("a late update created the stream of a room no instance holds")
	}
	if got := relayA.DroppedWrites(); got != 0 {
		t.Fatalf("DroppedWrites = %d, want 0: nothing is lost", got)
	}
}
