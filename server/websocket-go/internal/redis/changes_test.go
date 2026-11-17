package redis

import (
	"context"
	"slices"
	"sync"
	"testing"
	"time"

	"github.com/reearth/reearth-flow/websocket-go/internal/versions"
)

var changesT0 = time.UnixMilli(1_700_000_000_000)

func ms(n int) time.Time { return changesT0.Add(time.Duration(n) * time.Millisecond) }

func TestChangeStore_MarkMergesAcrossInstances(t *testing.T) {
	t.Parallel()

	c, _ := newTestClient(t)
	ctx := context.Background()
	// Two instances share one Redis, so their batches land in one record.
	a, b := NewChangeStore(c), NewChangeStore(c)
	if err := a.Mark(ctx, "p1", versions.Mark{Editors: []string{"bob", ""}, LastEditor: "bob", LastEditorAt: ms(10), FirstAt: ms(5), LastAt: ms(10)}); err != nil {
		t.Fatal(err)
	}
	if err := b.Mark(ctx, "p1", versions.Mark{Editors: []string{"alice"}, LastEditor: "alice", LastEditorAt: ms(7), FirstAt: ms(1), LastAt: ms(7)}); err != nil {
		t.Fatal(err)
	}
	r, ok, err := a.Peek(ctx, "p1")
	if err != nil || !ok {
		t.Fatalf("Peek = %v, %v", ok, err)
	}
	if !slices.Equal(r.Contributors, []string{"alice", "bob"}) {
		t.Fatalf("contributors = %v", r.Contributors)
	}
	if r.LastEditor != "bob" || !r.LastEditorAt.Equal(ms(10)) {
		t.Fatalf("last editor = %q at %v, want bob: alice's batch was older", r.LastEditor, r.LastEditorAt)
	}
	if !r.FirstEditAt.Equal(ms(1)) || !r.LastEditAt.Equal(ms(10)) {
		t.Fatalf("first/last = %v/%v", r.FirstEditAt, r.LastEditAt)
	}
}

func TestChangeStore_EditWithoutSenderKeepsTheLastEditor(t *testing.T) {
	t.Parallel()

	c, _ := newTestClient(t)
	ctx := context.Background()
	s := NewChangeStore(c)
	_ = s.Mark(ctx, "p1", versions.Mark{Editors: []string{"bob"}, LastEditor: "bob", LastEditorAt: ms(1), FirstAt: ms(1), LastAt: ms(1)})
	_ = s.Mark(ctx, "p1", versions.Mark{FirstAt: ms(2), LastAt: ms(2)})
	r, _, _ := s.Peek(ctx, "p1")
	if r.LastEditor != "bob" || !r.LastEditAt.Equal(ms(2)) {
		t.Fatalf("record = %+v", r)
	}
}

func TestChangeStore_TakeGivesTheRecordToOneCaller(t *testing.T) {
	t.Parallel()

	c, _ := newTestClient(t)
	ctx := context.Background()
	s := NewChangeStore(c)
	if err := s.Mark(ctx, "p1", versions.Mark{Editors: []string{"bob"}, FirstAt: ms(1), LastAt: ms(1)}); err != nil {
		t.Fatal(err)
	}

	var (
		wg   sync.WaitGroup
		mu   sync.Mutex
		gots int
	)
	for range 8 {
		wg.Add(1)
		go func() {
			defer wg.Done()
			_, ok, err := s.Take(ctx, "p1")
			if err != nil {
				t.Errorf("Take: %v", err)
			}
			if ok {
				mu.Lock()
				gots++
				mu.Unlock()
			}
		}()
	}
	wg.Wait()
	if gots != 1 {
		t.Fatalf("%d callers got the record, want exactly 1", gots)
	}
	if _, ok, _ := s.Peek(ctx, "p1"); ok {
		t.Fatal("Take left the record behind")
	}
}

func TestChangeStore_NoRecord(t *testing.T) {
	t.Parallel()

	c, _ := newTestClient(t)
	s := NewChangeStore(c)
	if _, ok, err := s.Take(context.Background(), "p1"); ok || err != nil {
		t.Fatalf("Take = %v, %v; want no record", ok, err)
	}
	if _, ok, err := s.Peek(context.Background(), "p1"); ok || err != nil {
		t.Fatalf("Peek = %v, %v; want no record", ok, err)
	}
}

func TestChangeStore_RecordExpires(t *testing.T) {
	t.Parallel()

	c, mr := newTestClient(t)
	s := NewChangeStore(c)
	_ = s.Mark(context.Background(), "p1", versions.Mark{FirstAt: ms(1), LastAt: ms(1)})
	if ttl := mr.TTL(changesKey("p1")); ttl <= 0 || ttl > changesTTLSeconds*time.Second {
		t.Fatalf("ttl = %v, want up to %ds", ttl, changesTTLSeconds)
	}
}

func TestWaitCaughtUp(t *testing.T) {
	t.Parallel()

	c, mr := newTestClient(t)
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	r := mustRelay(t, mr.Addr())
	defer r.Close()
	sink := &fakeSink{}
	if err := r.Start(ctx, sink); err != nil {
		t.Fatalf("Start: %v", err)
	}
	// Another instance published an update before this one held the room.
	if err := xaddEntry(ctx, c, streamKey("p1"), msgTypeSync, []byte{1, 2, 3}, r.clientID+1); err != nil {
		t.Fatalf("xadd: %v", err)
	}

	if err := r.WaitCaughtUp(ctx, "p1"); err != ErrRoomNotActive {
		t.Fatalf("WaitCaughtUp before activation = %v, want ErrRoomNotActive", err)
	}
	r.RoomActivated("p1")
	wctx, wcancel := context.WithTimeout(ctx, 3*time.Second)
	defer wcancel()
	if err := r.WaitCaughtUp(wctx, "p1"); err != nil {
		t.Fatalf("WaitCaughtUp = %v", err)
	}
	if sink.count() == 0 {
		t.Fatal("WaitCaughtUp returned before the earlier update was replayed")
	}
}

// lastInstanceFlusher records which seam the relay used.
type lastInstanceFlusher struct {
	mu                 sync.Mutex
	checkpoints, lasts int
}

func (f *lastInstanceFlusher) FlushRoom(context.Context, string) error {
	f.mu.Lock()
	defer f.mu.Unlock()
	f.checkpoints++
	return nil
}

func (f *lastInstanceFlusher) FlushLastInstance(context.Context, string) error {
	f.mu.Lock()
	defer f.mu.Unlock()
	f.lasts++
	return nil
}

func TestEvictUsesTheLastInstanceFlush(t *testing.T) {
	t.Parallel()

	_, mr := newTestClient(t)
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	fl := &lastInstanceFlusher{}
	r := mustRelay(t, mr.Addr())
	defer r.Close()
	r.flusher = fl
	if err := r.Start(ctx, &fakeSink{}); err != nil {
		t.Fatalf("Start: %v", err)
	}
	r.RoomActivated("p1")
	r.RoomDeactivated("p1")

	fl.mu.Lock()
	defer fl.mu.Unlock()
	if fl.lasts != 1 || fl.checkpoints != 0 {
		t.Fatalf("last-instance flushes %d, checkpoints %d; want the last-instance flush only", fl.lasts, fl.checkpoints)
	}
}
