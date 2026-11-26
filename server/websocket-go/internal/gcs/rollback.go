package gcs

import (
	"context"
	"errors"

	"github.com/reearth/ygo/persistence"
)

// errSimulatedRecoveryCrash is returned by AppendUpdate when the recovery-write
// crash seam fires. It models an unacked crash and is used only by tests.
var errSimulatedRecoveryCrash = errors.New("gcs: simulated crash after recovery write (test injection)")

// PruneAfter is the crash-safe rollback primitive: snapshot-before-delete. Under
// gcs:lock it (1) writes rolledBack as the new doc_v2, (2) writes checkpoint and
// ceiling = target, then (3) deletes every update > target. A crash between (2)
// and (3) is safe: the ceiling and v2-first Load hide any surviving update > target.
func (a *Adapter) PruneAfter(ctx context.Context, room string, target persistence.Version, rolledBack []byte) error {
	if err := a.validate(room); err != nil {
		return err
	}
	oid, err := a.oidFor(ctx, room)
	if err != nil {
		return err
	}
	t := uint32(target)
	return a.locker.WithLock(ctx, gcsDocLockKey(room), func(ctx context.Context) error {
		// (1) durable rolled-back snapshot.
		if err := a.writeV2Snapshot(ctx, room, oid, rolledBack); err != nil {
			return err
		}
		// (2a) ceiling = target, written before the delete so a crash hides any
		// surviving update > target.
		if err := a.store.put(ctx, a.ceilingName(room), be32(t)); err != nil {
			return err
		}
		// (2b) checkpoint = target.
		if err := a.store.put(ctx, a.layout.CheckpointName(room), be32(t)); err != nil {
			return err
		}
		// Conformance crash injection: return after the durable checkpoint+ceiling
		// but before deleting future updates.
		a.mu.Lock()
		crash := a.crashAfterCheckpoint
		a.mu.Unlock()
		if crash != nil && crash() {
			return nil
		}
		// (3) delete every update > target.
		return a.deleteUpdatesAfter(ctx, room, oid, t)
	})
}

// deleteUpdatesAfter removes every update object whose clock > target.
func (a *Adapter) deleteUpdatesAfter(ctx context.Context, room DocID, oid, target uint32) error {
	updates, err := a.listUpdates(ctx, room, oid)
	if err != nil {
		return err
	}
	for c, name := range updates {
		if c > target {
			if err := a.store.delete(ctx, name); err != nil {
				return err
			}
		}
	}
	return nil
}

// Compact folds the oldest updates into the base snapshot, keeping at most `keep`
// recent updates and advancing the checkpoint monotonically. No-op when ≤ keep
// updates exist, and when keep ≤ 0, which ygo's stores read as "keep every
// update". The new base is written before any update is deleted.
func (a *Adapter) Compact(ctx context.Context, room string, keep int) (int, error) {
	// Validation runs first so an unusable room reports itself whatever keep is.
	if err := a.validate(room); err != nil {
		return 0, err
	}
	if keep <= 0 {
		return 0, nil
	}
	oid, err := a.oidFor(ctx, room)
	if err != nil {
		return 0, err
	}
	var deleted int
	err = a.locker.WithLock(ctx, gcsDocLockKey(room), func(ctx context.Context) error {
		// Read before anything else, so a rollback that rewrites the checkpoint at
		// any later point makes the conditional publish below fail.
		cpGen, err := a.store.generation(ctx, a.layout.CheckpointName(room))
		if err != nil {
			return err
		}
		// Attrs rather than names: the deletes below are pinned to the generation
		// listed here, so an object recreated under the same clock survives them.
		listed, err := a.listUpdatesAttrs(ctx, room, oid)
		if err != nil {
			return err
		}
		updates := make(map[uint32]string, len(listed))
		for c, at := range listed {
			updates[c] = at.Name
		}
		m, err := a.readMarks(ctx, room)
		if err != nil {
			return err
		}
		clocks := sortedClocks(updates)
		// An interrupted rollback leaves update objects above the ceiling. Those are
		// abandoned rather than history, so a cutoff chosen among them would publish
		// a checkpoint the surviving log does not reach. Rollback recovery removes
		// them; compaction leaves them alone.
		if m.hasCeil {
			clocks = clocksUpTo(clocks, m.ceil)
		}
		if len(clocks) <= keep {
			return nil
		}
		foldUpto := len(clocks) - keep // fold clocks[0:foldUpto] into the base
		cutoff := clocks[foldUpto-1]   // highest clock folded in
		if a.afterCompactSelect != nil {
			a.afterCompactSelect()
		}

		// Bounded to cutoff: the kept objects stay above the new checkpoint, where
		// Load folds them and MaterializeAt replays them per version. Folding them
		// into the base instead would make every kept version rebuild as the
		// latest state.
		st, err := a.readStateUpTo(ctx, room, oid, cutoff)
		if err != nil {
			return err
		}
		// A lapsed lease does not stop this section, so another holder can roll back
		// between the selection above and that read. Everything below acts on the
		// selection, so it must still describe the state just read.
		if st.marks != m || !sameObjects(st.updates, updates) {
			a.log.Warn("gcs compact: the stored state moved after the selection, leaving it for the next run",
				"room", room, "cutoff", cutoff)
			return errStateMoved
		}
		// readState applied the objects above the checkpoint. The checkpointed ones
		// go in too: a doc_v2 written before checkpoints were unions can lack one.
		for _, c := range clocks[:foldUpto] {
			if c > m.cp {
				continue
			}
			if err := a.applyObject(ctx, st.doc, updates[c]); err != nil {
				return err
			}
		}
		if ps := st.doc.PendingStats(); ps.Items > 0 {
			a.log.Debug("gcs compact: new base holds parked items",
				"room", room, "parked", ps.Items, "awaiting_clients", len(ps.MissingFor), "kept", keep)
		}
		// Write the new base before deleting any update (abort-without-loss). It
		// only lands on the doc_v2 readState read, so an expired lock deletes nothing.
		baseGen, err := a.writeV2DocIf(ctx, room, oid, st.doc, st.v2Gen)
		if err != nil {
			return err
		}
		if a.afterCompactBase != nil {
			a.afterCompactBase()
		}
		// The checkpoint gets its own fence: a rollback since the selection has
		// rewritten it, and a stale cutoff over that would hide acknowledged updates.
		if cutoff > m.cp {
			g, err := a.store.putIfGeneration(ctx, a.layout.CheckpointName(room), be32(cutoff), cpGen)
			if errors.Is(err, errGenerationMoved) {
				a.log.Warn("gcs compact: the checkpoint moved since the selection, leaving it for the next run",
					"room", room, "cutoff", cutoff)
				return errStateMoved
			}
			if err != nil {
				return err
			}
			cpGen = g
		}
		if a.afterCompactPublish != nil {
			a.afterCompactPublish()
		}
		// Deletes cannot be undone, so confirm both objects are still the ones this
		// run wrote before removing anything they fold in.
		moved, err := a.compactPublishMoved(ctx, room, baseGen, cpGen)
		if err != nil {
			return err
		}
		if moved {
			a.log.Warn("gcs compact: doc_v2 or the checkpoint moved after publishing, deleting nothing",
				"room", room, "cutoff", cutoff)
			return errStateMoved
		}
		if a.beforeCompactDelete != nil {
			a.beforeCompactDelete()
		}
		// The check above cannot cover the deletes themselves: a lease can lapse
		// after it, and another holder can roll back and reuse a folded clock's
		// name. Each delete only lands on the object generation that was folded.
		for i := 0; i < foldUpto; i++ {
			at := listed[clocks[i]]
			err := a.store.deleteIfGeneration(ctx, at.Name, at.Generation)
			if errors.Is(err, errGenerationMoved) {
				a.log.Warn("gcs compact: an update was rewritten after it was folded, deleting no further",
					"room", room, "clock", clocks[i])
				return errStateMoved
			}
			if err != nil {
				return err
			}
			deleted++
		}
		return nil
	})
	return deleted, err
}

