package versions

import (
	"context"
	"slices"
	"testing"
	"time"

	"github.com/reearth/ygo/crdt"
)

var t0 = time.UnixMilli(1_700_000_000_000)

func at(ms int) time.Time { return t0.Add(time.Duration(ms) * time.Millisecond) }

// testDoc is a document the tests edit, as a browser tab would.
type testDoc struct{ doc *crdt.Doc }

func newTestDoc(client uint64) *testDoc {
	return &testDoc{doc: crdt.New(crdt.WithClientID(crdt.ClientID(client)))}
}

// edit runs fn in a transaction and returns the update it produced.
func (d *testDoc) edit(fn func(txn *crdt.Transaction, txt *crdt.YText)) []byte {
	txt := d.doc.GetText("t")
	before := d.doc.StateVector()
	d.doc.Transact(func(txn *crdt.Transaction) { fn(txn, txt) })
	return crdt.EncodeStateAsUpdateV1(d.doc, before)
}

func (d *testDoc) write(s string) []byte {
	return d.edit(func(txn *crdt.Transaction, txt *crdt.YText) { txt.Insert(txn, txt.Len(), s, nil) })
}

func (d *testDoc) deleteFirst() []byte {
	return d.edit(func(txn *crdt.Transaction, txt *crdt.YText) { txt.Delete(txn, 0, 1) })
}

func (d *testDoc) state(context.Context) ([]byte, error) {
	return crdt.EncodeStateAsUpdateV1(d.doc, nil), nil
}

// apply brings an update made elsewhere into d.
func (d *testDoc) apply(t *testing.T, update []byte) {
	t.Helper()
	if err := crdt.ApplyUpdateV1(d.doc, update, nil); err != nil {
		t.Fatal(err)
	}
}

func idsOf(t *testing.T, update []byte) crdt.ContentIDs {
	t.Helper()
	ids, err := crdt.ContentIDsFromUpdateV1(update)
	if err != nil {
		t.Fatal(err)
	}
	return ids
}

// batch is a record of user's update made at ms.
func batch(t *testing.T, user string, update []byte, ms int) Record {
	t.Helper()
	e := NewEdits()
	e.add(idsOf(t, update), at(ms))
	return Record{user: e}
}

func TestRecordMerge(t *testing.T) {
	t.Parallel()

	d := newTestDoc(1)
	r := Record{}
	r.Merge(batch(t, "bob", d.write("a"), 10))
	r.Merge(batch(t, "alice", d.write("b"), 5))
	r.Merge(batch(t, "bob", d.write("c"), 2)) // a slower instance's older batch
	r.Merge(batch(t, "", d.write("d"), 20))

	if got := r.Contributors(); !slices.Equal(got, []string{"alice", "bob"}) {
		t.Fatalf("contributors = %v", got)
	}
	if who, when := r.LastEditor(); who != "bob" || !when.Equal(at(10)) {
		t.Fatalf("last editor = %q at %v, want bob: an edit without a sender names nobody", who, when)
	}
	if !r.FirstEditAt().Equal(at(2)) || !r.LastEditAt().Equal(at(20)) {
		t.Fatalf("first/last = %v/%v", r.FirstEditAt(), r.LastEditAt())
	}
	if b := r["bob"]; !b.FirstAt.Equal(at(2)) || !b.LastAt.Equal(at(10)) || len(b.Inserts.Ranges(1)) != 2 { // "a" and "c", with alice's "b" between
		t.Fatalf("bob's edits = %+v", b)
	}
}

func TestRecordEncodeRoundTrip(t *testing.T) {
	t.Parallel()

	d := newTestDoc(1)
	r := batch(t, "alice", d.write("ab"), 1)
	r.Merge(batch(t, "alice", d.deleteFirst(), 2))
	r.Merge(batch(t, "", d.write("c"), 3))

	b, err := EncodeRecord(r)
	if err != nil {
		t.Fatal(err)
	}
	other, _ := EncodeRecord(batch(t, "bob", d.write("x"), 4))
	back, err := DecodeRecords([][]byte{b, other})
	if err != nil {
		t.Fatal(err)
	}
	if got := back.Contributors(); !slices.Equal(got, []string{"alice", "bob"}) {
		t.Fatalf("contributors = %v", got)
	}
	a := back["alice"]
	if a.Inserts.IsEmpty() || a.Deletes.IsEmpty() || !a.FirstAt.Equal(at(1)) || !a.LastAt.Equal(at(2)) {
		t.Fatalf("alice = %+v", a)
	}
	if _, ok := back[""]; !ok {
		t.Fatal("edits without a sender were lost")
	}
}

