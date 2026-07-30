package gcs

import (
	"context"
	"testing"

	"cloud.google.com/go/storage"
	"github.com/reearth/ygo/crdt"
	"github.com/reearth/ygo/persistence"
)

// genIncUpdates appends n single-character inserts to a fresh doc and returns the
// incremental V1 updates plus the final text.
func genIncUpdates(t *testing.T, n int) (updates [][]byte, finalText string) {
	t.Helper()
	doc := crdt.New(crdt.WithClientID(7))
	txt := doc.GetText("t")
	var prev []byte
	for i := 0; i < n; i++ {
		ch := string(rune('a' + i))
		doc.Transact(func(txn *crdt.Transaction) { txt.Insert(txn, 0, ch, nil) })
		full := crdt.EncodeStateAsUpdateV1(doc, nil)
		var inc []byte
		if prev == nil {
			inc = full
		} else {
			d, err := crdt.DiffUpdateV1(full, svOf(t, prev))
			if err != nil {
				t.Fatalf("DiffUpdateV1: %v", err)
			}
			inc = d
		}
		updates = append(updates, inc)
		prev = full
	}
	return updates, txt.ToString()
}

// runCrashedPruneThenAppend: a PruneAfter(target=N) that crashes mid-delete leaves
// ceiling=N + orphans > N. The next AppendUpdate must finish the interrupted prune
// before clearing the ceiling, else orphans resurrect. After append + reopen, no
// version > N+1 may survive.
func runCrashedPruneThenAppend(t *testing.T, newAdapter func(t *testing.T) *Adapter) {
	t.Helper()
	ctx := context.Background()
	const target = persistence.Version(2)

	a := newAdapter(t)
	updates, _ := genIncUpdates(t, 5) // clocks 1..5
	for _, u := range updates {
		if _, err := a.AppendUpdate(ctx, "room", u); err != nil {
			t.Fatalf("AppendUpdate: %v", err)
		}
	}
	rolledBack, err := a.MaterializeAt(ctx, "room", target)
	if err != nil {
		t.Fatalf("MaterializeAt(%d): %v", target, err)
	}

	// Crash PruneAfter mid-prune: ceiling+checkpoint written, orphans 3,4,5 remain.
	a.SetCrashAfterCheckpoint(func() bool { return true })
	if err := a.PruneAfter(ctx, "room", target, rolledBack); err != nil {
		t.Fatalf("PruneAfter (crashing): %v", err)
	}
	a.SetCrashAfterCheckpoint(nil)

	// Sanity: the orphan objects are physically present.
	oid, err := a.oidFor(ctx, "room")
	if err != nil {
		t.Fatalf("oidFor: %v", err)
	}
	orphans, err := a.listUpdates(ctx, "room", oid)
	if err != nil {
		t.Fatalf("listUpdates: %v", err)
	}
	var orphanCount int
	for c := range orphans {
		if c > uint32(target) {
			orphanCount++
		}
	}
	if orphanCount == 0 {
		t.Fatalf("expected orphan update objects > %d after crashed prune; window not reproduced", target)
	}

	// Append on the recovery path: must finish the interrupted prune first.
	newUpd, _ := genIncUpdates(t, 1)
	v, err := a.AppendUpdate(ctx, "room", newUpd[0])
	if err != nil {
		t.Fatalf("AppendUpdate (recovery): %v", err)
	}
	if v != target+1 {
		t.Fatalf("recovery AppendUpdate version = %d, want %d (target+1)", v, target+1)
	}

	// Reopen (simulate a process restart reading durable state).
	reopened, err := a.Reopen()
	if err != nil {
		t.Fatalf("Reopen: %v", err)
	}

	metas, err := reopened.ListVersions(ctx, "room")
	if err != nil {
		t.Fatalf("ListVersions: %v", err)
	}
	for _, m := range metas {
		if m.Version > target+1 {
			t.Fatalf("RESURRECTED version %d after crashed-prune + recovery append (target=%d)", m.Version, target)
		}
	}
	lr, err := reopened.Load(ctx, "room")
	if err != nil {
		t.Fatalf("Load: %v", err)
	}
	if lr.Version > target+1 {
		t.Fatalf("Load head %d > target+1 (%d) after crashed-prune + recovery append", lr.Version, target+1)
	}
}

