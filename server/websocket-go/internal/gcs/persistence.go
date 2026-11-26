package gcs

import (
	"context"
	"errors"
	"fmt"
	"sort"

	"github.com/reearth/ygo/crdt"
	"github.com/reearth/ygo/persistence"
)

// ceilingName is the durable rollback ceiling. PruneAfter writes it (= target)
// before deleting future updates, so a crash mid-prune still hides any update >
// target from ListVersions/GetUpdate/Load. AppendUpdate clears it.
func (a *Adapter) ceilingName(room DocID) string {
	if a.phase2 {
		return ProjectPrefix(room) + "prune_ceiling"
	}
	return legacyCeilingName(room)
}

// legacyCeilingName / legacySnapshotPrefix are the Phase-1 (legacy-root) object
// names. They are also the fallback names swept by Delete during Phase-2
// coexistence, so they must stay byte-identical to the non-Phase-2 branches.
func legacyCeilingName(room DocID) string {
	return hexb([]byte("prune_ceiling:" + hexb([]byte(room))))
}

func legacySnapshotPrefix(room DocID) string {
	return hexb([]byte("snapshot:" + hexb([]byte(room)) + ":"))
}

// ceiling reads the rollback ceiling. ok=false means no ceiling is set.
func (a *Adapter) ceiling(ctx context.Context, room DocID) (uint32, bool, error) {
	c, ok, _, _, err := a.ceilingState(ctx, room)
	return c, ok, err
}

// noCeiling stands in for "no ceiling" inside a marked ceiling object. As a
// ceiling it hides nothing, so a reader that ignores the mark sees no change.
const noCeiling = ^uint32(0)

// rollMark is an in-progress rollback recorded on the ceiling object: the
// ceiling it replaced and the version being rolled back to.
type rollMark struct {
	prev, target uint32
}

// markedCeiling keeps prev in the first four bytes, which is all an older build
// reads, and appends the mark and the target.
func markedCeiling(prev, target uint32) []byte {
	b := make([]byte, 9)
	copy(b, be32(prev))
	b[4] = 1
	copy(b[5:], be32(target))
	return b
}

// ceilingState reads the ceiling and any in-progress rollback mark in one get.
func (a *Adapter) ceilingState(ctx context.Context, room DocID) (uint32, bool, rollMark, bool, error) {
	b, err := a.store.get(ctx, a.ceilingName(room))
	if err == errNotFound {
		return 0, false, rollMark{}, false, nil
	}
	if err != nil {
		return 0, false, rollMark{}, false, err
	}
	if len(b) < 4 {
		return 0, false, rollMark{}, false, nil
	}
	if len(b) >= 9 && b[4] == 1 {
		return be32ToU32(b), true, rollMark{prev: be32ToU32(b[:4]), target: be32ToU32(b[5:9])}, true, nil
	}
	return be32ToU32(b), true, rollMark{}, false, nil
}

// filterByCeiling drops any update clock strictly above an active rollback
// ceiling. Returns the clocks unchanged when no ceiling is set.
func (a *Adapter) filterByCeiling(ctx context.Context, room DocID, clocks []uint32) ([]uint32, error) {
	c, ok, err := a.ceiling(ctx, room)
	if err != nil || !ok {
		return clocks, err
	}
	out := clocks[:0:0]
	for _, x := range clocks {
		if x <= c {
			out = append(out, x)
		}
	}
	return out, nil
}

// snapshotName returns the named-snapshot object name. Named snapshots reuse the
// doc_v2 brotli(V2) format under a distinct name so they survive compaction/prune.
func (a *Adapter) snapshotName(room DocID, name string) string {
	if a.phase2 {
		return ProjectPrefix(room) + "snapshot:" + name
	}
	return hexb([]byte("snapshot:" + hexb([]byte(room)) + ":" + name))
}

// snapshotPrefix is the object-name prefix shared by all named snapshots of room,
// used by Delete to sweep caller-named snapshots the static-name list can't reach.
func (a *Adapter) snapshotPrefix(room DocID) string {
	if a.phase2 {
		return ProjectPrefix(room) + "snapshot:"
	}
	return legacySnapshotPrefix(room)
}

// loadAttempts bounds one round of Load's unlocked reads of a state other
// writers keep changing. Load tries a round, then the same read under
// gcsDocLockKey, then a further round if the lock was unavailable.
const loadAttempts = 4

// Load returns the latest merged state for room (v2-first → v1 fallback), folding
// any tail update objects on top.
//
// Errors are logged at ERROR here (with the room) because the WebSocket upgrade
// path discards the cause behind a bare "500 room unavailable": this is the only
// place the real reason a connect failed is recorded.
func (a *Adapter) Load(ctx context.Context, room string) (res persistence.LoadResult, err error) {
	defer func() {
		if err != nil {
			a.log.Error("gcs load failed", "room", room, "err", err)
		}
	}()
	if err := a.validate(room); err != nil {
		return persistence.LoadResult{}, err
	}
	oid, err := a.oidFor(ctx, room)
	if err != nil {
		return persistence.LoadResult{}, err
	}
	st, err := a.readStateRetrying(ctx, room, oid)
	if err != nil {
		return persistence.LoadResult{}, err
	}
	if st.empty {
		return persistence.LoadResult{Version: persistence.Version(st.head)}, nil
	}
	// Served anyway: the parked items stay in every later encode, so they are
	// kept until the edit they wait for arrives, whereas refusing withholds the
	// whole project.
	if ps := st.doc.PendingStats(); ps.Items > 0 {
		a.log.Error("gcs load: serving a document with parked items; an edit they build on is missing from storage",
			"room", room, "parked", ps.Items, "awaiting_clients", len(ps.MissingFor))
	}
	return persistence.LoadResult{Update: crdt.EncodeStateAsUpdateV1(st.doc, nil), Version: persistence.Version(st.head)}, nil
}

