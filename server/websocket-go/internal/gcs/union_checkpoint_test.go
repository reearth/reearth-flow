package gcs

import (
	"bytes"
	"context"
	"errors"
	"sync"
	"sync/atomic"
	"testing"
	"time"

	"github.com/alicebob/miniredis/v2"
	goredis "github.com/redis/go-redis/v9"
	"github.com/reearth/ygo/crdt"
	"github.com/reearth/ygo/persistence"
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

// TestMaterializeAt_RefusesAVersionTheLogCannotRebuild: once an unload
// compaction trims the log, a version below the checkpoint has neither the base
// (MaterializeAt leaves it out, it is past the version) nor the updates it needs
// (they were folded and deleted). The rebuild would be a fragment, and rollback
// writes it over doc_v2 and deletes the log above the version, so it must be
// refused rather than returned.
func TestMaterializeAt_RefusesAVersionTheLogCannotRebuild(t *testing.T) {
	t.Parallel()
	ctx := context.Background()
	a := newParkedTestAdapter(t)
	appendAll(t, a, fillers(t, 30)...) // clocks 1..30; clock 30 checkpoints

	if _, err := a.Compact(ctx, "room", 10); err != nil { // deletes 1..20, keeps 21..30
		t.Fatal(err)
	}
	cp, err := a.checkpoint(ctx, "room")
	if err != nil {
		t.Fatal(err)
	}
	if cp != 30 {
		t.Fatalf("checkpoint = %d, want 30; the setup needs the kept versions below it", cp)
	}

	// 21..29 sit below the checkpoint with updates 1..20 gone.
	for _, v := range []uint32{21, 25, 29} {
		state, err := a.MaterializeAt(ctx, "room", persistence.Version(v))
		if !errors.Is(err, ErrVersionNotRebuildable) {
			t.Errorf("MaterializeAt(%d) = %d bytes, %v; want ErrVersionNotRebuildable", v, len(state), err)
		}
		if state != nil {
			t.Errorf("MaterializeAt(%d) returned %d bytes alongside the refusal", v, len(state))
		}
	}

	// The checkpoint itself still rebuilds, from the base.
	state, err := a.MaterializeAt(ctx, "room", persistence.Version(cp))
	if err != nil {
		t.Fatalf("MaterializeAt(%d): %v", cp, err)
	}
	if got := fillerValue(t, state); got != 29 {
		t.Fatalf("MaterializeAt(%d) f.k = %v, want 29", cp, got)
	}
}

// TestMaterializeAt_StillRebuildsWhileTheLogIsWhole: the refusal must be narrow.
// A project whose log still reaches back to its first update rebuilds every
// version, below the checkpoint included.
func TestMaterializeAt_StillRebuildsWhileTheLogIsWhole(t *testing.T) {
	t.Parallel()
	ctx := context.Background()
	a := newParkedTestAdapter(t)
	appendAll(t, a, fillers(t, 30)...) // clocks 1..30, nothing compacted away

	cp, err := a.checkpoint(ctx, "room")
	if err != nil {
		t.Fatal(err)
	}
	if cp != 30 {
		t.Fatalf("checkpoint = %d, want 30", cp)
	}
	for _, v := range []uint32{1, 7, 22, 29, 30} {
		state, err := a.MaterializeAt(ctx, "room", persistence.Version(v))
		if err != nil {
			t.Fatalf("MaterializeAt(%d): %v", v, err)
		}
		if got, want := fillerValue(t, state), float64(v-1); got != want {
			t.Errorf("MaterializeAt(%d) f.k = %v, want %v", v, got, want)
		}
	}
}

// TestFlushSnapshot_CopyParkedOnACheckpointedEditIsCompleted: doc_v2, written by
// an earlier build, lacks a checkpointed edit and has nothing parked. A copy
// waiting on that edit checkpoints, and doc_v2 must come out complete.
func TestFlushSnapshot_CopyParkedOnACheckpointedEditIsCompleted(t *testing.T) {
	t.Parallel()
	ctx := context.Background()
	a := newParkedTestAdapter(t)
	first, second := editPair(t)
	fs := fillers(t, 9)
	appendAll(t, a, first)
	appendAll(t, a, fs...) // clock 10 checkpoints first's object
	behind := crdt.New()
	for _, f := range fs {
		if err := crdt.ApplyUpdateV1(behind, f, nil); err != nil {
			t.Fatal(err)
		}
	}
	writeOldCheckpoint(t, a, "room", crdt.EncodeStateAsUpdateV1(behind, nil))

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

// textEdit is an update from its own client that sets the text name to s.
func textEdit(client crdt.ClientID, name, s string) []byte {
	d := crdt.New(crdt.WithClientID(client))
	txt := d.GetText(name)
	d.Transact(func(txn *crdt.Transaction) { txt.Insert(txn, 0, s, nil) })
	return crdt.EncodeStateAsUpdateV1(d, nil)
}

// checkpointDuringWrite runs a checkpoint of the text "x" once, just before a's
// next conditional doc_v2 write, as another instance would after a's doc lock
// expired. The test adapter does not lock, so the checkpoint runs at once.
func checkpointDuringWrite(t *testing.T, a *Adapter) {
	t.Helper()
	fired := false
	a.beforeV2Write = func() {
		if fired {
			return
		}
		fired = true
		if err := a.FlushSnapshot(context.Background(), "room", textEdit(8, "x", "x")); err != nil {
			t.Error(err)
		}
	}
	t.Cleanup(func() {
		if !fired {
			t.Error("the checkpoint did not run")
		}
	})
}

// TestFlushSnapshot_AnExpiredLockCannotReplaceANewerCheckpoint: a checkpoint
// whose lock expired after it read the stored state must not write over the
// checkpoint another instance wrote meanwhile.
func TestFlushSnapshot_AnExpiredLockCannotReplaceANewerCheckpoint(t *testing.T) {
	t.Parallel()
	ctx := context.Background()
	a := newParkedTestAdapter(t)
	appendAll(t, a, fillers(t, 3)...)
	checkpointDuringWrite(t, a)
	if err := a.FlushSnapshot(ctx, "room", textEdit(9, "y", "y")); err != nil {
		t.Fatal(err)
	}
	a.beforeV2Write = nil
	v1, _, err := a.loadPrimaryV2(ctx, "room")
	if err != nil {
		t.Fatal(err)
	}
	if x, _ := loaded(t, v1, "x"); x != "x" {
		t.Fatalf("doc_v2 x = %q, want \"x\"; the other instance's checkpoint was replaced", x)
	}
	if y, _ := loaded(t, v1, "y"); y != "y" {
		t.Fatalf("doc_v2 y = %q, want \"y\"", y)
	}
}

// TestCompact_AnExpiredLockDeletesNothing: a compaction whose lock expired after
// it read the stored state must neither replace the newer checkpoint nor delete
// the update objects it folded.
func TestCompact_AnExpiredLockDeletesNothing(t *testing.T) {
	t.Parallel()
	ctx := context.Background()
	a := newParkedTestAdapter(t)
	appendAll(t, a, fillers(t, 12)...)
	checkpointDuringWrite(t, a)
	n, err := a.Compact(ctx, "room", 1)
	a.beforeV2Write = nil
	if !errors.Is(err, errGenerationMoved) || n != 0 {
		t.Fatalf("Compact = %d, %v; want 0 deleted and errGenerationMoved", n, err)
	}
	vs, err := a.ListVersions(ctx, "room")
	if err != nil || len(vs) != 12 {
		t.Fatalf("versions = %d, %v; want all 12 kept", len(vs), err)
	}
	v1, _, err := a.loadPrimaryV2(ctx, "room")
	if err != nil {
		t.Fatal(err)
	}
	if x, _ := loaded(t, v1, "x"); x != "x" {
		t.Fatalf("doc_v2 x = %q, want \"x\"; the other instance's checkpoint was replaced", x)
	}
	if v := fillerValue(t, mustLoad(t, a)); v != 11 {
		t.Fatalf("f.k = %v, want 11", v)
	}
}

// lockerStub is a Locker whose WithLock can be switched to unavailable, as a
// contended doc lock is once its acquisition budget runs out. Each Locker has
// its own timeout error, so what matters is that fn never runs.
type lockerStub struct {
	inner       Locker
	unavailable atomic.Bool
	refused     atomic.Int64
}

func (l *lockerStub) WithLock(ctx context.Context, key string, fn func(context.Context) error) error {
	if l.unavailable.Load() {
		l.refused.Add(1)
		return ErrLockTimeout
	}
	return l.inner.WithLock(ctx, key, fn)
}

func (l *lockerStub) TryWithLock(ctx context.Context, key string, ttl time.Duration, fn func(context.Context) error) error {
	return l.inner.TryWithLock(ctx, key, ttl, fn)
}

func newLockerStubAdapter(t *testing.T) (*Adapter, *lockerStub) {
	t.Helper()
	client, bucket := newFakeGCS(t)
	lk := &lockerStub{inner: NewNoLock()}
	a, err := New(Options{Client: client, Bucket: bucket, Locker: lk})
	if err != nil {
		t.Fatal(err)
	}
	return a, lk
}

// movingWriter moves the room's marks from inside readState, as another
// instance's checkpoint landing mid-read does, for the first n reads. It writes
// the checkpoint directly because AppendUpdate would need the doc lock, which
// these tests make unavailable.
func movingWriter(t *testing.T, a *Adapter, n int) func() {
	t.Helper()
	moved := 0
	return func() {
		if moved >= n {
			return
		}
		moved++
		if err := a.store.put(context.Background(), a.layout.CheckpointName("room"), be32(uint32(moved))); err != nil {
			t.Error(err)
		}
	}
}

// TestLoad_OpensWhenTheDocLockIsUnavailable: the doc lock is an optimisation for
// Load's read, not a correctness requirement, and Load is the connect path. A
// room being written hard is exactly when the lock is contended and also exactly
// when someone is trying to open it, so an unavailable lock must degrade to
// another unlocked read rather than fail the open.
func TestLoad_OpensWhenTheDocLockIsUnavailable(t *testing.T) {
	t.Parallel()
	a, lk := newLockerStubAdapter(t)
	fs := fillers(t, 10)
	appendAll(t, a, fs[:9]...) // clocks 1..9, while the lock still works

	lk.unavailable.Store(true)
	// Enough to exhaust the first unlocked round, then the writer is done.
	a.afterReadBase = movingWriter(t, a, loadAttempts-1)

	state := mustLoad(t, a)
	a.afterReadBase = nil

	if got := fillerValue(t, state); got != 8 {
		t.Fatalf("f.k = %v, want 8", got)
	}
	if n := lk.refused.Load(); n != 1 {
		t.Errorf("the doc lock was asked for %d times, want exactly 1", n)
	}
}

// TestLoad_StillRefusesAStateThatKeepsMoving: the fallback above widens the
// window Load tolerates, and must not widen what it is willing to serve. A
// writer that never stops leaves every attempt inconsistent, so Load fails
// rather than hand back a state assembled across a write.
func TestLoad_StillRefusesAStateThatKeepsMoving(t *testing.T) {
	t.Parallel()
	a, lk := newLockerStubAdapter(t)
	appendAll(t, a, fillers(t, 9)...)

	lk.unavailable.Store(true)
	a.afterReadBase = movingWriter(t, a, 1<<30) // never stops
	defer func() { a.afterReadBase = nil }()

	_, err := a.Load(context.Background(), "room")
	if !errors.Is(err, errStateMoved) {
		t.Fatalf("Load error = %v, want errStateMoved", err)
	}
}

// TestMaterializeAt_RefusesAVersionBelowAnImportedBase: Backfill copies the
// legacy root into doc_v2 and writes no update objects, so a log that is
// contiguous from clock 1 still does not carry that content. Below the
// checkpoint the base is excluded, so such a version rebuilds short.
func TestMaterializeAt_RefusesAVersionBelowAnImportedBase(t *testing.T) {
	ctx := context.Background()
	client, bucket := newFakeGCS(t)
	const room = "proj-imported-base"
	seedLegacy(t, client, bucket, room)

	p2, err := New(Options{Client: client, Bucket: bucket, Locker: NewNoLock(), Phase2: true})
	if err != nil {
		t.Fatalf("New phase2: %v", err)
	}
	if err := p2.Backfill(ctx, room); err != nil {
		t.Fatalf("Backfill: %v", err)
	}

	// Twelve primary edits: the tenth trips the inline compaction, so the
	// checkpoint lands at 10 with clocks 1..12 all still on disk.
	doc := crdt.New(crdt.WithClientID(11))
	txt := doc.GetText("p")
	var prev []byte
	for i := 0; i < 12; i++ {
		doc.Transact(func(txn *crdt.Transaction) { txt.Insert(txn, 0, "x", nil) })
		full := crdt.EncodeStateAsUpdateV1(doc, nil)
		inc := full
		if prev != nil {
			d, derr := crdt.DiffUpdateV1(full, svOf(t, prev))
			if derr != nil {
				t.Fatalf("DiffUpdateV1: %v", derr)
			}
			inc = d
		}
		if _, err := p2.AppendUpdate(ctx, room, inc); err != nil {
			t.Fatalf("AppendUpdate: %v", err)
		}
		prev = full
	}

	cp, err := p2.checkpoint(ctx, room)
	if err != nil {
		t.Fatalf("checkpoint: %v", err)
	}
	if cp == 0 {
		t.Fatal("setup: the inline compaction never advanced the checkpoint")
	}

	if _, err := p2.MaterializeAt(ctx, room, persistence.Version(5)); !errors.Is(err, ErrVersionNotRebuildable) {
		t.Errorf("MaterializeAt(5) with checkpoint %d = %v, want ErrVersionNotRebuildable", cp, err)
	}
}

// TestWriteV2Doc_LeavesTheMirrorsWhenDocV2MovedOn: the SUB_DOC and SUB_STATE_VEC
// writes carry no precondition, so a writer whose lease lapsed could replace a
// newer checkpoint's mirrors with older state and leave the three objects
// disagreeing. The mirrors are now skipped once doc_v2 is no longer ours.
func TestWriteV2Doc_LeavesTheMirrorsWhenDocV2MovedOn(t *testing.T) {
	t.Parallel()

	ctx := context.Background()
	client, bucket := newFakeGCS(t)
	a, err := New(Options{Client: client, Bucket: bucket, Locker: NewNoLock()})
	if err != nil {
		t.Fatalf("New: %v", err)
	}
	const room = "mirrors"
	oid, err := a.oidFor(ctx, room)
	if err != nil {
		t.Fatalf("oidFor: %v", err)
	}

	winner := crdt.New(crdt.WithClientID(1))
	wt := winner.GetText("t") // outside Transact: GetText takes the doc lock
	winner.Transact(func(txn *crdt.Transaction) { wt.Insert(txn, 0, "winner", nil) })
	if err := a.writeV2Doc(ctx, room, oid, winner); err != nil {
		t.Fatalf("seed writeV2Doc: %v", err)
	}
	wantDoc, err := a.store.get(ctx, a.layout.DocStateName(room, oid))
	if err != nil {
		t.Fatalf("read seeded SUB_DOC: %v", err)
	}
	wantSV, err := a.store.get(ctx, a.layout.StateVectorName(room, oid))
	if err != nil {
		t.Fatalf("read seeded SUB_STATE_VEC: %v", err)
	}

	// A lapsed writer: its doc_v2 put reports a generation that is no longer the
	// one on the bucket, which is what a newer publish leaves behind.
	stale := crdt.New(crdt.WithClientID(2))
	st := stale.GetText("t")
	stale.Transact(func(txn *crdt.Transaction) { st.Insert(txn, 0, "stale", nil) })
	err = a.writeV2DocWith(ctx, room, oid, stale, func(string, []byte) (int64, error) {
		return 999999, nil
	})
	if err != nil {
		t.Fatalf("writeV2DocWith: %v", err)
	}

	gotDoc, err := a.store.get(ctx, a.layout.DocStateName(room, oid))
	if err != nil {
		t.Fatalf("read SUB_DOC: %v", err)
	}
	if !bytes.Equal(gotDoc, wantDoc) {
		t.Error("a lapsed writer replaced SUB_DOC, so it no longer matches doc_v2")
	}
	gotSV, err := a.store.get(ctx, a.layout.StateVectorName(room, oid))
	if err != nil {
		t.Fatalf("read SUB_STATE_VEC: %v", err)
	}
	if !bytes.Equal(gotSV, wantSV) {
		t.Error("a lapsed writer replaced SUB_STATE_VEC, so it no longer matches doc_v2")
	}
}

// TestRollbackCommitted: writeUnion drops a flush candidate when a rollback
// landed after the candidate was built, but a ceiling merely being cleared is an
// ordinary append after a rollback the candidate already reflects.
func TestRollbackCommitted(t *testing.T) {
	t.Parallel()

	t.Run("a ceiling appearing is a rollback", func(t *testing.T) {
		t.Parallel()
		if !rollbackCommitted(marks{cp: 9}, marks{cp: 5, ceil: 5, hasCeil: true}) {
			t.Error("a new ceiling was not treated as a rollback")
		}
	})

	t.Run("a ceiling moving is another rollback", func(t *testing.T) {
		t.Parallel()
		if !rollbackCommitted(marks{cp: 9, ceil: 9, hasCeil: true}, marks{cp: 5, ceil: 5, hasCeil: true}) {
			t.Error("a moved ceiling was not treated as a rollback")
		}
	})

	t.Run("a checkpoint going backwards is a rollback", func(t *testing.T) {
		t.Parallel()
		if !rollbackCommitted(marks{cp: 9}, marks{cp: 4}) {
			t.Error("a checkpoint regression was not treated as a rollback")
		}
	})

	t.Run("a ceiling being cleared is not", func(t *testing.T) {
		t.Parallel()
		if rollbackCommitted(marks{cp: 5, ceil: 5, hasCeil: true}, marks{cp: 5}) {
			t.Error("clearing a ceiling dropped a candidate built after that rollback")
		}
	})

	t.Run("an ordinary checkpoint advance is not", func(t *testing.T) {
		t.Parallel()
		if rollbackCommitted(marks{cp: 5}, marks{cp: 9}) {
			t.Error("a competing checkpoint dropped the candidate; the union should keep both writers' edits")
		}
	})
}

// TestWriteUnion_DropsACandidateBuiltBeforeARollback: the flush candidate is the
// live room state, built before a rollback that then commits. Applying it would
// union the rolled-back edits straight back into doc_v2.
func TestWriteUnion_DropsACandidateBuiltBeforeARollback(t *testing.T) {
	t.Parallel()

	ctx := context.Background()
	client, bucket := newFakeGCS(t)
	a, err := New(Options{Client: client, Bucket: bucket, Locker: NewNoLock()})
	if err != nil {
		t.Fatalf("New: %v", err)
	}
	const room = "pre-rollback-candidate"
	oid, err := a.oidFor(ctx, room)
	if err != nil {
		t.Fatalf("oidFor: %v", err)
	}

	doc := crdt.New(crdt.WithClientID(5))
	txt := doc.GetText("t")
	var prev []byte
	for i := 0; i < 6; i++ {
		doc.Transact(func(txn *crdt.Transaction) { txt.Insert(txn, txt.Len(), string(rune('a'+i)), nil) })
		full := crdt.EncodeStateAsUpdateV1(doc, nil)
		inc := full
		if prev != nil {
			d, derr := crdt.DiffUpdateV1(full, svOf(t, prev))
			if derr != nil {
				t.Fatalf("DiffUpdateV1: %v", derr)
			}
			inc = d
		}
		if _, err := a.AppendUpdate(ctx, room, inc); err != nil {
			t.Fatalf("AppendUpdate: %v", err)
		}
		prev = full
	}
	candidate := prev // the live state, "abcdef"

	before, err := a.readMarks(ctx, room)
	if err != nil {
		t.Fatalf("readMarks: %v", err)
	}

	rolled, err := a.MaterializeAt(ctx, room, persistence.Version(3))
	if err != nil {
		t.Fatalf("MaterializeAt(3): %v", err)
	}
	if err := a.PruneAfter(ctx, room, persistence.Version(3), rolled); err != nil {
		t.Fatalf("PruneAfter: %v", err)
	}

	if err := a.writeUnionOnce(ctx, room, oid, candidate, before); err != nil {
		t.Fatalf("writeUnionOnce: %v", err)
	}

	v1, ok, err := a.loadV2(ctx, room)
	if err != nil || !ok {
		t.Fatalf("loadV2: ok=%v err=%v", ok, err)
	}
	got := crdt.New()
	if err := crdt.ApplyUpdateV1(got, v1, nil); err != nil {
		t.Fatalf("ApplyUpdateV1: %v", err)
	}
	if s := got.GetText("t").ToString(); s != "abc" {
		t.Errorf("doc_v2 text = %q, want %q: the pre-rollback candidate restored edits the rollback removed", s, "abc")
	}
}

// staleMirrorWrite runs a writer whose lease lapsed: it publishes doc_v2, then
// pauses before writing the mirror named by pauseAt while a newer writer
// publishes doc_v2 and both mirrors. It returns the newer writer's doc.
func staleMirrorWrite(t *testing.T, a *Adapter, room string, oid uint32, pauseAt string) *crdt.Doc {
	t.Helper()
	ctx := context.Background()

	newer := crdt.New(crdt.WithClientID(1))
	nt := newer.GetText("t") // outside Transact: GetText takes the doc lock
	newer.Transact(func(txn *crdt.Transaction) { nt.Insert(txn, 0, "newer", nil) })

	fired := false
	a.beforeMirrorPut = func(name string) {
		if fired || name != pauseAt {
			return
		}
		fired = true
		g, err := a.store.generation(ctx, a.layout.DocV2Name(room))
		if err != nil {
			t.Errorf("generation: %v", err)
			return
		}
		if _, err := a.writeV2DocIf(ctx, room, oid, newer, g); err != nil {
			t.Errorf("newer writeV2DocIf: %v", err)
		}
	}

	stale := crdt.New(crdt.WithClientID(2))
	st := stale.GetText("t")
	stale.Transact(func(txn *crdt.Transaction) { st.Insert(txn, 0, "stale", nil) })
	if _, err := a.writeV2DocIf(ctx, room, oid, stale, 0); err != nil {
		t.Fatalf("stale writeV2DocIf: %v", err)
	}
	if !fired {
		t.Fatal("setup: the stale writer never reached the paused mirror")
	}
	return newer
}

func assertMirrorsMatch(t *testing.T, a *Adapter, room string, oid uint32, want *crdt.Doc) {
	t.Helper()
	ctx := context.Background()
	v1, err := a.store.get(ctx, a.layout.DocStateName(room, oid))
	if err != nil {
		t.Fatalf("read SUB_DOC: %v", err)
	}
	got, err := docFromV1(v1)
	if err != nil {
		t.Fatalf("decode SUB_DOC: %v", err)
	}
	if s, w := got.GetText("t").ToString(), want.GetText("t").ToString(); s != w {
		t.Errorf("SUB_DOC text = %q, want the newer writer's %q", s, w)
	}
	sv, err := a.store.get(ctx, a.layout.StateVectorName(room, oid))
	if err != nil {
		t.Fatalf("read SUB_STATE_VEC: %v", err)
	}
	if !bytes.Equal(sv, crdt.EncodeStateVectorV1(want)) {
		t.Error("SUB_STATE_VEC is not the newer writer's state vector")
	}
}

// TestWriteV2Doc_StaleWriterPausedAfterItsCheckLeavesTheMirrors: the lapsed
// writer passes its doc_v2 check, then pauses before SUB_DOC while a newer
// writer publishes everything. Its mirror writes must fail when it resumes.
func TestWriteV2Doc_StaleWriterPausedAfterItsCheckLeavesTheMirrors(t *testing.T) {
	t.Parallel()
	ctx := context.Background()
	client, bucket := newFakeGCS(t)
	a, err := New(Options{Client: client, Bucket: bucket, Locker: NewNoLock()})
	if err != nil {
		t.Fatalf("New: %v", err)
	}
	const room = "mirrors-pause"
	oid, err := a.oidFor(ctx, room)
	if err != nil {
		t.Fatalf("oidFor: %v", err)
	}
	newer := staleMirrorWrite(t, a, room, oid, a.layout.DocStateName(room, oid))
	assertMirrorsMatch(t, a, room, oid, newer)
}

// TestWriteV2Doc_StaleWriterPausedBetweenMirrorsLeavesTheSecond: the pause falls
// after SUB_DOC and before SUB_STATE_VEC, so the newer writer replaces SUB_DOC
// and the stale writer must not then replace its SUB_STATE_VEC.
func TestWriteV2Doc_StaleWriterPausedBetweenMirrorsLeavesTheSecond(t *testing.T) {
	t.Parallel()
	ctx := context.Background()
	client, bucket := newFakeGCS(t)
	a, err := New(Options{Client: client, Bucket: bucket, Locker: NewNoLock()})
	if err != nil {
		t.Fatalf("New: %v", err)
	}
	const room = "mirrors-between"
	oid, err := a.oidFor(ctx, room)
	if err != nil {
		t.Fatalf("oidFor: %v", err)
	}
	newer := staleMirrorWrite(t, a, room, oid, a.layout.StateVectorName(room, oid))
	assertMirrorsMatch(t, a, room, oid, newer)
}

// TestFlushRoom_DropsAStateTakenBeforeARollback: the flusher takes the live
// state, then a rollback commits before FlushSnapshot gets the doc lock. Marks
// read under that lock already include the rollback, so the flusher reads them
// before taking the state.
func TestFlushRoom_DropsAStateTakenBeforeARollback(t *testing.T) {
	t.Parallel()
	ctx := context.Background()
	client, bucket := newFakeGCS(t)
	a, err := New(Options{Client: client, Bucket: bucket, Locker: NewNoLock()})
	if err != nil {
		t.Fatalf("New: %v", err)
	}
	const room = "flush-before-rollback"

	doc := crdt.New(crdt.WithClientID(5))
	txt := doc.GetText("t")
	var prev []byte
	for i := 0; i < 6; i++ {
		doc.Transact(func(txn *crdt.Transaction) { txt.Insert(txn, txt.Len(), string(rune('a'+i)), nil) })
		full := crdt.EncodeStateAsUpdateV1(doc, nil)
		inc := full
		if prev != nil {
			d, derr := crdt.DiffUpdateV1(full, svOf(t, prev))
			if derr != nil {
				t.Fatalf("DiffUpdateV1: %v", derr)
			}
			inc = d
		}
		if _, err := a.AppendUpdate(ctx, room, inc); err != nil {
			t.Fatalf("AppendUpdate: %v", err)
		}
		prev = full
	}
	live := prev // "abcdef"

	f := NewFlusher(FlusherOptions{Adapter: a, StateOf: func(string) []byte {
		state := live
		// The rollback commits once the state is taken, before the flush locks.
		rolled, err := a.MaterializeAt(ctx, room, persistence.Version(3))
		if err != nil {
			t.Errorf("MaterializeAt(3): %v", err)
			return state
		}
		if err := a.PruneAfter(ctx, room, persistence.Version(3), rolled); err != nil {
			t.Errorf("PruneAfter: %v", err)
		}
		return state
	}})
	if err := f.FlushRoom(ctx, room); err != nil {
		t.Fatalf("FlushRoom: %v", err)
	}

	v1, ok, err := a.loadV2(ctx, room)
	if err != nil || !ok {
		t.Fatalf("loadV2: ok=%v err=%v", ok, err)
	}
	got := crdt.New()
	if err := crdt.ApplyUpdateV1(got, v1, nil); err != nil {
		t.Fatalf("ApplyUpdateV1: %v", err)
	}
	if s := got.GetText("t").ToString(); s != "abc" {
		t.Errorf("doc_v2 text = %q, want %q: the flush restored edits the rollback removed", s, "abc")
	}
}
