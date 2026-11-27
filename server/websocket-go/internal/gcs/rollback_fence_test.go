package gcs

import (
	"context"
	"errors"
	"testing"
	"time"

	"cloud.google.com/go/storage"
	"github.com/reearth/ygo/crdt"
	"github.com/reearth/ygo/persistence"
)

// heldLock is a Locker whose doc lock is always held by someone else, the way
// it is for every other instance while a PruneAfter runs.
type heldLock struct{}

func (heldLock) WithLock(context.Context, string, func(context.Context) error) error {
	return errors.New("gcs: lock acquisition timed out")
}

func (heldLock) TryWithLock(ctx context.Context, _ string, _ time.Duration, fn func(context.Context) error) error {
	return fn(ctx)
}

// seedFence appends "abcdef" as clocks 1..6 and returns the oid and the state
// at version 3, which a rollback to 3 publishes.
func seedFence(t *testing.T, a *Adapter, room string) (uint32, []byte) {
	t.Helper()
	appendLetters(t, a, room, 6, nil)
	oid, err := a.oidFor(context.Background(), room)
	if err != nil {
		t.Fatalf("oidFor: %v", err)
	}
	rolled, err := a.MaterializeAt(context.Background(), room, persistence.Version(3))
	if err != nil {
		t.Fatalf("MaterializeAt(3): %v", err)
	}
	if s := textOf(t, rolled); s != "abc" {
		t.Fatalf("setup: state at 3 = %q", s)
	}
	return oid, rolled
}

func loadText(t *testing.T, a *Adapter, room string) (string, error) {
	t.Helper()
	lr, err := a.Load(context.Background(), room)
	if err != nil {
		return "", err
	}
	return textOf(t, lr.Update), nil
}

func newFenceAdapters(t *testing.T) (*Adapter, *storage.Client, string) {
	t.Helper()
	client, bucket := newFakeGCS(t)
	a, err := New(Options{Client: client, Bucket: bucket, Locker: NewNoLock()})
	if err != nil {
		t.Fatalf("New: %v", err)
	}
	return a, client, bucket
}

// TestLoad_RefusesARollbackPublishedOnlyAsFarAsDocV2: a Load between
// PruneAfter's doc_v2 write and its ceiling write used to see matching marks and
// serve the rolled-back document with the old head and the old updates on top,
// putting back the edits the rollback removed.
func TestLoad_RefusesARollbackPublishedOnlyAsFarAsDocV2(t *testing.T) {
	t.Parallel()
	ctx := context.Background()
	a, _, _ := newFenceAdapters(t)
	const room = "fence-mid"
	oid, rolled := seedFence(t, a, room)

	var midErr error
	ran := false
	a.afterRollbackDocV2 = func() bool {
		ran = true
		_, midErr = a.readStateUnlocked(ctx, room, oid) // a Load that does not hold the lock
		return false
	}
	if err := a.PruneAfter(ctx, room, persistence.Version(3), rolled); err != nil {
		t.Fatalf("PruneAfter: %v", err)
	}
	if !ran {
		t.Fatal("setup: the hook never ran")
	}
	if !errors.Is(midErr, errStateMoved) {
		t.Errorf("Load between doc_v2 and the ceiling = %v, want errStateMoved", midErr)
	}
	if s, err := loadText(t, a, room); err != nil || s != "abc" {
		t.Errorf("Load after the rollback = %q, %v; want %q", s, err, "abc")
	}
}

