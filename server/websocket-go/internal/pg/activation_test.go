package pg

import (
	"context"
	"errors"
	"sync/atomic"
	"testing"
	"time"

	"github.com/reearth/ygo/cluster"
)

// TestRoomStaysActiveUntilEveryActivationEnds: ygo can activate a room again
// before deactivating the room it replaces. The first deactivation must leave
// the room live for the second.
func TestRoomStaysActiveUntilEveryActivationEnds(t *testing.T) {
	pool := newPool(t)
	ctx := context.Background()
	fl := &countingFlusher{}
	r := startRelay(t, pool, &fakeSink{}, fl)

	r.RoomActivated(room)
	r.RoomActivated(room)
	r.RoomDeactivated(room)

	if err := r.WaitCaughtUp(ctx, room); err != nil {
		t.Fatalf("room not active after one of two deactivations: %v", err)
	}
	if n, _ := r.activeInstances(ctx, room); n != 1 {
		t.Fatalf("heartbeats = %d, want 1", n)
	}
	if err := r.Publish(ctx, cluster.Outbound{Room: room, Kind: cluster.KindSync, Data: []byte("kept")}); err != nil {
		t.Fatal(err)
	}
	eventually(t, 3*time.Second, "the update to be written", func() bool { return rowCount(t, pool, room) >= 1 })

	r.RoomDeactivated(room)
	if err := r.WaitCaughtUp(ctx, room); !errors.Is(err, ErrRoomNotActive) {
		t.Fatalf("room still active after the last deactivation: %v", err)
	}
	if n, _ := r.activeInstances(ctx, room); n != 0 {
		t.Fatalf("heartbeats after the last deactivation = %d, want 0", n)
	}
	if fl.count() != 1 {
		t.Fatalf("last-instance flushes = %d, want 1", fl.count())
	}
}

// reactivatingSink re-activates the room from inside Inject once the relay has
// started evicting it, as ygo does when a relay entry re-creates a room: the call
// runs on the evicting state's reader goroutine, which the eviction waits for.
type reactivatingSink struct {
	fakeSink
	relay   *Relay
	trigger string
	fired   atomic.Bool
	done    chan struct{}
}

func (s *reactivatingSink) Inject(ctx context.Context, in cluster.Inbound) error {
	_ = s.fakeSink.Inject(ctx, in)
	if string(in.Data) != s.trigger || !s.fired.CompareAndSwap(false, true) {
		return nil
	}
	for !s.evicting(in.Room) {
		time.Sleep(time.Millisecond)
	}
	s.relay.RoomActivated(in.Room)
	close(s.done)
	return nil
}

func (s *reactivatingSink) evicting(room string) bool {
	s.relay.mu.Lock()
	defer s.relay.mu.Unlock()
	rs, ok := s.relay.rooms[room]
	return ok && rs.evicting
}

// TestActivationDuringEvictionKeepsTheInstanceLive: a room re-activated while its
// previous state is evicting must end with this instance's heartbeat in place and
// the rows kept, and the eviction must not deadlock.
func TestActivationDuringEvictionKeepsTheInstanceLive(t *testing.T) {
	pool := newPool(t)
	ctx := context.Background()
	fl := &countingFlusher{}
	sink := &reactivatingSink{trigger: "reactivate", done: make(chan struct{})}
	r := startRelay(t, pool, sink, fl)
	sink.relay = r
	r.RoomActivated(room)

	// Another instance writes the entry the reader is injecting when the eviction
	// starts, then lets the room go so this instance holds it alone.
	peer := startRelay(t, pool, &fakeSink{}, nil)
	peer.RoomActivated(room)
	if err := peer.Publish(ctx, cluster.Outbound{Room: room, Kind: cluster.KindSync, Data: []byte(sink.trigger)}); err != nil {
		t.Fatal(err)
	}
	eventually(t, 3*time.Second, "the trigger to be injected", func() bool { return sink.fired.Load() })
	peer.RoomDeactivated(room)

	deactivated := make(chan struct{})
	go func() {
		r.RoomDeactivated(room)
		close(deactivated)
	}()
	select {
	case <-deactivated:
	case <-time.After(5 * time.Second):
		t.Fatal("RoomDeactivated deadlocked on a re-activation from its own reader")
	}
	<-sink.done

	if err := r.WaitCaughtUp(ctx, room); err != nil {
		t.Fatalf("successor not active: %v", err)
	}
	if n, _ := r.activeInstances(ctx, room); n != 1 {
		t.Fatalf("heartbeats = %d, want 1", n)
	}
	if rowCount(t, pool, room) == 0 {
		t.Fatal("the rows were deleted under the successor")
	}
	if fl.count() != 0 {
		t.Fatalf("last-instance flushes = %d, want 0", fl.count())
	}
}

// TestActivationOntoALiveStateReplaysTheRows: a room activated onto a state that
// is already live was loaded from storage, which can lack rows the reader applied
// to the room it replaces, so the reader reads the rows again.
func TestActivationOntoALiveStateReplaysTheRows(t *testing.T) {
	pool := newPool(t)
	ctx := context.Background()
	sink := &fakeSink{}
	r := startRelay(t, pool, sink, nil)
	peer := startRelay(t, pool, &fakeSink{}, nil)
	peer.RoomActivated(room)
	if err := peer.Publish(ctx, cluster.Outbound{Room: room, Kind: cluster.KindSync, Data: []byte("x")}); err != nil {
		t.Fatal(err)
	}
	eventually(t, 3*time.Second, "the row", func() bool { return rowCount(t, pool, room) == 1 })

	r.RoomActivated(room)
	eventually(t, 3*time.Second, "the first read", func() bool { return sink.count() == 1 })
	r.RoomActivated(room)
	eventually(t, 3*time.Second, "the replay", func() bool { return sink.count() == 2 })
	if got := sink.payloads()[1]; string(got) != "x" {
		t.Fatalf("replayed %q, want %q", got, "x")
	}
}