// readStateRetrying reads the stored state without the doc lock, retrying while
// a writer changes it underneath, and takes the lock for the last attempt.
//
// The lock is an optimisation for this read, never a correctness requirement:
// every unlocked attempt checks the marks around what it reads, so a read that
// returns at all is consistent. Load is the connect path, so when the lock is
// unavailable it must degrade to another unlocked read rather than fail the
// room open. A room being written hard is exactly when the lock is contended,
// and it is also exactly when someone is trying to open the room.
func (a *Adapter) readStateRetrying(ctx context.Context, room DocID, oid uint32) (storedState, error) {
	st, err := a.readStateUnlocked(ctx, room, oid)
	if !errors.Is(err, errStateMoved) {
		return st, err
	}

	// ran distinguishes "the lock was not acquired" from "the read under it
	// failed". Each Locker has its own timeout error, so the flag is the only
	// implementation-independent way to tell them apart.
	ran := false
	if lockErr := a.locker.WithLock(ctx, gcsDocLockKey(room), func(ctx context.Context) error {
		ran = true
		if err = a.settleRollback(ctx, room, oid); err != nil {
			return err
		}
		st, err = a.readState(ctx, room, oid)
		return err
	}); lockErr != nil && !ran {
		a.log.Warn("gcs load: the doc lock was unavailable, reading without it",
			"room", room, "err", lockErr)
	}
	if ran && !errors.Is(err, errStateMoved) {
		return st, err
	}

	// Either the lock never came free, or the state moved even under it (a lapsed
	// lease). Another unlocked round still has to come back clean, so this widens
	// the window Load tolerates without ever serving a state a writer moved.
	return a.readStateUnlocked(ctx, room, oid)
}

// readStateUnlocked reads the state without the doc lock, retrying while a
// writer keeps changing it underneath. The last attempt's result is returned,
// so the caller still sees errStateMoved when none of them came back clean.
func (a *Adapter) readStateUnlocked(ctx context.Context, room DocID, oid uint32) (storedState, error) {
	var st storedState
	var err error
	for attempt := 1; attempt < loadAttempts; attempt++ {
		if st, err = a.readState(ctx, room, oid); !errors.Is(err, errStateMoved) {
			break
		}
	}
	return st, err
}

// errStateMoved reports that a writer changed the room's stored state while it
// was being read without the doc lock.
var errStateMoved = errors.New("gcs: stored state changed while it was read")

// storedState is a room's stored state decoded into one document.
type storedState struct {
	doc  *crdt.Doc
	head uint32
	// empty means nothing is stored for the room.
	empty bool
	// v2Gen is the generation of the doc_v2 that doc was read from, 0 for none.
	v2Gen int64
	// updates are the listed update objects, and checkpointed the clocks among
	// them at or below the checkpoint. folded reports that doc already has those.
	updates      map[uint32]string
	checkpointed []uint32
	folded       bool
	// marks are the ones this read validated, so a caller that chose what to fold
	// or delete beforehand can check its choice still describes this state.
	marks marks
}

// marks are the checkpoint, the rollback ceiling and doc_v2's generation. Every
// writer that rewrites doc_v2, folds the update log or prunes it changes one of them.
type marks struct {
	cp      uint32
	ceil    uint32
	hasCeil bool
	// v2Gen catches a rollback whose later appends brought cp and the ceiling
	// back to the values read before it.
	v2Gen int64
	// cov is doc_v2's recorded coverage; hasCov is false for a doc_v2 written
	// before coverage was recorded.
	cov    coverage
	hasCov bool
	// rolling means a rollback has marked the ceiling and not yet cleared it.
	rolling bool
	mark    rollMark
}

// covers is what doc_v2 holds. Without recorded coverage it is taken to be the
// checkpoint, which is how every rebuild treated it before coverage existed.
func (m marks) covers() coverage {
	if m.hasCov {
		return m.cov
	}
	return coverage{clock: m.cp}
}

func (a *Adapter) readMarks(ctx context.Context, room DocID) (marks, error) {
	cp, err := a.checkpoint(ctx, room)
	if err != nil {
		return marks{}, err
	}
	ceil, hasCeil, mark, rolling, err := a.ceilingState(ctx, room)
	if err != nil {
		return marks{}, err
	}
	gen, meta, err := a.store.attrs(ctx, a.layout.DocV2Name(room))
	if err != nil {
		return marks{}, err
	}
	cov, hasCov := coverageOf(meta)
	return marks{cp: cp, ceil: ceil, hasCeil: hasCeil, v2Gen: gen, cov: cov, hasCov: hasCov, rolling: rolling, mark: mark}, nil
}

