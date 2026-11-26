package redis

import (
	"context"
	"errors"
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
	waitFor(t, 8*time.Second, func() bool { return relay.DroppedWrites() == 1 })
	if !relay.OutboundGap(room) {
		t.Error("a room whose update was dropped is not flagged")
	}
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
	sinkB.failWith["\x09"] = []error{errors.New("gcs: load failed")}
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

	publish(t, relayA, room, 9)
	waitFor(t, 5*time.Second, func() bool { return sinkB.timesApplied("\x09") == 1 })
	if n := sinkB.tries("\x09"); n != 2 {
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
