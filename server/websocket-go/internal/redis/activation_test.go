package redis

import (
	"context"
	"errors"
	"strconv"
	"sync/atomic"
	"testing"
	"time"

	"github.com/reearth/ygo/cluster"
)

// TestRoomStaysActiveUntilEveryActivationEnds: ygo can activate a room again
// before deactivating the room it replaces. The first deactivation must leave
// the room live for the second.
func TestRoomStaysActiveUntilEveryActivationEnds(t *testing.T) {
	c, mr := newTestClient(t)
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	const room = "proj1"

	flusher := &fakeFlusher{}
	relay := mustRelay(t, mr.Addr())
	defer func() { _ = relay.Close() }()
	relay.flusher = flusher
	if err := relay.Start(ctx, &fakeSink{}); err != nil {
		t.Fatal(err)
	}
	relay.RoomActivated(room)
	relay.RoomActivated(room)
	relay.RoomDeactivated(room)

	if err := relay.WaitCaughtUp(ctx, room); err != nil {
		t.Fatalf("room not active after one of two deactivations: %v", err)
	}
	if n, _ := getActiveInstances(ctx, c, room, activeTimeoutSecs); n != 1 {
		t.Fatalf("heartbeats = %d, want 1", n)
	}
	if err := relay.Publish(ctx, cluster.Outbound{Room: room, Kind: cluster.KindSync, Data: []byte{7}}); err != nil {
		t.Fatal(err)
	}
	waitFor(t, 3*time.Second, func() bool {
		msgs, _ := c.XRange(ctx, streamKey(room), "-", "+").Result()
		for _, m := range msgs {
			if e := parseEntry(m, 0); e.kind == kindSync && len(e.data) == 1 && e.data[0] == 7 {
				return true
			}
		}
		return false
	})

	relay.RoomDeactivated(room)
	if err := relay.WaitCaughtUp(ctx, room); !errors.Is(err, ErrRoomNotActive) {
		t.Fatalf("room still active after the last deactivation: %v", err)
	}
	if n, _ := getActiveInstances(ctx, c, room, activeTimeoutSecs); n != 0 {
		t.Fatalf("heartbeats after the last deactivation = %d, want 0", n)
	}
	if got := len(flusher.calls()); got != 1 {
		t.Fatalf("last-instance flushes = %d, want 1", got)
	}
}

// reactivatingSink re-activates the room from inside Inject once the relay has
// started evicting it, as ygo does when a relay entry re-creates a room: the call
// runs on the evicting state's reader goroutine, which the eviction waits for.
type reactivatingSink struct {
	fakeSink
	relay   *Relay
	trigger byte
	fired   atomic.Bool
	done    chan struct{}
}

func (s *reactivatingSink) Inject(ctx context.Context, in cluster.Inbound) error {
	_ = s.fakeSink.Inject(ctx, in)
	if len(in.Data) != 1 || in.Data[0] != s.trigger || !s.fired.CompareAndSwap(false, true) {
		return nil
	}
	for !s.evicting(in.Room) {
		time.Sleep(time.Millisecond)
	}
	s.relay.RoomActivated(in.Room)
	// Let the successor catch up so it has a cursor to advertise.
	wctx, cancel := context.WithTimeout(context.WithoutCancel(ctx), 3*time.Second)
	defer cancel()
	_ = s.relay.WaitCaughtUp(wctx, in.Room)
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
// previous state is evicting must end with this instance's heartbeat and cursor
// in place and the stream kept, and the eviction must not deadlock.
func TestActivationDuringEvictionKeepsTheInstanceLive(t *testing.T) {
	c, mr := newTestClient(t)
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	const room = "proj1"

	flusher := &fakeFlusher{}
	relay := mustRelay(t, mr.Addr())
	defer func() { _ = relay.Close() }()
	relay.flusher = flusher
	sink := &reactivatingSink{relay: relay, trigger: 9, done: make(chan struct{})}
	if err := relay.Start(ctx, sink); err != nil {
		t.Fatal(err)
	}
	relay.RoomActivated(room)
	if err := relay.WaitCaughtUp(ctx, room); err != nil {
		t.Fatal(err)
	}
	// A remote entry the reader is injecting when the eviction starts.
	if err := xaddEntry(ctx, c, streamKey(room), msgTypeSync, []byte{9}, 808080); err != nil {
		t.Fatal(err)
	}
	waitFor(t, 3*time.Second, func() bool {
		for _, in := range sink.snapshot() {
			if len(in.Data) == 1 && in.Data[0] == 9 {
				return true
			}
		}
		return false
	})

	deactivated := make(chan struct{})
	go func() {
		relay.RoomDeactivated(room)
		close(deactivated)
	}()
	select {
	case <-deactivated:
	case <-time.After(5 * time.Second):
		t.Fatal("RoomDeactivated deadlocked on a re-activation from its own reader")
	}
	<-sink.done

	if err := relay.WaitCaughtUp(ctx, room); err != nil {
		t.Fatalf("successor not active: %v", err)
	}
	if n, _ := getActiveInstances(ctx, c, room, activeTimeoutSecs); n != 1 {
		t.Fatalf("heartbeats = %d, want 1", n)
	}
	if ok, _ := c.HExists(ctx, cursorsKey(room), strconv.FormatUint(relay.clientID, 10)).Result(); !ok {
		t.Fatal("the successor's cursor was dropped")
	}
	if n, _ := c.Exists(ctx, streamKey(room)).Result(); n != 1 {
		t.Fatal("the stream was deleted under the successor")
	}
	if got := len(flusher.calls()); got != 0 {
		t.Fatalf("last-instance flushes = %d, want 0", got)
	}
}

// TestActivationOntoALiveStateReplaysTheStream: a room activated onto a state
// that is already live was loaded from storage, which can lack entries the
// reader applied to the room it replaces, so the reader replays the stream.
func TestActivationOntoALiveStateReplaysTheStream(t *testing.T) {
	c, mr := newTestClient(t)
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	const room = "proj1"

	sink := &fakeSink{}
	relay := mustRelay(t, mr.Addr())
	defer func() { _ = relay.Close() }()
	if err := relay.Start(ctx, sink); err != nil {
		t.Fatal(err)
	}
	if err := xaddEntry(ctx, c, streamKey(room), msgTypeSync, []byte{5}, 808080); err != nil {
		t.Fatal(err)
	}
	relay.RoomActivated(room)
	waitFor(t, 3*time.Second, func() bool { return sink.count() == 1 })

	relay.RoomActivated(room)
	waitFor(t, 3*time.Second, func() bool { return sink.count() == 2 })
	if got := sink.snapshot()[1].Data; len(got) != 1 || got[0] != 5 {
		t.Fatalf("replayed %v, want [5]", got)
	}
}