// readState reads doc_v2 (or the v1 base) and every update object above the
// checkpoint into one document, adding the checkpointed objects when items are
// left parked. Without gcsDocLockKey a concurrent writer can change what it
// reads, which it reports as errStateMoved.
func (a *Adapter) readState(ctx context.Context, room DocID, oid uint32) (storedState, error) {
	return a.readStateUpTo(ctx, room, oid, noClockLimit)
}

// noClockLimit reads every update object, which is what every caller but Compact
// wants: a checkpoint must cover the whole stored state.
const noClockLimit = ^uint32(0)

// readStateUpTo is readState bounded to the update objects at or below upTo.
// Compact passes its cutoff so the base it writes does not run ahead of the
// checkpoint it sets: an object above the checkpoint is one MaterializeAt still
// replays on top of the base, so folding it in would make every version above
// the checkpoint rebuild as the latest state. The base still starts from the
// stored doc_v2, so bounding the read can never shrink it.
func (a *Adapter) readStateUpTo(ctx context.Context, room DocID, oid uint32, upTo uint32) (storedState, error) {
	before, err := a.readMarks(ctx, room)
	if err != nil {
		return storedState{}, err
	}
	// A marked ceiling means doc_v2, the ceiling and the checkpoint may disagree
	// mid-rollback. Only a holder of the doc lock can settle that.
	if before.rolling {
		return storedState{}, errStateMoved
	}
	doc := crdt.New()
	hasBase := false
	if b, err := a.store.get(ctx, a.layout.DocV2Name(room)); err == nil {
		v2, err := decompressBrotli(b)
		if err != nil {
			return storedState{}, err
		}
		if err := crdt.ApplyUpdateV2(doc, v2, nil); err != nil {
			return storedState{}, err
		}
		hasBase = true
	} else if err != errNotFound {
		return storedState{}, err
	} else if b, err := a.store.get(ctx, a.layout.DocStateName(room, oid)); err == nil {
		// v1 SUB_DOC fallback, primary layout only: the OID differs between
		// layouts, so a cross-layout v1 fallback is not byte-valid.
		if err := crdt.ApplyUpdateV1(doc, b, nil); err != nil {
			return storedState{}, err
		}
		hasBase = len(b) > 0
	} else if err != errNotFound {
		return storedState{}, err
	}
	if a.afterReadBase != nil {
		a.afterReadBase()
	}
	updates, err := a.listUpdates(ctx, room, oid)
	if err != nil {
		return storedState{}, err
	}
	m, err := a.readMarks(ctx, room)
	if err != nil {
		return storedState{}, err
	}
	if m != before {
		return storedState{}, errStateMoved
	}
	if a.afterCheckMarks != nil {
		a.afterCheckMarks()
	}

	// Phase-2 dual-read: when the primary prefix holds nothing, materialize the
	// legacy-root state for the same validated doc id. Never folds across layouts.
	if !hasBase && len(updates) == 0 && a.fallback != nil {
		legacy, err := a.materializeLegacy(ctx, room)
		if err != nil || len(legacy) == 0 {
			return storedState{doc: doc, empty: true, v2Gen: before.v2Gen, marks: m}, err
		}
		if err := crdt.ApplyUpdateV1(doc, legacy, nil); err != nil {
			return storedState{}, err
		}
		return storedState{doc: doc, head: legacyHeadClock(ctx, a, room), v2Gen: before.v2Gen, marks: m}, nil
	}

	clocks := sortedClocks(updates)
	// Drop any update > ceiling a crashed prune failed to delete; the v2 base
	// already reflects the rolled-back state.
	if m.hasCeil {
		clocks = clocksUpTo(clocks, m.ceil)
	}
	clocks = clocksUpTo(clocks, upTo)
	// Clamp the reported head to an active ceiling. After a crash between the
	// ceiling write and the checkpoint lowering in PruneAfter, the checkpoint can
	// still hold the stale pre-rollback value, but no version above the ceiling is
	// visible — so the head must never exceed it.
	head := m.cp
	if m.hasCeil && head > m.ceil {
		head = m.ceil
	}
	readObjects := false
	for _, c := range clocks {
		if c <= m.cp {
			continue // already folded into the snapshot/base
		}
		if err := a.applyObject(ctx, doc, updates[c]); err != nil {
			return storedState{}, err
		}
		readObjects = true
		head = c
	}
	st := storedState{doc: doc, head: head, v2Gen: before.v2Gen, updates: updates, checkpointed: clocksUpTo(clocks, m.cp), marks: m}
	if !hasBase && !readObjects {
		st.empty = true
		return st, nil
	}
	folded, err := a.foldParked(ctx, room, doc, updates, st.checkpointed)
	if err != nil {
		return storedState{}, err
	}
	st.folded = folded
	// A rollback and appends after the marks check can replace a listed object
	// with a later edit, which only the marks show.
	if readObjects || folded {
		m, err := a.readMarks(ctx, room)
		if err != nil {
			return storedState{}, err
		}
		if m != before {
			return storedState{}, errStateMoved
		}
	}
	return st, nil
}

// applyObject applies the update object name to doc. An object deleted since it
// was listed means a compaction ran meanwhile, reported as errStateMoved.
func (a *Adapter) applyObject(ctx context.Context, doc *crdt.Doc, name string) error {
	b, err := a.store.get(ctx, name)
	if err == errNotFound {
		return errStateMoved
	}
	if err != nil {
		return err
	}
	return crdt.ApplyUpdateV1(doc, b, nil)
}

