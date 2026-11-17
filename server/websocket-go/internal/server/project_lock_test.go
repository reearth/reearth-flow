package server

import (
	"context"
	"errors"
	"net/http/httptest"
	"testing"
	"time"

	gws "github.com/gorilla/websocket"
	"github.com/reearth/ygo/cluster"
	"github.com/reearth/ygo/crdt"
	ygsync "github.com/reearth/ygo/sync"
)

const lockRoom = "550e8400-e29b-41d4-a716-446655440000"

// insertText sends an edit from c, as a browser tab does.
func (c *e2eClient) insertText(s string) {
	txt := c.doc.GetText("content")
	before := c.doc.StateVector()
	c.doc.Transact(func(txn *crdt.Transaction) {
		txt.Insert(txn, txt.Len(), s, nil)
	})
	c.writeSync(ygsync.EncodeUpdate(crdt.EncodeStateAsUpdateV1(c.doc, before)))
}

// closeCode reads until the server closes c and returns the close code.
func (c *e2eClient) closeCode(t *testing.T) int {
	t.Helper()
	_ = c.conn.SetReadDeadline(time.Now().Add(3 * time.Second))
	for {
		if _, _, err := c.conn.ReadMessage(); err != nil {
			var ce *gws.CloseError
			if errors.As(err, &ce) {
				return ce.Code
			}
			t.Fatalf("connection ended without a close frame: %v", err)
		}
	}
}

func serverText(s *Server, room string) string {
	doc := s.WSProvider().GetDoc(room)
	if doc == nil {
		return ""
	}
	return doc.GetText("content").ToString()
}

func waitFor(t *testing.T, what string, cond func() bool) {
	t.Helper()
	deadline := time.Now().Add(3 * time.Second)
	for time.Now().Before(deadline) {
		if cond() {
			return
		}
		time.Sleep(10 * time.Millisecond)
	}
	t.Fatalf("timed out waiting for %s", what)
}

// storedLock is the room's signed lock entry, or ok == false when the room is
// not loaded or holds no signed entry.
func storedLock(s *Server, room string) (l projectLock, ok bool) {
	doc := s.WSProvider().GetDoc(room)
	if doc == nil {
		return projectLock{}, false
	}
	v, _ := doc.GetMap(metadataMapName).Get(projectLockKey)
	return s.signedLock(room, v)
}

// lockEntry is the document value the server writes for l.
func lockEntry(s *Server, room string, l projectLock) map[string]any {
	return map[string]any{"locked": l.Locked, "at": float64(l.At), "sig": s.lockSignature(room, l)}
}

// setLockEntry writes a lock entry from c, as a client editing the document
// could.
func (c *e2eClient) setLockEntry(v map[string]any) {
	m := c.doc.GetMap(metadataMapName)
	before := c.doc.StateVector()
	c.doc.Transact(func(txn *crdt.Transaction) {
		m.Set(txn, projectLockKey, v)
	})
	c.writeSync(ygsync.EncodeUpdate(crdt.EncodeStateAsUpdateV1(c.doc, before)))
}

func TestSignalLock_RefusesWritesFromPeersConnectedBeforeTheLock(t *testing.T) {
	t.Parallel()

	srv := New(testConfig())
	ts := httptest.NewServer(srv.Handler())
	defer ts.Close()

	a := dialE2E(t, ts, lockRoom, 101)
	defer a.close()
	a.completeHandshake()
	// b keeps the room loaded after a is closed.
	b := dialE2E(t, ts, lockRoom, 202)
	defer b.close()
	b.completeHandshake()
	a.insertText("before")
	waitFor(t, "the edit before the lock", func() bool { return serverText(srv, lockRoom) == "before" })

	at := time.UnixMilli(1_700_000_000_000)
	if err := srv.SignalLock(context.Background(), lockRoom, true, at); err != nil {
		t.Fatalf("SignalLock: %v", err)
	}
	if got, ok := storedLock(srv, lockRoom); !ok || got != (projectLock{Locked: true, At: at.UnixMilli()}) {
		t.Fatalf("stored lock = %+v, %v", got, ok)
	}

	a.insertText(" after")
	if code := a.closeCode(t); code != CloseProjectLocked {
		t.Fatalf("close code = %d, want %d", code, CloseProjectLocked)
	}
	if got := serverText(srv, lockRoom); got != "before" {
		t.Fatalf("server text = %q, want the locked write refused", got)
	}
}

