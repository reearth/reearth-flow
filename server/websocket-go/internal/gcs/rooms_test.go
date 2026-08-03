package gcs

import (
	"context"
	"reflect"
	"sort"
	"testing"

	"github.com/reearth/ygo/persistence"
)

func TestRoomListerConformance_GCS(t *testing.T) {
	persistence.RunRoomListerConformance(t, func() persistence.SnapshotVersionedPersistence {
		client, bucket := newFakeGCS(t)
		a, err := New(Options{Client: client, Bucket: bucket, Locker: NewNoLock()})
		if err != nil {
			t.Fatalf("gcs.New: %v", err)
		}
		return a
	})
}

// TestListRoomsPhase2 exercises the a.phase2 branch of ListRooms, which the
// conformance suite above never reaches (its factory always builds a Phase-1
// adapter). It follows the existing Phase-2-construction pattern used
// throughout this package (New(Options{..., Phase2: true})) rather than
// introducing a new one, and covers: a room with an update log, a
// snapshot-only room (no update log — the case most likely to be missed),
// and room names containing a colon and a non-ASCII character, asserting the
// ORIGINAL name comes back unencoded.
//
// A room name containing '/' is intentionally NOT covered here: Phase 2 uses
// the room id raw as a "{room}/" path prefix, and ValidateDocIDForPrefix
// rejects any id containing '/' for that reason. The Phase-1 conformance run
// above already covers the slash case.
func TestListRoomsPhase2(t *testing.T) {
	ctx := context.Background()
	client, bucket := newFakeGCS(t)
	a, err := New(Options{Client: client, Bucket: bucket, Locker: NewNoLock(), Phase2: true})
	if err != nil {
		t.Fatalf("gcs.New phase2: %v", err)
	}

	// Room with an update log.
	if _, err := a.AppendUpdate(ctx, "with-updates", []byte{1}); err != nil {
		t.Fatalf("AppendUpdate(with-updates): %v", err)
	}

	// Snapshot-only room: no update log, must still be listed.
	if _, err := a.SaveSnapshot(ctx, "snaponly", "lbl", []byte("state")); err != nil {
		t.Fatalf("SaveSnapshot(snaponly): %v", err)
	}

	// Room names with special characters must round-trip exactly.
	if _, err := a.AppendUpdate(ctx, "with:colon", []byte{1}); err != nil {
		t.Fatalf("AppendUpdate(with:colon): %v", err)
	}
	if _, err := a.AppendUpdate(ctx, "üñïçödé", []byte{1}); err != nil {
		t.Fatalf("AppendUpdate(non-ascii): %v", err)
	}

	got, err := a.ListRooms(ctx)
	if err != nil {
		t.Fatalf("ListRooms: %v", err)
	}
	sort.Strings(got)
	want := []string{"snaponly", "with-updates", "with:colon", "üñïçödé"}
	sort.Strings(want)
	if !reflect.DeepEqual(got, want) {
		t.Fatalf("ListRooms = %v, want %v", got, want)
	}
}
