package gcs

import (
	"context"
	"errors"
	"testing"

	"github.com/reearth/ygo/crdt"
	"github.com/reearth/ygo/persistence"
)

// appendOn appends each letter of s as its own update, made by client on top of
// base, and returns the state after the last one.
func appendOn(t *testing.T, a *Adapter, room string, client uint64, base []byte, s string) []byte {
	t.Helper()
	d := crdt.New(crdt.WithClientID(crdt.ClientID(client)))
	if len(base) > 0 {
		if err := crdt.ApplyUpdateV1(d, base, nil); err != nil {
			t.Fatalf("apply base: %v", err)
		}
	}
	txt := d.GetText("t")
	for _, r := range s {
		sv := d.StateVector()
		ch := string(r)
		d.Transact(func(txn *crdt.Transaction) { txt.Insert(txn, txt.Len(), ch, nil) })
		inc, err := crdt.DiffUpdateV1(crdt.EncodeStateAsUpdateV1(d, nil), sv)
		if err != nil {
			t.Fatalf("diff: %v", err)
		}
		if _, err := a.AppendUpdate(context.Background(), room, inc); err != nil {
			t.Fatalf("AppendUpdate(%q): %v", ch, err)
		}
	}
	return crdt.EncodeStateAsUpdateV1(d, nil)
}

// TestPruneAfter_CannotPublishOnceItsMarkWasAbandoned: a rollback whose lease
// lapsed right after marking can resume once another holder has abandoned its
// mark. Abandoning changed only the ceiling, so its doc_v2 write used to land on
// the generation it read before marking and publish the rollback with no mark,
// where an append's compaction made the edits it dropped unrecoverable.
func TestPruneAfter_CannotPublishOnceItsMarkWasAbandoned(t *testing.T) {
	t.Parallel()
	for _, tc := range []struct {
		name    string
		letters int
	}{
		{name: "doc_v2 present", letters: 15}, // the tenth leaves a doc_v2
		{name: "doc_v2 absent", letters: 6},
	} {
		t.Run(tc.name, func(t *testing.T) {
			t.Parallel()
			ctx := context.Background()
			a, _, _ := newFenceAdapters(t)
			room := "abandon-" + string(rune('a'+tc.letters))
			full := appendLetters(t, a, room, tc.letters, nil)
			want := textOf(t, full)
			rolled, err := a.MaterializeAt(ctx, room, persistence.Version(3))
			if err != nil {
				t.Fatalf("MaterializeAt(3): %v", err)
			}

			fired := false
			a.afterRollbackMark = func() bool {
				if fired {
					return false
				}
				fired = true
				// Another holder, once this one's lease lapsed: its Load settles the
				// stopped-looking rollback by abandoning it, then a client edits.
				if s, err := loadText(t, a, room); err != nil || s != want {
					t.Errorf("Load in the window = %q, %v; want the untouched %q", s, err, want)
				}
				full = appendOn(t, a, room, 77, full, "Z")
				return false
			}
			if err := a.PruneAfter(ctx, room, persistence.Version(3), rolled); !errors.Is(err, errStateMoved) {
				t.Errorf("the resumed rollback = %v, want errStateMoved", err)
			}
			a.afterRollbackMark = nil

			if s, err := loadText(t, a, room); err != nil || s != want+"Z" {
				t.Errorf("Load = %q, %v; want every edit kept, %q", s, err, want+"Z")
			}
			m, err := a.readMarks(ctx, room)
			if err != nil {
				t.Fatalf("readMarks: %v", err)
			}
			if m.rolling || (m.hasCov && m.cov.rollbackOp != "") {
				t.Errorf("marks: rolling=%v rollbackOp=%q, want no trace of the abandoned rollback", m.rolling, m.cov.rollbackOp)
			}
		})
	}
}

