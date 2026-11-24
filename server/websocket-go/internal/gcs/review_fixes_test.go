package gcs

import (
	"context"
	"testing"

	"github.com/reearth/ygo/crdt"
	"github.com/reearth/ygo/persistence"
)

// TestLoadClampsHeadToCeiling reproduces the stale-head bug: a PruneAfter that
// crashes between writing the ceiling (2a) and lowering the checkpoint (2b)
// leaves ceiling=target with a stale checkpoint > target. Load must report a
// Version clamped to the ceiling, not the stale checkpoint.
func TestLoadClampsHeadToCeiling(t *testing.T) {
	client, bucket := newFakeGCS(t)
	a, err := New(Options{Client: client, Bucket: bucket, Locker: NewNoLock()})
	if err != nil {
		t.Fatalf("New: %v", err)
	}
	ctx := context.Background()
	const room = "clamp-room"

	updates, _ := genIncUpdates(t, 10) // clocks 1..10 → compaction sets checkpoint=10
	for _, u := range updates {
		if _, err := a.AppendUpdate(ctx, room, u); err != nil {
			t.Fatalf("AppendUpdate: %v", err)
		}
	}
	if cp, err := a.checkpoint(ctx, room); err != nil || cp != 10 {
		t.Fatalf("checkpoint = %d, err=%v; want 10 (compaction did not run)", cp, err)
	}

	// Simulate the crash window: ceiling=5 is durable but the checkpoint is still
	// the stale 10 (2b never ran).
	if err := a.store.put(ctx, a.ceilingName(room), be32(5)); err != nil {
		t.Fatalf("seed ceiling: %v", err)
	}

	lr, err := a.Load(ctx, room)
	if err != nil {
		t.Fatalf("Load: %v", err)
	}
	if lr.Version > 5 {
		t.Fatalf("Load Version = %d, want <= 5 (clamped to ceiling); stale checkpoint leaked", lr.Version)
	}
}

// TestMaterializeAtLegacyOnlyPhase2 reproduces the critical rollback-destroys-doc
// bug (C2): MaterializeAt gated the base on cp>0, so a legacy-only Phase-2 room
// (checkpoint==0) materialized empty and a rollback wrote an empty doc_v2 over
// real content. MaterializeAt must reconstruct the dual-read base.
func TestMaterializeAtLegacyOnlyPhase2(t *testing.T) {
	client, bucket := newFakeGCS(t)
	const room = "proj-materialize-legacy"
	want := seedLegacy(t, client, bucket, room)

	p2, err := New(Options{Client: client, Bucket: bucket, Locker: NewNoLock(), Phase2: true})
	if err != nil {
		t.Fatalf("New phase2: %v", err)
	}
	ctx := context.Background()

	versions, err := p2.ListVersions(ctx, room)
	_ = versions
	if err != nil {
		t.Fatalf("ListVersions: %v", err)
	}
	// Materialize at the legacy head (dual-read reports it as the Version).
	lr, err := p2.Load(ctx, room)
	if err != nil {
		t.Fatalf("Load: %v", err)
	}
	state, err := p2.MaterializeAt(ctx, room, lr.Version)
	if err != nil {
		t.Fatalf("MaterializeAt: %v", err)
	}
	if len(state) == 0 {
		t.Fatalf("MaterializeAt returned empty for a legacy-only room; rollback would destroy it")
	}
	rebuilt := crdt.New()
	if err := crdt.ApplyUpdateV1(rebuilt, state, nil); err != nil {
		t.Fatalf("ApplyUpdateV1: %v", err)
	}
	if got := rebuilt.GetText("t").ToString(); got != want {
		t.Fatalf("materialized text = %q, want %q", got, want)
	}
}

