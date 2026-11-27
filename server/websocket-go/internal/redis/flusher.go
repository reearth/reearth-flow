package redis

import "context"

// Flusher persists a room's converged document to GCS when this node is the last
// active instance, before the relay deletes the Redis stream. The default is a
// no-op so the relay is usable stand-alone and in tests.
type Flusher interface {
	// FlushRoom persists the room's converged document to GCS.
	//
	// CONTRACT: FlushRoom MUST hold read:lock:{room} for its entire critical
	// section (it fences the stream DEL the relay runs immediately after) and MUST
	// NOT delete the stream itself. On error the relay skips the delete so the
	// un-persisted stream survives for a reconnect to replay.
	FlushRoom(ctx context.Context, room string) error
}

// LastInstanceFlusher is a Flusher with more to do when the last instance lets
// a room go than when the relay only needs a durable checkpoint of a live room:
// the relay calls FlushLastInstance instead of FlushRoom on that path. The same
// CONTRACT applies.
type LastInstanceFlusher interface {
	Flusher
	FlushLastInstance(ctx context.Context, room string) error
}

// flushLastInstance persists a room the last instance is letting go.
func flushLastInstance(ctx context.Context, f Flusher, room string) error {
	if lf, ok := f.(LastInstanceFlusher); ok {
		return lf.FlushLastInstance(ctx, room)
	}
	return f.FlushRoom(ctx, room)
}

// LiveCheckpointer is a Flusher that can say whether its checkpoint wrote the
// room's resident document or rebuilt it from storage. A trim needs the former:
// only a resident document is known to hold every entry the reader applied.
type LiveCheckpointer interface {
	CheckpointLive(ctx context.Context, room string) (live bool, err error)
}

// checkpointLive checkpoints room and reports whether the checkpoint wrote its
// resident document. A Flusher that cannot tell is taken at its word.
func checkpointLive(ctx context.Context, f Flusher, room string) (bool, error) {
	if lc, ok := f.(LiveCheckpointer); ok {
		return lc.CheckpointLive(ctx, room)
	}
	return true, f.FlushRoom(ctx, room)
}

// noopFlusher is the default when no real Flusher is injected.
type noopFlusher struct{}

func (noopFlusher) FlushRoom(context.Context, string) error { return nil }
