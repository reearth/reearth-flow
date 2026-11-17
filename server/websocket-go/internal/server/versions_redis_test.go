package server

import (
	"context"
	"net/http/httptest"
	"slices"
	"testing"

	"github.com/alicebob/miniredis/v2"
	goredis "github.com/redis/go-redis/v9"
	"github.com/reearth/ygo/crdt"
	"github.com/reearth/ygo/persistence"

	redisrelay "github.com/reearth/reearth-flow/websocket-go/internal/redis"
)

// TestVersions_AcrossInstancesThroughRedis runs the production wiring on one
// Redis: a version requested from an instance that does not hold the room is
// saved from what the holder published, once, with the holder's editors.
func TestVersions_AcrossInstancesThroughRedis(t *testing.T) {
	t.Parallel()

	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	mr, err := miniredis.Run()
	if err != nil {
		t.Fatal(err)
	}
	defer mr.Close()
	client := goredis.NewClient(&goredis.Options{Addr: mr.Addr()})
	defer func() { _ = client.Close() }()
	store := redisrelay.NewChangeStore(client)
	mem := persistence.NewMemoryPersistence()
	snaps := &memVersions{}

	instance := func() *Server {
		s := NewWithPersistence(ctx, testConfig(), mem)
		enableTestVersions(s, store, snaps, 0)
		relay, err := redisrelay.New(redisrelay.Options{Addr: mr.Addr(), Flusher: s.VersionedFlusher(&countingFlusher{})})
		if err != nil {
			t.Fatal(err)
		}
		t.Cleanup(func() { _ = relay.Close() })
		if err := s.AttachRedisRelay(ctx, relay); err != nil {
			t.Fatal(err)
		}
		s.SetPresence(relay.ActiveInstances)
		s.SetCatchUp(relay.WaitCaughtUp)
		return s
	}
	holder, other := instance(), instance()

	ts := httptest.NewServer(holder.Handler())
	defer ts.Close()
	dialAs(t, ts, "alice", 101).insertText("hello")
	waitFor(t, "the edit", func() bool { return serverText(holder, versionRoom) == "hello" })
	if _, ok := record(t, holder, store); !ok {
		t.Fatal("the edit was not recorded in Redis")
	}

	res, err := other.SaveVersion(ctx, versionRoom, "Saved")
	if err != nil || !res.Saved {
		t.Fatalf("SaveVersion = %+v, %v", res, err)
	}
	saved := snaps.all()
	d := crdt.New()
	if err := crdt.ApplyUpdateV1(d, saved[0].State, nil); err != nil {
		t.Fatal(err)
	}
	if got := d.GetText("content").ToString(); got != "hello" {
		t.Fatalf("version holds %q, want the holder's edit", got)
	}
	if !slices.Equal(saved[0].Contributors, []string{"alice"}) {
		t.Fatalf("contributors = %v", saved[0].Contributors)
	}
	if other.ws.GetDoc(versionRoom) != nil {
		t.Fatal("the room loaded only for the version is still open")
	}

	if res, err := holder.SaveVersion(ctx, versionRoom, "Saved"); err != nil || res.Saved {
		t.Fatalf("holder SaveVersion = %+v, %v; want nothing: the change was saved", res, err)
	}
}
