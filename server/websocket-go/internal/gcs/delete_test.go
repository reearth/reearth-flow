package gcs

import (
	"context"
	"testing"

	"github.com/reearth/ygo/crdt"
)

// runDeleteRemovesNamedSnapshots checks Delete removes all state for a room,
// including named snapshots the static-name list previously missed.
func runDeleteRemovesNamedSnapshots(t *testing.T, newAdapter func(t *testing.T) *Adapter) {
	t.Helper()
	ctx := context.Background()
	a := newAdapter(t)

	doc := crdt.New(crdt.WithClientID(7))
	txt := doc.GetText("t")
	doc.Transact(func(txn *crdt.Transaction) { txt.Insert(txn, 0, "abc", nil) })
	state := crdt.EncodeStateAsUpdateV1(doc, nil)
	if _, err := a.AppendUpdate(ctx, "room", state); err != nil {
		t.Fatalf("AppendUpdate: %v", err)
	}
	if _, err := a.CaptureSnapshot(ctx, "room", "snap-A", state); err != nil {
		t.Fatalf("CaptureSnapshot A: %v", err)
	}
	if _, err := a.CaptureSnapshot(ctx, "room", "snap-B", state); err != nil {
		t.Fatalf("CaptureSnapshot B: %v", err)
	}

	// Confirm the snapshot objects exist pre-delete.
	if _, err := a.store.get(ctx, a.snapshotName("room", "snap-A")); err != nil {
		t.Fatalf("snap-A should exist pre-delete: %v", err)
	}

	if err := a.Delete(ctx, "room"); err != nil {
		t.Fatalf("Delete: %v", err)
	}

	// Named snapshots must be gone.
	for _, name := range []string{"snap-A", "snap-B"} {
		if _, err := a.store.get(ctx, a.snapshotName("room", name)); err != errNotFound {
			t.Fatalf("named snapshot %q survived Delete (err=%v), want errNotFound", name, err)
		}
		if _, _, ok, err := a.RestoreSnapshot(ctx, "room", name); err != nil || ok {
			t.Fatalf("RestoreSnapshot(%q) after Delete: ok=%v err=%v, want ok=false", name, ok, err)
		}
	}

	// And nothing else lingers for the room (project-scoped sweep).
	if a.phase2 {
		left, err := a.store.list(ctx, ProjectPrefix("room"))
		if err != nil {
			t.Fatalf("list after delete: %v", err)
		}
		if len(left) != 0 {
			t.Fatalf("Phase-2 Delete left %d objects under prefix: %v", len(left), left)
		}
	}
}

func TestDeleteRemovesNamedSnapshots_Phase1(t *testing.T) {
	runDeleteRemovesNamedSnapshots(t, func(t *testing.T) *Adapter {
		client, bucket := newFakeGCS(t)
		a, err := New(Options{Client: client, Bucket: bucket, Locker: NewNoLock()})
		if err != nil {
			t.Fatalf("New: %v", err)
		}
		return a
	})
}

func TestDeleteRemovesNamedSnapshots_Phase2(t *testing.T) {
	runDeleteRemovesNamedSnapshots(t, func(t *testing.T) *Adapter {
		client, bucket := newFakeGCS(t)
		a, err := New(Options{Client: client, Bucket: bucket, Locker: NewNoLock(), Phase2: true})
		if err != nil {
			t.Fatalf("New phase2: %v", err)
		}
		return a
	})
}
