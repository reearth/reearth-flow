package server

import (
	"context"
	"errors"
	"net/http"
	"net/http/httptest"
	"slices"
	"sync"
	"testing"
	"time"

	"github.com/reearth/ygo/cluster"
	"github.com/reearth/ygo/crdt"
	"github.com/reearth/ygo/persistence"
	ygws "github.com/reearth/ygo/provider/websocket"
	ygsync "github.com/reearth/ygo/sync"

	"github.com/reearth/reearth-flow/websocket-go/internal/auth"
	"github.com/reearth/reearth-flow/websocket-go/internal/versions"
)

const versionRoom = "550e8400-e29b-41d4-a716-446655440077"

// memVersions stores versions in memory and records checkpoints.
type memVersions struct {
	mu          sync.Mutex
	saved       []versions.StoredVersion
	checkpoints int
}

func (m *memVersions) Save(_ context.Context, _ string, v versions.Version) (int64, error) {
	m.mu.Lock()
	defer m.mu.Unlock()
	id := int64(len(m.saved) + 1)
	m.saved = append(m.saved, versions.StoredVersion{ID: id, Label: v.Label, CreatedAt: time.Now(), State: v.State, Attribution: v.Attribution})
	return id, nil
}

func (m *memVersions) Latest(context.Context, string) (versions.StoredVersion, bool, error) {
	m.mu.Lock()
	defer m.mu.Unlock()
	if len(m.saved) == 0 {
		return versions.StoredVersion{}, false, nil
	}
	return m.saved[len(m.saved)-1], true, nil
}

func (m *memVersions) Attribute(_ context.Context, _ string, id int64, a versions.Attribution) error {
	m.mu.Lock()
	defer m.mu.Unlock()
	m.saved[id-1].Attribution = a
	return nil
}

func (m *memVersions) checkpoint(context.Context, string) error {
	m.mu.Lock()
	defer m.mu.Unlock()
	m.checkpoints++
	return nil
}

func (m *memVersions) all() []versions.StoredVersion {
	m.mu.Lock()
	defer m.mu.Unlock()
	return slices.Clone(m.saved)
}

// insertDoc returns a document holding text and alice's record of inserting
// it, as if she had typed it.
func insertDoc(t *testing.T, text string) (*crdt.Doc, versions.Record) {
	t.Helper()
	d := crdt.New(crdt.WithClientID(900))
	txt := d.GetText("content")
	d.Transact(func(txn *crdt.Transaction) { txt.Insert(txn, 0, text, nil) })
	ids, err := crdt.ContentIDsFromUpdateV1(encodeFullState(d))
	if err != nil {
		t.Fatal(err)
	}
	now := time.Now()
	return d, versions.Record{"alice": {Inserts: ids.Inserts, Deletes: ids.Deletes, FirstAt: now, LastAt: now}}
}

// versionedServer is a Server with versions enabled over store and snaps. Its
// connections are named by the ?user= query parameter.
func versionedServer(store versions.Store, snaps *memVersions, every time.Duration) *Server {
	s := New(testConfig())
	enableTestVersions(s, store, snaps, every)
	return s
}

func enableTestVersions(s *Server, store versions.Store, snaps *memVersions, every time.Duration) {
	s.ws.Authorize = func(r *http.Request) (ygws.ConnectionConfig, bool) {
		var id any
		if u := r.URL.Query().Get("user"); u != "" {
			id = auth.Identity{UserID: u}
		}
		return ygws.ConnectionConfig{Identity: id}, true
	}
	s.EnableVersions(VersionsConfig{
		Store:      store,
		Snapshots:  snaps,
		Checkpoint: snaps.checkpoint,
		Every:      every,
		MarkEvery:  10 * time.Millisecond,
	})
}

// dialAs connects a client to room as user.
func dialAs(t *testing.T, ts *httptest.Server, user string, clientID uint64) *e2eClient {
	t.Helper()
	c := dialE2E(t, ts, versionRoom+"?user="+user, clientID)
	t.Cleanup(c.close)
	c.completeHandshake()
	return c
}

