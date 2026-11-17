package versions

import (
	"context"
	"errors"
	"slices"
	"sync"
	"testing"
	"time"
)

// countingStore is a MemoryStore that counts Mark calls and can fail them.
type countingStore struct {
	*MemoryStore
	mu    sync.Mutex
	marks int
	fail  int // how many of the next Marks fail
}

func newCountingStore() *countingStore { return &countingStore{MemoryStore: NewMemoryStore()} }

func (s *countingStore) Mark(ctx context.Context, room string, m Mark) error {
	s.mu.Lock()
	s.marks++
	if s.fail > 0 {
		s.fail--
		s.mu.Unlock()
		return errors.New("store down")
	}
	s.mu.Unlock()
	return s.MemoryStore.Mark(ctx, room, m)
}

func (s *countingStore) markCount() int {
	s.mu.Lock()
	defer s.mu.Unlock()
	return s.marks
}

func waitUntil(t *testing.T, what string, cond func() bool) {
	t.Helper()
	deadline := time.Now().Add(3 * time.Second)
	for !cond() {
		if time.Now().After(deadline) {
			t.Fatalf("timed out waiting for %s", what)
		}
		time.Sleep(5 * time.Millisecond)
	}
}

func TestTracker_FirstEditReachesTheStoreAtOnce(t *testing.T) {
	t.Parallel()

	store := newCountingStore()
	tr := NewTracker(store, time.Hour, nil)
	tr.Edit("room", "u1")
	waitUntil(t, "the first edit to be written", func() bool {
		_, ok, _ := store.Peek(context.Background(), "room")
		return ok
	})
}

func TestTracker_GathersEditsWithinTheInterval(t *testing.T) {
	t.Parallel()

	store := newCountingStore()
	tr := NewTracker(store, 50*time.Millisecond, nil)
	tr.Edit("room", "u1")
	waitUntil(t, "the first write", func() bool { return store.markCount() == 1 })
	for range 100 {
		tr.Edit("room", "u2")
	}
	tr.Edit("room", "u3")
	waitUntil(t, "the gathered write", func() bool { return store.markCount() == 2 })
	time.Sleep(120 * time.Millisecond)
	if n := store.markCount(); n != 2 {
		t.Fatalf("marks = %d, want 2: 101 edits within one interval are one write", n)
	}
	r, _, _ := store.Peek(context.Background(), "room")
	if !slices.Equal(r.Contributors, []string{"u1", "u2", "u3"}) || r.LastEditor != "u3" {
		t.Fatalf("record = %+v", r)
	}
}

func TestTracker_FlushWritesGatheredEditsNow(t *testing.T) {
	t.Parallel()

	store := newCountingStore()
	tr := NewTracker(store, time.Hour, nil)
	tr.Edit("room", "u1")
	tr.Edit("room", "u2") // held back for an hour
	if err := tr.Flush(context.Background(), "room"); err != nil {
		t.Fatalf("Flush: %v", err)
	}
	r, _, _ := store.Peek(context.Background(), "room")
	if !slices.Contains(r.Contributors, "u2") {
		t.Fatalf("contributors = %v, want u2 written by Flush", r.Contributors)
	}
}

func TestTracker_FailedWriteIsRetried(t *testing.T) {
	t.Parallel()

	store := newCountingStore()
	store.fail = 1
	tr := NewTracker(store, 20*time.Millisecond, nil)
	tr.Edit("room", "u1")
	waitUntil(t, "the retried write", func() bool {
		r, ok, _ := store.Peek(context.Background(), "room")
		return ok && slices.Equal(r.Contributors, []string{"u1"})
	})
}

func TestTracker_EditAfterTakeIsRecordedAgain(t *testing.T) {
	t.Parallel()

	// Another instance takes the record between this instance's writes: the
	// edits after that must still reach the store.
	ctx := context.Background()
	store := newCountingStore()
	tr := NewTracker(store, 20*time.Millisecond, nil)
	tr.Edit("room", "u1")
	waitUntil(t, "the first write", func() bool { return store.markCount() == 1 })
	if _, ok, _ := store.Take(ctx, "room"); !ok {
		t.Fatal("no record to take")
	}
	tr.Edit("room", "u1")
	waitUntil(t, "the edit after the take", func() bool {
		_, ok, _ := store.Peek(ctx, "room")
		return ok
	})
}

func TestTracker_ForgetsAQuietRoom(t *testing.T) {
	t.Parallel()

	store := newCountingStore()
	tr := NewTracker(store, 10*time.Millisecond, nil)
	tr.Edit("room", "u1")
	waitUntil(t, "the room to be forgotten", func() bool {
		tr.mu.Lock()
		defer tr.mu.Unlock()
		return len(tr.rooms) == 0
	})
}