func TestCrashedPruneThenAppend_Phase1(t *testing.T) {
	runCrashedPruneThenAppend(t, func(t *testing.T) *Adapter {
		client, bucket := newFakeGCS(t)
		a, err := New(Options{Client: client, Bucket: bucket, Locker: NewNoLock()})
		if err != nil {
			t.Fatalf("New: %v", err)
		}
		return a
	})
}

func TestCrashedPruneThenAppend_Phase2(t *testing.T) {
	runCrashedPruneThenAppend(t, func(t *testing.T) *Adapter {
		client, bucket := newFakeGCS(t)
		a, err := New(Options{Client: client, Bucket: bucket, Locker: NewNoLock(), Phase2: true})
		if err != nil {
			t.Fatalf("New phase2: %v", err)
		}
		return a
	})
}

// runRecoveryWriteCrashNoSilentDrop: a crashed PruneAfter leaves ceiling=N +
// orphans. The recovery AppendUpdate finishes the prune and writes the new update
// at N+1, then the process crashes BEFORE the ceiling is cleared. After reopen,
// the durably-written update at N+1 must still be visible (at-least-once) — it
// must NOT be hidden by the stale ceiling, because a later recovery append would
// then physically delete it. This pins the recovery path as crash-idempotent.
func runRecoveryWriteCrashNoSilentDrop(t *testing.T, newAdapter func(t *testing.T) *Adapter) {
	t.Helper()
	ctx := context.Background()
	const target = persistence.Version(2)

	a := newAdapter(t)
	updates, _ := genIncUpdates(t, 5) // clocks 1..5
	for _, u := range updates {
		if _, err := a.AppendUpdate(ctx, "room", u); err != nil {
			t.Fatalf("AppendUpdate: %v", err)
		}
	}
	rolledBack, err := a.MaterializeAt(ctx, "room", target)
	if err != nil {
		t.Fatalf("MaterializeAt(%d): %v", target, err)
	}

	// Crash PruneAfter mid-prune: ceiling=2 + orphans 3,4,5 remain.
	a.SetCrashAfterCheckpoint(func() bool { return true })
	if err := a.PruneAfter(ctx, "room", target, rolledBack); err != nil {
		t.Fatalf("PruneAfter (crashing): %v", err)
	}
	a.SetCrashAfterCheckpoint(nil)

	// Recovery append that crashes right after durably writing update@(target+1),
	// before clearing the ceiling.
	a.SetCrashAfterRecoveryWrite(func() bool { return true })
	newUpd, _ := genIncUpdates(t, 1)
	if _, err := a.AppendUpdate(ctx, "room", newUpd[0]); err != errSimulatedRecoveryCrash {
		t.Fatalf("recovery AppendUpdate err = %v, want errSimulatedRecoveryCrash", err)
	}
	a.SetCrashAfterRecoveryWrite(nil)

	// Reopen (simulate a process restart reading durable state).
	reopened, err := a.Reopen()
	if err != nil {
		t.Fatalf("Reopen: %v", err)
	}

	lr, err := reopened.Load(ctx, "room")
	if err != nil {
		t.Fatalf("Load: %v", err)
	}
	if lr.Version != target+1 {
		t.Fatalf("recovery write at v%d not visible after crash+reopen: Load head=%d (stale ceiling silently hid the durable update)", target+1, lr.Version)
	}
}

func TestRecoveryWriteCrashNoSilentDrop_Phase1(t *testing.T) {
	runRecoveryWriteCrashNoSilentDrop(t, func(t *testing.T) *Adapter {
		client, bucket := newFakeGCS(t)
		a, err := New(Options{Client: client, Bucket: bucket, Locker: NewNoLock()})
		if err != nil {
			t.Fatalf("New: %v", err)
		}
		return a
	})
}

func TestRecoveryWriteCrashNoSilentDrop_Phase2(t *testing.T) {
	runRecoveryWriteCrashNoSilentDrop(t, func(t *testing.T) *Adapter {
		client, bucket := newFakeGCS(t)
		a, err := New(Options{Client: client, Bucket: bucket, Locker: NewNoLock(), Phase2: true})
		if err != nil {
			t.Fatalf("New phase2: %v", err)
		}
		return a
	})
}

var _ = storage.ErrObjectNotExist
