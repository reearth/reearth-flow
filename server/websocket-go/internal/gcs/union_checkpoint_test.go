package gcs

import (
	"context"
	"testing"

	"github.com/reearth/ygo/crdt"
)

// fillers returns n independent incremental updates from client 3 on map "f".
func fillers(t *testing.T, n int) [][]byte {
	t.Helper()
	d := crdt.New(crdt.WithClientID(3))
	m := d.GetMap("f")
	var out [][]byte
	for i := 0; i < n; i++ {
		sv := d.StateVector()
		d.Transact(func(txn *crdt.Transaction) { m.Set(txn, "k", float64(i)) })
		out = append(out, crdt.EncodeStateAsUpdateV1(d, sv))
	}
	return out
}

func appendAll(t *testing.T, a *Adapter, ups ...[]byte) {
	t.Helper()
	for _, u := range ups {
		if _, err := a.AppendUpdate(context.Background(), "room", u); err != nil {
			t.Fatal(err)
		}
	}
}

// fillerValue is the value of f.k in a V1 state, or -1 when absent.
func fillerValue(t *testing.T, state []byte) float64 {
	t.Helper()
	d, err := docFromV1(state)
	if err != nil {
		t.Fatal(err)
	}
	v, ok := d.GetMap("f").Get("k")
	if !ok {
		return -1
	}
	switch n := v.(type) {
	case float32:
		return float64(n)
	case float64:
		return n
	}
	t.Fatalf("f.k is %T", v)
	return 0
}

func mustLoad(t *testing.T, a *Adapter) []byte {
	t.Helper()
	lr, err := a.Load(context.Background(), "room")
	if err != nil {
		t.Fatalf("Load: %v", err)
	}
	return lr.Update
}

// TestFlushSnapshot_StaleCopyCannotRemoveAnEdit: a copy that missed an edit,
// with nothing depending on it yet, checkpoints after the edit's object was
// checkpointed. The edit must stay, and a later edit building on it must load.
func TestFlushSnapshot_StaleCopyCannotRemoveAnEdit(t *testing.T) {
	t.Parallel()
	ctx := context.Background()
	a := newParkedTestAdapter(t)
	first, second := editPair(t)
	fs := fillers(t, 9)
	appendAll(t, a, first)
	appendAll(t, a, fs...) // clock 10 checkpoints

	behind := crdt.New()
	for _, f := range fs {
		if err := crdt.ApplyUpdateV1(behind, f, nil); err != nil {
			t.Fatal(err)
		}
	}
	if err := a.FlushSnapshot(ctx, "room", crdt.EncodeStateAsUpdateV1(behind, nil)); err != nil {
		t.Fatal(err)
	}
	if got, _ := loaded(t, mustLoad(t, a), "t"); got != "a" {
		t.Fatalf("after the stale checkpoint: text %q, want \"a\"", got)
	}

	appendAll(t, a, second)
	if got, parked := loaded(t, mustLoad(t, a), "t"); got != "ab" || parked != 0 {
		t.Fatalf("after the dependent edit: %q with %d parked, want \"ab\" and none", got, parked)
	}
}

// TestFlushSnapshot_ParkedCopyIsCompletedFromStorage: a checkpoint from a copy
// holding parked items includes the edit they wait for when storage has it.
func TestFlushSnapshot_ParkedCopyIsCompletedFromStorage(t *testing.T) {
	t.Parallel()
	ctx := context.Background()
	a := newParkedTestAdapter(t)
	first, second := editPair(t)
	appendAll(t, a, first)
	if err := a.FlushSnapshot(ctx, "room", parkedState(t, second)); err != nil {
		t.Fatal(err)
	}
	v1, _, err := a.loadPrimaryV2(ctx, "room")
	if err != nil {
		t.Fatal(err)
	}
	if got, parked := loaded(t, v1, "t"); got != "ab" || parked != 0 {
		t.Fatalf("doc_v2: %q with %d parked, want \"ab\" and none", got, parked)
	}
}

// TestCompact_FoldsCheckpointedObjectsItDeletes: a doc_v2 an earlier build wrote
// from a copy that was merely behind lacks a checkpointed edit. Compact must
// fold that edit's object in before deleting it.
func TestCompact_FoldsCheckpointedObjectsItDeletes(t *testing.T) {
	t.Parallel()
	ctx := context.Background()
	a := newParkedTestAdapter(t)
	first, _ := editPair(t)
	fs := fillers(t, 9)
	appendAll(t, a, first)
	appendAll(t, a, fs...)
	behind := crdt.New()
	for _, f := range fs {
		if err := crdt.ApplyUpdateV1(behind, f, nil); err != nil {
			t.Fatal(err)
		}
	}
	writeOldCheckpoint(t, a, "room", crdt.EncodeStateAsUpdateV1(behind, nil))

	if n, err := a.Compact(ctx, "room", 1); err != nil || n != 9 {
		t.Fatalf("Compact = %d, %v; want 9 deleted", n, err)
	}
	if got, _ := loaded(t, mustLoad(t, a), "t"); got != "a" {
		t.Fatalf("after Compact: text %q, want \"a\"", got)
	}
}

