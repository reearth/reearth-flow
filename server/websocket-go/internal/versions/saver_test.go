package versions

import (
	"context"
	"errors"
	"slices"
	"sync"
	"sync/atomic"
	"testing"
	"time"
)

// memSnapshots is an in-memory Snapshots.
type memSnapshots struct {
	mu          sync.Mutex
	saved       []StoredVersion
	fail        error
	failLatest  error
	failAttribs error
}

func (m *memSnapshots) Save(_ context.Context, _ string, v Version) (int64, error) {
	m.mu.Lock()
	defer m.mu.Unlock()
	if m.fail != nil {
		return 0, m.fail
	}
	id := int64(len(m.saved) + 1)
	m.saved = append(m.saved, StoredVersion{ID: id, Label: v.Label, CreatedAt: at(1000 * int(id)), State: v.State, Attribution: v.Attribution})
	return id, nil
}

func (m *memSnapshots) Latest(context.Context, string) (StoredVersion, bool, error) {
	m.mu.Lock()
	defer m.mu.Unlock()
	if m.failLatest != nil {
		return StoredVersion{}, false, m.failLatest
	}
	if len(m.saved) == 0 {
		return StoredVersion{}, false, nil
	}
	return m.saved[len(m.saved)-1], true, nil
}

func (m *memSnapshots) Attribute(_ context.Context, _ string, id int64, a Attribution) error {
	m.mu.Lock()
	defer m.mu.Unlock()
	if m.failAttribs != nil {
		return m.failAttribs
	}
	m.saved[id-1].Attribution = a
	return nil
}

func (m *memSnapshots) all() []StoredVersion {
	m.mu.Lock()
	defer m.mu.Unlock()
	return slices.Clone(m.saved)
}

type savedLog struct {
	mu  sync.Mutex
	all []Saved
}

func (l *savedLog) notify(s Saved) { l.mu.Lock(); l.all = append(l.all, s); l.mu.Unlock() }

func (l *savedLog) list() []Saved {
	l.mu.Lock()
	defer l.mu.Unlock()
	return slices.Clone(l.all)
}

func newSaver() (*Saver, *MemoryStore, *memSnapshots, *savedLog) {
	store := NewMemoryStore()
	snaps := &memSnapshots{}
	notes := &savedLog{}
	return &Saver{Store: store, Snapshots: snaps, Notify: notes.notify, Now: func() time.Time { return at(99_000) }}, store, snaps, notes
}

// mark records user's update, as the Tracker would.
func mark(t *testing.T, store Store, room, user string, update []byte, ms int) {
	t.Helper()
	if err := store.Mark(context.Background(), room, batch(t, user, update, ms)); err != nil {
		t.Fatal(err)
	}
}

func TestSave_NothingChangedSavesNothing(t *testing.T) {
	t.Parallel()

	s, _, snaps, notes := newSaver()
	doc := newTestDoc(1)
	doc.write("x")
	res, err := s.Save(context.Background(), "p1", "Saved", doc.state)
	if err != nil || res.Saved || len(snaps.all()) != 0 || len(notes.list()) != 0 {
		t.Fatalf("Save = %+v, %v; want nothing saved without a change record", res, err)
	}
}

func TestSave_SavesTheChangeOnceWithItsContributors(t *testing.T) {
	t.Parallel()

	ctx := context.Background()
	s, store, snaps, notes := newSaver()
	checkpoints := 0
	s.Checkpoint = func(context.Context, string) error { checkpoints++; return nil }
	doc := newTestDoc(1)
	mark(t, store, "p1:main", "alice", doc.write("a"), 1)
	mark(t, store, "p1:main", "bob", doc.write("b"), 2)

	res, err := s.Save(ctx, "p1:main", "Saved", doc.state)
	if err != nil || !res.Saved || res.SnapshotID != 1 {
		t.Fatalf("Save = %+v, %v", res, err)
	}
	v := snaps.all()[0]
	if v.Label != "Saved" || !slices.Equal(v.Contributors, []string{"alice", "bob"}) || v.LastEditor != "bob" || !v.LastEditedAt.Equal(at(2)) {
		t.Fatalf("version = %+v", v)
	}
	if checkpoints != 1 {
		t.Fatalf("checkpoints = %d", checkpoints)
	}
	n := notes.list()
	if len(n) != 1 || n[0].ProjectID != "p1" || n[0].VersionID != 1 || n[0].LastEditorID != "bob" ||
		!slices.Equal(n[0].ContributorIDs, []string{"alice", "bob"}) || !n[0].LastEditedAt.Equal(at(2)) || !n[0].SavedAt.Equal(at(99_000)) {
		t.Fatalf("notices = %+v", n)
	}

	if res, err := s.Save(ctx, "p1:main", "Saved", doc.state); err != nil || res.Saved {
		t.Fatalf("second Save = %+v, %v; want nothing: the record was taken", res, err)
	}
}

