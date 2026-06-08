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

// noopFlusher is the default when no real Flusher is injected.
type noopFlusher struct{}

func (noopFlusher) FlushRoom(context.Context, string) error { return nil }