// TestCompact_KeepZeroKeepsTheLog: ygo compacts every room it unloads with
// KeepVersions, whose zero means "keep everything".
func TestCompact_KeepZeroKeepsTheLog(t *testing.T) {
	t.Parallel()
	ctx := context.Background()
	a := newParkedTestAdapter(t)
	appendAll(t, a, fillers(t, 5)...)
	if n, err := a.Compact(ctx, "room", 0); err != nil || n != 0 {
		t.Fatalf("Compact(keep=0) = %d, %v; want 0 deleted", n, err)
	}
	vs, err := a.ListVersions(ctx, "room")
	if err != nil || len(vs) != 5 {
		t.Fatalf("versions after Compact(keep=0) = %d, %v; want 5", len(vs), err)
	}
}

// loadRace seeds doc_v2 holding "X" and fillers at clocks 1..9, then runs op once
// between Load's doc_v2 read and its listing of the update log.
func loadRace(t *testing.T, op func(a *Adapter, fs [][]byte)) (*Adapter, [][]byte) {
	t.Helper()
	ctx := context.Background()
	a := newParkedTestAdapter(t)
	fs := fillers(t, 10)
	base := crdt.New(crdt.WithClientID(9))
	txt := base.GetText("t")
	base.Transact(func(txn *crdt.Transaction) { txt.Insert(txn, 0, "X", nil) })
	if err := a.FlushSnapshot(ctx, "room", crdt.EncodeStateAsUpdateV1(base, nil)); err != nil {
		t.Fatal(err)
	}
	appendAll(t, a, fs[:9]...)
	fired := false
	a.afterReadBase = func() {
		if !fired {
			fired = true
			op(a, fs)
		}
	}
	return a, fs
}

// TestLoad_ACheckpointDuringTheReadIsNotLost: another instance's append lands on
// a 10th clock while Load reads, checkpointing everything Load's doc_v2 lacks.
func TestLoad_ACheckpointDuringTheReadIsNotLost(t *testing.T) {
	t.Parallel()
	a, fs := loadRace(t, func(a *Adapter, fs [][]byte) { appendAll(t, a, fs[9]) })
	state := mustLoad(t, a)
	a.afterReadBase = nil
	if got, _ := loaded(t, state, "t"); got != "X" {
		t.Fatalf("text %q, want \"X\"", got)
	}
	if v := fillerValue(t, state); v != 9 {
		t.Fatalf("f.k = %v, want 9; the updates checkpointed during the read are missing", v)
	}
	_ = fs
}

// TestLoad_ACompactionDuringTheReadIsNotLost: a compaction folds and deletes
// update objects while Load reads.
func TestLoad_ACompactionDuringTheReadIsNotLost(t *testing.T) {
	t.Parallel()
	a, _ := loadRace(t, func(a *Adapter, _ [][]byte) {
		if _, err := a.Compact(context.Background(), "room", 1); err != nil {
			t.Error(err)
		}
	})
	state := mustLoad(t, a)
	a.afterReadBase = nil
	if got, _ := loaded(t, state, "t"); got != "X" {
		t.Fatalf("text %q, want \"X\"", got)
	}
	if v := fillerValue(t, state); v != 8 {
		t.Fatalf("f.k = %v, want 8; updates compacted during the read are missing", v)
	}
}

// TestSnapshotFromStore_NeverWritesAStateACheckpointOvertook: a writer that
// ignores the doc lock checkpoints while the rebuild reads. The rebuild may fail,
// but must not write the doc_v2 it read before that checkpoint.
func TestSnapshotFromStore_NeverWritesAStateACheckpointOvertook(t *testing.T) {
	t.Parallel()
	ctx := context.Background()
	a, _ := loadRace(t, func(a *Adapter, fs [][]byte) { appendAll(t, a, fs[9]) })
	err := a.SnapshotFromStore(ctx, "room")
	t.Logf("SnapshotFromStore: %v", err)
	a.afterReadBase = nil
	if v := fillerValue(t, mustLoad(t, a)); v != 9 {
		t.Fatalf("f.k = %v, want 9", v)
	}
}
