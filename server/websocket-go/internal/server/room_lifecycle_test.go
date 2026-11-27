package server

import (
	"context"
	"errors"
	"fmt"
	"net/http"
	"net/http/httptest"
	"runtime"
	"strings"
	"sync"
	"sync/atomic"
	"testing"
	"time"

	"cloud.google.com/go/storage"
	"github.com/alicebob/miniredis/v2"
	"github.com/fsouza/fake-gcs-server/fakestorage"
	gws "github.com/gorilla/websocket"
	goredis "github.com/redis/go-redis/v9"
	"github.com/reearth/ygo/awareness"
	ygws "github.com/reearth/ygo/provider/websocket"

	"github.com/reearth/reearth-flow/websocket-go/internal/auth"
	"github.com/reearth/reearth-flow/websocket-go/internal/config"
	"github.com/reearth/reearth-flow/websocket-go/internal/gcs"
	redisrelay "github.com/reearth/reearth-flow/websocket-go/internal/redis"
)

// These tests use the production wiring of cmd/websocket/main.go (GCS adapter,
// Redis locker, flusher, versions, Redis relay) over miniredis and fake GCS.
// MemoryPersistence tears a room down at once and hides the teardown window.

const lifecycleRoom = "550e8400-e29b-41d4-a716-4466554400aa"

type lcEnv struct {
	t      *testing.T
	ctx    context.Context
	mr     *miniredis.Miniredis
	client *storage.Client
	bucket string
}

func newLCEnv(t *testing.T) *lcEnv {
	t.Helper()
	ctx, cancel := context.WithCancel(context.Background())
	t.Cleanup(cancel)
	mr, err := miniredis.Run()
	if err != nil {
		t.Fatal(err)
	}
	t.Cleanup(mr.Close)
	srv, err := fakestorage.NewServerWithOptions(fakestorage.Options{Scheme: "http"})
	if err != nil {
		t.Fatal(err)
	}
	t.Cleanup(srv.Stop)
	srv.CreateBucketWithOpts(fakestorage.CreateBucketOpts{Name: "yrs-test"})
	return &lcEnv{t: t, ctx: ctx, mr: mr, client: srv.Client(), bucket: "yrs-test"}
}

type lcInstance struct {
	s     *Server
	relay *redisrelay.Relay
	ts    *httptest.Server
	// onLastPeer runs after ygo's OnLastPeer: the old room is out of ygo's map
	// and the relay has not been told yet.
	onLastPeer atomic.Pointer[func()]
	// onLastInstance runs inside the relay's last-instance flush.
	onLastInstance atomic.Pointer[func()]
	// lastPeers and unloads count ygo's OnLastPeer and OnUnloadDocument. On the
	// eager path OnUnloadDocument fires once the room's relay teardown returned.
	lastPeers atomic.Int32
	unloads   atomic.Int32
}

// lcFlusher lets a test act inside the relay's last-instance flush.
type lcFlusher struct {
	*VersionedFlusher
	in *lcInstance
}

func (f lcFlusher) FlushLastInstance(ctx context.Context, room string) error {
	if hook := f.in.onLastInstance.Load(); hook != nil {
		(*hook)()
	}
	return f.VersionedFlusher.FlushLastInstance(ctx, room)
}

