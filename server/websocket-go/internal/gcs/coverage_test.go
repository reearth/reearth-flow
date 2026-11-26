package gcs

import (
	"context"
	"errors"
	"testing"

	"github.com/reearth/ygo/crdt"
	"github.com/reearth/ygo/persistence"
)

// appendLetters appends one letter per clock, "a" at clock 1, calling after (if
// set) with the clock and full state after each append. It returns the final state.
func appendLetters(t *testing.T, a *Adapter, room string, n int, after func(clock int, full []byte)) []byte {
	t.Helper()
	ctx := context.Background()
	doc := crdt.New(crdt.WithClientID(31))
	txt := doc.GetText("t")
	var prev []byte
	for i := 0; i < n; i++ {
		doc.Transact(func(txn *crdt.Transaction) { txt.Insert(txn, txt.Len(), string(rune('a'+i)), nil) })
		full := crdt.EncodeStateAsUpdateV1(doc, nil)
		inc := full
		if prev != nil {
			d, err := crdt.DiffUpdateV1(full, svOf(t, prev))
			if err != nil {
				t.Fatalf("DiffUpdateV1: %v", err)
			}
			inc = d
		}
		if _, err := a.AppendUpdate(ctx, room, inc); err != nil {
			t.Fatalf("AppendUpdate: %v", err)
		}
		prev = full
		if after != nil {
			after(i+1, full)
		}
	}
	return prev
}

func textOf(t *testing.T, v1 []byte) string {
	t.Helper()
	d := crdt.New()
	if err := crdt.ApplyUpdateV1(d, v1, nil); err != nil {
		t.Fatalf("ApplyUpdateV1: %v", err)
	}
	return d.GetText("t").ToString()
}

func newCoverageAdapter(t *testing.T) *Adapter {
	t.Helper()
	client, bucket := newFakeGCS(t)
	a, err := New(Options{Client: client, Bucket: bucket, Locker: NewNoLock()})
	if err != nil {
		t.Fatalf("New: %v", err)
	}
	return a
}

// TestMaterializeAt_LiveBaseServesTheHeadOnly: a live base is the room's current
// state. Below the head it can run past the version and is refused, but the head
// keeps what only the base holds: rebuilding it from the log alone would drop a
// real edit, and a rollback to it would then delete that edit.
func TestMaterializeAt_LiveBaseServesTheHeadOnly(t *testing.T) {
	t.Parallel()
	ctx := context.Background()
	a := newCoverageAdapter(t)
	const room = "live-head"
	logged := appendLetters(t, a, room, 3, nil)

	live := crdt.New(crdt.WithClientID(88))
	if err := crdt.ApplyUpdateV1(live, logged, nil); err != nil {
		t.Fatalf("apply: %v", err)
	}
	lt := live.GetText("t")
	live.Transact(func(txn *crdt.Transaction) { lt.Insert(txn, lt.Len(), "X", nil) })
	if err := a.FlushSnapshot(ctx, room, crdt.EncodeStateAsUpdateV1(live, nil)); err != nil {
		t.Fatalf("FlushSnapshot: %v", err)
	}

	got, err := a.MaterializeAt(ctx, room, persistence.Version(3))
	if err != nil {
		t.Fatalf("MaterializeAt(3), the head: %v", err)
	}
	if s := textOf(t, got); s != "abcX" {
		t.Errorf("MaterializeAt(3) = %q, want %q: the edit only the base holds was dropped", s, "abcX")
	}
	got, err = a.MaterializeAt(ctx, room, persistence.Version(1))
	if err != nil {
		t.Fatalf("MaterializeAt(1): %v", err)
	}
	if s := textOf(t, got); s != "a" {
		t.Errorf("MaterializeAt(1) = %q, want %q", s, "a")
	}
}

// TestMaterializeAt_RefusesBelowTheHeadOfALiveBase: once the head moves past a
// live flush, a version at the flush point could hold edits that raced it.
func TestMaterializeAt_RefusesBelowTheHeadOfALiveBase(t *testing.T) {
	t.Parallel()
	ctx := context.Background()
	a := newCoverageAdapter(t)
	const room = "live-below"
	appendLetters(t, a, room, 4, func(clock int, full []byte) {
		if clock == 3 {
			if err := a.FlushSnapshot(ctx, room, full); err != nil {
				t.Fatalf("FlushSnapshot: %v", err)
			}
		}
	})

	if got, err := a.MaterializeAt(ctx, room, persistence.Version(3)); !errors.Is(err, ErrVersionNotRebuildable) {
		t.Errorf("MaterializeAt(3) = %q, %v; want ErrVersionNotRebuildable", textOf(t, got), err)
	}
}

// TestMaterializeAt_RebuildsAVersionAStoreSnapshotOvertook: a snapshot built
// from the store alone carries nothing the log lacks, so once its coverage is
// known a version below it is rebuilt exactly from the log instead of from a
// base that already holds later edits.
func TestMaterializeAt_RebuildsAVersionAStoreSnapshotOvertook(t *testing.T) {
	t.Parallel()
	ctx := context.Background()
	a := newCoverageAdapter(t)
	const room = "store-snapshot"
	appendLetters(t, a, room, 3, nil)
	if err := a.SnapshotFromStore(ctx, room); err != nil {
		t.Fatalf("SnapshotFromStore: %v", err)
	}

	got, err := a.MaterializeAt(ctx, room, persistence.Version(1))
	if err != nil {
		t.Fatalf("MaterializeAt(1): %v", err)
	}
	if s := textOf(t, got); s != "a" {
		t.Errorf("MaterializeAt(1) = %q, want %q: the base already held later edits", s, "a")
	}
}

