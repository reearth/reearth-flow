package server

import (
	"context"
	"sync"
	"testing"
	"time"

	"github.com/fsouza/fake-gcs-server/fakestorage"
	"github.com/reearth/ygo/crdt"

	"github.com/reearth/reearth-flow/websocket-go/internal/gcs"
)

// TestUnload_KeepsTheUpdateLog: ygo compacts a room each time it unloads it. The
// update log must survive that, as the history the recovery view lists and a
// source for edits a checkpoint missed.
func TestUnload_KeepsTheUpdateLog(t *testing.T) {
	t.Parallel()

	fake, err := fakestorage.NewServerWithOptions(fakestorage.Options{Scheme: "http"})
	if err != nil {
		t.Fatal(err)
	}
	t.Cleanup(fake.Stop)
	fake.CreateBucketWithOpts(fakestorage.CreateBucketOpts{Name: "yrs-test"})
	adapter, err := gcs.New(gcs.Options{Client: fake.Client(), Bucket: "yrs-test"})
	if err != nil {
		t.Fatal(err)
	}
	s := NewWithPersistence(context.Background(), testConfig(), adapter)
	// ygo runs the unload compaction before OnUnloadDocument, but after the room
	// has left its map.
	unloaded := make(chan struct{})
	var once sync.Once
	unload := s.ws.OnUnloadDocument
	s.ws.OnUnloadDocument = func(ctx context.Context, room string) {
		unload(ctx, room)
		if room == versionRoom {
			once.Do(func() { close(unloaded) })
		}
	}

	c := dialE2E(t, httptestServer(t, s), versionRoom, 101)
	c.completeHandshake()
	for _, x := range []string{"a", "b", "c"} {
		c.insertText(x)
	}
	waitFor(t, "the edits", func() bool { return serverText(s, versionRoom) == "abc" })
	c.close()
	select {
	case <-unloaded:
	case <-time.After(10 * time.Second):
		t.Fatal("the room never unloaded")
	}

	vs, err := adapter.ListVersions(context.Background(), versionRoom)
	if err != nil {
		t.Fatal(err)
	}
	if len(vs) == 0 {
		t.Fatal("unloading the room deleted the whole update log")
	}
	lr, err := adapter.Load(context.Background(), versionRoom)
	if err != nil {
		t.Fatal(err)
	}
	d := crdt.New()
	if err := crdt.ApplyUpdateV1(d, lr.Update, nil); err != nil {
		t.Fatal(err)
	}
	if got := d.GetText("content").ToString(); got != "abc" {
		t.Fatalf("stored text %q, want \"abc\"", got)
	}
}
