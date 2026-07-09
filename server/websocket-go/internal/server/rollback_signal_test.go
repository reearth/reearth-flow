package server

import (
	"context"
	"testing"

	"github.com/reearth/reearth-flow/websocket-go/internal/config"
)

// TestSignalRollbackTogglesMetadata asserts SignalRollback sets and clears
// metadata.rollbackInProgress on the live room's doc.
func TestSignalRollbackTogglesMetadata(t *testing.T) {
	s := New(&config.Config{Origins: []string{"*"}, MaxConnections: 10, MaxPeersPerRoom: 10, MaxRooms: 10})
	ctx := context.Background()
	const room = "proj1"

	if err := s.SignalRollback(ctx, room, true); err != nil {
		t.Fatalf("set: %v", err)
	}
	doc := s.WSProvider().GetDoc(room)
	if doc == nil {
		t.Fatalf("room doc not resident after signal")
	}
	v, ok := doc.GetMap("metadata").Get("rollbackInProgress")
	if !ok || v != true {
		t.Fatalf("rollbackInProgress = %v, ok=%v; want true", v, ok)
	}

	if err := s.SignalRollback(ctx, room, false); err != nil {
		t.Fatalf("clear: %v", err)
	}
	if _, ok := doc.GetMap("metadata").Get("rollbackInProgress"); ok {
		t.Fatalf("rollbackInProgress still present after clear")
	}
}
