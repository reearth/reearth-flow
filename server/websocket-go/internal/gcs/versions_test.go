package gcs

import (
	"context"
	"fmt"
	"slices"
	"strings"
	"testing"

	"github.com/reearth/reearth-flow/websocket-go/internal/versions"
)

func newVersionAdapter(t *testing.T) *Adapter {
	t.Helper()
	client, bucket := newFakeGCS(t)
	a, err := New(Options{Client: client, Bucket: bucket, Locker: NewNoLock()})
	if err != nil {
		t.Fatalf("gcs.New: %v", err)
	}
	return a
}

func TestVersionStore_SavesAndListsAttribution(t *testing.T) {
	t.Parallel()

	ctx := context.Background()
	a := newVersionAdapter(t)
	vs := a.Versions(0)
	id, err := vs.Save(ctx, "room", versions.Version{
		Label: "Saved", State: []byte("state"),
		Contributors: []string{"carol", "alice", "bob"}, LastEditor: "bob",
	})
	if err != nil {
		t.Fatalf("Save: %v", err)
	}
	details, err := a.ListSnapshotDetails(ctx, "room")
	if err != nil || len(details) != 1 {
		t.Fatalf("ListSnapshotDetails = %v, %v", details, err)
	}
	d := details[0]
	if d.ID != id || d.Label != "Saved" || d.LastEditor != "bob" || d.ContributorCount != 3 {
		t.Fatalf("detail = %+v", d)
	}
	// The last editor first, the rest sorted.
	if want := []string{"bob", "alice", "carol"}; !slices.Equal(d.Contributors, want) {
		t.Fatalf("contributors = %v, want %v", d.Contributors, want)
	}
}

func TestVersionStore_UnattributedSnapshotHasNoContributors(t *testing.T) {
	t.Parallel()

	ctx := context.Background()
	a := newVersionAdapter(t)
	if _, err := a.SaveSnapshot(ctx, "room", "Before restore", []byte("state")); err != nil {
		t.Fatal(err)
	}
	details, _ := a.ListSnapshotDetails(ctx, "room")
	if len(details) != 1 || details[0].Contributors != nil || details[0].ContributorCount != 0 || details[0].LastEditor != "" {
		t.Fatalf("details = %+v", details)
	}
}

func TestAuthorsMeta_CutsTheListToFitKeepingTheLastEditor(t *testing.T) {
	t.Parallel()

	var ids []string
	for i := range 400 {
		ids = append(ids, fmt.Sprintf("01J%023d", i)) // 26 bytes, as user ids are
	}
	last := "zz-last-editor"
	meta := authorsMeta(Authors{Contributors: append(ids, last), LastEditor: last})

	if len(meta[snapMetaContributors]) > maxContributorBytes {
		t.Fatalf("list is %d bytes, want at most %d", len(meta[snapMetaContributors]), maxContributorBytes)
	}
	listed := strings.Split(meta[snapMetaContributors], ",")
	if listed[0] != last {
		t.Fatalf("first listed = %q, want the last editor", listed[0])
	}
	if meta[snapMetaContributorCount] != "401" {
		t.Fatalf("count = %q, want 401", meta[snapMetaContributorCount])
	}
	total := 0
	for k, v := range meta {
		total += len(k) + len(v)
	}
	if total+maxLabelBytes+64 > 8*1024 {
		t.Fatalf("attribution metadata is %d bytes; with a full label it would pass GCS's 8 KiB cap", total)
	}
}

func TestAuthorsMeta_SkipsUnlistableIDs(t *testing.T) {
	t.Parallel()

	meta := authorsMeta(Authors{Contributors: []string{"a,b", "", "c", "c"}})
	if meta[snapMetaContributors] != "c" || meta[snapMetaContributorCount] != "1" {
		t.Fatalf("meta = %v", meta)
	}
	if _, ok := meta[snapMetaLastEditor]; ok {
		t.Fatal("no last editor was given")
	}
}

func TestTrimSnapshots_IsPerLabel(t *testing.T) {
	t.Parallel()

	ctx := context.Background()
	a := newVersionAdapter(t)
	vs := a.Versions(2)
	named, err := a.SaveSnapshot(ctx, "room", "Before restore", []byte("s"))
	if err != nil {
		t.Fatal(err)
	}
	for range 4 {
		if _, err := vs.Save(ctx, "room", versions.Version{Label: versions.AutoLabel, State: []byte("s")}); err != nil {
			t.Fatal(err)
		}
	}
	snaps, _ := a.ListSnapshots(ctx, "room")
	var auto int
	var keptNamed bool
	for _, sn := range snaps {
		switch sn.Label {
		case versions.AutoLabel:
			auto++
		default:
			keptNamed = keptNamed || sn.ID == named
		}
	}
	if auto != 2 || !keptNamed {
		t.Fatalf("auto = %d, named kept = %v; want 2 automatic versions and the named one", auto, keptNamed)
	}
}

func TestLatestSnapshotState(t *testing.T) {
	t.Parallel()

	ctx := context.Background()
	a := newVersionAdapter(t)
	if st, err := a.LatestSnapshotState(ctx, "room"); st != nil || err != nil {
		t.Fatalf("empty room = %v, %v", st, err)
	}
	_, _ = a.SaveSnapshot(ctx, "room", "a", []byte("first"))
	_, _ = a.SaveSnapshot(ctx, "room", "b", []byte("second"))
	st, err := a.LatestSnapshotState(ctx, "room")
	if err != nil || string(st) != "second" {
		t.Fatalf("latest = %q, %v", st, err)
	}
}
