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
)

// xrangeRecorder captures every XRANGE a client sends.
type xrangeRecorder struct {
	mu   sync.Mutex
	args [][]any
}

func (h *xrangeRecorder) DialHook(next goredis.DialHook) goredis.DialHook { return next }
func (h *xrangeRecorder) ProcessPipelineHook(next goredis.ProcessPipelineHook) goredis.ProcessPipelineHook {
	return next
}
func (h *xrangeRecorder) ProcessHook(next goredis.ProcessHook) goredis.ProcessHook {
	return func(ctx context.Context, cmd goredis.Cmder) error {
		if strings.EqualFold(cmd.Name(), "xrange") {
			h.mu.Lock()
			h.args = append(h.args, cmd.Args())
			h.mu.Unlock()
		}
		return next(ctx, cmd)
	}
}

func (h *xrangeRecorder) calls() [][]any {
	h.mu.Lock()
	defer h.mu.Unlock()
	return append([][]any(nil), h.args...)
}

// TestCatchUpPagesTheStream: catch-up must read the stream in bounded pages, not
// one XRANGE - + that materializes every entry, and must still replay all of them
// in order.
func TestCatchUpPagesTheStream(t *testing.T) {
	c, mr := newTestClient(t)
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	const room = "proj-page"

	total := readCount*2 + 5
	pipe := c.Pipeline()
	for i := 0; i < total; i++ {
		pipe.XAdd(ctx, &goredis.XAddArgs{
			Stream: streamKey(room),
			ID:     "*",
			Values: xaddValues(msgTypeSync, []byte(fmt.Sprintf("u%d", i)), 424242),
		})
	}
	if _, err := pipe.Exec(ctx); err != nil {
		t.Fatalf("seed: %v", err)
	}

	sink := &fakeSink{}
	relay := mustRelay(t, mr.Addr())
	defer relay.Close()
	rec := &xrangeRecorder{}
	relay.client.AddHook(rec)
	if err := relay.Start(ctx, sink); err != nil {
		t.Fatalf("Start: %v", err)
	}
	relay.RoomActivated(room)

	waitFor(t, 5*time.Second, func() bool { return sink.count() >= total })
	got := sink.snapshot()
	for i := 0; i < total; i++ {
		if want := fmt.Sprintf("u%d", i); string(got[i].Data) != want {
			t.Fatalf("entry %d = %q, want %q (catch-up out of order or incomplete)", i, got[i].Data, want)
		}
	}

	calls := rec.calls()
	if len(calls) < 3 {
		t.Fatalf("catch-up issued %d XRANGE call(s) for %d entries; want bounded pages", len(calls), total)
	}
	for _, a := range calls {
		if len(a) < 6 || !strings.EqualFold(fmt.Sprint(a[4]), "count") {
			t.Fatalf("XRANGE without COUNT: %v", a)
		}
	}
}

// TestTrimStreamDropsEntriesPastRetention: entries older than the retention
// window behind the reader's cursor are trimmed; recent ones survive.
func TestTrimStreamDropsEntriesPastRetention(t *testing.T) {
	c, mr := newTestClient(t)
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	const room = "proj-trim"
	key := streamKey(room)

	now := time.Now().UnixMilli()
	old := now - (streamRetention + time.Minute).Milliseconds()
	recent := now - (streamRetention - time.Minute).Milliseconds()
	for _, id := range []string{
		fmt.Sprintf("%d-0", old),
		fmt.Sprintf("%d-1", old),
		fmt.Sprintf("%d-0", recent),
	} {
		if err := c.XAdd(ctx, &goredis.XAddArgs{Stream: key, ID: id, Values: xaddValues(msgTypeSync, []byte{1}, 424242)}).Err(); err != nil {
			t.Fatalf("seed %s: %v", id, err)
		}
	}

	sink := &fakeSink{}
	relay := mustRelay(t, mr.Addr())
	defer relay.Close()
	if err := relay.Start(ctx, sink); err != nil {
		t.Fatalf("Start: %v", err)
	}
	relay.RoomActivated(room)
	rs := relay.rooms[room]
	waitFor(t, 3*time.Second, func() bool {
		relay.mu.Lock()
		defer relay.mu.Unlock()
		return rs.lastID != "0"
	})

	if err := relay.trimStream(ctx, room, rs); err != nil {
		t.Fatalf("trimStream: %v", err)
	}

	left, err := c.XRange(ctx, key, "-", "+").Result()
	if err != nil {
		t.Fatalf("XRANGE: %v", err)
	}
	for _, m := range left {
		if strings.HasPrefix(m.ID, fmt.Sprintf("%d-", old)) {
			t.Fatalf("entry %s is past retention and should have been trimmed", m.ID)
		}
	}
	if len(left) < 2 {
		t.Fatalf("only %d entries left; the recent entry and the activation marker must survive", len(left))
	}
}

