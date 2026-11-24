package gcs

import (
	"context"
	"errors"
	"testing"

	"github.com/reearth/ygo/crdt"
)

// editPair returns two updates to one text: first inserts "a", second inserts
// "b" after it, so second cannot apply without first.
func editPair(t *testing.T) (first, second []byte) {
	t.Helper()
	a := crdt.New(crdt.WithClientID(1))
	txt := a.GetText("t")
	a.Transact(func(txn *crdt.Transaction) { txt.Insert(txn, 0, "a", nil) })
	first = crdt.EncodeStateAsUpdateV1(a, nil)
	b := crdt.New(crdt.WithClientID(2))
	if err := crdt.ApplyUpdateV1(b, first, nil); err != nil {
		t.Fatal(err)
	}
	sv := b.StateVector()
	btxt := b.GetText("t")
	b.Transact(func(txn *crdt.Transaction) { btxt.Insert(txn, 1, "b", nil) })
	return first, crdt.EncodeStateAsUpdateV1(b, sv)
}

// parkedState is the state of a copy that received second but missed first.
func parkedState(t *testing.T, second []byte) []byte {
	t.Helper()
	d := crdt.New()
	if err := crdt.ApplyUpdateV1(d, second, nil); err != nil {
		t.Fatal(err)
	}
	if d.PendingStats().Items == 0 {
		t.Fatal("setup: nothing parked")
	}
	return crdt.EncodeStateAsUpdateV1(d, nil)
}

func newParkedTestAdapter(t *testing.T) *Adapter {
	t.Helper()
	client, bucket := newFakeGCS(t)
	a, err := New(Options{Client: client, Bucket: bucket, Locker: NewNoLock()})
	if err != nil {
		t.Fatal(err)
	}
	return a
}

// TestLoad_UpdateLogCompletesAParkedDocV2: an instance that missed an edit
// another instance applied and persisted wrote the checkpoint, so doc_v2 holds
// the edit's dependents without the edit. The update log has the edit, and the
// room must open with the whole document.
func TestLoad_UpdateLogCompletesAParkedDocV2(t *testing.T) {
	t.Parallel()

	ctx := context.Background()
	a := newParkedTestAdapter(t)
	first, second := editPair(t)
	if _, err := a.AppendUpdate(ctx, "room", first); err != nil {
		t.Fatal(err)
	}
	if err := a.FlushSnapshot(ctx, "room", parkedState(t, second)); err != nil {
		t.Fatal(err)
	}

	lr, err := a.Load(ctx, "room")
	if err != nil {
		t.Fatalf("Load: %v", err)
	}
	d := crdt.New()
	if err := crdt.ApplyUpdateV1(d, lr.Update, nil); err != nil {
		t.Fatal(err)
	}
	if got := d.GetText("t").ToString(); got != "ab" || d.PendingStats().Items != 0 {
		t.Fatalf("loaded %q with %d parked, want \"ab\" and none", got, d.PendingStats().Items)
	}

	// Rebuilding the checkpoint from storage repairs doc_v2.
	if err := a.SnapshotFromStore(ctx, "room"); err != nil {
		t.Fatalf("SnapshotFromStore: %v", err)
	}
	if _, _, parked, err := a.loadPrimaryV2(ctx, "room"); err != nil || parked != 0 {
		t.Fatalf("doc_v2 after the rebuild: %d parked, %v", parked, err)
	}
}

// TestLoad_RefusesAParkedDocV2TheUpdateLogCannotComplete: when nothing stored
// supplies the missing edit, the room still refuses to open rather than serve a
// document missing content.
func TestLoad_RefusesAParkedDocV2TheUpdateLogCannotComplete(t *testing.T) {
	t.Parallel()

	ctx := context.Background()
	a := newParkedTestAdapter(t)
	_, second := editPair(t)
	if err := a.FlushSnapshot(ctx, "room", parkedState(t, second)); err != nil {
		t.Fatal(err)
	}
	if _, err := a.Load(ctx, "room"); !errors.Is(err, ErrPartialV2Decode) {
		t.Fatalf("Load = %v, want ErrPartialV2Decode", err)
	}
	if _, err := a.MaterializeAt(ctx, "room", 0); !errors.Is(err, ErrPartialV2Decode) {
		t.Fatalf("MaterializeAt = %v, want ErrPartialV2Decode", err)
	}
}
