package redis

import (
	"context"
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
