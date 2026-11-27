package redis

import (
	"context"
	"errors"
	"fmt"
	"strings"
	"sync"
	"testing"
	"time"

	goredis "github.com/redis/go-redis/v9"
	"github.com/reearth/ygo/awareness"
	"github.com/reearth/ygo/cluster"
	"github.com/reearth/ygo/crdt"
	ygows "github.com/reearth/ygo/provider/websocket"
)

// failXAdd fails pipelines that carry an XADD: the next left of them, or all of
// them when left is negative.
type failXAdd struct {
	mu     sync.Mutex
	left   int
	failed int
}

func (f *failXAdd) DialHook(next goredis.DialHook) goredis.DialHook { return next }

func (f *failXAdd) ProcessHook(next goredis.ProcessHook) goredis.ProcessHook { return next }

func (f *failXAdd) ProcessPipelineHook(next goredis.ProcessPipelineHook) goredis.ProcessPipelineHook {
	return func(ctx context.Context, cmds []goredis.Cmder) error {
		if hasXAdd(cmds...) {
			f.mu.Lock()
			fail := f.left != 0
			if f.left > 0 {
				f.left--
			}
			if fail {
				f.failed++
			}
			f.mu.Unlock()
			if fail {
				return errors.New("injected XADD failure")
			}
		}
		return next(ctx, cmds)
	}
}

func (f *failXAdd) failures() int {
	f.mu.Lock()
	defer f.mu.Unlock()
	return f.failed
}

// scriptedSink fails an inject of a given payload with the errors queued for it,
// one per attempt, and records what it applied and how often each was tried.
type scriptedSink struct {
	mu       sync.Mutex
	failWith map[string][]error
	attempts map[string]int
	applied  []string
}

func newScriptedSink() *scriptedSink {
	return &scriptedSink{failWith: map[string][]error{}, attempts: map[string]int{}}
}

func (s *scriptedSink) Inject(_ context.Context, in cluster.Inbound) error {
	s.mu.Lock()
	defer s.mu.Unlock()
	key := string(in.Data)
	s.attempts[key]++
	if q := s.failWith[key]; len(q) > 0 {
		err := q[0]
		if len(q) > 1 {
			s.failWith[key] = q[1:]
		} else {
			delete(s.failWith, key)
		}
		return err
	}
	s.applied = append(s.applied, key)
	return nil
}

func (s *scriptedSink) Rooms() []string                                  { return nil }
func (s *scriptedSink) GetAwareness(string) (*awareness.Awareness, bool) { return nil, false }
func (s *scriptedSink) GetDoc(string) *crdt.Doc                          { return nil }

func (s *scriptedSink) timesApplied(data string) int {
	s.mu.Lock()
	defer s.mu.Unlock()
	n := 0
	for _, a := range s.applied {
		if a == data {
			n++
		}
	}
	return n
}

func (s *scriptedSink) tries(data string) int {
	s.mu.Lock()
	defer s.mu.Unlock()
	return s.attempts[data]
}

func publish(t *testing.T, r *Relay, room string, data ...byte) {
	t.Helper()
	if err := r.Publish(context.Background(), cluster.Outbound{Room: room, Kind: cluster.KindSync, Data: data}); err != nil {
		t.Fatalf("Publish: %v", err)
	}
}

// TestWriter_RetriesAFailedXAddBeforeDropping: a batched XADD that failed was
// dropped, so every instance holding the room missed it until it reloaded.
func TestWriter_RetriesAFailedXAddBeforeDropping(t *testing.T) {
	t.Parallel()
	_, mr := newTestClient(t)
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	const room = "retry-room"

	sinkB := &fakeSink{}
	relayB := startedRelay(t, ctx, mr.Addr(), sinkB)
	relayB.RoomActivated(room)
	if err := relayB.WaitCaughtUp(ctx, room); err != nil {
		t.Fatalf("WaitCaughtUp: %v", err)
	}

	relayA := mustRelay(t, mr.Addr())
	hook := &failXAdd{left: 2}
	relayA.client.AddHook(hook)
	t.Cleanup(func() { _ = relayA.Close() })
	if err := relayA.Start(ctx, &fakeSink{}); err != nil {
		t.Fatalf("Start: %v", err)
	}
	relayA.RoomActivated(room)

	publish(t, relayA, room, 7)
	waitFor(t, 5*time.Second, func() bool { return injected(sinkB, []byte{7}) })
	if n := hook.failures(); n != 2 {
		t.Errorf("XADD failed %d times, want the injected 2", n)
	}
	if d := relayA.DroppedWrites(); d != 0 {
		t.Errorf("DroppedWrites = %d, want 0 after a retry succeeded", d)
	}
	if relayA.OutboundGap(room) {
		t.Error("the room was flagged although its update reached the stream")
	}
}

