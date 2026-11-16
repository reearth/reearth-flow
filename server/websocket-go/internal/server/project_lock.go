package server

import (
	"context"
	"errors"
	"sync"
	"time"

	"github.com/reearth/ygo/crdt"
	ygws "github.com/reearth/ygo/provider/websocket"
)

// projectLockKey is the metadata entry the server writes when a project is
// locked or unlocked: {"locked": bool, "at": unix ms of the change}. Only the
// server writes it. UI clients watch it to learn of a lock change live; the
// lock itself comes from the Project.
const projectLockKey = "projectLock"

// CloseProjectLocked is the close code sent to a peer that writes to a locked
// project's document. The client reconnects, and /auth/verify then makes the
// new connection read-only.
const CloseProjectLocked = 4423

// projectLock is one lock change. At orders changes: a newer change wins
// whatever order the instances apply them in.
type projectLock struct {
	Locked bool
	At     int64
}

// Presence reports how many instances hold a room. Backends without a shared
// coordination store have none, and only the local instance counts.
type Presence func(ctx context.Context, room string) (int, error)

// roomLock is the lock state of one loaded copy of a room's document.
type roomLock struct {
	doc    *crdt.Doc
	latest projectLock
	// enforced is set once a lock change arrives while the document is loaded.
	// The value a document is loaded with is not enforced: a connection opened
	// after the change was already checked by /auth/verify, and a stale value
	// left in storage must not refuse anyone's writes.
	enforced bool
}

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
	for _, rl := range l.rooms[room] {
		if rl.doc == doc {
			return rl.enforced && rl.latest.Locked
		}
	}
	return false
}

// observe records the lock value the document now holds. It returns the newest
// change seen when the document holds an older one, which the caller writes
// again.
func (l *projectLocks) observe(rl *roomLock, cur projectLock) (newest projectLock, reassert bool) {
	l.mu.Lock()
	defer l.mu.Unlock()
	if cur.At < rl.latest.At {
		// Two instances set the entry concurrently and the document kept the
		// older change. Writing the newer one again puts it after both.
		return rl.latest, true
	}
	rl.latest = cur
	rl.enforced = true
	return projectLock{}, false
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
		rl.latest, _ = decodeProjectLock(v)
	}
	s.locks.add(room, rl)
	m.Observe(func(ev crdt.YMapEvent) {
		if _, ok := ev.Keys[projectLockKey]; !ok {
			return
		}
		v, ok := m.Get(projectLockKey)
		if !ok {
			return
		}
		cur, ok := decodeProjectLock(v)
		if !ok {
			return
		}
		if newest, reassert := s.locks.observe(rl, cur); reassert {
			// Observers run inside the transaction's caller; write from outside it.
			go func() {
				if err := s.applyProjectLock(context.Background(), room, newest); err != nil {
					s.log.Warn("project lock reassert failed", "room", room, "err", err)
				}
			}()
		}
	})
	return nil
}

// gateLockedWrite refuses a write to a project locked while the peer was
// connected. Refusing closes the peer, whose reconnect is read-only.
func (s *Server) gateLockedWrite(info ygws.PeerUpdateInfo, apply func() error) error {
	if s.locks.locked(info.Room, info.Doc) {
		s.log.Info("refused write to locked project", "room", info.Room)
		return &ygws.PeerClose{Code: CloseProjectLocked, Reason: "project locked"}
	}
	return apply()
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

func (s *Server) applyProjectLock(ctx context.Context, room string, l projectLock) error {
	err := s.ws.Apply(ctx, room, func(doc *crdt.Doc, transact func(func(*crdt.Transaction))) {
		// GetMap must be called outside transact, which already holds the doc lock.
		m := doc.GetMap(metadataMapName)
		transact(func(txn *crdt.Transaction) {
			// at is stored as a float64 so a JS client reads it as a number.
			m.Set(txn, projectLockKey, map[string]any{"locked": l.Locked, "at": float64(l.At)})
		})
	})
	if errors.Is(err, ygws.ErrNoChanges) {
		return nil
	}
	return err
}

func decodeProjectLock(v any) (projectLock, bool) {
	m, ok := v.(map[string]any)
	if !ok {
		return projectLock{}, false
	}
	locked, ok := m["locked"].(bool)
	if !ok {
		return projectLock{}, false
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
		return projectLock{}, false
	}
	return projectLock{Locked: locked, At: at}, true
}
