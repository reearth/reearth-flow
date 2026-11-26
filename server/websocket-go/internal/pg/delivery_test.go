package pg

import (
	"context"
	"errors"
	"sync"
	"testing"
	"time"

	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgconn"
	"github.com/jackc/pgx/v5/pgxpool"
	"github.com/reearth/ygo/awareness"
	"github.com/reearth/ygo/cluster"
	"github.com/reearth/ygo/crdt"
	ygows "github.com/reearth/ygo/provider/websocket"
)

const deliveryRoom = "550e8400-e29b-41d4-a716-446655440077"

// faultyQ fails or blocks the relay's own insert batches and passes everything
// else through, so eviction and heartbeats behave as usual.
type faultyQ struct {
	*pgxpool.Pool
	mu       sync.Mutex
	failLeft int // inserts still to fail; negative fails every one
	failed   int
	block    chan struct{} // non-nil: inserts wait for it to close
}

func isAppendBatch(b *pgx.Batch) bool {
	for _, qq := range b.QueuedQueries {
		if qq.SQL == appendSQL {
			return true
		}
	}
	return false
}

func (q *faultyQ) SendBatch(ctx context.Context, b *pgx.Batch) pgx.BatchResults {
	if isAppendBatch(b) {
		if q.block != nil {
			select {
			case <-q.block:
			case <-ctx.Done():
				return errResults{ctx.Err()}
			}
		}
		q.mu.Lock()
		fail := q.failLeft != 0
		if q.failLeft > 0 {
			q.failLeft--
		}
		if fail {
			q.failed++
		}
		q.mu.Unlock()
		if fail {
			return errResults{errors.New("injected insert failure")}
		}
	}
	return q.Pool.SendBatch(ctx, b)
}

func (q *faultyQ) failures() int {
	q.mu.Lock()
	defer q.mu.Unlock()
	return q.failed
}

type errResults struct{ err error }

func (r errResults) Exec() (pgconn.CommandTag, error) { return pgconn.CommandTag{}, r.err }
func (r errResults) Query() (pgx.Rows, error)         { return nil, r.err }
func (r errResults) QueryRow() pgx.Row                { return errRow(r) }
func (r errResults) Close() error                     { return r.err }

type errRow struct{ err error }

func (r errRow) Scan(...any) error { return r.err }