// TestWriter_FlagsTheRoomOnceRetriesRunOut: when every attempt fails the batch
// is dropped, and the room is flagged so a repair from storage can find it.
func TestWriter_FlagsTheRoomOnceRetriesRunOut(t *testing.T) {
	t.Parallel()
	_, mr := newTestClient(t)
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	const room = "gap-room"

	relay := mustRelay(t, mr.Addr())
	relay.client.AddHook(&failXAdd{left: -1})
	t.Cleanup(func() { _ = relay.Close() })
	if err := relay.Start(ctx, &fakeSink{}); err != nil {
		t.Fatalf("Start: %v", err)
	}
	relay.RoomActivated(room)

	publish(t, relay, room, 7)
	// droppedWrites is counted before the room is flagged, so wait for both.
	waitFor(t, 8*time.Second, func() bool { return relay.DroppedWrites() == 1 && relay.OutboundGap(room) })
}

// TestPublish_FlagsTheRoomWhenTheQueueIsFull: Publish cannot block ygo's lane
// worker, so a full queue still drops, but the room is now flagged.
func TestPublish_FlagsTheRoomWhenTheQueueIsFull(t *testing.T) {
	t.Parallel()
	_, mr := newTestClient(t)
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	const room = "full-room"

	relay := mustRelay(t, mr.Addr())
	gate := newXAddGate()
	relay.client.AddHook(gate)
	t.Cleanup(func() { close(gate.release); _ = relay.Close() })
	if err := relay.Start(ctx, &fakeSink{}); err != nil {
		t.Fatalf("Start: %v", err)
	}
	relay.RoomActivated(room)

	for i := 0; i < writeQueueCap+2*syncBatch; i++ {
		publish(t, relay, room, byte(i))
	}
	if relay.DroppedWrites() == 0 {
		t.Fatal("setup: nothing overflowed the queue")
	}
	if !relay.OutboundGap(room) {
		t.Error("a room whose update overflowed the queue is not flagged")
	}
}

// TestReader_RetriesAnEntryWhoseInjectFailed: the reader moved its cursor past
// an entry even when injecting it failed, so that update was never applied.
func TestReader_RetriesAnEntryWhoseInjectFailed(t *testing.T) {
	t.Parallel()
	_, mr := newTestClient(t)
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	const room = "inject-room"

	sinkB := newScriptedSink()
	// An empty V1 update: a payload the room could take, so the failure is
	// the load's and not the entry's, and the entry is retried, not skipped.
	sinkB.failWith["\x00\x00"] = []error{errors.New("gcs: load failed")}
	relayB := mustRelay(t, mr.Addr())
	t.Cleanup(func() { _ = relayB.Close() })
	if err := relayB.Start(ctx, sinkB); err != nil {
		t.Fatalf("Start: %v", err)
	}
	relayB.RoomActivated(room)
	if err := relayB.WaitCaughtUp(ctx, room); err != nil {
		t.Fatalf("WaitCaughtUp: %v", err)
	}
	relayA := startedRelay(t, ctx, mr.Addr(), &fakeSink{})
	relayA.RoomActivated(room)

	publish(t, relayA, room, 0, 0)
	waitFor(t, 5*time.Second, func() bool { return sinkB.timesApplied("\x00\x00") == 1 })
	if n := sinkB.tries("\x00\x00"); n != 2 {
		t.Errorf("the entry was tried %d times, want 2: one failure, one success", n)
	}
}

// TestReader_SkipsAnEntryTheRoomCannotApply: holding the cursor must not let one
// bad entry wedge a room, so an entry the room can never apply is moved past.
func TestReader_SkipsAnEntryTheRoomCannotApply(t *testing.T) {
	t.Parallel()
	_, mr := newTestClient(t)
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	const room = "bad-entry-room"

	sinkB := newScriptedSink()
	sinkB.failWith["\x01"] = []error{ygows.ErrInvalidUpdate, ygows.ErrInvalidUpdate, ygows.ErrInvalidUpdate}
	relayB := mustRelay(t, mr.Addr())
	t.Cleanup(func() { _ = relayB.Close() })
	if err := relayB.Start(ctx, sinkB); err != nil {
		t.Fatalf("Start: %v", err)
	}
	relayB.RoomActivated(room)
	if err := relayB.WaitCaughtUp(ctx, room); err != nil {
		t.Fatalf("WaitCaughtUp: %v", err)
	}
	relayA := startedRelay(t, ctx, mr.Addr(), &fakeSink{})
	relayA.RoomActivated(room)

	publish(t, relayA, room, 1)
	publish(t, relayA, room, 2)
	waitFor(t, 5*time.Second, func() bool { return sinkB.timesApplied("\x02") == 1 })
	if n := sinkB.tries("\x01"); n != 1 {
		t.Errorf("the bad entry was tried %d times, want 1", n)
	}
}

