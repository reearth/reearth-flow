package gcs

import (
	"context"
	"errors"
	"fmt"
	"slices"
	"strings"
	"testing"
	"time"

	"github.com/reearth/ygo/persistence"

	"github.com/reearth/reearth-flow/websocket-go/internal/versions"
)

func newVersionAdapter(t *testing.T) *Adapter {
	t.Helper()
	return newVersionAdapterKeeping(t, 0)
}

func newVersionAdapterKeeping(t *testing.T, keep int) *Adapter {
	t.Helper()
	client, bucket := newFakeGCS(t)
	a, err := New(Options{Client: client, Bucket: bucket, Locker: NewNoLock(), KeepSnapshots: keep})
	if err != nil {
		t.Fatalf("gcs.New: %v", err)
	}
	return a
}

var editedAt = time.UnixMilli(1_700_000_000_123)

func TestVersionStore_SavesAndListsAttribution(t *testing.T) {
	t.Parallel()

	ctx := context.Background()
	a := newVersionAdapter(t)
	id, err := a.Versions(0).Save(ctx, "room", versions.Version{
		Label: "Saved", State: []byte("state"),
		Attribution: versions.Attribution{
			Contributors: []string{"carol", "alice", "bob"}, LastEditor: "bob", LastEditedAt: editedAt,
		},
	})
	if err != nil {
		t.Fatalf("Save: %v", err)
	}
	listed, err := a.ListSnapshotDetails(ctx, "room")
	if err != nil || len(listed) != 1 {
		t.Fatalf("ListSnapshotDetails = %v, %v", listed, err)
	}
	l := listed[0]
	if l.ID != id || l.Label != "Saved" || l.LastEditor != "bob" || l.ContributorCount != 3 || !l.LastEditedAt.Equal(editedAt) {
		t.Fatalf("listed = %+v", l)
	}
	if want := []string{"alice", "bob", "carol"}; !slices.Equal(l.Contributors, want) {
		t.Fatalf("contributors = %v, want %v", l.Contributors, want)
	}
}

func TestVersionStore_UnattributedSnapshotHasNoContributors(t *testing.T) {
	t.Parallel()

	ctx := context.Background()
	a := newVersionAdapter(t)
	if _, err := a.SaveSnapshot(ctx, "room", "Before restore", []byte("state")); err != nil {
		t.Fatal(err)
	}
	listed, _ := a.ListSnapshotDetails(ctx, "room")
	if len(listed) != 1 || listed[0].Contributors != nil || listed[0].ContributorCount != 0 || listed[0].LastEditor != "" {
		t.Fatalf("listed = %+v", listed)
	}
}

func TestVersionStore_AttributeRewritesOnlyTheAttribution(t *testing.T) {
	t.Parallel()

	ctx := context.Background()
	a := newVersionAdapter(t)
	vs := a.Versions(0)
	id, _ := vs.Save(ctx, "room", versions.Version{
		Label: "Saved", State: []byte("state"),
		Attribution: versions.Attribution{Contributors: []string{"alice"}, LastEditor: "alice", LastEditedAt: editedAt},
	})
	later := editedAt.Add(time.Second)
	if err := vs.Attribute(ctx, "room", id, versions.Attribution{
		Contributors: []string{"alice", "bob"}, LastEditor: "bob", LastEditedAt: later,
	}); err != nil {
		t.Fatalf("Attribute: %v", err)
	}
	latest, ok, err := vs.Latest(ctx, "room")
	if err != nil || !ok {
		t.Fatalf("Latest = %v, %v", ok, err)
	}
	if latest.ID != id || latest.Label != "Saved" || string(latest.State) != "state" ||
		!slices.Equal(latest.Contributors, []string{"alice", "bob"}) || latest.LastEditor != "bob" || !latest.LastEditedAt.Equal(later) {
		t.Fatalf("latest = %+v", latest)
	}
	listed, _ := a.ListSnapshots(ctx, "room")
	if listed[0].Size != int64(len("state")) {
		t.Fatalf("size = %d; the rest of the metadata must survive", listed[0].Size)
	}
	if err := vs.Attribute(ctx, "room", id+5, versions.Attribution{}); !errors.Is(err, persistence.ErrSnapshotNotFound) {
		t.Fatalf("Attribute(unknown) = %v", err)
	}
}

