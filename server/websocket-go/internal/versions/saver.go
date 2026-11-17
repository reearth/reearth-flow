package versions

import (
	"context"
	"errors"
	"fmt"
	"log/slog"
	"strings"
	"time"

	"github.com/reearth/ygo/crdt"
)

// AutoLabel labels the versions saved without a request: every interval while
// a room changes, and when the last person leaves it. The editor shows it as
// "Autosaved".
const AutoLabel = "auto"

// restoreTimeout bounds putting a record back after a version failed.
const restoreTimeout = 5 * time.Second

// Version is one version to store.
type Version struct {
	Label string
	// State is the document's full state as a V1 update.
	State []byte
	// Contributors are the users who edited since the previous version, and
	// LastEditor the one who edited last (empty when unknown).
	Contributors []string
	LastEditor   string
}

// Snapshots stores versions.
type Snapshots interface {
	// Save stores v as room's newest version and returns its id, which is
	// unique within the room.
	Save(ctx context.Context, room string, v Version) (int64, error)
	// Latest returns the state of room's newest version, or nil when it has
	// none.
	Latest(ctx context.Context, room string) ([]byte, error)
}

// Saved describes a version just saved. It is what the API is told.
type Saved struct {
	ProjectID string `json:"projectId"`
	// VersionID is the version's id, unique within the project.
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
	// Checkpoint, when set, rewrites the room's persisted full state from the
	// state just saved as a version.
	Checkpoint func(ctx context.Context, room string, state []byte) error
	// Notify, when set, is told of every version saved. It must not block.
	Notify func(Saved)
	Log    *slog.Logger
	Now    func() time.Time
}

// Save saves room's document as a version labelled label, if users changed it
// since its last version. state returns the document's current full state.
//
// The room's change record is taken before the state is read, so an edit
// arriving in between starts a new record: the next version accounts for it
// even if this one already holds it. When the state then turns out to equal
// the newest version, nothing is saved.
func (s *Saver) Save(ctx context.Context, room, label string, state func(context.Context) ([]byte, error)) (Result, error) {
	log := s.log()
	if s.Tracker != nil {
		if err := s.Tracker.Flush(ctx, room); err != nil {
			// The Tracker keeps the edits and writes them later, so the next
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
	restore := func(cause error) {
		rctx, cancel := context.WithTimeout(context.WithoutCancel(ctx), restoreTimeout)
		defer cancel()
		if err := s.Store.Mark(rctx, room, rec.Mark()); err != nil {
			log.Error("a version failed and its change record could not be put back; the change will be saved with the next edit",
				"room", room, "cause", cause, "err", err)
		}
	}

	st, err := state(ctx)
	if err != nil {
		restore(err)
		return Result{}, fmt.Errorf("versions: read document state: %w", err)
	}
	if len(st) == 0 {
		// Even an empty document encodes to a few bytes, so this is a document
		// that could not be read: keep the change for the next trigger.
		restore(errors.New("no document state"))
		log.Warn("no document state to save as a version", "room", room)
		return Result{}, nil
	}
	if s.sameAsLatest(ctx, room, st) {
		log.Debug("document equals its newest version; nothing to save", "room", room)
		return Result{}, nil
	}

	id, err := s.Snapshots.Save(ctx, room, Version{
		Label:        label,
		State:        st,
		Contributors: rec.Contributors,
		LastEditor:   rec.LastEditor,
	})
	if err != nil {
		restore(err)
		return Result{}, fmt.Errorf("versions: save snapshot: %w", err)
	}
	res := Result{Saved: true, SnapshotID: id}
	log.Info("saved document version", "room", room, "snapshot", id, "label", label,
		"contributors", len(rec.Contributors))

	if s.Notify != nil {
		contributors := rec.Contributors
		if contributors == nil {
			contributors = []string{}
		}
		s.Notify(Saved{
			ProjectID:      ProjectOf(room),
			VersionID:      id,
			Label:          label,
			SavedAt:        s.now().UTC(),
			LastEditedAt:   rec.LastEditAt.UTC(),
			LastEditorID:   rec.LastEditor,
			ContributorIDs: contributors,
		})
	}

	if s.Checkpoint != nil {
		if err := s.Checkpoint(ctx, room, st); err != nil {
			return res, fmt.Errorf("%w: %w", ErrCheckpoint, err)
		}
	}
	return res, nil
}

// sameAsLatest reports whether state holds exactly what room's newest version
// holds. It compares content, not bytes: two copies of a document can encode
// the same content differently. Anything that prevents the comparison counts
// as a difference, so a change is never dropped for it.
func (s *Saver) sameAsLatest(ctx context.Context, room string, state []byte) bool {
	latest, err := s.Snapshots.Latest(ctx, room)
	if err != nil {
		s.log().Warn("reading the newest version failed; saving without comparing", "room", room, "err", err)
		return false
	}
	if len(latest) == 0 {
		return false
	}
	return holds(latest, state) && holds(state, latest)
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
