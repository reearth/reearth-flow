package server

import (
	"context"
	"testing"
	"time"

	"github.com/reearth/ygo/crdt"
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

// TestNewWithPersistence_KeepsYgoAutoVersioningOff: ygo's auto-versioning
// counts every persisted update as a change, so it stays off whatever the
// configured interval; EnableVersions drives versions instead.
func TestNewWithPersistence_KeepsYgoAutoVersioningOff(t *testing.T) {
	t.Parallel()

	cfg := &config.Config{AutoVersionEvery: 15 * time.Minute, KeepSnapshots: 50}
	s := NewWithPersistence(context.Background(), cfg, persistence.NewMemoryPersistence())
	if got := s.ws.AutoVersionEvery; got != 0 {
		t.Fatalf("ygo AutoVersionEvery = %v, want 0", got)
	}
}

// applyOneChange commits one real doc mutation to room via the exported
// Apply, mirroring rollback_signal.go's SignalRollback usage: GetMap outside
// transact (which already holds the doc lock), then the actual mutation
// inside transact.
func applyOneChange(t *testing.T, s *Server, room string) {
	t.Helper()
	err := s.ws.Apply(context.Background(), room, func(doc *crdt.Doc, transact func(func(*crdt.Transaction))) {
		m := doc.GetMap("content")
		transact(func(txn *crdt.Transaction) {
			m.Set(txn, "key", "value")
		})
	})
	if err != nil {
		t.Fatalf("Apply: %v", err)
	}
}

// TestNewWithPersistence_ServerWriteSavesNoVersion: a change the server makes
// itself (a project lock, a rollback signal) is no user edit, so closing the
// room afterwards saves no version. ygo's auto-versioning saved one here.
func TestNewWithPersistence_ServerWriteSavesNoVersion(t *testing.T) {
	t.Parallel()

	const room = "550e8400-e29b-41d4-a716-446655440099"
	ctx := context.Background()
	mem := persistence.NewMemoryPersistence()
	cfg := &config.Config{AutoVersionEvery: time.Minute, KeepSnapshots: 50}
	s := NewWithPersistence(ctx, cfg, mem)

	applyOneChange(t, s, room)
	if err := s.ws.CloseRoom(room, true); err != nil {
		t.Fatalf("CloseRoom: %v", err)
	}

	snaps, err := mem.ListSnapshots(ctx, room)
	if err != nil {
		t.Fatalf("ListSnapshots: %v", err)
	}
	if len(snaps) != 0 {
		t.Fatalf("len(snapshots) = %d, want 0: a server write is not a user edit", len(snaps))
	}
}