// record flushes s's gathered edits and returns room's change record, leaving
// it in place.
func record(t *testing.T, s *Server, store versions.Store) (versions.Record, bool) {
	t.Helper()
	ctx := context.Background()
	if err := s.tracker.Flush(ctx, versionRoom); err != nil {
		t.Fatalf("Flush: %v", err)
	}
	r, ok, err := store.Take(ctx, versionRoom)
	if err != nil {
		t.Fatalf("Take: %v", err)
	}
	if ok {
		if err := store.Mark(ctx, versionRoom, r); err != nil {
			t.Fatal(err)
		}
	}
	return r, ok
}

func TestVersions_UserEditIsRecordedWithItsSender(t *testing.T) {
	t.Parallel()

	store := versions.NewMemoryStore()
	s := versionedServer(store, &memVersions{}, 0)
	ts := httptest.NewServer(s.Handler())
	defer ts.Close()

	dialAs(t, ts, "alice", 101).insertText("a")
	waitFor(t, "the edit", func() bool { return serverText(s, versionRoom) == "a" })

	r, ok := record(t, s, store)
	if who, _ := r.LastEditor(); !ok || !slices.Equal(r.Contributors(), []string{"alice"}) || who != "alice" {
		t.Fatalf("record = %+v, %v", r, ok)
	}
	if r["alice"].Inserts.IsEmpty() {
		t.Fatal("the record must name the items the edit inserted")
	}
}

func TestVersions_ReconnectResendIsNoEdit(t *testing.T) {
	t.Parallel()

	ctx := context.Background()
	store := versions.NewMemoryStore()
	s := versionedServer(store, &memVersions{}, 0)
	ts := httptest.NewServer(s.Handler())
	defer ts.Close()

	a := dialAs(t, ts, "alice", 101)
	a.insertText("a")
	waitFor(t, "the edit", func() bool { return serverText(s, versionRoom) == "a" })
	if _, ok := record(t, s, store); !ok {
		t.Fatal("the edit was not recorded")
	}
	if _, _, err := store.Take(ctx, versionRoom); err != nil {
		t.Fatal(err)
	}

	// bob reconnects holding alice's edit and sends everything he holds.
	b := dialAs(t, ts, "bob", 102)
	if err := crdt.ApplyUpdateV1(b.doc, crdt.EncodeStateAsUpdateV1(a.doc, nil), nil); err != nil {
		t.Fatal(err)
	}
	b.writeSync(ygsync.EncodeUpdate(crdt.EncodeStateAsUpdateV1(b.doc, nil)))
	time.Sleep(50 * time.Millisecond)

	if r, ok := record(t, s, store); ok {
		t.Fatalf("record = %+v; re-sending what the room holds is no edit", r)
	}
}

func TestVersions_ServerWritesAreNoEdits(t *testing.T) {
	t.Parallel()

	ctx := context.Background()
	store := versions.NewMemoryStore()
	s := versionedServer(store, &memVersions{}, 0)
	ts := httptest.NewServer(s.Handler())
	defer ts.Close()
	dialAs(t, ts, "alice", 101)

	if err := s.SignalLock(ctx, versionRoom, true, time.UnixMilli(1000)); err != nil {
		t.Fatal(err)
	}
	if err := s.SignalRollback(ctx, versionRoom, true); err != nil {
		t.Fatal(err)
	}
	if r, ok := record(t, s, store); ok {
		t.Fatalf("record = %+v; a lock or rollback signal is no user edit", r)
	}
}

func TestVersions_RefusedWriteIsNoEdit(t *testing.T) {
	t.Parallel()

	store := versions.NewMemoryStore()
	s := versionedServer(store, &memVersions{}, 0)
	ts := httptest.NewServer(s.Handler())
	defer ts.Close()
	a := dialAs(t, ts, "alice", 101)

	if err := s.SignalLock(context.Background(), versionRoom, true, time.UnixMilli(1000)); err != nil {
		t.Fatal(err)
	}
	a.insertText("x")
	if code := a.closeCode(t); code != CloseProjectLocked {
		t.Fatalf("close code = %d", code)
	}
	if r, ok := record(t, s, store); ok {
		t.Fatalf("record = %+v; a refused write is no edit", r)
	}
}

