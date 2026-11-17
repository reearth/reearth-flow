package versions

import (
	"context"
	"errors"
	"slices"
	"sync"
	"testing"

	"github.com/reearth/ygo/crdt"
)

// memSnapshots is an in-memory Snapshots.
type memSnapshots struct {
	mu    sync.Mutex
	saved []Version
	fail  error
}

func (m *memSnapshots) Save(_ context.Context, _ string, v Version) (int64, error) {
	m.mu.Lock()
	defer m.mu.Unlock()
	if m.fail != nil {
		return 0, m.fail
	}
	m.saved = append(m.saved, v)
	return int64(len(m.saved)), nil
}

func (m *memSnapshots) Latest(context.Context, string) ([]byte, error) {
	m.mu.Lock()
	defer m.mu.Unlock()
	if len(m.saved) == 0 {
		return nil, nil
	}
	return m.saved[len(m.saved)-1].State, nil
}

// testDoc is a document whose state the tests change.
type testDoc struct{ doc *crdt.Doc }

func newTestDoc() *testDoc { return &testDoc{doc: crdt.New(crdt.WithClientID(1))} }

func (d *testDoc) write(s string) {
	txt := d.doc.GetText("t")
	d.doc.Transact(func(txn *crdt.Transaction) { txt.Insert(txn, txt.Len(), s, nil) })
}

func (d *testDoc) state(context.Context) ([]byte, error) {
	return crdt.EncodeStateAsUpdateV1(d.doc, nil), nil
}

type savedLog struct {
	mu  sync.Mutex
	all []Saved
}

func (l *savedLog) notify(s Saved) { l.mu.Lock(); l.all = append(l.all, s); l.mu.Unlock() }

func newSaver() (*Saver, *MemoryStore, *memSnapshots, *savedLog) {
	store := NewMemoryStore()
	snaps := &memSnapshots{}
	notes := &savedLog{}
	return &Saver{Store: store, Snapshots: snaps, Notify: notes.notify}, store, snaps, notes
}

func markEdit(t *testing.T, store Store, room, user string, ms int) {
	t.Helper()
	if err := store.Mark(context.Background(), room, Mark{
		Editors: []string{user}, LastEditor: user, LastEditorAt: at(ms), FirstAt: at(ms), LastAt: at(ms),
	}); err != nil {
		t.Fatal(err)
	}
}

func TestSave_NothingChangedSavesNothing(t *testing.T) {
	t.Parallel()

	s, _, snaps, notes := newSaver()
	doc := newTestDoc()
	doc.write("x")
	res, err := s.Save(context.Background(), "p1", "Saved", doc.state)
	if err != nil || res.Saved {
		t.Fatalf("Save = %+v, %v; want nothing saved without a change record", res, err)
	}
	if len(snaps.saved) != 0 || len(notes.all) != 0 {
		t.Fatal("a version was stored or announced without a change")
	}
}

func TestSave_SavesTheChangeOnceWithItsContributors(t *testing.T) {
	t.Parallel()

	ctx := context.Background()
	s, store, snaps, notes := newSaver()
	var checkpointed []byte
	s.Checkpoint = func(_ context.Context, _ string, st []byte) error { checkpointed = st; return nil }
	doc := newTestDoc()
	doc.write("x")
	markEdit(t, store, "p1:main", "alice", 1)
	markEdit(t, store, "p1:main", "bob", 2)

	res, err := s.Save(ctx, "p1:main", "Saved", doc.state)
	if err != nil || !res.Saved || res.SnapshotID != 1 {
		t.Fatalf("Save = %+v, %v", res, err)
	}
	v := snaps.saved[0]
	if v.Label != "Saved" || !slices.Equal(v.Contributors, []string{"alice", "bob"}) || v.LastEditor != "bob" {
		t.Fatalf("version = %+v", v)
	}
	if !slices.Equal(checkpointed, v.State) {
		t.Fatal("the checkpoint must hold the state the version holds")
	}
	if len(notes.all) != 1 {
		t.Fatalf("notices = %d, want 1", len(notes.all))
	}
	n := notes.all[0]
	if n.ProjectID != "p1" || n.VersionID != 1 || n.LastEditorID != "bob" ||
		!slices.Equal(n.ContributorIDs, []string{"alice", "bob"}) || !n.LastEditedAt.Equal(at(2)) || n.SavedAt.IsZero() {
		t.Fatalf("notice = %+v", n)
	}

	res, err = s.Save(ctx, "p1:main", "Saved", doc.state)
	if err != nil || res.Saved {
		t.Fatalf("second Save = %+v, %v; want nothing: the record was taken", res, err)
	}
}

