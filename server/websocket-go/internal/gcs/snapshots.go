package gcs

import (
	"context"
	"errors"
	"fmt"
	"sort"
	"strconv"
	"strings"
	"unicode/utf8"

	"github.com/reearth/ygo/persistence"
)

// Snapver object metadata. Stored so ListSnapshots never reads a payload; the
// size is the UNCOMPRESSED length, unlike attrs.Size.
const (
	snapMetaLabel = "ygo-label"
	snapMetaSize  = "ygo-size"
)

// SaveSnapshot stores state as a new labelled snapshot and returns its id.
// state is an opaque blob, only compressed, never decoded as a CRDT update:
// SnapshotStore's contract allows any non-empty bytes.
func (a *Adapter) SaveSnapshot(ctx context.Context, room, label string, state []byte) (int64, error) {
	if err := a.validate(room); err != nil {
		return 0, err
	}
	if len(state) == 0 {
		return 0, persistence.ErrEmptySnapshot
	}
	if err := validateSnapshotLabel(label); err != nil {
		return 0, err
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
			// Id taken, so the counter is behind reality: re-derive the floor and retry.
			// Safe because the write is create-only, so the loser destroys nothing.
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

// maxSnapshotIDAttempts bounds the retry when an allocated id is already taken.
const maxSnapshotIDAttempts = 3

// maxLabelBytes bounds the label; GCS caps custom metadata at 8 KiB per object.
const maxLabelBytes = 1024

// ErrInvalidSnapshotLabel reports a label that cannot be stored in object
// metadata.
var ErrInvalidSnapshotLabel = errors.New("gcs: invalid snapshot label")

// validateSnapshotLabel rejects labels GCS cannot round-trip. Without it an
// oversized label 400s only after an id is consumed, and bad UTF-8 is mangled.
func validateSnapshotLabel(label string) error {
	if len(label) > maxLabelBytes {
		return fmt.Errorf("%w: %d bytes exceeds the %d-byte limit", ErrInvalidSnapshotLabel, len(label), maxLabelBytes)
	}
	if !utf8.ValidString(label) {
		return fmt.Errorf("%w: not valid UTF-8", ErrInvalidSnapshotLabel)
	}
	return nil
}

// nextSnapID returns the room's next snapshot id. The counter object is a fast
// path, NOT the source of truth: trusting it alone means a lost counter restarts
// at 1 and overwrites live snapshots, so an unusable counter falls back to the
// ids that exist. Caller holds gcsDocLockKey(d).
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

// maxSnapIDFromObjects returns the highest stored snapshot id, or 0 if none.
// Scans both layouts so a Phase-2 id cannot collide with a legacy-root one.
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

// snapshotLayouts returns the readable layouts, primary first so it wins
// de-duplication. The legacy root is included under Phase 2.
func (a *Adapter) snapshotLayouts() []Layout {
	if a.fallback == nil {
		return []Layout{a.layout}
	}
	return []Layout{a.layout, a.fallback}
}

// ListSnapshots returns snapshot metadata newest-first, reading only object
// attributes. Merges the legacy root under Phase 2, without which flipping
// REEARTH_FLOW_GCS_PHASE2 would silently empty every existing room's history.
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

// GetSnapshotState returns the state as saved, falling back to the legacy root
// under Phase 2 so every id ListSnapshots reports stays readable.
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

// DeleteSnapshot removes one snapshot; unknown ids are a no-op. Sweeps both
// layouts so a legacy copy cannot reappear on the next list.
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