// TestTrimStreamNeverPassesTheCursor: the boundary is anchored on what this
// reader has applied, so nothing newer than cursor-retention is ever trimmed even
// when the stream holds entries the reader has not reached.
func TestTrimStreamNeverPassesTheCursor(t *testing.T) {
	c, mr := newTestClient(t)
	ctx := context.Background()
	const room = "proj-cursor"
	key := streamKey(room)

	relay := mustRelay(t, mr.Addr())
	defer relay.Close()

	cursorMs := time.Now().Add(-time.Hour).UnixMilli()
	afterBoundary := cursorMs - (streamRetention - time.Minute).Milliseconds()
	if err := c.XAdd(ctx, &goredis.XAddArgs{Stream: key, ID: fmt.Sprintf("%d-0", afterBoundary), Values: xaddValues(msgTypeSync, []byte{1}, 424242)}).Err(); err != nil {
		t.Fatalf("seed: %v", err)
	}
	rs := &roomState{lastID: fmt.Sprintf("%d-0", cursorMs)}
	if err := relay.trimStream(ctx, room, rs); err != nil {
		t.Fatalf("trimStream: %v", err)
	}
	if n, _ := c.XLen(ctx, key).Result(); n != 1 {
		t.Fatalf("XLEN = %d, want 1: trimmed an entry within retention of the cursor", n)
	}

	rs.lastID = "0"
	if err := relay.trimStream(ctx, room, rs); err != nil {
		t.Fatalf("trimStream with no cursor: %v", err)
	}
	if n, _ := c.XLen(ctx, key).Result(); n != 1 {
		t.Fatalf("XLEN = %d, want 1: trimmed before the reader caught up", n)
	}
}

// xtrimRecorder captures every XTRIM a client sends.
type xtrimRecorder struct {
	mu   sync.Mutex
	args [][]any
}

func (h *xtrimRecorder) DialHook(next goredis.DialHook) goredis.DialHook { return next }
func (h *xtrimRecorder) ProcessPipelineHook(next goredis.ProcessPipelineHook) goredis.ProcessPipelineHook {
	return next
}
func (h *xtrimRecorder) ProcessHook(next goredis.ProcessHook) goredis.ProcessHook {
	return func(ctx context.Context, cmd goredis.Cmder) error {
		if strings.EqualFold(cmd.Name(), "xtrim") {
			h.mu.Lock()
			h.args = append(h.args, cmd.Args())
			h.mu.Unlock()
		}
		return next(ctx, cmd)
	}
}

func (h *xtrimRecorder) calls() [][]any {
	h.mu.Lock()
	defer h.mu.Unlock()
	return append([][]any(nil), h.args...)
}

// failingFlusher refuses every checkpoint, standing in for a GCS outage.
type failingFlusher struct{ err error }

func (f *failingFlusher) FlushRoom(context.Context, string) error { return f.err }

// seedAged puts n entries at cursorMs-ageBack into the room's stream and returns
// the stream key.
func seedAged(t *testing.T, c *goredis.Client, room string, ids ...string) string {
	t.Helper()
	key := streamKey(room)
	for _, id := range ids {
		if err := c.XAdd(context.Background(), &goredis.XAddArgs{
			Stream: key, ID: id, Values: xaddValues(msgTypeSync, []byte{1}, 424242),
		}).Err(); err != nil {
			t.Fatalf("seed %s: %v", id, err)
		}
	}
	return key
}