// writeUnion writes doc_v2 as the union of the stored state and candidate (a V1
// update, or nil), so doc_v2 never loses an item. The caller holds gcsDocLockKey.
//
// The write only lands on the doc_v2 it read, so a caller whose lock expired
// cannot replace a newer checkpoint. It reads the state again once, then fails.
func (a *Adapter) writeUnion(ctx context.Context, room DocID, oid uint32, candidate []byte) error {
	if err := a.settleRollback(ctx, room, oid); err != nil {
		return err
	}
	// A rollback committing after the candidate was built deliberately removed
	// edits the candidate still carries, and the union would restore them. doc_v2's
	// generation cannot tell a rollback from a competing checkpoint, so capture the
	// rollback marks here and recheck them before the candidate is applied, on the
	// retry as well as the first attempt.
	before, err := a.readMarks(ctx, room)
	if err != nil {
		return err
	}
	err = a.writeUnionOnce(ctx, room, oid, candidate, before)
	if errors.Is(err, errGenerationMoved) {
		err = a.writeUnionOnce(ctx, room, oid, candidate, before)
	}
	return err
}

// rollbackCommitted reports whether a rollback published between two marks
// reads. A ceiling being cleared is an ordinary append after a rollback the
// candidate already reflects, so that direction does not count.
func rollbackCommitted(before, now marks) bool {
	if !before.hasCeil && now.hasCeil {
		return true
	}
	if before.hasCeil && now.hasCeil && before.ceil != now.ceil {
		return true
	}
	return now.cp < before.cp
}

func (a *Adapter) writeUnionOnce(ctx context.Context, room DocID, oid uint32, candidate []byte, before marks) error {
	st, err := a.readState(ctx, room, oid)
	if err != nil {
		return err
	}
	if len(candidate) > 0 {
		now, err := a.readMarks(ctx, room)
		if err != nil {
			return err
		}
		if rollbackCommitted(before, now) {
			a.log.Warn("gcs: dropping a flush candidate built before a rollback",
				"room", room, "cp_before", before.cp, "cp_now", now.cp,
				"ceil_before", before.ceil, "ceil_now", now.ceil)
			return nil
		}
		if err := crdt.ApplyUpdateV1(st.doc, candidate, nil); err != nil {
			return err
		}
		// The candidate can park items on an edit that only a checkpointed object
		// holds, which readState did not fold when nothing was parked before.
		if !st.folded {
			if _, err := a.foldParked(ctx, room, st.doc, st.updates, st.checkpointed); err != nil {
				return err
			}
		}
	} else if st.empty {
		return nil
	}
	prev := st.marks.covers()
	cov := coverage{clock: max(prev.clock, st.head)}
	// A live candidate can carry edits no update object holds yet. A union from
	// the store alone that reaches past the earlier coverage has folded in what
	// raced that flush, so it clears the flag.
	cov.live = len(candidate) > 0 || (prev.live && cov.clock <= prev.clock)
	_, err = a.writeV2DocIf(ctx, room, oid, st.doc, st.v2Gen, cov)
	return err
}

// AppendUpdate persists one incremental V1 update at clock+1 and returns the
// assigned Version. Triggers every-10th-clock compaction. The read-modify-write
// (and inline compaction) run under the per-doc save lock so they are mutually
// exclusive with PruneAfter, Compact, and other AppendUpdate callers that share
// the same lock — otherwise two concurrent appends can assign the same clock and
// silently overwrite one update.
func (a *Adapter) AppendUpdate(ctx context.Context, room string, update []byte) (persistence.Version, error) {
	if err := a.validate(room); err != nil {
		return 0, err
	}
	oid, err := a.oidFor(ctx, room)
	if err != nil {
		return 0, err
	}
	var clock uint32
	err = a.locker.WithLock(ctx, gcsDocLockKey(room), func(ctx context.Context) error {
		c, e := a.appendLocked(ctx, room, oid, update)
		clock = c
		return e
	})
	if err != nil {
		return 0, err
	}
	return persistence.Version(clock), nil
}

// appendLocked is AppendUpdate's body; the caller holds gcsDocLockKey, so it uses
// the non-locking deleteUpdatesAfter (not finishInterruptedPrune, which re-takes
// the same lock and would deadlock a non-reentrant locker).
// appendClockAttempts bounds how many taken clocks appendLocked skips before
// giving up.
const appendClockAttempts = 5

