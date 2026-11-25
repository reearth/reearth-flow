package redis

import (
	"context"
	"errors"
	"strconv"
	"sync/atomic"
	"testing"
	"time"

	goredis "github.com/redis/go-redis/v9"
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
	// abort ends the wait for the eviction when the test ends first.
	abort chan struct{}
	done  chan struct{}
	// err is what went wrong in Inject; read it after done.
	err error
}

func newReactivatingSink(trigger byte) *reactivatingSink {
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
	if len(in.Data) != 1 || in.Data[0] != s.trigger || !s.fired.CompareAndSwap(false, true) {
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
	// Let the successor catch up so it has a cursor to advertise.
	wctx, cancel := context.WithTimeout(context.WithoutCancel(ctx), 3*time.Second)
	defer cancel()
	s.err = s.relay.WaitCaughtUp(wctx, in.Room)
	return nil
}

func (s *reactivatingSink) evicting(room string) bool {
	s.relay.mu.Lock()
	defer s.relay.mu.Unlock()
	rs, ok := s.relay.rooms[room]
	return ok && rs.evicting
}

// waitDone waits for the sink's re-activation and requires it to have worked.
func (s *reactivatingSink) waitDone(t *testing.T) {
	t.Helper()
	select {
	case <-s.done:
	case <-time.After(10 * time.Second):
		t.Fatal("the sink never re-activated the room")
	}
	if s.err != nil {
		t.Fatalf("re-activation: %v", s.err)
	}
}

// startEvictionWithSuccessor activates room, then deactivates it while the
// reader is injecting an entry, during which the sink re-activates the room.
// It returns once the first eviction has finished.
func startEvictionWithSuccessor(t *testing.T, ctx context.Context, c *goredis.Client, relay *Relay, sink *reactivatingSink, room string) {
	t.Helper()
	relay.RoomActivated(room)
	if err := relay.WaitCaughtUp(ctx, room); err != nil {
		t.Fatal(err)
	}
	if err := xaddEntry(ctx, c, streamKey(room), msgTypeSync, []byte{sink.trigger}, 808080); err != nil {
		t.Fatal(err)
	}
	waitFor(t, 3*time.Second, func() bool { return sink.fired.Load() })

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
	sink.waitDone(t)
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
	t.Cleanup(func() { _ = relay.Close() })
	relay.flusher = flusher
	sink := newReactivatingSink(9)
	sink.attach(t, relay)
	if err := relay.Start(ctx, sink); err != nil {
		t.Fatal(err)
	}
	startEvictionWithSuccessor(t, ctx, c, relay, sink, room)

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

// TestSuccessorLeavingDuringTheRestoreLeavesNoHeartbeat: the state that replaced
// an evicting one leaves while the older eviction is restoring its heartbeat. The
// last departure must remove the instance's heartbeat and cursor.
func TestSuccessorLeavingDuringTheRestoreLeavesNoHeartbeat(t *testing.T) {
	c, mr := newTestClient(t)
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	const room = "proj1"

	relay := mustRelay(t, mr.Addr())
	t.Cleanup(func() { _ = relay.Close() })
	sink := newReactivatingSink(9)
	sink.attach(t, relay)
	if err := relay.Start(ctx, sink); err != nil {
		t.Fatal(err)
	}
	successorLeft := make(chan struct{})
	var once atomic.Bool
	relay.beforeRestore = func() {
		if !once.CompareAndSwap(false, true) {
			return
		}
		go func() {
			relay.RoomDeactivated(room)
			close(successorLeft)
		}()
		// Give the successor's departure, whose reader may sit in a blocking
		// XREAD, the chance to finish first.
		select {
		case <-successorLeft:
		case <-time.After(2 * readBlock):
		}
	}
	startEvictionWithSuccessor(t, ctx, c, relay, sink, room)
	select {
	case <-successorLeft:
	case <-time.After(10 * time.Second):
		t.Fatal("the successor's departure never finished")
	}

	if n, _ := getActiveInstances(ctx, c, room, activeTimeoutSecs); n != 0 {
		t.Fatalf("heartbeats after both left = %d, want 0", n)
	}
	if ok, _ := c.HExists(ctx, cursorsKey(room), strconv.FormatUint(relay.clientID, 10)).Result(); ok {
		t.Fatal("a cursor is left for a room the instance no longer holds")
	}
}

// gatedSink blocks the injection of one payload while gated, so a test can hold
// a replay in progress.
type gatedSink struct {
	fakeSink
	payload byte
	gated   atomic.Bool
	entered chan struct{}
	release chan struct{}
	once    atomic.Bool
}

func (s *gatedSink) Inject(ctx context.Context, in cluster.Inbound) error {
	_ = s.fakeSink.Inject(ctx, in)
	if s.gated.Load() && len(in.Data) == 1 && in.Data[0] == s.payload && s.once.CompareAndSwap(false, true) {
		close(s.entered)
		<-s.release
	}
	return nil
}

// TestWaitCaughtUpWaitsForTheReplay: a second activation onto a live state whose
// cursor is at the head must not count as caught up until its replay is done.
func TestWaitCaughtUpWaitsForTheReplay(t *testing.T) {
	c, mr := newTestClient(t)
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	const room = "proj1"

	sink := &gatedSink{payload: 5, entered: make(chan struct{}), release: make(chan struct{})}
	var released atomic.Bool
	releaseOnce := func() {
		if released.CompareAndSwap(false, true) {
			close(sink.release)
		}
	}
	relay := mustRelay(t, mr.Addr())
	defer func() { _ = relay.Close() }()
	defer releaseOnce() // before Close, which waits for the held reader
	if err := relay.Start(ctx, sink); err != nil {
		t.Fatal(err)
	}
	if err := xaddEntry(ctx, c, streamKey(room), msgTypeSync, []byte{5}, 808080); err != nil {
		t.Fatal(err)
	}
	relay.RoomActivated(room)
	if err := relay.WaitCaughtUp(ctx, room); err != nil {
		t.Fatal(err)
	}

	sink.gated.Store(true)
	relay.RoomActivated(room)
	waited := make(chan error, 1)
	go func() { waited <- relay.WaitCaughtUp(ctx, room) }()
	select {
	case <-sink.entered:
	case <-time.After(5 * time.Second):
		t.Fatal("the replay never started")
	}
	select {
	case err := <-waited:
		t.Fatalf("WaitCaughtUp returned %v while the replay was held", err)
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
