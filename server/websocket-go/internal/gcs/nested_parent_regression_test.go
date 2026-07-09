package gcs

import (
	"context"
	"testing"

	"github.com/reearth/ygo/crdt"
)

// TestLoad_NestedParentCrossClient_Regression drives a nested container whose
// first child was authored by a lower-clientID peer through the adapter's Load,
// asserting it reconstructs with no error and no data loss. This shape returned
// HTTP 500 on ygo before v1.23.1, then silently dropped the child on ygo
// v1.27.0 through v1.30.0; it is the prod failure fixed by the v1.30.1 bump.
// See reearth/ygo#140.
func TestLoad_NestedParentCrossClient_Regression(t *testing.T) {
	// Client 200 creates the nested container (an XML element).
	d200 := crdt.New(crdt.WithClientID(200))
	frag200 := d200.GetXmlFragment("f")
	el := crdt.NewYXmlElement("div")
	d200.Transact(func(txn *crdt.Transaction) { frag200.InsertElement(txn, 0, el) })

	// Client 100 (lower ID) syncs, then writes the first child into that
	// container: an attribute whose parent-by-ID points at client 200's element.
	d100 := crdt.New(crdt.WithClientID(100))
	if err := crdt.ApplyUpdateV1(d100, crdt.EncodeStateAsUpdateV1(d200, nil), nil); err != nil {
		t.Fatalf("seed d100 from d200 full state: %v", err)
	}
	children := d100.GetXmlFragment("f").Children()
	if len(children) != 1 {
		t.Fatalf("d100 expected 1 child, got %d", len(children))
	}
	el100, ok := children[0].(*crdt.YXmlElement)
	if !ok {
		t.Fatalf("d100 child is not *YXmlElement: %T", children[0])
	}
	d100.Transact(func(txn *crdt.Transaction) { el100.SetAttribute(txn, "class", "hello") })

	// The full-state encode now holds both client groups; group 100 (the
	// attribute) sorts before group 200 (the element it references).
	full := crdt.EncodeStateAsUpdateV1(d100, nil)

	// Drive it through the adapter exactly as production does: persist the state
	// then Load (which folds it via crdt.MergeUpdatesV1).
	client, bucket := newFakeGCS(t)
	a, err := New(Options{Client: client, Bucket: bucket, Locker: NewNoLock()})
	if err != nil {
		t.Fatalf("gcs.New: %v", err)
	}
	ctx := context.Background()
	const room = "01ktn5as6ndekfx7xk02dmxs63" // shaped like a real flow room id

	if _, err := a.AppendUpdate(ctx, room, full); err != nil {
		t.Fatalf("AppendUpdate: %v", err)
	}

	lr, err := a.Load(ctx, room)
	if err != nil {
		t.Fatalf("Load returned error (the production 500): %v", err)
	}
	if len(lr.Update) == 0 {
		t.Fatal("Load returned empty state; expected the reconstructed document")
	}

	// The reconstructed state must still carry the cross-client attribute.
	got := crdt.New()
	if err := crdt.ApplyUpdateV1(got, lr.Update, nil); err != nil {
		t.Fatalf("re-apply reconstructed state: %v", err)
	}
	gc := got.GetXmlFragment("f").Children()
	if len(gc) != 1 {
		t.Fatalf("reconstructed expected 1 child, got %d", len(gc))
	}
	got0, isEl := gc[0].(*crdt.YXmlElement)
	if !isEl {
		t.Fatalf("reconstructed child is not *YXmlElement: %T", gc[0])
	}
	v, hasAttr := got0.GetAttribute("class")
	if !hasAttr || v != "hello" {
		t.Fatalf("reconstructed attribute class=%q ok=%v, want \"hello\"", v, hasAttr)
	}
}
