package server

import (
	"context"
	"testing"

	"github.com/reearth/ygo/persistence"

	"github.com/reearth/reearth-flow/websocket-go/internal/config"
)

// TestNewWithPersistenceReplicatesDoSCaps verifies the persistence-wired
// constructor sets the same caps as New, never leaving them at ygo's 0 (unlimited).
func TestNewWithPersistenceReplicatesDoSCaps(t *testing.T) {
	cfg := &config.Config{
		MaxConnections:  111,
		MaxPeersPerRoom: 22,
		MaxRooms:        333,
		Origins:         []string{"https://example.test"},
	}
	s := NewWithPersistence(context.Background(), cfg, persistence.NewMemoryPersistence())
	if s.ws.MaxConnections != 111 {
		t.Errorf("MaxConnections = %d, want 111", s.ws.MaxConnections)
	}
	if s.ws.MaxPeersPerRoom != 22 {
		t.Errorf("MaxPeersPerRoom = %d, want 22", s.ws.MaxPeersPerRoom)
	}
	if s.ws.MaxRooms != 333 {
		t.Errorf("MaxRooms = %d, want 333", s.ws.MaxRooms)
	}
	if len(s.ws.AllowedOrigins) != 1 || s.ws.AllowedOrigins[0] != "https://example.test" {
		t.Errorf("AllowedOrigins = %v, want [https://example.test]", s.ws.AllowedOrigins)
	}
}
