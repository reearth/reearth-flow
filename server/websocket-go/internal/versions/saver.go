package versions

import (
	"context"
	"errors"
	"fmt"
	"hash/fnv"
	"log/slog"
	"slices"
	"strings"
	"sync"
	"time"

	"github.com/reearth/ygo/crdt"
	"github.com/reearth/ygo/persistence"
)

// AutoLabel labels the versions saved without a request: once a room's first
// unsaved edit is old enough, and when the last person leaves it. The editor
// shows it as "Autosaved".
const AutoLabel = "auto"

// putBackTimeout bounds putting edits back into a record.
const putBackTimeout = 5 * time.Second

// Attribution is who a version is attributed to.
type Attribution struct {
	// Contributors are the users whose edits the version holds, sorted.
	Contributors []string
	// LastEditor made the latest of those edits that named its sender; empty
	// when none did.
	LastEditor string
	// LastEditedAt is when the latest of those edits was made.
	LastEditedAt time.Time
}

// attributionOf credits the edits in r.
func attributionOf(r Record) Attribution {
	editor, _ := r.LastEditor()
	return Attribution{Contributors: r.Contributors(), LastEditor: editor, LastEditedAt: r.LastEditAt()}
}

// merge credits b's edits in addition to a's.
func (a Attribution) merge(b Attribution) Attribution {
	out := Attribution{
		Contributors: slices.Clone(a.Contributors),
		LastEditor:   a.LastEditor,
		LastEditedAt: a.LastEditedAt,
	}
	for _, u := range b.Contributors {
		if i, found := slices.BinarySearch(out.Contributors, u); !found {
			out.Contributors = slices.Insert(out.Contributors, i, u)
		}
	}
	if b.LastEditedAt.After(out.LastEditedAt) {
		out.LastEditedAt = b.LastEditedAt
		if b.LastEditor != "" {
			out.LastEditor = b.LastEditor
		}
	}
	if out.LastEditor == "" {
		out.LastEditor = b.LastEditor
	}
	return out
}

func (a Attribution) equal(b Attribution) bool {
	return slices.Equal(a.Contributors, b.Contributors) && a.LastEditor == b.LastEditor && a.LastEditedAt.Equal(b.LastEditedAt)
}

// Version is one version to store.
type Version struct {
	Label string
	// State is the document's full state as a V1 update.
	State []byte
	Attribution
}

// StoredVersion is a stored version.
type StoredVersion struct {
	ID        int64
	Label     string
	CreatedAt time.Time
	State     []byte
	Attribution
}

// ListedVersion is a stored version as listed: no state, and an attribution
// whose contributor list can be cut short to fit storage; ContributorCount is
// how many contributors there were.
type ListedVersion struct {
	persistence.SnapshotInfo
	Attribution
	ContributorCount int
}

// Snapshots stores versions.
type Snapshots interface {
	// Save stores v as room's newest version and returns its id.
	Save(ctx context.Context, room string, v Version) (int64, error)
	// Latest returns room's newest version; ok is false when it has none.
	Latest(ctx context.Context, room string) (v StoredVersion, ok bool, err error)
	// Attribute replaces the attribution of room's version id.
	Attribute(ctx context.Context, room string, id int64, a Attribution) error
}

// Saved describes a version just saved, or a saved version whose attribution
// grew. It is what the API is told.
type Saved struct {
	ProjectID string `json:"projectId"`
	// VersionID is the version's id. It identifies the version among the
	// project's current ones; deleting the project's document restarts the
	// ids, so it is not unique over the project's lifetime.
	VersionID    int64     `json:"versionId"`
	Label        string    `json:"label"`
	SavedAt      time.Time `json:"savedAt"`
	LastEditedAt time.Time `json:"lastEditedAt"`
	// LastEditorID is empty when no edit named its sender.
	LastEditorID   string   `json:"lastEditorId,omitempty"`
	ContributorIDs []string `json:"contributorIds"`
}

// ProjectOf returns the project a room belongs to: rooms are named
// "{projectId}" or "{projectId}:{suffix}".
func ProjectOf(room string) string {
	p, _, _ := strings.Cut(room, ":")
	return p
}

// Result reports what Save did.
type Result struct {
	// Saved is false when there was nothing to save.
	Saved      bool
	SnapshotID int64
}

// ErrCheckpoint wraps a failure to refresh the room's checkpoint after its
// version was saved. The version itself stands.
var ErrCheckpoint = errors.New("versions: refresh checkpoint")

// Saver saves versions. Every trigger goes through Save.
type Saver struct {
	// Tracker, when set, is this instance's Tracker: its gathered edits are
	// written before the record is taken.
	Tracker *Tracker
	Store   Store
	// Snapshots stores the versions.
	Snapshots Snapshots
	// Lock, when set, runs fn while no other instance saves room. Saves on
	// this instance are serialized per room either way.
	Lock func(ctx context.Context, room string, fn func(context.Context) error) error
	// Checkpoint, when set, rewrites the room's persisted full state after a
	// version, from the room's current state.
	Checkpoint func(ctx context.Context, room string) error
	// Notify, when set, is told of every version saved, and of a version
	// whose attribution grew. It must not block.
	Notify func(Saved)
	Log    *slog.Logger
	Now    func() time.Time

	stripes [64]sync.Mutex
}

// Save saves room's document as a version labelled label, if users changed it
// since its last version. state returns the document's current full state; it
// is called after the change record is taken.
//
// Saves of one room never overlap, here or on another instance, so a slower
// save cannot store an older state after a newer one. Edits the record names
// but the state does not hold go back into the record for the next version.
// Edits the state holds that turn out to be in the newest version already
// (they arrived during the previous save) are credited to that version.
func (s *Saver) Save(ctx context.Context, room, label string, state func(context.Context) ([]byte, error)) (Result, error) {
	mu := &s.stripes[stripe(room)]
	mu.Lock()
	defer mu.Unlock()
	if s.Lock == nil {
		return s.save(ctx, room, label, state)
	}
	var res Result
	err := s.Lock(ctx, room, func(ctx context.Context) error {
		var err error
		res, err = s.save(ctx, room, label, state)
		return err
	})
	return res, err
}

