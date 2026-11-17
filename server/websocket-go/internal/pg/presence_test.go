package pg

import (
	"context"
	"testing"
)

func TestActiveInstancesCountsInstancesHoldingTheRoom(t *testing.T) {
	t.Parallel()

	pool := newPool(t)
	a := startRelay(t, pool, &fakeSink{}, nil)
	b := startRelay(t, pool, &fakeSink{}, nil)

	count := func() int {
		t.Helper()
		n, err := a.ActiveInstances(context.Background(), room)
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