// sameObjects reports whether two update listings name the same objects.
func sameObjects(a, b map[uint32]string) bool {
	if len(a) != len(b) {
		return false
	}
	for c, n := range a {
		if b[c] != n {
			return false
		}
	}
	return true
}

// compactPublishMoved reports whether doc_v2 or the checkpoint changed after
// Compact wrote them, which a holder that took over a lapsed lease can do.
func (a *Adapter) compactPublishMoved(ctx context.Context, room DocID, baseGen, cpGen int64) (bool, error) {
	v2, err := a.store.generation(ctx, a.layout.DocV2Name(room))
	if err != nil {
		return false, err
	}
	if baseGen != 0 && v2 != baseGen {
		return true, nil
	}
	cg, err := a.store.generation(ctx, a.layout.CheckpointName(room))
	if err != nil {
		return false, err
	}
	return cg != cpGen, nil
}

// compactToCheckpoint is the every-10th-clock inline compaction: rewrite the
// base (SUB_DOC/SUB_STATE_VEC + doc_v2) from the stored state and bump the
// checkpoint to clock. It does not delete update objects. The caller holds
// gcsDocLockKey.
//
// Intentional deviation: this also writes doc_v2 (Rust writes it only on
// flush/snapshot). The name and encoding are byte-identical, so it is a safe extra.
func (a *Adapter) compactToCheckpoint(ctx context.Context, room DocID, oid, clock uint32) error {
	if err := a.writeUnion(ctx, room, oid, nil); err != nil {
		return err
	}
	cp, err := a.checkpoint(ctx, room)
	if err != nil {
		return err
	}
	if clock > cp { // monotonic
		if err := a.store.put(ctx, a.layout.CheckpointName(room), be32(clock)); err != nil {
			return err
		}
	}
	return nil
}

// SetCrashAfterCheckpoint installs a crash predicate for the conformance
// CrashInjector: when true, PruneAfter returns after the checkpoint write but
// before deleting future updates.
func (a *Adapter) SetCrashAfterCheckpoint(fn func() bool) {
	a.mu.Lock()
	a.crashAfterCheckpoint = fn
	a.mu.Unlock()
}

// SetCrashAfterRecoveryWrite installs a crash predicate (test injection): when
// true, AppendUpdate returns errSimulatedRecoveryCrash right after durably
// writing the recovery update, before any subsequent bookkeeping.
func (a *Adapter) SetCrashAfterRecoveryWrite(fn func() bool) {
	a.mu.Lock()
	a.crashAfterRecoveryWrite = fn
	a.mu.Unlock()
}

// Reopen returns a fresh Adapter over the same bucket/layout/locker, simulating a
// process restart: the crash predicate is dropped and the OID cache is empty so
// OIDs are re-derived from the durable index.
func (a *Adapter) Reopen() (persistence.VersionedPersistence, error) {
	n := &Adapter{
		store:         a.store,
		layout:        a.layout,
		fallback:      a.fallback,
		locker:        a.locker,
		log:           a.log,
		phase2:        a.phase2,
		oidCache:      make(map[string]uint32),
		transientKeys: a.transientKeys,
	}
	return n, nil
}