// TestLoad_RollsForwardARollbackThatDiedAfterPublishingDocV2: the next holder
// of the lock finishes it from doc_v2's coverage instead of serving the mix.
func TestLoad_RollsForwardARollbackThatDiedAfterPublishingDocV2(t *testing.T) {
	t.Parallel()
	ctx := context.Background()
	a, _, _ := newFenceAdapters(t)
	const room = "fence-forward"
	oid, rolled := seedFence(t, a, room)

	a.afterRollbackDocV2 = func() bool { return true }
	if err := a.PruneAfter(ctx, room, persistence.Version(3), rolled); err != nil {
		t.Fatalf("PruneAfter: %v", err)
	}
	a.afterRollbackDocV2 = nil

	if s, err := loadText(t, a, room); err != nil || s != "abc" {
		t.Fatalf("Load = %q, %v; want %q", s, err, "abc")
	}
	m, err := a.readMarks(ctx, room)
	if err != nil {
		t.Fatalf("readMarks: %v", err)
	}
	if m.rolling || m.cp != 3 || !m.hasCeil || m.ceil != 3 {
		t.Errorf("after roll-forward: rolling=%v cp=%d ceil=%d(%v), want settled at 3", m.rolling, m.cp, m.ceil, m.hasCeil)
	}
	updates, err := a.listUpdates(ctx, room, oid)
	if err != nil {
		t.Fatalf("listUpdates: %v", err)
	}
	for c := range updates {
		if c > 3 {
			t.Errorf("update %d survived the finished rollback", c)
		}
	}
}

// TestLoad_AbandonsARollbackThatDiedBeforePublishing: with doc_v2 untouched the
// rollback took no effect, so it is undone rather than finished; finishing it
// would delete edits nobody had rolled back.
func TestLoad_AbandonsARollbackThatDiedBeforePublishing(t *testing.T) {
	t.Parallel()
	ctx := context.Background()
	a, _, _ := newFenceAdapters(t)
	const room = "fence-abandon"
	oid, rolled := seedFence(t, a, room)

	a.afterRollbackMark = func() bool { return true }
	if err := a.PruneAfter(ctx, room, persistence.Version(3), rolled); err != nil {
		t.Fatalf("PruneAfter: %v", err)
	}
	a.afterRollbackMark = nil

	if s, err := loadText(t, a, room); err != nil || s != "abcdef" {
		t.Fatalf("Load = %q, %v; want the untouched %q", s, err, "abcdef")
	}
	m, err := a.readMarks(ctx, room)
	if err != nil {
		t.Fatalf("readMarks: %v", err)
	}
	if m.rolling || m.hasCeil {
		t.Errorf("after abandoning: rolling=%v hasCeil=%v, want no ceiling at all", m.rolling, m.hasCeil)
	}
	updates, err := a.listUpdates(ctx, room, oid)
	if err != nil {
		t.Fatalf("listUpdates: %v", err)
	}
	if len(updates) != 6 {
		t.Errorf("%d updates left, want all 6", len(updates))
	}
}

// TestLoad_FailsTheOpenWhileAMarkedRollbackHoldsTheLock: an instance that
// cannot take the lock retries and then fails rather than serve the mix; once a
// holder settles the rollback it opens normally.
func TestLoad_FailsTheOpenWhileAMarkedRollbackHoldsTheLock(t *testing.T) {
	t.Parallel()
	ctx := context.Background()
	a, client, bucket := newFenceAdapters(t)
	const room = "fence-held"
	_, rolled := seedFence(t, a, room)

	a.afterRollbackDocV2 = func() bool { return true }
	if err := a.PruneAfter(ctx, room, persistence.Version(3), rolled); err != nil {
		t.Fatalf("PruneAfter: %v", err)
	}
	a.afterRollbackDocV2 = nil

	other, err := New(Options{Client: client, Bucket: bucket, Locker: heldLock{}})
	if err != nil {
		t.Fatalf("New: %v", err)
	}
	if s, err := loadText(t, other, room); !errors.Is(err, errStateMoved) {
		t.Errorf("Load without the lock = %q, %v; want errStateMoved", s, err)
	}
	if _, err := loadText(t, a, room); err != nil {
		t.Fatalf("Load by a lock holder: %v", err)
	}
	if s, err := loadText(t, other, room); err != nil || s != "abc" {
		t.Errorf("Load without the lock after settling = %q, %v; want %q", s, err, "abc")
	}
}

