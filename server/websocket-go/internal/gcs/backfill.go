package gcs

import (
	"bytes"
	"context"
	"fmt"

	"github.com/reearth/ygo/crdt"
)

// Backfill migrates a room's legacy-root state into the Phase-2 {projectId}/
// prefix and verifies state_vector(new) == state_vector(legacy) before the prefix
// becomes authoritative, erroring on mismatch. It is idempotent and crash-safe
// (writes only under the primary prefix, never mutating the legacy root), and
// project-scoped. No-op in Phase 1. It does not delete the legacy objects.
func (a *Adapter) Backfill(ctx context.Context, room string) error {
	if !a.phase2 || a.fallback == nil {
		return nil
	}
	if err := ValidateDocIDForPrefix(room); err != nil {
		return err
	}

	// If the primary already has a snapshot, backfill already happened.
	if _, err := a.store.get(ctx, a.layout.DocV2Name(room)); err == nil {
		return nil
	} else if err != errNotFound {
		return err
	}

	legacyV1, err := a.materializeLegacy(ctx, room)
	if err != nil {
		return err
	}
	if legacyV1 == nil {
		return nil // nothing in the legacy root; new room
	}

	oid := FolderOID
	if err := a.writeV2Snapshot(ctx, room, oid, legacyV1); err != nil {
		return err
	}

	// Verify state-vector equality before the prefix is trusted.
	newV1, _, err := a.loadPrimaryV2(ctx, room)
	if err != nil {
		return err
	}
	if !bytes.Equal(stateVectorOf(legacyV1), stateVectorOf(newV1)) {
		return fmt.Errorf("gcs: backfill state-vector mismatch for room (refusing to switch)")
	}
	return nil
}

// materializeLegacy rebuilds the full legacy-root V1 state for room (doc_v2-first,
// then SUB_DOC base + replayed updates).
func (a *Adapter) materializeLegacy(ctx context.Context, room string) ([]byte, error) {
	leg := a.fallback
	if b, err := a.store.get(ctx, leg.DocV2Name(room)); err == nil {
		v2, derr := decompressBrotli(b)
		if derr != nil {
			return nil, derr
		}
		doc := crdt.New()
		if aerr := crdt.ApplyUpdateV2(doc, v2, nil); aerr != nil {
			return nil, aerr
		}
		base := crdt.EncodeStateAsUpdateV1(doc, nil)
		return a.foldLegacyUpdates(ctx, room, base)
	} else if err != errNotFound {
		return nil, err
	}
	// v1 fallback: SUB_DOC base under the legacy OID.
	oid, err := a.legacyOID(ctx, room)
	if err != nil {
		return nil, err
	}
	var base []byte
	if b, err := a.store.get(ctx, leg.DocStateName(room, oid)); err == nil {
		base = b
	} else if err != errNotFound {
		return nil, err
	}
	return a.foldLegacyUpdatesOID(ctx, room, oid, base)
}

// legacyOID reads the legacy-root OID index for room (0 if absent).
func (a *Adapter) legacyOID(ctx context.Context, room string) (uint32, error) {
	b, err := a.store.get(ctx, a.fallback.OIDIndexName(room))
	if err == errNotFound {
		return 0, nil
	}
	if err != nil {
		return 0, err
	}
	if len(b) < 4 {
		return 0, nil
	}
	return be32ToU32(b), nil
}

// foldLegacyUpdates folds legacy update objects (under the legacy OID) onto base.
func (a *Adapter) foldLegacyUpdates(ctx context.Context, room string, base []byte) ([]byte, error) {
	oid, err := a.legacyOID(ctx, room)
	if err != nil {
		return nil, err
	}
	return a.foldLegacyUpdatesOID(ctx, room, oid, base)
}

func (a *Adapter) foldLegacyUpdatesOID(ctx context.Context, room string, oid uint32, base []byte) ([]byte, error) {
	prefix := a.fallback.UpdatePrefix(room, oid)
	names, err := a.store.list(ctx, prefix)
	if err != nil {
		return nil, err
	}
	byClock := map[uint32]string{}
	for _, n := range names {
		if c, ok := updateClock(n); ok {
			byClock[c] = n
		}
	}
	parts := [][]byte{}
	if len(base) > 0 {
		parts = append(parts, base)
	}
	for _, c := range sortedClocks(byClock) {
		b, err := a.store.get(ctx, byClock[c])
		if err != nil {
			return nil, err
		}
		parts = append(parts, b)
	}
	if len(parts) == 0 {
		return nil, nil
	}
	return crdt.MergeUpdatesV1(parts...)
}

// loadPrimaryV2 reads + decodes the primary (prefix) doc_v2 only, no fallback.
func (a *Adapter) loadPrimaryV2(ctx context.Context, room string) ([]byte, bool, error) {
	b, err := a.store.get(ctx, a.layout.DocV2Name(room))
	if err == errNotFound {
		return nil, false, nil
	}
	if err != nil {
		return nil, false, err
	}
	v2, err := decompressBrotli(b)
	if err != nil {
		return nil, false, err
	}
	doc := crdt.New()
	if err := crdt.ApplyUpdateV2(doc, v2, nil); err != nil {
		return nil, false, err
	}
	return crdt.EncodeStateAsUpdateV1(doc, nil), true, nil
}

// legacyHeadClock returns the highest legacy-root update clock for room, used as
// the Version of a dual-read Load result. Best-effort: a transient error degrades
// the reported Version to 0 (with a WARN) rather than failing the Load.
func legacyHeadClock(ctx context.Context, a *Adapter, room string) uint32 {
	oid, err := a.legacyOID(ctx, room)
	if err != nil {
		a.warn("dual-read legacy head: OID index read failed; reporting Version=0", room, err)
		return 0
	}
	names, err := a.store.list(ctx, a.fallback.UpdatePrefix(room, oid))
	if err != nil {
		a.warn("dual-read legacy head: update-prefix list failed; reporting Version=0", room, err)
		return 0
	}
	var max uint32
	for _, n := range names {
		if c, ok := updateClock(n); ok && c > max {
			max = c
		}
	}
	return max
}

// warn logs an operational warning. Nil-safe, and never logs payloads.
func (a *Adapter) warn(msg, room string, err error) {
	if a.log == nil {
		return
	}
	a.log.Warn(msg, "room", room, "err", err)
}

// stateVectorOf returns the lib0-v1 state vector of a V1 update (empty for nil).
func stateVectorOf(v1 []byte) []byte {
	doc := crdt.New()
	if len(v1) > 0 {
		_ = crdt.ApplyUpdateV1(doc, v1, nil)
	}
	return crdt.EncodeStateVectorV1(doc)
}
