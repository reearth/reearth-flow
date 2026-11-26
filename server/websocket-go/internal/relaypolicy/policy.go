// Package relaypolicy holds the delivery rules both relays share: how a failed
// write is retried, and what the reader does with an entry it could not inject.
package relaypolicy

import (
	"context"
	"errors"
	"time"

	ygows "github.com/reearth/ygo/provider/websocket"
)

// WriteRetryDelays spaces the retries of a failed batched write, about 2 s in
// all. A retry after a write that landed but lost its reply duplicates entries,
// which Yjs ignores.
var WriteRetryDelays = []time.Duration{100 * time.Millisecond, 400 * time.Millisecond, 1600 * time.Millisecond}

// InjectOutcome is what the reader does with an entry after injecting it.
type InjectOutcome int

const (
	// Applied: the room took it, or there was nothing to apply. Move past it.
	Applied InjectOutcome = iota
	// Skip: the room can never apply it. Move past it so it cannot wedge the room.
	Skip
	// Retry: a transient failure. Keep the cursor before it and read it again.
	Retry
	// Stop: the server is shutting down. Stop where the reader is.
	Stop
)

// ClassifyInject decides what an Inject error means for the reader.
func ClassifyInject(ctx context.Context, err error) InjectOutcome {
	switch {
	case err == nil:
		return Applied
	case ctx.Err() != nil, errors.Is(err, context.Canceled), errors.Is(err, ygows.ErrServerShutdown):
		return Stop
	case errors.Is(err, ygows.ErrInvalidUpdate), errors.Is(err, ygows.ErrUpdateTooLarge), errors.Is(err, ygows.ErrInvalidRoomName):
		return Skip
	default:
		return Retry
	}
}
