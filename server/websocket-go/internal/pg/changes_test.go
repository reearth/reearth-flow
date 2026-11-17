package pg

import (
	"context"
	"fmt"
	"slices"
	"sync"
	"testing"
	"time"

	"github.com/jackc/pgx/v5/pgxpool"
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
	a, b := NewChangeStore(pool, "a"), NewChangeStore(pool, "b")
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
	s := NewChangeStore(pool, "a")
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

	s := NewChangeStore(newPool(t), "a")
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

func TestChangeStore_OneEntryPerInstanceHoweverManyMarks(t *testing.T) {
	t.Parallel()

	pool := newPool(t)
	ctx := context.Background()
	doc := crdt.New(crdt.WithClientID(1))
	a, b := NewChangeStore(pool, "a"), NewChangeStore(pool, "b")
	for i := range 50 {
		if err := a.Mark(ctx, room, edit(t, doc, "alice", "x", ms(i))); err != nil {
			t.Fatal(err)
		}
	}
	if err := b.Mark(ctx, room, edit(t, doc, "bob", "y", ms(100))); err != nil {
		t.Fatal(err)
	}
	if n := changeRows(t, pool); n != 2 {
		t.Fatalf("%d entries, want one per instance", n)
	}
	r, _, err := a.Take(ctx, room)
	if err != nil {
		t.Fatal(err)
	}
	if !slices.Equal(r.Contributors(), []string{"alice", "bob"}) || len(r["alice"].Inserts.Ranges(1)) != 1 {
		t.Fatalf("record = %+v; want alice's 50 edits merged", r)
	}
	if first := r["alice"].FirstAt; !first.Equal(ms(0)) {
		t.Fatalf("alice's first edit = %v, want the oldest", first)
	}
}

func TestChangeStore_MarkAfterTakeDoesNotRestoreTakenEdits(t *testing.T) {
	t.Parallel()

	pool := newPool(t)
	ctx := context.Background()
	doc := crdt.New(crdt.WithClientID(1))
	s := NewChangeStore(pool, "a")
	if err := s.Mark(ctx, room, edit(t, doc, "alice", "x", ms(1))); err != nil {
		t.Fatal(err)
	}
	if _, ok, _ := s.Take(ctx, room); !ok {
		t.Fatal("no record")
	}
	if err := s.Mark(ctx, room, edit(t, doc, "bob", "y", ms(2))); err != nil {
		t.Fatal(err)
	}
	r, _, _ := s.Take(ctx, room)
	if !slices.Equal(r.Contributors(), []string{"bob"}) {
		t.Fatalf("contributors = %v; alice's edit went to the earlier version", r.Contributors())
	}
}

func TestChangeStore_MarksRacingTakesLoseNothing(t *testing.T) {
	t.Parallel()

	pool := newPool(t)
	ctx := context.Background()
	s := NewChangeStore(pool, "a")
	const users = 40
	var (
		wg    sync.WaitGroup
		mu    sync.Mutex
		taken = map[string]bool{}
	)
	keep := func(r versions.Record) {
		mu.Lock()
		defer mu.Unlock()
		for u := range r {
			taken[u] = true
		}
	}
	stop, takerDone := make(chan struct{}), make(chan struct{})
	go func() {
		defer close(takerDone)
		for {
			select {
			case <-stop:
				return
			default:
			}
			r, _, err := s.Take(ctx, room)
			if err != nil {
				t.Errorf("Take: %v", err)
				return
			}
			keep(r)
		}
	}()
	for i := range users {
		wg.Add(1)
		go func() {
			defer wg.Done()
			doc := crdt.New(crdt.WithClientID(crdt.ClientID(i + 1)))
			if err := s.Mark(ctx, room, edit(t, doc, fmt.Sprintf("u%02d", i), "x", ms(i))); err != nil {
				t.Errorf("Mark: %v", err)
			}
		}()
	}
	wg.Wait()
	close(stop)
	<-takerDone
	r, _, err := s.Take(ctx, room)
	if err != nil {
		t.Fatal(err)
	}
	keep(r)
	if len(taken) != users {
		t.Fatalf("%d of %d users' edits were taken; the rest were lost", len(taken), users)
	}
}

func changeRows(t *testing.T, pool *pgxpool.Pool) int {
	t.Helper()
	var n int
	if err := pool.QueryRow(context.Background(), `SELECT count(*) FROM ws_version_change WHERE doc_id = $1`, room).Scan(&n); err != nil {
		t.Fatal(err)
	}
	return n
}

func TestWaitCaughtUp_NotifyOnlyReaderWithReadLag(t *testing.T) {
	t.Parallel()

	pool := newPool(t)
	sink := &fakeSink{}
	// No polling, and rows are read only once 300ms old: the commit notice
	// wakes the reader while the row is still too young.
	r := startRelayWith(t, pool, sink, nil, Options{Notify: true, ReadLag: 300 * time.Millisecond})
	other := startRelay(t, pool, &fakeSink{}, nil)
	r.RoomActivated(room)
	other.RoomActivated(room)
	ctx, cancel := context.WithTimeout(context.Background(), 3*time.Second)
	defer cancel()
	if err := r.WaitCaughtUp(ctx, room); err != nil {
		t.Fatal(err)
	}
	if err := other.Publish(ctx, cluster.Outbound{Room: room, Kind: cluster.KindSync, Data: []byte("young")}); err != nil {
		t.Fatal(err)
	}
	eventually(t, 3*time.Second, "the write to land", func() bool { return rowCount(t, pool, room) >= 1 })
	if err := r.WaitCaughtUp(ctx, room); err != nil {
		t.Fatalf("WaitCaughtUp = %v; the reader must be woken again once the row is old enough", err)
	}
	sink.mu.Lock()
	defer sink.mu.Unlock()
	if !slices.ContainsFunc(sink.got, func(in cluster.Inbound) bool { return string(in.Data) == "young" }) {
		t.Fatal("WaitCaughtUp returned before the row was applied")
	}
}