func TestVersions_SaveVersionSavesEachChangeOnce(t *testing.T) {
	t.Parallel()

	ctx := context.Background()
	snaps := &memVersions{}
	s := versionedServer(versions.NewMemoryStore(), snaps, 0)
	ts := httptest.NewServer(s.Handler())
	defer ts.Close()

	dialAs(t, ts, "alice", 101).insertText("a")
	dialAs(t, ts, "bob", 102).insertText("b")
	waitFor(t, "both edits", func() bool { return len(serverText(s, versionRoom)) == 2 })

	res, err := s.SaveVersion(ctx, versionRoom, "Saved")
	if err != nil || !res.Saved {
		t.Fatalf("SaveVersion = %+v, %v", res, err)
	}
	v := snaps.all()[0]
	if v.Label != "Saved" || !slices.Equal(v.Contributors, []string{"alice", "bob"}) {
		t.Fatalf("version = %+v", v)
	}
	if snaps.checkpoints != 1 {
		t.Fatalf("checkpoints = %d, want 1", snaps.checkpoints)
	}

	if res, err := s.SaveVersion(ctx, versionRoom, "Saved"); err != nil || res.Saved {
		t.Fatalf("second SaveVersion = %+v, %v; want nothing saved", res, err)
	}
}

func TestVersions_OneVersionPerChangeAcrossInstances(t *testing.T) {
	t.Parallel()

	ctx := context.Background()
	relay := cluster.NewMemRelay()
	defer func() { _ = relay.Close() }()
	store, snaps := versions.NewMemoryStore(), &memVersions{}
	x, y := versionedServer(store, snaps, 0), versionedServer(store, snaps, 0)
	for _, s := range []*Server{x, y} {
		if err := s.AttachRedisRelay(ctx, relay); err != nil {
			t.Fatal(err)
		}
		s.SetPresence(func(context.Context, string) (int, error) { return 2, nil })
	}
	tsX, tsY := httptest.NewServer(x.Handler()), httptest.NewServer(y.Handler())
	defer tsX.Close()
	defer tsY.Close()

	dialAs(t, tsX, "alice", 101).insertText("a")
	dialAs(t, tsY, "bob", 102).insertText("b")
	waitFor(t, "both edits on both instances", func() bool {
		return len(serverText(x, versionRoom)) == 2 && len(serverText(y, versionRoom)) == 2
	})

	var wg sync.WaitGroup
	results := make([]versions.Result, 2)
	for i, s := range []*Server{x, y} {
		wg.Add(1)
		go func() {
			defer wg.Done()
			r, err := s.SaveVersion(ctx, versionRoom, "Saved")
			if err != nil {
				t.Errorf("SaveVersion: %v", err)
			}
			results[i] = r
		}()
	}
	wg.Wait()

	saved := snaps.all()
	if len(saved) != 1 {
		t.Fatalf("versions = %d, want 1 for one change seen by two instances", len(saved))
	}
	if !slices.Equal(saved[0].Contributors, []string{"alice", "bob"}) {
		t.Fatalf("contributors = %v, want both instances' editors", saved[0].Contributors)
	}
	if results[0].Saved == results[1].Saved {
		t.Fatalf("results = %+v, want exactly one saved", results)
	}
}

