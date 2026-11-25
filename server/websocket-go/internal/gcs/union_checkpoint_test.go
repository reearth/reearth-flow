package gcs

import (
	"context"
	"sync"
	"testing"

	"github.com/alicebob/miniredis/v2"
	goredis "github.com/redis/go-redis/v9"
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

// TestCompact_KeptVersionsStillRebuildTheirOwnState: the base Compact writes
// must not run ahead of the checkpoint it sets. MaterializeAt starts from the
// base for any version at or above the checkpoint and replays the update objects
// on top, so a base holding the kept objects makes every one of those versions
// rebuild as the latest state instead of its own.
func TestCompact_KeptVersionsStillRebuildTheirOwnState(t *testing.T) {
	t.Parallel()
	ctx := context.Background()
	a := newParkedTestAdapter(t)
	appendAll(t, a, fillers(t, 27)...) // clocks 1..27 set f.k to 0..26; clock 20 checkpoints

	if _, err := a.Compact(ctx, "room", 10); err != nil {
		t.Fatal(err)
	}
	cp, err := a.checkpoint(ctx, "room")
	if err != nil {
		t.Fatal(err)
	}
	vs, err := a.ListVersions(ctx, "room")
	if err != nil {
		t.Fatal(err)
	}

	checked := 0
	for _, v := range vs {
		if uint32(v.Version) < cp {
			// Below the checkpoint MaterializeAt leaves the base out by design and
			// replays a log that no longer reaches clock 1. Out of scope here.
			continue
		}
		state, err := a.MaterializeAt(ctx, "room", v.Version)
		if err != nil {
			t.Fatalf("MaterializeAt(%d): %v", v.Version, err)
		}
		if got, want := fillerValue(t, state), float64(uint32(v.Version)-1); got != want {
			t.Errorf("MaterializeAt(%d) f.k = %v, want %v; the kept objects leaked into the base", v.Version, got, want)
		}
		checked++
	}
	if checked < 7 {
		t.Fatalf("checked only %d versions at or above checkpoint %d, want at least 7", checked, cp)
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

// TestLoad_ARollbackDuringTheReadIsNotServed: a rollback lands while Load reads,
// and the appends after it bring the checkpoint back to the value Load first read.
// Load must not serve the doc_v2 it read before the rollback.
func TestLoad_ARollbackDuringTheReadIsNotServed(t *testing.T) {
	t.Parallel()
	ctx := context.Background()
	a := newParkedTestAdapter(t)
	fs := fillers(t, 10)
	appendAll(t, a, fs...) // clock 10 checkpoints
	rolled := crdt.New()
	for _, f := range fs[:5] {
		if err := crdt.ApplyUpdateV1(rolled, f, nil); err != nil {
			t.Fatal(err)
		}
	}
	later := crdt.New(crdt.WithClientID(4))
	g := later.GetMap("g")
	var laterUps [][]byte
	for i := 0; i < 5; i++ {
		sv := later.StateVector()
		later.Transact(func(txn *crdt.Transaction) { g.Set(txn, "k", "after") })
		laterUps = append(laterUps, crdt.EncodeStateAsUpdateV1(later, sv))
	}
	fired := false
	a.afterReadBase = func() {
		if fired {
			return
		}
		fired = true
		if err := a.PruneAfter(ctx, "room", 5, crdt.EncodeStateAsUpdateV1(rolled, nil)); err != nil {
			t.Error(err)
		}
		appendAll(t, a, laterUps...) // clocks 6..10; clock 10 checkpoints again
	}
	state := mustLoad(t, a)
	a.afterReadBase = nil
	if v := fillerValue(t, state); v != 4 {
		t.Fatalf("f.k = %v, want 4; Load served the state from before the rollback", v)
	}
	d, err := docFromV1(state)
	if err != nil {
		t.Fatal(err)
	}
	if v, _ := d.GetMap("g").Get("k"); v != "after" {
		t.Fatalf("g.k = %v, want \"after\"; the edits after the rollback are missing", v)
	}
}

// TestLoad_ARollbackBeforeTheObjectReadsIsNotServed: a rollback and appends
// land after Load checked the marks but before it read the update objects, so
// the objects it listed now hold the edits made after the rollback.
func TestLoad_ARollbackBeforeTheObjectReadsIsNotServed(t *testing.T) {
	t.Parallel()
	ctx := context.Background()
	a := newParkedTestAdapter(t)
	fs := fillers(t, 12)
	appendAll(t, a, fs...) // clock 10 checkpoints; 11 and 12 stay above it
	rolled := crdt.New()
	for _, f := range fs[:5] {
		if err := crdt.ApplyUpdateV1(rolled, f, nil); err != nil {
			t.Fatal(err)
		}
	}
	later := crdt.New(crdt.WithClientID(4))
	g := later.GetMap("g")
	var laterUps [][]byte
	for i := 0; i < 7; i++ {
		sv := later.StateVector()
		later.Transact(func(txn *crdt.Transaction) { g.Set(txn, "k", "after") })
		laterUps = append(laterUps, crdt.EncodeStateAsUpdateV1(later, sv))
	}
	fired := false
	a.afterCheckMarks = func() {
		if fired {
			return
		}
		fired = true
		if err := a.PruneAfter(ctx, "room", 5, crdt.EncodeStateAsUpdateV1(rolled, nil)); err != nil {
			t.Error(err)
		}
		appendAll(t, a, laterUps...) // clocks 6..12 replace the listed 11 and 12
	}
	state := mustLoad(t, a)
	a.afterCheckMarks = nil
	if !fired {
		t.Fatal("the hook did not run")
	}
	if v := fillerValue(t, state); v != 4 {
		t.Fatalf("f.k = %v, want 4; Load mixed the base from before the rollback with updates from after it", v)
	}
	d, err := docFromV1(state)
	if err != nil {
		t.Fatal(err)
	}
	if v, _ := d.GetMap("g").Get("k"); v != "after" {
		t.Fatalf("g.k = %v, want \"after\"; the edits after the rollback are missing", v)
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

// twoInstances returns two adapters on one bucket, each with its own Redis
// locker owner, as two websocket instances have.
func twoInstances(t *testing.T) (*Adapter, *Adapter) {
	t.Helper()
	client, bucket := newFakeGCS(t)
	mr := miniredis.RunT(t)
	rc := goredis.NewClient(&goredis.Options{Addr: mr.Addr()})
	t.Cleanup(func() { _ = rc.Close() })
	var out [2]*Adapter
	for i, owner := range []string{"instance-1", "instance-2"} {
		a, err := New(Options{Client: client, Bucket: bucket, Locker: NewRedisLocker(rc, owner)})
		if err != nil {
			t.Fatal(err)
		}
		out[i] = a
	}
	return out[0], out[1]
}

// TestCompact_TwoInstancesLeaveTheLogConsistent: every instance that held a room
// compacts it when it unloads its copy, in turn or at the same time. The log,
// checkpoint and doc_v2 must agree afterwards, and later appends must load.
func TestCompact_TwoInstancesLeaveTheLogConsistent(t *testing.T) {
	t.Parallel()
	for _, tc := range []struct {
		name     string
		together bool
	}{{"in turn", false}, {"at once", true}} {
		t.Run(tc.name, func(t *testing.T) {
			t.Parallel()
			ctx := context.Background()
			a1, a2 := twoInstances(t)
			first, second := editPair(t)
			appendAll(t, a1, first)
			appendAll(t, a2, second)
			for i, f := range fillers(t, 25) { // clocks 3..27; checkpoints at 10 and 20
				appendAll(t, []*Adapter{a1, a2}[i%2], f)
			}

			var n [2]int
			var errs [2]error
			if tc.together {
				var wg sync.WaitGroup
				for i, a := range []*Adapter{a1, a2} {
					wg.Add(1)
					go func() {
						defer wg.Done()
						n[i], errs[i] = a.Compact(ctx, "room", 5)
					}()
				}
				wg.Wait()
			} else {
				n[0], errs[0] = a1.Compact(ctx, "room", 5)
				n[1], errs[1] = a2.Compact(ctx, "room", 5)
			}
			if errs[0] != nil || errs[1] != nil || n[0]+n[1] != 22 {
				t.Fatalf("Compact = %v %v, %v; want 22 deleted between them", n[0], n[1], errs)
			}

			if cp, err := a1.checkpoint(ctx, "room"); err != nil || cp != 22 {
				t.Fatalf("checkpoint = %d, %v; want 22", cp, err)
			}
			for _, a := range []*Adapter{a1, a2} {
				vs, err := a.ListVersions(ctx, "room")
				if err != nil || len(vs) != 5 || vs[0].Version != 27 || vs[4].Version != 23 {
					t.Fatalf("versions = %v, %v; want clocks 23..27", vs, err)
				}
			}
			base, _, err := a1.loadPrimaryV2(ctx, "room")
			if err != nil {
				t.Fatal(err)
			}
			if got, parked := loaded(t, base, "t"); got != "ab" || parked != 0 {
				t.Fatalf("doc_v2: %q with %d parked, want \"ab\" and none", got, parked)
			}
			// doc_v2 must hold every deleted update and stop there: clock 22 is the
			// highest one folded in and it set f.k to 19. A later value means the
			// kept objects (23..27) leaked into the base, which would make every
			// one of them rebuild as the latest state.
			if v := fillerValue(t, base); v != 19 {
				t.Fatalf("doc_v2 f.k = %v, want exactly 19 (clock 22, the highest folded in)", v)
			}

			extra := fillers(t, 26)[25]
			appendAll(t, a2, extra)
			for _, a := range []*Adapter{a1, a2} {
				state := mustLoad(t, a)
				if got, parked := loaded(t, state, "t"); got != "ab" || parked != 0 {
					t.Fatalf("Load: %q with %d parked, want \"ab\" and none", got, parked)
				}
				if v := fillerValue(t, state); v != 25 {
					t.Fatalf("Load f.k = %v, want 25", v)
				}
			}
		})
	}
}
