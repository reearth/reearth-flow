package http

import (
	"context"
	"errors"
	"testing"
	"time"

	"github.com/reearth/ygo/persistence"
)

// memPersist is an in-memory persistence fake for testing the StoreAdapter bridge.
type memPersist struct {
	updates    map[string][][]byte // room -> ordered update bytes
	flushed    map[string]bool
	snapshots  map[string][]byte
	deleted    map[string]bool
	pruned     map[string]uint64
	rolledBack map[string][]byte
	compacted  map[string]int
	// compactErrOn makes Compact(room) fail for the listed rooms.
	compactErrOn map[string]error
}

func newMemPersist() *memPersist {
	return &memPersist{
		updates:      map[string][][]byte{},
		flushed:      map[string]bool{},
		snapshots:    map[string][]byte{},
		deleted:      map[string]bool{},
		pruned:       map[string]uint64{},
		rolledBack:   map[string][]byte{},
		compacted:    map[string]int{},
		compactErrOn: map[string]error{},
	}
}

func (m *memPersist) Load(ctx context.Context, room string) (persistence.LoadResult, error) {
	us := m.updates[room]
	if len(us) == 0 {
		return persistence.LoadResult{}, nil
	}
	var all []byte
	for _, u := range us {
		all = append(all, u...)
	}
	return persistence.LoadResult{Update: all, Version: persistence.Version(len(us))}, nil
}
func (m *memPersist) AppendUpdate(ctx context.Context, room string, update []byte) (persistence.Version, error) {
	m.updates[room] = append(m.updates[room], update)
	return persistence.Version(len(m.updates[room])), nil
}
func (m *memPersist) ListVersions(ctx context.Context, room string) ([]persistence.VersionMeta, error) {
	us := m.updates[room]
	out := make([]persistence.VersionMeta, 0, len(us))
	for i := len(us); i >= 1; i-- { // newest-first
		out = append(out, persistence.VersionMeta{Version: persistence.Version(i), UpdatedAt: time.Unix(int64(i), 0).UTC()})
	}
	return out, nil
}
func (m *memPersist) GetUpdate(ctx context.Context, room string, v persistence.Version) ([]byte, persistence.VersionMeta, bool, error) {
	us := m.updates[room]
	i := int(v) - 1
	if i < 0 || i >= len(us) {
		return nil, persistence.VersionMeta{}, false, nil
	}
	return us[i], persistence.VersionMeta{Version: v, UpdatedAt: time.Unix(int64(v), 0).UTC()}, true, nil
}
func (m *memPersist) MaterializeAt(ctx context.Context, room string, v persistence.Version) ([]byte, error) {
	us := m.updates[room]
	var all []byte
	for i := 0; i < int(v) && i < len(us); i++ {
		all = append(all, us[i]...)
	}
	return all, nil
}
func (m *memPersist) PruneAfter(ctx context.Context, room string, target persistence.Version, rolledBack []byte) error {
	m.pruned[room] = uint64(target)
	m.rolledBack[room] = rolledBack
	return nil
}
func (m *memPersist) CaptureSnapshot(ctx context.Context, room, name string, state []byte) (persistence.Version, error) {
	m.snapshots[room+"/"+name] = state
	return persistence.Version(len(m.updates[room])), nil
}
func (m *memPersist) RestoreSnapshot(ctx context.Context, room, name string) ([]byte, persistence.Version, bool, error) {
	b, ok := m.snapshots[room+"/"+name]
	return b, 0, ok, nil
}
func (m *memPersist) Delete(ctx context.Context, room string) error {
	m.deleted[room] = true
	delete(m.updates, room)
	return nil
}
func (m *memPersist) Compact(ctx context.Context, room string, keep int) (int, error) {
	if err := m.compactErrOn[room]; err != nil {
		return 0, err
	}
	m.compacted[room] = keep
	return 1, nil
}

// memDocLister is a memPersist that also enumerates the full doc keyspace.
type memDocLister struct {
	*memPersist
	allDocs []string
}

func (m *memDocLister) ListAllDocs(ctx context.Context) ([]string, error) {
	return m.allDocs, nil
}

func TestStoreAdapterFlush(t *testing.T) {
	mp := newMemPersist()
	flushed := ""
	st := NewStoreAdapter(StoreAdapterDeps{
		P:         mp,
		FlushFn:   func(ctx context.Context, room string) error { flushed = room; return nil },
		ListRooms: func() []string { return nil },
	})
	if err := st.Flush(context.Background(), "proj1"); err != nil {
		t.Fatalf("flush: %v", err)
	}
	if flushed != "proj1" {
		t.Fatalf("flush room = %q", flushed)
	}
}

func TestStoreAdapterImport(t *testing.T) {
	mp := newMemPersist()
	st := NewStoreAdapter(StoreAdapterDeps{P: mp})
	v, err := st.Import(context.Background(), "proj1", []byte{1, 2, 3})
	if err != nil {
		t.Fatalf("import: %v", err)
	}
	if v != 1 {
		t.Fatalf("version = %d, want 1", v)
	}
	if len(mp.updates["proj1"]) != 1 {
		t.Fatalf("update not appended")
	}
}