func TestSignalLock_UnlockAcceptsWritesAgain(t *testing.T) {
	t.Parallel()

	srv := New(testConfig())
	ts := httptest.NewServer(srv.Handler())
	defer ts.Close()

	a := dialE2E(t, ts, lockRoom, 101)
	defer a.close()
	a.completeHandshake()
	// Locking before the server has read a's handshake would refuse it.
	a.insertText("1")
	waitFor(t, "the edit before the lock", func() bool { return serverText(srv, lockRoom) == "1" })

	ctx := context.Background()
	if err := srv.SignalLock(ctx, lockRoom, true, time.UnixMilli(1000)); err != nil {
		t.Fatalf("lock: %v", err)
	}
	if err := srv.SignalLock(ctx, lockRoom, false, time.UnixMilli(2000)); err != nil {
		t.Fatalf("unlock: %v", err)
	}

	a.insertText("2")
	waitFor(t, "the edit after unlocking", func() bool { return serverText(srv, lockRoom) == "12" })
}

func TestSignalLock_LeavesAnUnloadedRoomAlone(t *testing.T) {
	t.Parallel()

	srv := New(testConfig())
	if err := srv.SignalLock(context.Background(), lockRoom, true, time.UnixMilli(1000)); err != nil {
		t.Fatalf("SignalLock: %v", err)
	}
	if srv.WSProvider().GetDoc(lockRoom) != nil {
		t.Fatal("SignalLock loaded a room no instance holds")
	}
}

func TestSignalLock_ReachesARoomHeldOnlyByAnotherInstance(t *testing.T) {
	t.Parallel()

	relay := cluster.NewMemRelay()
	defer func() { _ = relay.Close() }()
	holder, other := New(testConfig()), New(testConfig())
	for _, s := range []*Server{holder, other} {
		if err := s.AttachRedisRelay(context.Background(), relay); err != nil {
			t.Fatalf("attach relay: %v", err)
		}
	}
	other.SetPresence(func(context.Context, string) (int, error) { return 1, nil })

	ts := httptest.NewServer(holder.Handler())
	defer ts.Close()
	a := dialE2E(t, ts, lockRoom, 101)
	defer a.close()
	a.completeHandshake()
	a.insertText("1")
	waitFor(t, "the edit before the lock", func() bool { return serverText(holder, lockRoom) == "1" })

	if err := other.SignalLock(context.Background(), lockRoom, true, time.UnixMilli(1000)); err != nil {
		t.Fatalf("SignalLock: %v", err)
	}
	waitFor(t, "the lock to reach the holding instance", func() bool {
		return holder.locks.locked(lockRoom, holder.WSProvider().GetDoc(lockRoom))
	})

	a.insertText("x")
	if code := a.closeCode(t); code != CloseProjectLocked {
		t.Fatalf("close code = %d, want %d", code, CloseProjectLocked)
	}
}

func TestSignalLock_ClosesARoomItLoadedForAnotherInstance(t *testing.T) {
	t.Parallel()

	srv := New(testConfig())
	srv.SetPresence(func(context.Context, string) (int, error) { return 1, nil })
	if err := srv.SignalLock(context.Background(), lockRoom, true, time.UnixMilli(1000)); err != nil {
		t.Fatalf("SignalLock: %v", err)
	}
	if srv.WSProvider().GetDoc(lockRoom) != nil {
		t.Fatal("the room loaded only to send the change is still open")
	}
	srv.locks.mu.Lock()
	tracked := len(srv.locks.rooms[lockRoom])
	srv.locks.mu.Unlock()
	if tracked != 0 {
		t.Fatalf("the closed room is still tracked (%d copies)", tracked)
	}
}

func TestSignalLock_SkipsARoomNoInstanceHolds(t *testing.T) {
	t.Parallel()

	srv := New(testConfig())
	srv.SetPresence(func(context.Context, string) (int, error) { return 0, nil })
	if err := srv.SignalLock(context.Background(), lockRoom, true, time.UnixMilli(1000)); err != nil {
		t.Fatalf("SignalLock: %v", err)
	}
	if srv.WSProvider().GetDoc(lockRoom) != nil {
		t.Fatal("SignalLock loaded a room no instance holds")
	}
}

