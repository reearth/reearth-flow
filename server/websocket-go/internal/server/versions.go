package server

import (
	"context"
	"errors"
	"fmt"
	"time"

	"github.com/reearth/ygo/crdt"
	ygws "github.com/reearth/ygo/provider/websocket"

	"github.com/reearth/reearth-flow/websocket-go/internal/auth"
	"github.com/reearth/reearth-flow/websocket-go/internal/versions"
)

// catchUpTimeout bounds the wait for a room loaded to save a version to
// replay what the instances holding it have published.
const catchUpTimeout = 10 * time.Second

// maxVersionCheckEvery caps how long a changed room can wait past its
// interval before the periodic trigger notices it.
const maxVersionCheckEvery = time.Minute

// ErrVersionsDisabled is what SaveVersion returns before EnableVersions.
var ErrVersionsDisabled = errors.New("server: versions are not enabled")

// VersionsConfig wires versioning into the Server.
type VersionsConfig struct {
	// Store holds the change records, shared by every instance.
	Store versions.Store
	// Snapshots stores the versions.
	Snapshots versions.Snapshots
	// Lock, when set, runs fn while no other instance saves room.
	Lock func(ctx context.Context, room string, fn func(context.Context) error) error
	// Checkpoint rewrites a room's persisted full state after a version.
	Checkpoint func(ctx context.Context, room string) error
	// Stored returns a room's persisted full state, for a room no instance
	// holds.
	Stored func(ctx context.Context, room string) ([]byte, error)
	// Notify is told of every version saved; it must not block.
	Notify func(versions.Saved)
	// Every is how long a room's first unsaved edit waits before the room is
	// saved as a version, if nothing saved it sooner. <= 0 disables it.
	Every time.Duration
	// MarkEvery is how often a busy room's edits reach Store (tests); <= 0
	// uses versions.DefaultMarkEvery.
	MarkEvery time.Duration
}

// CatchUp waits until this instance's copy of a room holds everything the
// other instances holding it had published when it was called.
type CatchUp func(ctx context.Context, room string) error

// EnableVersions makes the Server record user edits and save versions. Call it
// before serving.
func (s *Server) EnableVersions(cfg VersionsConfig) {
	s.changes = cfg.Store
	s.tracker = versions.NewTracker(cfg.Store, cfg.MarkEvery, s.log)
	s.stored = cfg.Stored
	s.versionEvery = cfg.Every
	s.versions = &versions.Saver{
		Tracker:    s.tracker,
		Store:      cfg.Store,
		Snapshots:  cfg.Snapshots,
		Lock:       cfg.Lock,
		Checkpoint: cfg.Checkpoint,
		Notify:     cfg.Notify,
		Log:        s.log,
	}
	s.ws.OnLastPeer = s.lastPeerLeft
}

// WaitVersions waits until every version save in progress has finished,
// whatever started it, including the put-back of one its trigger cancelled, or
// until ctx ends. ygo runs OnLastPeer after its Shutdown stops waiting, so a
// hook that has not fired yet is not covered; that path serves only
// deployments without a shared store, whose change records live in this
// process anyway.
func (s *Server) WaitVersions(ctx context.Context) error {
	if err := s.waitLeaveSaves(ctx); err != nil {
		return err
	}
	if s.versions == nil {
		return nil
	}
	return s.versions.Wait(ctx)
}

// waitLeaveSaves waits for the saves lastPeerLeft started, some of which may
// not have reached the Saver yet.
func (s *Server) waitLeaveSaves(ctx context.Context) error {
	for {
		s.leaveMu.Lock()
		if s.leaveSaves == 0 {
			s.leaveMu.Unlock()
			return nil
		}
		if s.leaveIdle == nil {
			s.leaveIdle = make(chan struct{})
		}
		idle := s.leaveIdle
		s.leaveMu.Unlock()
		select {
		case <-idle:
		case <-ctx.Done():
			return ctx.Err()
		}
	}
}

func (s *Server) leaveSaveStarted() {
	s.leaveMu.Lock()
	s.leaveSaves++
	s.leaveMu.Unlock()
}

func (s *Server) leaveSaveDone() {
	s.leaveMu.Lock()
	defer s.leaveMu.Unlock()
	s.leaveSaves--
	if s.leaveSaves == 0 && s.leaveIdle != nil {
		close(s.leaveIdle)
		s.leaveIdle = nil
	}
}

// SetCatchUp lets SaveVersion read a room another instance holds from the
// cluster relay.
func (s *Server) SetCatchUp(c CatchUp) { s.catchUp = c }