// TestSave_EditNotYetInTheStateWaitsForTheNextVersion is the case where
// another instance recorded an edit that has not reached the copy saved from.
func TestSave_EditNotYetInTheStateWaitsForTheNextVersion(t *testing.T) {
	t.Parallel()

	ctx := context.Background()
	s, store, snaps, _ := newSaver()
	alice, bob, server := newTestDoc(1), newTestDoc(2), newTestDoc(3)
	ua := alice.write("a")
	ub := bob.write("b")
	server.apply(t, ua)
	mark(t, store, "p1", "alice", ua, 1)
	mark(t, store, "p1", "bob", ub, 2) // recorded, not yet on this copy

	if res, err := s.Save(ctx, "p1", "Saved", server.state); err != nil || !res.Saved {
		t.Fatalf("Save = %+v, %v", res, err)
	}
	if v := snaps.all()[0]; !slices.Equal(v.Contributors, []string{"alice"}) || v.LastEditor != "alice" {
		t.Fatalf("version = %+v, want alice only: bob's edit is not in it", v.Attribution)
	}
	r, ok, _ := store.Take(ctx, "p1")
	if !ok || !slices.Equal(r.Contributors(), []string{"bob"}) {
		t.Fatalf("record = %v, %v; want bob's edit put back", r, ok)
	}
	if err := store.Mark(ctx, "p1", r); err != nil {
		t.Fatal(err)
	}

	server.apply(t, ub)
	if res, err := s.Save(ctx, "p1", "Saved", server.state); err != nil || !res.Saved {
		t.Fatalf("next Save = %+v, %v", res, err)
	}
	if v := snaps.all()[1]; !slices.Equal(v.Contributors, []string{"bob"}) {
		t.Fatalf("next version = %+v, want bob", v.Attribution)
	}
}

func TestSave_NoHeldEditSavesNothing(t *testing.T) {
	t.Parallel()

	ctx := context.Background()
	s, store, snaps, _ := newSaver()
	bob, server := newTestDoc(2), newTestDoc(3)
	server.write("server write")
	mark(t, store, "p1", "bob", bob.write("b"), 1)

	if res, err := s.Save(ctx, "p1", "Saved", server.state); err != nil || res.Saved {
		t.Fatalf("Save = %+v, %v; want nothing: the state holds no recorded edit", res, err)
	}
	if len(snaps.all()) != 0 {
		t.Fatal("a version was saved")
	}
	if _, ok, _ := store.FirstEdit(ctx, "p1"); !ok {
		t.Fatal("bob's edit was dropped")
	}
}

// TestSave_EditDuringSaveIsCreditedToTheVersionHoldingIt: bob's edit lands
// after the record was taken but before the state is read, so the version
// holds it without naming him; the next save credits him to that version.
func TestSave_EditDuringSaveIsCreditedToTheVersionHoldingIt(t *testing.T) {
	t.Parallel()

	ctx := context.Background()
	s, store, snaps, notes := newSaver()
	doc := newTestDoc(1)
	mark(t, store, "p1", "alice", doc.write("a"), 1)

	state := func(ctx context.Context) ([]byte, error) {
		mark(t, store, "p1", "bob", doc.write("b"), 5)
		return doc.state(ctx)
	}
	if res, err := s.Save(ctx, "p1", "Saved", state); err != nil || !res.Saved {
		t.Fatalf("Save = %+v, %v", res, err)
	}

	res, err := s.Save(ctx, "p1", "auto", doc.state)
	if err != nil || res.Saved {
		t.Fatalf("next Save = %+v, %v; want no new version: nothing changed since", res, err)
	}
	v := snaps.all()
	if len(v) != 1 || !slices.Equal(v[0].Contributors, []string{"alice", "bob"}) || v[0].LastEditor != "bob" || !v[0].LastEditedAt.Equal(at(5)) {
		t.Fatalf("versions = %+v, want bob credited to the one version", v)
	}
	n := notes.list()
	if len(n) != 2 || n[1].VersionID != 1 || n[1].Label != "Saved" || n[1].LastEditorID != "bob" || !n[1].LastEditedAt.Equal(at(5)) {
		t.Fatalf("notices = %+v, want the version's notice sent again with bob", n)
	}
	if _, ok, _ := store.FirstEdit(ctx, "p1"); ok {
		t.Fatal("the credited record must be taken")
	}
}