func TestRecordSplit_ByWhatTheStateHolds(t *testing.T) {
	t.Parallel()

	// alice's tab and bob's tab; the server copy has alice's edit only.
	alice, bob, server := newTestDoc(1), newTestDoc(2), newTestDoc(3)
	ua := alice.write("a")
	ub := bob.write("b")
	server.apply(t, ua)

	r := batch(t, "alice", ua, 1)
	r.Merge(batch(t, "bob", ub, 2))
	st, _ := server.state(context.Background())
	held, missing, err := r.Split(st)
	if err != nil {
		t.Fatal(err)
	}
	if got := held.Contributors(); !slices.Equal(got, []string{"alice"}) {
		t.Fatalf("held = %v, want alice", got)
	}
	if got := missing.Contributors(); !slices.Equal(got, []string{"bob"}) {
		t.Fatalf("missing = %v, want bob", got)
	}
}

func TestRecordSplit_DeletionNotYetApplied(t *testing.T) {
	t.Parallel()

	alice, server := newTestDoc(1), newTestDoc(3)
	ins := alice.write("ab")
	server.apply(t, ins)
	del := alice.deleteFirst() // not yet on the server

	r := batch(t, "alice", del, 2)
	st, _ := server.state(context.Background())
	held, missing, err := r.Split(st)
	if err != nil {
		t.Fatal(err)
	}
	if len(held) != 0 || len(missing) != 1 {
		t.Fatalf("held %v missing %v; a delete the state lacks is missing", held, missing)
	}

	server.apply(t, del)
	st, _ = server.state(context.Background())
	held, missing, _ = r.Split(st)
	if len(held) != 1 || len(missing) != 0 {
		t.Fatalf("held %v missing %v after the delete arrived", held, missing)
	}
}

func TestRecordSplit_PartlyHeld(t *testing.T) {
	t.Parallel()

	alice, server := newTestDoc(1), newTestDoc(3)
	u1 := alice.write("a")
	u2 := alice.write("b")
	server.apply(t, u1)

	r := batch(t, "alice", u1, 1)
	r.Merge(batch(t, "alice", u2, 2))
	st, _ := server.state(context.Background())
	held, missing, _ := r.Split(st)
	if _, ok := held["alice"]; !ok {
		t.Fatal("alice's first edit is held")
	}
	if m, ok := missing["alice"]; !ok || m.Inserts.IsEmpty() {
		t.Fatal("alice's second edit is missing")
	}
}

func TestRecordSplit_EditWithoutIDsCountsAsHeld(t *testing.T) {
	t.Parallel()

	r := Record{"alice": Edits{Inserts: crdt.NewIDSet(), Deletes: crdt.NewIDSet(), FirstAt: at(1), LastAt: at(1)}}
	st, _ := newTestDoc(3).state(context.Background())
	held, missing, _ := r.Split(st)
	if len(held) != 1 || len(missing) != 0 {
		t.Fatalf("held %v missing %v: an edit whose items are unknown must not wait forever", held, missing)
	}
}

func TestMemoryStore_TakeGivesTheRecordOnce(t *testing.T) {
	t.Parallel()

	ctx := context.Background()
	s := NewMemoryStore()
	d := newTestDoc(1)
	if err := s.Mark(ctx, "room", batch(t, "u", d.write("x"), 3)); err != nil {
		t.Fatal(err)
	}
	if first, ok, _ := s.FirstEdit(ctx, "room"); !ok || !first.Equal(at(3)) {
		t.Fatalf("FirstEdit = %v, %v", first, ok)
	}
	r, ok, err := s.Take(ctx, "room")
	if err != nil || !ok || !slices.Equal(r.Contributors(), []string{"u"}) {
		t.Fatalf("Take = %+v, %v, %v", r, ok, err)
	}
	if _, ok, _ := s.Take(ctx, "room"); ok {
		t.Fatal("a second Take found the record again")
	}
}
