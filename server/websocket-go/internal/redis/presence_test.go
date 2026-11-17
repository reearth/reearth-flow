package redis

import (
	"context"
	"testing"
)

func TestActiveInstancesCountsInstancesHoldingTheRoom(t *testing.T) {
	t.Parallel()

	_, mr := newTestClient(t)
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	const room = "proj1"

	a, b := mustRelay(t, mr.Addr()), mustRelay(t, mr.Addr())
	defer a.Close()
	defer b.Close()
	for _, r := range []*Relay{a, b} {
		if err := r.Start(ctx, &fakeSink{}); err != nil {
			t.Fatalf("Start: %v", err)
		}
	}

	count := func() int {
		t.Helper()
		n, err := a.ActiveInstances(ctx, room)
		if err != nil {
			t.Fatalf("ActiveInstances: %v", err)
		}
		return n
	}
	if n := count(); n != 0 {
		t.Fatalf("before any instance holds the room: %d, want 0", n)
	}
	b.RoomActivated(room)
	if n := count(); n != 1 {
		t.Fatalf("held by another instance: %d, want 1", n)
	}
	a.RoomActivated(room)
	if n := count(); n != 2 {
		t.Fatalf("held by both instances: %d, want 2", n)
	}
}
