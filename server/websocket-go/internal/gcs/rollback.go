package gcs

import (
	"context"
	"crypto/rand"
	"encoding/hex"
	"errors"
	"fmt"

	"github.com/reearth/ygo/crdt"
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
	return a.pruneAfter(ctx, room, target, rolledBack, nil)
}

// pruneAfter is PruneAfter. With sinceEpoch set it announces itself only while
// the rollback epoch is still at that generation, so a rollback that committed
// after the caller built rolledBack makes it fail with errRollbackRaced instead
// of publishing a document rebuilt from the state that rollback replaced.
func (a *Adapter) pruneAfter(ctx context.Context, room string, target persistence.Version, rolledBack []byte, sinceEpoch *int64) error {
	if err := a.validate(room); err != nil {
		return err
	}
	oid, err := a.oidFor(ctx, room)
	if err != nil {
		return err
	}
	t := uint32(target)
	return a.locker.WithLock(ctx, gcsDocLockKey(room), func(ctx context.Context) error {
		// A rollback that stopped earlier is settled before this one starts.
		if err := a.settleRollback(ctx, room, oid); err != nil {
			return err
		}
		// The lock callback keeps running after a lapsed lease, so each write below
		// lands only on the generation read here. A holder that resumes after
		// another rollback committed fails instead of overwriting it.
		v2Gen, err := a.store.generation(ctx, a.layout.DocV2Name(room))
		if err != nil {
			return err
		}
		cpGen, err := a.store.generation(ctx, a.layout.CheckpointName(room))
		if err != nil {
			return err
		}
		ceilGen, err := a.store.generation(ctx, a.ceilingName(room))
		if err != nil {
			return err
		}
		prev, has, err := a.ceiling(ctx, room)
		if err != nil {
			return err
		}
		if !has {
			prev = noCeiling
		}
		doc, err := docFromV1(rolledBack)
		if err != nil {
			return err
		}
		op, opID, err := newRollbackOp()
		if err != nil {
			return err
		}
		// (0a) announce the rollback before changing anything, so a flush candidate
		// built earlier is dropped even if this rollback dies partway.
		if sinceEpoch != nil {
			if _, err := a.store.putIfGeneration(ctx, a.rollbackEpochName(room), be32(t), *sinceEpoch); err != nil {
				if errors.Is(err, errGenerationMoved) {
					return errRollbackRaced
				}
				return err
			}
		} else if err := a.store.put(ctx, a.rollbackEpochName(room), be32(t)); err != nil {
			return err
		}
		// (0b) Mark the ceiling, naming this rollback. Until (3) clears it, doc_v2,
		// the checkpoint and the ceiling may disagree, and readers refuse the mix.
		markGen, err := a.store.putIfGeneration(ctx, a.ceilingName(room), markedCeiling(prev, t, op), ceilGen)
		if err != nil {
			return lostLease(err)
		}
		if a.afterRollbackMark != nil && a.afterRollbackMark() {
			return nil
		}
		// (1) The rolled-back document, carrying this rollback's id so a later
		// holder can tell that it was published.
		if _, err := a.writeV2DocIf(ctx, room, oid, doc, v2Gen, coverage{clock: t, rollbackOp: opID}); err != nil {
			return lostLease(err)
		}
		if a.afterRollbackDocV2 != nil && a.afterRollbackDocV2() {
			return nil
		}
		// (2) checkpoint = target, while the mark still holds readers and the
		// append path off, so a failure here is settled rather than skipped.
		if _, err := a.store.putIfGeneration(ctx, a.layout.CheckpointName(room), be32(t), cpGen); err != nil {
			return lostLease(err)
		}
		if a.afterRollbackCheckpoint != nil && a.afterRollbackCheckpoint() {
			return nil
		}
		// (2b) announce the commit too. A flush that read the marks between (0a)
		// and (0b) saw no mark and the epoch already moved, so only a second
		// rewrite tells it this rollback committed.
		if err := a.store.put(ctx, a.rollbackEpochName(room), be32(t)); err != nil {
			return err
		}
		// (3) ceiling = target, clearing the mark. It hides any update above the
		// target that survives a stop before (4).
		ceilGen, err = a.store.putIfGeneration(ctx, a.ceilingName(room), be32(t), markGen)
		if err != nil {
			return lostLease(err)
		}
		// Conformance crash injection: return after the durable checkpoint+ceiling
		// but before deleting future updates.
		a.mu.Lock()
		crash := a.crashAfterCheckpoint
		a.mu.Unlock()
		if crash != nil && crash() {
			return nil
		}
		// (4) delete every update > target.
		return a.deleteUpdatesAfterFenced(ctx, room, oid, t, ceilGen)
	})
}