func stripe(room string) int {
	h := fnv.New32a()
	_, _ = h.Write([]byte(room))
	return int(h.Sum32() % 64)
}

func (s *Saver) save(ctx context.Context, room, label string, state func(context.Context) ([]byte, error)) (Result, error) {
	log := s.log()
	if s.Tracker != nil {
		if err := s.Tracker.Flush(ctx, room); err != nil {
			// The Tracker keeps the edits and writes them later, so a later
			// version accounts for them.
			log.Warn("writing this instance's edits before a version failed", "room", room, "err", err)
		}
	}
	rec, ok, err := s.Store.Take(ctx, room)
	if err != nil {
		return Result{}, fmt.Errorf("versions: take change record: %w", err)
	}
	if !ok {
		return Result{}, nil
	}
	putBack := func(r Record, cause error) {
		if len(r) == 0 {
			return
		}
		pctx, cancel := context.WithTimeout(context.WithoutCancel(ctx), putBackTimeout)
		defer cancel()
		if err := s.Store.Mark(pctx, room, r); err != nil {
			log.Error("edits taken for a version could not be put back; the room's next edit records it again",
				"room", room, "cause", cause, "err", err)
		}
	}

	st, err := state(ctx)
	if err != nil {
		putBack(rec, err)
		return Result{}, fmt.Errorf("versions: read document state: %w", err)
	}
	if len(st) == 0 {
		// Even an empty document encodes to a few bytes, so this is a document
		// that could not be read.
		putBack(rec, errors.New("no document state"))
		log.Warn("no document state to save as a version", "room", room)
		return Result{}, nil
	}
	held, missing, err := rec.Split(st)
	if err != nil {
		putBack(rec, err)
		return Result{}, err
	}
	if len(missing) > 0 {
		// Recorded edits the state does not hold yet, such as one another
		// instance received that has not reached this copy of the room.
		putBack(missing, errors.New("not in the saved state"))
		log.Debug("edits not yet in the state wait for the next version", "room", room, "users", len(missing))
	}
	if len(held) == 0 {
		return Result{}, nil
	}

	latest, hasLatest, err := s.Snapshots.Latest(ctx, room)
	if err != nil {
		log.Warn("reading the newest version failed; saving without comparing", "room", room, "err", err)
		hasLatest = false
	}
	if hasLatest && sameContent(latest.State, st) {
		s.credit(ctx, room, latest, attributionOf(held))
		return Result{}, nil
	}

	a := attributionOf(held)
	id, err := s.Snapshots.Save(ctx, room, Version{Label: label, State: st, Attribution: a})
	if err != nil {
		putBack(held, err)
		return Result{}, fmt.Errorf("versions: save snapshot: %w", err)
	}
	log.Info("saved document version", "room", room, "snapshot", id, "label", label, "contributors", len(a.Contributors))
	s.notify(room, id, label, s.now(), a)

	res := Result{Saved: true, SnapshotID: id}
	if s.Checkpoint != nil {
		if err := s.Checkpoint(ctx, room); err != nil {
			return res, fmt.Errorf("%w: %w", ErrCheckpoint, err)
		}
	}
	return res, nil
}

// credit adds a to the attribution of latest, which already holds the edits a
// describes.
func (s *Saver) credit(ctx context.Context, room string, latest StoredVersion, a Attribution) {
	merged := latest.merge(a)
	if merged.equal(latest.Attribution) {
		return
	}
	if err := s.Snapshots.Attribute(ctx, room, latest.ID, merged); err != nil {
		s.log().Warn("crediting edits to the newest version failed", "room", room, "snapshot", latest.ID, "err", err)
		return
	}
	s.log().Info("credited edits to the newest version", "room", room, "snapshot", latest.ID)
	s.notify(room, latest.ID, latest.Label, latest.CreatedAt, merged)
}

func (s *Saver) notify(room string, id int64, label string, savedAt time.Time, a Attribution) {
	if s.Notify == nil {
		return
	}
	contributors := a.Contributors
	if contributors == nil {
		contributors = []string{}
	}
	lastEdited := a.LastEditedAt
	if lastEdited.IsZero() {
		lastEdited = savedAt
	}
	s.Notify(Saved{
		ProjectID:      ProjectOf(room),
		VersionID:      id,
		Label:          label,
		SavedAt:        savedAt.UTC(),
		LastEditedAt:   lastEdited.UTC(),
		LastEditorID:   a.LastEditor,
		ContributorIDs: contributors,
	})
}

// sameContent reports whether two full states hold exactly the same content.
// It compares content, not bytes: two copies of a document can encode the same
// content differently. Anything that prevents the comparison counts as a
// difference, so a change is never dropped for it.
func sameContent(a, b []byte) bool {
	return holds(a, b) && holds(b, a)
}

// holds reports whether the document encoded by doc holds everything update
// carries.
func holds(doc, update []byte) bool {
	d := crdt.New()
	if err := crdt.ApplyUpdateV1(d, doc, nil); err != nil {
		return false
	}
	held, err := crdt.DocContainsUpdateV1(d, update)
	return err == nil && held
}

func (s *Saver) log() *slog.Logger {
	if s.Log != nil {
		return s.Log
	}
	return slog.Default()
}

func (s *Saver) now() time.Time {
	if s.Now != nil {
		return s.Now()
	}
	return time.Now()
}