// applyAndRecord applies a peer's update and records it as a user edit when
// the room did not already hold everything in it: a reconnecting client
// re-sends what it holds, and that is no edit. The record names the items the
// update touched, so a version can tell whether it holds the edit.
func (s *Server) applyAndRecord(info ygws.PeerUpdateInfo, apply func() error) error {
	if s.tracker == nil {
		return apply()
	}
	held, err := crdt.DocContainsUpdateV1(info.Doc, info.Update)
	if aerr := apply(); aerr != nil {
		return aerr
	}
	if err != nil {
		// apply decoded it, so this should not happen; count it rather than
		// lose an edit.
		s.log.Warn("could not compare a peer update with the room; recording it as an edit", "room", info.Room, "err", err)
	} else if held {
		return nil
	}
	ids, err := crdt.ContentIDsFromUpdateV1(info.Update)
	if err != nil {
		s.log.Warn("could not read the items a peer update touched; recording the edit without them", "room", info.Room, "err", err)
	}
	s.tracker.Edit(info.Room, auth.UserIDOf(info.Identity), ids)
	return nil
}

// SaveVersion saves room's document as a version labelled label, if users
// changed it since its last version. It saves from wherever the latest state
// is: this instance's copy of the room; else, when other instances hold the
// room, a copy loaded here that replays what they published; else the
// persisted state.
func (s *Server) SaveVersion(ctx context.Context, room, label string) (versions.Result, error) {
	if s.versions == nil {
		return versions.Result{}, ErrVersionsDisabled
	}
	if s.ws.GetDoc(room) != nil || s.presence == nil {
		return s.versions.Save(ctx, room, label, s.currentState(room))
	}
	n, err := s.presence(ctx, room)
	if err != nil {
		// Better to load the room than to miss edits another instance holds.
		s.log.Warn("version presence check failed", "room", room, "err", err)
	} else if n == 0 {
		return s.versions.Save(ctx, room, label, s.currentState(room))
	}
	return s.saveFromHolders(ctx, room, label)
}

// saveFromHolders loads room here and saves from that copy once the relay has
// replayed what the instances holding it published. The room is closed again
// afterwards unless a peer joined it meanwhile.
func (s *Server) saveFromHolders(ctx context.Context, room, label string) (versions.Result, error) {
	loaded := s.ws.GetDoc(room) != nil
	var (
		res  versions.Result
		serr error
	)
	// Apply loads the room and keeps it loaded until fn returns. fn changes
	// nothing, so Apply returns ErrNoChanges.
	err := s.ws.Apply(ctx, room, func(doc *crdt.Doc, _ func(func(*crdt.Transaction))) {
		res, serr = s.versions.Save(ctx, room, label, s.caughtUpState(room, doc))
	})
	if !loaded {
		s.closeLoadedRoom(room, "a version")
	}
	if err != nil && !errors.Is(err, ygws.ErrNoChanges) {
		return versions.Result{}, err
	}
	return res, serr
}

// caughtUpState reads doc, this instance's copy of room, once the relay has
// replayed what the other instances holding the room had published. The saver
// calls it after taking the change record, so the wait covers the edits the
// record names.
func (s *Server) caughtUpState(room string, doc *crdt.Doc) func(context.Context) ([]byte, error) {
	return func(ctx context.Context) ([]byte, error) {
		if s.catchUp != nil {
			cctx, cancel := context.WithTimeout(ctx, catchUpTimeout)
			defer cancel()
			if err := s.catchUp(cctx, room); err != nil {
				return nil, fmt.Errorf("server: replay room before version: %w", err)
			}
		}
		return s.completeState(ctx, room, doc)
	}
}

// completeState returns doc's full state for a version. A copy holding parked
// updates missed an edit another instance applied and persisted, so the stored
// state is folded in to supply it.
func (s *Server) completeState(ctx context.Context, room string, doc *crdt.Doc) ([]byte, error) {
	live := encodeFullState(doc)
	if doc.PendingStats().Items == 0 || s.stored == nil {
		return live, nil
	}
	stored, err := s.stored(ctx, room)
	if err != nil {
		return nil, fmt.Errorf("server: read stored state to complete a version: %w", err)
	}
	if len(stored) == 0 {
		return live, nil
	}
	merged, err := crdt.MergeUpdatesV1(live, stored)
	if err != nil {
		return nil, fmt.Errorf("server: complete a version with the stored state: %w", err)
	}
	return merged, nil
}

// currentState reads room's full state from this instance's copy, caught up
// with the other instances, or from persistence when this instance does not
// hold the room.
func (s *Server) currentState(room string) func(context.Context) ([]byte, error) {
	return func(ctx context.Context) ([]byte, error) {
		if doc := s.ws.GetDoc(room); doc != nil {
			return s.caughtUpState(room, doc)(ctx)
		}
		if s.stored == nil {
			return nil, nil
		}
		return s.stored(ctx, room)
	}
}