func (e *lcEnv) instance(name string, cfg *config.Config) *lcInstance {
	t := e.t
	rc := goredis.NewClient(&goredis.Options{Addr: e.mr.Addr()})
	t.Cleanup(func() { _ = rc.Close() })
	locker := gcs.NewRedisLocker(rc, name)
	adapter, err := gcs.New(gcs.Options{Client: e.client, Bucket: e.bucket, Locker: locker, TransientMapKeys: TransientMapKeys()})
	if err != nil {
		t.Fatal(err)
	}
	s := NewWithPersistence(e.ctx, cfg, adapter)
	s.ws.Authorize = func(r *http.Request) (ygws.ConnectionConfig, bool) {
		return ygws.ConnectionConfig{Identity: auth.Identity{UserID: r.URL.Query().Get("user")}}, true
	}
	flusher := gcs.NewFlusher(gcs.FlusherOptions{Adapter: adapter, Locker: locker, StateOf: s.CheckpointState})
	s.EnableVersions(VersionsConfig{
		Store:      redisrelay.NewChangeStore(rc, name),
		Snapshots:  adapter.Versions(0),
		Checkpoint: flusher.FlushRoom,
		Stored: func(ctx context.Context, room string) ([]byte, error) {
			res, err := adapter.Load(ctx, room)
			return res.Update, err
		},
		MarkEvery: 10 * time.Millisecond,
	})
	in := &lcInstance{s: s}
	lastPeer := s.ws.OnLastPeer
	s.ws.OnLastPeer = func(ctx context.Context, room string) {
		lastPeer(ctx, room)
		in.lastPeers.Add(1)
		if hook := in.onLastPeer.Load(); hook != nil {
			(*hook)()
		}
	}
	unload := s.ws.OnUnloadDocument
	s.ws.OnUnloadDocument = func(ctx context.Context, room string) {
		unload(ctx, room)
		in.unloads.Add(1)
	}
	relay, err := redisrelay.New(redisrelay.Options{Addr: e.mr.Addr(), Flusher: lcFlusher{s.VersionedFlusher(flusher), in}})
	if err != nil {
		t.Fatal(err)
	}
	t.Cleanup(func() { _ = relay.Close() })
	if err := s.AttachRedisRelay(e.ctx, relay); err != nil {
		t.Fatal(err)
	}
	s.SetPresence(relay.ActiveInstances)
	s.SetCatchUp(relay.WaitCaughtUp)
	in.relay = relay
	in.ts = httptest.NewServer(s.Handler())
	t.Cleanup(in.ts.Close)
	// Runs first: no room may still write to fake GCS when it stops.
	t.Cleanup(func() {
		ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
		defer cancel()
		_ = s.Shutdown(ctx)
	})
	return in
}

func (p *lcInstance) text() string { return serverText(p.s, lifecycleRoom) }

// waitUnloads waits until ygo has unloaded the room n times on p.
func (p *lcInstance) waitUnloads(t *testing.T, n int32) {
	t.Helper()
	waitFor(t, "the old room's teardown", func() bool { return p.unloads.Load() >= n })
}

func (p *lcInstance) loaded() bool { return p.s.ws.GetDoc(lifecycleRoom) != nil }

// relayActive reports whether this instance's relay holds the room.
func (p *lcInstance) relayActive() bool {
	ctx, cancel := context.WithTimeout(context.Background(), time.Second)
	defer cancel()
	return !errors.Is(p.relay.WaitCaughtUp(ctx, lifecycleRoom), redisrelay.ErrRoomNotActive)
}

func (e *lcEnv) heartbeats(p *lcInstance) int {
	n, err := p.relay.ActiveInstances(e.ctx, lifecycleRoom)
	if err != nil {
		e.t.Fatal(err)
	}
	return n
}

// lcTab is a browser tab: one document that survives reconnects.
type lcTab struct {
	*e2eClient
	user string
}

func openLCTab(t *testing.T, p *lcInstance, user string, clientID uint64) *lcTab {
	t.Helper()
	c := dialE2E(t, p.ts, lifecycleRoom+"?user="+user, clientID)
	c.completeHandshake()
	return &lcTab{e2eClient: c, user: user}
}

// reconnect keeps the tab's document, as y-websocket does.
func (tb *lcTab) reconnect(t *testing.T, p *lcInstance) {
	t.Helper()
	conn, _, err := gws.DefaultDialer.Dial(wsURL(p.ts, lifecycleRoom+"?user="+tb.user), nil)
	if err != nil {
		t.Fatalf("reconnect: %v", err)
	}
	tb.conn = conn
	tb.aw = awareness.New(uint64(tb.doc.ClientID()))
	tb.completeHandshake()
}

// pauseIn returns a hook that, the first time it runs, holds its caller until
// resume, so the test acts meanwhile. reached waits for the hook, bounded, and a
// cleanup resumes it if the test ends first.
func pauseIn(t *testing.T) (hook, reached, resume func()) {
	var once, resumed sync.Once
	in, out := make(chan struct{}), make(chan struct{})
	resume = func() { resumed.Do(func() { close(out) }) }
	t.Cleanup(resume)
	hook = func() {
		once.Do(func() {
			close(in)
			<-out
		})
	}
	reached = func() {
		t.Helper()
		select {
		case <-in:
		case <-time.After(10 * time.Second):
			t.Fatal("the paused callback never ran")
		}
	}
	return hook, reached, resume
}

