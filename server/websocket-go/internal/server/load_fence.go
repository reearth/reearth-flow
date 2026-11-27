package server

import (
	"context"

	"github.com/reearth/ygo/persistence"
)

// LoadFence is called before ygo loads a room from storage, and the func it
// returns with the load's result. The Redis relay uses it so that no instance
// trims or deletes stream entries a room that loaded storage without them still
// needs; see the relay's loadfence.go.
type LoadFence func(ctx context.Context, room string) func(err error)

// SetLoadFence installs the fence for room loads, or turns fencing off when f is
// nil. Install it before the relay attaches so startup loads are fenced too.
func (s *Server) SetLoadFence(f LoadFence) {
	if f == nil {
		s.loadFence.Store(nil)
		return
	}
	s.loadFence.Store(&f)
}

// fencedStore wraps the store ygo loads rooms from so every load is fenced. A
// store that is also a SnapshotStore stays one: ygo's LegacyAdapter saves
// versions only through that interface.
func (s *Server) fencedStore(p persistence.VersionedPersistence) persistence.VersionedPersistence {
	fs := fencingStore{VersionedPersistence: p, s: s}
	if ss, ok := p.(persistence.SnapshotStore); ok {
		return fencingSnapshotStore{fencingStore: fs, SnapshotStore: ss}
	}
	return fs
}

type fencingStore struct {
	persistence.VersionedPersistence
	s *Server
}

// Load fences the room, then loads it. ygo calls LoadDoc only to open a room.
func (f fencingStore) Load(ctx context.Context, room string) (persistence.LoadResult, error) {
	fence := f.s.loadFence.Load()
	if fence == nil {
		return f.VersionedPersistence.Load(ctx, room)
	}
	done := (*fence)(ctx, room)
	res, err := f.VersionedPersistence.Load(ctx, room)
	done(err)
	return res, err
}

type fencingSnapshotStore struct {
	fencingStore
	persistence.SnapshotStore
}
