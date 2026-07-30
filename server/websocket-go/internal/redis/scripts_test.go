package redis

import (
	"context"
	"testing"
	"time"
)

func TestHeartbeatAndActiveCount(t *testing.T) {
	c, mr := newTestClient(t)
	ctx := context.Background()
	const doc = "proj1"

	if n, err := getActiveInstances(ctx, c, doc, 60); err != nil || n != 0 {
		t.Fatalf("active=%d err=%v, want 0", n, err)
	}

	if err := updateHeartbeat(ctx, c, doc, 111); err != nil {
		t.Fatalf("hb1: %v", err)
	}
	if err := updateHeartbeat(ctx, c, doc, 222); err != nil {
		t.Fatalf("hb2: %v", err)
	}
	if n, _ := getActiveInstances(ctx, c, doc, 60); n != 2 {
		t.Fatalf("active=%d, want 2", n)
	}

	if ttl := mr.TTL(instancesKey(doc)); ttl <= 0 || ttl > 120*time.Second {
		t.Fatalf("instances TTL = %v, want (0,120s]", ttl)
	}

	// An entry older than the timeout window is not counted active.
	mr.HSet(instancesKey(doc), "333", "0") // last_seen far in the past
	if n, _ := getActiveInstances(ctx, c, doc, 60); n != 2 {
		t.Fatalf("active=%d with one stale, want 2", n)
	}
}

func TestRemoveHeartbeat(t *testing.T) {
	c, mr := newTestClient(t)
	ctx := context.Background()
	const doc = "proj1"

	_ = updateHeartbeat(ctx, c, doc, 111)
	_ = updateHeartbeat(ctx, c, doc, 222)

	empty, err := removeHeartbeat(ctx, c, doc, 111)
	if err != nil || empty {
		t.Fatalf("removeHeartbeat(111) empty=%v err=%v, want false", empty, err)
	}
	if !mr.Exists(instancesKey(doc)) {
		t.Fatal("instances hash deleted prematurely")
	}

	empty, err = removeHeartbeat(ctx, c, doc, 222)
	if err != nil || !empty {
		t.Fatalf("removeHeartbeat(222) empty=%v err=%v, want true", empty, err)
	}
	if mr.Exists(instancesKey(doc)) {
		t.Fatal("instances hash not deleted when empty")
	}
}

func TestPublishEmptyMarkerSetsExpiry(t *testing.T) {
	c, mr := newTestClient(t)
	ctx := context.Background()
	const doc = "proj1"

	if err := publishEmptyMarker(ctx, c, doc, 111); err != nil {
		t.Fatalf("publishEmptyMarker: %v", err)
	}
	msgs, _ := c.XRange(ctx, streamKey(doc), "-", "+").Result()
	if len(msgs) != 1 {
		t.Fatalf("got %d entries, want 1", len(msgs))
	}
	if msgs[0].Values["type"] != "sync" || msgs[0].Values["data"] != "" {
		t.Fatalf("marker = %+v, want empty sync", msgs[0].Values)
	}
	ttl := mr.TTL(streamKey(doc))
	if ttl <= 0 || ttl > streamTTLSeconds*time.Second {
		t.Fatalf("stream TTL = %v, want (0,21600s]", ttl)
	}
}

func TestLockAcquireRelease(t *testing.T) {
	c, _ := newTestClient(t)
	ctx := context.Background()
	const key = "lock:doc:proj1"

	ok, err := acquireLock(ctx, c, key, "instance-1", 10)
	if err != nil || !ok {
		t.Fatalf("acquire = %v err=%v, want true", ok, err)
	}
	// Contended acquire fails (NX).
	if ok, _ := acquireLock(ctx, c, key, "instance-2", 10); ok {
		t.Fatal("second acquire succeeded, want false")
	}
	// Wrong owner cannot release (compare-and-del).
	if released, _ := releaseLock(ctx, c, key, "instance-2"); released {
		t.Fatal("released with wrong owner")
	}
	if released, err := releaseLock(ctx, c, key, "instance-1"); err != nil || !released {
		t.Fatalf("release = %v err=%v, want true", released, err)
	}
	if ok, _ := acquireLock(ctx, c, key, "instance-2", 10); !ok {
		t.Fatal("re-acquire after release failed")
	}
}

func TestSafeDeleteStream(t *testing.T) {
	ctx := context.Background()
	const doc = "proj1"

	t.Run("deletes when no active instances and no read lock", func(t *testing.T) {
		c, mr := newTestClient(t)
		_ = publishEmptyMarker(ctx, c, doc, 1)
		if err := safeDeleteStream(ctx, c, doc, "instance-1"); err != nil {
			t.Fatalf("safeDeleteStream: %v", err)
		}
		if mr.Exists(streamKey(doc)) {
			t.Fatal("stream not deleted")
		}
	})

	t.Run("bails when read lock present", func(t *testing.T) {
		c, mr := newTestClient(t)
		_ = publishEmptyMarker(ctx, c, doc, 1)
		_ = mr.Set(readLockKey(doc), "held")
		if err := safeDeleteStream(ctx, c, doc, "instance-1"); err != nil {
			t.Fatalf("safeDeleteStream: %v", err)
		}
		if !mr.Exists(streamKey(doc)) {
			t.Fatal("stream deleted despite read lock")
		}
	})

	t.Run("bails when an instance is still active", func(t *testing.T) {
		c, mr := newTestClient(t)
		_ = publishEmptyMarker(ctx, c, doc, 1)
		_ = updateHeartbeat(ctx, c, doc, 777)
		if err := safeDeleteStream(ctx, c, doc, "instance-1"); err != nil {
			t.Fatalf("safeDeleteStream: %v", err)
		}
		if !mr.Exists(streamKey(doc)) {
			t.Fatal("stream deleted despite an active instance")
		}
	})
}