// within reports whether cond holds before d passes.
func within(d time.Duration, cond func() bool) bool {
	deadline := time.Now().Add(d)
	for time.Now().Before(deadline) {
		if cond() {
			return true
		}
		time.Sleep(10 * time.Millisecond)
	}
	return cond()
}

func idleConfig(d time.Duration) *config.Config {
	cfg := testConfig()
	cfg.RoomIdleTimeout = d
	return cfg
}

// TestRoomLifecycle_RelayEntryDuringTeardownKeepsTheRoomLive: the last peer
// leaves one instance while a user on another keeps editing. A relay entry that
// re-creates the room during its teardown must find a live relay behind it, not
// a room that hears and sends nothing.
func TestRoomLifecycle_RelayEntryDuringTeardownKeepsTheRoomLive(t *testing.T) {
	const rounds = 20
	cutOff, hit := 0, 0
	for i := range rounds {
		t.Run(fmt.Sprintf("round %d", i), func(t *testing.T) {
			e := newLCEnv(t)
			A, B := e.instance("A", testConfig()), e.instance("B", testConfig())
			alice := openLCTab(t, A, "alice", 101)
			defer runtime.KeepAlive(alice)
			bob := openLCTab(t, B, "bob", 202)
			defer runtime.KeepAlive(bob)
			alice.insertText("a1 ")
			waitFor(t, "a1 on B", func() bool { return strings.Contains(B.text(), "a1 ") })

			stop, done := make(chan struct{}), make(chan struct{})
			go func() {
				defer close(done)
				for {
					select {
					case <-stop:
						return
					default:
						bob.insertText("b")
						time.Sleep(2 * time.Millisecond)
					}
				}
			}()
			time.Sleep(100 * time.Millisecond)
			alice.close()
			A.waitUnloads(t, 1)
			time.Sleep(100 * time.Millisecond) // more relay traffic after the teardown
			close(stop)
			<-done
			if !A.loaded() {
				return
			}
			hit++
			live := A.relayActive() && e.heartbeats(A) == 2 &&
				within(3*time.Second, func() bool { return A.text() == B.text() })
			if !live {
				cutOff++
				t.Logf("A holds the room with no live relay: relayActive=%v heartbeats=%d", A.relayActive(), e.heartbeats(A))
			}
		})
	}
	if cutOff > 0 {
		t.Fatalf("a relay entry during teardown left a cut-off room in %d of %d disconnects", cutOff, rounds)
	}
	if hit == 0 {
		t.Fatal("no disconnect had a relay entry re-create the room; the test exercised nothing")
	}
	t.Logf("a relay entry re-created the room in %d of %d disconnects, all live", hit, rounds)
}

// TestRoomLifecycle_ReconnectDuringTeardownKeepsTheRelay: a user reconnects as
// soon as their old room leaves ygo's map. The new room must exchange edits with
// the other instance and keep this instance's heartbeat.
func TestRoomLifecycle_ReconnectDuringTeardownKeepsTheRelay(t *testing.T) {
	const rounds = 20
	cutOff := 0
	for i := range rounds {
		t.Run(fmt.Sprintf("round %d", i), func(t *testing.T) {
			e := newLCEnv(t)
			A, B := e.instance("A", testConfig()), e.instance("B", testConfig())
			alice := openLCTab(t, A, "alice", 101)
			defer runtime.KeepAlive(alice)
			bob := openLCTab(t, B, "bob", 202)
			defer runtime.KeepAlive(bob)
			alice.insertText("a1 ")
			waitFor(t, "a1 on B", func() bool { return strings.Contains(B.text(), "a1 ") })

			alice.close()
			waitFor(t, "the room leaving A's map", func() bool { return !A.loaded() })
			alice.reconnect(t, A)
			A.waitUnloads(t, 1)
			alice.insertText("a2 ")
			bob.insertText("b2 ")
			synced := within(3*time.Second, func() bool {
				return strings.Contains(B.text(), "a2 ") && strings.Contains(A.text(), "b2 ")
			})
			if n := e.heartbeats(A); !synced || n != 2 {
				cutOff++
				t.Logf("A=%q B=%q heartbeats=%d", A.text(), B.text(), n)
			}
		})
	}
	if cutOff > 0 {
		t.Fatalf("an immediate reconnect left the new room cut off in %d of %d", cutOff, rounds)
	}
}