func TestSave_FailedSnapshotPutsTheRecordBack(t *testing.T) {
	t.Parallel()

	ctx := context.Background()
	s, store, snaps, notes := newSaver()
	snaps.fail = errors.New("gcs down")
	doc := newTestDoc()
	doc.write("x")
	markEdit(t, store, "p1", "alice", 1)

	if _, err := s.Save(ctx, "p1", "Saved", doc.state); err == nil {
		t.Fatal("Save succeeded with a failing store")
	}
	r, ok, _ := store.Peek(ctx, "p1")
	if !ok || !slices.Equal(r.Contributors, []string{"alice"}) {
		t.Fatalf("record after failure = %+v, %v; want it back", r, ok)
	}
	if len(notes.all) != 0 {
		t.Fatal("a failed version was announced")
	}
}

func TestSave_FailedStateReadPutsTheRecordBack(t *testing.T) {
	t.Parallel()

	ctx := context.Background()
	s, store, _, _ := newSaver()
	markEdit(t, store, "p1", "alice", 1)
	_, err := s.Save(ctx, "p1", "Saved", func(context.Context) ([]byte, error) { return nil, errors.New("load failed") })
	if err == nil {
		t.Fatal("Save succeeded without a state")
	}
	if _, ok, _ := store.Peek(ctx, "p1"); !ok {
		t.Fatal("the record was lost")
	}
}

func TestSave_MissingStateKeepsTheRecord(t *testing.T) {
	t.Parallel()

	ctx := context.Background()
	s, store, snaps, _ := newSaver()
	markEdit(t, store, "p1", "alice", 1)
	res, err := s.Save(ctx, "p1", "Saved", func(context.Context) ([]byte, error) { return nil, nil })
	if err != nil || res.Saved || len(snaps.saved) != 0 {
		t.Fatalf("Save = %+v, %v", res, err)
	}
	if _, ok, _ := store.Peek(ctx, "p1"); !ok {
		t.Fatal("the change record was dropped though nothing was saved")
	}
}

func TestSave_EditDuringSaveIsKeptForTheNextVersion(t *testing.T) {
	t.Parallel()

	ctx := context.Background()
	s, store, snaps, _ := newSaver()
	doc := newTestDoc()
	doc.write("x")
	markEdit(t, store, "p1", "alice", 1)

	// bob's edit lands after the record was taken, before the state is read.
	state := func(ctx context.Context) ([]byte, error) {
		doc.write("y")
		markEdit(t, store, "p1", "bob", 2)
		return doc.state(ctx)
	}
	if res, err := s.Save(ctx, "p1", "Saved", state); err != nil || !res.Saved {
		t.Fatalf("Save = %+v, %v", res, err)
	}
	r, ok, _ := store.Peek(ctx, "p1")
	if !ok || !slices.Equal(r.Contributors, []string{"bob"}) {
		t.Fatalf("record = %+v, %v; want bob's edit recorded for the next version", r, ok)
	}

	// The version already holds bob's edit, so the next save has nothing new.
	res, err := s.Save(ctx, "p1", "Saved", doc.state)
	if err != nil || res.Saved {
		t.Fatalf("next Save = %+v, %v; want nothing saved: the document equals the newest version", res, err)
	}
	if len(snaps.saved) != 1 {
		t.Fatalf("versions = %d, want 1", len(snaps.saved))
	}
	if _, ok, _ := store.Peek(ctx, "p1"); ok {
		t.Fatal("the record must be taken even when nothing was saved")
	}
}

