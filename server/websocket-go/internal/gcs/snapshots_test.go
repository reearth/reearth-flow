package gcs

import (
	"context"
	"testing"

	"github.com/reearth/ygo/persistence"
)

// The ygo conformance suite is the contract. Passing it means Flow's adapter
// behaves exactly like the three in-tree backends.
func TestSnapshotStoreConformance_GCS(t *testing.T) {
	persistence.RunSnapshotStoreConformance(t, func() persistence.SnapshotStore {
		client, bucket := newFakeGCS(t)
		a, err := New(Options{Client: client, Bucket: bucket, Locker: NewNoLock()})
		if err != nil {
			t.Fatalf("gcs.New: %v", err)
		}
		return a
	})
}

// Size must be the UNCOMPRESSED state length, not the brotli-compressed object
// size. Highly compressible input makes the difference large and obvious.
func TestGCSSnapshot_SizeIsUncompressedLength(t *testing.T) {
	ctx := context.Background()
	client, bucket := newFakeGCS(t)
	a, err := New(Options{Client: client, Bucket: bucket, Locker: NewNoLock()})
	if err != nil {
		t.Fatalf("gcs.New: %v", err)
	}

	state := make([]byte, 32*1024) // all 'a' bytes: compresses to almost nothing
	for i := range state {
		state[i] = 'a'
	}
	if _, err := a.SaveSnapshot(ctx, "room", "lbl", state); err != nil {
		t.Fatalf("SaveSnapshot: %v", err)
	}
	got, err := a.ListSnapshots(ctx, "room")
	if err != nil {
		t.Fatalf("ListSnapshots: %v", err)
	}
	if len(got) != 1 {
		t.Fatalf("len = %d, want 1", len(got))
	}
	if got[0].Size != int64(len(state)) {
		t.Fatalf("Size = %d, want %d (must be uncompressed length)", got[0].Size, len(state))
	}
}

// Delete(room) must remove snapshot objects too, matching the documented
// "removes all data for room" contract.
func TestGCSDelete_RemovesSnapshots(t *testing.T) {
	ctx := context.Background()
	client, bucket := newFakeGCS(t)
	a, err := New(Options{Client: client, Bucket: bucket, Locker: NewNoLock()})
	if err != nil {
		t.Fatalf("gcs.New: %v", err)
	}
	if _, err := a.SaveSnapshot(ctx, "room", "lbl", []byte("state")); err != nil {
		t.Fatalf("SaveSnapshot: %v", err)
	}
	if err := a.Delete(ctx, "room"); err != nil {
		t.Fatalf("Delete: %v", err)
	}
	got, err := a.ListSnapshots(ctx, "room")
	if err != nil {
		t.Fatalf("ListSnapshots: %v", err)
	}
	if len(got) != 0 {
		t.Fatalf("after Delete(room) snapshots = %+v, want none", got)
	}
}

// TestGCSDelete_RemovesSnapshots_Phase2 is the Phase-2 counterpart: the
// ProjectFolderLayout's SnapVersionPrefix/SnapNextIDName both live under the
// project's "{room}/" prefix, so the existing Phase-2 prefix sweep in Delete
// already covers them — this pins that behavior.
func TestGCSDelete_RemovesSnapshots_Phase2(t *testing.T) {
	ctx := context.Background()
	client, bucket := newFakeGCS(t)
	a, err := New(Options{Client: client, Bucket: bucket, Locker: NewNoLock(), Phase2: true})
	if err != nil {
		t.Fatalf("gcs.New phase2: %v", err)
	}
	if _, err := a.SaveSnapshot(ctx, "00000000-0000-0000-0000-000000000001", "lbl", []byte("state")); err != nil {
		t.Fatalf("SaveSnapshot: %v", err)
	}
	if err := a.Delete(ctx, "00000000-0000-0000-0000-000000000001"); err != nil {
		t.Fatalf("Delete: %v", err)
	}
	got, err := a.ListSnapshots(ctx, "00000000-0000-0000-0000-000000000001")
	if err != nil {
		t.Fatalf("ListSnapshots: %v", err)
	}
	if len(got) != 0 {
		t.Fatalf("after Delete(room) snapshots = %+v, want none", got)
	}
}
