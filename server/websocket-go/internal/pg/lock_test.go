package pg

import (
	"context"
	"errors"
	"sync"
	"testing"
	"time"
)

func TestWithLockIsMutuallyExclusive(t *testing.T) {
	pool := newPool(t)
	a := NewLocker(pool, "owner-a")
	b := NewLocker(pool, "owner-b")

	// b retries fast so the test does not wait the production 5s budget.
	b.retries, b.delay = 2, 10*time.Millisecond

	release := make(chan struct{})
	var inside sync.WaitGroup
	inside.Add(1)

	go func() {
		_ = a.WithLock(context.Background(), "k", func(context.Context) error {
			inside.Done()
			<-release
			return nil
		})
	}()
	inside.Wait()

	err := b.WithLock(context.Background(), "k", func(context.Context) error {
		t.Error("b entered the critical section while a held the lock")
		return nil
	})
	if !errors.Is(err, ErrLockTimeout) {
		t.Errorf("contended WithLock = %v, want ErrLockTimeout", err)
	}

	close(release)

	// Once a releases, b must get in — a lock that stayed held would wedge the
	// connect path for the rest of the process's life.
	eventually(t, 2*time.Second, "the lock to become available", func() bool {
		return b.WithLock(context.Background(), "k", func(context.Context) error { return nil }) == nil
	})
}

// TestExpiredLockIsTakenOver: the TTL is the only recovery path when a holder dies
// mid-critical-section. Without it a crashed instance would block the document
// forever.
func TestExpiredLockIsTakenOver(t *testing.T) {
	pool := newPool(t)
	dead := NewLocker(pool, "owner-dead")
	live := NewLocker(pool, "owner-live")

	// Simulate a holder that died: lock taken with a TTL that has already lapsed.
	held, err := dead.tryAcquire(context.Background(), "k", -time.Second)
	if err != nil || !held {
		t.Fatalf("seed expired lock: held=%v err=%v", held, err)
	}

	got, err := live.tryAcquire(context.Background(), "k", time.Minute)
	if err != nil {
		t.Fatalf("tryAcquire: %v", err)
	}
	if !got {
		t.Error("an expired lock was not taken over")
	}
}

// TestTryWithLockRunsEvenWhenHeld: the flusher's read lock is a fence, not mutual
// exclusion. Blocking the flush when the lock is unavailable would mean a failed
// lock acquisition silently stops persisting the document.
func TestTryWithLockRunsEvenWhenHeld(t *testing.T) {
	pool := newPool(t)
	holder := NewLocker(pool, "owner-holder")
	other := NewLocker(pool, "owner-other")

	held, err := holder.tryAcquire(context.Background(), "k", time.Minute)
	if err != nil || !held {
		t.Fatalf("seed lock: held=%v err=%v", held, err)
	}

	var ran bool
	if err := other.TryWithLock(context.Background(), "k", time.Minute, func(context.Context) error {
		ran = true
		return nil
	}); err != nil {
		t.Errorf("TryWithLock = %v, want nil", err)
	}
	if !ran {
		t.Error("TryWithLock skipped fn when the lock was held; the flush would not happen")
	}

	// And it must not have released a lock it never acquired.
	var owner string
	if err := pool.QueryRow(context.Background(),
		`SELECT owner FROM ws_lock WHERE key = 'k'`).Scan(&owner); err != nil {
		t.Fatalf("read lock row: %v", err)
	}
	if owner != "owner-holder" {
		t.Errorf("owner = %q, want owner-holder; a foreign lock was released", owner)
	}
}

// TestReleaseOnlyOwnLock: if the owner predicate were dropped, a holder whose TTL
// lapsed mid-work would delete the lock its successor legitimately holds, and two
// writers would run the critical section at once.
func TestReleaseOnlyOwnLock(t *testing.T) {
	pool := newPool(t)
	first := NewLocker(pool, "owner-first")
	second := NewLocker(pool, "owner-second")

	if _, err := second.tryAcquire(context.Background(), "k", time.Minute); err != nil {
		t.Fatalf("seed: %v", err)
	}
	first.release("k") // first never held it

	var n int
	if err := pool.QueryRow(context.Background(),
		`SELECT count(*) FROM ws_lock WHERE key = 'k' AND owner = 'owner-second'`).Scan(&n); err != nil {
		t.Fatalf("count: %v", err)
	}
	if n != 1 {
		t.Error("a non-owner released someone else's lock")
	}
}

func TestPingerReportsLiveness(t *testing.T) {
	pool := newPool(t)
	if err := NewPinger(pool).Ping(context.Background()); err != nil {
		t.Errorf("Ping = %v, want nil", err)
	}
}

// TestWithLockRenewsWhileHeld proves a critical section longer than the lease
// keeps its exclusion: without renewal the row would expire mid-section and a
// second instance could enter it behind the first.
func TestWithLockRenewsWhileHeld(t *testing.T) {
	pool := newPool(t)
	holder := NewLocker(pool, "owner-holder")
	other := NewLocker(pool, "owner-other")
	other.retries, other.delay = 1, time.Millisecond

	// A lease far shorter than the critical section, so only renewal can keep it.
	const ttl = 300 * time.Millisecond
	holder.ttl = ttl

	entered := make(chan struct{})
	release := make(chan struct{})
	errs := make(chan error, 1)
	go func() {
		errs <- holder.WithLock(context.Background(), "k", func(context.Context) error {
			close(entered)
			<-release
			return nil
		})
	}()
	<-entered

	// Outlive the original lease, then check nobody can take the key over.
	time.Sleep(2 * ttl)
	err := other.WithLock(context.Background(), "k", func(context.Context) error {
		t.Error("a second owner entered the critical section; the lease was not renewed")
		return nil
	})
	if !errors.Is(err, ErrLockTimeout) {
		t.Errorf("contended WithLock = %v, want ErrLockTimeout", err)
	}

	close(release)
	if err := <-errs; err != nil {
		t.Fatalf("WithLock: %v", err)
	}
}

// TestRenewalStopsAfterLoss proves the renewer never pushes out a lease another
// owner has taken over, which would let two writers run the section at once.
func TestRenewalStopsAfterLoss(t *testing.T) {
	pool := newPool(t)
	lost := NewLocker(pool, "owner-lost")
	lost.ttl = 150 * time.Millisecond

	ctx := context.Background()
	stop := lost.renew("k")
	defer stop()

	// Someone else owns the key with a long lease.
	successor := NewLocker(pool, "owner-successor")
	if ok, err := successor.tryAcquire(ctx, "k", time.Hour); err != nil || !ok {
		t.Fatalf("seed successor lock: ok=%v err=%v", ok, err)
	}

	time.Sleep(3 * lost.ttl)

	var owner string
	var remaining time.Duration
	if err := pool.QueryRow(ctx,
		`SELECT owner, expires_at - now() FROM ws_lock WHERE key = 'k'`).Scan(&owner, &remaining); err != nil {
		t.Fatalf("read lock row: %v", err)
	}
	if owner != "owner-successor" {
		t.Fatalf("owner = %q, want owner-successor", owner)
	}
	if remaining < 30*time.Minute {
		t.Errorf("successor lease has %v left; the previous owner's renewer shortened it", remaining)
	}
}
