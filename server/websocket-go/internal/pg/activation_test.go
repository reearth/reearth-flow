package pg

import (
	"context"
	"errors"
	"sync/atomic"
	"testing"
	"time"

	"github.com/jackc/pgx/v5/pgxpool"
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
	// abort ends the wait for the eviction when the test ends first.
	abort chan struct{}
	done  chan struct{}
	// err is what went wrong in Inject; read it after done.
	err error
}

func newReactivatingSink(trigger string) *reactivatingSink {
	return &reactivatingSink{trigger: trigger, abort: make(chan struct{}), done: make(chan struct{})}
}

// attach binds the sink to relay. Call it after relay's Close is registered with
// t.Cleanup, so the abort runs before Close waits for the reader.
func (s *reactivatingSink) attach(t *testing.T, relay *Relay) {
	s.relay = relay
	t.Cleanup(func() { close(s.abort) })
}

func (s *reactivatingSink) Inject(ctx context.Context, in cluster.Inbound) error {
	_ = s.fakeSink.Inject(ctx, in)
	if string(in.Data) != s.trigger || !s.fired.CompareAndSwap(false, true) {
		return nil
	}
	defer close(s.done)
	deadline := time.After(5 * time.Second)
	for !s.evicting(in.Room) {
		select {
		case <-s.abort:
			s.err = errors.New("test ended before the eviction started")
			return nil
		case <-deadline:
			s.err = errors.New("the eviction never started")
			return nil
		case <-time.After(time.Millisecond):
		}
	}
	s.relay.RoomActivated(in.Room)
	return nil
}

func (s *reactivatingSink) evicting(room string) bool {
	s.relay.mu.Lock()
	defer s.relay.mu.Unlock()
	rs, ok := s.relay.rooms[room]
	return ok && rs.evicting
}

// evictWithSuccessor activates room on r, has a peer write the trigger and leave,
// then deactivates the room on r while its reader injects the trigger, during
// which the sink re-activates it. It returns once the first eviction finished.
func evictWithSuccessor(t *testing.T, pool *pgxpool.Pool, r *Relay, sink *reactivatingSink) {
	t.Helper()
	ctx := context.Background()
	r.RoomActivated(room)
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
	select {
	case <-sink.done:
	case <-time.After(10 * time.Second):
		t.Fatal("the sink never re-activated the room")
	}
	if sink.err != nil {
		t.Fatalf("re-activation: %v", sink.err)
	}
}

// TestActivationDuringEvictionKeepsTheInstanceLive: a room re-activated while its
// previous state is evicting must end with this instance's heartbeat in place and
// the rows kept, and the eviction must not deadlock.
func TestActivationDuringEvictionKeepsTheInstanceLive(t *testing.T) {
	pool := newPool(t)
	ctx := context.Background()
	fl := &countingFlusher{}
	sink := newReactivatingSink("reactivate")
	r := startRelay(t, pool, sink, fl)
	sink.attach(t, r)
	evictWithSuccessor(t, pool, r, sink)

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

// TestSuccessorLeavingDuringTheRestoreLeavesNoHeartbeat: the state that replaced
// an evicting one leaves while the older eviction is restoring its heartbeat. The
// last departure must remove the instance's heartbeat.
func TestSuccessorLeavingDuringTheRestoreLeavesNoHeartbeat(t *testing.T) {
	pool := newPool(t)
	ctx := context.Background()
	sink := newReactivatingSink("reactivate")
	r := startRelay(t, pool, sink, &countingFlusher{})
	sink.attach(t, r)
	successorLeft := make(chan struct{})
	var once atomic.Bool
	r.beforeRestore = func() {
		if !once.CompareAndSwap(false, true) {
			return
		}
		go func() {
			r.RoomDeactivated(room)
			close(successorLeft)
		}()
		// Give the successor's departure the chance to finish first.
		select {
		case <-successorLeft:
		case <-time.After(time.Second):
		}
	}
	evictWithSuccessor(t, pool, r, sink)
	select {
	case <-successorLeft:
	case <-time.After(10 * time.Second):
		t.Fatal("the successor's departure never finished")
	}
	if n, _ := r.activeInstances(ctx, room); n != 0 {
		t.Fatalf("heartbeats after both left = %d, want 0", n)
	}
}

// TestWaitCaughtUpWaitsForTheReplay: a second activation onto a live state whose
// cursor is at the head must not count as caught up until its replay is done.
func TestWaitCaughtUpWaitsForTheReplay(t *testing.T) {
	pool := newPool(t)
	ctx := context.Background()
	r := startRelay(t, pool, &fakeSink{}, nil)
	peer := startRelay(t, pool, &fakeSink{}, nil)
	peer.RoomActivated(room)
	if err := peer.Publish(ctx, cluster.Outbound{Room: room, Kind: cluster.KindSync, Data: []byte("x")}); err != nil {
		t.Fatal(err)
	}
	eventually(t, 3*time.Second, "the row", func() bool { return rowCount(t, pool, room) == 1 })
	r.RoomActivated(room)
	if err := r.WaitCaughtUp(ctx, room); err != nil {
		t.Fatal(err)
	}

	// Hold the reader before it starts the replay the next activation asks for.
	entered, release := make(chan struct{}), make(chan struct{})
	var held, released atomic.Bool
	releaseOnce := func() {
		if released.CompareAndSwap(false, true) {
			close(release)
		}
	}
	// Registered after startRelay's cleanup, so it runs first: Close waits for the
	// held reader.
	t.Cleanup(releaseOnce)
	hold := func() {
		if held.CompareAndSwap(false, true) {
			close(entered)
			<-release
		}
	}
	r.beforePass.Store(&hold)
	select {
	case <-entered:
	case <-time.After(5 * time.Second):
		t.Fatal("the reader never started another pass")
	}
	r.RoomActivated(room)
	waited := make(chan error, 1)
	go func() { waited <- r.WaitCaughtUp(ctx, room) }()
	select {
	case err := <-waited:
		t.Fatalf("WaitCaughtUp returned %v before the replay ran", err)
	case <-time.After(200 * time.Millisecond):
	}
	releaseOnce()
	select {
	case err := <-waited:
		if err != nil {
			t.Fatalf("WaitCaughtUp after the replay: %v", err)
		}
	case <-time.After(5 * time.Second):
		t.Fatal("WaitCaughtUp did not return after the replay")
	}
}