// TestRoomLifecycle_RelayEntryInTheTeardownWindow holds the window open: a relay
// entry lands after ygo removed the room from its map and before it told the
// relay.
func TestRoomLifecycle_RelayEntryInTheTeardownWindow(t *testing.T) {
	e := newLCEnv(t)
	A, B := e.instance("A", testConfig()), e.instance("B", testConfig())
	alice := openLCTab(t, A, "alice", 101)
	defer runtime.KeepAlive(alice)
	bob := openLCTab(t, B, "bob", 202)
	defer runtime.KeepAlive(bob)
	alice.insertText("a1 ")
	waitFor(t, "a1 on B", func() bool { return strings.Contains(B.text(), "a1 ") })

	hook, reached, resume := pauseIn(t)
	A.onLastPeer.Store(&hook)
	alice.close()
	reached()
	bob.insertText("b1 ")
	waitFor(t, "b1 re-creating the room on A", func() bool { return strings.Contains(A.text(), "b1 ") })
	resume()
	A.waitUnloads(t, 1)

	if !A.relayActive() || e.heartbeats(A) != 2 {
		t.Fatalf("the re-created room has no live relay: relayActive=%v heartbeats=%d", A.relayActive(), e.heartbeats(A))
	}
	alice.reconnect(t, A)
	alice.insertText("a2 ")
	bob.insertText("b2 ")
	waitFor(t, "a2 on B and b2 on A", func() bool {
		return strings.Contains(B.text(), "a2 ") && strings.Contains(A.text(), "b2 ")
	})
}

// TestRoomLifecycle_ReconnectDuringTheLastInstanceFlush: a user alone on an
// instance reconnects while the relay is evicting the room as its last instance.
// The new room must keep its heartbeat and the stream.
func TestRoomLifecycle_ReconnectDuringTheLastInstanceFlush(t *testing.T) {
	e := newLCEnv(t)
	A, B := e.instance("A", testConfig()), e.instance("B", testConfig())
	alice := openLCTab(t, A, "alice", 101)
	defer runtime.KeepAlive(alice)
	alice.insertText("a1 ")
	waitFor(t, "a1 on A", func() bool { return strings.Contains(A.text(), "a1 ") })

	rc := goredis.NewClient(&goredis.Options{Addr: e.mr.Addr()})
	defer func() { _ = rc.Close() }()
	stream := "yjs:stream:" + lifecycleRoom
	head, err := rc.XRevRangeN(e.ctx, stream, "+", "-", 1).Result()
	if err != nil || len(head) == 0 {
		t.Fatalf("stream head: %v (%d entries)", err, len(head))
	}

	hook, reached, resume := pauseIn(t)
	A.onLastInstance.Store(&hook)
	alice.close()
	reached()
	alice.reconnect(t, A)
	resume()
	A.waitUnloads(t, 1)

	// A later write would re-create a deleted stream, so look for the old entry.
	if kept, err := rc.XRange(e.ctx, stream, head[0].ID, head[0].ID).Result(); err != nil || len(kept) != 1 {
		t.Fatalf("the stream was deleted under the reconnected room: %v (%d entries)", err, len(kept))
	}
	if !A.relayActive() || e.heartbeats(A) != 1 {
		t.Fatalf("the reconnected room lost its relay: relayActive=%v heartbeats=%d", A.relayActive(), e.heartbeats(A))
	}
	bob := openLCTab(t, B, "bob", 202)
	defer runtime.KeepAlive(bob)
	alice.insertText("a2 ")
	bob.insertText("b2 ")
	waitFor(t, "a2 on B and b2 on A", func() bool {
		return strings.Contains(B.text(), "a2 ") && strings.Contains(A.text(), "b2 ")
	})
}