func TestSave_CreditedEditOlderThanTheVersionKeepsItsEditTime(t *testing.T) {
	t.Parallel()

	ctx := context.Background()
	s, store, snaps, notes := newSaver()
	doc := newTestDoc(1)
	mark(t, store, "p1", "alice", doc.write("a"), 10)
	// bob's edit was made before alice's, but its record arrives during the save.
	state := func(ctx context.Context) ([]byte, error) {
		mark(t, store, "p1", "bob", doc.write("b"), 3)
		return doc.state(ctx)
	}
	if res, _ := s.Save(ctx, "p1", "Saved", state); !res.Saved {
		t.Fatal("first version not saved")
	}
	if res, err := s.Save(ctx, "p1", "auto", doc.state); err != nil || res.Saved {
		t.Fatalf("Save = %+v, %v", res, err)
	}
	if v := snaps.all(); !slices.Equal(v[0].Contributors, []string{"alice", "bob"}) || v[0].LastEditor != "alice" {
		t.Fatalf("version = %+v", v[0])
	}
	// What the callback contract has to handle: the second notice has more
	// contributors and the same lastEditedAt.
	n := notes.list()
	if len(n) != 2 || !n[1].LastEditedAt.Equal(n[0].LastEditedAt) || len(n[1].ContributorIDs) != 2 {
		t.Fatalf("notices = %+v", n)
	}
}

func TestSave_CreditKeepsTheCountOfACutList(t *testing.T) {
	t.Parallel()

	ctx := context.Background()
	s, store, snaps, notes := newSaver()
	doc := newTestDoc(1)
	doc.write("a")
	ub := doc.write("b")
	st, _ := doc.state(ctx)
	// The stored list was cut to fit: 400 contributors, one listed.
	snaps.saved = []StoredVersion{{ID: 1, Label: "auto", State: st, Attribution: Attribution{
		Contributors: []string{"alice"}, LastEditor: "alice", LastEditedAt: at(10), ContributorCount: 400,
	}}}
	mark(t, store, "p1", "bob", ub, 5)
	if res, err := s.Save(ctx, "p1", "auto", doc.state); err != nil || res.Saved {
		t.Fatalf("Save = %+v, %v", res, err)
	}
	if v := snaps.all()[0]; v.Count() != 400 || !slices.Equal(v.Contributors, []string{"alice", "bob"}) {
		t.Fatalf("version = %+v; the count must not shrink to the cut list", v)
	}
	if len(notes.list()) != 1 {
		t.Fatalf("notices = %+v", notes.list())
	}
}

func TestSave_FailedCreditPutsTheRecordBack(t *testing.T) {
	t.Parallel()

	ctx := context.Background()
	s, store, snaps, notes := newSaver()
	doc := newTestDoc(1)
	mark(t, store, "p1", "alice", doc.write("a"), 1)
	state := func(ctx context.Context) ([]byte, error) {
		mark(t, store, "p1", "bob", doc.write("b"), 5)
		return doc.state(ctx)
	}
	if res, _ := s.Save(ctx, "p1", "Saved", state); !res.Saved {
		t.Fatal("first version not saved")
	}
	snaps.failAttribs = errors.New("gcs down")
	if res, err := s.Save(ctx, "p1", "auto", doc.state); err == nil || res.Saved {
		t.Fatalf("Save = %+v, %v; want the failure reported", res, err)
	}
	if _, ok, _ := store.FirstEdit(ctx, "p1"); !ok {
		t.Fatal("bob's edit was dropped")
	}
	snaps.failAttribs = nil
	if _, err := s.Save(ctx, "p1", "auto", doc.state); err != nil {
		t.Fatal(err)
	}
	if v := snaps.all(); len(v) != 1 || !slices.Equal(v[0].Contributors, []string{"alice", "bob"}) {
		t.Fatalf("versions = %+v, want bob credited on the retry", v)
	}
	if len(notes.list()) != 2 {
		t.Fatalf("notices = %d, want 2", len(notes.list()))
	}
}

func TestSave_FailedNewestVersionLookupPutsTheRecordBack(t *testing.T) {
	t.Parallel()

	s, store, snaps, notes := newSaver()
	snaps.failLatest = errors.New("gcs down")
	doc := newTestDoc(1)
	mark(t, store, "p1", "alice", doc.write("a"), 1)
	if res, err := s.Save(context.Background(), "p1", "Saved", doc.state); res.Saved || err == nil {
		t.Fatalf("Save = %+v, %v; want no version without the comparison", res, err)
	}
	if len(snaps.all()) != 0 {
		t.Fatal("a version was saved without comparing it to the newest")
	}
	assertPutBack(t, store, notes)
}