func TestVersions_SaveFromAnInstanceThatDoesNotHoldTheRoom(t *testing.T) {
	t.Parallel()

	ctx := context.Background()
	store, snaps := versions.NewMemoryStore(), &memVersions{}
	holder, other := versionedServer(store, snaps, 0), versionedServer(store, snaps, 0)
	other.SetPresence(func(context.Context, string) (int, error) { return 1, nil })
	// The relay's catch-up replays what the holder published.
	other.SetCatchUp(func(ctx context.Context, room string) error {
		return other.ws.Inject(ctx, cluster.Inbound{
			Room: room, Kind: cluster.KindSync, Data: encodeFullState(holder.ws.GetDoc(room)),
		})
	})
	ts := httptest.NewServer(holder.Handler())
	defer ts.Close()
	dialAs(t, ts, "alice", 101).insertText("hello")
	waitFor(t, "the edit", func() bool { return serverText(holder, versionRoom) == "hello" })
	if _, ok := record(t, holder, store); !ok {
		t.Fatal("the edit was not recorded")
	}

	res, err := other.SaveVersion(ctx, versionRoom, "Saved")
	if err != nil || !res.Saved {
		t.Fatalf("SaveVersion = %+v, %v", res, err)
	}
	d := crdt.New()
	if err := crdt.ApplyUpdateV1(d, snaps.all()[0].State, nil); err != nil {
		t.Fatal(err)
	}
	if got := d.GetText("content").ToString(); got != "hello" {
		t.Fatalf("version holds %q, want the holder's document", got)
	}
	if other.ws.GetDoc(versionRoom) != nil {
		t.Fatal("the room loaded only for the version is still open")
	}
}

func TestVersions_FailedCatchUpSavesNothing(t *testing.T) {
	t.Parallel()

	ctx := context.Background()
	store, snaps := versions.NewMemoryStore(), &memVersions{}
	s := versionedServer(store, snaps, 0)
	s.SetPresence(func(context.Context, string) (int, error) { return 1, nil })
	s.SetCatchUp(func(context.Context, string) error { return errors.New("redis down") })
	_, rec := insertDoc(t, "x")
	if err := store.Mark(ctx, versionRoom, rec); err != nil {
		t.Fatal(err)
	}

	if _, err := s.SaveVersion(ctx, versionRoom, "Saved"); err == nil {
		t.Fatal("SaveVersion saved without the holders' state")
	}
	if _, ok, _ := store.FirstEdit(ctx, versionRoom); !ok {
		t.Fatal("the change record was lost")
	}
	if len(snaps.all()) != 0 {
		t.Fatal("a version was saved")
	}
}

func TestVersions_SaveFromStoreWhenNoInstanceHoldsTheRoom(t *testing.T) {
	t.Parallel()

	ctx := context.Background()
	store, snaps := versions.NewMemoryStore(), &memVersions{}
	s := versionedServer(store, snaps, 0)
	s.SetPresence(func(context.Context, string) (int, error) { return 0, nil })
	stored, rec := insertDoc(t, "persisted")
	s.stored = func(context.Context, string) ([]byte, error) { return encodeFullState(stored), nil }
	if err := store.Mark(ctx, versionRoom, rec); err != nil {
		t.Fatal(err)
	}

	res, err := s.SaveVersion(ctx, versionRoom, "Saved")
	if err != nil || !res.Saved {
		t.Fatalf("SaveVersion = %+v, %v", res, err)
	}
	if s.ws.GetDoc(versionRoom) != nil {
		t.Fatal("saving from the store loaded the room")
	}
}

func TestVersions_PeriodicTriggerSavesAChangedRoom(t *testing.T) {
	t.Parallel()

	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	snaps := &memVersions{}
	s := versionedServer(versions.NewMemoryStore(), snaps, 100*time.Millisecond)
	go s.StartPeriodicVersions(ctx)
	ts := httptest.NewServer(s.Handler())
	defer ts.Close()

	c := dialAs(t, ts, "alice", 101)
	time.Sleep(250 * time.Millisecond)
	if n := len(snaps.all()); n != 0 {
		t.Fatalf("versions = %d before any edit, want 0", n)
	}
	c.insertText("a")
	waitFor(t, "the periodic version", func() bool { return len(snaps.all()) == 1 })
	if v := snaps.all()[0]; v.Label != versions.AutoLabel {
		t.Fatalf("label = %q, want %q", v.Label, versions.AutoLabel)
	}
	time.Sleep(250 * time.Millisecond)
	if n := len(snaps.all()); n != 1 {
		t.Fatalf("versions = %d, want 1: nothing changed since", n)
	}
}

