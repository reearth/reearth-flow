package redis

import (
	"context"
	"strings"
	"sync"
	"testing"
	"time"

	goredis "github.com/redis/go-redis/v9"
	"github.com/reearth/ygo/cluster"
)

// xaddGate is a go-redis Hook that stalls every XADD (single or pipelined) until
// released, letting a test prove Publish returns before the XADD completes. A Hook
// is required because the batched writer uses a pipeline.
type xaddGate struct {
	release  chan struct{}
	mu       sync.Mutex
	xaddSeen int
}

func (g *xaddGate) note(cmds ...goredis.Cmder) {
	for _, c := range cmds {
		if strings.EqualFold(c.Name(), "xadd") {
			g.mu.Lock()
			g.xaddSeen++
			g.mu.Unlock()
		}
	}
}

func (g *xaddGate) gate(ctx context.Context, has bool) {
	if !has {
		return
	}
	select {
	case <-g.release:
	case <-ctx.Done():
	}
}

func hasXAdd(cmds ...goredis.Cmder) bool {
	for _, c := range cmds {
		if strings.EqualFold(c.Name(), "xadd") {
			return true
		}
	}
	return false
}

func (g *xaddGate) DialHook(next goredis.DialHook) goredis.DialHook { return next }

func (g *xaddGate) ProcessHook(next goredis.ProcessHook) goredis.ProcessHook {
	return func(ctx context.Context, cmd goredis.Cmder) error {
		g.note(cmd)
		g.gate(ctx, hasXAdd(cmd))
		return next(ctx, cmd)
	}
}

func (g *xaddGate) ProcessPipelineHook(next goredis.ProcessPipelineHook) goredis.ProcessPipelineHook {
	return func(ctx context.Context, cmds []goredis.Cmder) error {
		g.note(cmds...)
		g.gate(ctx, hasXAdd(cmds...))
		return next(ctx, cmds)
	}
}

func (g *xaddGate) seen() int {
	g.mu.Lock()
	defer g.mu.Unlock()
	return g.xaddSeen
}

func newXAddGate() *xaddGate { return &xaddGate{release: make(chan struct{})} }

// TestPublishReturnsImmediatelyWhenXAddBlocks: Publish must not block on the
// underlying XADD. We stall the writer's XADD and assert Publish still returns.
func TestPublishReturnsImmediatelyWhenXAddBlocks(t *testing.T) {
	_, mr := newTestClient(t)
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	const room = "proj1"

	relay := mustRelay(t, mr.Addr())
	gate := newXAddGate()
	relay.client.AddHook(gate)
	defer relay.Close()
	_ = relay.Start(ctx, &fakeSink{})
	relay.RoomActivated(room)

	// The empty-marker on RoomActivated is a Lua EVAL, not XADD, so the gate only
	// stalls the first publish's XADD.
	done := make(chan struct{})
	go func() {
		for i := 0; i < 10; i++ {
			_ = relay.Publish(ctx, cluster.Outbound{Room: room, Kind: cluster.KindSync, Data: []byte{byte(i)}})
		}
		close(done)
	}()

	select {
	case <-done:
	case <-time.After(2 * time.Second):
		t.Fatal("Publish blocked on a stalled XADD (async write queue not in effect)")
	}

	close(gate.release)
	waitFor(t, 3*time.Second, func() bool { return gate.seen() >= 1 })
}

// TestWriterDrainsInOrder asserts the writer flushes enqueued updates in order.
func TestWriterDrainsInOrder(t *testing.T) {
	c, mr := newTestClient(t)
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	const room = "proj1"

	relay := mustRelay(t, mr.Addr())
	defer relay.Close()
	_ = relay.Start(ctx, &fakeSink{})
	relay.RoomActivated(room)

	want := [][]byte{{1}, {2}, {3}, {4}, {5}}
	for _, b := range want {
		if err := relay.Publish(ctx, cluster.Outbound{Room: room, Kind: cluster.KindSync, Data: b}); err != nil {
			t.Fatalf("publish: %v", err)
		}
	}

	// Wait until all data entries (plus the empty marker) are on the stream.
	waitFor(t, 3*time.Second, func() bool {
		n, _ := c.XLen(ctx, streamKey(room)).Result()
		return n >= int64(len(want)+1)
	})

	msgs, err := c.XRange(ctx, streamKey(room), "-", "+").Result()
	if err != nil {
		t.Fatalf("xrange: %v", err)
	}
	var got [][]byte
	for _, m := range msgs {
		e := parseEntry(m, relay.clientID)
		if e.kind == kindSync && len(e.data) > 0 {
			got = append(got, e.data)
		}
	}
	if len(got) != len(want) {
		t.Fatalf("got %d data entries, want %d", len(got), len(want))
	}
	for i := range want {
		if string(got[i]) != string(want[i]) {
			t.Fatalf("entry[%d] = %v, want %v (order broken)", i, got[i], want[i])
		}
	}
}