func TestSaver_WaitWaitsForASaveInProgress(t *testing.T) {
	t.Parallel()

	s, store, _, _ := newSaver()
	doc := newTestDoc(1)
	mark(t, store, "p1", "alice", doc.write("a"), 1)
	entered, release := make(chan struct{}), make(chan struct{})
	state := func(ctx context.Context) ([]byte, error) {
		close(entered)
		<-release
		return doc.state(ctx)
	}
	go func() { _, _ = s.Save(context.Background(), "p1", "auto", state) }()
	<-entered

	ctx, cancel := context.WithTimeout(context.Background(), 50*time.Millisecond)
	defer cancel()
	if err := s.Wait(ctx); err == nil {
		t.Fatal("Wait returned while a save was in progress")
	}
	close(release)
	if err := s.Wait(context.Background()); err != nil {
		t.Fatal(err)
	}
	if _, ok, _ := store.FirstEdit(context.Background(), "p1"); ok {
		t.Fatal("Wait returned before the save finished")
	}
}

func TestSave_RecordAlreadyCreditedSendsNothing(t *testing.T) {
	t.Parallel()

	ctx := context.Background()
	s, store, snaps, notes := newSaver()
	doc := newTestDoc(1)
	u := doc.write("x")
	mark(t, store, "p1", "alice", u, 5)
	if res, _ := s.Save(ctx, "p1", "Saved", doc.state); !res.Saved {
		t.Fatal("first version not saved")
	}
	// The same edit recorded again (an instance retried a write that had
	// landed): nothing new to save, nobody new to credit.
	mark(t, store, "p1", "alice", u, 5)
	if res, err := s.Save(ctx, "p1", "Saved", doc.state); err != nil || res.Saved {
		t.Fatalf("Save = %+v, %v", res, err)
	}
	if len(snaps.all()) != 1 || len(notes.list()) != 1 {
		t.Fatalf("versions %d notices %d, want 1 and 1", len(snaps.all()), len(notes.list()))
	}
}

func TestSave_DeletionIsAChange(t *testing.T) {
	t.Parallel()

	ctx := context.Background()
	s, store, snaps, _ := newSaver()
	doc := newTestDoc(1)
	mark(t, store, "p1", "alice", doc.write("xy"), 1)
	if res, _ := s.Save(ctx, "p1", "Saved", doc.state); !res.Saved {
		t.Fatal("first version not saved")
	}
	mark(t, store, "p1", "alice", doc.deleteFirst(), 2)
	if res, err := s.Save(ctx, "p1", "Saved", doc.state); err != nil || !res.Saved {
		t.Fatalf("Save = %+v, %v; want the deletion saved", res, err)
	}
	if len(snaps.all()) != 2 {
		t.Fatalf("versions = %d, want 2", len(snaps.all()))
	}
}

// assertPutBack checks that alice's taken edit is in p1's record again and
// that nothing was announced.
func assertPutBack(t *testing.T, store *MemoryStore, notes *savedLog) {
	t.Helper()
	r, ok, _ := store.Take(context.Background(), "p1")
	if !ok || !slices.Equal(r.Contributors(), []string{"alice"}) {
		t.Fatalf("record = %v, %v; want it back", r, ok)
	}
	if len(notes.list()) != 0 {
		t.Fatal("an unsaved version was announced")
	}
}

func TestSave_FailedSnapshotPutsTheRecordBack(t *testing.T) {
	t.Parallel()

	s, store, snaps, notes := newSaver()
	snaps.fail = errors.New("gcs down")
	doc := newTestDoc(1)
	mark(t, store, "p1", "alice", doc.write("a"), 1)
	if res, err := s.Save(context.Background(), "p1", "Saved", doc.state); res.Saved || err == nil {
		t.Fatalf("Save = %+v, %v", res, err)
	}
	assertPutBack(t, store, notes)
}

func TestSave_FailedStateReadPutsTheRecordBack(t *testing.T) {
	t.Parallel()

	s, store, _, notes := newSaver()
	mark(t, store, "p1", "alice", newTestDoc(1).write("a"), 1)
	state := func(context.Context) ([]byte, error) { return nil, errors.New("load failed") }
	if res, err := s.Save(context.Background(), "p1", "Saved", state); res.Saved || err == nil {
		t.Fatalf("Save = %+v, %v", res, err)
	}
	assertPutBack(t, store, notes)
}