// StartPeriodicVersions saves a version of each room this instance holds once
// its first unsaved edit is Every old, until ctx is cancelled. Every instance
// holding a room runs it; the change record makes one of them save it.
func (s *Server) StartPeriodicVersions(ctx context.Context) {
	if s.versions == nil || s.versionEvery <= 0 {
		return
	}
	check := s.versionEvery / 4
	if check > maxVersionCheckEvery {
		check = maxVersionCheckEvery
	}
	t := time.NewTicker(check)
	defer t.Stop()
	for {
		select {
		case <-ctx.Done():
			return
		case <-t.C:
			s.saveDueVersions(ctx)
		}
	}
}

func (s *Server) saveDueVersions(ctx context.Context) {
	now := time.Now()
	for _, room := range s.ws.Rooms() {
		first, ok, err := s.changes.FirstEdit(ctx, room)
		if err != nil {
			s.log.Warn("reading a change record failed", "room", room, "err", err)
			continue
		}
		// A record without a time (none should exist) counts from now, so it
		// does not look infinitely old.
		if !ok || first.IsZero() || now.Sub(first) < s.versionEvery {
			continue
		}
		s.logVersion("periodic", room, s.saveAuto(ctx, room))
	}
}

func (s *Server) saveAuto(ctx context.Context, room string) error {
	_, err := s.versions.Save(ctx, room, versions.AutoLabel, s.currentState(room))
	return err
}

// logVersion logs a failed automatic version; a version saved whose
// checkpoint then failed is logged as such.
func (s *Server) logVersion(trigger, room string, err error) {
	switch {
	case err == nil:
	case errors.Is(err, versions.ErrCheckpoint):
		s.log.Warn("version saved but checkpoint refresh failed", "trigger", trigger, "room", room, "err", err)
	default:
		s.log.Warn("version failed", "trigger", trigger, "room", room, "err", err)
	}
}

// lastPeerLeft saves a version when the last peer leaves a room, on a
// deployment without a shared coordination store: there this instance is the
// only one. With a shared store, the relay calls VersionedFlusher's
// FlushLastInstance when the last instance lets the room go instead.
func (s *Server) lastPeerLeft(_ context.Context, room string) {
	if s.presence != nil {
		return
	}
	s.leaveSaveStarted()
	go func() {
		defer s.leaveSaveDone()
		s.logVersion("last peer left", room, s.saveAuto(context.Background(), room))
	}()
}

// RoomFlusher persists a room's state for the cluster relay.
type RoomFlusher interface {
	FlushRoom(ctx context.Context, room string) error
}

// VersionedFlusher wraps the relay's flusher so that, when the last instance
// lets a room go, the room is first saved as a version if users changed it. The
// relay's checkpoints of a live room still only flush.
func (s *Server) VersionedFlusher(inner RoomFlusher) *VersionedFlusher {
	return &VersionedFlusher{s: s, inner: inner}
}

// VersionedFlusher is the relay's flusher with versioning on last-instance
// eviction.
type VersionedFlusher struct {
	s     *Server
	inner RoomFlusher
}

// FlushRoom checkpoints a room without saving a version.
func (f *VersionedFlusher) FlushRoom(ctx context.Context, room string) error {
	return f.inner.FlushRoom(ctx, room)
}

// FlushLastInstance saves the room as a version if users changed it, which
// rewrites the checkpoint too, and otherwise only checkpoints it.
func (f *VersionedFlusher) FlushLastInstance(ctx context.Context, room string) error {
	if f.s.versions == nil {
		return f.inner.FlushRoom(ctx, room)
	}
	res, err := f.s.versions.Save(ctx, room, versions.AutoLabel, f.s.currentState(room))
	if res.Saved {
		// nil, or the checkpoint failed: the relay then keeps the room's
		// stream, as it does when its own flush fails.
		return err
	}
	switch {
	case err == nil:
	case errors.Is(err, versions.ErrBusy):
		// Another instance is saving this room. This instance's edits were
		// written to the record before waiting, so that save, or the room's
		// next one, includes them.
		f.s.log.Info("another instance is saving this room; leaving the version to it", "room", room)
	default:
		f.s.log.Warn("version on leave failed", "room", room, "err", err)
	}
	return f.inner.FlushRoom(ctx, room)
}

// closeLoadedRoom closes a room this instance loaded only to act on it. It has
// no peers, and left open it would count this instance as a holder of the room
// for as long as the process runs.
func (s *Server) closeLoadedRoom(room, purpose string) {
	if err := s.ws.CloseRoom(room, false); err != nil &&
		!errors.Is(err, ygws.ErrRoomHasPeers) && !errors.Is(err, ygws.ErrRoomNotFound) {
		s.log.Warn("closing a room loaded for "+purpose+" failed", "room", room, "err", err)
	}
}
