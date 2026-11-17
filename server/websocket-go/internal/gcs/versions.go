package gcs

import (
	"context"

	"github.com/reearth/reearth-flow/websocket-go/internal/versions"
)

// VersionStore stores versions as attributed snapshots.
type VersionStore struct {
	a *Adapter
	// keep bounds retained snapshots per class (see trimSnapshots).
	keep int
}

// Versions returns the adapter as a versions.Snapshots, keeping at most keep
// automatic versions and keep versions saved on request (see trimSnapshots).
func (a *Adapter) Versions(keep int) *VersionStore {
	return &VersionStore{a: a, keep: keep}
}

var _ versions.Snapshots = (*VersionStore)(nil)

// Save stores v and then trims the snapshots of its class. A failed trim is
// logged: the version is stored, and the next save trims again.
func (s *VersionStore) Save(ctx context.Context, room string, v versions.Version) (int64, error) {
	id, err := s.a.SaveAttributedSnapshot(ctx, room, v.Label, v.State, v.Attribution)
	if err != nil {
		return 0, err
	}
	class := classVersion
	if v.Label == versions.AutoLabel {
		class = classAuto
	}
	if err := s.a.trimSnapshots(ctx, room, class, s.keep); err != nil {
		s.a.log.Warn("trimming old versions failed", "room", room, "label", v.Label, "err", err)
	}
	return id, nil
}

// Latest returns room's newest version; ok is false when it has none.
func (s *VersionStore) Latest(ctx context.Context, room string) (versions.StoredVersion, bool, error) {
	return s.a.LatestVersion(ctx, room)
}

// Attribute replaces the attribution of room's snapshot id.
func (s *VersionStore) Attribute(ctx context.Context, room string, id int64, a versions.Attribution) error {
	return s.a.AttributeSnapshot(ctx, room, id, a)
}