// TestAppendUpdateFoldsLegacyBasePhase2 reproduces the critical split-brain (C1):
// the first edit to a legacy-only Phase-2 room was written to the primary prefix
// without folding the legacy base, so a cold reload lost all prior content. After
// an append, a fresh Phase-2 adapter must still see the legacy text plus the edit.
func TestAppendUpdateFoldsLegacyBasePhase2(t *testing.T) {
	client, bucket := newFakeGCS(t)
	const room = "proj-splitbrain"
	want := seedLegacy(t, client, bucket, room) // "dcba"

	p2, err := New(Options{Client: client, Bucket: bucket, Locker: NewNoLock(), Phase2: true})
	if err != nil {
		t.Fatalf("New phase2: %v", err)
	}
	ctx := context.Background()

	// Load the legacy state, apply one incremental edit, append it (mirrors the
	// ygo persistence worker's StoreUpdate on the first client edit).
	lr, err := p2.Load(ctx, room)
	if err != nil {
		t.Fatalf("Load: %v", err)
	}
	doc := crdt.New()
	if err := crdt.ApplyUpdateV1(doc, lr.Update, nil); err != nil {
		t.Fatalf("seed live doc: %v", err)
	}
	before := crdt.EncodeStateAsUpdateV1(doc, nil)
	txt := doc.GetText("t")
	doc.Transact(func(txn *crdt.Transaction) { txt.Insert(txn, 0, "Z", nil) })
	full := crdt.EncodeStateAsUpdateV1(doc, nil)
	inc, err := crdt.DiffUpdateV1(full, svOf(t, before))
	if err != nil {
		t.Fatalf("DiffUpdateV1: %v", err)
	}
	if _, err := p2.AppendUpdate(ctx, room, inc); err != nil {
		t.Fatalf("AppendUpdate: %v", err)
	}

	// Cold reload with a fresh Phase-2 adapter (simulates a restart / new reader).
	fresh, err := New(Options{Client: client, Bucket: bucket, Locker: NewNoLock(), Phase2: true})
	if err != nil {
		t.Fatalf("New phase2 (fresh): %v", err)
	}
	lr2, err := fresh.Load(ctx, room)
	if err != nil {
		t.Fatalf("Load (fresh): %v", err)
	}
	rebuilt := crdt.New()
	if err := crdt.ApplyUpdateV1(rebuilt, lr2.Update, nil); err != nil {
		t.Fatalf("ApplyUpdateV1: %v", err)
	}
	got := rebuilt.GetText("t").ToString()
	if got != "Z"+want {
		t.Fatalf("post-edit reload text = %q, want %q (legacy base lost → split-brain)", got, "Z"+want)
	}
}

// TestPhase2DeleteRemovesLegacyObjects reproduces H1: Phase-2 Delete only swept
// the {room}/ prefix, leaving legacy-root objects that resurrect via dual-read on
// the next Load. After Delete, a Load must not return the old content.
func TestPhase2DeleteRemovesLegacyObjects(t *testing.T) {
	client, bucket := newFakeGCS(t)
	const room = "proj-delete-legacy"
	_ = seedLegacy(t, client, bucket, room)

	p2, err := New(Options{Client: client, Bucket: bucket, Locker: NewNoLock(), Phase2: true})
	if err != nil {
		t.Fatalf("New phase2: %v", err)
	}
	ctx := context.Background()

	if err := p2.Delete(ctx, room); err != nil {
		t.Fatalf("Delete: %v", err)
	}
	lr, err := p2.Load(ctx, room)
	if err != nil {
		t.Fatalf("Load after delete: %v", err)
	}
	if len(lr.Update) != 0 {
		rebuilt := crdt.New()
		_ = crdt.ApplyUpdateV1(rebuilt, lr.Update, nil)
		t.Fatalf("legacy content resurrected after Delete: text=%q", rebuilt.GetText("t").ToString())
	}
}

// TestFlushStripsTransientMetadata reproduces M4: a snapshot captured while
// metadata.rollbackInProgress is set must not persist that transient flag into
// doc_v2 (the Rust reader loads doc_v2 alone and would see a stuck flag). Content
// must survive; only the configured transient key is stripped.
func TestFlushStripsTransientMetadata(t *testing.T) {
	client, bucket := newFakeGCS(t)
	a, err := New(Options{
		Client: client, Bucket: bucket, Locker: NewNoLock(),
		TransientMapKeys: map[string][]string{"metadata": {"rollbackInProgress"}},
	})
	if err != nil {
		t.Fatalf("New: %v", err)
	}
	ctx := context.Background()
	const room = "proj-transient"

	doc := crdt.New(crdt.WithClientID(9))
	txt := doc.GetText("t")
	m := doc.GetMap("metadata")
	doc.Transact(func(txn *crdt.Transaction) {
		txt.Insert(txn, 0, "hello", nil)
		m.Set(txn, "rollbackInProgress", true)
	})
	state := crdt.EncodeStateAsUpdateV1(doc, nil)

	if err := a.FlushSnapshot(ctx, room, state); err != nil {
		t.Fatalf("FlushSnapshot: %v", err)
	}

	// Read doc_v2 alone (as the Rust reader does) and confirm the flag is gone.
	v1, ok, err := a.loadV2(ctx, room)
	if err != nil || !ok {
		t.Fatalf("loadV2 ok=%v err=%v", ok, err)
	}
	rebuilt := crdt.New()
	if err := crdt.ApplyUpdateV1(rebuilt, v1, nil); err != nil {
		t.Fatalf("ApplyUpdateV1: %v", err)
	}
	if rebuilt.GetMap("metadata").Has("rollbackInProgress") {
		t.Fatalf("transient flag rollbackInProgress persisted into doc_v2")
	}
	if got := rebuilt.GetText("t").ToString(); got != "hello" {
		t.Fatalf("content = %q, want hello (strip damaged real content)", got)
	}
}

var _ = persistence.Version(0)
