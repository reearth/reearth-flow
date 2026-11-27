package redis

import (
	"context"
	"strconv"
	"sync"
	"time"

	goredis "github.com/redis/go-redis/v9"
)

// Load fences. ygo loads a room from storage before it calls RoomActivated, so
// the relay hears of a room only after the room has read storage. A room that
// loaded storage lacking some update needs that update from the stream, and
// until a pass over the stream that began after the load has put it there, no
// trim or stream delete, on any instance, may remove it.
//
// LoadStarting fences each load before it reads storage: locally, and in the
// Go-only doc:loads:{room} hash (field = clientId, value = deadline in unix
// seconds) so other instances see it. A fence clears when a catch-up pass that
// began after the load completes, when the load fails, or at its deadline.
const (
	// loadFenceTTL is a fence's life without a pass refreshing it.
	loadFenceTTL = 2 * time.Minute
	// loadFenceMax caps a fence from its load's start, however often a pass
	// refreshes it, so a catch-up that keeps failing cannot hold trims forever.
	loadFenceMax = 10 * time.Minute
	// loadFenceWriteTimeout bounds the fence write a load waits on: the load's own
	// context lives as long as the server.
	loadFenceWriteTimeout = 2 * time.Second
)

// loadsKey is the per-doc load-fence hash. Go-only, like cursorsKey.
func loadsKey(docID string) string { return "doc:loads:" + docID }

// loadFence is one room's pending loads on this instance.
type loadFence struct {
	// io orders this room's fence writes, so a pass's clear cannot land after a
	// newer load's write and remove it. Never taken under Relay.mu.
	io sync.Mutex
	// last is the newest load's sequence number (Relay.loadSeq); 0 if none yet.
	last    uint64
	pending map[uint64]fenceEntry
}

type fenceEntry struct {
	started  time.Time
	deadline time.Time
}

// fenceLocked returns room's fence, creating it. Caller holds r.mu.
func (r *Relay) fenceLocked(room string) *loadFence {
	f, ok := r.fences[room]
	if !ok {
		f = &loadFence{pending: make(map[uint64]fenceEntry)}
		r.fences[room] = f
	}
	return f
}

// pruneLocked drops expired loads and returns the latest remaining deadline,
// zero when none remain. Caller holds r.mu.
func (r *Relay) pruneLocked(room string, f *loadFence) time.Time {
	now := r.now()
	var latest time.Time
	for seq, e := range f.pending {
		if !e.deadline.After(now) {
			delete(f.pending, seq)
			r.log.Warn("relay load fence expired before a pass covered it; trims of the room may resume", "room", room)
			continue
		}
		if e.deadline.After(latest) {
			latest = e.deadline
		}
	}
	return latest
}

// LoadStarting fences a load of room before it reads storage. Call the returned
// func with the load's result: a failed load drops its fence at once. A Redis
// failure is logged and never fails or stalls the load past
// loadFenceWriteTimeout; the local fence still holds.
func (r *Relay) LoadStarting(ctx context.Context, room string) func(err error) {
	r.mu.Lock()
	f := r.fenceLocked(room)
	r.mu.Unlock()

	f.io.Lock()
	r.mu.Lock()
	r.loadSeq++
	seq := r.loadSeq
	f.last = seq
	now := r.now()
	f.pending[seq] = fenceEntry{started: now, deadline: now.Add(loadFenceTTL)}
	latest := r.pruneLocked(room, f)
	r.mu.Unlock()
	r.writeFence(ctx, room, latest)
	f.io.Unlock()

	return func(err error) {
		if err == nil {
			return
		}
		f.io.Lock()
		defer f.io.Unlock()
		r.mu.Lock()
		delete(f.pending, seq)
		latest := r.pruneLocked(room, f)
		r.mu.Unlock()
		r.writeFence(ctx, room, latest)
	}
}