// lostLease reports a conditional write that lost to another holder, which
// only happens once this holder's lease has lapsed.
func lostLease(err error) error {
	if errors.Is(err, errGenerationMoved) {
		return fmt.Errorf("%w: another holder changed the room mid-rollback", errStateMoved)
	}
	return err
}

// newRollbackOp names one rollback.
func newRollbackOp() ([8]byte, string, error) {
	var b [8]byte
	if _, err := rand.Read(b[:]); err != nil {
		return b, "", err
	}
	return b, hex.EncodeToString(b[:]), nil
}

// deleteUpdatesAfterFenced deletes every update above target, each only at the
// generation listed, and only while the ceiling is the one this rollback wrote:
// an append clears it, after which a reused clock is someone's new edit.
func (a *Adapter) deleteUpdatesAfterFenced(ctx context.Context, room DocID, oid, target uint32, ceilGen int64) error {
	listed, err := a.listUpdatesAttrs(ctx, room, oid)
	if err != nil {
		return err
	}
	if g, err := a.store.generation(ctx, a.ceilingName(room)); err != nil {
		return err
	} else if g != ceilGen {
		return lostLease(errGenerationMoved)
	}
	for c, at := range listed {
		if c <= target {
			continue
		}
		if err := a.store.deleteIfGeneration(ctx, at.Name, at.Generation); err != nil {
			return lostLease(err)
		}
	}
	return nil
}

// settleRollback finishes or abandons a rollback whose writer stopped after
// marking the ceiling. The caller holds gcsDocLockKey, which PruneAfter holds for
// its whole run, so a mark seen here belongs to a writer that is gone.
func (a *Adapter) settleRollback(ctx context.Context, room DocID, oid uint32) error {
	// Each pass either settles the rollback or sees its writer publish doc_v2
	// meanwhile, after which the next pass finishes it.
	for attempt := 1; ; attempt++ {
		_, _, mark, rolling, err := a.ceilingState(ctx, room)
		if err != nil || !rolling {
			return err
		}
		v2Gen, meta, err := a.store.attrs(ctx, a.layout.DocV2Name(room))
		if err != nil {
			return err
		}
		// Holding the lock shows no rollback is running, not that the last one
		// finished. Equal coverage is no proof either, since a store snapshot can
		// already cover the target: only doc_v2 naming this rollback means it
		// published, so only then is it finished.
		if cov, ok := coverageOf(meta); ok && mark.op != "" && cov.rollbackOp == mark.op && cov.clock == mark.target {
			a.log.Warn("gcs: finishing a rollback whose writer stopped", "room", room, "target", mark.target)
			// Checkpoint first: the mark keeps the append path off until it is durable.
			if err := a.store.put(ctx, a.layout.CheckpointName(room), be32(mark.target)); err != nil {
				return err
			}
			// Committing it is a rollback a flush candidate must not outlive.
			if err := a.store.put(ctx, a.rollbackEpochName(room), be32(mark.target)); err != nil {
				return err
			}
			if err := a.store.put(ctx, a.ceilingName(room), be32(mark.target)); err != nil {
				return err
			}
			return a.deleteUpdatesAfter(ctx, room, oid, mark.target)
		}
		// Otherwise it stopped before replacing doc_v2 and took no effect so far.
		// Its writer may only have lost its lease, though, and can still resume:
		// its doc_v2 write is conditional on the generation it read before marking,
		// which clearing the mark leaves alone, so it would publish the rollback
		// with no mark. Moving doc_v2's generation first makes that write fail.
		err = a.fenceDocV2(ctx, room, oid, v2Gen, meta)
		if errors.Is(err, errGenerationMoved) && attempt < settleAttempts {
			continue // the writer published meanwhile
		}
		if err != nil {
			return err
		}
		a.log.Warn("gcs: abandoning a rollback that stopped before publishing", "room", room, "target", mark.target)
		if mark.prev == noCeiling {
			return a.store.delete(ctx, a.ceilingName(room))
		}
		return a.store.put(ctx, a.ceilingName(room), be32(mark.prev))
	}
}