// TestMaterializeAt_StillRebuildsWhileCoverageIsBelowTheTarget: the other side.
// A base that covers less than the target is still used, with the log replayed
// on top, so the check refuses only what it has to.
func TestMaterializeAt_StillRebuildsWhileCoverageIsBelowTheTarget(t *testing.T) {
	t.Parallel()
	ctx := context.Background()
	a := newCoverageAdapter(t)
	const room = "below-target"
	appendLetters(t, a, room, 15, nil) // the tenth trips the inline compaction

	for v, want := range map[int]string{10: "abcdefghij", 12: "abcdefghijkl", 15: "abcdefghijklmno"} {
		got, err := a.MaterializeAt(ctx, room, persistence.Version(v))
		if err != nil {
			t.Errorf("MaterializeAt(%d): %v", v, err)
			continue
		}
		if s := textOf(t, got); s != want {
			t.Errorf("MaterializeAt(%d) = %q, want %q", v, s, want)
		}
	}
}

// TestMaterializeAt_TreatsAnUnstampedDocV2AsTheCheckpoint: a doc_v2 written
// before coverage was recorded is read as covering the checkpoint, exactly as
// every rebuild treated it before, so nothing changes for it at deploy.
func TestMaterializeAt_TreatsAnUnstampedDocV2AsTheCheckpoint(t *testing.T) {
	t.Parallel()
	ctx := context.Background()
	a := newCoverageAdapter(t)
	const room = "unstamped"
	appendLetters(t, a, room, 15, nil)
	// A metadata update only removes a key when it is set to "".
	if err := a.store.updateMeta(ctx, a.layout.DocV2Name(room), func(m map[string]string) {
		m[metaCovers] = ""
		m[metaLive] = ""
	}); err != nil {
		t.Fatalf("strip coverage: %v", err)
	}
	if mk, err := a.readMarks(ctx, room); err != nil || mk.hasCov {
		t.Fatalf("setup: coverage still recorded (hasCov=%v, err=%v)", mk.hasCov, err)
	}

	for v, want := range map[int]string{5: "abcde", 12: "abcdefghijkl"} {
		got, err := a.MaterializeAt(ctx, room, persistence.Version(v))
		if err != nil {
			t.Errorf("MaterializeAt(%d): %v", v, err)
			continue
		}
		if s := textOf(t, got); s != want {
			t.Errorf("MaterializeAt(%d) = %q, want %q", v, s, want)
		}
	}
}

// TestCoverage_AStoreCheckpointPastTheFlushClearsLive: the live flag would
// otherwise stick forever, since every later union keeps the live content, and
// leave every version below the head unrestorable after any last-instance flush.
func TestCoverage_AStoreCheckpointPastTheFlushClearsLive(t *testing.T) {
	t.Parallel()
	ctx := context.Background()
	a := newCoverageAdapter(t)
	const room = "live-clears"
	appendLetters(t, a, room, 12, func(clock int, full []byte) {
		if clock == 3 {
			if err := a.FlushSnapshot(ctx, room, full); err != nil {
				t.Fatalf("FlushSnapshot: %v", err)
			}
		}
	})
	mk, err := a.readMarks(ctx, room)
	if err != nil {
		t.Fatalf("readMarks: %v", err)
	}
	if !mk.hasCov || mk.cov.live || mk.cov.clock < 10 {
		t.Fatalf("after the inline compaction at 10: coverage = %+v (recorded %v), want clock >= 10 and not live", mk.cov, mk.hasCov)
	}
	got, err := a.MaterializeAt(ctx, room, persistence.Version(11))
	if err != nil {
		t.Fatalf("MaterializeAt(11): %v", err)
	}
	if s := textOf(t, got); s != "abcdefghijk" {
		t.Errorf("MaterializeAt(11) = %q, want %q", s, "abcdefghijk")
	}
}

// TestRebuildableVersions_AgreesWithMaterializeAt: the recovery list and the
// rebuild must not disagree, so every listed version is judged both ways. The
// room carries both kinds of refusal: entries the trimmed log no longer reaches,
// and entries a live base overtakes.
func TestRebuildableVersions_AgreesWithMaterializeAt(t *testing.T) {
	t.Parallel()
	ctx := context.Background()
	a := newCoverageAdapter(t)
	const room = "agree"
	appendLetters(t, a, room, 15, func(clock int, full []byte) {
		if clock == 12 {
			if err := a.FlushSnapshot(ctx, room, full); err != nil {
				t.Fatalf("FlushSnapshot: %v", err)
			}
		}
	})
	if _, err := a.Compact(ctx, room, 5); err != nil {
		t.Fatalf("Compact: %v", err)
	}

	metas, err := a.ListVersions(ctx, room)
	if err != nil {
		t.Fatalf("ListVersions: %v", err)
	}
	vs := make([]persistence.Version, len(metas))
	for i, m := range metas {
		vs[i] = m.Version
	}
	ok, err := a.RebuildableVersions(ctx, room, vs)
	if err != nil {
		t.Fatalf("RebuildableVersions: %v", err)
	}

	var listed, refused int
	for _, v := range vs {
		_, merr := a.MaterializeAt(ctx, room, v)
		if merr != nil && !errors.Is(merr, ErrVersionNotRebuildable) {
			t.Fatalf("MaterializeAt(%d): unexpected %v", v, merr)
		}
		if ok[v] != (merr == nil) {
			t.Errorf("version %d: listed=%v but MaterializeAt err=%v", v, ok[v], merr)
		}
		if ok[v] {
			listed++
		} else {
			refused++
		}
	}
	if listed == 0 || refused == 0 {
		t.Fatalf("setup: %d listed and %d refused; want both kinds present", listed, refused)
	}
}
