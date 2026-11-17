package versions

import (
	"context"
	"slices"
	"testing"
	"time"
)

var t0 = time.UnixMilli(1_700_000_000_000)

func at(ms int) time.Time { return t0.Add(time.Duration(ms) * time.Millisecond) }

func TestRecordMerge_UnitesContributorsSorted(t *testing.T) {
	t.Parallel()

	var r Record
	r.Merge(Mark{Editors: []string{"b", "a"}, FirstAt: at(0), LastAt: at(1)})
	r.Merge(Mark{Editors: []string{"c", "a", ""}, FirstAt: at(2), LastAt: at(3)})
	if want := []string{"a", "b", "c"}; !slices.Equal(r.Contributors, want) {
		t.Fatalf("contributors = %v, want %v", r.Contributors, want)
	}
}

func TestRecordMerge_KeepsEarliestFirstAndLatestLast(t *testing.T) {
	t.Parallel()

	var r Record
	r.Merge(Mark{FirstAt: at(5), LastAt: at(9)})
	// A batch from a slower instance arrives with older edits.
	r.Merge(Mark{FirstAt: at(2), LastAt: at(4)})
	if !r.FirstEditAt.Equal(at(2)) || !r.LastEditAt.Equal(at(9)) {
		t.Fatalf("first/last = %v/%v, want %v/%v", r.FirstEditAt, r.LastEditAt, at(2), at(9))
	}
}

func TestRecordMerge_LastEditorFollowsTheLatestEdit(t *testing.T) {
	t.Parallel()

	var r Record
	r.Merge(Mark{Editors: []string{"late"}, LastEditor: "late", LastEditorAt: at(10), FirstAt: at(10), LastAt: at(10)})
	r.Merge(Mark{Editors: []string{"early"}, LastEditor: "early", LastEditorAt: at(5), FirstAt: at(5), LastAt: at(5)})
	if r.LastEditor != "late" {
		t.Fatalf("last editor = %q, want late: a batch with older edits must not take over", r.LastEditor)
	}
	r.Merge(Mark{FirstAt: at(20), LastAt: at(20)})
	if r.LastEditor != "late" {
		t.Fatalf("last editor = %q, want late: an edit without a sender must not clear it", r.LastEditor)
	}
}

func TestRecordMark_RoundTrips(t *testing.T) {
	t.Parallel()

	r := Record{Contributors: []string{"a", "b"}, LastEditor: "b", LastEditorAt: at(3), FirstEditAt: at(1), LastEditAt: at(3)}
	var back Record
	back.Merge(r.Mark())
	if !slices.Equal(back.Contributors, r.Contributors) || back.LastEditor != r.LastEditor ||
		!back.LastEditorAt.Equal(r.LastEditorAt) || !back.FirstEditAt.Equal(r.FirstEditAt) || !back.LastEditAt.Equal(r.LastEditAt) {
		t.Fatalf("merged back %+v, want %+v", back, r)
	}
}

func TestMemoryStore_TakeGivesTheRecordOnce(t *testing.T) {
	t.Parallel()

	ctx := context.Background()
	s := NewMemoryStore()
	if err := s.Mark(ctx, "room", Mark{Editors: []string{"u"}, FirstAt: at(0), LastAt: at(0)}); err != nil {
		t.Fatal(err)
	}
	if _, ok, _ := s.Peek(ctx, "room"); !ok {
		t.Fatal("Peek found no record")
	}
	r, ok, err := s.Take(ctx, "room")
	if err != nil || !ok || !slices.Equal(r.Contributors, []string{"u"}) {
		t.Fatalf("Take = %+v, %v, %v", r, ok, err)
	}
	if _, ok, _ := s.Take(ctx, "room"); ok {
		t.Fatal("a second Take found the record again")
	}
}