func TestSave_MissingStatePutsTheRecordBack(t *testing.T) {
	t.Parallel()

	s, store, _, notes := newSaver()
	mark(t, store, "p1", "alice", newTestDoc(1).write("a"), 1)
	state := func(context.Context) ([]byte, error) { return nil, nil }
	if res, _ := s.Save(context.Background(), "p1", "Saved", state); res.Saved {
		t.Fatal("saved")
	}
	assertPutBack(t, store, notes)
}

func TestSave_CheckpointFailureKeepsTheVersion(t *testing.T) {
	t.Parallel()

	ctx := context.Background()
	s, store, _, notes := newSaver()
	s.Checkpoint = func(context.Context, string) error { return errors.New("gcs down") }
	doc := newTestDoc(1)
	mark(t, store, "p1", "alice", doc.write("x"), 1)

	res, err := s.Save(ctx, "p1", "Saved", doc.state)
	if !res.Saved || !errors.Is(err, ErrCheckpoint) {
		t.Fatalf("Save = %+v, %v; want the version saved and ErrCheckpoint", res, err)
	}
	if len(notes.list()) != 1 {
		t.Fatal("the version must still be announced")
	}
}

func TestSave_FlushesThisInstancesEditsFirst(t *testing.T) {
	t.Parallel()

	ctx := context.Background()
	s, store, snaps, _ := newSaver()
	s.Tracker = NewTracker(store, time.Hour, nil)
	doc := newTestDoc(1)
	s.Tracker.Edit("p1", "alice", idsOf(t, doc.write("a")))
	if err := s.Tracker.Flush(ctx, "p1"); err != nil {
		t.Fatal(err)
	}
	s.Tracker.Edit("p1", "bob", idsOf(t, doc.write("b"))) // gathered, not yet written

	if res, err := s.Save(ctx, "p1", "Saved", doc.state); err != nil || !res.Saved {
		t.Fatalf("Save = %+v, %v", res, err)
	}
	if v := snaps.all()[0]; !slices.Equal(v.Contributors, []string{"alice", "bob"}) {
		t.Fatalf("contributors = %v; bob's gathered edit must reach the record first", v.Contributors)
	}
}

func TestSave_SavesOfARoomDoNotOverlap(t *testing.T) {
	t.Parallel()

	ctx := context.Background()
	s, store, _, _ := newSaver()
	var locked atomic.Int32
	s.Lock = func(ctx context.Context, room string, fn func(context.Context) error) error {
		locked.Add(1)
		return fn(ctx)
	}
	doc := newTestDoc(1)
	mark(t, store, "p1", "alice", doc.write("a"), 1)

	var running, overlapped atomic.Int32
	state := func(ctx context.Context) ([]byte, error) {
		if running.Add(1) > 1 {
			overlapped.Store(1)
		}
		time.Sleep(20 * time.Millisecond)
		running.Add(-1)
		return doc.state(ctx)
	}
	var wg sync.WaitGroup
	for range 4 {
		wg.Add(1)
		go func() {
			defer wg.Done()
			mark(t, store, "p1", "alice", doc.write("a"), 1)
			_, _ = s.Save(ctx, "p1", "Saved", state)
		}()
	}
	wg.Wait()
	if overlapped.Load() != 0 {
		t.Fatal("two saves of one room overlapped")
	}
	if locked.Load() != 4 {
		t.Fatalf("Lock taken %d times, want once per save", locked.Load())
	}
}

func TestSave_NoticeWithoutEditTimeUsesTheSaveTime(t *testing.T) {
	t.Parallel()

	ctx := context.Background()
	s, store, _, notes := newSaver()
	doc := newTestDoc(1)
	e := NewEdits()
	e.add(idsOf(t, doc.write("a")), time.Time{})
	if err := store.Mark(ctx, "p1", Record{"alice": e}); err != nil {
		t.Fatal(err)
	}
	if res, _ := s.Save(ctx, "p1", "Saved", doc.state); !res.Saved {
		t.Fatal("not saved")
	}
	if n := notes.list(); n[0].LastEditedAt.IsZero() || !n[0].LastEditedAt.Equal(n[0].SavedAt) {
		t.Fatalf("lastEditedAt = %v, want the save time rather than year 1", n[0].LastEditedAt)
	}
}

func TestProjectOf(t *testing.T) {
	t.Parallel()

	if got := ProjectOf("p1"); got != "p1" {
		t.Errorf("ProjectOf(p1) = %q", got)
	}
	if got := ProjectOf("p1:wf"); got != "p1" {
		t.Errorf("ProjectOf(p1:wf) = %q", got)
	}
}