func TestStoreAdapterCopy(t *testing.T) {
	mp := newMemPersist()
	_, _ = mp.AppendUpdate(context.Background(), "src", []byte{7, 8})
	st := NewStoreAdapter(StoreAdapterDeps{P: mp})
	if err := st.Copy(context.Background(), "dst", "src"); err != nil {
		t.Fatalf("copy: %v", err)
	}
	if len(mp.updates["dst"]) == 0 {
		t.Fatalf("dst not written")
	}
}

func TestStoreAdapterDelegates(t *testing.T) {
	mp := newMemPersist()
	_, _ = mp.AppendUpdate(context.Background(), "proj1", []byte{1})
	_, _ = mp.AppendUpdate(context.Background(), "proj1", []byte{2})
	st := NewStoreAdapter(StoreAdapterDeps{P: mp})
	ctx := context.Background()

	vs, err := st.ListVersions(ctx, "proj1")
	if err != nil || len(vs) != 2 || vs[0].Version != 2 {
		t.Fatalf("ListVersions = %+v err=%v", vs, err)
	}
	if err := st.PruneAfter(ctx, "proj1", 1, []byte{9}); err != nil {
		t.Fatalf("prune: %v", err)
	}
	if mp.pruned["proj1"] != 1 {
		t.Fatalf("prune target = %d", mp.pruned["proj1"])
	}
	if err := st.Delete(ctx, "proj1"); err != nil || !mp.deleted["proj1"] {
		t.Fatalf("delete failed")
	}
}

// TestStoreAdapterCleanupAll_FallbackResidentRooms: without ListAllDocs,
// CleanupAll falls back to the resident rooms from ListRooms.
func TestStoreAdapterCleanupAll_FallbackResidentRooms(t *testing.T) {
	mp := newMemPersist()
	st := NewStoreAdapter(StoreAdapterDeps{
		P:         mp,
		ListRooms: func() []string { return []string{"a", "b"} },
	})
	if _, err := st.CleanupAll(context.Background(), 10); err != nil {
		t.Fatalf("cleanupall: %v", err)
	}
	if mp.compacted["a"] != 10 || mp.compacted["b"] != 10 {
		t.Fatalf("not all compacted: %+v", mp.compacted)
	}
}

// TestStoreAdapterCleanupAll_FullBucketParity: with ListAllDocs, CleanupAll
// compacts every persisted doc (including non-resident ones) and ignores ListRooms.
func TestStoreAdapterCleanupAll_FullBucketParity(t *testing.T) {
	mp := newMemPersist()
	dl := &memDocLister{
		memPersist: mp,
		allDocs:    []string{"r1", "nonresident-1", "nonresident-2"},
	}
	st := NewStoreAdapter(StoreAdapterDeps{
		P:         dl,
		ListRooms: func() []string { return []string{"r1"} }, // must be ignored
	})
	deleted, err := st.CleanupAll(context.Background(), 10)
	if err != nil {
		t.Fatalf("cleanupall: %v", err)
	}
	for _, room := range dl.allDocs {
		if mp.compacted[room] != 10 {
			t.Fatalf("doc %q not compacted via ListAllDocs: %+v", room, mp.compacted)
		}
	}
	if _, ok := mp.compacted["resident-only-leak"]; ok {
		t.Fatalf("unexpected room compacted")
	}
	if deleted != 3 {
		t.Fatalf("CleanupAll deleted = %d, want 3 (one per ListAllDocs doc)", deleted)
	}
}

// TestStoreAdapterCleanupAll_ContinuesPastDocError: one doc failing Compact must
// not abort the sweep; the aggregate error still surfaces the failure.
func TestStoreAdapterCleanupAll_ContinuesPastDocError(t *testing.T) {
	mp := newMemPersist()
	boom := errors.New("transient gcs 503")
	mp.compactErrOn["bad-doc"] = boom
	dl := &memDocLister{
		memPersist: mp,
		allDocs:    []string{"good-1", "bad-doc", "good-2"}, // bad-doc in the middle
	}
	st := NewStoreAdapter(StoreAdapterDeps{P: dl})

	deleted, err := st.CleanupAll(context.Background(), 10)

	if mp.compacted["good-1"] != 10 || mp.compacted["good-2"] != 10 {
		t.Fatalf("good docs not compacted after a sibling error: %+v", mp.compacted)
	}
	if _, ok := mp.compacted["bad-doc"]; ok {
		t.Fatalf("bad-doc should not have compacted")
	}
	if deleted != 2 {
		t.Fatalf("CleanupAll deleted = %d, want 2 (good docs only)", deleted)
	}
	if err == nil {
		t.Fatalf("CleanupAll returned nil error, want the bad-doc failure surfaced")
	}
	if !errors.Is(err, boom) {
		t.Fatalf("aggregate error does not wrap the per-doc error: %v", err)
	}
}
