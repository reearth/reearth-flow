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
	adapter := persistence.NewLegacyAdapterContext(ctx, p)
	// Bound retained snapshots. LegacyAdapter.SaveVersion applies this after each
	// save; it is a different axis from KeepVersions (the update log).
	adapter.KeepSnapshots = cfg.KeepSnapshots
	s := &Server{
		cfg: cfg,
		ws:  ygws.NewServerWithPersistence(adapter),
		log: slog.Default(),
		// Retained so the retention wiring is assertable: ygo keeps the adapter in
		// an unexported field with no getter, so without this a test cannot tell
		// KeepSnapshots was ever applied.
		adapter: adapter,
	}
	s.ws.AllowedOrigins = cfg.Origins
	s.ws.Logger = s.log
	// DoS caps must mirror New; ygo treats 0 as unlimited.
	s.ws.MaxConnections = cfg.MaxConnections
	s.ws.MaxPeersPerRoom = cfg.MaxPeersPerRoom
	s.ws.MaxRooms = cfg.MaxRooms
	if cfg.SlowPeerResync {
		s.ws.SlowPeerPolicy = ygws.SlowPeerResync
	}
	// Auto-versioning: ygo captures a labelled snapshot at most once per interval
	// per room, and only when the room changed. Requires the store to implement
	// persistence.SnapshotStore, which the GCS adapter does.
	s.ws.AutoVersionEvery = cfg.AutoVersionEvery
	return s
}
