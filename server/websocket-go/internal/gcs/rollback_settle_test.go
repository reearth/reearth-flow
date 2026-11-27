package gcs

import (
	"context"
	"errors"
	"testing"

	"github.com/reearth/ygo/crdt"
	"github.com/reearth/ygo/persistence"
)

// stopAfterDocV2 runs a rollback of room to target that stops right after
// publishing doc_v2, leaving its mark for a later holder to finish.
func stopAfterDocV2(t *testing.T, a *Adapter, room string, target persistence.Version, rolled []byte) {
	t.Helper()
	a.afterRollbackDocV2 = func() bool { return true }
	defer func() { a.afterRollbackDocV2 = nil }()
	if err := a.PruneAfter(context.Background(), room, target, rolled); err != nil {
		t.Fatalf("PruneAfter: %v", err)
	}
}

// once wraps fn so it runs on the first call only: a hook fires again inside
// the nested call it makes.
func once(fn func()) func() {
	fired := false
	return func() {
		if fired {
			return
		}
		fired = true
		fn()
	}
}

// TestSettle_DoesNotOverwriteARollbackCommittedWhileItPaused: a holder finishing
// a stopped rollback can lose its lease after reading the mark. If another
// holder finishes it and commits a newer rollback meanwhile, the first one's
// checkpoint and ceiling writes must fail rather than put the older target back.
func TestSettle_DoesNotOverwriteARollbackCommittedWhileItPaused(t *testing.T) {
	t.Parallel()
	ctx := context.Background()
	a, _, _ := newFenceAdapters(t)
	const room = "settle-paused"
	oid, rolledTo3 := seedFence(t, a, room)
	rolledTo2, err := a.MaterializeAt(ctx, room, persistence.Version(2))
	if err != nil {
		t.Fatalf("MaterializeAt(2): %v", err)
	}
	stopAfterDocV2(t, a, room, persistence.Version(3), rolledTo3)

	a.afterSettleRead = once(func() {
		// Another holder: it settles the stopped rollback and commits its own.
		if err := a.PruneAfter(ctx, room, persistence.Version(2), rolledTo2); err != nil {
			t.Errorf("the other rollback: %v", err)
		}
	})
	if err := a.settleRollback(ctx, room, oid); !errors.Is(err, errStateMoved) {
		t.Errorf("the paused settle = %v, want errStateMoved", err)
	}
	a.afterSettleRead = nil

	m, err := a.readMarks(ctx, room)
	if err != nil {
		t.Fatalf("readMarks: %v", err)
	}
	if m.rolling || m.cp != 2 || m.ceil != 2 {
		t.Errorf("marks: rolling=%v cp=%d ceil=%d, want the committed rollback's 2", m.rolling, m.cp, m.ceil)
	}
	if s, err := loadText(t, a, room); err != nil || s != "ab" {
		t.Errorf("Load = %q, %v; want the committed rollback's %q", s, err, "ab")
	}
}

// TestSettle_KeepsAnEditAppendedWhileItCleansUp: finishing a stopped rollback
// deletes the log above its target. An append can clear the ceiling and take a
// freed clock in between, and deleting by name then removed that saved edit.
func TestSettle_KeepsAnEditAppendedWhileItCleansUp(t *testing.T) {
	t.Parallel()
	ctx := context.Background()
	a, _, _ := newFenceAdapters(t)
	const room = "settle-cleanup"
	oid, rolled := seedFence(t, a, room)
	stopAfterDocV2(t, a, room, persistence.Version(3), rolled)

	a.beforeRollbackDelete = once(func() { appendOn(t, a, room, 77, rolled, "Z") })
	if err := a.settleRollback(ctx, room, oid); err != nil {
		t.Errorf("settleRollback = %v, want the cleanup left to the append that did it", err)
	}
	a.beforeRollbackDelete = nil

	updates, err := a.listUpdates(ctx, room, oid)
	if err != nil {
		t.Fatalf("listUpdates: %v", err)
	}
	if _, ok := updates[4]; !ok {
		t.Error("the edit appended at 4 was deleted")
	}
	if s, err := loadText(t, a, room); err != nil || s != "abcZ" {
		t.Errorf("Load = %q, %v; want %q", s, err, "abcZ")
	}
}

// TestRollback_SucceedsWhenAnAppendFinishedItsCleanup: once a rollback has
// committed, an append that clears its ceiling before the cleanup ends takes
// the cleanup over. The rollback took effect, so reporting a failure would only
// invite the client to restore again over the edit made since.
func TestRollback_SucceedsWhenAnAppendFinishedItsCleanup(t *testing.T) {
	t.Parallel()
	ctx := context.Background()
	a, _, _ := newFenceAdapters(t)
	const room = "rollback-cleanup"
	_, rolled := seedFence(t, a, room)

	a.beforeRollbackDelete = once(func() { appendOn(t, a, room, 77, rolled, "Z") })
	got, err := a.Rollback(ctx, room, persistence.Version(3), nil)
	a.beforeRollbackDelete = nil
	if err != nil {
		t.Fatalf("Rollback = %v, want success", err)
	}
	if s := textOf(t, got); s != "abc" {
		t.Errorf("Rollback returned %q, want %q", s, "abc")
	}
	if s, err := loadText(t, a, room); err != nil || s != "abcZ" {
		t.Errorf("Load = %q, %v; want %q", s, err, "abcZ")
	}
}

