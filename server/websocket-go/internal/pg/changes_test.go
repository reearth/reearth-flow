package pg

import (
	"context"
	"slices"
	"sync"
	"testing"
	"time"

	"github.com/reearth/ygo/cluster"
	"github.com/reearth/ygo/crdt"

	"github.com/reearth/reearth-flow/websocket-go/internal/versions"
)

var changesT0 = time.UnixMilli(1_700_000_000_000).UTC()

func ms(n int) time.Time { return changesT0.Add(time.Duration(n) * time.Millisecond) }

// edit returns user's batch inserting s at client's next clock.
func edit(t *testing.T, doc *crdt.Doc, user, s string, at time.Time) versions.Record {
	t.Helper()
	txt := doc.GetText("t")
	before := doc.StateVector()
	doc.Transact(func(txn *crdt.Transaction) { txt.Insert(txn, txt.Len(), s, nil) })
	ids, err := crdt.ContentIDsFromUpdateV1(crdt.EncodeStateAsUpdateV1(doc, before))
	if err != nil {
		t.Fatal(err)
	}
	return versions.Record{user: {Inserts: ids.Inserts, Deletes: ids.Deletes, FirstAt: at, LastAt: at}}
}

func TestChangeStore_MarkMergesAcrossInstances(t *testing.T) {
	t.Parallel()

	pool := newPool(t)
	ctx := context.Background()
	doc := crdt.New(crdt.WithClientID(1))
	a, b := NewChangeStore(pool), NewChangeStore(pool)
	if err := a.Mark(ctx, room, edit(t, doc, "bob", "x", ms(10))); err != nil {
		t.Fatal(err)
	}
	if err := b.Mark(ctx, room, edit(t, doc, "alice", "y", ms(1))); err != nil {
		t.Fatal(err)
	}
	if first, ok, err := a.FirstEdit(ctx, room); err != nil || !ok || !first.Equal(ms(1)) {
		t.Fatalf("FirstEdit = %v, %v, %v; want the older batch's", first, ok, err)
	}
	r, ok, err := a.Take(ctx, room)
	if err != nil || !ok {
		t.Fatalf("Take = %v, %v", ok, err)
	}
	if !slices.Equal(r.Contributors(), []string{"alice", "bob"}) {
		t.Fatalf("contributors = %v", r.Contributors())
	}
	if who, at := r.LastEditor(); who != "bob" || !at.Equal(ms(10)) {
		t.Fatalf("last editor = %q at %v", who, at)
	}
}

func TestChangeStore_TakeGivesTheRecordToOneCaller(t *testing.T) {
	t.Parallel()

	pool := newPool(t)
	ctx := context.Background()
	s := NewChangeStore(pool)
	if err := s.Mark(ctx, room, edit(t, crdt.New(), "bob", "x", ms(1))); err != nil {
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
	if _, ok, _ := s.FirstEdit(ctx, room); ok {
		t.Fatal("Take left the record behind")
	}
}

func TestChangeStore_NoRecord(t *testing.T) {
	t.Parallel()

	s := NewChangeStore(newPool(t))
	if _, ok, err := s.Take(context.Background(), room); ok || err != nil {
		t.Fatalf("Take = %v, %v", ok, err)
	}
	if _, ok, err := s.FirstEdit(context.Background(), room); ok || err != nil {
		t.Fatalf("FirstEdit = %v, %v", ok, err)
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

func TestWaitCaughtUp_WaitsForRowsWrittenAfterActivation(t *testing.T) {
	t.Parallel()

	pool := newPool(t)
	sink := &fakeSink{}
	r := startRelay(t, pool, sink, nil)
	other := startRelay(t, pool, &fakeSink{}, nil)
	r.RoomActivated(room)
	other.RoomActivated(room)
	ctx, cancel := context.WithTimeout(context.Background(), 3*time.Second)
	defer cancel()
	if err := r.WaitCaughtUp(ctx, room); err != nil {
		t.Fatal(err)
	}
	if err := other.Publish(ctx, cluster.Outbound{Room: room, Kind: cluster.KindSync, Data: []byte("later")}); err != nil {
		t.Fatal(err)
	}
	eventually(t, 3*time.Second, "the write to land", func() bool { return rowCount(t, pool, room) >= 1 })
	if err := r.WaitCaughtUp(ctx, room); err != nil {
		t.Fatal(err)
	}
	sink.mu.Lock()
	got := slices.ContainsFunc(sink.got, func(in cluster.Inbound) bool { return string(in.Data) == "later" })
	sink.mu.Unlock()
	if !got {
		t.Fatal("WaitCaughtUp returned before the newer row was applied")
	}
}
