package gcs

import (
	"context"

	"github.com/reearth/reearth-flow/websocket-go/internal/versions"
)

// VersionStore stores versions as attributed snapshots.
type VersionStore struct {
	a *Adapter
	// keep bounds retained snapshots per label; <= 0 keeps everything.
	keep int
}

// Versions returns the adapter as a versions.Snapshots, keeping at most keep
// snapshots per label.
func (a *Adapter) Versions(keep int) *VersionStore {
	return &VersionStore{a: a, keep: keep}
}

var _ versions.Snapshots = (*VersionStore)(nil)

// Save stores v and then trims the snapshots sharing its label. A failed trim
// is logged: the version is stored, and the next save trims again.
func (s *VersionStore) Save(ctx context.Context, room string, v versions.Version) (int64, error) {
	id, err := s.a.SaveAttributedSnapshot(ctx, room, v.Label, v.State, Authors{
		Contributors: v.Contributors,
		LastEditor:   v.LastEditor,
	})
	if err != nil {
		return 0, err
	}
	if err := s.a.TrimSnapshots(ctx, room, v.Label, s.keep); err != nil {
		s.a.log.Warn("trimming old versions failed", "room", room, "label", v.Label, "err", err)
	}
	return id, nil
}

// Latest returns the state of room's newest snapshot, or nil when it has none.
func (s *VersionStore) Latest(ctx context.Context, room string) ([]byte, error) {
	return s.a.LatestSnapshotState(ctx, room)
}
