package server

import (
	"context"
	"crypto/hmac"
	"crypto/sha256"
	"encoding/hex"
	"errors"
	"strconv"
	"sync"
	"time"

	"github.com/reearth/ygo/crdt"
	ygws "github.com/reearth/ygo/provider/websocket"
)

// projectLockKey is the metadata entry the server writes when a project is
// locked or unlocked: {"locked": bool, "at": unix ms of the change, "sig":
// signature}. Any peer with write access can write a document entry, so the
// server acts only on entries it signed (see lockSignature). UI clients treat
// a change of the entry as a cue to fetch the Project again, which holds the
// lock; they never read the lock from it.
const projectLockKey = "projectLock"

// CloseProjectLocked is the close code sent to a peer that writes to a locked
// project's document. The client reconnects, and /auth/verify then makes the
// new connection read-only.
const CloseProjectLocked = 4423

// projectLock is one lock change. At orders changes: a newer change wins
// whatever order the instances apply them in. The API keeps At increasing for
// each project, so it orders changes made through different API replicas too.
type projectLock struct {
	Locked bool
	At     int64
}

// lockSignature signs a lock change for one room with the secret the API calls
// this server with, which every instance shares. A peer cannot produce it, so
// a peer cannot lock a room, keep it locked, or move another project's entry
// into it through a document copy.
func (s *Server) lockSignature(room string, l projectLock) string {
	mac := hmac.New(sha256.New, []byte(s.cfg.APISecret))
	mac.Write([]byte(room))
	mac.Write([]byte{0})
	mac.Write([]byte(strconv.FormatBool(l.Locked)))
	mac.Write([]byte{0})
	mac.Write([]byte(strconv.FormatInt(l.At, 10)))
	return hex.EncodeToString(mac.Sum(nil))
}

// signedLock reads a lock entry the server signed for room. Anything else,
// including an entry a peer wrote, reads as no entry.
func (s *Server) signedLock(room string, v any) (projectLock, bool) {
	l, sig, ok := decodeProjectLock(v)
	if !ok || !hmac.Equal([]byte(sig), []byte(s.lockSignature(room, l))) {
		return projectLock{}, false
	}
	return l, true
}

// Presence reports how many instances hold a room. Backends without a shared
// coordination store have none, and only the local instance counts.
type Presence func(ctx context.Context, room string) (int, error)

// roomLock is the lock state of one loaded copy of a room's document.
type roomLock struct {
	doc *crdt.Doc
	// latest is the newest signed change seen, starting from the one the
	// document was loaded with, so a peer writing back an older signed entry
	// cannot undo a newer change.
	latest projectLock
	// enforced is set once a lock change arrives, or this instance writes one,
	// while the document is loaded.
	// The value a document is loaded with is not enforced: a connection opened
	// after the change was already checked by /auth/verify.
	enforced bool
	// gates counts the peer updates gateLockedWrite is applying to doc. The
	// gate repairs the entry itself, before the room sends the update on.
	gates int
	// lastWarn and suppressed rate-limit the warning about unsigned entries,
	// which a peer can drive as often as it writes.
	lastWarn   time.Time
	suppressed int
}

// unsignedLockWarnEvery is the most often a loaded document logs a warning
// about lock entries the server did not sign.
const unsignedLockWarnEvery = time.Minute

// projectLocks tracks the loaded documents' lock state for the write gate.
type projectLocks struct {
	mu sync.Mutex
	// rooms lists each room's loaded documents in load order. A room has one,
	// except while a reload overlaps the previous copy's unload.
	rooms map[string][]*roomLock
}

func newProjectLocks() *projectLocks {
	return &projectLocks{rooms: map[string][]*roomLock{}}
}

func (l *projectLocks) add(room string, rl *roomLock) {
	l.mu.Lock()
	defer l.mu.Unlock()
	l.rooms[room] = append(l.rooms[room], rl)
}