// TestAppendUpdate_SettlesAMarkedRollbackBeforeAppending: the append path's own
// recovery clears any ceiling. Clearing a mark without settling it would leave
// the old updates above the rolled-back doc_v2 and bring their edits back.
func TestAppendUpdate_SettlesAMarkedRollbackBeforeAppending(t *testing.T) {
	t.Parallel()
	ctx := context.Background()
	a, _, _ := newFenceAdapters(t)
	const room = "fence-append"
	_, rolled := seedFence(t, a, room)

	a.afterRollbackDocV2 = func() bool { return true }
	if err := a.PruneAfter(ctx, room, persistence.Version(3), rolled); err != nil {
		t.Fatalf("PruneAfter: %v", err)
	}
	a.afterRollbackDocV2 = nil

	d := crdt.New(crdt.WithClientID(77))
	if err := crdt.ApplyUpdateV1(d, rolled, nil); err != nil {
		t.Fatalf("apply rolled: %v", err)
	}
	sv := d.StateVector()
	txt := d.GetText("t")
	d.Transact(func(txn *crdt.Transaction) { txt.Insert(txn, txt.Len(), "Z", nil) })
	inc, err := crdt.DiffUpdateV1(crdt.EncodeStateAsUpdateV1(d, nil), sv)
	if err != nil {
		t.Fatalf("diff: %v", err)
	}
	v, err := a.AppendUpdate(ctx, room, inc)
	if err != nil {
		t.Fatalf("AppendUpdate: %v", err)
	}
	if v != 4 {
		t.Errorf("AppendUpdate took version %d, want 4, right after the target", v)
	}
	if s, err := loadText(t, a, room); err != nil || s != "abcZ" {
		t.Errorf("Load = %q, %v; want %q", s, err, "abcZ")
	}
}

// TestFlush_SettlesAStoppedRollbackInsteadOfFailing: the last-instance flush
// goes straight to writeUnionSince, past writeUnion's settle. Meeting a stopped
// rollback's mark, it would fail on every attempt; it now settles the rollback,
// and drops its candidate, which was built before the rollback and still holds
// the edits it removed.
func TestFlush_SettlesAStoppedRollbackInsteadOfFailing(t *testing.T) {
	t.Parallel()
	ctx := context.Background()
	a, _, _ := newFenceAdapters(t)
	const room = "fence-flush"
	_, rolled := seedFence(t, a, room)

	before, err := a.flushMarks(ctx, room)
	if err != nil {
		t.Fatalf("flushMarks: %v", err)
	}
	live := appendLettersState(t, 6) // the live room, "abcdef", as the flush takes it

	a.afterRollbackDocV2 = func() bool { return true }
	if err := a.PruneAfter(ctx, room, persistence.Version(3), rolled); err != nil {
		t.Fatalf("PruneAfter: %v", err)
	}
	a.afterRollbackDocV2 = nil

	if err := a.flushSnapshotSince(ctx, room, live, before); err != nil {
		t.Fatalf("flushSnapshotSince = %v, want the stopped rollback settled", err)
	}
	if s, err := loadText(t, a, room); err != nil || s != "abc" {
		t.Errorf("Load = %q, %v; want %q: the pre-rollback candidate was applied", s, err, "abc")
	}
}

// appendLettersState is the state appendLetters builds for n letters, without
// writing anything.
func appendLettersState(t *testing.T, n int) []byte {
	t.Helper()
	doc := crdt.New(crdt.WithClientID(31))
	txt := doc.GetText("t")
	for i := 0; i < n; i++ {
		doc.Transact(func(txn *crdt.Transaction) { txt.Insert(txn, txt.Len(), string(rune('a'+i)), nil) })
	}
	return crdt.EncodeStateAsUpdateV1(doc, nil)
}