// TestFlush_DropsACandidateWhoseMarksWereReadMidRollback: marks read while a
// rollback is running show no ceiling and the epoch it already wrote. Once an
// append cleared its ceiling, nothing told the flush that the rollback had
// committed, and its candidate put the removed edits back.
func TestFlush_DropsACandidateWhoseMarksWereReadMidRollback(t *testing.T) {
	t.Parallel()
	ctx := context.Background()
	a, _, _ := newFenceAdapters(t)
	const room = "flush-mid-rollback"
	_, rolled := seedFence(t, a, room)
	live := appendLettersState(t, 6) // the live room, "abcdef", as the flush takes it

	var before marks
	a.afterRollbackMark = func() bool {
		var err error
		if before, err = a.flushMarks(ctx, room); err != nil {
			t.Errorf("flushMarks: %v", err)
		}
		return false
	}
	if err := a.PruneAfter(ctx, room, persistence.Version(3), rolled); err != nil {
		t.Fatalf("PruneAfter: %v", err)
	}
	a.afterRollbackMark = nil
	appendOn(t, a, room, 77, rolled, "Z")

	if err := a.flushSnapshotSince(ctx, room, live, before); err != nil {
		t.Fatalf("flushSnapshotSince: %v", err)
	}
	if s, err := loadText(t, a, room); err != nil || s != "abcZ" {
		t.Errorf("Load = %q, %v; want %q: the candidate built before the rollback was applied", s, err, "abcZ")
	}
}

// TestPruneAfter_BumpsTheEpochWhenItCommits: a flush that read the marks after
// the rollback announced itself but before it marked the ceiling sees neither
// a mark nor, later, a different epoch, unless committing rewrites the epoch.
func TestPruneAfter_BumpsTheEpochWhenItCommits(t *testing.T) {
	t.Parallel()
	ctx := context.Background()
	a, _, _ := newFenceAdapters(t)
	const room = "epoch-commit"
	_, rolled := seedFence(t, a, room)

	var announced int64
	a.afterRollbackMark = func() bool {
		var err error
		if announced, err = a.store.generation(ctx, a.rollbackEpochName(room)); err != nil {
			t.Errorf("generation: %v", err)
		}
		return false
	}
	if err := a.PruneAfter(ctx, room, persistence.Version(3), rolled); err != nil {
		t.Fatalf("PruneAfter: %v", err)
	}
	committed, err := a.store.generation(ctx, a.rollbackEpochName(room))
	if err != nil {
		t.Fatalf("generation: %v", err)
	}
	if committed == announced {
		t.Error("the epoch was not rewritten when the rollback committed")
	}
}

// TestSnapshotFromStore_StampsALegacyOnlyRoomAsCoverageZero: promoting a
// legacy-only room stamped the legacy head as primary coverage while primary
// appends restart at clock 1, so MaterializeAt at that clock used the imported
// base alone and dropped every primary edit, and a rollback to it lost them.
func TestSnapshotFromStore_StampsALegacyOnlyRoomAsCoverageZero(t *testing.T) {
	t.Parallel()
	ctx := context.Background()
	client, bucket := newFakeGCS(t)
	const room = "legacy-promote"
	seedLegacy(t, client, bucket, room) // four legacy updates
	a, err := New(Options{Client: client, Bucket: bucket, Locker: NewNoLock(), Phase2: true})
	if err != nil {
		t.Fatalf("New: %v", err)
	}
	if err := a.SnapshotFromStore(ctx, room); err != nil {
		t.Fatalf("SnapshotFromStore: %v", err)
	}
	lr, err := a.Load(ctx, room)
	if err != nil {
		t.Fatalf("Load: %v", err)
	}
	appendOn(t, a, room, 77, lr.Update, "WXYZ") // primary clocks 1..4

	want, err := loadText(t, a, room)
	if err != nil {
		t.Fatalf("Load: %v", err)
	}
	got, err := a.MaterializeAt(ctx, room, persistence.Version(4))
	if err != nil {
		t.Fatalf("MaterializeAt(4): %v", err)
	}
	if s := textOf(t, got); s != want {
		t.Errorf("MaterializeAt(4) = %q, want the head %q", s, want)
	}
}

