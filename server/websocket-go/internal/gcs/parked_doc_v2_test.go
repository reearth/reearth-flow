package gcs

import (
	"context"
	"fmt"
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

// loaded decodes a V1 state and returns the text named name with the number of
// parked items.
func loaded(t *testing.T, state []byte, name string) (string, int) {
	t.Helper()
	d, err := docFromV1(state)
	if err != nil {
		t.Fatal(err)
	}
	return d.GetText(name).ToString(), d.PendingStats().Items
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
	if got, parked := loaded(t, lr.Update, "t"); got != "ab" || parked != 0 {
		t.Fatalf("loaded %q with %d parked, want \"ab\" and none", got, parked)
	}

	// Rebuilding the checkpoint from storage repairs doc_v2.
	if err := a.SnapshotFromStore(ctx, "room"); err != nil {
		t.Fatalf("SnapshotFromStore: %v", err)
	}
	v1, _, err := a.loadPrimaryV2(ctx, "room")
	if err != nil {
		t.Fatal(err)
	}
	if got, parked := loaded(t, v1, "t"); got != "ab" || parked != 0 {
		t.Fatalf("doc_v2 after the rebuild: %q with %d parked", got, parked)
	}
}

// TestLoad_ServesAParkedDocV2TheUpdateLogCannotComplete: when nothing stored
// supplies the missing edit, the room still opens, and the parked items survive
// every rewrite of doc_v2 until the edit arrives.
func TestLoad_ServesAParkedDocV2TheUpdateLogCannotComplete(t *testing.T) {
	t.Parallel()

	ctx := context.Background()
	a := newParkedTestAdapter(t)
	first, second := editPair(t)
	if err := a.FlushSnapshot(ctx, "room", parkedState(t, second)); err != nil {
		t.Fatal(err)
	}
	stillParked := func(step string) []byte {
		t.Helper()
		lr, err := a.Load(ctx, "room")
		if err != nil {
			t.Fatalf("%s: Load: %v", step, err)
		}
		if _, parked := loaded(t, lr.Update, "t"); parked == 0 {
			t.Fatalf("%s: the parked items are gone", step)
		}
		return lr.Update
	}

	state := stillParked("first load")
	if err := a.FlushSnapshot(ctx, "room", state); err != nil {
		t.Fatal(err)
	}
	stillParked("after a checkpoint of the loaded state")
	if _, err := a.Compact(ctx, "room", 0); err != nil {
		t.Fatalf("Compact: %v", err)
	}
	stillParked("after Compact")
	if m, err := a.MaterializeAt(ctx, "room", 0); err != nil {
		t.Fatalf("MaterializeAt: %v", err)
	} else if _, parked := loaded(t, m, "t"); parked == 0 {
		t.Fatal("MaterializeAt dropped the parked items")
	}

	// The missing edit arrives (a browser that still holds it reconnects).
	if _, err := a.AppendUpdate(ctx, "room", first); err != nil {
		t.Fatal(err)
	}
	lr, err := a.Load(ctx, "room")
	if err != nil {
		t.Fatal(err)
	}
	if got, parked := loaded(t, lr.Update, "t"); got != "ab" || parked != 0 {
		t.Fatalf("after the missing edit arrived: %q with %d parked, want \"ab\" and none", got, parked)
	}
}

// appendEdits returns n incremental updates from one client, each appending to
// text "content", so every update builds on the one before it.
func appendEdits(t *testing.T, n int) [][]byte {
	t.Helper()
	src := crdt.New(crdt.WithClientID(101))
	txt := src.GetText("content")
	var ups [][]byte
	unsub := src.OnUpdate(func(u []byte, _ any) { ups = append(ups, append([]byte(nil), u...)) })
	for i := 0; i < n; i++ {
		src.Transact(func(txn *crdt.Transaction) { txt.Insert(txn, txt.Len(), fmt.Sprintf("a%d ", i+1), nil) })
	}
	unsub()
	return ups
}

// TestLoad_OpensProjectsAlreadyBrokenByAParkedCheckpoint covers the doc_v2 an
// earlier build wrote from a copy that missed one edit: every case opens, and
// completely whenever the missing edit still exists as an update object.
func TestLoad_OpensProjectsAlreadyBrokenByAParkedCheckpoint(t *testing.T) {
	t.Parallel()

	const n = 25
	cases := []struct {
		name string
		skip int // index of the edit the copy missed; its clock is skip+1
		// compactBefore runs an unload compaction (keep=0, as ygo does) before
		// the bad checkpoint, which folds and deletes every update object.
		compactBefore bool
		// compactAfter runs it after the bad checkpoint.
		compactAfter bool
		complete     bool
	}{
		{name: "missing edit above the checkpoint", skip: 23, complete: true},
		{name: "missing edit at or below the checkpoint, object still stored", skip: 17, complete: true},
		{name: "missing edit below the checkpoint, unload compaction afterwards", skip: 17, compactAfter: true, complete: true},
		{name: "missing edit folded and deleted before the bad checkpoint", skip: 23, compactBefore: true},
	}
	for _, c := range cases {
		t.Run(c.name, func(t *testing.T) {
			t.Parallel()

			ctx := context.Background()
			a := newParkedTestAdapter(t)
			ups := appendEdits(t, n)
			for _, u := range ups {
				if _, err := a.AppendUpdate(ctx, "room", u); err != nil {
					t.Fatal(err)
				}
			}
			if c.compactBefore {
				if _, err := a.Compact(ctx, "room", 0); err != nil {
					t.Fatal(err)
				}
			}
			copyB := crdt.New()
			for i, u := range ups {
				if i != c.skip {
					if err := crdt.ApplyUpdateV1(copyB, u, nil); err != nil {
						t.Fatal(err)
					}
				}
			}
			if copyB.PendingStats().Items == 0 {
				t.Fatal("setup: the copy parked nothing")
			}
			if err := a.FlushSnapshot(ctx, "room", crdt.EncodeStateAsUpdateV1(copyB, nil)); err != nil {
				t.Fatal(err)
			}
			if c.compactAfter {
				if _, err := a.Compact(ctx, "room", 0); err != nil {
					t.Fatalf("Compact: %v", err)
				}
			}

			lr, err := a.Load(ctx, "room")
			if err != nil {
				t.Fatalf("Load: %v", err)
			}
			got, parked := loaded(t, lr.Update, "content")
			var want string
			for i := 0; i < n; i++ {
				want += fmt.Sprintf("a%d ", i+1)
			}
			if c.complete {
				if got != want || parked != 0 {
					t.Fatalf("opened %q with %d parked, want the whole text and none", got, parked)
				}
				return
			}
			if parked == 0 {
				t.Fatalf("opened %q with nothing parked; the copy's edits after the missing one are lost", got)
			}
		})
	}
}
