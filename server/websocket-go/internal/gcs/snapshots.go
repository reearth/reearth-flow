package gcs

import (
	"context"
	"sort"
	"strconv"

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

	var id int64
	err := a.locker.WithLock(ctx, gcsDocLockKey(d), func(ctx context.Context) error {
		next, lerr := a.nextSnapID(ctx, d)
		if lerr != nil {
			return lerr
		}
		// Bump the counter BEFORE writing the record: a crash in between leaks an
		// id (harmless) rather than risking two snapshots sharing one.
		if perr := a.store.put(ctx, a.layout.SnapNextIDName(d), []byte(strconv.FormatInt(next+1, 10))); perr != nil {
			return perr
		}
		meta := map[string]string{
			snapMetaLabel: label,
			snapMetaSize:  strconv.Itoa(len(state)),
		}
		if perr := a.store.putWithMeta(ctx, a.layout.SnapVersionName(d, next), compressed, meta); perr != nil {
			return perr
		}
		id = next
		return nil
	})
	if err != nil {
		return 0, err
	}
	return id, nil
}

// nextSnapID reads the room's snapshot id counter (1 when unset). Caller holds
// gcsDocLockKey(d), so this read-modify-write is race-free against concurrent
// SaveSnapshot calls for the same room.
func (a *Adapter) nextSnapID(ctx context.Context, d DocID) (int64, error) {
	b, err := a.store.get(ctx, a.layout.SnapNextIDName(d))
	if err == errNotFound {
		return 1, nil
	}
	if err != nil {
		return 0, err
	}
	id, perr := strconv.ParseInt(string(b), 10, 64)
	if perr != nil || id < 1 {
		return 1, nil
	}
	return id, nil
}

// ListSnapshots returns snapshot metadata newest-first. It reads only object
// attributes (name + custom metadata), never a state blob.
func (a *Adapter) ListSnapshots(ctx context.Context, room string) ([]persistence.SnapshotInfo, error) {
	if err := a.validate(room); err != nil {
		return nil, err
	}
	d := DocID(room)
	attrs, err := a.store.listAttrs(ctx, a.layout.SnapVersionPrefix(d))
	if err != nil {
		return nil, err
	}
	out := make([]persistence.SnapshotInfo, 0, len(attrs))
	for _, at := range attrs {
		// SnapVersionIDFromName trusts its input to already be filtered to the
		// snapshot prefix (it does not re-check the "snapver:" marker) — safe here
		// because at.Name always comes from listing under SnapVersionPrefix above.
		id, ok := a.layout.SnapVersionIDFromName(at.Name)
		if !ok {
			continue
		}
		size, _ := strconv.ParseInt(at.Metadata[snapMetaSize], 10, 64)
		out = append(out, persistence.SnapshotInfo{
			ID:        id,
			Label:     at.Metadata[snapMetaLabel],
			CreatedAt: at.Created.UTC(),
			Size:      size,
		})
	}
	sort.Slice(out, func(i, j int) bool { return out[i].ID > out[j].ID }) // newest first
	return out, nil
}

// GetSnapshotState returns the snapshot's state exactly as it was saved.
func (a *Adapter) GetSnapshotState(ctx context.Context, room string, id int64) ([]byte, error) {
	if err := a.validate(room); err != nil {
		return nil, err
	}
	b, err := a.store.get(ctx, a.layout.SnapVersionName(DocID(room), id))
	if err == errNotFound {
		return nil, persistence.ErrSnapshotNotFound
	}
	if err != nil {
		return nil, err
	}
	return decompressBrotli(b)
}

// DeleteSnapshot removes one snapshot. Deleting an unknown snapshot is a no-op.
func (a *Adapter) DeleteSnapshot(ctx context.Context, room string, id int64) error {
	if err := a.validate(room); err != nil {
		return err
	}
	err := a.store.delete(ctx, a.layout.SnapVersionName(DocID(room), id))
	if err == errNotFound {
		return nil
	}
	return err
}

var _ persistence.SnapshotStore = (*Adapter)(nil)