// TestSettle_DoesNotFinishARollbackThatNeverPublished: a store snapshot can
// already cover the target exactly. A rollback that stopped right after marking
// must then be abandoned, not finished: finishing it would delete newer edits
// for a request its caller was told had failed.
func TestSettle_DoesNotFinishARollbackThatNeverPublished(t *testing.T) {
	t.Parallel()
	ctx := context.Background()
	a, _, _ := newFenceAdapters(t)
	const room = "settle-proof"
	appendLetters(t, a, room, 15, nil) // the tenth leaves doc_v2 covering exactly 10
	oid, err := a.oidFor(ctx, room)
	if err != nil {
		t.Fatalf("oidFor: %v", err)
	}
	rolled, err := a.MaterializeAt(ctx, room, persistence.Version(10))
	if err != nil {
		t.Fatalf("MaterializeAt(10): %v", err)
	}

	a.afterRollbackMark = func() bool { return true }
	if err := a.PruneAfter(ctx, room, persistence.Version(10), rolled); err != nil {
		t.Fatalf("PruneAfter: %v", err)
	}
	a.afterRollbackMark = nil

	if s, err := loadText(t, a, room); err != nil || s != "abcdefghijklmno" {
		t.Errorf("Load = %q, %v; want all fifteen edits kept", s, err)
	}
	updates, err := a.listUpdates(ctx, room, oid)
	if err != nil {
		t.Fatalf("listUpdates: %v", err)
	}
	for c := uint32(11); c <= 15; c++ {
		if _, ok := updates[c]; !ok {
			t.Errorf("update %d was deleted by a rollback that never published", c)
		}
	}
}

// TestPruneAfter_KeepsTheMarkUntilTheCheckpointIsDurable: with the checkpoint
// above the target, clearing the mark first and then failing to move the
// checkpoint let the next append skip settlement and take clock 11, not 4.
func TestPruneAfter_KeepsTheMarkUntilTheCheckpointIsDurable(t *testing.T) {
	t.Parallel()
	ctx := context.Background()
	a, _, _ := newFenceAdapters(t)
	const room = "fence-order"
	appendLetters(t, a, room, 15, nil) // the checkpoint sits at 10
	rolled, err := a.MaterializeAt(ctx, room, persistence.Version(3))
	if err != nil {
		t.Fatalf("MaterializeAt(3): %v", err)
	}

	a.afterRollbackCheckpoint = func() bool { return true }
	if err := a.PruneAfter(ctx, room, persistence.Version(3), rolled); err != nil {
		t.Fatalf("PruneAfter: %v", err)
	}
	a.afterRollbackCheckpoint = nil

	if m, err := a.readMarks(ctx, room); err != nil || !m.rolling {
		t.Fatalf("after the checkpoint write: rolling=%v err=%v, want the mark still set", m.rolling, err)
	}
	d := crdt.New(crdt.WithClientID(77))
	if err := crdt.ApplyUpdateV1(d, rolled, nil); err != nil {
		t.Fatalf("apply rolled: %v", err)
	}
	sv := d.StateVector()
	txt := d.GetText("t")
	d.Transact(func(txn *crdt.Transaction) { txt.Insert(txn, txt.Len(), "Z", nil) })
	inc, err := crdt.DiffUpdateV1(crdt.EncodeStateAsUpdateV1(d, nil), sv)
	if err != nil {
		t.Fatalf("diff: %v", err)
	}
	v, err := a.AppendUpdate(ctx, room, inc)
	if err != nil {
		t.Fatalf("AppendUpdate: %v", err)
	}
	if v != 4 {
		t.Errorf("AppendUpdate took version %d, want 4, right after the target", v)
	}
	if s, err := loadText(t, a, room); err != nil || s != "abcZ" {
		t.Errorf("Load = %q, %v; want %q", s, err, "abcZ")
	}
}