// passBegin returns the newest load a pass starting now covers.
func (r *Relay) passBegin(room string) uint64 {
	r.mu.Lock()
	defer r.mu.Unlock()
	if f, ok := r.fences[room]; ok {
		return f.last
	}
	return 0
}

// passRefresh extends the loads a running pass covers, each up to its cap.
func (r *Relay) passRefresh(ctx context.Context, room string, snapshot uint64) {
	r.mu.Lock()
	f, ok := r.fences[room]
	r.mu.Unlock()
	if !ok {
		return
	}
	f.io.Lock()
	defer f.io.Unlock()
	r.mu.Lock()
	now := r.now()
	for seq, e := range f.pending {
		if seq > snapshot {
			continue
		}
		d := now.Add(loadFenceTTL)
		if limit := e.started.Add(loadFenceMax); d.After(limit) {
			d = limit
		}
		e.deadline = d
		f.pending[seq] = e
	}
	latest := r.pruneLocked(room, f)
	r.mu.Unlock()
	if !latest.IsZero() {
		r.writeFence(ctx, room, latest)
	}
}

// passEnd clears the loads a completed pass covered.
func (r *Relay) passEnd(ctx context.Context, room string, snapshot uint64) {
	r.mu.Lock()
	f, ok := r.fences[room]
	r.mu.Unlock()
	if !ok {
		return
	}
	f.io.Lock()
	defer f.io.Unlock()
	r.mu.Lock()
	for seq := range f.pending {
		if seq <= snapshot {
			delete(f.pending, seq)
		}
	}
	latest := r.pruneLocked(room, f)
	r.mu.Unlock()
	r.writeFence(ctx, room, latest)
}

// localFenceLive reports whether a load of room on this instance is still fenced.
func (r *Relay) localFenceLive(room string) bool {
	r.mu.Lock()
	defer r.mu.Unlock()
	f, ok := r.fences[room]
	return ok && !r.pruneLocked(room, f).IsZero()
}

// loadGen is the newest load of room on this instance; it changes whenever a
// load starts, even one that is fenced and cleared again in between two reads.
func (r *Relay) loadGen(room string) uint64 {
	return r.passBegin(room)
}

// dropFenceIfIdleLocked forgets room's fence state once nothing is pending. Called when
// the room's relay state is torn down. Caller holds r.mu.
func (r *Relay) dropFenceIfIdleLocked(room string) {
	if f, ok := r.fences[room]; ok && r.pruneLocked(room, f).IsZero() {
		delete(r.fences, room)
	}
}

// writeFence advertises this instance's latest fence deadline for room, or
// removes its field when latest is zero. Caller holds the fence's io lock.
func (r *Relay) writeFence(ctx context.Context, room string, latest time.Time) {
	wctx, cancel := context.WithTimeout(context.WithoutCancel(ctx), loadFenceWriteTimeout)
	defer cancel()
	key := loadsKey(room)
	field := strconv.FormatUint(r.clientID, 10)
	var err error
	if latest.IsZero() {
		err = r.client.HDel(wctx, key, field).Err()
	} else {
		_, err = r.client.TxPipelined(wctx, func(p goredis.Pipeliner) error {
			p.HSet(wctx, key, field, latest.Unix())
			p.Expire(wctx, key, loadFenceMax)
			return nil
		})
	}
	if err != nil {
		r.log.Warn("relay could not advertise a load fence; other instances may trim what this load needs", "room", room, "err", err)
	}
}

// loadFenced reports whether any instance holds a live load fence on room.
func loadFenced(ctx context.Context, c goredis.Cmdable, room string, nowSecs int64) (bool, error) {
	fields, err := c.HGetAll(ctx, loadsKey(room)).Result()
	if err != nil {
		return false, err
	}
	for _, v := range fields {
		if d, err := strconv.ParseInt(v, 10, 64); err == nil && d > nowSecs {
			return true, nil
		}
	}
	return false, nil
}