// TestRoomLifecycle_IdleRoomStaysWhileOthersEditThenGoes: with an idle timeout, a
// room whose last peer left stays loaded and live while the other instance's
// edits keep arriving, and is let go once they stop.
func TestRoomLifecycle_IdleRoomStaysWhileOthersEditThenGoes(t *testing.T) {
	e := newLCEnv(t)
	A, B := e.instance("A", idleConfig(300*time.Millisecond)), e.instance("B", idleConfig(300*time.Millisecond))
	alice := openLCTab(t, A, "alice", 101)
	defer runtime.KeepAlive(alice)
	bob := openLCTab(t, B, "bob", 202)
	defer runtime.KeepAlive(bob)
	alice.insertText("a1 ")
	waitFor(t, "a1 on B", func() bool { return strings.Contains(B.text(), "a1 ") })

	alice.close()
	deadline := time.Now().Add(2500 * time.Millisecond)
	for time.Now().Before(deadline) {
		bob.insertText("b")
		time.Sleep(100 * time.Millisecond)
		if !A.loaded() {
			t.Fatal("A let the room go while edits were arriving")
		}
	}
	if !A.relayActive() || e.heartbeats(A) != 2 {
		t.Fatalf("A's idle room lost its relay: relayActive=%v heartbeats=%d", A.relayActive(), e.heartbeats(A))
	}
	if !within(3*time.Second, func() bool { return A.text() == B.text() }) {
		t.Fatalf("A's idle room fell behind: A=%q B=%q", A.text(), B.text())
	}

	// A rejoin reuses the warm room.
	alice.reconnect(t, A)
	alice.insertText("a2 ")
	waitFor(t, "a2 on B", func() bool { return strings.Contains(B.text(), "a2 ") })
	alice.close()

	if !within(5*time.Second, func() bool { return !A.loaded() && !A.relayActive() && e.heartbeats(A) == 1 }) {
		t.Fatalf("A kept the idle room after the edits stopped: loaded=%v heartbeats=%d", A.loaded(), e.heartbeats(A))
	}
}

// TestRoomLifecycle_ShutdownLetsIdleRoomsGo: ygo's Shutdown leaves rooms with no
// peers loaded; Server.Shutdown must still let the relay release them.
func TestRoomLifecycle_ShutdownLetsIdleRoomsGo(t *testing.T) {
	e := newLCEnv(t)
	A, B := e.instance("A", idleConfig(time.Hour)), e.instance("B", idleConfig(time.Hour))
	alice := openLCTab(t, A, "alice", 101)
	defer runtime.KeepAlive(alice)
	bob := openLCTab(t, B, "bob", 202)
	defer runtime.KeepAlive(bob)
	alice.insertText("a1 ")
	waitFor(t, "a1 on B", func() bool { return strings.Contains(B.text(), "a1 ") })
	alice.close()
	waitFor(t, "alice's disconnect", func() bool { return A.lastPeers.Load() == 1 })
	if !A.loaded() || e.heartbeats(A) != 2 {
		t.Fatalf("A let the idle room go before Shutdown: loaded=%v heartbeats=%d", A.loaded(), e.heartbeats(A))
	}

	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()
	if err := A.s.Shutdown(ctx); err != nil {
		t.Fatal(err)
	}
	if A.relayActive() {
		t.Fatal("A's relay still holds the room after Shutdown")
	}
	if n := e.heartbeats(A); n != 1 {
		t.Fatalf("heartbeats after A shut down = %d, want 1 (B)", n)
	}
	bob.insertText("b1 ")
	waitFor(t, "b1 on B", func() bool { return strings.Contains(B.text(), "b1 ") })
}

// TestRoomLifecycle_ShutdownRefusesNewConnections: a connection opened while
// Shutdown closes rooms is refused, and no room is left loaded.
func TestRoomLifecycle_ShutdownRefusesNewConnections(t *testing.T) {
	e := newLCEnv(t)
	A := e.instance("A", testConfig())
	alice := openLCTab(t, A, "alice", 101)
	defer runtime.KeepAlive(alice)
	alice.insertText("a1 ")
	waitFor(t, "a1 on A", func() bool { return strings.Contains(A.text(), "a1 ") })

	hook, reached, resume := pauseIn(t)
	A.onLastPeer.Store(&hook)
	shut := make(chan error, 1)
	go func() {
		ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
		defer cancel()
		shut <- A.s.Shutdown(ctx)
	}()
	reached()

	conn, resp, err := gws.DefaultDialer.Dial(wsURL(A.ts, lifecycleRoom+"?user=bob"), nil)
	if conn != nil {
		_ = conn.Close()
	}
	if err == nil || resp == nil || resp.StatusCode != http.StatusServiceUnavailable {
		status := 0
		if resp != nil {
			status = resp.StatusCode
		}
		t.Fatalf("a dial during Shutdown got status %d, err %v; want 503", status, err)
	}
	resume()
	select {
	case err := <-shut:
		if err != nil {
			t.Fatal(err)
		}
	case <-time.After(10 * time.Second):
		t.Fatal("Shutdown did not return")
	}
	if rooms := A.s.ws.Rooms(); len(rooms) != 0 {
		t.Fatalf("rooms still loaded after Shutdown: %v", rooms)
	}
}