func (a *Adapter) appendLocked(ctx context.Context, room DocID, oid uint32, update []byte) (uint32, error) {
	// The recovery below treats any ceiling as a finished prune and clears it,
	// which would drop an in-progress mark without settling it.
	if err := a.settleRollback(ctx, room, oid); err != nil {
		return 0, err
	}
	// Recovery path: if a ceiling is present, a prior PruneAfter may have crashed
	// mid-delete. Finish the interrupted prune, then clear the ceiling BEFORE
	// writing the new update below — so a crash after the write can never leave a
	// durable-but-ceiling-hidden update that the next recovery would delete.
	ceil, hasCeil, err := a.ceiling(ctx, room)
	if err != nil {
		return 0, err
	}
	if hasCeil {
		if err := a.deleteUpdatesAfter(ctx, room, oid, ceil); err != nil {
			return 0, err
		}
		// Safe to clear now: deleteUpdatesAfter removed every orphan > ceil,
		// so clearing the ceiling resurrects nothing.
		if err := a.store.delete(ctx, a.ceilingName(room)); err != nil {
			return 0, err
		}
	}

	last, err := a.lastClock(ctx, room, oid)
	if err != nil {
		return 0, err
	}
	// Phase-2: on the first primary write, fold the legacy-root base into the
	// primary prefix so the prefix is self-contained. Without this, an incremental
	// update would be the only primary object and a later Load would drop the
	// legacy base entirely (silent total data loss on the first edit of a
	// not-yet-backfilled room). No-op when a primary snapshot already exists or the
	// legacy root is empty.
	if a.phase2 && last == 0 {
		if err := a.Backfill(ctx, string(room)); err != nil {
			return 0, err
		}
		if last, err = a.lastClock(ctx, room, oid); err != nil {
			return 0, err
		}
	}
	// Create-only write: if our lock lapsed and another writer already took this
	// clock, re-read and move to the next one instead of overwriting its update.
	var clock uint32
	for attempt := 1; ; attempt++ {
		clock = last + 1
		if a.beforeUpdatePut != nil {
			a.beforeUpdatePut(clock)
		}
		err := a.store.putWithMetaIfAbsent(ctx, a.layout.UpdateName(room, oid, clock), update, nil)
		if err == nil {
			break
		}
		if !errors.Is(err, errObjectExists) || attempt == appendClockAttempts {
			return 0, err
		}
		if last, err = a.lastClock(ctx, room, oid); err != nil {
			return 0, err
		}
	}
	// Test-injection crash seam: simulate the process dying right after the
	// recovery update is durably written. The ceiling was already cleared above,
	// so the written update stays visible (at-least-once) rather than hidden.
	a.mu.Lock()
	crashW := a.crashAfterRecoveryWrite
	a.mu.Unlock()
	if crashW != nil && crashW() {
		return 0, errSimulatedRecoveryCrash
	}
	if clock%10 == 0 {
		if err := a.compactToCheckpoint(ctx, room, oid, clock); err != nil {
			return 0, err
		}
	}
	return clock, nil
}

// FlushSnapshot persists state as the COMPLETE doc_v2 snapshot (plus the v1 doc
// state and state vector), matching the Rust server's flush_doc_v2 contract: the
// doc_v2 object alone must reconstruct the full document, because the Rust reader
// (yrs load_doc_v2) folds no tail update objects. The last-instance flush uses
// this so a Rust instance can cold-load a Go-flushed room once the Redis stream
// is gone. state is the room's full current state as one V1 update.
//
// What is written is the union of state and the stored state, so a copy that
// missed an edit cannot remove it from doc_v2.
func (a *Adapter) FlushSnapshot(ctx context.Context, room string, state []byte) error {
	if err := a.validate(room); err != nil {
		return err
	}
	oid, err := a.oidFor(ctx, room)
	if err != nil {
		return err
	}
	return a.locker.WithLock(ctx, gcsDocLockKey(room), func(ctx context.Context) error {
		return a.writeUnion(ctx, room, oid, state)
	})
}

// SnapshotFromStore rewrites doc_v2 from the stored state alone (doc_v2 plus the
// update log). Used when no live copy is resident. No-op when nothing is
// persisted yet.
func (a *Adapter) SnapshotFromStore(ctx context.Context, room string) error {
	if err := a.validate(room); err != nil {
		return err
	}
	oid, err := a.oidFor(ctx, room)
	if err != nil {
		return err
	}
	return a.locker.WithLock(ctx, gcsDocLockKey(room), func(ctx context.Context) error {
		return a.writeUnion(ctx, room, oid, nil)
	})
}

// lastClock returns the highest update clock present (0 if none, falling back to
// the checkpoint when updates have been compacted away).
func (a *Adapter) lastClock(ctx context.Context, room DocID, oid uint32) (uint32, error) {
	updates, err := a.listUpdates(ctx, room, oid)
	if err != nil {
		return 0, err
	}
	cp, err := a.checkpoint(ctx, room)
	if err != nil {
		return 0, err
	}
	max := cp
	for c := range updates {
		if c > max {
			max = c
		}
	}
	// Cap at an active ceiling so the next AppendUpdate after a rollback writes at
	// target+1, not stale+1.
	if ceil, ok, err := a.ceiling(ctx, room); err != nil {
		return 0, err
	} else if ok && max > ceil {
		max = ceil
	}
	return max, nil
}

// ListVersions returns update metadata newest-first.
func (a *Adapter) ListVersions(ctx context.Context, room string) ([]persistence.VersionMeta, error) {
	if err := a.validate(room); err != nil {
		return nil, err
	}
	oid, err := a.oidFor(ctx, room)
	if err != nil {
		return nil, err
	}
	attrs, err := a.listUpdatesAttrs(ctx, room, oid)
	if err != nil {
		return nil, err
	}
	clocks := make([]uint32, 0, len(attrs))
	for c := range attrs {
		clocks = append(clocks, c)
	}
	// A clock above an active ceiling is logically gone (crash-safety).
	clocks, err = a.filterByCeiling(ctx, room, clocks)
	if err != nil {
		return nil, err
	}
	sort.Slice(clocks, func(i, j int) bool { return clocks[i] > clocks[j] }) // desc
	out := make([]persistence.VersionMeta, 0, len(clocks))
	for _, c := range clocks {
		out = append(out, persistence.VersionMeta{
			Version:   persistence.Version(c),
			UpdatedAt: attrs[c].Updated,
		})
	}
	return out, nil
}

