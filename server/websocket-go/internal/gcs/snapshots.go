package gcs

import (
	"context"
	"errors"
	"fmt"
	"slices"
	"sort"
	"strconv"
	"strings"
	"time"
	"unicode/utf8"

	"github.com/reearth/ygo/persistence"

	"github.com/reearth/reearth-flow/websocket-go/internal/versions"
)

// Snapver object metadata. Stored so ListSnapshots never reads a payload; the
// size is the UNCOMPRESSED length, unlike attrs.Size.
const (
	snapMetaLabel = "ygo-label"
	snapMetaSize  = "ygo-size"
	// snapMetaContributors lists the users whose edits the version holds,
	// comma-separated, the last editor first and the rest sorted;
	// snapMetaContributorCount is how many there were, which is more than the
	// list holds when it was cut to fit. snapMetaLastEditor is the last editor
	// and snapMetaLastEditedAt the time of the latest edit (Unix ms).
	snapMetaContributors     = "flow-contributors"
	snapMetaContributorCount = "flow-contributor-count"
	snapMetaLastEditor       = "flow-last-editor"
	snapMetaLastEditedAt     = "flow-last-edited-at"
	// snapMetaOrigin is "version" on a snapshot versions.Saver stored.
	snapMetaOrigin = "flow-origin"
	originVersion  = "version"
)

// snapshotClass is a retention class: TrimSnapshots bounds each separately.
type snapshotClass int

const (
	// classAuto is the automatic versions (versions.AutoLabel).
	classAuto snapshotClass = iota
	// classVersion is the versions saved on request (Ctrl+S, runs,
	// deployments), whatever their label.
	classVersion
	// classOther is every other snapshot: the one the editor takes before a
	// restore, and named snapshots saved before versions were attributed.
	classOther
)

func classOf(label string, meta map[string]string) snapshotClass {
	switch {
	case label == versions.AutoLabel:
		return classAuto
	case meta[snapMetaOrigin] == originVersion:
		return classVersion
	default:
		return classOther
	}
}

// maxContributorBytes bounds the contributor list. GCS caps an object's custom
// metadata at 8 KiB, keys included; the label takes up to maxLabelBytes of it.
const maxContributorBytes = 4096

// SaveSnapshot stores state as a new labelled snapshot and returns its id,
// then trims its class to the adapter's KeepSnapshots (see TrimSnapshots); a
// failed trim is logged. state is an opaque blob, only compressed, never
// decoded as a CRDT update: SnapshotStore's contract allows any non-empty bytes.
func (a *Adapter) SaveSnapshot(ctx context.Context, room, label string, state []byte) (int64, error) {
	id, err := a.saveSnapshot(ctx, room, label, state, nil)
	if err != nil {
		return 0, err
	}
	if err := a.trimSnapshots(ctx, room, classOf(label, nil), a.keepSnapshots); err != nil {
		a.log.Warn("trimming old snapshots failed", "room", room, "label", label, "err", err)
	}
	return id, nil
}

// SaveAttributedSnapshot stores a version versions.Saver saved, recording who
// it is attributed to. It does not trim.
func (a *Adapter) SaveAttributedSnapshot(ctx context.Context, room, label string, state []byte, attr versions.Attribution) (int64, error) {
	meta := attributionMeta(attr)
	meta[snapMetaOrigin] = originVersion
	return a.saveSnapshot(ctx, room, label, state, meta)
}

// attributionMeta encodes attr as object metadata, cutting the contributor
// list to maxContributorBytes. The last editor is kept first so a cut list
// still names them. An id containing a comma cannot be listed and is left out.
func attributionMeta(attr versions.Attribution) map[string]string {
	ids := make([]string, 0, len(attr.Contributors)+1)
	seen := map[string]bool{}
	add := func(id string) {
		if id == "" || seen[id] || strings.Contains(id, ",") || !utf8.ValidString(id) {
			return
		}
		seen[id] = true
		ids = append(ids, id)
	}
	add(attr.LastEditor)
	rest := append([]string(nil), attr.Contributors...)
	sort.Strings(rest)
	for _, id := range rest {
		add(id)
	}
	var b strings.Builder
	for _, id := range ids {
		n := len(id)
		if b.Len() > 0 {
			n++
		}
		if b.Len()+n > maxContributorBytes {
			break
		}
		if b.Len() > 0 {
			b.WriteByte(',')
		}
		b.WriteString(id)
	}
	meta := map[string]string{
		snapMetaContributors:     b.String(),
		snapMetaContributorCount: strconv.Itoa(max(attr.ContributorCount, len(ids))),
	}
	if seen[attr.LastEditor] {
		meta[snapMetaLastEditor] = attr.LastEditor
	}
	if !attr.LastEditedAt.IsZero() {
		meta[snapMetaLastEditedAt] = strconv.FormatInt(attr.LastEditedAt.UnixMilli(), 10)
	}
	return meta
}

// attributionFromMeta reads a snapshot's attribution from its object metadata.
// The contributor list is as stored, so it can be cut short; ContributorCount
// is how many contributors there were.
func attributionFromMeta(meta map[string]string) (attr versions.Attribution) {
	attr.LastEditor = meta[snapMetaLastEditor]
	if v := meta[snapMetaContributors]; v != "" {
		attr.Contributors = strings.Split(v, ",")
		sort.Strings(attr.Contributors)
	}
	if ms, err := strconv.ParseInt(meta[snapMetaLastEditedAt], 10, 64); err == nil {
		attr.LastEditedAt = time.UnixMilli(ms)
	}
	count, _ := strconv.Atoi(meta[snapMetaContributorCount])
	attr.ContributorCount = max(count, len(attr.Contributors))
	return attr
}