// TestMaterializeAt_RefusesAVersionAboveAnActiveCeiling: a rollback to 3 that
// stopped before deleting 4..6 leaves them hidden by the ceiling. A stale request
// for 6 rebuilt them, and a rollback to it published the removed edits again.
func TestMaterializeAt_RefusesAVersionAboveAnActiveCeiling(t *testing.T) {
	t.Parallel()
	ctx := context.Background()
	a, _, _ := newFenceAdapters(t)
	const room = "above-ceiling"
	_, rolled := seedFence(t, a, room)

	a.SetCrashAfterCheckpoint(func() bool { return true })
	if err := a.PruneAfter(ctx, room, persistence.Version(3), rolled); err != nil {
		t.Fatalf("PruneAfter: %v", err)
	}
	a.SetCrashAfterCheckpoint(nil)

	if _, err := a.MaterializeAt(ctx, room, persistence.Version(6)); !errors.Is(err, ErrVersionNotRebuildable) {
		t.Errorf("MaterializeAt(6) above the ceiling = %v, want ErrVersionNotRebuildable", err)
	}
	got, err := a.MaterializeAt(ctx, room, persistence.Version(3))
	if err != nil {
		t.Fatalf("MaterializeAt(3): %v", err)
	}
	if s := textOf(t, got); s != "abc" {
		t.Errorf("MaterializeAt(3) = %q, want %q", s, "abc")
	}
}

// TestMaterializeAt_RereadsABaseReplacedWhileItRead: an append and a store
// snapshot between sampling the marks and reading doc_v2 replaced the base
// with one that already held version 11, which a rebuild of 10 then carried.
func TestMaterializeAt_RereadsABaseReplacedWhileItRead(t *testing.T) {
	t.Parallel()
	ctx := context.Background()
	a, _, _ := newFenceAdapters(t)
	const room = "base-swap"
	full := appendLetters(t, a, room, 10, nil) // doc_v2 covers exactly 10

	fired := false
	a.beforeMaterializeBase = func() {
		if fired {
			return
		}
		fired = true
		appendOn(t, a, room, 77, full, "K")
		if err := a.SnapshotFromStore(ctx, room); err != nil {
			t.Errorf("SnapshotFromStore: %v", err)
		}
	}
	got, err := a.MaterializeAt(ctx, room, persistence.Version(10))
	if err != nil {
		t.Fatalf("MaterializeAt(10): %v", err)
	}
	if !fired {
		t.Fatal("setup: the hook never ran")
	}
	if s := textOf(t, got); s != "abcdefghij" {
		t.Errorf("MaterializeAt(10) = %q, want %q", s, "abcdefghij")
	}
}

// TestRollback_RefusesWhenAnotherRollbackCommitsAfterTheRebuild: the handler
// rebuilt 10, another holder rolled back to 5 and a client added PQ, and the
// first rollback then published 10: the edits the other rollback removed came
// back and PQ, now below the checkpoint, was gone.
func TestRollback_RefusesWhenAnotherRollbackCommitsAfterTheRebuild(t *testing.T) {
	t.Parallel()
	ctx := context.Background()
	a, _, _ := newFenceAdapters(t)
	const room = "rollback-race"
	appendLetters(t, a, room, 12, nil)

	_, err := a.Rollback(ctx, room, persistence.Version(10), func() {
		rolledTo5, err := a.Rollback(ctx, room, persistence.Version(5), nil)
		if err != nil {
			t.Errorf("the other rollback: %v", err)
			return
		}
		appendOn(t, a, room, 77, rolledTo5, "PQ")
	})
	if err == nil {
		t.Error("the first rollback published after another one committed")
	}
	if s, err := loadText(t, a, room); err != nil || s != "abcdePQ" {
		t.Errorf("Load = %q, %v; want the committed rollback plus PQ, %q", s, err, "abcdePQ")
	}
}

// TestRollback_PublishesTheRebuiltVersion is the plain path: the rebuilt state
// is returned and published, with beforePublish run once in between.
func TestRollback_PublishesTheRebuiltVersion(t *testing.T) {
	t.Parallel()
	ctx := context.Background()
	a, _, _ := newFenceAdapters(t)
	const room = "rollback-plain"
	appendLetters(t, a, room, 12, nil)

	calls := 0
	got, err := a.Rollback(ctx, room, persistence.Version(5), func() { calls++ })
	if err != nil {
		t.Fatalf("Rollback: %v", err)
	}
	if calls != 1 {
		t.Errorf("beforePublish ran %d times, want once", calls)
	}
	if s := textOf(t, got); s != "abcde" {
		t.Errorf("returned %q, want %q", s, "abcde")
	}
	if s, err := loadText(t, a, room); err != nil || s != "abcde" {
		t.Errorf("Load = %q, %v; want %q", s, err, "abcde")
	}
}
