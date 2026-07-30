package gcs

import (
	"context"
	"math"
	"testing"

	"github.com/reearth/ygo/crdt"
	"github.com/reearth/ygo/persistence"
)

// Ported from logarithmic_retention_test.rs test_first_zero_bit.
func TestFirstZeroBit(t *testing.T) {
	cases := map[uint32]uint32{
		0: 1, 1: 2, 2: 1, 3: 4, 4: 1, 5: 2, 7: 8, 8: 1, 15: 16, 16: 1, 0xFFFF: 0x10000,
	}
	for x, want := range cases {
		if got := firstZeroBit(x); got != want {
			t.Errorf("firstZeroBit(%d) = %d, want %d", x, got, want)
		}
	}
}

// Ported from test_logarithmic_retention_algorithm: ~2·log2(n) survivors, newest
// always kept.
func TestLogarithmicRetentionAlgorithm(t *testing.T) {
	retained := trimUpdatesLogarithmic(100, 1)
	if len(retained) > 100 {
		t.Fatalf("retained %d > 100", len(retained))
	}
	expected := 2.0 * math.Log2(100)
	if float64(len(retained)) > expected*1.5 {
		t.Fatalf("retained %d exceeds %.1f (1.5x of 2·log2(100))", len(retained), expected*1.5)
	}
	if _, ok := retained[100]; !ok {
		t.Fatalf("newest clock 100 must be retained")
	}
}

// Ported from test_density_parameter_impact: runs without panic across shifts.
func TestDensityParameterImpact(t *testing.T) {
	for shift := uint32(0); shift <= 4; shift++ {
		got := trimUpdatesLogarithmic(200, shift)
		if _, ok := got[200]; !ok {
			t.Fatalf("shift=%d dropped newest clock 200", shift)
		}
	}
}

// Ported from test_edge_cases.
func TestRetentionEdgeCases(t *testing.T) {
	if bit := firstZeroBit(0); bit<<1 != 2 {
		t.Fatalf("firstZeroBit(0)<<1 = %d, want 2", bit<<1)
	}
	const largeN = uint32(1_000_000)
	bit := firstZeroBit(largeN)
	if bit == 0 {
		t.Fatal("firstZeroBit(1e6) = 0")
	}
	if largeN-(bit<<1) >= largeN {
		t.Fatal("large-n delete offset not below n")
	}
}

// TestCompactKeepsMostRecentN ports snapshot_management_test cleanup_keeps_most_
// recent_n + cleanup_preserves_document_state: Compact(keep) trims old updates,
// keeps the newest `keep`, and Load still reproduces the full state.
func TestCompactKeepsMostRecentN(t *testing.T) {
	client, bucket := newFakeGCS(t)
	a, err := New(Options{Client: client, Bucket: bucket, Locker: NewNoLock()})
	if err != nil {
		t.Fatalf("New: %v", err)
	}
	ctx := context.Background()

	doc := crdt.New(crdt.WithClientID(11))
	txt := doc.GetText("t")
	var prev []byte
	for i := 0; i < 15; i++ {
		ch := string(rune('a' + i))
		doc.Transact(func(txn *crdt.Transaction) { txt.Insert(txn, 0, ch, nil) })
		full := crdt.EncodeStateAsUpdateV1(doc, nil)
		var inc []byte
		if prev == nil {
			inc = full
		} else {
			sv := svOf(t, prev)
			d, err := crdt.DiffUpdateV1(full, sv)
			if err != nil {
				t.Fatalf("DiffUpdateV1: %v", err)
			}
			inc = d
		}
		if _, err := a.AppendUpdate(ctx, "room", inc); err != nil {
			t.Fatalf("AppendUpdate: %v", err)
		}
		prev = full
	}
	wantText := txt.ToString()

	deleted, err := a.Compact(ctx, "room", 10)
	if err != nil {
		t.Fatalf("Compact: %v", err)
	}
	if deleted != 5 {
		t.Fatalf("Compact(keep=10) deleted = %d, want 5", deleted)
	}
	metas, err := a.ListVersions(ctx, "room")
	if err != nil {
		t.Fatalf("ListVersions: %v", err)
	}
	if len(metas) != 10 {
		t.Fatalf("after compact have %d versions, want 10", len(metas))
	}
	lr, err := a.Load(ctx, "room")
	if err != nil {
		t.Fatalf("Load: %v", err)
	}
	rebuilt := crdt.New()
	if err := crdt.ApplyUpdateV1(rebuilt, lr.Update, nil); err != nil {
		t.Fatalf("ApplyUpdateV1: %v", err)
	}
	if s := rebuilt.GetText("t").ToString(); s != wantText {
		t.Fatalf("post-compact Load text = %q, want %q", s, wantText)
	}
}

// TestCompactNoopWhenFewUpdates ports cleanup_*_noop.
func TestCompactNoopWhenFewUpdates(t *testing.T) {
	client, bucket := newFakeGCS(t)
	a, _ := New(Options{Client: client, Bucket: bucket, Locker: NewNoLock()})
	ctx := context.Background()
	for i := 0; i < 3; i++ {
		if _, err := a.AppendUpdate(ctx, "room", []byte{byte(i + 1)}); err != nil {
			t.Fatalf("AppendUpdate: %v", err)
		}
	}
	deleted, err := a.Compact(ctx, "room", 10)
	if err != nil {
		t.Fatalf("Compact: %v", err)
	}
	if deleted != 0 {
		t.Fatalf("Compact noop deleted = %d, want 0", deleted)
	}
}

func svOf(t *testing.T, v1 []byte) crdt.StateVector {
	t.Helper()
	d := crdt.New()
	if err := crdt.ApplyUpdateV1(d, v1, nil); err != nil {
		t.Fatalf("ApplyUpdateV1: %v", err)
	}
	return d.StateVector()
}

var _ = persistence.Version(0)
