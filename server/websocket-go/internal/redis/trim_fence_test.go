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

// liveFlusher is a LiveCheckpointer whose checkpoint runs hook first, so a test
// can start a load (or a replay) while a trim is inside its flush.
type liveFlusher struct {
	mu   sync.Mutex
	live bool
	hook func()
}

func (f *liveFlusher) set(live bool, hook func()) {
	f.mu.Lock()
	f.live, f.hook = live, hook
	f.mu.Unlock()
}

func (f *liveFlusher) CheckpointLive(context.Context, string) (bool, error) {
	f.mu.Lock()
	live, hook := f.live, f.hook
	f.mu.Unlock()
	if hook != nil {
		hook()
	}
	return live, nil
}

func (f *liveFlusher) FlushRoom(ctx context.Context, room string) error {
	_, err := f.CheckpointLive(ctx, room)
	return err
}

// trimFixture is an active room whose stream holds one entry past retention
// and one at now, with the reader caught up, so a trim that is allowed to run
// removes the old entry.
type trimFixture struct {
	relay  *Relay
	rs     *roomState
	room   string
	client *goredis.Client
	old    int64
}

func newTrimFixture(t *testing.T, room string, fl *liveFlusher) *trimFixture {
	t.Helper()
	c, mr := newTestClient(t)
	ctx, cancel := context.WithCancel(context.Background())
	t.Cleanup(cancel)
	now := time.Now().UnixMilli()
	old := now - (streamRetention + time.Minute).Milliseconds()
	for _, id := range []string{fmt.Sprintf("%d-0", old), fmt.Sprintf("%d-0", now)} {
		if err := c.XAdd(ctx, &goredis.XAddArgs{Stream: streamKey(room), ID: id, Values: xaddValues(msgTypeSync, []byte{1}, 424242)}).Err(); err != nil {
			t.Fatalf("seed %s: %v", id, err)
		}
	}
	relay, err := New(Options{Addr: mr.Addr(), Flusher: fl})
	if err != nil {
		t.Fatalf("New: %v", err)
	}
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
	return &trimFixture{relay: relay, rs: rs, room: room, client: c, old: old}
}

func (f *trimFixture) oldLeft(t *testing.T) bool {
	t.Helper()
	left, err := f.client.XRange(context.Background(), streamKey(f.room), "-", "+").Result()
	if err != nil {
		t.Fatalf("XRANGE: %v", err)
	}
	for _, m := range left {
		if strings.HasPrefix(m.ID, fmt.Sprintf("%d-", f.old)) {
			return true
		}
	}
	return false
}

func (f *trimFixture) trim(t *testing.T) {
	t.Helper()
	if err := f.relay.trimStream(context.Background(), f.room, f.rs); err != nil {
		t.Fatalf("trimStream: %v", err)
	}
}

// TestTrimStream_LoadDuringFlushBlocksTrim: a room that starts loading while
// the trim flushes may have read storage without what the trim removes.
func TestTrimStream_LoadDuringFlushBlocksTrim(t *testing.T) {
	t.Parallel()
	fl := &liveFlusher{live: true}
	f := newTrimFixture(t, "trim-load-during-flush", fl)
	fl.set(true, func() { f.relay.LoadStarting(context.Background(), f.room)(nil) })

	f.trim(t)
	if !f.oldLeft(t) {
		t.Fatal("the stream was trimmed while a load that began during the flush was pending")
	}

	fl.set(true, nil)
	snap := f.relay.passBegin(f.room)
	f.relay.passEnd(context.Background(), f.room, snap)
	f.trim(t)
	if f.oldLeft(t) {
		t.Error("trimming did not resume once a pass covered the load")
	}
}

// TestTrimStream_LoadClearedInsideFlushBlocksTrim: a load can start and have
// its fence cleared while the flush runs, after the flush took a document that
// lacked the entry; only the load's generation shows it happened.
func TestTrimStream_LoadClearedInsideFlushBlocksTrim(t *testing.T) {
	t.Parallel()
	fl := &liveFlusher{live: true}
	f := newTrimFixture(t, "trim-load-cleared", fl)
	fl.set(true, func() {
		ctx := context.Background()
		f.relay.LoadStarting(ctx, f.room)(nil)
		snap := f.relay.passBegin(f.room)
		f.relay.passEnd(ctx, f.room, snap)
	})

	f.trim(t)
	if !f.oldLeft(t) {
		t.Fatal("the stream was trimmed although a load started and cleared during the flush")
	}
	fl.set(true, nil)
	f.trim(t)
	if f.oldLeft(t) {
		t.Error("trimming did not resume on the next tick")
	}
}

// TestTrimStream_ReplayInsideFlushBlocksTrim: a room activated onto this state
// during the flush replays and finishes before the re-check, leaving no pending
// replay behind; the replay count still shows it.
func TestTrimStream_ReplayInsideFlushBlocksTrim(t *testing.T) {
	t.Parallel()
	fl := &liveFlusher{live: true}
	f := newTrimFixture(t, "trim-replay-inside", fl)
	t.Cleanup(func() { f.relay.RoomDeactivated(f.room) })
	fl.set(true, func() {
		f.relay.RoomActivated(f.room)
		waitFor(t, 3*time.Second, func() bool {
			f.relay.mu.Lock()
			defer f.relay.mu.Unlock()
			return f.rs.replayed == nil
		})
	})

	f.trim(t)
	if !f.oldLeft(t) {
		t.Fatal("the stream was trimmed although a replay ran during the flush")
	}
	fl.set(true, nil)
	f.trim(t)
	if f.oldLeft(t) {
		t.Error("trimming did not resume on the next tick")
	}
}

// TestTrimStream_OtherInstanceFenceBlocksTrim: another instance loading the
// room advertises its fence, and no instance trims under it.
func TestTrimStream_OtherInstanceFenceBlocksTrim(t *testing.T) {
	t.Parallel()
	fl := &liveFlusher{live: true}
	f := newTrimFixture(t, "trim-other-fence", fl)
	ctx := context.Background()
	if err := f.client.HSet(ctx, loadsKey(f.room), "987654321", time.Now().Add(time.Minute).Unix()).Err(); err != nil {
		t.Fatalf("HSET: %v", err)
	}

	f.trim(t)
	if !f.oldLeft(t) {
		t.Fatal("the stream was trimmed under another instance's load fence")
	}
	if err := f.client.HSet(ctx, loadsKey(f.room), "987654321", time.Now().Add(-time.Second).Unix()).Err(); err != nil {
		t.Fatalf("HSET: %v", err)
	}
	f.trim(t)
	if f.oldLeft(t) {
		t.Error("an expired fence still blocked the trim")
	}
}

// TestTrimStream_NonLiveFlushBlocksTrim: a checkpoint rebuilt from storage, with
// no resident document, cannot vouch for what the reader applied.
func TestTrimStream_NonLiveFlushBlocksTrim(t *testing.T) {
	t.Parallel()
	fl := &liveFlusher{live: false}
	f := newTrimFixture(t, "trim-not-live", fl)

	f.trim(t)
	if !f.oldLeft(t) {
		t.Fatal("the stream was trimmed after a checkpoint of no resident document")
	}
	fl.set(true, nil)
	f.trim(t)
	if f.oldLeft(t) {
		t.Error("a live checkpoint did not trim")
	}
}