func TestVersionStore_CreditingACutListKeepsItsCount(t *testing.T) {
	t.Parallel()

	ctx := context.Background()
	a := newVersionAdapter(t)
	vs := a.Versions(0)
	var ids []string
	for i := range 400 {
		ids = append(ids, fmt.Sprintf("01J%023d", i))
	}
	id, err := vs.Save(ctx, "room", versions.Version{Label: "auto", State: []byte("s"), Attribution: versions.Attribution{
		Contributors: ids, LastEditor: ids[0], LastEditedAt: editedAt,
	}})
	if err != nil {
		t.Fatal(err)
	}
	latest, _, err := vs.Latest(ctx, "room")
	if err != nil {
		t.Fatal(err)
	}
	if len(latest.Contributors) >= 400 || latest.Count() != 400 {
		t.Fatalf("latest lists %d and counts %d; want a cut list counting 400", len(latest.Contributors), latest.Count())
	}
	// What Saver.credit stores after merging one more contributor in.
	grown := latest.Attribution
	grown.Contributors = append(slices.Clone(grown.Contributors), "zz-new")
	if err := vs.Attribute(ctx, "room", id, grown); err != nil {
		t.Fatal(err)
	}
	listed, _ := a.ListSnapshotDetails(ctx, "room")
	if listed[0].Count() != 400 {
		t.Fatalf("count = %d after crediting; it must not shrink to the cut list", listed[0].Count())
	}
}

