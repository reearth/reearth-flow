package http

import (
	"context"
	"errors"
	"testing"
	"time"

	"github.com/reearth/ygo/persistence"
)

// memSnapshotStore is a hand-rolled, in-memory persistence.SnapshotStore that
// mirrors the real contract (ID monotonically increasing per room, newest
// first on list, ErrSnapshotNotFound for an unknown pair). It embeds
// *memPersist (from adapt_test.go) so it also satisfies the local Persistence
// interface, letting these tests exercise StoreAdapter's real bridging logic
// end to end without pulling in genuine Yjs-encoded update bytes (Task 3's
// GCS-adapter tests already cover the production SnapshotStore backend).
type memSnapshotStore struct {
	*memPersist
	byRoom map[string][]persistence.SnapshotInfo
	states map[string]map[int64][]byte
	nextID int64
	now    func() time.Time
}

func newMemSnapshotStore() *memSnapshotStore {
	return &memSnapshotStore{
		memPersist: newMemPersist(),
		byRoom:     map[string][]persistence.SnapshotInfo{},
		states:     map[string]map[int64][]byte{},
		nextID:     1,
		now:        time.Now,
	}
}

func (m *memSnapshotStore) SaveSnapshot(ctx context.Context, room, label string, state []byte) (int64, error) {
	if len(state) == 0 {
		return 0, persistence.ErrEmptySnapshot
	}
	id := m.nextID
	m.nextID++
	info := persistence.SnapshotInfo{ID: id, Label: label, CreatedAt: m.now().UTC(), Size: int64(len(state))}
	// Prepend so the slice stays newest-first, matching the real contract.
	m.byRoom[room] = append([]persistence.SnapshotInfo{info}, m.byRoom[room]...)
	if m.states[room] == nil {
		m.states[room] = map[int64][]byte{}
	}
	m.states[room][id] = state
	return id, nil
}

func (m *memSnapshotStore) ListSnapshots(ctx context.Context, room string) ([]persistence.SnapshotInfo, error) {
	if m.byRoom[room] == nil {
		return []persistence.SnapshotInfo{}, nil
	}
	return m.byRoom[room], nil
}

func (m *memSnapshotStore) GetSnapshotState(ctx context.Context, room string, id int64) ([]byte, error) {
	b, ok := m.states[room][id]
	if !ok {
		return nil, persistence.ErrSnapshotNotFound
	}
	return b, nil
}

func (m *memSnapshotStore) DeleteSnapshot(ctx context.Context, room string, id int64) error {
	delete(m.states[room], id)
	return nil
}

var _ persistence.SnapshotStore = (*memSnapshotStore)(nil)

// TestStoreAdapterSaveSnapshot_NewestFirstWithLabels: two real SaveSnapshot
// calls through the StoreAdapter bridge must come back from ListSnapshots
// newest-first, with labels and sizes intact — this is the exact shape Task 6
// promises the Project History panel.
func TestStoreAdapterSaveSnapshot_NewestFirstWithLabels(t *testing.T) {
	mss := newMemSnapshotStore()
	base := time.Date(2026, 7, 30, 9, 0, 0, 0, time.UTC)
	tick := 0
	mss.now = func() time.Time { tick++; return base.Add(time.Duration(tick) * time.Minute) }
	_, _ = mss.AppendUpdate(context.Background(), "room1", []byte{1, 2, 3})

	st := NewStoreAdapter(StoreAdapterDeps{P: mss})
	ctx := context.Background()

	if _, err := st.SaveSnapshot(ctx, "room1", "first"); err != nil {
		t.Fatalf("SaveSnapshot(first): %v", err)
	}
	if _, err := st.SaveSnapshot(ctx, "room1", "second"); err != nil {
		t.Fatalf("SaveSnapshot(second): %v", err)
	}

	got, err := st.ListSnapshots(ctx, "room1")
	if err != nil {
		t.Fatalf("ListSnapshots: %v", err)
	}
	if len(got) != 2 {
		t.Fatalf("len = %d, want 2: %+v", len(got), got)
	}
	if got[0].Label != "second" {
		t.Fatalf("got[0].Label = %q, want %q (newest-first)", got[0].Label, "second")
	}
	if got[1].Label != "first" {
		t.Fatalf("got[1].Label = %q, want %q", got[1].Label, "first")
	}
	if got[0].Timestamp == "" {
		t.Fatal("Timestamp must be populated (RFC3339)")
	}
	if _, err := time.Parse(time.RFC3339, got[0].Timestamp); err != nil {
		t.Fatalf("Timestamp not RFC3339: %v", err)
	}
	if got[0].Size != 3 {
		t.Fatalf("Size = %d, want 3 (len of the room's update bytes)", got[0].Size)
	}
}