// startRelayOn starts a relay on a caller's Querier.
func startRelayOn(t *testing.T, q Querier, sink cluster.Sink) *Relay {
	t.Helper()
	r, err := New(Options{Q: q, PollEvery: 5 * time.Millisecond, Logger: testLogger(t), Flusher: noopFlusher{}})
	if err != nil {
		t.Fatalf("New: %v", err)
	}
	ctx, cancel := context.WithCancel(context.Background())
	t.Cleanup(func() { cancel(); _ = r.Close() })
	if err := r.Start(ctx, sink); err != nil {
		t.Fatalf("Start: %v", err)
	}
	return r
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

func publishSync(t *testing.T, r *Relay, data ...byte) {
	t.Helper()
	if err := r.Publish(context.Background(), cluster.Outbound{Room: deliveryRoom, Kind: cluster.KindSync, Data: data}); err != nil {
		t.Fatalf("Publish: %v", err)
	}
}

func received(s *fakeSink, want byte) bool {
	for _, p := range s.payloads() {
		if len(p) == 1 && p[0] == want {
			return true
		}
	}
	return false
}

// TestWriter_RetriesAFailedInsertBeforeDropping: a failed insert was dropped
// with only a debug log and no count, so the loss was invisible as well.
func TestWriter_RetriesAFailedInsertBeforeDropping(t *testing.T) {
	t.Parallel()
	pool := newPool(t)
	ctx := context.Background()

	sinkB := &fakeSink{}
	relayB := startRelay(t, pool, sinkB, noopFlusher{})
	relayB.RoomActivated(deliveryRoom)
	if err := relayB.WaitCaughtUp(ctx, deliveryRoom); err != nil {
		t.Fatalf("WaitCaughtUp: %v", err)
	}
	fq := &faultyQ{Pool: pool, failLeft: 2}
	relayA := startRelayOn(t, fq, &fakeSink{})
	relayA.RoomActivated(deliveryRoom)

	publishSync(t, relayA, 7)
	eventually(t, 5*time.Second, "the retried update to arrive", func() bool { return received(sinkB, 7) })
	if n := fq.failures(); n != 2 {
		t.Errorf("the insert failed %d times, want the injected 2", n)
	}
	if d := relayA.DroppedWrites(); d != 0 {
		t.Errorf("DroppedWrites = %d, want 0 after a retry succeeded", d)
	}
	if relayA.OutboundGap(deliveryRoom) {
		t.Error("the room was flagged although its update reached the stream")
	}
}

// TestWriter_FlagsTheRoomOnceInsertRetriesRunOut: once every attempt fails the
// batch is dropped, counted, and the room flagged.
func TestWriter_FlagsTheRoomOnceInsertRetriesRunOut(t *testing.T) {
	t.Parallel()
	pool := newPool(t)

	relay := startRelayOn(t, &faultyQ{Pool: pool, failLeft: -1}, &fakeSink{})
	relay.RoomActivated(deliveryRoom)

	publishSync(t, relay, 7)
	eventually(t, 8*time.Second, "the batch to be dropped", func() bool { return relay.DroppedWrites() == 1 })
	if !relay.OutboundGap(deliveryRoom) {
		t.Error("a room whose update was dropped is not flagged")
	}
}

// TestPublish_FlagsTheRoomWhenTheInsertQueueIsFull: a full queue still drops,
// since Publish cannot block ygo's lane worker, but the room is now flagged.
func TestPublish_FlagsTheRoomWhenTheInsertQueueIsFull(t *testing.T) {
	t.Parallel()
	pool := newPool(t)

	fq := &faultyQ{Pool: pool, block: make(chan struct{})}
	relay := startRelayOn(t, fq, &fakeSink{})
	t.Cleanup(func() { close(fq.block) }) // runs before the relay's own cleanup
	relay.RoomActivated(deliveryRoom)

	for i := 0; i < writeQueueCap+2*syncBatch; i++ {
		publishSync(t, relay, byte(i))
	}
	if relay.DroppedWrites() == 0 {
		t.Fatal("setup: nothing overflowed the queue")
	}
	if !relay.OutboundGap(deliveryRoom) {
		t.Error("a room whose update overflowed the queue is not flagged")
	}
}

// TestReader_RetriesARowWhoseInjectFailed: the reader moved its cursor past a
// row even when injecting it failed, so that update was never applied.
func TestReader_RetriesARowWhoseInjectFailed(t *testing.T) {
	t.Parallel()
	pool := newPool(t)
	ctx := context.Background()

	sinkB := newScriptedSink()
	sinkB.failWith["\x09"] = []error{errors.New("gcs: load failed")}
	relayB := startRelay(t, pool, sinkB, noopFlusher{})
	relayB.RoomActivated(deliveryRoom)
	if err := relayB.WaitCaughtUp(ctx, deliveryRoom); err != nil {
		t.Fatalf("WaitCaughtUp: %v", err)
	}
	relayA := startRelay(t, pool, &fakeSink{}, noopFlusher{})
	relayA.RoomActivated(deliveryRoom)

	publishSync(t, relayA, 9)
	eventually(t, 5*time.Second, "the row to be applied once", func() bool { return sinkB.timesApplied("\x09") == 1 })
	if n := sinkB.tries("\x09"); n != 2 {
		t.Errorf("the row was tried %d times, want 2: one failure, one success", n)
	}
}

// TestReader_SkipsARowTheRoomCannotApply: holding the cursor must not let one bad
// row wedge a room, so a row the room can never apply is moved past.
func TestReader_SkipsARowTheRoomCannotApply(t *testing.T) {
	t.Parallel()
	pool := newPool(t)
	ctx := context.Background()

	sinkB := newScriptedSink()
	sinkB.failWith["\x01"] = []error{ygows.ErrInvalidUpdate, ygows.ErrInvalidUpdate, ygows.ErrInvalidUpdate}
	relayB := startRelay(t, pool, sinkB, noopFlusher{})
	relayB.RoomActivated(deliveryRoom)
	if err := relayB.WaitCaughtUp(ctx, deliveryRoom); err != nil {
		t.Fatalf("WaitCaughtUp: %v", err)
	}
	relayA := startRelay(t, pool, &fakeSink{}, noopFlusher{})
	relayA.RoomActivated(deliveryRoom)

	publishSync(t, relayA, 1)
	publishSync(t, relayA, 2)
	eventually(t, 5*time.Second, "the next row to be applied", func() bool { return sinkB.timesApplied("\x02") == 1 })
	if n := sinkB.tries("\x01"); n != 1 {
		t.Errorf("the bad row was tried %d times, want 1", n)
	}
}

// TestPublish_DeliversALateUpdateToAnotherHolder: ygo hands a room's last
// updates over after deactivating it here. The Postgres relay dropped them, so
// instances still holding the room missed them until they reloaded it.
func TestPublish_DeliversALateUpdateToAnotherHolder(t *testing.T) {
	t.Parallel()
	pool := newPool(t)
	ctx := context.Background()

	sinkB := &fakeSink{}
	relayB := startRelay(t, pool, sinkB, noopFlusher{})
	relayB.RoomActivated(deliveryRoom)
	if err := relayB.WaitCaughtUp(ctx, deliveryRoom); err != nil {
		t.Fatalf("WaitCaughtUp: %v", err)
	}
	relayA := startRelay(t, pool, &fakeSink{}, noopFlusher{})
	relayA.RoomActivated(deliveryRoom)
	relayA.RoomDeactivated(deliveryRoom)

	publishSync(t, relayA, 5)
	eventually(t, 5*time.Second, "the late update to reach the other holder", func() bool { return received(sinkB, 5) })
	if n := relayA.LateWrites(); n != 1 {
		t.Errorf("LateWrites = %d, want 1", n)
	}
}

// TestPublish_SkipsALateUpdateNobodyElseHolds: with no other holder the update
// is already persisted, so nothing is written, as Redis's NOMKSTREAM decides.
func TestPublish_SkipsALateUpdateNobodyElseHolds(t *testing.T) {
	t.Parallel()
	pool := newPool(t)
	ctx := context.Background()

	relayA := startRelay(t, pool, &fakeSink{}, noopFlusher{})
	relayA.RoomActivated(deliveryRoom)
	relayA.RoomDeactivated(deliveryRoom)

	publishSync(t, relayA, 5)
	var rows int
	if err := pool.QueryRow(ctx, `SELECT count(*) FROM ws_stream WHERE doc_id = $1 AND data = $2`, deliveryRoom, []byte{5}).Scan(&rows); err != nil {
		t.Fatalf("count rows: %v", err)
	}
	if rows != 0 {
		t.Errorf("%d rows written for a late update nobody else holds, want 0", rows)
	}
	if n := relayA.LateWrites(); n != 0 {
		t.Errorf("LateWrites = %d, want 0", n)
	}
}
