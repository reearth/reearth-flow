package gcs

import (
	"context"
	"errors"
	"strings"
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

// TestGCSSnapshot_LostCounterDoesNotOverwriteSnapshots: the id counter is a fast
// path, not the source of truth. If it is trusted alone, a room that loses its
// counter restarts allocation at id 1 and silently overwrites live snapshot 1 —
// destroying its payload, label and timestamp with no error to anyone.
//
// This is routinely reachable rather than theoretical: Delete removes objects one
// at a time and unlocked, so a cancelled request can take the counter while
// snapshots survive. With auto-versioning on (the default, every 15m) the room's
// history is then eaten one snapshot at a time.
func TestGCSSnapshot_LostCounterDoesNotOverwriteSnapshots(t *testing.T) {
	ctx := context.Background()
	client, bucket := newFakeGCS(t)
	a, err := New(Options{Client: client, Bucket: bucket, Locker: NewNoLock()})
	if err != nil {
		t.Fatalf("gcs.New: %v", err)
	}
	const room = "room1"

	for _, s := range []string{"state-1", "state-2", "state-3"} {
		if _, err := a.SaveSnapshot(ctx, room, s, []byte(s)); err != nil {
			t.Fatalf("SaveSnapshot(%s): %v", s, err)
		}
	}

	// Simulate the interrupted-Delete outcome: counter gone, snapshots intact.
	if err := a.store.delete(ctx, a.layout.SnapNextIDName(DocID(room))); err != nil {
		t.Fatalf("delete counter: %v", err)
	}

	id, err := a.SaveSnapshot(ctx, room, "after-counter-loss", []byte("NEW"))
	if err != nil {
		t.Fatalf("SaveSnapshot after counter loss: %v", err)
	}
	if id <= 3 {
		t.Fatalf("new snapshot got id %d, want > 3: a reused id overwrites an existing snapshot", id)
	}

	// The originals must all still be readable, byte-for-byte.
	for i, want := range []string{"state-1", "state-2", "state-3"} {
		got, err := a.GetSnapshotState(ctx, room, int64(i+1))
		if err != nil {
			t.Fatalf("GetSnapshotState(%d): %v", i+1, err)
		}
		if string(got) != want {
			t.Fatalf("snapshot %d = %q, want %q (it was overwritten)", i+1, got, want)
		}
	}
	if snaps, err := a.ListSnapshots(ctx, room); err != nil {
		t.Fatalf("ListSnapshots: %v", err)
	} else if len(snaps) != 4 {
		t.Fatalf("len(snapshots) = %d, want 4", len(snaps))
	}
}

// TestGCSSnapshot_CorruptCounterDoesNotOverwrite: same hazard via a different
// route. A counter that fails to parse must not be silently treated as "start
// over at 1"; it must fall back to the ids that actually exist.
func TestGCSSnapshot_CorruptCounterDoesNotOverwrite(t *testing.T) {
	ctx := context.Background()
	client, bucket := newFakeGCS(t)
	a, err := New(Options{Client: client, Bucket: bucket, Locker: NewNoLock()})
	if err != nil {
		t.Fatalf("gcs.New: %v", err)
	}
	const room = "room1"
	if _, err := a.SaveSnapshot(ctx, room, "first", []byte("state-1")); err != nil {
		t.Fatalf("SaveSnapshot: %v", err)
	}
	if err := a.store.put(ctx, a.layout.SnapNextIDName(DocID(room)), []byte("not-a-number")); err != nil {
		t.Fatalf("corrupt counter: %v", err)
	}

	id, err := a.SaveSnapshot(ctx, room, "second", []byte("state-2"))
	if err != nil {
		t.Fatalf("SaveSnapshot with corrupt counter: %v", err)
	}
	if id == 1 {
		t.Fatal("reused id 1, overwriting the existing snapshot")
	}
	got, err := a.GetSnapshotState(ctx, room, 1)
	if err != nil {
		t.Fatalf("GetSnapshotState(1): %v", err)
	}
	if string(got) != "state-1" {
		t.Fatalf("snapshot 1 = %q, want state-1", got)
	}
}

// TestGCSSnapshot_WriteIsCreateOnly asserts the storage-level guarantee the two
// tests above depend on. Snapshot records are write-once (ids are never reused
// within a room), so the write must REFUSE an existing name rather than
// overwrite it. Without this precondition an id collision from any cause — a
// stale counter, an expired lock lease — destroys bytes instead of erroring.
func TestGCSSnapshot_WriteIsCreateOnly(t *testing.T) {
	ctx := context.Background()
	client, bucket := newFakeGCS(t)
	a, err := New(Options{Client: client, Bucket: bucket, Locker: NewNoLock()})
	if err != nil {
		t.Fatalf("gcs.New: %v", err)
	}
	name := a.layout.SnapVersionName(DocID("room1"), 1)
	if err := a.store.putWithMetaIfAbsent(ctx, name, []byte("original"), nil); err != nil {
		t.Fatalf("first write: %v", err)
	}
	err = a.store.putWithMetaIfAbsent(ctx, name, []byte("clobber"), nil)
	if !errors.Is(err, errObjectExists) {
		t.Fatalf("second write err = %v, want errObjectExists", err)
	}
	got, err := a.store.get(ctx, name)
	if err != nil {
		t.Fatalf("get: %v", err)
	}
	if string(got) != "original" {
		t.Fatalf("object = %q, want %q: the refused write still overwrote it", got, "original")
	}
}

// TestGCSDelete_RemovesIDCounter pins the counter's lifecycle directly. It was
// previously only covered as a side effect of a RoomLister conformance case, and
// the counter's removal is exactly what makes id reuse possible, so it deserves
// its own assertion in both layouts.
func TestGCSDelete_RemovesIDCounter(t *testing.T) {
	for _, phase2 := range []bool{false, true} {
		ctx := context.Background()
		client, bucket := newFakeGCS(t)
		a, err := New(Options{Client: client, Bucket: bucket, Locker: NewNoLock(), Phase2: phase2})
		if err != nil {
			t.Fatalf("phase2=%v gcs.New: %v", phase2, err)
		}
		const room = "room1"
		if _, err := a.SaveSnapshot(ctx, room, "x", []byte("s")); err != nil {
			t.Fatalf("phase2=%v SaveSnapshot: %v", phase2, err)
		}
		if err := a.Delete(ctx, room); err != nil {
			t.Fatalf("phase2=%v Delete: %v", phase2, err)
		}
		if _, err := a.store.get(ctx, a.layout.SnapNextIDName(DocID(room))); err != errNotFound {
			t.Fatalf("phase2=%v: id counter survived Delete (err=%v)", phase2, err)
		}
	}
}

// TestGCSSnapshot_NextIDDerivesFromObjectsWhenCounterUnusable pins nextSnapID
// itself, because the end-to-end tests above cannot: the create-only write plus
// retry masks a bad allocation, so they still pass even if the counter is trusted
// blindly. That layering is deliberate (the precondition is what prevents data
// loss), but it means the derive-from-reality behaviour needs its own assertion
// or it could be removed silently, leaving every post-counter-loss save to burn a
// failed write and a retry.
func TestGCSSnapshot_NextIDDerivesFromObjectsWhenCounterUnusable(t *testing.T) {
	ctx := context.Background()
	client, bucket := newFakeGCS(t)
	a, err := New(Options{Client: client, Bucket: bucket, Locker: NewNoLock()})
	if err != nil {
		t.Fatalf("gcs.New: %v", err)
	}
	const room = "room1"
	d := DocID(room)
	for i := 0; i < 3; i++ {
		if _, err := a.SaveSnapshot(ctx, room, "x", []byte("s")); err != nil {
			t.Fatalf("SaveSnapshot: %v", err)
		}
	}

	for _, tc := range []struct {
		name  string
		setup func()
	}{
		{"counter missing", func() { _ = a.store.delete(ctx, a.layout.SnapNextIDName(d)) }},
		{"counter unparseable", func() { _ = a.store.put(ctx, a.layout.SnapNextIDName(d), []byte("xyz")) }},
		{"counter below reality", func() { _ = a.store.put(ctx, a.layout.SnapNextIDName(d), []byte("0")) }},
	} {
		t.Run(tc.name, func(t *testing.T) {
			tc.setup()
			got, err := a.nextSnapID(ctx, d)
			if err != nil {
				t.Fatalf("nextSnapID: %v", err)
			}
			if got != 4 {
				t.Fatalf("nextSnapID = %d, want 4 (max stored id 3 + 1); returning a live id would target an existing snapshot", got)
			}
		})
	}
}

// TestGCSSnapshot_Phase2ReadsLegacyRootSnapshots: flipping
// REEARTH_FLOW_GCS_PHASE2 must not hide the version history of rooms written
// before the cutover.
//
// This is the nastiest shape of failure this subsystem can produce: document
// state keeps loading correctly via the existing dual-read, so the flag flip
// looks clean, while every existing room's history panel silently empties. No
// error, nothing in the logs, and to a user indistinguishable from having lost
// their versions. Every other layout-scoped read here already falls back to the
// legacy root, and Delete already sweeps it — the read path was the gap.
func TestGCSSnapshot_Phase2ReadsLegacyRootSnapshots(t *testing.T) {
	ctx := context.Background()
	client, bucket := newFakeGCS(t)

	// Pre-cutover: a Phase-1 adapter writes the room's snapshots.
	p1, err := New(Options{Client: client, Bucket: bucket, Locker: NewNoLock()})
	if err != nil {
		t.Fatalf("gcs.New phase1: %v", err)
	}
	const room = "550e8400-e29b-41d4-a716-446655440099"
	want := map[int64]string{}
	for _, s := range []string{"legacy-1", "legacy-2", "legacy-3"} {
		id, err := p1.SaveSnapshot(ctx, room, s, []byte(s))
		if err != nil {
			t.Fatalf("phase1 SaveSnapshot(%s): %v", s, err)
		}
		want[id] = s
	}

	// Post-cutover: the same bucket, read through a Phase-2 adapter.
	p2, err := New(Options{Client: client, Bucket: bucket, Locker: NewNoLock(), Phase2: true})
	if err != nil {
		t.Fatalf("gcs.New phase2: %v", err)
	}

	got, err := p2.ListSnapshots(ctx, room)
	if err != nil {
		t.Fatalf("phase2 ListSnapshots: %v", err)
	}
	if len(got) != len(want) {
		t.Fatalf("phase2 ListSnapshots returned %d snapshots, want %d: the flag flip hid the room's history", len(got), len(want))
	}
	for _, sn := range got {
		state, err := p2.GetSnapshotState(ctx, room, sn.ID)
		if err != nil {
			t.Fatalf("phase2 GetSnapshotState(%d): %v", sn.ID, err)
		}
		if string(state) != want[sn.ID] {
			t.Fatalf("phase2 snapshot %d = %q, want %q", sn.ID, state, want[sn.ID])
		}
		if sn.Label != want[sn.ID] {
			t.Fatalf("phase2 snapshot %d label = %q, want %q", sn.ID, sn.Label, want[sn.ID])
		}
	}

	// A Phase-2 save must not collide with a legacy id the merged list exposes.
	newID, err := p2.SaveSnapshot(ctx, room, "post-cutover", []byte("new"))
	if err != nil {
		t.Fatalf("phase2 SaveSnapshot: %v", err)
	}
	if _, clash := want[newID]; clash {
		t.Fatalf("phase2 allocated id %d, which a legacy snapshot already uses", newID)
	}
	merged, err := p2.ListSnapshots(ctx, room)
	if err != nil {
		t.Fatalf("phase2 ListSnapshots after save: %v", err)
	}
	if len(merged) != len(want)+1 {
		t.Fatalf("merged list has %d entries, want %d", len(merged), len(want)+1)
	}
	// The legacy payloads must survive the new write untouched.
	for id, w := range want {
		state, err := p2.GetSnapshotState(ctx, room, id)
		if err != nil {
			t.Fatalf("legacy snapshot %d after phase2 save: %v", id, err)
		}
		if string(state) != w {
			t.Fatalf("legacy snapshot %d = %q, want %q: the phase-2 save clobbered it", id, state, w)
		}
	}
}

// TestSnapVersionIDFromName_RejectsNonSnapshotNames: the parser must validate,
// not just extract. Without a marker check, "everything after the last
// separator" makes any all-decimal tail look like a snapshot id — so a room
// whose name hex-encodes to digits has its own id COUNTER read as a snapshot,
// and any unrelated path ending in a number does the same.
//
// It is currently safe only because the one caller pre-filters by prefix. That
// makes correctness depend on caller discipline for a method on the Layout
// interface, and ListRooms already establishes the habit of scanning broader
// prefixes, so the next loose caller would mint phantom ids.
func TestSnapVersionIDFromName_RejectsNonSnapshotNames(t *testing.T) {
	legacy := LegacyRootLayout{}
	folder := ProjectFolderLayout{}

	// The room names that used to make a counter object parse as a snapshot.
	for _, room := range []string{"0", "9", "12345", "room1"} {
		d := DocID(room)
		if id, ok := legacy.SnapVersionIDFromName(legacy.SnapNextIDName(d)); ok {
			t.Errorf("legacy: counter object for room %q parsed as snapshot id %d", room, id)
		}
		if id, ok := folder.SnapVersionIDFromName(folder.SnapNextIDName(d)); ok {
			t.Errorf("folder: counter object for room %q parsed as snapshot id %d", room, id)
		}
		// A real snapshot name must still round-trip.
		if got, ok := legacy.SnapVersionIDFromName(legacy.SnapVersionName(d, 42)); !ok || got != 42 {
			t.Errorf("legacy: SnapVersionName(%q,42) -> (%d,%v), want (42,true)", room, got, ok)
		}
		if got, ok := folder.SnapVersionIDFromName(folder.SnapVersionName(d, 42)); !ok || got != 42 {
			t.Errorf("folder: SnapVersionName(%q,42) -> (%d,%v), want (42,true)", room, got, ok)
		}
	}

	// An unrelated path whose tail happens to be numeric.
	if id, ok := folder.SnapVersionIDFromName("proj/anything/77"); ok {
		t.Errorf("folder: %q parsed as snapshot id %d, want rejected", "proj/anything/77", id)
	}
	// An update-log object must not look like a snapshot either.
	if id, ok := legacy.SnapVersionIDFromName(legacy.UpdateName(DocID("room1"), 1, 5)); ok {
		t.Errorf("legacy: update-log name parsed as snapshot id %d, want rejected", id)
	}
	if id, ok := folder.SnapVersionIDFromName(folder.UpdateName(DocID("room1"), FolderOID, 5)); ok {
		t.Errorf("folder: update-log name parsed as snapshot id %d, want rejected", id)
	}
}

// TestGCSSnapshot_RejectsUnstorableLabel: the label goes into GCS custom object
// metadata, which caps at 8 KiB and silently mangles invalid UTF-8. Both
// failures are invisible without a guard — an oversized label surfaces as an
// opaque 400 AFTER the id counter has been consumed (so every retry burns an
// id), and bad UTF-8 is accepted and comes back different. Guarded in the
// adapter rather than only at the HTTP boundary because auto-versioning is a
// second caller.
func TestGCSSnapshot_RejectsUnstorableLabel(t *testing.T) {
	ctx := context.Background()
	client, bucket := newFakeGCS(t)
	a, err := New(Options{Client: client, Bucket: bucket, Locker: NewNoLock()})
	if err != nil {
		t.Fatalf("gcs.New: %v", err)
	}

	for _, tc := range []struct {
		name  string
		label string
	}{
		{"oversized", strings.Repeat("a", maxLabelBytes+1)},
		{"invalid utf-8", string([]byte{0xff, 0xfe, 0x00})},
	} {
		t.Run(tc.name, func(t *testing.T) {
			if _, err := a.SaveSnapshot(ctx, "room1", tc.label, []byte("s")); !errors.Is(err, ErrInvalidSnapshotLabel) {
				t.Fatalf("SaveSnapshot err = %v, want ErrInvalidSnapshotLabel", err)
			}
			// The rejection must happen before an id is consumed.
			if snaps, err := a.ListSnapshots(ctx, "room1"); err != nil {
				t.Fatalf("ListSnapshots: %v", err)
			} else if len(snaps) != 0 {
				t.Fatalf("a snapshot was recorded despite the bad label: %+v", snaps)
			}
		})
	}

	// A label exactly at the limit, and a multi-byte one, must both be accepted.
	if _, err := a.SaveSnapshot(ctx, "room1", strings.Repeat("a", maxLabelBytes), []byte("s")); err != nil {
		t.Fatalf("label at the limit was rejected: %v", err)
	}
	if _, err := a.SaveSnapshot(ctx, "room1", "リリース前", []byte("s")); err != nil {
		t.Fatalf("multi-byte UTF-8 label was rejected: %v", err)
	}
}

// TestGCSDelete_RemovesLegacyRootSnapshots_Phase2 covers the legacy-root
// snapshot cleanup in deleteLegacyRoot, which no test reached: removing either
// of its two snapshot lines left the whole suite passing.
// TestDeleteRemovesNamedSnapshots_Phase2 looks like it covers this and does not,
// because its SaveSnapshot writes through the PHASE-2 layout, so the legacy
// branch never runs. Here the snapshots are written by a Phase-1 adapter first.
//
// If it regresses, a deleted project's full document state survives in the
// bucket indefinitely — a snapshot is a complete copy — which is both a storage
// cost and a data-deletion-guarantee problem.
func TestGCSDelete_RemovesLegacyRootSnapshots_Phase2(t *testing.T) {
	ctx := context.Background()
	client, bucket := newFakeGCS(t)
	const room = "550e8400-e29b-41d4-a716-446655440099"

	p1, err := New(Options{Client: client, Bucket: bucket, Locker: NewNoLock()})
	if err != nil {
		t.Fatalf("gcs.New phase1: %v", err)
	}
	for i := 0; i < 3; i++ {
		if _, err := p1.SaveSnapshot(ctx, room, "legacy", []byte("state")); err != nil {
			t.Fatalf("phase1 SaveSnapshot: %v", err)
		}
	}

	p2, err := New(Options{Client: client, Bucket: bucket, Locker: NewNoLock(), Phase2: true})
	if err != nil {
		t.Fatalf("gcs.New phase2: %v", err)
	}
	if _, err := p2.SaveSnapshot(ctx, room, "primary", []byte("state")); err != nil {
		t.Fatalf("phase2 SaveSnapshot: %v", err)
	}
	if err := p2.Delete(ctx, room); err != nil {
		t.Fatalf("phase2 Delete: %v", err)
	}

	// Assert at the OBJECT level: a ListSnapshots check could pass while objects
	// linger, which is exactly the failure being guarded against.
	leg := LegacyRootLayout{}
	for _, prefix := range []string{leg.SnapVersionPrefix(DocID(room)), legacySnapshotPrefix(DocID(room))} {
		objs, err := p2.store.list(ctx, prefix)
		if err != nil {
			t.Fatalf("list %q: %v", prefix, err)
		}
		if len(objs) != 0 {
			t.Fatalf("legacy-root objects survived Delete under %q: %v", prefix, objs)
		}
	}
	if _, err := p2.store.get(ctx, leg.SnapNextIDName(DocID(room))); err != errNotFound {
		t.Fatalf("legacy id counter survived Delete (err=%v)", err)
	}
}