// TestCatchUp_StopsOnShutdownWithoutClaimingToBeCaughtUp: a shutdown error with
// the context still live made catch-up report success with an empty cursor, so
// the reader marked the room caught up and read on from the wrong place.
func TestCatchUp_StopsOnShutdownWithoutClaimingToBeCaughtUp(t *testing.T) {
	t.Parallel()
	c, mr := newTestClient(t)
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	const room = "shutdown-room"

	if err := c.XAdd(ctx, &goredis.XAddArgs{Stream: streamKey(room), ID: "*", Values: xaddValues(msgTypeSync, []byte{4}, 424242)}).Err(); err != nil {
		t.Fatalf("seed: %v", err)
	}
	sink := newScriptedSink()
	sink.failWith["\x04"] = []error{ygows.ErrServerShutdown}
	relay := mustRelay(t, mr.Addr())
	t.Cleanup(func() { _ = relay.Close() })
	if err := relay.Start(ctx, sink); err != nil {
		t.Fatalf("Start: %v", err)
	}
	relay.RoomActivated(room)
	relay.mu.Lock()
	rs := relay.rooms[room]
	relay.mu.Unlock()

	select {
	case <-rs.done:
	case <-time.After(5 * time.Second):
		t.Fatal("the reader kept reading after the server reported shutdown")
	}
	select {
	case <-rs.caughtUp:
		t.Error("catch-up was marked complete though it stopped partway")
	default:
	}
}

// TestTrimStream_WaitsForAPendingReplay: a replay that has not finished can
// still need entries below the live cursor that the resident document lacks, so
// flushing and trimming then would discard the only copy. Trimming resumes once
// the replay is done.
func TestTrimStream_WaitsForAPendingReplay(t *testing.T) {
	t.Parallel()
	c, mr := newTestClient(t)
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	const room = "replay-trim"
	key := streamKey(room)

	// The cursor lands on the newest entry, so seed one at now: the old entry is
	// then past retention on its own, without leaning on the marker Start adds.
	now := time.Now().UnixMilli()
	old := now - (streamRetention + time.Minute).Milliseconds()
	for _, id := range []string{fmt.Sprintf("%d-0", old), fmt.Sprintf("%d-0", now)} {
		if err := c.XAdd(ctx, &goredis.XAddArgs{Stream: key, ID: id, Values: xaddValues(msgTypeSync, []byte{1}, 424242)}).Err(); err != nil {
			t.Fatalf("seed %s: %v", id, err)
		}
	}
	relay := mustRelay(t, mr.Addr())
	t.Cleanup(func() { _ = relay.Close() })
	if err := relay.Start(ctx, &fakeSink{}); err != nil {
		t.Fatalf("Start: %v", err)
	}
	relay.RoomActivated(room)
	relay.mu.Lock()
	rs := relay.rooms[room]
	relay.mu.Unlock()
	waitFor(t, 3*time.Second, func() bool {
		relay.mu.Lock()
		defer relay.mu.Unlock()
		return rs.lastID != "0" && rs.lastID != ""
	})
	oldLeft := func() bool {
		left, err := c.XRange(ctx, key, "-", "+").Result()
		if err != nil {
			t.Fatalf("XRANGE: %v", err)
		}
		for _, m := range left {
			if strings.HasPrefix(m.ID, fmt.Sprintf("%d-", old)) {
				return true
			}
		}
		return false
	}

	relay.mu.Lock()
	rs.replayed = make(chan struct{}) // a replay asked for and not yet finished
	relay.mu.Unlock()
	if err := relay.trimStream(ctx, room, rs); err != nil {
		t.Fatalf("trimStream: %v", err)
	}
	if !oldLeft() {
		t.Fatal("the stream was trimmed while a replay was pending")
	}

	relay.mu.Lock()
	rs.replayed = nil
	relay.mu.Unlock()
	if err := relay.trimStream(ctx, room, rs); err != nil {
		t.Fatalf("trimStream: %v", err)
	}
	if oldLeft() {
		t.Error("trimming did not resume once the replay was done")
	}
}