// remove drops the room's oldest copy. Unloads follow loads, so the oldest copy
// is the one being unloaded; if two unloads ever ran out of order, both copies
// are gone by the second one anyway.
func (l *projectLocks) remove(room string) {
	l.mu.Lock()
	defer l.mu.Unlock()
	rls := l.rooms[room]
	if len(rls) <= 1 {
		delete(l.rooms, room)
		return
	}
	l.rooms[room] = rls[1:]
}

func (l *projectLocks) locked(room string, doc *crdt.Doc) bool {
	l.mu.Lock()
	defer l.mu.Unlock()
	rl := l.get(room, doc)
	return rl != nil && rl.enforced && rl.latest.Locked
}

// get returns the state of room's loaded copy doc, or nil. l.mu must be held.
func (l *projectLocks) get(room string, doc *crdt.Doc) *roomLock {
	for _, rl := range l.rooms[room] {
		if rl.doc == doc {
			return rl
		}
	}
	return nil
}

// observe records the lock value the document now holds. It reports whether
// the document holds an older change than the newest seen, which the caller
// must then repair.
func (l *projectLocks) observe(rl *roomLock, cur projectLock) (stale bool) {
	l.mu.Lock()
	defer l.mu.Unlock()
	if cur.At < rl.latest.At {
		// Two instances set the entry concurrently and the document kept the
		// older change, or a peer wrote back an older entry. Writing the newer
		// one again puts it after both.
		return true
	}
	rl.latest = cur
	rl.enforced = true
	return false
}

// newest returns the newest signed change seen for rl, if any. At is the unix
// ms of a real change, so a zero value means none.
func (l *projectLocks) newest(rl *roomLock) (projectLock, bool) {
	l.mu.Lock()
	defer l.mu.Unlock()
	return rl.latest, rl.latest != (projectLock{})
}

// beginGate records that a peer update is being applied to room's copy doc and
// returns that copy's state, or nil when doc is not tracked.
func (l *projectLocks) beginGate(room string, doc *crdt.Doc) *roomLock {
	l.mu.Lock()
	defer l.mu.Unlock()
	rl := l.get(room, doc)
	if rl != nil {
		rl.gates++
	}
	return rl
}

// endGate ends a beginGate.
func (l *projectLocks) endGate(rl *roomLock) {
	l.mu.Lock()
	defer l.mu.Unlock()
	rl.gates--
}

// gated reports whether a peer update is being applied to rl's document.
func (l *projectLocks) gated(rl *roomLock) bool {
	l.mu.Lock()
	defer l.mu.Unlock()
	return rl.gates > 0
}

// warnUnsigned reports whether to log an unsigned entry now, and how many were
// left unlogged since the last warning.
func (l *projectLocks) warnUnsigned(rl *roomLock, now time.Time) (warn bool, suppressed int) {
	l.mu.Lock()
	defer l.mu.Unlock()
	if !rl.lastWarn.IsZero() && now.Sub(rl.lastWarn) < unsignedLockWarnEvery {
		rl.suppressed++
		return false, 0
	}
	suppressed = rl.suppressed
	rl.lastWarn, rl.suppressed = now, 0
	return true, suppressed
}

// wrote records and enforces a change this instance is about to write to doc.
// Observers run after the write lets go of the document and can read a
// concurrent older change instead, so they must already know the newer one to
// write it again, and the gate must not wait for that round trip.
func (l *projectLocks) wrote(room string, doc *crdt.Doc, c projectLock) {
	l.mu.Lock()
	defer l.mu.Unlock()
	for _, rl := range l.rooms[room] {
		if rl.doc == doc && c.At > rl.latest.At {
			rl.latest = c
			rl.enforced = true
		}
	}
}

// enableProjectLock installs the hooks that keep a locked project's document
// read-only for peers connected before the lock.
func (s *Server) enableProjectLock() {
	s.locks = newProjectLocks()
	s.ws.OnLoadDocument = s.watchProjectLock
	s.ws.OnUnloadDocument = func(_ context.Context, room string) { s.locks.remove(room) }
	s.ws.OnPeerUpdate = s.gateLockedWrite
}

