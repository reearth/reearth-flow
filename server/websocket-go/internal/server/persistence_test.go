package server

import (
	"context"
	"testing"
	"time"

	"github.com/reearth/ygo/crdt"
	"github.com/reearth/ygo/persistence"
	ygws "github.com/reearth/ygo/provider/websocket"

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

func TestNewWithPersistence_WiresAutoVersioning(t *testing.T) {
	cfg := &config.Config{
		AutoVersionEvery: 15 * time.Minute,
		KeepSnapshots:    50,
	}
	s := NewWithPersistence(context.Background(), cfg, persistence.NewMemoryPersistence())
	if got := s.ws.AutoVersionEvery; got != 15*time.Minute {
		t.Fatalf("AutoVersionEvery = %v, want 15m", got)
	}
	// Assert the retention bound too. Without this, deleting the
	// `adapter.KeepSnapshots = cfg.KeepSnapshots` line leaves every test passing
	// while retention silently reverts to keep-all — the config would advertise a
	// bound that nothing enforces.
	if s.adapter == nil {
		t.Fatal("adapter reference not retained; cannot verify retention wiring")
	}
	if got := s.adapter.KeepSnapshots; got != 50 {
		t.Fatalf("adapter.KeepSnapshots = %d, want 50 (config value must reach the adapter)", got)
	}
}

func TestNewWithPersistence_ZeroConfigDisablesAutoVersioning(t *testing.T) {
	s := NewWithPersistence(context.Background(), &config.Config{}, persistence.NewMemoryPersistence())
	if got := s.ws.AutoVersionEvery; got != 0 {
		t.Fatalf("AutoVersionEvery = %v, want 0 (off)", got)
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

// TestNewWithPersistence_AutoVersionSavesSnapshotOnClose proves the wiring is
// not just plumbing: with a real SnapshotVersionedPersistence
// (MemoryPersistence, which implements SaveSnapshot/ListSnapshots per
// persistence/snapshots.go's compile-time assertion) behind NewWithPersistence,
// a real CRDT change followed by a forced CloseRoom must produce exactly one
// labelled snapshot carrying ygws.AutoVersionLabel. CloseRoom(room, true)
// closes persistStop and blocks on persistDone, which the persistence worker
// only closes after running its forced final maybeVersion, so no sleeping or
// fake clock is needed to observe the result.
//
// What each subcase actually guards: "enabled" is what fails if
// s.ws.AutoVersionEvery is left unwired (drop the assignment and it sees 0, so
// no snapshot is written). "disabled" guards the opposite mistake — a hardcoded
// non-zero default that would version rooms an operator asked to leave alone.
func TestNewWithPersistence_AutoVersionSavesSnapshotOnClose(t *testing.T) {
	const room = "550e8400-e29b-41d4-a716-446655440099"
	ctx := context.Background()

	t.Run("enabled", func(t *testing.T) {
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
		if len(snaps) != 1 {
			t.Fatalf("len(snapshots) = %d, want 1 (auto-version on unload of a dirty room)", len(snaps))
		}
		if snaps[0].Label != ygws.AutoVersionLabel {
			t.Errorf("snapshot label = %q, want %q", snaps[0].Label, ygws.AutoVersionLabel)
		}
	})

	t.Run("disabled", func(t *testing.T) {
		mem := persistence.NewMemoryPersistence()
		s := NewWithPersistence(ctx, &config.Config{}, mem) // AutoVersionEvery unset (0)

		applyOneChange(t, s, room)

		if err := s.ws.CloseRoom(room, true); err != nil {
			t.Fatalf("CloseRoom: %v", err)
		}

		snaps, err := mem.ListSnapshots(ctx, room)
		if err != nil {
			t.Fatalf("ListSnapshots: %v", err)
		}
		if len(snaps) != 0 {
			t.Fatalf("len(snapshots) = %d, want 0: auto-versioning must stay off when AutoVersionEvery is unset", len(snaps))
		}
	})
}