// TestStoreAdapterSaveSnapshot_FlushesBeforeCapturing: SaveSnapshot must flush
// the live room before reading its state, so a named snapshot reflects
// in-memory edits that haven't been durably persisted yet.
func TestStoreAdapterSaveSnapshot_FlushesBeforeCapturing(t *testing.T) {
	mss := newMemSnapshotStore()
	flushCalled := false
	st := NewStoreAdapter(StoreAdapterDeps{
		P: mss,
		FlushFn: func(ctx context.Context, room string) error {
			flushCalled = true
			// Simulate flushing in-memory edits into the persisted log.
			_, err := mss.AppendUpdate(ctx, room, []byte{9, 9, 9})
			return err
		},
	})
	ctx := context.Background()

	id, err := st.SaveSnapshot(ctx, "room1", "post-flush")
	if err != nil {
		t.Fatalf("SaveSnapshot: %v", err)
	}
	if !flushCalled {
		t.Fatal("flushFn was not invoked before capturing the snapshot")
	}
	state, err := st.GetSnapshotState(ctx, "room1", id)
	if err != nil {
		t.Fatalf("GetSnapshotState: %v", err)
	}
	if string(state) != "\x09\x09\x09" {
		t.Fatalf("snapshot state = %v, want the flushed bytes (flush ran before capture)", state)
	}
}

// TestStoreAdapterSaveSnapshot_FlushErrorFailsClosed: when the flush fails,
// SaveSnapshot must propagate the error and record NOTHING. Falling through to
// capture the pre-flush state would be the dangerous outcome: the user asked to
// version what is on their canvas, and they would silently get a snapshot
// missing their most recent edits, with no indication anything went wrong. A
// visible failure they can retry is strictly better than a quietly stale
// version, so this asserts both halves — the error surfaces AND no snapshot is
// written.
func TestStoreAdapterSaveSnapshot_FlushErrorFailsClosed(t *testing.T) {
	mss := newMemSnapshotStore()
	flushErr := errors.New("gcs flush unavailable")
	st := NewStoreAdapter(StoreAdapterDeps{
		P: mss,
		FlushFn: func(ctx context.Context, room string) error {
			return flushErr
		},
	})
	ctx := context.Background()

	// Give the room durable state, so a fall-through would successfully capture
	// the stale bytes rather than tripping the empty-room no-op. Without this the
	// test would pass for the wrong reason.
	if _, err := mss.AppendUpdate(ctx, "room1", []byte{1, 2, 3}); err != nil {
		t.Fatalf("seed AppendUpdate: %v", err)
	}

	id, err := st.SaveSnapshot(ctx, "room1", "label")
	if !errors.Is(err, flushErr) {
		t.Fatalf("SaveSnapshot err = %v, want %v", err, flushErr)
	}
	if id != 0 {
		t.Fatalf("id = %d, want 0 on a failed flush", id)
	}
	if len(mss.byRoom["room1"]) != 0 {
		t.Fatalf("a stale pre-flush snapshot was recorded: %+v", mss.byRoom["room1"])
	}
}

// TestStoreAdapterSaveSnapshot_EmptyRoomIsNoop: an unknown/empty room has no
// state worth versioning; SaveSnapshot must not error and must not call the
// underlying SnapshotStore (which would itself reject empty state).
func TestStoreAdapterSaveSnapshot_EmptyRoomIsNoop(t *testing.T) {
	mss := newMemSnapshotStore()
	st := NewStoreAdapter(StoreAdapterDeps{P: mss})
	id, err := st.SaveSnapshot(context.Background(), "never-written", "label")
	if err != nil {
		t.Fatalf("SaveSnapshot on empty room: %v", err)
	}
	if id != 0 {
		t.Fatalf("id = %d, want 0", id)
	}
	if len(mss.byRoom["never-written"]) != 0 {
		t.Fatalf("a snapshot was recorded for an empty room: %+v", mss.byRoom["never-written"])
	}
}

// TestStoreAdapterSnapshots_UnsupportedStoreFallsBackGracefully: a Persistence
// that does not implement persistence.SnapshotStore (the plain memPersist
// fake, matching a hypothetical non-GCS backend) must degrade gracefully:
// ListSnapshots is an empty list (not an error, so the History panel just shows
// nothing), while GetSnapshotState and SaveSnapshot both report
// ErrSnapshotsUnsupported so the router can answer 501.
//
// GetSnapshotState deliberately does NOT report ErrSnapshotNotFound here. That
// would render as 404 and claim this particular snapshot is missing, sending an
// operator to look for a deleted object when the real situation is that the
// backend has no snapshot support at all. Listing is the one lenient case, and
// only because an empty history panel is a better outcome than an error toast.
func TestStoreAdapterSnapshots_UnsupportedStoreFallsBackGracefully(t *testing.T) {
	mp := newMemPersist() // does NOT implement persistence.SnapshotStore
	st := NewStoreAdapter(StoreAdapterDeps{P: mp})
	ctx := context.Background()

	items, err := st.ListSnapshots(ctx, "room1")
	if err != nil {
		t.Fatalf("ListSnapshots: %v", err)
	}
	if items == nil || len(items) != 0 {
		t.Fatalf("items = %+v, want a non-nil empty slice", items)
	}

	if _, err := st.GetSnapshotState(ctx, "room1", 1); !errors.Is(err, persistence.ErrSnapshotsUnsupported) {
		t.Fatalf("GetSnapshotState err = %v, want ErrSnapshotsUnsupported (not ErrSnapshotNotFound: the feature is absent, the snapshot is not missing)", err)
	}

	if _, err := st.SaveSnapshot(ctx, "room1", "x"); !errors.Is(err, persistence.ErrSnapshotsUnsupported) {
		t.Fatalf("SaveSnapshot err = %v, want ErrSnapshotsUnsupported", err)
	}
}