func TestSignalLock_NewestChangeWinsAcrossInstances(t *testing.T) {
	t.Parallel()

	relay := cluster.NewMemRelay()
	defer func() { _ = relay.Close() }()
	x, y := New(testConfig()), New(testConfig())
	for _, s := range []*Server{x, y} {
		if err := s.AttachRedisRelay(context.Background(), relay); err != nil {
			t.Fatalf("attach relay: %v", err)
		}
		s.SetPresence(func(context.Context, string) (int, error) { return 2, nil })
	}
	for i, s := range []*Server{x, y} {
		ts := httptest.NewServer(s.Handler())
		defer ts.Close()
		c := dialE2E(t, ts, lockRoom, uint64(101+i))
		defer c.close()
		c.completeHandshake()
	}

	// Both instances write without waiting for each other, so the document may
	// keep either entry; every instance must still end on the newer change.
	ctx := context.Background()
	errs := make(chan error, 2)
	go func() { errs <- x.SignalLock(ctx, lockRoom, true, time.UnixMilli(2000)) }()
	go func() { errs <- y.SignalLock(ctx, lockRoom, false, time.UnixMilli(1000)) }()
	for range 2 {
		if err := <-errs; err != nil {
			t.Fatalf("SignalLock: %v", err)
		}
	}

	want := projectLock{Locked: true, At: 2000}
	for _, s := range []*Server{x, y} {
		waitFor(t, "the newer lock change", func() bool {
			got, ok := storedLock(s, lockRoom)
			return ok && got == want && s.locks.locked(lockRoom, s.WSProvider().GetDoc(lockRoom))
		})
	}
}

func TestSignalLock_IgnoresAnEntryAPeerWrote(t *testing.T) {
	t.Parallel()

	srv := New(testConfig())
	ts := httptest.NewServer(srv.Handler())
	defer ts.Close()

	a := dialE2E(t, ts, lockRoom, 101)
	defer a.close()
	a.completeHandshake()
	b := dialE2E(t, ts, lockRoom, 202)
	defer b.close()
	b.completeHandshake()

	// An entry far in the future would outrank every real change if trusted.
	a.setLockEntry(map[string]any{"locked": true, "at": float64(9_000_000_000_000), "sig": "forged"})
	waitFor(t, "the peer's entry", func() bool {
		v, _ := srv.WSProvider().GetDoc(lockRoom).GetMap(metadataMapName).Get(projectLockKey)
		_, sig, ok := decodeProjectLock(v)
		return ok && sig == "forged"
	})
	if srv.locks.locked(lockRoom, srv.WSProvider().GetDoc(lockRoom)) {
		t.Fatal("a peer's entry locked the room")
	}
	b.insertText("b")
	waitFor(t, "another peer's edit", func() bool { return serverText(srv, lockRoom) == "b" })

	ctx := context.Background()
	if err := srv.SignalLock(ctx, lockRoom, true, time.UnixMilli(1000)); err != nil {
		t.Fatalf("lock: %v", err)
	}
	if err := srv.SignalLock(ctx, lockRoom, false, time.UnixMilli(2000)); err != nil {
		t.Fatalf("unlock: %v", err)
	}
	if got, ok := storedLock(srv, lockRoom); !ok || got != (projectLock{Locked: false, At: 2000}) {
		t.Fatalf("stored lock = %+v, %v, want the unlock", got, ok)
	}
	if srv.locks.locked(lockRoom, srv.WSProvider().GetDoc(lockRoom)) {
		t.Fatal("the unlock did not take")
	}
}

func TestSignalLock_APeerCannotBringBackAnOlderChange(t *testing.T) {
	t.Parallel()

	srv := New(testConfig())
	ts := httptest.NewServer(srv.Handler())
	defer ts.Close()

	holder := dialE2E(t, ts, lockRoom, 101)
	defer holder.close()
	holder.completeHandshake()
	holder.insertText("1")
	waitFor(t, "the edit before the lock", func() bool { return serverText(srv, lockRoom) == "1" })

	ctx := context.Background()
	lock := projectLock{Locked: true, At: 1000}
	if err := srv.SignalLock(ctx, lockRoom, lock.Locked, time.UnixMilli(lock.At)); err != nil {
		t.Fatalf("lock: %v", err)
	}
	if err := srv.SignalLock(ctx, lockRoom, false, time.UnixMilli(2000)); err != nil {
		t.Fatalf("unlock: %v", err)
	}

	// a joins after the unlock and writes back the lock's own signed entry.
	a := dialE2E(t, ts, lockRoom, 202)
	defer a.close()
	a.completeHandshake()
	a.setLockEntry(lockEntry(srv, lockRoom, lock))

	waitFor(t, "the unlock to be written again", func() bool {
		got, ok := storedLock(srv, lockRoom)
		return ok && got == (projectLock{Locked: false, At: 2000})
	})
	if srv.locks.locked(lockRoom, srv.WSProvider().GetDoc(lockRoom)) {
		t.Fatal("an older entry written back locked the room")
	}
}

