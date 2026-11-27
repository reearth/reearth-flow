package server

import (
	"context"
	"sync"
	"testing"

	"github.com/reearth/ygo/cluster"
	"github.com/reearth/ygo/persistence"
)

// TestFencedStore_CallsFenceAroundLoad: a room load is fenced before it reads
// storage, and the fence hears how the load ended.
func TestFencedStore_CallsFenceAroundLoad(t *testing.T) {
	t.Parallel()
	ctx := context.Background()
	s := NewWithPersistence(ctx, testConfig(), persistence.NewMemoryPersistence())

	var mu sync.Mutex
	var events []string
	s.SetLoadFence(func(_ context.Context, room string) func(error) {
		mu.Lock()
		events = append(events, "start "+room)
		mu.Unlock()
		return func(err error) {
			mu.Lock()
			defer mu.Unlock()
			if err != nil {
				events = append(events, "failed "+room)
				return
			}
			events = append(events, "loaded "+room)
		}
	})

	// An empty V1 update: Inject loads the room to apply it.
	if err := s.ws.Inject(ctx, cluster.Inbound{Room: "fenced-room", Kind: cluster.KindSync, Data: []byte{0, 0}}); err != nil {
		t.Fatalf("Inject: %v", err)
	}
	mu.Lock()
	defer mu.Unlock()
	if len(events) != 2 || events[0] != "start fenced-room" || events[1] != "loaded fenced-room" {
		t.Errorf("fence events = %v, want [start fenced-room loaded fenced-room]", events)
	}
}

// TestFencedStore_KeepsSnapshotStore: ygo's LegacyAdapter saves versions only
// when its store is a SnapshotStore, so the fence must not hide that.
func TestFencedStore_KeepsSnapshotStore(t *testing.T) {
	t.Parallel()
	s := &Server{}
	wrapped := s.fencedStore(persistence.NewMemoryPersistence())
	if _, ok := wrapped.(persistence.SnapshotStore); !ok {
		t.Error("fencing the store hid its SnapshotStore")
	}
}

type plainFlusher struct{}

func (plainFlusher) FlushRoom(context.Context, string) error { return nil }

type notLiveFlusher struct{ plainFlusher }

func (notLiveFlusher) CheckpointLive(context.Context, string) (bool, error) { return false, nil }

// TestVersionedFlusher_ForwardsCheckpointLive: the relay sees the GCS flusher
// only through this wrapper, so its answer must come through.
func TestVersionedFlusher_ForwardsCheckpointLive(t *testing.T) {
	t.Parallel()
	s := &Server{}

	t.Run("inner reports", func(t *testing.T) {
		t.Parallel()
		live, err := s.VersionedFlusher(notLiveFlusher{}).CheckpointLive(context.Background(), "r")
		if err != nil || live {
			t.Errorf("CheckpointLive = %v, %v; want false, nil", live, err)
		}
	})

	t.Run("inner cannot tell", func(t *testing.T) {
		t.Parallel()
		live, err := s.VersionedFlusher(plainFlusher{}).CheckpointLive(context.Background(), "r")
		if err != nil || !live {
			t.Errorf("CheckpointLive = %v, %v; want true, nil", live, err)
		}
	})
}