// TestSettle_LeavesANewerRollbacksMarkInPlace: a holder abandoning a stopped
// rollback can lose its lease after fencing doc_v2. If another holder abandons
// it too and a newer rollback marks the ceiling and publishes partway, clearing
// that newer mark let readers serve its base with the edits it removed on top.
func TestSettle_LeavesANewerRollbacksMarkInPlace(t *testing.T) {
	t.Parallel()
	ctx := context.Background()
	a, _, _ := newFenceAdapters(t)
	const room = "settle-abandon"
	oid, rolledTo3 := seedFence(t, a, room)
	rolledTo2, err := a.MaterializeAt(ctx, room, persistence.Version(2))
	if err != nil {
		t.Fatalf("MaterializeAt(2): %v", err)
	}
	a.afterRollbackMark = func() bool { return true }
	if err := a.PruneAfter(ctx, room, persistence.Version(3), rolledTo3); err != nil {
		t.Fatalf("PruneAfter: %v", err)
	}
	a.afterRollbackMark = nil

	a.afterSettleFence = once(func() {
		// Another holder abandons the same rollback, then a newer one stops right
		// after writing its checkpoint, with its mark still set.
		a.afterRollbackCheckpoint = func() bool { return true }
		if err := a.PruneAfter(ctx, room, persistence.Version(2), rolledTo2); err != nil {
			t.Errorf("the newer rollback: %v", err)
		}
		a.afterRollbackCheckpoint = nil
	})
	if err := a.settleRollback(ctx, room, oid); !errors.Is(err, errStateMoved) {
		t.Errorf("the paused settle = %v, want errStateMoved", err)
	}
	a.afterSettleFence = nil

	m, err := a.readMarks(ctx, room)
	if err != nil {
		t.Fatalf("readMarks: %v", err)
	}
	if !m.rolling || m.mark.target != 2 {
		t.Fatalf("marks: rolling=%v target=%d, want the newer rollback's mark kept", m.rolling, m.mark.target)
	}
	if s, err := loadText(t, a, room); err != nil || s != "ab" {
		t.Errorf("Load = %q, %v; want the newer rollback finished, %q", s, err, "ab")
	}
}

// TestMaterializeAt_RereadsWhenFoldingMeetsARollback: a parked base makes
// MaterializeAt read checkpointed objects after its marks check. A rollback and
// an append in between put a different edit under a listed clock, and the
// result mixed the old base with the new history.
func TestMaterializeAt_RereadsWhenFoldingMeetsARollback(t *testing.T) {
	t.Parallel()
	ctx := context.Background()
	a := newParkedTestAdapter(t)
	first, second := editPair(t)
	fs := fillers(t, 9)
	appendAll(t, a, first)
	appendAll(t, a, fs...) // clocks 1..10, the checkpoint at 10
	behind := crdt.New()
	for _, u := range append(fs, second) {
		if err := crdt.ApplyUpdateV1(behind, u, nil); err != nil {
			t.Fatal(err)
		}
	}
	if behind.PendingStats().Items == 0 {
		t.Fatal("setup: the base parks nothing")
	}
	writeOldCheckpoint(t, a, "room", crdt.EncodeStateAsUpdateV1(behind, nil))
	rolledTo9, err := a.MaterializeAt(ctx, "room", persistence.Version(9))
	if err != nil {
		t.Fatalf("MaterializeAt(9): %v", err)
	}
	if v := fillerValue(t, rolledTo9); v != 7 {
		t.Fatalf("setup: version 9 holds filler %v, want 7", v)
	}

	ran := false
	a.afterMaterializeCheck = once(func() {
		ran = true
		// Roll back to 9 and append a different edit, which takes clock 10.
		if err := a.PruneAfter(ctx, "room", persistence.Version(9), rolledTo9); err != nil {
			t.Errorf("PruneAfter: %v", err)
		}
		if v, err := a.AppendUpdate(ctx, "room", textEdit(9, "q", "Q")); err != nil || v != 10 {
			t.Errorf("AppendUpdate = %d, %v; want version 10", v, err)
		}
	})
	got, err := a.MaterializeAt(ctx, "room", persistence.Version(10))
	a.afterMaterializeCheck = nil
	if !ran {
		t.Fatal("setup: the hook never ran")
	}
	if err != nil {
		t.Fatalf("MaterializeAt(10): %v", err)
	}
	if v := fillerValue(t, got); v != 7 {
		t.Errorf("MaterializeAt(10) holds filler %v, want 7: the base from before the rollback was mixed in", v)
	}
	if s, _ := loaded(t, got, "q"); s != "Q" {
		t.Errorf("MaterializeAt(10) has q = %q, want the new version 10's %q", s, "Q")
	}
}