// TestTrimStreamStopsAtTheSlowestReader: XTRIM mutates a stream every instance
// shares. A second instance stuck well behind — blocked in Sink.Inject, say —
// must not have the entries it still owes its room deleted underneath it.
func TestTrimStreamStopsAtTheSlowestReader(t *testing.T) {
	c, mr := newTestClient(t)
	ctx := context.Background()
	const room = "proj-slow-reader"

	relay := mustRelay(t, mr.Addr())
	defer relay.Close()

	nowMs := time.Now().UnixMilli()
	fastCursorMs := nowMs
	// The lagging instance is an hour behind, far past the retention window.
	slowCursorMs := nowMs - time.Hour.Milliseconds()
	// An entry the slow reader has not applied yet, but which is old enough that
	// the age bound alone would discard it.
	unread := slowCursorMs + time.Minute.Milliseconds()
	key := seedAged(t, c, room, fmt.Sprintf("%d-0", unread), fmt.Sprintf("%d-0", nowMs))

	// Both instances heartbeat; both advertise a cursor.
	const slowID uint64 = 777
	if err := updateHeartbeat(ctx, c, room, relay.clientID); err != nil {
		t.Fatalf("self heartbeat: %v", err)
	}
	if err := updateHeartbeat(ctx, c, room, slowID); err != nil {
		t.Fatalf("slow heartbeat: %v", err)
	}
	if err := c.HSet(ctx, cursorsKey(room), fmt.Sprint(slowID), slowCursorMs).Err(); err != nil {
		t.Fatalf("slow cursor: %v", err)
	}

	rs := &roomState{lastID: fmt.Sprintf("%d-0", fastCursorMs)}
	if err := relay.trimStream(ctx, room, rs); err != nil {
		t.Fatalf("trimStream: %v", err)
	}

	left, err := c.XRange(ctx, key, "-", "+").Result()
	if err != nil {
		t.Fatalf("XRANGE: %v", err)
	}
	for _, m := range left {
		if strings.HasPrefix(m.ID, fmt.Sprintf("%d-", unread)) {
			return // survived, as it must
		}
	}
	t.Fatalf("entry %d-0 was trimmed past the lagging instance's cursor; it would never be delivered", unread)
}

// TestTrimStreamSkipsWhenAnInstanceAdvertisesNoCursor: a coexisting Rust node (or
// a Go reader whose cursor entry has expired) is active in the heartbeat hash but
// writes no cursor. With nothing to bound the trim against, it must not happen.
func TestTrimStreamSkipsWhenAnInstanceAdvertisesNoCursor(t *testing.T) {
	c, mr := newTestClient(t)
	ctx := context.Background()
	const room = "proj-unknown-reader"

	relay := mustRelay(t, mr.Addr())
	defer relay.Close()

	nowMs := time.Now().UnixMilli()
	old := nowMs - (streamRetention + time.Hour).Milliseconds()
	key := seedAged(t, c, room, fmt.Sprintf("%d-0", old), fmt.Sprintf("%d-0", nowMs))

	if err := updateHeartbeat(ctx, c, room, relay.clientID); err != nil {
		t.Fatalf("self heartbeat: %v", err)
	}
	// Active, but never writes doc:cursors.
	if err := updateHeartbeat(ctx, c, room, 999); err != nil {
		t.Fatalf("foreign heartbeat: %v", err)
	}

	rs := &roomState{lastID: fmt.Sprintf("%d-0", nowMs)}
	if err := relay.trimStream(ctx, room, rs); err != nil {
		t.Fatalf("trimStream: %v", err)
	}
	if n, _ := c.XLen(ctx, key).Result(); n != 2 {
		t.Fatalf("XLEN = %d, want 2: trimmed while an active instance advertised no cursor", n)
	}
}

