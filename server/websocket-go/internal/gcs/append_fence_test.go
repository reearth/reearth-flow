package gcs

import (
	"bytes"
	"context"
	"testing"
	"time"

	"github.com/alicebob/miniredis/v2"
	goredis "github.com/redis/go-redis/v9"
)

// TestAppendUpdateDoesNotOverwriteConcurrentClock reproduces the lapsed-lock
// race: a second writer lands the same clock between our lastClock read and our
// put. The put must refuse to overwrite and retry at the next clock, so both
// updates survive.
func TestAppendUpdateDoesNotOverwriteConcurrentClock(t *testing.T) {
	client, bucket := newFakeGCS(t)
	ctx := context.Background()
	const room = "fence-room"

	a, err := New(Options{Client: client, Bucket: bucket, Locker: NewNoLock()})
	if err != nil {
		t.Fatalf("New a: %v", err)
	}
	b, err := New(Options{Client: client, Bucket: bucket, Locker: NewNoLock()})
	if err != nil {
		t.Fatalf("New b: %v", err)
	}

	updates, _ := genIncUpdates(t, 2)
	fired := false
	a.beforeUpdatePut = func(uint32) {
		if fired {
			return
		}
		fired = true
		if _, err := b.AppendUpdate(ctx, room, updates[1]); err != nil {
			t.Fatalf("racing AppendUpdate: %v", err)
		}
	}

	v, err := a.AppendUpdate(ctx, room, updates[0])
	if err != nil {
		t.Fatalf("AppendUpdate: %v", err)
	}
	if v != 2 {
		t.Fatalf("AppendUpdate version = %d, want 2 (retried past the racing clock)", v)
	}

	oid, err := a.oidFor(ctx, room)
	if err != nil {
		t.Fatalf("oidFor: %v", err)
	}
	for clock, want := range map[uint32][]byte{1: updates[1], 2: updates[0]} {
		got, err := a.store.get(ctx, a.layout.UpdateName(room, oid, clock))
		if err != nil {
			t.Fatalf("get clock %d: %v", clock, err)
		}
		if !bytes.Equal(got, want) {
			t.Fatalf("clock %d holds the wrong update; one writer's edit was overwritten", clock)
		}
	}
}

// TestRedisLockerRenewsWhileHeld proves a long critical section keeps its lock:
// without renewal the key would expire mid-section and a second instance could
// enter it.
func TestRedisLockerRenewsWhileHeld(t *testing.T) {
	mr, err := miniredis.Run()
	if err != nil {
		t.Fatalf("miniredis: %v", err)
	}
	t.Cleanup(mr.Close)
	rc := goredis.NewClient(&goredis.Options{Addr: mr.Addr()})
	t.Cleanup(func() { _ = rc.Close() })

	const ttl = 300 * time.Millisecond
	const key = "lock:renew"
	l := &RedisLocker{client: rc, value: "me", ttl: ttl, retries: 1, delay: 1}
	err = l.WithLock(context.Background(), key, func(context.Context) error {
		// Burn almost the whole TTL, then give the renewer time to tick.
		mr.FastForward(ttl - 50*time.Millisecond)
		time.Sleep(ttl/3 + 100*time.Millisecond)
		if !mr.Exists(key) {
			t.Fatal("lock expired while still held")
		}
		if got := mr.TTL(key); got <= 100*time.Millisecond {
			t.Fatalf("lock TTL = %v, want it renewed back toward %v", got, ttl)
		}
		return nil
	})
	if err != nil {
		t.Fatalf("WithLock: %v", err)
	}
	if mr.Exists(key) {
		t.Fatal("lock not released after WithLock returned")
	}
}

// TestRedisLockerRenewalStopsAfterLoss proves the renewer never extends a key
// another owner has taken over.
func TestRedisLockerRenewalStopsAfterLoss(t *testing.T) {
	mr, err := miniredis.Run()
	if err != nil {
		t.Fatalf("miniredis: %v", err)
	}
	t.Cleanup(mr.Close)
	rc := goredis.NewClient(&goredis.Options{Addr: mr.Addr()})
	t.Cleanup(func() { _ = rc.Close() })

	const ttl = 300 * time.Millisecond
	const key = "lock:lost"
	l := &RedisLocker{client: rc, value: "me", ttl: ttl, retries: 1, delay: 1}
	err = l.WithLock(context.Background(), key, func(context.Context) error {
		if err := mr.Set(key, "other"); err != nil {
			t.Fatalf("steal lock: %v", err)
		}
		mr.SetTTL(key, 10*time.Second)
		time.Sleep(ttl/3 + 100*time.Millisecond)
		if got := mr.TTL(key); got < 9*time.Second {
			t.Fatalf("foreign lock TTL = %v; the renewer extended a lock it no longer owns", got)
		}
		return nil
	})
	if err != nil {
		t.Fatalf("WithLock: %v", err)
	}
	if v, _ := mr.Get(key); v != "other" {
		t.Fatalf("lock value = %q, release must not delete a foreign owner's lock", v)
	}
}
