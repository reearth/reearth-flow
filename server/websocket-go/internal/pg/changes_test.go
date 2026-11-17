package pg

import (
	"context"
	"slices"
	"sync"
	"testing"
	"time"

	"github.com/reearth/reearth-flow/websocket-go/internal/versions"
)

var changesT0 = time.UnixMilli(1_700_000_000_000).UTC()

func ms(n int) time.Time { return changesT0.Add(time.Duration(n) * time.Millisecond) }

func TestChangeStore_MarkMergesAcrossInstances(t *testing.T) {
	t.Parallel()

	pool := newPool(t)
	ctx := context.Background()
	a, b := NewChangeStore(pool), NewChangeStore(pool)
	if err := a.Mark(ctx, room, versions.Mark{Editors: []string{"bob", ""}, LastEditor: "bob", LastEditorAt: ms(10), FirstAt: ms(5), LastAt: ms(10)}); err != nil {
		t.Fatal(err)
	}
	if err := b.Mark(ctx, room, versions.Mark{Editors: []string{"alice", "bob"}, LastEditor: "alice", LastEditorAt: ms(7), FirstAt: ms(1), LastAt: ms(7)}); err != nil {
		t.Fatal(err)
	}
	r, ok, err := a.Peek(ctx, room)
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

	pool := newPool(t)
	ctx := context.Background()
	s := NewChangeStore(pool)
	if err := s.Mark(ctx, room, versions.Mark{FirstAt: ms(0), LastAt: ms(0)}); err != nil {
		t.Fatal(err)
	}
	if r, _, _ := s.Peek(ctx, room); r.LastEditor != "" || !r.LastEditorAt.IsZero() || r.Contributors != nil {
		t.Fatalf("record without senders = %+v", r)
	}
	_ = s.Mark(ctx, room, versions.Mark{Editors: []string{"bob"}, LastEditor: "bob", LastEditorAt: ms(1), FirstAt: ms(1), LastAt: ms(1)})
	_ = s.Mark(ctx, room, versions.Mark{FirstAt: ms(2), LastAt: ms(2)})
	r, _, _ := s.Peek(ctx, room)
	if r.LastEditor != "bob" || !r.LastEditAt.Equal(ms(2)) {
		t.Fatalf("record = %+v", r)
	}
}

func TestChangeStore_TakeGivesTheRecordToOneCaller(t *testing.T) {
	t.Parallel()

	pool := newPool(t)
	ctx := context.Background()
	s := NewChangeStore(pool)
	if err := s.Mark(ctx, room, versions.Mark{Editors: []string{"bob"}, FirstAt: ms(1), LastAt: ms(1)}); err != nil {
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
			_, ok, err := s.Take(ctx, room)
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
	if _, ok, _ := s.Peek(ctx, room); ok {
		t.Fatal("Take left the record behind")
	}
}

func TestWaitCaughtUp(t *testing.T) {
	t.Parallel()

	pool := newPool(t)
	r := startRelay(t, pool, &fakeSink{}, nil)
	ctx, cancel := context.WithTimeout(context.Background(), 3*time.Second)
	defer cancel()
	if err := r.WaitCaughtUp(ctx, room); err != ErrRoomNotActive {
		t.Fatalf("WaitCaughtUp before activation = %v, want ErrRoomNotActive", err)
	}
	r.RoomActivated(room)
	if err := r.WaitCaughtUp(ctx, room); err != nil {
		t.Fatalf("WaitCaughtUp = %v", err)
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

	pool := newPool(t)
	fl := &lastInstanceFlusher{}
	r := startRelay(t, pool, &fakeSink{}, fl)
	r.RoomActivated(room)
	r.RoomDeactivated(room)

	fl.mu.Lock()
	defer fl.mu.Unlock()
	if fl.lasts != 1 || fl.checkpoints != 0 {
		t.Fatalf("last-instance flushes %d, checkpoints %d; want the last-instance flush only", fl.lasts, fl.checkpoints)
	}
}