func TestVersions_LastPeerLeavingSavesOnASingleInstance(t *testing.T) {
	t.Parallel()

	// The room is gone from memory by the time the last peer has left, so the
	// version is read from persistence, which ygo drained first.
	mem := persistence.NewMemoryPersistence()
	s := NewWithPersistence(context.Background(), testConfig(), mem)
	snaps := &memVersions{}
	enableTestVersions(s, versions.NewMemoryStore(), snaps, 0)
	s.stored = func(ctx context.Context, room string) ([]byte, error) {
		res, err := mem.Load(ctx, room)
		return res.Update, err
	}
	ts := httptest.NewServer(s.Handler())
	defer ts.Close()

	c := dialAs(t, ts, "alice", 101)
	c.insertText("a")
	waitFor(t, "the edit", func() bool { return serverText(s, versionRoom) == "a" })
	c.close()
	waitFor(t, "the version on leave", func() bool { return len(snaps.all()) == 1 })
}

type countingFlusher struct{ calls int }

func (f *countingFlusher) FlushRoom(context.Context, string) error { f.calls++; return nil }

func TestVersionedFlusher(t *testing.T) {
	t.Parallel()

	ctx := context.Background()
	store, snaps := versions.NewMemoryStore(), &memVersions{}
	s := versionedServer(store, snaps, 0)
	stored, rec := insertDoc(t, "x")
	s.stored = func(context.Context, string) ([]byte, error) { return encodeFullState(stored), nil }
	inner := &countingFlusher{}
	f := s.VersionedFlusher(inner)

	if err := f.FlushLastInstance(ctx, versionRoom); err != nil {
		t.Fatal(err)
	}
	if inner.calls != 1 || len(snaps.all()) != 0 {
		t.Fatalf("unchanged room: inner flushes %d, versions %d; want the plain flush only", inner.calls, len(snaps.all()))
	}

	if err := store.Mark(ctx, versionRoom, rec); err != nil {
		t.Fatal(err)
	}
	// A checkpoint of a live room (the Redis relay's stream trim) saves no
	// version, even of a changed room.
	if err := f.FlushRoom(ctx, versionRoom); err != nil {
		t.Fatal(err)
	}
	if len(snaps.all()) != 0 || inner.calls != 2 {
		t.Fatalf("checkpoint: versions %d, inner flushes %d; want a plain flush", len(snaps.all()), inner.calls)
	}

	if err := f.FlushLastInstance(ctx, versionRoom); err != nil {
		t.Fatal(err)
	}
	if len(snaps.all()) != 1 || snaps.checkpoints != 1 || inner.calls != 2 {
		t.Fatalf("last instance: versions %d, checkpoints %d, inner flushes %d; want the version to write the checkpoint",
			len(snaps.all()), snaps.checkpoints, inner.calls)
	}
}

// lateEdit is bob's edit made on another instance: recorded in the shared
// store, but not yet relayed to the instance that saves.
func lateEdit(t *testing.T, store versions.Store) []byte {
	t.Helper()
	bob := crdt.New(crdt.WithClientID(202))
	txt := bob.GetText("content")
	bob.Transact(func(txn *crdt.Transaction) { txt.Insert(txn, 0, "B", nil) })
	update := encodeFullState(bob)
	ids, err := crdt.ContentIDsFromUpdateV1(update)
	if err != nil {
		t.Fatal(err)
	}
	now := time.Now()
	if err := store.Mark(context.Background(), versionRoom, versions.Record{"bob": {Inserts: ids.Inserts, Deletes: ids.Deletes, FirstAt: now, LastAt: now}}); err != nil {
		t.Fatal(err)
	}
	return update
}

