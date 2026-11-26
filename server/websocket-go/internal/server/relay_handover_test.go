package server

import (
	"context"
	"errors"
	"net/http/httptest"
	"testing"
	"time"

	"github.com/alicebob/miniredis/v2"
	"github.com/reearth/ygo/crdt"
	"github.com/reearth/ygo/persistence"
	ygws "github.com/reearth/ygo/provider/websocket"

	"github.com/reearth/reearth-flow/websocket-go/internal/config"
	redisrelay "github.com/reearth/reearth-flow/websocket-go/internal/redis"
)

// redisCluster starts n instances sharing one Redis relay and one store.
func redisCluster(t *testing.T, n int) ([]*Server, []*httptest.Server) {
	t.Helper()
	return redisClusterWith(t, n, testConfig())
}

func redisClusterWith(t *testing.T, n int, cfg *config.Config) ([]*Server, []*httptest.Server) {
	t.Helper()
	ctx, cancel := context.WithCancel(context.Background())
	t.Cleanup(cancel)
	mr, err := miniredis.Run()
	if err != nil {
		t.Fatal(err)
	}
	t.Cleanup(mr.Close)
	mem := persistence.NewMemoryPersistence()
	servers := make([]*Server, n)
	https := make([]*httptest.Server, n)
	for i := range servers {
		s := NewWithPersistence(ctx, cfg, mem)
		relay, err := redisrelay.New(redisrelay.Options{Addr: mr.Addr()})
		if err != nil {
			t.Fatal(err)
		}
		t.Cleanup(func() { _ = relay.Close() })
		if err := s.AttachRedisRelay(ctx, relay); err != nil {
			t.Fatal(err)
		}
		s.SetPresence(relay.ActiveInstances)
		servers[i] = s
		https[i] = httptest.NewServer(s.Handler())
		t.Cleanup(https[i].Close)
	}
	return servers, https
}

// TestRelay_EditJustBeforeLeavingReachesOtherInstances: a user alone on an
// instance edits and drops the connection at once, so the instance unloads the
// room right after publishing the edit. The edit must still reach the instance
// another user is on; otherwise every later edit by the same user, which builds
// on it, waits there forever and a checkpoint from that instance stores them
// unloadable.
func TestRelay_EditJustBeforeLeavingReachesOtherInstances(t *testing.T) {
	t.Parallel()

	servers, https := redisCluster(t, 2)
	alice := dialE2E(t, https[0], versionRoom, 101)
	defer alice.close()
	alice.completeHandshake()

	bobDoc := crdt.New(crdt.WithClientID(202))
	for round := 0; round < 3; round++ {
		bob := dialE2E(t, https[1], versionRoom, 202)
		bob.doc = bobDoc
		bob.completeHandshake()
		bob.insertText("a")
		bob.close()
		waitFor(t, "bob's instance to unload the room", func() bool { return servers[1].ws.GetDoc(versionRoom) == nil })

		bob = dialE2E(t, https[1], versionRoom, 202)
		bob.doc = bobDoc
		bob.completeHandshake()
		bob.insertText("b")
		want := bobDoc.GetText("content").ToString()
		waitFor(t, "alice's instance to hold all of bob's edits", func() bool {
			doc := servers[0].ws.GetDoc(versionRoom)
			return doc.GetText("content").ToString() == want && doc.PendingStats().Items == 0
		})
		bob.close()
	}
}

// TestSignalLock_FromInstanceWithoutTheRoomReachesHolders: a lock change the
// API sends to an instance that does not hold the room is written there, the
// room is closed again at once, and the change must still reach the holders.
func TestSignalLock_FromInstanceWithoutTheRoomReachesHolders(t *testing.T) {
	t.Parallel()

	// A relay entry can load the room on other again after it is closed; the idle
	// timeout lets that copy go too.
	servers, https := redisClusterWith(t, 2, idleConfig(100*time.Millisecond))
	holder, other := servers[0], servers[1]
	c := dialE2E(t, https[0], lockRoom, 101)
	defer c.close()
	c.completeHandshake()
	c.insertText("x")
	waitFor(t, "the edit", func() bool { return serverText(holder, lockRoom) == "x" })

	at := time.Now()
	for i := 1; i <= 3; i++ {
		at = at.Add(time.Millisecond)
		if err := other.SignalLock(context.Background(), lockRoom, i%2 == 1, at); err != nil {
			t.Fatalf("SignalLock: %v", err)
		}
		waitFor(t, "the holder to receive the lock change", func() bool {
			l, ok := storedLock(holder, lockRoom)
			return ok && l.At == at.UnixMilli()
		})
		waitFor(t, "other to close the room loaded only to send the lock change", func() bool {
			return other.ws.GetDoc(lockRoom) == nil
		})
	}
	if p := holder.ws.GetDoc(lockRoom).PendingStats().Items; p != 0 {
		t.Fatalf("holder parked %d items", p)
	}
}

