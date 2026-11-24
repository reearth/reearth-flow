package gcs

import (
	"context"
	"testing"

	"cloud.google.com/go/storage"
	"github.com/reearth/ygo/crdt"
)

// seedLegacy writes a legacy-root doc for room via a Phase-1 adapter and returns
// the final text.
func seedLegacy(t *testing.T, client *storage.Client, bucket, room string) string {
	t.Helper()
	p1, err := New(Options{Client: client, Bucket: bucket, Locker: NewNoLock()})
	if err != nil {
		t.Fatalf("New phase1: %v", err)
	}
	ctx := context.Background()
	doc := crdt.New(crdt.WithClientID(3))
	txt := doc.GetText("t")
	var prev []byte
	for i := 0; i < 4; i++ {
		ch := string(rune('a' + i))
		doc.Transact(func(txn *crdt.Transaction) { txt.Insert(txn, 0, ch, nil) })
		full := crdt.EncodeStateAsUpdateV1(doc, nil)
		var inc []byte
		if prev == nil {
			inc = full
		} else {
			d, _ := crdt.DiffUpdateV1(full, svOf(t, prev))
			inc = d
		}
		if _, err := p1.AppendUpdate(ctx, room, inc); err != nil {
			t.Fatalf("seed AppendUpdate: %v", err)
		}
		prev = full
	}
	return txt.ToString()
}

// TestPhase2DualReadFallsBackToLegacy: a Phase-2 adapter reads a room that only
// exists in the legacy root (no prefix objects yet) via the dual-read resolver.
func TestPhase2DualReadFallsBackToLegacy(t *testing.T) {
	client, bucket := newFakeGCS(t)
	const room = "proj-dual"
	want := seedLegacy(t, client, bucket, room)

	p2, err := New(Options{Client: client, Bucket: bucket, Locker: NewNoLock(), Phase2: true})
	if err != nil {
		t.Fatalf("New phase2: %v", err)
	}
	ctx := context.Background()
	lr, err := p2.Load(ctx, room)
	if err != nil {
		t.Fatalf("Load: %v", err)
	}
	rebuilt := crdt.New()
	if err := crdt.ApplyUpdateV1(rebuilt, lr.Update, nil); err != nil {
		t.Fatalf("ApplyUpdateV1: %v", err)
	}
	if got := rebuilt.GetText("t").ToString(); got != want {
		t.Fatalf("dual-read Load text = %q, want %q", got, want)
	}
}

// TestPhase2Backfill migrates legacy→prefix, verifies state-vector equality, and
// confirms the prefix is now authoritative (a primary-only read reproduces it).
func TestPhase2Backfill(t *testing.T) {
	client, bucket := newFakeGCS(t)
	const room = "proj-backfill"
	want := seedLegacy(t, client, bucket, room)

	p2, _ := New(Options{Client: client, Bucket: bucket, Locker: NewNoLock(), Phase2: true})
	ctx := context.Background()

	if err := p2.Backfill(ctx, room); err != nil {
		t.Fatalf("Backfill: %v", err)
	}
	// Prefix doc_v2 reproduces the state without the fallback.
	v1, ok, err := p2.loadPrimaryV2(ctx, room)
	if err != nil || !ok {
		t.Fatalf("loadPrimaryV2 ok=%v err=%v", ok, err)
	}
	rebuilt := crdt.New()
	if err := crdt.ApplyUpdateV1(rebuilt, v1, nil); err != nil {
		t.Fatalf("ApplyUpdateV1: %v", err)
	}
	if got := rebuilt.GetText("t").ToString(); got != want {
		t.Fatalf("post-backfill primary text = %q, want %q", got, want)
	}
	// Idempotent: a second backfill is a no-op.
	if err := p2.Backfill(ctx, room); err != nil {
		t.Fatalf("Backfill (2nd): %v", err)
	}
}

// TestPhase2RejectsMaliciousDocIDs: every Phase-2 method rejects a path-traversal
// doc id before using it as a prefix; Phase 1 (hex-encoded) does not reject them.
func TestPhase2RejectsMaliciousDocIDs(t *testing.T) {
	client, bucket := newFakeGCS(t)
	p2, _ := New(Options{Client: client, Bucket: bucket, Locker: NewNoLock(), Phase2: true})
	p1, _ := New(Options{Client: client, Bucket: bucket, Locker: NewNoLock()})
	ctx := context.Background()

	malicious := []string{"a/b", "a/../b", "..", ".", "a/", "/a", "a\x00b", "a\nb", "", "  "}
	for _, d := range malicious {
		if _, err := p2.Load(ctx, d); err == nil {
			t.Errorf("Phase2 Load(%q) = nil err, want rejection", d)
		}
		if _, err := p2.AppendUpdate(ctx, d, []byte{1}); err == nil {
			t.Errorf("Phase2 AppendUpdate(%q) = nil err, want rejection", d)
		}
		if err := p2.Backfill(ctx, d); err == nil {
			t.Errorf("Phase2 Backfill(%q) = nil err, want rejection", d)
		}
	}
	// "main" and a ':'-containing id are valid (no UUID gate).
	for _, d := range []string{"main", "project:main"} {
		if _, err := p2.AppendUpdate(ctx, d, []byte{1}); err != nil {
			t.Errorf("Phase2 AppendUpdate(%q) = %v, want nil (valid id)", d, err)
		}
	}
	// Phase 1 must not reject a '/'-containing id (it hex-encodes it).
	if _, err := p1.AppendUpdate(ctx, "a/b", []byte{1}); err != nil {
		t.Errorf("Phase1 AppendUpdate(%q) = %v, want nil (no stricter than Rust)", "a/b", err)
	}
}
