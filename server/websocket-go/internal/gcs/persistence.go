package gcs

import (
	"context"
	"sort"

	"github.com/reearth/ygo/crdt"
	"github.com/reearth/ygo/persistence"
)

var (
	_ persistence.VersionedPersistence = (*Adapter)(nil)
	_ persistence.CrashInjector        = (*Adapter)(nil)
	_ persistence.Reopener             = (*Adapter)(nil)
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
	b, err := a.store.get(ctx, a.ceilingName(room))
	if err == errNotFound {
		return 0, false, nil
	}
	if err != nil {
		return 0, false, err
	}
	if len(b) < 4 {
		return 0, false, nil
	}
	return be32ToU32(b), true, nil
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

	// v2-first → v1 SUB_DOC fallback, primary layout only (the OID differs between
	// layouts, so a cross-layout v1 fallback is not byte-valid).
	var base []byte
	primaryHasV2 := false
	if v1, ok, err := a.loadPrimaryV2(ctx, room); err != nil {
		return persistence.LoadResult{}, err
	} else if ok {
		base = v1
		primaryHasV2 = true
	} else if b, err := a.store.get(ctx, a.layout.DocStateName(room, oid)); err == nil {
		base = b
	} else if err != errNotFound {
		return persistence.LoadResult{}, err
	}

	updates, err := a.listUpdates(ctx, room, oid)
	if err != nil {
		return persistence.LoadResult{}, err
	}

	// Phase-2 dual-read: when the primary prefix holds nothing, materialize the
	// legacy-root state for the same validated doc id. Never folds across layouts.
	if !primaryHasV2 && len(base) == 0 && len(updates) == 0 && a.fallback != nil {
		legacy, lerr := a.materializeLegacy(ctx, room)
		if lerr != nil {
			return persistence.LoadResult{}, lerr
		}
		if len(legacy) == 0 {
			return persistence.LoadResult{}, nil
		}
		return persistence.LoadResult{Update: legacy, Version: persistence.Version(legacyHeadClock(ctx, a, room))}, nil
	}
	cp, err := a.checkpoint(ctx, room)
	if err != nil {
		return persistence.LoadResult{}, err
	}
	ceil, hasCeil, err := a.ceiling(ctx, room)
	if err != nil {
		return persistence.LoadResult{}, err
	}

	clocks := sortedClocks(updates)
	// Drop any update > ceiling a crashed prune failed to delete; the v2 base
	// already reflects the rolled-back state.
	if hasCeil {
		kept := clocks[:0:0]
		for _, x := range clocks {
			if x <= ceil {
				kept = append(kept, x)
			}
		}
		clocks = kept
	}
	merged := base
	// Clamp the reported head to an active ceiling. After a crash between the
	// ceiling write and the checkpoint lowering in PruneAfter, the checkpoint can
	// still hold the stale pre-rollback value, but no version above the ceiling is
	// visible — so the head must never exceed it.
	head := cp
	if hasCeil && head > ceil {
		head = ceil
	}
	parts := [][]byte{}
	if len(merged) > 0 {
		parts = append(parts, merged)
	}
	for _, c := range clocks {
		if c <= cp {
			continue // already folded into the snapshot/base
		}
		b, err := a.store.get(ctx, updates[c])
		if err != nil {
			return persistence.LoadResult{}, err
		}
		parts = append(parts, b)
		if c > head {
			head = c
		}
	}
	if len(parts) == 0 {
		return persistence.LoadResult{Version: persistence.Version(head)}, nil
	}
	out, err := crdt.MergeUpdatesV1(parts...)
	if err != nil {
		return persistence.LoadResult{}, err
	}
	return persistence.LoadResult{Update: out, Version: persistence.Version(head)}, nil
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
func (a *Adapter) appendLocked(ctx context.Context, room DocID, oid uint32, update []byte) (uint32, error) {
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
	clock := last + 1
	if err := a.store.put(ctx, a.layout.UpdateName(room, oid, clock), update); err != nil {
		return 0, err
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
func (a *Adapter) FlushSnapshot(ctx context.Context, room string, state []byte) error {
	if err := a.validate(room); err != nil {
		return err
	}
	oid, err := a.oidFor(ctx, room)
	if err != nil {
		return err
	}
	return a.writeV2Snapshot(ctx, room, oid, state)
}

// SnapshotFromStore reconstructs the room's complete state from GCS (Load folds
// doc_v2 + tail updates) and rewrites it as a complete doc_v2. Used by the
// last-instance flush: ygo removes the live doc from GetDoc before firing
// RoomDeactivated, but it first drains the per-update persistence worker, so the
// full state is already in GCS and reconstructable without the live doc. No-op
// when nothing is persisted yet.
func (a *Adapter) SnapshotFromStore(ctx context.Context, room string) error {
	lr, err := a.Load(ctx, room)
	if err != nil {
		return err
	}
	if len(lr.Update) == 0 {
		return nil
	}
	return a.FlushSnapshot(ctx, room, lr.Update)
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

	cp, err := a.checkpoint(ctx, room)
	if err != nil {
		return nil, err
	}

	var parts [][]byte
	// Include the base snapshot when the checkpoint is at or below target. The
	// base is folded whenever a doc_v2 (or v1 SUB_DOC) exists, NOT only when
	// cp>0: a legacy-only/backfilled/flushed room has a real base at checkpoint 0,
	// and gating on cp>0 dropped it — materializing empty and letting a rollback
	// overwrite real content with an empty snapshot. CRDT merge is idempotent, so
	// folding the full base plus replayed updates never double-applies.
	if cp <= target {
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
	for _, c := range sortedClocks(updates) {
		if c > target {
			break
		}
		if cp <= target && c <= cp {
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
	return crdt.MergeUpdatesV1(parts...)
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
	doc := crdt.New()
	if err := crdt.ApplyUpdateV2(doc, v2, nil); err != nil {
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
	}
	for _, n := range names {
		if n == "" {
			continue
		}
		if err := a.store.delete(ctx, n); err != nil {
			return err
		}
	}
	for _, prefix := range []string{a.layout.UpdatePrefix(room, oid), a.snapshotPrefix(room)} {
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
	}
	for _, n := range names {
		if n == "" {
			continue
		}
		if err := a.store.delete(ctx, n); err != nil {
			return err
		}
	}
	for _, prefix := range []string{leg.UpdatePrefix(room, oid), legacySnapshotPrefix(room)} {
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
