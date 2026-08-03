package gcs

import (
	"context"
	"errors"
	"sort"
	"strconv"
	"strings"

	"github.com/reearth/ygo/persistence"
)

// Custom-metadata keys on a snapver object. Label and the exact state length are
// stored here so ListSnapshots is a single listAttrs call that never reads a
// payload. attrs.Size is the brotli-COMPRESSED size and is deliberately unused
// for SnapshotInfo.Size.
const (
	snapMetaLabel = "ygo-label"
	snapMetaSize  = "ygo-size"
)

// SaveSnapshot stores state as a new labelled snapshot and returns its id.
//
// state is treated as an opaque, self-contained blob — exactly like the ygo
// memory and file backends treat it — and is never decoded as a CRDT update. It
// is only brotli-compressed for storage; GetSnapshotState decompresses and
// returns it byte-for-byte. This matters because SnapshotStore's contract (and
// its conformance suite) allows any non-empty byte string as state, not only a
// valid Yjs update.
func (a *Adapter) SaveSnapshot(ctx context.Context, room, label string, state []byte) (int64, error) {
	if err := a.validate(room); err != nil {
		return 0, err
	}
	if len(state) == 0 {
		return 0, persistence.ErrEmptySnapshot
	}
	d := DocID(room)
	compressed := compressBrotli(state)

	meta := map[string]string{
		snapMetaLabel: label,
		snapMetaSize:  strconv.Itoa(len(state)),
	}

	var id int64
	err := a.locker.WithLock(ctx, gcsDocLockKey(d), func(ctx context.Context) error {
		next, lerr := a.nextSnapID(ctx, d)
		if lerr != nil {
			return lerr
		}
		for attempt := 1; ; attempt++ {
			// Bump the counter BEFORE writing the record: a crash in between leaks an
			// id (harmless) rather than risking two snapshots sharing one.
			if perr := a.store.put(ctx, a.layout.SnapNextIDName(d), []byte(strconv.FormatInt(next+1, 10))); perr != nil {
				return perr
			}
			perr := a.store.putWithMetaIfAbsent(ctx, a.layout.SnapVersionName(d, next), compressed, meta)
			if perr == nil {
				id = next
				return nil
			}
			if !errors.Is(perr, errObjectExists) || attempt >= maxSnapshotIDAttempts {
				return perr
			}
			// The id was already taken, so the counter was behind the objects that
			// actually exist — a lock lease that expired mid-allocation, or a counter
			// restored from an older state. Re-derive the floor from reality rather
			// than trusting the counter again, and retry. The write-once precondition
			// is what makes this safe: the losing writer never destroys the winner.
			a.log.Warn("snapshot id already taken, re-deriving from stored objects",
				"room", string(d), "id", next, "attempt", attempt)
			maxID, ferr := a.maxSnapIDFromObjects(ctx, d)
			if ferr != nil {
				return ferr
			}
			next = maxID + 1
		}
	})
	if err != nil {
		return 0, err
	}
	return id, nil
}

// maxSnapshotIDAttempts bounds SaveSnapshot's retry when an allocated id turns
// out to be taken. Each retry re-derives the floor from stored objects, so more
// than a couple of rounds means something is badly wrong rather than racing.
const maxSnapshotIDAttempts = 3

// nextSnapID returns the next snapshot id for the room.
//
// The counter object is a fast path, NOT the source of truth. When it is missing
// or unparseable this falls back to deriving the floor from the snapshot objects
// that exist. Trusting the counter alone is a data-loss bug: SaveSnapshot would
// restart at id 1 and overwrite live snapshot 1, destroying its payload, label
// and timestamp. That is routinely reachable, because Delete removes objects one
// at a time and a cancelled request can take the counter while snapshots remain.
//
// Caller holds gcsDocLockKey(d), so this read-modify-write is race-free against
// concurrent SaveSnapshot calls for the same room.
func (a *Adapter) nextSnapID(ctx context.Context, d DocID) (int64, error) {
	b, err := a.store.get(ctx, a.layout.SnapNextIDName(d))
	if err != nil && err != errNotFound {
		return 0, err
	}
	if err == nil {
		if id, perr := strconv.ParseInt(strings.TrimSpace(string(b)), 10, 64); perr == nil && id >= 1 {
			return id, nil
		}
		a.log.Warn("snapshot id counter is unparseable; deriving from stored objects",
			"room", string(d))
	}
	maxID, err := a.maxSnapIDFromObjects(ctx, d)
	if err != nil {
		return 0, err
	}
	return maxID + 1, nil
}

