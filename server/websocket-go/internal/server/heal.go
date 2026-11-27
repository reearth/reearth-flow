package server

import (
	"context"
	"sync"
	"time"

	"github.com/reearth/ygo/cluster"
	"github.com/reearth/ygo/crdt"
)

// A copy of a room that missed an edit parks every later edit that builds on it,
// in the room and in the browsers connected to it. The periodic sync heals such
// a copy from storage, backing off while it stays parked.
const (
	healFirstRetry = 30 * time.Second
	healMaxRetry   = 15 * time.Minute
)

// gapTaker is a relay that reports, once, that an update for a room failed to
// reach the other instances.
type gapTaker interface {
	TakeOutboundGap(room string) bool
}

// healer tracks, per room, the heal in flight and when the next may start.
type healer struct {
	now func() time.Time

	mu      sync.Mutex
	rooms   map[string]*healState
	active  int
	idle    chan struct{} // closed when active drops to 0; nil while none waits
	stopped bool
}

type healState struct {
	running bool
	next    time.Time
	retry   time.Duration
}

func newHealer() *healer {
	return &healer{now: time.Now, rooms: map[string]*healState{}}
}

// start reports whether a heal of room may begin now, and marks it running.
func (h *healer) start(room string, parked bool) bool {
	h.mu.Lock()
	defer h.mu.Unlock()
	st := h.rooms[room]
	if !parked {
		if st != nil && !st.running {
			delete(h.rooms, room)
		}
		return false
	}
	if h.stopped {
		return false
	}
	if st == nil {
		st = &healState{}
		h.rooms[room] = st
	}
	if st.running || h.now().Before(st.next) {
		return false
	}
	st.running = true
	h.active++
	return true
}

// finish ends room's heal. A room still parked waits twice as long as last
// time before the next, up to healMaxRetry; a healed room starts afresh.
func (h *healer) finish(room string, parked bool) {
	h.mu.Lock()
	defer h.mu.Unlock()
	h.active--
	if h.active == 0 && h.idle != nil {
		close(h.idle)
		h.idle = nil
	}
	st := h.rooms[room]
	if st == nil {
		return
	}
	st.running = false
	if !parked {
		delete(h.rooms, room)
		return
	}
	st.retry = min(max(2*st.retry, healFirstRetry), healMaxRetry)
	st.next = h.now().Add(st.retry)
}

// forget drops the state of rooms no longer loaded.
func (h *healer) forget(loaded []string) {
	keep := make(map[string]struct{}, len(loaded))
	for _, room := range loaded {
		keep[room] = struct{}{}
	}
	h.mu.Lock()
	defer h.mu.Unlock()
	for room, st := range h.rooms {
		if _, ok := keep[room]; !ok && !st.running {
			delete(h.rooms, room)
		}
	}
}

// stop refuses new heals and waits for those in flight until ctx ends.
func (h *healer) stop(ctx context.Context) {
	h.mu.Lock()
	h.stopped = true
	h.mu.Unlock()
	h.waitIdle(ctx)
}

// waitIdle waits until no heal is in flight or ctx ends.
func (h *healer) waitIdle(ctx context.Context) {
	h.mu.Lock()
	if h.active == 0 {
		h.mu.Unlock()
		return
	}
	if h.idle == nil {
		h.idle = make(chan struct{})
	}
	idle := h.idle
	h.mu.Unlock()
	select {
	case <-idle:
	case <-ctx.Done():
	}
}

// isParked reports whether doc holds items or deletions waiting on an edit it
// has not received.
func isParked(doc *crdt.Doc) bool {
	ps := doc.PendingStats()
	return ps.Items > 0 || ps.DeleteRanges > 0
}

// healRooms republishes rooms whose updates the relay lost and starts a heal
// for each room whose copy is parked.
func (s *Server) healRooms(ctx context.Context) {
	if s.shuttingDown.Load() {
		return
	}
	rooms := s.ws.Rooms()
	for _, room := range rooms {
		doc := s.ws.GetDoc(room)
		if doc == nil {
			continue
		}
		s.republishIfGap(ctx, room, doc)
		if s.heals.start(room, isParked(doc)) {
			go s.healRoom(ctx, room)
		}
	}
	s.heals.forget(rooms)
}

// republishIfGap sends room's full state to the other instances once the relay
// reports that an update for it never reached them.
func (s *Server) republishIfGap(ctx context.Context, room string, doc *crdt.Doc) {
	if s.gaps == nil || !s.gaps.TakeOutboundGap(room) {
		return
	}
	err := s.relay.Publish(ctx, cluster.Outbound{Room: room, Kind: cluster.KindSync, Data: encodeFullState(doc)})
	if err != nil {
		s.log.Warn("republishing a room the relay lost an update for failed", "room", room, "err", err)
		return
	}
	s.log.Info("republished a room's full state after the relay lost an update for it", "room", room)
}

// healRoom folds room's stored state into its copy and logs whether that left
// anything parked.
func (s *Server) healRoom(ctx context.Context, room string) {
	parked := true
	defer func() { s.heals.finish(room, parked) }()
	ran, err := s.injectStored(ctx, room)
	if err != nil {
		s.log.Warn("healing a parked room from storage failed", "room", room, "err", err)
	}
	doc := s.ws.GetDoc(room)
	if doc == nil {
		parked = false // unloaded: the next load starts from storage
		return
	}
	if !isParked(doc) {
		parked = false
		if ran {
			s.log.Info("healed a room whose copy had missed an edit, from storage", "room", room)
		}
		return
	}
	ps := doc.PendingStats()
	s.log.Warn("a room's copy is still parked; an edit it builds on is missing from storage",
		"room", room, "parked", ps.Items, "parked_deletes", ps.DeleteRanges, "awaiting_clients", len(ps.MissingFor))
}

// injectStored applies room's stored state to the loaded room the way a relay
// entry is applied: local peers receive what was new and it is not republished.
func (s *Server) injectStored(ctx context.Context, room string) (bool, error) {
	if s.stored == nil || s.shuttingDown.Load() || s.ws.GetDoc(room) == nil {
		return false, nil
	}
	state, err := s.stored(ctx, room)
	if err != nil || len(state) == 0 {
		return false, err
	}
	// Inject loads a room that is not resident, so check again just before it.
	if s.shuttingDown.Load() || s.ws.GetDoc(room) == nil {
		return false, nil
	}
	return true, s.ws.Inject(ctx, cluster.Inbound{Room: room, Kind: cluster.KindSync, Data: state})
}