// SetPresence makes SignalLock reach rooms held only by other instances.
func (s *Server) SetPresence(p Presence) { s.presence = p }

func (s *Server) watchProjectLock(_ context.Context, room string, doc *crdt.Doc) error {
	m := doc.GetMap(metadataMapName)
	rl := &roomLock{doc: doc}
	if v, ok := m.Get(projectLockKey); ok {
		// An entry the server did not sign reads as no change. One written
		// before entries were repaired came back with the document; the next
		// peer update repairs it (see gateLockedWrite).
		rl.latest, _ = s.signedLock(room, v)
	}
	s.locks.add(room, rl)
	m.Observe(func(ev crdt.YMapEvent) {
		if _, ok := ev.Keys[projectLockKey]; !ok {
			return
		}
		v, present := m.Get(projectLockKey)
		if cur, signed := s.signedLock(room, v); signed {
			if !s.locks.observe(rl, cur) {
				return
			}
		} else if present {
			if warn, n := s.locks.warnUnsigned(rl, time.Now()); warn {
				s.log.Warn("repairing a project lock entry the server did not sign", "room", room, "suppressed", n)
			}
		} else if _, ok := s.locks.newest(rl); !ok {
			return // removed, and there is no signed change to put back
		}
		if s.locks.gated(rl) {
			// gateLockedWrite repairs it before the room sends the update on.
			// A repair from here could reach the document first but other
			// peers only after the update.
			return
		}
		// Observers run inside the transaction's caller; write from outside it.
		go s.repairProjectLock(context.Background(), room, rl)
	})
	return nil
}

// lockEntryStale reports whether rl's document holds a lock entry other than
// the newest signed change seen: one the server did not sign, an older change,
// or none where there was one.
func (s *Server) lockEntryStale(room string, rl *roomLock) bool {
	v, present := rl.doc.GetMap(metadataMapName).Get(projectLockKey)
	newest, known := s.locks.newest(rl)
	if !present {
		return known
	}
	cur, signed := s.signedLock(room, v)
	return !signed || cur.At < newest.At
}

// repairProjectLock writes the newest signed change seen back into rl's
// document, or removes the entry when there is none, so a peer's entry neither
// stays in the document and its storage nor keeps clients fetching the project.
func (s *Server) repairProjectLock(ctx context.Context, room string, rl *roomLock) {
	if s.ws.GetDoc(room) != rl.doc {
		return // unloaded or reloaded since: nobody to hold to the change
	}
	if !s.lockEntryStale(room, rl) {
		return // repaired already, or a newer change landed
	}
	var err error
	if newest, ok := s.locks.newest(rl); ok {
		err = s.applyProjectLock(ctx, room, newest)
	} else {
		err = s.clearProjectLock(ctx, room)
	}
	if err != nil {
		s.log.Warn("project lock repair failed", "room", room, "err", err)
	}
}

// clearProjectLock removes a lock entry the server did not sign from the
// room's document. A signed entry is left alone: it landed after the caller
// found none.
func (s *Server) clearProjectLock(ctx context.Context, room string) error {
	err := s.ws.Apply(ctx, room, func(doc *crdt.Doc, transact func(func(*crdt.Transaction))) {
		// GetMap must be called outside transact, which already holds the doc lock.
		m := doc.GetMap(metadataMapName)
		v, ok := m.Get(projectLockKey)
		if !ok {
			return
		}
		if _, ok := s.signedLock(room, v); ok {
			return
		}
		transact(func(txn *crdt.Transaction) {
			m.Delete(txn, projectLockKey)
		})
	})
	if errors.Is(err, ygws.ErrNoChanges) {
		err = nil
	}
	return err
}

