package gcs

import (
	"bytes"
	"context"
	"encoding/binary"
	"log/slog"
	"strings"
	"testing"

	"github.com/alicebob/miniredis/v2"
	goredis "github.com/redis/go-redis/v9"
	"github.com/reearth/ygo/crdt"
	"github.com/reearth/ygo/persistence"
)

// TestLoadLogsErrorWithRoom: when a room load fails, the cause is logged at
// ERROR with the room id. The WebSocket upgrade path (ygo getOrCreateRoom →
// LoadDoc) discards this error behind a bare "500 room unavailable", so logging
// it at the source is the only way to see why a connect 500s.
func TestLoadLogsErrorWithRoom(t *testing.T) {
	client, bucket := newFakeGCS(t)
	var buf bytes.Buffer
	log := slog.New(slog.NewJSONHandler(&buf, &slog.HandlerOptions{Level: slog.LevelDebug}))
	a, err := New(Options{Client: client, Bucket: bucket, Locker: NewNoLock(), Phase2: true, Logger: log})
	if err != nil {
		t.Fatalf("New: %v", err)
	}
	// A '/' in the doc id fails ValidateDocIDForPrefix in Phase 2: a deterministic
	// Load error that does not need a broken GCS backend.
	if _, err := a.Load(context.Background(), "bad/id"); err == nil {
		t.Fatal("expected error for unsafe doc id")
	}
	out := buf.String()
	if !strings.Contains(out, "bad/id") {
		t.Fatalf("room id not logged:\n%s", out)
	}
	if !strings.Contains(out, `"level":"ERROR"`) {
		t.Fatalf("load failure not logged at ERROR:\n%s", out)
	}
}

// TestBrotliRoundTrip proves the v2 snapshot survives a brotli
// compress → decompress → V2-decode cycle.
func TestBrotliRoundTrip(t *testing.T) {
	doc := crdt.New(crdt.WithClientID(7))
	txt := doc.GetText("t")
	doc.Transact(func(txn *crdt.Transaction) { txt.Insert(txn, 0, "hello", nil) })
	v2 := crdt.EncodeStateAsUpdateV2(doc, nil)

	comp := compressBrotli(v2)
	if len(comp) == 0 {
		t.Fatal("empty brotli output")
	}
	back, err := decompressBrotli(comp)
	if err != nil {
		t.Fatalf("decompressBrotli: %v", err)
	}
	got := crdt.New()
	if err := crdt.ApplyUpdateV2(got, back, nil); err != nil {
		t.Fatalf("ApplyUpdateV2: %v", err)
	}
	if s := got.GetText("t").ToString(); s != "hello" {
		t.Fatalf("round-tripped text = %q, want %q", s, "hello")
	}
}

// TestLoadReadsV2Snapshot proves Load reconstructs state via the brotli(V2)
// doc_v2 path after PruneAfter writes the snapshot.
func TestLoadReadsV2Snapshot(t *testing.T) {
	client, bucket := newFakeGCS(t)
	a, err := New(Options{Client: client, Bucket: bucket, Locker: NewNoLock()})
	if err != nil {
		t.Fatalf("New: %v", err)
	}
	ctx := context.Background()
	doc := crdt.New(crdt.WithClientID(9))
	txt := doc.GetText("t")
	doc.Transact(func(txn *crdt.Transaction) { txt.Insert(txn, 0, "world", nil) })
	state := crdt.EncodeStateAsUpdateV1(doc, nil)

	if err := a.PruneAfter(ctx, "room", 0, state); err != nil {
		t.Fatalf("PruneAfter: %v", err)
	}
	raw, err := a.store.get(ctx, a.layout.DocV2Name("room"))
	if err != nil {
		t.Fatalf("get doc_v2: %v", err)
	}
	if _, err := decompressBrotli(raw); err != nil {
		t.Fatalf("doc_v2 is not valid brotli: %v", err)
	}
	lr, err := a.Load(ctx, "room")
	if err != nil {
		t.Fatalf("Load: %v", err)
	}
	rebuilt := crdt.New()
	if err := crdt.ApplyUpdateV1(rebuilt, lr.Update, nil); err != nil {
		t.Fatalf("ApplyUpdateV1: %v", err)
	}
	if s := rebuilt.GetText("t").ToString(); s != "world" {
		t.Fatalf("Load text = %q, want %q", s, "world")
	}
}

// TestOIDAllocationUnderRedisLock exercises Phase-1 monotonic OID allocation from
// system:last_oid under the global lock: distinct rooms get distinct OIDs.
func TestOIDAllocationUnderRedisLock(t *testing.T) {
	mr, err := miniredis.Run()
	if err != nil {
		t.Fatalf("miniredis: %v", err)
	}
	t.Cleanup(mr.Close)
	rc := goredis.NewClient(&goredis.Options{Addr: mr.Addr()})
	t.Cleanup(func() { _ = rc.Close() })

	client, bucket := newFakeGCS(t)
	a, err := New(Options{Client: client, Bucket: bucket, Locker: NewRedisLocker(rc, "instance-test")})
	if err != nil {
		t.Fatalf("New: %v", err)
	}
	ctx := context.Background()

	oid1, err := a.oidFor(ctx, "projA")
	if err != nil {
		t.Fatalf("oidFor projA: %v", err)
	}
	oid2, err := a.oidFor(ctx, "projB")
	if err != nil {
		t.Fatalf("oidFor projB: %v", err)
	}
	if oid1 != 1 || oid2 != 2 {
		t.Fatalf("OIDs = %d,%d, want monotonic 1,2", oid1, oid2)
	}
	again, _ := a.oidFor(ctx, "projA")
	if again != oid1 {
		t.Fatalf("oidFor(projA) re-read = %d, want %d", again, oid1)
	}
	last, gerr := a.store.get(ctx, SystemLastOIDName())
	if gerr != nil {
		t.Fatalf("get system:last_oid: %v", gerr)
	}
	if got := binary.BigEndian.Uint32(last); got != 2 {
		t.Fatalf("system:last_oid = %d, want 2", got)
	}
}

// TestRedisLockerBounded proves OID lock acquisition is bounded when the lock is
// already held by another owner.
func TestRedisLockerBounded(t *testing.T) {
	mr, err := miniredis.Run()
	if err != nil {
		t.Fatalf("miniredis: %v", err)
	}
	t.Cleanup(mr.Close)
	rc := goredis.NewClient(&goredis.Options{Addr: mr.Addr()})
	t.Cleanup(func() { _ = rc.Close() })

	if err := rc.Set(context.Background(), oidGenerationLockKey, "other-owner", 0).Err(); err != nil {
		t.Fatalf("pre-set lock: %v", err)
	}
	l := &RedisLocker{client: rc, value: "me", ttl: oidLockTTL, retries: 2, delay: 1}
	err = l.WithLock(context.Background(), oidGenerationLockKey, func(context.Context) error {
		t.Fatal("should not have acquired a held lock")
		return nil
	})
	if err != ErrLockTimeout {
		t.Fatalf("WithLock = %v, want ErrLockTimeout", err)
	}
}

var _ = persistence.Version(0)