// TestPruneAfter_DoesNotOverwriteARollbackCommittedMeanwhile: a rollback whose
// lease lapsed after marking can resume once another rollback has settled its
// mark and committed. Its writes must then fail rather than replace that one.
func TestPruneAfter_DoesNotOverwriteARollbackCommittedMeanwhile(t *testing.T) {
	t.Parallel()
	ctx := context.Background()
	a, _, _ := newFenceAdapters(t)
	const room = "fence-lease"
	_, rolledTo3 := seedFence(t, a, room)
	rolledTo2, err := a.MaterializeAt(ctx, room, persistence.Version(2))
	if err != nil {
		t.Fatalf("MaterializeAt(2): %v", err)
	}

	fired := false
	a.afterRollbackMark = func() bool {
		if fired {
			return false
		}
		fired = true
		// Another holder, once this one's lease lapsed: it abandons this rollback's
		// mark and commits its own.
		if err := a.PruneAfter(ctx, room, persistence.Version(2), rolledTo2); err != nil {
			t.Errorf("the other rollback: %v", err)
		}
		return false
	}
	if err := a.PruneAfter(ctx, room, persistence.Version(3), rolledTo3); !errors.Is(err, errStateMoved) {
		t.Errorf("the stalled rollback = %v, want errStateMoved", err)
	}
	a.afterRollbackMark = nil

	if s, err := loadText(t, a, room); err != nil || s != "ab" {
		t.Errorf("Load = %q, %v; want the committed rollback's %q", s, err, "ab")
	}
	m, err := a.readMarks(ctx, room)
	if err != nil {
		t.Fatalf("readMarks: %v", err)
	}
	if m.rolling || m.cp != 2 || m.ceil != 2 {
		t.Errorf("marks: rolling=%v cp=%d ceil=%d, want the committed rollback's 2", m.rolling, m.cp, m.ceil)
	}
}

// TestMaterializeAt_SettlesAStoppedRollbackItCanLock: the recovery view may be
// the only thing touching a room, so a recovery read settles a stopped rollback
// itself instead of failing on its mark every time.
func TestMaterializeAt_SettlesAStoppedRollbackItCanLock(t *testing.T) {
	t.Parallel()
	ctx := context.Background()
	a, _, _ := newFenceAdapters(t)
	const room = "recover-settle"
	_, rolled := seedFence(t, a, room)

	a.afterRollbackMark = func() bool { return true }
	if err := a.PruneAfter(ctx, room, persistence.Version(3), rolled); err != nil {
		t.Fatalf("PruneAfter: %v", err)
	}
	a.afterRollbackMark = nil

	got, err := a.MaterializeAt(ctx, room, persistence.Version(2))
	if err != nil {
		t.Fatalf("MaterializeAt(2) = %v, want the stopped rollback settled", err)
	}
	if s := textOf(t, got); s != "ab" {
		t.Errorf("MaterializeAt(2) = %q, want %q", s, "ab")
	}
	if ok, err := a.RebuildableVersions(ctx, room, []persistence.Version{2, 4}); err != nil || !ok[2] || !ok[4] {
		t.Errorf("RebuildableVersions = %v, %v; want both listed", ok, err)
	}
}

// TestMaterializeAt_ReportsARollbackStillHoldingTheLock: while the rollback's
// writer still holds the lock the read cannot settle it, so it reports a
// retryable state rather than serving the mix or a plain failure.
func TestMaterializeAt_ReportsARollbackStillHoldingTheLock(t *testing.T) {
	t.Parallel()
	ctx := context.Background()
	a, client, bucket := newFenceAdapters(t)
	const room = "recover-held"
	_, rolled := seedFence(t, a, room)

	a.afterRollbackMark = func() bool { return true }
	if err := a.PruneAfter(ctx, room, persistence.Version(3), rolled); err != nil {
		t.Fatalf("PruneAfter: %v", err)
	}
	a.afterRollbackMark = nil

	other, err := New(Options{Client: client, Bucket: bucket, Locker: heldLock{}})
	if err != nil {
		t.Fatalf("New: %v", err)
	}
	_, err = other.MaterializeAt(ctx, room, persistence.Version(2))
	var ri interface{ RollbackInProgress() bool }
	if !errors.As(err, &ri) || !ri.RollbackInProgress() {
		t.Errorf("MaterializeAt with the lock held = %v, want a rollback-in-progress error", err)
	}
}
