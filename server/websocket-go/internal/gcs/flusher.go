package gcs

import (
	"context"
	"time"
)

// Flusher implements the redis.Flusher seam (FlushRoom): when this node is the
// last active instance for a room, FlushRoom persists the in-memory doc state to
// GCS before the stream is deleted.
//
// FlushRoom MUST hold read:lock:{room} for its critical section so the relay's
// subsequent stream-delete cannot race a reader catching up.
type Flusher struct {
	adapter *Adapter
	locker  Locker
	stateOf func(room string) []byte
	lockTTL time.Duration
}

// FlusherOptions configure a Flusher.
type FlusherOptions struct {
	Adapter *Adapter
	// Locker fences the read lock. Nil means no fencing — the flush still runs.
	// The Locker carries its own owner token, so the flusher does not need one.
	Locker Locker
	// StateOf returns the room's current in-memory doc state as a single V1
	// update (e.g. server.WSProvider().GetDoc(room) → EncodeStateAsUpdateV1), or
	// nil when no doc is resident.
	StateOf func(room string) []byte
}

const defaultReadLockTTL = 30 * time.Second

// NewFlusher builds a Flusher.
func NewFlusher(opts FlusherOptions) *Flusher {
	return &Flusher{
		adapter: opts.Adapter,
		locker:  opts.Locker,
		stateOf: opts.StateOf,
		lockTTL: defaultReadLockTTL,
	}
}

// readLockName is the per-doc read lock the flusher holds across persistence.
// Phase-invariant — never folder-prefixed.
func readLockName(room string) string { return "read:lock:" + room }

// FlushRoom persists the room's current in-memory state to GCS while holding
// read:lock:{room}. No-op when there is no resident state.
func (f *Flusher) FlushRoom(ctx context.Context, room string) error {
	return f.withReadLock(ctx, room, func(ctx context.Context) error {
		// Write the COMPLETE state as doc_v2 (not an incremental update): the Rust
		// reader loads doc_v2 alone and folds no tail updates, so a Go-flushed room
		// must leave a complete doc_v2 for cross-implementation cold-loads.
		if state := f.stateFor(room); len(state) > 0 {
			return f.adapter.FlushSnapshot(ctx, room, state)
		}
		// ygo removes the live doc from GetDoc before firing RoomDeactivated, so
		// StateOf is empty here. It first drained the per-update persistence worker,
		// so the full state is already in GCS: reconstruct it and write doc_v2.
		return f.adapter.SnapshotFromStore(ctx, room)
	})
}

// stateFor returns the room's current state, tolerating a nil provider.
func (f *Flusher) stateFor(room string) []byte {
	if f.stateOf == nil {
		return nil
	}
	return f.stateOf(room)
}

// withReadLock runs fn while holding read:lock:{room}. The lock is best-effort:
// if the lock store is unreachable or the key is already held, fn still proceeds
// (the relay re-checks active instances before deleting, and AppendUpdate is
// idempotent). TryWithLock never releases a lock it did not acquire.
func (f *Flusher) withReadLock(ctx context.Context, room string, fn func(context.Context) error) error {
	if f.locker == nil {
		return fn(ctx)
	}
	return f.locker.TryWithLock(ctx, readLockName(room), f.lockTTL, fn)
}