// maxSnapIDFromObjects returns the highest snapshot id actually present for the
// room, or 0 when there are none. Both layouts are scanned when a fallback is
// configured (Phase 2), so a Phase-2 allocation can never collide with a
// legacy-root id that ListSnapshots still surfaces.
func (a *Adapter) maxSnapIDFromObjects(ctx context.Context, d DocID) (int64, error) {
	var maxID int64
	for _, l := range a.snapshotLayouts() {
		attrs, err := a.store.listAttrs(ctx, l.SnapVersionPrefix(d))
		if err != nil {
			return 0, err
		}
		for _, at := range attrs {
			if id, ok := l.SnapVersionIDFromName(at.Name); ok && id > maxID {
				maxID = id
			}
		}
	}
	return maxID, nil
}

// snapshotLayouts returns the layouts whose snapshot objects are readable: the
// primary, plus the legacy root when running Phase 2 so pre-cutover snapshots
// stay visible. Order matters — the primary comes first so it wins de-duplication.
func (a *Adapter) snapshotLayouts() []Layout {
	if a.fallback == nil {
		return []Layout{a.layout}
	}
	return []Layout{a.layout, a.fallback}
}

// ListSnapshots returns snapshot metadata newest-first. It reads only object
// attributes (name + custom metadata), never a state blob.
//
// Under Phase 2 this merges the primary prefix with the legacy root, because a
// room written before the cutover keeps its snapshots there. Without the merge,
// flipping REEARTH_FLOW_GCS_PHASE2 would empty the version history of every
// existing room while document state kept loading fine via dual-read — silent,
// error-free, and indistinguishable from data loss to a user. Every other
// layout-scoped read here already has a legacy fallback; so does Delete.
func (a *Adapter) ListSnapshots(ctx context.Context, room string) ([]persistence.SnapshotInfo, error) {
	if err := a.validate(room); err != nil {
		return nil, err
	}
	d := DocID(room)
	// seen de-duplicates by id across layouts. The primary layout is scanned
	// first, so a room that already re-saved a snapshot post-cutover shows the
	// primary copy and the legacy one is ignored.
	seen := make(map[int64]struct{})
	out := make([]persistence.SnapshotInfo, 0)
	for _, l := range a.snapshotLayouts() {
		attrs, err := a.store.listAttrs(ctx, l.SnapVersionPrefix(d))
		if err != nil {
			return nil, err
		}
		for _, at := range attrs {
			// Parse with the layout that produced the name — the two encode ids
			// differently, so crossing them would yield wrong or missing ids.
			id, ok := l.SnapVersionIDFromName(at.Name)
			if !ok {
				continue
			}
			if _, dup := seen[id]; dup {
				continue
			}
			seen[id] = struct{}{}
			size, _ := strconv.ParseInt(at.Metadata[snapMetaSize], 10, 64)
			out = append(out, persistence.SnapshotInfo{
				ID:        id,
				Label:     at.Metadata[snapMetaLabel],
				CreatedAt: at.Created.UTC(),
				Size:      size,
			})
		}
	}
	sort.Slice(out, func(i, j int) bool { return out[i].ID > out[j].ID }) // newest first
	return out, nil
}

// GetSnapshotState returns the snapshot's state exactly as it was saved. Falls
// back to the legacy root under Phase 2 so pre-cutover snapshots stay readable —
// otherwise every id ListSnapshots reports would 404.
func (a *Adapter) GetSnapshotState(ctx context.Context, room string, id int64) ([]byte, error) {
	if err := a.validate(room); err != nil {
		return nil, err
	}
	d := DocID(room)
	for _, l := range a.snapshotLayouts() {
		b, err := a.store.get(ctx, l.SnapVersionName(d, id))
		if err == nil {
			return decompressBrotli(b)
		}
		if err != errNotFound {
			return nil, err
		}
	}
	return nil, persistence.ErrSnapshotNotFound
}

// DeleteSnapshot removes one snapshot. Deleting an unknown snapshot is a no-op.
// Both layouts are swept under Phase 2, so deleting a snapshot the merged list
// reported cannot leave a legacy copy behind that reappears on the next list.
func (a *Adapter) DeleteSnapshot(ctx context.Context, room string, id int64) error {
	if err := a.validate(room); err != nil {
		return err
	}
	d := DocID(room)
	for _, l := range a.snapshotLayouts() {
		if err := a.store.delete(ctx, l.SnapVersionName(d, id)); err != nil && err != errNotFound {
			return err
		}
	}
	return nil
}

var _ persistence.SnapshotStore = (*Adapter)(nil)
