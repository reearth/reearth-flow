package server

import (
	"context"
	"log/slog"

	"github.com/reearth/ygo/persistence"
	ygws "github.com/reearth/ygo/provider/websocket"

	"github.com/reearth/reearth-flow/websocket-go/internal/config"
)

// legacyAdapter wraps p for ygo. ygo compacts on every room unload with the
// adapter's KeepVersions, and 0 would keep the whole update log. The retention
// is cfg.KeepUpdates, the same one the admin cleanup routes trim to, so neither
// undoes the other.
func legacyAdapter(ctx context.Context, cfg *config.Config, p persistence.VersionedPersistence) *persistence.LegacyAdapter {
	la := persistence.NewLegacyAdapterContext(ctx, p)
	la.KeepVersions = keepUpdatesOf(cfg)
	return la
}

// keepUpdatesOf is cfg.KeepUpdates, falling back to the default for a config
// built without it (tests construct literals).
func keepUpdatesOf(cfg *config.Config) int {
	if cfg == nil || cfg.KeepUpdates < 1 {
		return config.DefaultKeepUpdates
	}
	return cfg.KeepUpdates
}

// NewWithPersistence builds a Server whose ygo provider loads and stores room
// state via the given VersionedPersistence. ctx is threaded into the adapter so
// I/O-backed stores abort in-flight writes on shutdown.
func NewWithPersistence(ctx context.Context, cfg *config.Config, p persistence.VersionedPersistence) *Server {
	s := &Server{
		cfg: cfg,
		ws:  ygws.NewServerWithPersistence(legacyAdapter(ctx, cfg, p)),
		log: slog.Default(),
	}
	s.ws.AllowedOrigins = cfg.Origins
	s.ws.Logger = s.log
	// DoS caps must mirror New; ygo treats 0 as unlimited.
	s.ws.MaxConnections = cfg.MaxConnections
	s.ws.MaxPeersPerRoom = cfg.MaxPeersPerRoom
	s.ws.MaxRooms = cfg.MaxRooms
	s.ws.RoomIdleTimeout = cfg.RoomIdleTimeout
	if cfg.SlowPeerResync {
		s.ws.SlowPeerPolicy = ygws.SlowPeerResync
	}
	// ygo's own auto-versioning stays off: it counts every persisted update as a
	// change, server writes and other instances' relayed edits included, and
	// each instance holding a room would save its own copy. EnableVersions saves
	// versions from user edits instead.
	s.ws.AutoVersionEvery = 0
	s.enableProjectLock()
	return s
}

// CheckpointState returns room's live state for a checkpoint, or nil when this
// instance does not hold the room and the checkpoint is rebuilt from storage. A
// copy holding parked updates is offered too: the checkpoint is its union with
// the stored state, which supplies any missed edit storage has.
func (s *Server) CheckpointState(room string) []byte {
	doc := s.ws.GetDoc(room)
	if doc == nil {
		return nil
	}
	return encodeFullState(doc)
}