// GetUpdate returns the single (non-cumulative) V1 update bytes at version v.
func (a *Adapter) GetUpdate(ctx context.Context, room string, v persistence.Version) ([]byte, persistence.VersionMeta, bool, error) {
	if err := a.validate(room); err != nil {
		return nil, persistence.VersionMeta{}, false, err
	}
	oid, err := a.oidFor(ctx, room)
	if err != nil {
		return nil, persistence.VersionMeta{}, false, err
	}
	attrs, err := a.listUpdatesAttrs(ctx, room, oid)
	if err != nil {
		return nil, persistence.VersionMeta{}, false, err
	}
	at, ok := attrs[uint32(v)]
	if !ok {
		return nil, persistence.VersionMeta{}, false, nil
	}
	// A version above an active ceiling is logically gone (crash-safety).
	if ceil, has, cerr := a.ceiling(ctx, room); cerr != nil {
		return nil, persistence.VersionMeta{}, false, cerr
	} else if has && uint32(v) > ceil {
		return nil, persistence.VersionMeta{}, false, nil
	}
	b, err := a.store.get(ctx, at.Name)
	if err != nil {
		return nil, persistence.VersionMeta{}, false, err
	}
	return b, persistence.VersionMeta{Version: v, UpdatedAt: at.Updated}, true, nil
}

// ErrVersionNotRebuildable reports that the update log no longer reaches back
// far enough to reconstruct the requested version, because a compaction folded
// the updates below it into a base that is itself past the version.
//
// Refusing matters because MaterializeAt's callers write the result somewhere
// durable: rollback replaces doc_v2 with it and deletes the log above the
// target, and a snapshot stores it. Returning a fragment of the document would
// destroy the history above the version while restoring nothing.
var ErrVersionNotRebuildable = errors.New("gcs: the update log was compacted past this version")

// notRebuildable carries ErrVersionNotRebuildable with the detail, and reports
// itself through a NotRebuildable method so a caller in another package can map
// it to a status without importing this one.
type notRebuildable struct{ detail string }

func (e *notRebuildable) Error() string        { return ErrVersionNotRebuildable.Error() + ": " + e.detail }
func (e *notRebuildable) Is(target error) bool { return target == ErrVersionNotRebuildable }
func (e *notRebuildable) NotRebuildable() bool { return true }

// ensureReplayComplete reports whether clocks (ascending) holds every update
// from 1 through target. Below the checkpoint MaterializeAt leaves the base out
// and replays update objects alone, so only an unbroken run from the first
// update rebuilds the whole state.
func ensureReplayComplete(clocks []uint32, target uint32) error {
	want := uint32(1)
	for _, c := range clocks {
		if c > target {
			break
		}
		if c != want {
			return &notRebuildable{detail: fmt.Sprintf(
				"version %d needs update %d, the oldest one retained is %d", target, want, c)}
		}
		want++
	}
	if want <= target {
		return &notRebuildable{detail: fmt.Sprintf(
			"version %d needs updates through %d, the log holds only %d", target, target, want-1)}
	}
	return nil
}

