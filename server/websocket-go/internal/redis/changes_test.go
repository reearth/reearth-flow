package redis

import (
	"context"
	"fmt"
	"slices"
	"sync"
	"testing"
	"time"

	"github.com/reearth/ygo/crdt"

	"github.com/reearth/reearth-flow/websocket-go/internal/versions"
)

var changesT0 = time.UnixMilli(1_700_000_000_000)

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

	c, _ := newTestClient(t)
	ctx := context.Background()
	doc := crdt.New(crdt.WithClientID(1))
	// Two instances share one Redis, so their batches land in one record.
	a, b := NewChangeStore(c, "a"), NewChangeStore(c, "b")
	if err := a.Mark(ctx, "p1", edit(t, doc, "bob", "x", ms(10))); err != nil {
		t.Fatal(err)
	}
	if err := b.Mark(ctx, "p1", edit(t, doc, "alice", "y", ms(1))); err != nil {
		t.Fatal(err)
	}
	if first, ok, err := a.FirstEdit(ctx, "p1"); err != nil || !ok || !first.Equal(ms(1)) {
		t.Fatalf("FirstEdit = %v, %v, %v; want the older batch's", first, ok, err)
	}
	r, ok, err := a.Take(ctx, "p1")
	if err != nil || !ok {
		t.Fatalf("Take = %v, %v", ok, err)
	}
	if !slices.Equal(r.Contributors(), []string{"alice", "bob"}) {
		t.Fatalf("contributors = %v", r.Contributors())
	}
	if who, at := r.LastEditor(); who != "bob" || !at.Equal(ms(10)) {
		t.Fatalf("last editor = %q at %v", who, at)
	}
	if r["bob"].Inserts.IsEmpty() {
		t.Fatal("bob's items were lost")
	}
}