// settleAttempts bounds how often settleRollback rereads a rollback whose
// writer keeps publishing while it is being abandoned.
const settleAttempts = 3

// fenceDocV2 rewrites doc_v2 as it is, at generation gen (0 for absent), so a
// writer holding an older generation can no longer replace it. It returns
// errGenerationMoved when doc_v2 changed since gen was read.
func (a *Adapter) fenceDocV2(ctx context.Context, room DocID, oid uint32, gen int64, meta map[string]string) error {
	name := a.layout.DocV2Name(room)
	if gen != 0 {
		b, err := a.store.get(ctx, name)
		if err == errNotFound {
			return errGenerationMoved
		}
		if err != nil {
			return err
		}
		// Bytes newer than gen fail the precondition, so this never republishes them.
		_, err = a.store.putIfGenerationMeta(ctx, name, b, gen, meta)
		return err
	}
	// Absent, the stopped writer's write is create-only, so create the doc_v2 a
	// reader already sees in its place: the v1 base at the checkpoint, or the
	// legacy root where a Phase-2 read would still fall back to it.
	cp, err := a.checkpoint(ctx, room)
	if err != nil {
		return err
	}
	doc := crdt.New()
	cov := coverage{clock: cp}
	if b, err := a.store.get(ctx, a.layout.DocStateName(room, oid)); err == nil {
		if err := crdt.ApplyUpdateV1(doc, b, nil); err != nil {
			return err
		}
	} else if err != errNotFound {
		return err
	} else if a.fallback != nil {
		updates, err := a.listUpdates(ctx, room, oid)
		if err != nil {
			return err
		}
		if len(updates) == 0 {
			legacy, err := a.materializeLegacy(ctx, room)
			if err != nil {
				return err
			}
			if len(legacy) > 0 {
				if err := crdt.ApplyUpdateV1(doc, legacy, nil); err != nil {
					return err
				}
				cov = coverage{} // as Backfill stamps it
			}
		}
	}
	_, err = a.writeV2DocIf(ctx, room, oid, doc, 0, cov)
	return err
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
		if err := a.settleRollback(ctx, room, oid); err != nil {
			return err
		}
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
		prev := st.marks.covers()
		cov := coverage{clock: max(prev.clock, cutoff), live: prev.live && cutoff <= prev.clock}
		baseGen, err := a.writeV2DocIf(ctx, room, oid, st.doc, st.v2Gen, cov)
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

// errRollbackRaced reports a rollback refused because another one committed
// after its version was rebuilt. Retrying rebuilds from the new state.
var errRollbackRaced error = rollbackRacedError{}

type rollbackRacedError struct{}

func (rollbackRacedError) Error() string {
	return "gcs: another rollback of this room committed while this one was prepared"
}
func (rollbackRacedError) Is(target error) bool     { return target == errStateMoved }
func (rollbackRacedError) RollbackInProgress() bool { return true }

// Rollback rebuilds version v and publishes it as the room's state, calling
// beforePublish (if set) once in between. It returns the rebuilt state.
//
// MaterializeAt then PruneAfter is not enough on its own: another rollback can
// commit between them, and publishing a version rebuilt before it would restore
// the edits that rollback removed and hide the ones made after it. The rollback
// epoch is read before rebuilding and PruneAfter announces itself only on that
// generation, so in that case this fails with a retryable error instead.
func (a *Adapter) Rollback(ctx context.Context, room string, v persistence.Version, beforePublish func()) ([]byte, error) {
	if err := a.validate(room); err != nil {
		return nil, err
	}
	oid, err := a.oidFor(ctx, room)
	if err != nil {
		return nil, err
	}
	// Settle a stopped rollback first: finishing one moves the epoch, which would
	// otherwise refuse this rollback for a commit that happened before it began.
	if _, err := a.settledMarks(ctx, room, oid); err != nil {
		return nil, err
	}
	epoch, err := a.store.generation(ctx, a.rollbackEpochName(room))
	if err != nil {
		return nil, err
	}
	rolled, err := a.MaterializeAt(ctx, room, v)
	if err != nil {
		return nil, err
	}
	if beforePublish != nil {
		beforePublish()
	}
	if err := a.pruneAfter(ctx, room, v, rolled, &epoch); err != nil {
		return nil, err
	}
	return rolled, nil
}
