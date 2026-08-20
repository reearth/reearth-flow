package gcs

import (
	"context"
	"sort"
	"testing"

	"github.com/reearth/ygo/crdt"
)

// seedDoc writes n incremental updates for room (Phase 1 OID index / Phase 2
// {room}/ prefix) without opening a live room.
func seedDoc(t *testing.T, a *Adapter, room string, n int) {
	t.Helper()
	ctx := context.Background()
	doc := crdt.New(crdt.WithClientID(1))
	txt := doc.GetText("t")
	for i := 0; i < n; i++ {
		before := doc.StateVector()
		doc.Transact(func(txn *crdt.Transaction) { txt.Insert(txn, 0, "x", nil) })
		update := crdt.EncodeStateAsUpdateV1(doc, before)
		if _, err := a.AppendUpdate(ctx, room, update); err != nil {
			t.Fatalf("AppendUpdate(%s): %v", room, err)
		}
	}
}

func runListAllDocs(t *testing.T, newAdapter func(t *testing.T) *Adapter) {
	t.Helper()
	ctx := context.Background()
	a := newAdapter(t)

	want := []string{
		"550e8400-e29b-41d4-a716-446655440000",
		"550e8400-e29b-41d4-a716-446655440001",
		"6ba7b810-9dad-11d1-80b4-00c04fd430c8",
	}
	for _, room := range want {
		seedDoc(t, a, room, 3) // non-resident: never opened as a live room
	}

	got, err := a.ListAllDocs(ctx)
	if err != nil {
		t.Fatalf("ListAllDocs: %v", err)
	}
	sort.Strings(got)
	sort.Strings(want)
	if len(got) != len(want) {
		t.Fatalf("ListAllDocs returned %d docs %v, want %d %v", len(got), got, len(want), want)
	}
	for i := range want {
		if got[i] != want[i] {
			t.Fatalf("ListAllDocs[%d] = %q, want %q (full: %v)", i, got[i], want[i], got)
		}
	}
}

func TestListAllDocs_Phase1(t *testing.T) {
	runListAllDocs(t, func(t *testing.T) *Adapter {
		client, bucket := newFakeGCS(t)
		a, err := New(Options{Client: client, Bucket: bucket, Locker: NewNoLock()})
		if err != nil {
			t.Fatalf("New: %v", err)
		}
		return a
	})
}

func TestListAllDocs_Phase2(t *testing.T) {
	runListAllDocs(t, func(t *testing.T) *Adapter {
		client, bucket := newFakeGCS(t)
		a, err := New(Options{Client: client, Bucket: bucket, Locker: NewNoLock(), Phase2: true})
		if err != nil {
			t.Fatalf("New phase2: %v", err)
		}
		return a
	})
}

// TestListAllDocs_Empty: an empty bucket yields no docs, no error.
func TestListAllDocs_Empty(t *testing.T) {
	client, bucket := newFakeGCS(t)
	a, err := New(Options{Client: client, Bucket: bucket, Locker: NewNoLock()})
	if err != nil {
		t.Fatalf("New: %v", err)
	}
	got, err := a.ListAllDocs(context.Background())
	if err != nil {
		t.Fatalf("ListAllDocs: %v", err)
	}
	if len(got) != 0 {
		t.Fatalf("ListAllDocs on empty bucket = %v, want empty", got)
	}
}

// TestListAllDocs_Phase1_IgnoresDocKeyspace asserts the returned ids are real doc
// ids, not stray hex from the doc keyspace (which shares the leading 00 byte).
func TestListAllDocs_Phase1_IgnoresDocKeyspace(t *testing.T) {
	ctx := context.Background()
	client, bucket := newFakeGCS(t)
	a, err := New(Options{Client: client, Bucket: bucket, Locker: NewNoLock()})
	if err != nil {
		t.Fatalf("New: %v", err)
	}
	// Many updates so the doc keyspace is populated and compaction runs.
	seedDoc(t, a, "doc-with-many-updates", 25)

	got, err := a.ListAllDocs(ctx)
	if err != nil {
		t.Fatalf("ListAllDocs: %v", err)
	}
	if len(got) != 1 || got[0] != "doc-with-many-updates" {
		t.Fatalf("ListAllDocs = %v, want exactly [doc-with-many-updates]", got)
	}
}