func TestSave_UndoneEditSavesNothing(t *testing.T) {
	t.Parallel()

	ctx := context.Background()
	s, store, snaps, _ := newSaver()
	doc := newTestDoc()
	doc.write("x")
	markEdit(t, store, "p1", "alice", 1)
	if res, _ := s.Save(ctx, "p1", "Saved", doc.state); !res.Saved {
		t.Fatal("first version not saved")
	}

	// A change record whose edits leave the document as the version has it.
	markEdit(t, store, "p1", "alice", 2)
	if res, err := s.Save(ctx, "p1", "Saved", doc.state); err != nil || res.Saved {
		t.Fatalf("Save = %+v, %v; want nothing saved", res, err)
	}
	if len(snaps.saved) != 1 {
		t.Fatalf("versions = %d, want 1", len(snaps.saved))
	}
}

func TestSave_DeletionIsAChange(t *testing.T) {
	t.Parallel()

	ctx := context.Background()
	s, store, snaps, _ := newSaver()
	doc := newTestDoc()
	doc.write("xy")
	markEdit(t, store, "p1", "alice", 1)
	if res, _ := s.Save(ctx, "p1", "Saved", doc.state); !res.Saved {
		t.Fatal("first version not saved")
	}

	// A deletion adds no item, only a deletion range.
	txt := doc.doc.GetText("t")
	doc.doc.Transact(func(txn *crdt.Transaction) { txt.Delete(txn, 0, 1) })
	markEdit(t, store, "p1", "alice", 2)
	if res, err := s.Save(ctx, "p1", "Saved", doc.state); err != nil || !res.Saved {
		t.Fatalf("Save = %+v, %v; want the deletion saved", res, err)
	}
	if len(snaps.saved) != 2 {
		t.Fatalf("versions = %d, want 2", len(snaps.saved))
	}
}

func TestSave_CheckpointFailureKeepsTheVersion(t *testing.T) {
	t.Parallel()

	ctx := context.Background()
	s, store, _, notes := newSaver()
	s.Checkpoint = func(context.Context, string, []byte) error { return errors.New("gcs down") }
	doc := newTestDoc()
	doc.write("x")
	markEdit(t, store, "p1", "alice", 1)

	res, err := s.Save(ctx, "p1", "Saved", doc.state)
	if !res.Saved || !errors.Is(err, ErrCheckpoint) {
		t.Fatalf("Save = %+v, %v; want the version saved and ErrCheckpoint", res, err)
	}
	if len(notes.all) != 1 {
		t.Fatal("the version must still be announced")
	}
}

func TestSave_FlushesThisInstancesEditsFirst(t *testing.T) {
	t.Parallel()

	ctx := context.Background()
	s, store, _, _ := newSaver()
	s.Tracker = NewTracker(store, 0, nil)
	s.Tracker.every = 1 << 62 // hold everything after the first write back
	s.Tracker.Edit("p1", "alice")
	if err := s.Tracker.Flush(ctx, "p1"); err != nil {
		t.Fatal(err)
	}
	s.Tracker.Edit("p1", "bob") // gathered, not yet written

	doc := newTestDoc()
	doc.write("x")
	res, err := s.Save(ctx, "p1", "Saved", doc.state)
	if err != nil || !res.Saved {
		t.Fatalf("Save = %+v, %v", res, err)
	}
	if _, ok, _ := store.Peek(ctx, "p1"); ok {
		t.Fatal("bob's gathered edit reached the store after the version instead of before it")
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