func TestVersions_EditNotYetRelayedIsKeptForTheNextVersion(t *testing.T) {
	t.Parallel()

	ctx := context.Background()
	store, snaps := versions.NewMemoryStore(), &memVersions{}
	s := versionedServer(store, snaps, 0)
	ts := httptest.NewServer(s.Handler())
	defer ts.Close()
	dialAs(t, ts, "alice", 101).insertText("a")
	waitFor(t, "alice's edit", func() bool { return serverText(s, versionRoom) == "a" })
	bobs := lateEdit(t, store)

	if res, err := s.SaveVersion(ctx, versionRoom, "Saved"); err != nil || !res.Saved {
		t.Fatalf("SaveVersion = %+v, %v", res, err)
	}
	if v := snaps.all()[0]; !slices.Equal(v.Contributors, []string{"alice"}) {
		t.Fatalf("contributors = %v, want alice: bob's edit is not in this version", v.Contributors)
	}
	if r, ok := record(t, s, store); !ok || !slices.Equal(r.Contributors(), []string{"bob"}) {
		t.Fatalf("record = %v, %v; bob's edit must survive the version", r, ok)
	}

	// The relay delivers bob's edit; the next version holds and credits it.
	if err := s.ws.Inject(ctx, cluster.Inbound{Room: versionRoom, Kind: cluster.KindSync, Data: bobs}); err != nil {
		t.Fatal(err)
	}
	if res, err := s.SaveVersion(ctx, versionRoom, "Saved"); err != nil || !res.Saved {
		t.Fatalf("next SaveVersion = %+v, %v", res, err)
	}
	if v := snaps.all()[1]; !slices.Equal(v.Contributors, []string{"bob"}) {
		t.Fatalf("contributors = %v, want bob", v.Contributors)
	}
}

func TestVersions_SaveWaitsForTheRelayAfterTakingTheRecord(t *testing.T) {
	t.Parallel()

	ctx := context.Background()
	store, snaps := versions.NewMemoryStore(), &memVersions{}
	s := versionedServer(store, snaps, 0)
	ts := httptest.NewServer(s.Handler())
	defer ts.Close()
	dialAs(t, ts, "alice", 101).insertText("a")
	waitFor(t, "alice's edit", func() bool { return serverText(s, versionRoom) == "a" })
	bobs := lateEdit(t, store)

	// The relay's catch-up delivers bob's edit while the save waits on it.
	s.SetCatchUp(func(ctx context.Context, room string) error {
		if _, ok, _ := store.FirstEdit(ctx, room); ok {
			t.Error("the save waited before taking the record")
		}
		return s.ws.Inject(ctx, cluster.Inbound{Room: room, Kind: cluster.KindSync, Data: bobs})
	})
	if res, err := s.SaveVersion(ctx, versionRoom, "Saved"); err != nil || !res.Saved {
		t.Fatalf("SaveVersion = %+v, %v", res, err)
	}
	if v := snaps.all()[0]; !slices.Equal(v.Contributors, []string{"alice", "bob"}) {
		t.Fatalf("contributors = %v, want bob's edit in this version", v.Contributors)
	}
}

func TestVersions_WaitVersionsCoversTheLastPeerSave(t *testing.T) {
	t.Parallel()

	store, snaps := versions.NewMemoryStore(), &memVersions{}
	s := versionedServer(store, snaps, 0)
	stored, rec := insertDoc(t, "x")
	release := make(chan struct{})
	s.stored = func(context.Context, string) ([]byte, error) {
		<-release // a slow persistence read
		return encodeFullState(stored), nil
	}
	if err := store.Mark(context.Background(), versionRoom, rec); err != nil {
		t.Fatal(err)
	}

	s.lastPeerLeft(context.Background(), versionRoom)
	short, cancel := context.WithTimeout(context.Background(), 50*time.Millisecond)
	defer cancel()
	if err := s.WaitVersions(short); err == nil {
		t.Fatal("WaitVersions returned while the version was still saving")
	}
	close(release)
	ctx, cancel2 := context.WithTimeout(context.Background(), 3*time.Second)
	defer cancel2()
	if err := s.WaitVersions(ctx); err != nil {
		t.Fatal(err)
	}
	if len(snaps.all()) != 1 {
		t.Fatal("WaitVersions returned before the version was saved")
	}
}