// TestTrimStreamRequiresADurableCheckpoint: age does not prove durability — the
// cursor advances on Inject while GCS persistence is asynchronous. If the
// checkpoint fails (a GCS outage), Redis holds the only replay copy and nothing
// may be discarded.
func TestTrimStreamRequiresADurableCheckpoint(t *testing.T) {
	c, mr := newTestClient(t)
	ctx := context.Background()
	const room = "proj-durable"

	relay := mustRelay(t, mr.Addr())
	defer relay.Close()
	relay.flusher = &failingFlusher{err: errors.New("gcs unavailable")}

	nowMs := time.Now().UnixMilli()
	old := nowMs - (streamRetention + time.Hour).Milliseconds()
	key := seedAged(t, c, room, fmt.Sprintf("%d-0", old), fmt.Sprintf("%d-0", nowMs))

	rs := &roomState{lastID: fmt.Sprintf("%d-0", nowMs)}
	if err := relay.trimStream(ctx, room, rs); err == nil {
		t.Fatal("trimStream = nil, want the checkpoint error surfaced")
	}
	if n, _ := c.XLen(ctx, key).Result(); n != 2 {
		t.Fatalf("XLEN = %d, want 2: trimmed without a durable checkpoint", n)
	}

	// With the checkpoint succeeding, the same call trims.
	fl := &fakeFlusher{}
	relay.flusher = fl
	if err := relay.trimStream(ctx, room, rs); err != nil {
		t.Fatalf("trimStream: %v", err)
	}
	if got := fl.calls(); len(got) != 1 || got[0] != room {
		t.Fatalf("FlushRoom calls = %v, want one for %q", got, room)
	}
	if n, _ := c.XLen(ctx, key).Result(); n != 1 {
		t.Fatalf("XLEN = %d, want 1 after a successful checkpoint", n)
	}
}

// TestTrimStreamSkipsTheCheckpointWithNothingToTrim: an idle room must settle
// into cheap no-op ticks, not a full GCS snapshot every minute.
func TestTrimStreamSkipsTheCheckpointWithNothingToTrim(t *testing.T) {
	c, mr := newTestClient(t)
	ctx := context.Background()
	const room = "proj-idle"

	relay := mustRelay(t, mr.Addr())
	defer relay.Close()
	fl := &fakeFlusher{}
	relay.flusher = fl

	nowMs := time.Now().UnixMilli()
	seedAged(t, c, room, fmt.Sprintf("%d-0", nowMs))

	rs := &roomState{lastID: fmt.Sprintf("%d-0", nowMs)}
	if err := relay.trimStream(ctx, room, rs); err != nil {
		t.Fatalf("trimStream: %v", err)
	}
	if got := fl.calls(); len(got) != 0 {
		t.Fatalf("FlushRoom calls = %v, want none with nothing below the boundary", got)
	}
}

// TestTrimStreamUsesApproximateMinID: exact MINID costs time proportional to the
// entries removed, so the first tick on an oversized stream would monopolize
// Redis. The Rust backend uses `MINID ~` for the same reason.
func TestTrimStreamUsesApproximateMinID(t *testing.T) {
	c, mr := newTestClient(t)
	ctx := context.Background()
	const room = "proj-approx"

	relay := mustRelay(t, mr.Addr())
	defer relay.Close()
	rec := &xtrimRecorder{}
	relay.client.AddHook(rec)

	nowMs := time.Now().UnixMilli()
	old := nowMs - (streamRetention + time.Hour).Milliseconds()
	seedAged(t, c, room, fmt.Sprintf("%d-0", old), fmt.Sprintf("%d-0", nowMs))

	rs := &roomState{lastID: fmt.Sprintf("%d-0", nowMs)}
	if err := relay.trimStream(ctx, room, rs); err != nil {
		t.Fatalf("trimStream: %v", err)
	}

	calls := rec.calls()
	if len(calls) != 1 {
		t.Fatalf("XTRIM calls = %d, want 1", len(calls))
	}
	var approx bool
	for _, a := range calls[0] {
		if fmt.Sprint(a) == "~" {
			approx = true
		}
	}
	if !approx {
		t.Fatalf("XTRIM %v is exact; want the approximate MINID ~ form", calls[0])
	}
}
