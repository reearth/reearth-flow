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
	s := &Server{
		cfg: cfg,
		ws:  ygws.NewServerWithPersistence(adapter),
		log: slog.Default(),
	}
	s.ws.AllowedOrigins = cfg.Origins
	s.ws.Logger = s.log
	// DoS caps must mirror New; ygo treats 0 as unlimited.
	s.ws.MaxConnections = cfg.MaxConnections
	s.ws.MaxPeersPerRoom = cfg.MaxPeersPerRoom
	s.ws.MaxRooms = cfg.MaxRooms
	return s
}