// parkedPair returns two updates: first inserts text, second builds on it.
func parkedPair(t *testing.T) (first, second []byte) {
	t.Helper()
	a := crdt.New(crdt.WithClientID(1))
	txt := a.GetText("content")
	a.Transact(func(txn *crdt.Transaction) { txt.Insert(txn, 0, "a", nil) })
	first = encodeFullState(a)
	b := crdt.New(crdt.WithClientID(2))
	if err := crdt.ApplyUpdateV1(b, first, nil); err != nil {
		t.Fatal(err)
	}
	sv := b.StateVector()
	btxt := b.GetText("content")
	b.Transact(func(txn *crdt.Transaction) { btxt.Insert(txn, 1, "b", nil) })
	return first, crdt.EncodeStateAsUpdateV1(b, sv)
}

// parkedRoom loads room on s holding second without first, as an instance that
// missed first holds it.
func parkedRoom(t *testing.T, s *Server, room string, second []byte) *crdt.Doc {
	t.Helper()
	err := s.ws.Apply(context.Background(), room, func(doc *crdt.Doc, _ func(func(*crdt.Transaction))) {
		if err := crdt.ApplyUpdateV1(doc, second, nil); err != nil {
			t.Error(err)
		}
	})
	// A parked apply adds nothing the document shows, which Apply reports.
	if err != nil && !errors.Is(err, ygws.ErrNoChanges) {
		t.Fatal(err)
	}
	doc := s.ws.GetDoc(room)
	if doc.PendingStats().Items == 0 {
		t.Fatal("setup: nothing parked")
	}
	return doc
}

// TestCheckpointState_RebuildsFromStorageWhenTheCopyIsParked: a copy holding
// parked updates is missing an edit storage has, so it must not be the
// checkpoint.
func TestCheckpointState_RebuildsFromStorageWhenTheCopyIsParked(t *testing.T) {
	t.Parallel()

	s := New(testConfig())
	if s.CheckpointState(versionRoom) != nil {
		t.Fatal("state for a room this instance does not hold")
	}
	_, second := parkedPair(t)
	parkedRoom(t, s, versionRoom, second)
	if s.CheckpointState(versionRoom) != nil {
		t.Fatal("a copy holding parked updates was offered as the checkpoint")
	}

	s2 := New(testConfig())
	c := dialE2E(t, httptestServer(t, s2), versionRoom, 101)
	defer c.close()
	c.completeHandshake()
	c.insertText("x")
	waitFor(t, "the edit", func() bool { return serverText(s2, versionRoom) == "x" })
	if s2.CheckpointState(versionRoom) == nil {
		t.Fatal("a complete copy was not offered as the checkpoint")
	}
}

// TestCompleteState_FillsParkedUpdatesFromStorage: a version saved from a copy
// holding parked updates takes the missing edit from the stored state.
func TestCompleteState_FillsParkedUpdatesFromStorage(t *testing.T) {
	t.Parallel()

	s := New(testConfig())
	first, second := parkedPair(t)
	doc := parkedRoom(t, s, versionRoom, second)
	s.stored = func(context.Context, string) ([]byte, error) { return first, nil }

	st, err := s.completeState(context.Background(), versionRoom, doc)
	if err != nil {
		t.Fatal(err)
	}
	d := crdt.New()
	if err := crdt.ApplyUpdateV1(d, st, nil); err != nil {
		t.Fatal(err)
	}
	if got := d.GetText("content").ToString(); got != "ab" || d.PendingStats().Items != 0 {
		t.Fatalf("version holds %q with %d parked, want \"ab\" and none", got, d.PendingStats().Items)
	}
}

func httptestServer(t *testing.T, s *Server) *httptest.Server {
	t.Helper()
	ts := httptest.NewServer(s.Handler())
	t.Cleanup(ts.Close)
	return ts
}