func TestAttributionMeta_CutsTheListToFitKeepingTheLastEditor(t *testing.T) {
	t.Parallel()

	var ids []string
	for i := range 400 {
		ids = append(ids, fmt.Sprintf("01J%023d", i)) // 26 bytes, as user ids are
	}
	last := "zz-last-editor"
	meta := attributionMeta(versions.Attribution{Contributors: append(ids, last), LastEditor: last, LastEditedAt: editedAt})

	if len(meta[snapMetaContributors]) > maxContributorBytes {
		t.Fatalf("list is %d bytes, want at most %d", len(meta[snapMetaContributors]), maxContributorBytes)
	}
	if listed := strings.Split(meta[snapMetaContributors], ","); listed[0] != last {
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

func TestAttributionMeta_SkipsUnlistableIDs(t *testing.T) {
	t.Parallel()

	meta := attributionMeta(versions.Attribution{Contributors: []string{"a,b", "", "c", "c"}})
	if meta[snapMetaContributors] != "c" || meta[snapMetaContributorCount] != "1" {
		t.Fatalf("meta = %v", meta)
	}
	if _, ok := meta[snapMetaLastEditor]; ok {
		t.Fatal("no last editor was given")
	}
}

// labelCounts counts room's snapshots by label.
func labelCounts(t *testing.T, a *Adapter) map[string]int {
	t.Helper()
	snaps, err := a.ListSnapshots(context.Background(), "room")
	if err != nil {
		t.Fatal(err)
	}
	n := map[string]int{}
	for _, sn := range snaps {
		n[sn.Label]++
	}
	return n
}

func TestTrimSnapshots_AutomaticVersionsNeverEvictOthers(t *testing.T) {
	t.Parallel()

	ctx := context.Background()
	a := newVersionAdapter(t)
	vs := a.Versions(2)
	if _, err := a.SaveSnapshot(ctx, "room", "Before restore", []byte("s")); err != nil {
		t.Fatal(err)
	}
	for range 4 {
		if _, err := vs.Save(ctx, "room", versions.Version{Label: versions.AutoLabel, State: []byte("s")}); err != nil {
			t.Fatal(err)
		}
	}
	if n := labelCounts(t, a); n[versions.AutoLabel] != 2 || n["Before restore"] != 1 {
		t.Fatalf("counts = %v; want 2 automatic and the other kept", n)
	}
}

func TestTrimSnapshots_DistinctLabelsShareOneBound(t *testing.T) {
	t.Parallel()

	ctx := context.Background()
	a := newVersionAdapter(t)
	vs := a.Versions(2)
	if _, err := vs.Save(ctx, "room", versions.Version{Label: versions.AutoLabel, State: []byte("s")}); err != nil {
		t.Fatal(err)
	}
	for i := range 5 {
		if _, err := vs.Save(ctx, "room", versions.Version{Label: fmt.Sprintf("Deployment %d", i), State: []byte("s")}); err != nil {
			t.Fatal(err)
		}
	}
	n := labelCounts(t, a)
	others := 0
	for label, c := range n {
		if label != versions.AutoLabel {
			others += c
		}
	}
	if others != 2 || n[versions.AutoLabel] != 1 || n["Deployment 4"] != 1 || n["Deployment 3"] != 1 {
		t.Fatalf("counts = %v; a label per save must still be bounded, newest kept", n)
	}
}

func TestTrimSnapshots_VersionsNeverEvictTheRestoreSnapshot(t *testing.T) {
	t.Parallel()

	ctx := context.Background()
	a := newVersionAdapterKeeping(t, 2)
	vs := a.Versions(2)
	if _, err := a.SaveSnapshot(ctx, "room", "Before restore", []byte("s")); err != nil {
		t.Fatal(err)
	}
	for range 4 {
		if _, err := vs.Save(ctx, "room", versions.Version{Label: "Saved", State: []byte("s")}); err != nil {
			t.Fatal(err)
		}
	}
	if n := labelCounts(t, a); n["Saved"] != 2 || n["Before restore"] != 1 {
		t.Fatalf("counts = %v; want 2 versions and the restore snapshot kept", n)
	}
}

func TestSaveSnapshot_TrimsOnlyItsOwnClass(t *testing.T) {
	t.Parallel()

	ctx := context.Background()
	a := newVersionAdapterKeeping(t, 2)
	if _, err := a.Versions(2).Save(ctx, "room", versions.Version{Label: "Saved", State: []byte("s")}); err != nil {
		t.Fatal(err)
	}
	for range 4 {
		if _, err := a.SaveSnapshot(ctx, "room", "Before restore", []byte("s")); err != nil {
			t.Fatal(err)
		}
	}
	if n := labelCounts(t, a); n["Before restore"] != 2 || n["Saved"] != 1 {
		t.Fatalf("counts = %v; want restore snapshots bounded and the version kept", n)
	}
}

func TestAttributeSnapshot_KeepsTheVersionsClass(t *testing.T) {
	t.Parallel()

	ctx := context.Background()
	a := newVersionAdapter(t)
	vs := a.Versions(0)
	id, _ := vs.Save(ctx, "room", versions.Version{Label: "Saved", State: []byte("s")})
	if err := vs.Attribute(ctx, "room", id, versions.Attribution{Contributors: []string{"bob"}}); err != nil {
		t.Fatal(err)
	}
	listed, err := a.listSnapshots(ctx, "room")
	if err != nil || len(listed) != 1 || listed[0].class != classVersion {
		t.Fatalf("listed = %+v, %v; crediting must keep the version in its class", listed, err)
	}
}

func TestLatestVersion(t *testing.T) {
	t.Parallel()

	ctx := context.Background()
	a := newVersionAdapter(t)
	vs := a.Versions(0)
	if _, ok, err := vs.Latest(ctx, "room"); ok || err != nil {
		t.Fatalf("empty room = %v, %v", ok, err)
	}
	_, _ = vs.Save(ctx, "room", versions.Version{Label: versions.AutoLabel, State: []byte("first")})
	_, _ = vs.Save(ctx, "room", versions.Version{Label: "Saved", State: []byte("second")})
	v, ok, err := vs.Latest(ctx, "room")
	if err != nil || !ok || string(v.State) != "second" || v.Label != "Saved" {
		t.Fatalf("latest = %+v, %v, %v", v, ok, err)
	}
}

func TestLatestVersion_SkipsOtherSnapshots(t *testing.T) {
	t.Parallel()

	ctx := context.Background()
	a := newVersionAdapter(t)
	vs := a.Versions(0)
	// Only a snapshot taken before a restore: no version to credit edits to.
	_, _ = a.SaveSnapshot(ctx, "room", "Before restore", []byte("restore"))
	if v, ok, err := vs.Latest(ctx, "room"); ok || err != nil {
		t.Fatalf("latest = %+v, %v, %v; a restore snapshot is not a version", v, ok, err)
	}
	_, _ = vs.Save(ctx, "room", versions.Version{Label: versions.AutoLabel, State: []byte("version")})
	_, _ = a.SaveSnapshot(ctx, "room", "Before restore", []byte("newer"))
	v, ok, err := vs.Latest(ctx, "room")
	if err != nil || !ok || string(v.State) != "version" {
		t.Fatalf("latest = %+v, %v, %v; want the version under the newer restore snapshot", v, ok, err)
	}
}
