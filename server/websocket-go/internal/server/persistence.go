package server

import (
	"context"
	"log/slog"

	"github.com/reearth/ygo/persistence"
	ygws "github.com/reearth/ygo/provider/websocket"

	"github.com/reearth/reearth-flow/websocket-go/internal/config"
)

// NewWithPersistence builds a Server whose ygo provider loads and stores room
// state via the given VersionedPersistence. ctx is threaded into the adapter so
// I/O-backed stores abort in-flight writes on shutdown.
func NewWithPersistence(ctx context.Context, cfg *config.Config, p persistence.VersionedPersistence) *Server {
	s := &Server{
		cfg: cfg,
		ws:  ygws.NewServerWithPersistence(persistence.NewLegacyAdapterContext(ctx, p)),
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

// CheckpointState returns room's live state for a checkpoint, or nil when the
// checkpoint must be rebuilt from storage instead: when this instance does not
// hold the room, or its copy holds parked updates. A copy with parked updates
// missed an edit another instance applied and persisted, so storage is
// complete where the copy is not.
func (s *Server) CheckpointState(room string) []byte {
	doc := s.ws.GetDoc(room)
	if doc == nil {
		return nil
	}
	if ps := doc.PendingStats(); ps.Items > 0 {
		s.log.Warn("room holds parked updates; checkpointing it from storage",
			"room", room, "parked", ps.Items, "awaitingClients", len(ps.MissingFor))
		return nil
	}
	return encodeFullState(doc)
}
