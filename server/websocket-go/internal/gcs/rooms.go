package gcs

import (
	"context"
	"strings"
	"unicode/utf8"
)

// ListRooms enumerates every room the bucket holds data for.
//
// NOTE: not wired yet; admin cleanup still enumerates live rooms only. See
// reearth/reearth-flow#2333, which also lists two bugs to fix before wiring.
//
// Phase 2 is a prefix listing. Phase 1 recovers rooms from the OID index objects
// plus the snapshot counters, which cover snapshot-only rooms.
func (a *Adapter) ListRooms(ctx context.Context) ([]string, error) {
	if a.phase2 {
		prefixes, err := a.store.listPrefixes(ctx, "")
		if err != nil {
			return nil, err
		}
		out := make([]string, 0, len(prefixes))
		for _, p := range prefixes {
			out = append(out, strings.TrimSuffix(p, "/"))
		}
		return out, nil
	}

	seen := map[string]struct{}{}

	// Rooms with an update log: the OID index object encodes the room name.
	oidPrefix := hexb([]byte{rsV1, rsKeyspaceOID})
	names, err := a.store.list(ctx, oidPrefix)
	if err != nil {
		return nil, err
	}
	for _, n := range names {
		raw, derr := hexDecode(n)
		if derr != nil || len(raw) < 4 {
			continue
		}
		// Defensive guard against unrelated keyspaces (unreachable under the
		// "0000" scan prefix today, but kept in case the prefix is ever
		// widened) — mirrors listAllDocsPhase1's identical check.
		if raw[0] != rsV1 || raw[1] != rsKeyspaceOID {
			continue
		}
		// V1 ‖ KEYSPACE_OID ‖ utf8(room) ‖ 0x00
		body := raw[2 : len(raw)-1]
		if len(body) > 0 && utf8.Valid(body) {
			seen[string(body)] = struct{}{}
		}
	}

	// Rooms that have only snapshots still own a counter object.
	counters, err := a.store.list(ctx, hexb([]byte("snapnextid:")))
	if err != nil {
		return nil, err
	}
	for _, n := range counters {
		raw, derr := hexDecode(n)
		if derr != nil {
			continue
		}
		hexRoom, ok := strings.CutPrefix(string(raw), "snapnextid:")
		if !ok {
			continue
		}
		roomBytes, derr := hexDecode(hexRoom)
		if derr != nil {
			continue
		}
		seen[string(roomBytes)] = struct{}{}
	}

	out := make([]string, 0, len(seen))
	for r := range seen {
		out = append(out, r)
	}
	return out, nil
}