func TestSignalLock_ARepeatedChangeIsWrittenOnce(t *testing.T) {
	t.Parallel()

	srv := New(testConfig())
	ts := httptest.NewServer(srv.Handler())
	defer ts.Close()
	a := dialE2E(t, ts, lockRoom, 101)
	defer a.close()
	a.completeHandshake()

	ctx := context.Background()
	at := time.UnixMilli(1000)
	if err := srv.SignalLock(ctx, lockRoom, true, at); err != nil {
		t.Fatalf("lock: %v", err)
	}
	doc := srv.WSProvider().GetDoc(lockRoom)
	before := doc.StateVector()
	if err := srv.SignalLock(ctx, lockRoom, true, at); err != nil {
		t.Fatalf("repeated lock: %v", err)
	}
	if after := doc.StateVector(); after[doc.ClientID()] != before[doc.ClientID()] {
		t.Fatalf("the repeated change was written again: clock %d -> %d", before[doc.ClientID()], after[doc.ClientID()])
	}
}

func TestWatchProjectLock_DoesNotEnforceTheLoadedValue(t *testing.T) {
	t.Parallel()

	srv := New(testConfig())
	doc := crdt.New()
	m := doc.GetMap(metadataMapName)
	doc.Transact(func(txn *crdt.Transaction) {
		m.Set(txn, projectLockKey, lockEntry(srv, lockRoom, projectLock{Locked: true, At: 1000}))
	})

	if err := srv.watchProjectLock(context.Background(), lockRoom, doc); err != nil {
		t.Fatalf("watchProjectLock: %v", err)
	}
	if srv.locks.locked(lockRoom, doc) {
		t.Fatal("a lock value loaded from storage refused writes")
	}

	doc.Transact(func(txn *crdt.Transaction) {
		m.Set(txn, projectLockKey, lockEntry(srv, lockRoom, projectLock{Locked: true, At: 2000}))
	})
	if !srv.locks.locked(lockRoom, doc) {
		t.Fatal("a lock change on a loaded document did not refuse writes")
	}
}

func TestLockSignature_IsBoundToTheRoom(t *testing.T) {
	t.Parallel()

	srv := New(testConfig())
	l := projectLock{Locked: true, At: 1000}
	if _, ok := srv.signedLock(lockRoom, lockEntry(srv, lockRoom, l)); !ok {
		t.Fatal("the server's own entry did not verify")
	}
	if _, ok := srv.signedLock("other-room", lockEntry(srv, lockRoom, l)); ok {
		t.Fatal("an entry copied from another project verified")
	}
	other := testConfig()
	other.APISecret = "another secret"
	if _, ok := New(other).signedLock(lockRoom, lockEntry(srv, lockRoom, l)); ok {
		t.Fatal("an entry signed with another secret verified")
	}
}

func TestProjectLocks_UnloadKeepsTheReloadedCopy(t *testing.T) {
	t.Parallel()

	l := newProjectLocks()
	oldDoc, newDoc := crdt.New(), crdt.New()
	l.add(lockRoom, &roomLock{doc: oldDoc, latest: projectLock{Locked: true, At: 1}, enforced: true})
	l.add(lockRoom, &roomLock{doc: newDoc, latest: projectLock{Locked: true, At: 1}, enforced: true})

	l.remove(lockRoom)
	if l.locked(lockRoom, oldDoc) {
		t.Fatal("the unloaded copy is still tracked")
	}
	if !l.locked(lockRoom, newDoc) {
		t.Fatal("unloading the old copy dropped the reloaded one")
	}

	l.remove(lockRoom)
	if _, ok := l.rooms[lockRoom]; ok {
		t.Fatal("the room is still tracked after its last copy unloaded")
	}
}

func TestDecodeProjectLock(t *testing.T) {
	t.Parallel()

	got, sig, ok := decodeProjectLock(map[string]any{"locked": true, "at": float64(1_700_000_000_123), "sig": "s"})
	if !ok || got != (projectLock{Locked: true, At: 1_700_000_000_123}) || sig != "s" {
		t.Fatalf("decode = %+v, %q, %v", got, sig, ok)
	}
	got, _, ok = decodeProjectLock(map[string]any{"locked": false, "at": int64(5), "sig": "s"})
	if !ok || got != (projectLock{Locked: false, At: 5}) {
		t.Fatalf("decode int64 = %+v, %v", got, ok)
	}
	if _, _, ok := decodeProjectLock(true); ok {
		t.Fatal("a bare bool decoded as a lock entry")
	}
	if _, _, ok := decodeProjectLock(map[string]any{"at": float64(1), "sig": "s"}); ok {
		t.Fatal("an entry without locked decoded")
	}
	if _, _, ok := decodeProjectLock(map[string]any{"locked": true, "at": float64(1)}); ok {
		t.Fatal("an entry without a signature decoded")
	}
}