func (a *Adapter) saveSnapshot(ctx context.Context, room, label string, state []byte, extra map[string]string) (int64, error) {
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
	for k, v := range extra {
		meta[k] = v
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
	listed, err := a.ListSnapshotDetails(ctx, room)
	if err != nil {
		return nil, err
	}
	out := make([]persistence.SnapshotInfo, len(listed))
	for i, l := range listed {
		out[i] = l.SnapshotInfo
	}
	return out, nil
}

// ListSnapshotDetails is ListSnapshots with each snapshot's attribution.
func (a *Adapter) ListSnapshotDetails(ctx context.Context, room string) ([]versions.ListedVersion, error) {
	listed, err := a.listSnapshots(ctx, room)
	if err != nil {
		return nil, err
	}
	out := make([]versions.ListedVersion, len(listed))
	for i, l := range listed {
		out[i] = l.ListedVersion
	}
	return out, nil
}

// listedSnapshot is a listed snapshot with its retention class.
type listedSnapshot struct {
	versions.ListedVersion
	class snapshotClass
}

// listSnapshots lists room's snapshots newest first, from object attributes.
func (a *Adapter) listSnapshots(ctx context.Context, room string) ([]listedSnapshot, error) {
	if err := a.validate(room); err != nil {
		return nil, err
	}
	d := DocID(room)
	// seen de-duplicates by id across layouts. The primary layout is scanned
	// first, so a room that already re-saved a snapshot post-cutover shows the
	// primary copy and the legacy one is ignored.
	seen := make(map[int64]struct{})
	out := make([]listedSnapshot, 0)
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
			label := at.Metadata[snapMetaLabel]
			out = append(out, listedSnapshot{
				ListedVersion: versions.ListedVersion{
					SnapshotInfo: persistence.SnapshotInfo{
						ID:        id,
						Label:     label,
						CreatedAt: at.Created.UTC(),
						Size:      size,
					},
					Attribution: attributionFromMeta(at.Metadata),
				},
				class: classOf(label, at.Metadata),
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

// trimSnapshots deletes the oldest of room's snapshots in class beyond keep.
// Each class is bounded on its own, so automatic versions never evict ones
// saved on request, neither evicts the snapshot taken before a restore, and a
// caller sending a new label each time still has its versions bounded.
// keep <= 0 keeps everything.
func (a *Adapter) trimSnapshots(ctx context.Context, room string, class snapshotClass, keep int) error {
	if keep <= 0 {
		return nil
	}
	snaps, err := a.listSnapshots(ctx, room)
	if err != nil {
		return err
	}
	var errs []error
	n := 0
	for _, sn := range snaps { // newest first
		if sn.class != class {
			continue
		}
		n++
		if n <= keep {
			continue
		}
		if err := a.DeleteSnapshot(ctx, room, sn.ID); err != nil {
			errs = append(errs, err)
		}
	}
	return errors.Join(errs...)
}

// LatestVersion returns room's newest version, automatic or saved on request,
// with its state; ok is false when it has none. Other snapshots, such as the
// one taken before a restore, are skipped: edits are never credited to them.
func (a *Adapter) LatestVersion(ctx context.Context, room string) (versions.StoredVersion, bool, error) {
	listed, err := a.listSnapshots(ctx, room)
	if err != nil {
		return versions.StoredVersion{}, false, err
	}
	i := slices.IndexFunc(listed, func(l listedSnapshot) bool { return l.class != classOther })
	if i < 0 {
		return versions.StoredVersion{}, false, nil
	}
	l := listed[i]
	state, err := a.GetSnapshotState(ctx, room, l.ID)
	if errors.Is(err, persistence.ErrSnapshotNotFound) {
		return versions.StoredVersion{}, false, nil // deleted since the listing
	}
	if err != nil {
		return versions.StoredVersion{}, false, err
	}
	return versions.StoredVersion{ID: l.ID, Label: l.Label, CreatedAt: l.CreatedAt, State: state, Attribution: l.Attribution}, true, nil
}

// AttributeSnapshot replaces the attribution of room's snapshot id, keeping its
// label and size. persistence.ErrSnapshotNotFound when there is no such
// snapshot.
func (a *Adapter) AttributeSnapshot(ctx context.Context, room string, id int64, attr versions.Attribution) error {
	if err := a.validate(room); err != nil {
		return err
	}
	d := DocID(room)
	for _, l := range a.snapshotLayouts() {
		err := a.store.updateMeta(ctx, l.SnapVersionName(d, id), func(meta map[string]string) {
			for _, k := range []string{snapMetaContributors, snapMetaContributorCount, snapMetaLastEditor, snapMetaLastEditedAt} {
				delete(meta, k)
			}
			// The origin and the rest of the metadata stay.
			for k, v := range attributionMeta(attr) {
				meta[k] = v
			}
		})
		if err == nil {
			return nil
		}
		if err != errNotFound {
			return err
		}
	}
	return persistence.ErrSnapshotNotFound
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