// MaterializeAt rebuilds the full state at version v as one V1 update. Read-only.
func (a *Adapter) MaterializeAt(ctx context.Context, room string, v persistence.Version) ([]byte, error) {
	if err := a.validate(room); err != nil {
		return nil, err
	}
	oid, err := a.oidFor(ctx, room)
	if err != nil {
		return nil, err
	}
	target := uint32(v)

	// Phase-2 dual-read: when the primary prefix holds nothing (no doc_v2, no v1
	// base, no updates) the state lives in the legacy root. Reconstruct it there,
	// mirroring Load — otherwise a legacy-only room materializes empty and a
	// rollback overwrites its real content with an empty snapshot.
	if a.fallback != nil {
		_, hasPrimaryV2, err := a.loadPrimaryV2(ctx, room)
		if err != nil {
			return nil, err
		}
		primaryUpdates, err := a.listUpdates(ctx, room, oid)
		if err != nil {
			return nil, err
		}
		var primaryBase []byte
		if !hasPrimaryV2 {
			if b, gerr := a.store.get(ctx, a.layout.DocStateName(room, oid)); gerr == nil {
				primaryBase = b
			} else if gerr != errNotFound {
				return nil, gerr
			}
		}
		if !hasPrimaryV2 && len(primaryBase) == 0 && len(primaryUpdates) == 0 {
			return a.materializeLegacy(ctx, room)
		}
	}

	m, err := a.readMarks(ctx, room)
	if err != nil {
		return nil, err
	}
	cp := m.cp
	// The base is usable for a version only if it holds nothing past it. The
	// checkpoint used to stand in for that, but a flush or snapshot from the store
	// rewrites doc_v2 without moving the checkpoint, so recorded coverage decides.
	cov := m.covers()
	base := cov.clock

	var parts [][]byte
	// Include the base snapshot when the checkpoint is at or below target. The
	// base is folded whenever a doc_v2 (or v1 SUB_DOC) exists, NOT only when
	// cp>0: a legacy-only/backfilled/flushed room has a real base at checkpoint 0,
	// and gating on cp>0 dropped it — materializing empty and letting a rollback
	// overwrite real content with an empty snapshot. CRDT merge is idempotent, so
	// folding the full base plus replayed updates never double-applies.
	if base <= target {
		if v1, ok, err := a.loadV2(ctx, room); err != nil {
			return nil, err
		} else if ok {
			parts = append(parts, v1)
		} else {
			if b, err := a.store.get(ctx, a.layout.DocStateName(room, oid)); err == nil {
				parts = append(parts, b)
			} else if err != errNotFound {
				return nil, err
			}
		}
	}
	// else target < checkpoint: base is past target, replay updates 0..target only.

	updates, err := a.listUpdates(ctx, room, oid)
	if err != nil {
		return nil, err
	}
	clocks := sortedClocks(updates)
	// A live base can hold edits that raced its flush and only reached the log at
	// later clocks, so used for a version below the head it can run past it.
	if base <= target && cov.live {
		head := base
		visible := clocks
		if m.hasCeil {
			visible = clocksUpTo(visible, m.ceil)
		}
		if n := len(visible); n > 0 && visible[n-1] > head {
			head = visible[n-1]
		}
		if target < head {
			err := &notRebuildable{detail: fmt.Sprintf(
				"version %d is below the head %d, and the base was flushed from a live room that can hold later edits", target, head)}
			a.log.Warn("gcs materialize: refusing a version a live base can overtake",
				"room", room, "version", target, "covers", base, "head", head)
			return nil, err
		}
	}
	// The base was left out above, so the update objects are the only source for
	// this version. A compaction that folded the oldest ones into a base already
	// past target leaves a log that cannot rebuild it: refuse rather than hand a
	// fragment to a caller that will write it over doc_v2.
	if base > target {
		if err := ensureReplayComplete(clocks, target); err != nil {
			a.log.Warn("gcs materialize: refusing a version the update log can no longer rebuild",
				"room", room, "version", target, "checkpoint", cp, "err", err)
			return nil, err
		}
		// A contiguous log from clock 1 rebuilds the whole document only when the
		// document started empty. Backfill copies the legacy root into doc_v2 and
		// writes no update objects, so that content rides in the excluded base alone.
		imported, err := a.hasImportedBase(ctx, room)
		if err != nil {
			return nil, err
		}
		if imported {
			rerr := &notRebuildable{detail: fmt.Sprintf(
				"version %d is below what the base covers (%d), and this room's base was imported from the legacy layout, which no update object carries",
				target, base)}
			a.log.Warn("gcs materialize: refusing a version below an imported base",
				"room", room, "version", target, "checkpoint", cp)
			return nil, rerr
		}
	}
	for _, c := range clocks {
		if c > target {
			break
		}
		if base <= target && c <= base {
			continue // already in the base snapshot
		}
		b, err := a.store.get(ctx, updates[c])
		if err != nil {
			return nil, err
		}
		parts = append(parts, b)
	}
	if len(parts) == 0 {
		return nil, nil // empty state
	}
	doc := crdt.New()
	for _, p := range parts {
		if err := crdt.ApplyUpdateV1(doc, p, nil); err != nil {
			return nil, err
		}
	}
	var checkpointed []uint32
	if base <= target {
		checkpointed = clocksUpTo(clocks, base)
	}
	if _, err := a.foldParked(ctx, room, doc, updates, checkpointed); err != nil {
		return nil, err
	}
	if ps := doc.PendingStats(); ps.Items > 0 {
		a.log.Debug("gcs materialize: state holds parked items; an edit they build on is missing from storage",
			"room", room, "version", target, "parked", ps.Items, "awaiting_clients", len(ps.MissingFor))
	}
	return crdt.EncodeStateAsUpdateV1(doc, nil), nil
}

// CaptureSnapshot stores a named snapshot of state and returns the current head
// version.
func (a *Adapter) CaptureSnapshot(ctx context.Context, room, name string, state []byte) (persistence.Version, error) {
	if err := a.validate(room); err != nil {
		return 0, err
	}
	oid, err := a.oidFor(ctx, room)
	if err != nil {
		return 0, err
	}
	doc, err := docFromV1(state)
	if err != nil {
		return 0, err
	}
	v2 := crdt.EncodeStateAsUpdateV2(doc, nil)
	if err := a.store.put(ctx, a.snapshotName(room, name), compressBrotli(v2)); err != nil {
		return 0, err
	}
	head, err := a.lastClock(ctx, room, oid)
	if err != nil {
		return 0, err
	}
	return persistence.Version(head), nil
}

// RestoreSnapshot returns the V1 update for the named snapshot.
func (a *Adapter) RestoreSnapshot(ctx context.Context, room, name string) ([]byte, persistence.Version, bool, error) {
	if err := a.validate(room); err != nil {
		return nil, 0, false, err
	}
	oid, err := a.oidFor(ctx, room)
	if err != nil {
		return nil, 0, false, err
	}
	b, err := a.store.get(ctx, a.snapshotName(room, name))
	if err == errNotFound {
		return nil, 0, false, nil
	}
	if err != nil {
		return nil, 0, false, err
	}
	v2, err := decompressBrotli(b)
	if err != nil {
		return nil, 0, false, err
	}
	doc, err := docFromV2(v2)
	if err != nil {
		return nil, 0, false, err
	}
	head, err := a.lastClock(ctx, room, oid)
	if err != nil {
		return nil, 0, false, err
	}
	return crdt.EncodeStateAsUpdateV1(doc, nil), persistence.Version(head), true, nil
}