// gateLockedWrite refuses a write to a project locked while the peer was
// connected. Refusing closes the peer, whose reconnect is read-only. A write it
// lets through is recorded as a user edit for versioning.
//
// A write that leaves the lock entry other than the newest signed change is
// repaired before the hook returns. The room sends the peer's update on only
// after that, so other peers receive the repair first. It builds on the peer's
// entry, so they apply both together and see no change to the entry, which
// would otherwise make each of them fetch the project again. The document is
// read once the update is applied, rather than relying on its observer, which
// can run after the hook returns.
func (s *Server) gateLockedWrite(info ygws.PeerUpdateInfo, apply func() error) error {
	if s.locks.locked(info.Room, info.Doc) {
		s.log.Info("refused write to locked project", "room", info.Room)
		return &ygws.PeerClose{Code: CloseProjectLocked, Reason: "project locked"}
	}
	rl := s.locks.beginGate(info.Room, info.Doc)
	err := s.applyAndRecord(info, apply)
	if rl == nil {
		return err
	}
	// An observer that ran while the gate was open left the repair to it, and
	// ran after the change it saw, so the read below sees that change too.
	s.locks.endGate(rl)
	s.repairProjectLock(context.Background(), info.Room, rl)
	return err
}

// SignalLock records a project lock change in the room's live document, which
// reaches every instance holding the room. Writers connected before a lock lose
// write access on every instance; connections opened afterwards are already
// read-only through /auth/verify. A room no instance holds is left alone.
func (s *Server) SignalLock(ctx context.Context, room string, locked bool, at time.Time) error {
	if s.ws.GetDoc(room) == nil {
		if s.presence == nil {
			return nil
		}
		n, err := s.presence(ctx, room)
		if err != nil {
			// Better to load the room than to miss writers on another instance.
			s.log.Warn("project lock presence check failed", "room", room, "err", err)
		} else if n == 0 {
			return nil
		}
	}
	return s.applyProjectLock(ctx, room, projectLock{Locked: locked, At: at.UnixMilli()})
}

// applyProjectLock writes l into the room's document. A room this instance did
// not hold is loaded only to send the change to the instances that do, and is
// closed again: it has no peers, and left open it would count this instance as
// a holder of the room for as long as the process runs.
func (s *Server) applyProjectLock(ctx context.Context, room string, l projectLock) error {
	loaded := s.ws.GetDoc(room) != nil
	sig := s.lockSignature(room, l)
	err := s.ws.Apply(ctx, room, func(doc *crdt.Doc, transact func(func(*crdt.Transaction))) {
		// GetMap must be called outside transact, which already holds the doc lock.
		m := doc.GetMap(metadataMapName)
		if v, ok := m.Get(projectLockKey); ok {
			if cur, ok := s.signedLock(room, v); ok && cur == l {
				return // a retried call: the document already holds this change
			}
		}
		s.locks.wrote(room, doc, l)
		transact(func(txn *crdt.Transaction) {
			// at is stored as a float64 so a JS client reads it as a number.
			m.Set(txn, projectLockKey, map[string]any{"locked": l.Locked, "at": float64(l.At), "sig": sig})
		})
	})
	if errors.Is(err, ygws.ErrNoChanges) {
		err = nil
	}
	if !loaded {
		s.closeLoadedRoom(room, "a project lock change")
	}
	return err
}

func decodeProjectLock(v any) (l projectLock, sig string, ok bool) {
	m, ok := v.(map[string]any)
	if !ok {
		return projectLock{}, "", false
	}
	locked, ok := m["locked"].(bool)
	if !ok {
		return projectLock{}, "", false
	}
	sig, ok = m["sig"].(string)
	if !ok {
		return projectLock{}, "", false
	}
	var at int64
	switch n := m["at"].(type) {
	case float64:
		at = int64(n)
	case float32:
		at = int64(n)
	case int64:
		at = n
	case int32:
		at = int64(n)
	case int:
		at = int64(n)
	default:
		return projectLock{}, "", false
	}
	return projectLock{Locked: locked, At: at}, sig, true
}