func TestChangeStore_TakeGivesTheRecordToOneCaller(t *testing.T) {
	t.Parallel()

	c, _ := newTestClient(t)
	ctx := context.Background()
	s := NewChangeStore(c, "a")
	if err := s.Mark(ctx, "p1", edit(t, crdt.New(), "bob", "x", ms(1))); err != nil {
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
	if _, ok, _ := s.FirstEdit(ctx, "p1"); ok {
		t.Fatal("Take left the record behind")
	}
}

func TestChangeStore_NoRecord(t *testing.T) {
	t.Parallel()

	c, _ := newTestClient(t)
	s := NewChangeStore(c, "a")
	if _, ok, err := s.Take(context.Background(), "p1"); ok || err != nil {
		t.Fatalf("Take = %v, %v; want no record", ok, err)
	}
	if _, ok, err := s.FirstEdit(context.Background(), "p1"); ok || err != nil {
		t.Fatalf("FirstEdit = %v, %v; want no record", ok, err)
	}
}

func TestChangeStore_RecordExpires(t *testing.T) {
	t.Parallel()

	c, mr := newTestClient(t)
	s := NewChangeStore(c, "a")
	_ = s.Mark(context.Background(), "p1", edit(t, crdt.New(), "bob", "x", ms(1)))
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

func TestWaitCaughtUp_WaitsForEntriesPublishedAfterActivation(t *testing.T) {
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
	r.RoomActivated("p1")
	wctx, wcancel := context.WithTimeout(ctx, 3*time.Second)
	defer wcancel()
	if err := r.WaitCaughtUp(wctx, "p1"); err != nil {
		t.Fatal(err)
	}
	before := sink.count()

	// Another instance publishes after the initial replay.
	if err := xaddEntry(ctx, c, streamKey("p1"), msgTypeSync, []byte{9}, r.clientID+1); err != nil {
		t.Fatal(err)
	}
	if err := r.WaitCaughtUp(wctx, "p1"); err != nil {
		t.Fatal(err)
	}
	if sink.count() <= before {
		t.Fatal("WaitCaughtUp returned before the newer entry was applied")
	}
}

func TestWaitCaughtUp_ReturnsWhenTheReaderStops(t *testing.T) {
	t.Parallel()

	c, mr := newTestClient(t)
	ctx, cancel := context.WithCancel(context.Background())
	r := mustRelay(t, mr.Addr())
	defer r.Close()
	if err := r.Start(ctx, &fakeSink{}); err != nil {
		t.Fatalf("Start: %v", err)
	}
	r.RoomActivated("p1")
	cancel() // the reader stops; the room stays registered
	r.mu.Lock()
	done := r.rooms["p1"].done
	r.mu.Unlock()
	<-done
	// An entry the stopped reader will never apply.
	if err := xaddEntry(context.Background(), c, streamKey("p1"), msgTypeSync, []byte{9}, r.clientID+1); err != nil {
		t.Fatal(err)
	}

	wctx, wcancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer wcancel()
	start := time.Now()
	if err := r.WaitCaughtUp(wctx, "p1"); err != ErrRoomNotActive {
		t.Fatalf("WaitCaughtUp = %v, want ErrRoomNotActive", err)
	}
	if time.Since(start) > time.Second {
		t.Fatal("WaitCaughtUp waited although the reader had stopped")
	}
}

func TestStreamIDBefore(t *testing.T) {
	t.Parallel()

	if !streamIDBefore("0", "1-0") {
		t.Error("the reader's initial cursor sorts first")
	}
	if !streamIDBefore("5-9", "6-0") || !streamIDBefore("5-1", "5-2") {
		t.Error("ids order by time, then sequence")
	}
	if streamIDBefore("5-2", "5-2") || streamIDBefore("10-0", "9-99") {
		t.Error("an id is not before itself or an earlier one")
	}
}

func TestChangeStore_OneEntryPerInstanceHoweverManyMarks(t *testing.T) {
	t.Parallel()

	c, _ := newTestClient(t)
	ctx := context.Background()
	doc := crdt.New(crdt.WithClientID(1))
	a, b := NewChangeStore(c, "a"), NewChangeStore(c, "b")
	for i := range 50 {
		if err := a.Mark(ctx, "p1", edit(t, doc, "alice", "x", ms(i))); err != nil {
			t.Fatal(err)
		}
	}
	if err := b.Mark(ctx, "p1", edit(t, doc, "bob", "y", ms(100))); err != nil {
		t.Fatal(err)
	}
	if n := c.HLen(ctx, changesKey("p1")).Val() - 1; n != 2 { // less "first"
		t.Fatalf("%d entries, want one per instance", n)
	}
	r, _, err := a.Take(ctx, "p1")
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

	c, _ := newTestClient(t)
	ctx := context.Background()
	doc := crdt.New(crdt.WithClientID(1))
	s := NewChangeStore(c, "a")
	if err := s.Mark(ctx, "p1", edit(t, doc, "alice", "x", ms(1))); err != nil {
		t.Fatal(err)
	}
	if _, ok, _ := s.Take(ctx, "p1"); !ok {
		t.Fatal("no record")
	}
	if err := s.Mark(ctx, "p1", edit(t, doc, "bob", "y", ms(2))); err != nil {
		t.Fatal(err)
	}
	r, _, _ := s.Take(ctx, "p1")
	if !slices.Equal(r.Contributors(), []string{"bob"}) {
		t.Fatalf("contributors = %v; alice's edit went to the earlier version", r.Contributors())
	}
}

func TestChangeStore_MarksRacingTakesLoseNothing(t *testing.T) {
	t.Parallel()

	c, _ := newTestClient(t)
	ctx := context.Background()
	s := NewChangeStore(c, "a")
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
			r, _, err := s.Take(ctx, "p1")
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
			if err := s.Mark(ctx, "p1", edit(t, doc, fmt.Sprintf("u%02d", i), "x", ms(i))); err != nil {
				t.Errorf("Mark: %v", err)
			}
		}()
	}
	wg.Wait()
	close(stop)
	<-takerDone
	r, _, err := s.Take(ctx, "p1")
	if err != nil {
		t.Fatal(err)
	}
	keep(r)
	if len(taken) != users {
		t.Fatalf("%d of %d users' edits were taken; the rest were lost", len(taken), users)
	}
}