// TestPublishDropsAndCountsWhenQueueFull: a full per-room queue makes Publish drop
// the update and increment the drop counter. We stall the writer, then overflow it.
func TestPublishDropsAndCountsWhenQueueFull(t *testing.T) {
	_, mr := newTestClient(t)
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	const room = "proj1"

	relay := mustRelay(t, mr.Addr())
	gate := newXAddGate()
	relay.client.AddHook(gate)
	defer relay.Close()
	_ = relay.Start(ctx, &fakeSink{})
	relay.RoomActivated(room)

	// Publish far more than the queue cap while the writer is stalled on XADD, so
	// the overflow is dropped.
	total := writeQueueCap + 200
	for i := 0; i < total; i++ {
		_ = relay.Publish(ctx, cluster.Outbound{Room: room, Kind: cluster.KindSync, Data: []byte{1}})
	}

	if got := relay.DroppedWrites(); got == 0 {
		t.Fatal("expected dropped writes counter > 0 when queue is full, got 0")
	}
	// Sanity: drops cannot exceed total published.
	if got := relay.DroppedWrites(); got > uint64(total) {
		t.Fatalf("dropped %d exceeds total published %d", got, total)
	}
	close(gate.release)
}

// TestEnqueueNoOpForEvictingRoom: Publish to an evicting room buffers nothing and
// drops nothing — it is simply refused.
func TestEnqueueNoOpForEvictingRoom(t *testing.T) {
	c, mr := newTestClient(t)
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	const room = "proj1"

	relay := mustRelay(t, mr.Addr())
	defer relay.Close()
	_ = relay.Start(ctx, &fakeSink{})
	relay.RoomActivated(room)
	waitFor(t, 2*time.Second, func() bool { return mr.Exists(streamKey(room)) })

	relay.mu.Lock()
	relay.rooms[room].evicting = true
	relay.mu.Unlock()

	before, _ := c.XLen(ctx, streamKey(room)).Result()
	dropsBefore := relay.DroppedWrites()
	if err := relay.Publish(ctx, cluster.Outbound{Room: room, Kind: cluster.KindSync, Data: []byte{1}}); err != nil {
		t.Fatalf("publish: %v", err)
	}
	// Give any (incorrect) writer a moment to flush.
	time.Sleep(200 * time.Millisecond)
	after, _ := c.XLen(ctx, streamKey(room)).Result()
	if after != before {
		t.Fatalf("evicting-room publish wrote to the stream: before=%d after=%d", before, after)
	}
	if relay.DroppedWrites() != dropsBefore {
		t.Fatal("evicting-room publish counted a drop; it should be a silent no-op")
	}
}

// TestPipelineXAddSharesOneTimestamp asserts a single pipelined flush stamps every
// entry with one timestamp. Tested at the codec seam so it is deterministic.
func TestPipelineXAddSharesOneTimestamp(t *testing.T) {
	c, _ := newTestClient(t)
	ctx := context.Background()
	const room = "proj1"
	const id uint64 = 4242

	const n = 10
	items := make([]writeItem, n)
	for i := range items {
		items[i] = writeItem{kind: cluster.KindSync, data: []byte{byte(i)}}
	}
	if err := pipelineXAdd(ctx, c, streamKey(room), id, items); err != nil {
		t.Fatalf("pipelineXAdd: %v", err)
	}

	msgs, _ := c.XRange(ctx, streamKey(room), "-", "+").Result()
	if len(msgs) != n {
		t.Fatalf("got %d entries, want %d", len(msgs), n)
	}
	timestamps := map[string]int{}
	for i, m := range msgs {
		ts, _ := m.Values["timestamp"].(string)
		timestamps[ts]++
		e := parseEntry(m, id)
		if e.kind != kindSync || len(e.data) != 1 || e.data[0] != byte(i) {
			t.Fatalf("entry[%d] = %+v, want sync {%d}", i, e, i)
		}
	}
	if len(timestamps) != 1 {
		t.Fatalf("batch used %d distinct timestamps, want 1: %v", len(timestamps), timestamps)
	}
	for ts, cnt := range timestamps {
		if cnt != n {
			t.Fatalf("timestamp %q covered %d entries, want %d", ts, cnt, n)
		}
	}
}
