package gcs

import (
	"context"

	"github.com/reearth/ygo/crdt"
	"github.com/reearth/ygo/persistence"
)

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

// finishInterruptedPrune completes a PruneAfter that crashed before deleting the
// future update objects, removing every orphan update > ceiling under gcs:lock so
// the recovery path can clear the ceiling safely. Idempotent.
func (a *Adapter) finishInterruptedPrune(ctx context.Context, room DocID, oid, ceiling uint32) error {
	return a.locker.WithLock(ctx, gcsDocLockKey(room), func(ctx context.Context) error {
		return a.deleteUpdatesAfter(ctx, room, oid, ceiling)
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
// updates exist. The new base is written before any update is deleted.
func (a *Adapter) Compact(ctx context.Context, room string, keep int) (int, error) {
	if err := a.validate(room); err != nil {
		return 0, err
	}
	oid, err := a.oidFor(ctx, room)
	if err != nil {
		return 0, err
	}
	var deleted int
	err = a.locker.WithLock(ctx, gcsDocLockKey(room), func(ctx context.Context) error {
		updates, err := a.listUpdates(ctx, room, oid)
		if err != nil {
			return err
		}
		clocks := sortedClocks(updates)
		if len(clocks) <= keep {
			return nil
		}
		foldUpto := len(clocks) - keep // fold clocks[0:foldUpto] into the base
		cutoff := clocks[foldUpto-1]   // highest clock folded in

		cp, err := a.checkpoint(ctx, room)
		if err != nil {
			return err
		}
		var parts [][]byte
		if v1, ok, err := a.loadV2(ctx, room); err != nil {
			return err
		} else if ok && cp > 0 {
			parts = append(parts, v1)
		} else if !ok {
			if b, err := a.store.get(ctx, a.layout.DocStateName(room, oid)); err == nil && cp > 0 {
				parts = append(parts, b)
			} else if err != nil && err != errNotFound {
				return err
			}
		}
		for i := 0; i < foldUpto; i++ {
			c := clocks[i]
			if c <= cp {
				continue
			}
			b, err := a.store.get(ctx, updates[c])
			if err != nil {
				return err
			}
			parts = append(parts, b)
		}
		var newBase []byte
		if len(parts) > 0 {
			newBase, err = crdt.MergeUpdatesV1(parts...)
			if err != nil {
				return err
			}
		}
		// Write the new base before deleting any update (abort-without-loss).
		if err := a.writeV2Snapshot(ctx, room, oid, newBase); err != nil {
			return err
		}
		if cutoff > cp {
			if err := a.store.put(ctx, a.layout.CheckpointName(room), be32(cutoff)); err != nil {
				return err
			}
		}
		for i := 0; i < foldUpto; i++ {
			if err := a.store.delete(ctx, updates[clocks[i]]); err != nil {
				return err
			}
			deleted++
		}
		return nil
	})
	return deleted, err
}

// compactToCheckpoint is the every-10th-clock inline compaction: refresh the
// base (SUB_DOC/SUB_STATE_VEC + doc_v2) from the materialized state and bump the
// checkpoint to clock. It does not delete update objects.
//
// Intentional deviation: this also writes doc_v2 (Rust writes it only on
// flush/snapshot). The name and encoding are byte-identical, so it is a safe extra.
func (a *Adapter) compactToCheckpoint(ctx context.Context, room DocID, oid, clock uint32) error {
	state, err := a.MaterializeAt(ctx, room, persistence.Version(clock))
	if err != nil {
		return err
	}
	if err := a.writeV2Snapshot(ctx, room, oid, state); err != nil {
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

// Reopen returns a fresh Adapter over the same bucket/layout/locker, simulating a
// process restart: the crash predicate is dropped and the OID cache is empty so
// OIDs are re-derived from the durable index.
func (a *Adapter) Reopen() (persistence.VersionedPersistence, error) {
	n := &Adapter{
		store:    a.store,
		layout:   a.layout,
		fallback: a.fallback,
		locker:   a.locker,
		log:      a.log,
		phase2:   a.phase2,
		oidCache: make(map[string]uint32),
	}
	return n, nil
}