// Delete removes all persisted state for room — snapshot, checkpoint, v1 base/SV,
// update log, OID index, ceiling, and every named snapshot. Idempotent and
// project-scoped.
func (a *Adapter) Delete(ctx context.Context, room string) error {
	if err := a.validate(room); err != nil {
		return err
	}

	// Phase 2: everything lives under "{room}/", so one prefix sweep removes the
	// primary layout. But the same doc id may still carry legacy-root objects
	// (pre-migration data, or writes by a coexisting Rust node) that the prefix
	// sweep cannot reach; leaving them behind resurrects the "deleted" room via
	// dual-read on the next Load. Sweep the legacy root too.
	if a.phase2 {
		names, err := a.store.list(ctx, ProjectPrefix(room))
		if err != nil {
			return err
		}
		for _, n := range names {
			if err := a.store.delete(ctx, n); err != nil {
				return err
			}
		}
		if a.fallback != nil {
			if err := a.deleteLegacyRoot(ctx, room); err != nil {
				return err
			}
		}
		a.mu.Lock()
		delete(a.oidCache, room)
		a.mu.Unlock()
		return nil
	}

	// Phase 1: delete the fixed-name objects, then sweep the update log and named
	// snapshots by prefix.
	oid, err := a.oidFor(ctx, room)
	if err != nil {
		return err
	}
	names := []string{
		a.layout.DocV2Name(room),
		a.layout.CheckpointName(room),
		a.layout.DocStateName(room, oid),
		a.layout.StateVectorName(room, oid),
		a.layout.OIDIndexName(room),
		a.ceilingName(room),
		// NB: SnapNextIDName is deliberately NOT here — see the deferred delete
		// after the prefix sweep below.
	}
	for _, n := range names {
		if n == "" {
			continue
		}
		if err := a.store.delete(ctx, n); err != nil {
			return err
		}
	}
	// Snapshot objects (SnapshotStore) and the id counter are room data too: the
	// documented contract is "removes all data for room".
	for _, prefix := range []string{a.layout.UpdatePrefix(room, oid), a.snapshotPrefix(room), a.layout.SnapVersionPrefix(room)} {
		objs, err := a.store.list(ctx, prefix)
		if err != nil {
			return err
		}
		for _, n := range objs {
			if err := a.store.delete(ctx, n); err != nil {
				return err
			}
		}
	}
	// The id counter goes LAST, after the snapshots it numbers. These deletes are
	// sequential and unlocked, so a cancelled context can stop this loop partway;
	// removing the counter first would leave live snapshots with no counter, and
	// the next SaveSnapshot would then reuse ids and overwrite them. Ordering it
	// last means a partial delete leaks a counter (harmless) instead.
	if n := a.layout.SnapNextIDName(room); n != "" {
		if err := a.store.delete(ctx, n); err != nil {
			return err
		}
	}
	a.mu.Lock()
	delete(a.oidCache, room)
	a.mu.Unlock()
	return nil
}

// deleteLegacyRoot removes every legacy-root object for room (Phase-2 only),
// mirroring the Phase-1 deletion: the fixed-name objects, the legacy prune
// ceiling, and the update-log + named-snapshot prefixes under the legacy OID.
func (a *Adapter) deleteLegacyRoot(ctx context.Context, room DocID) error {
	leg := a.fallback
	oid, err := a.legacyOID(ctx, room)
	if err != nil {
		return err
	}
	names := []string{
		leg.DocV2Name(room),
		leg.CheckpointName(room),
		leg.DocStateName(room, oid),
		leg.StateVectorName(room, oid),
		leg.OIDIndexName(room),
		legacyCeilingName(room),
		// SnapNextIDName deferred to after the sweep, as in the Phase-1 path.
	}
	for _, n := range names {
		if n == "" {
			continue
		}
		if err := a.store.delete(ctx, n); err != nil {
			return err
		}
	}
	for _, prefix := range []string{leg.UpdatePrefix(room, oid), legacySnapshotPrefix(room), leg.SnapVersionPrefix(room)} {
		objs, err := a.store.list(ctx, prefix)
		if err != nil {
			return err
		}
		for _, n := range objs {
			if err := a.store.delete(ctx, n); err != nil {
				return err
			}
		}
	}
	// Counter last: same reasoning as the Phase-1 path — a cancelled delete must
	// not strand live snapshots without the counter that keeps their ids unique.
	if n := leg.SnapNextIDName(room); n != "" {
		if err := a.store.delete(ctx, n); err != nil {
			return err
		}
	}
	return nil
}

// sortedClocks returns the update clocks in ascending order.
func sortedClocks[V any](m map[uint32]V) []uint32 {
	cs := make([]uint32, 0, len(m))
	for c := range m {
		cs = append(cs, c)
	}
	sort.Slice(cs, func(i, j int) bool { return cs[i] < cs[j] })
	return cs
}
