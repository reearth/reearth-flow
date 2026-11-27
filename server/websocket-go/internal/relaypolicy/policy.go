// Package relaypolicy holds the delivery rules both relays share: how a failed
// write is retried, and what the reader does with an entry it could not inject.
package relaypolicy

import (
	"context"
	"errors"
	"math"
	"time"

	"github.com/reearth/ygo/awareness"
	"github.com/reearth/ygo/cluster"
	"github.com/reearth/ygo/crdt"
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

// ClassifyInject decides what the error from injecting in means for the reader.
//
// Whether the entry itself is bad is decided from its payload, not from the
// error: the server returns the same decoder errors when the room's stored
// document fails to load, and that failure must be retried, not skipped past.
// So an entry is skipped when its payload fails on its own, and any other
// failure, a load or persistence one included, is retried.
func ClassifyInject(ctx context.Context, in cluster.Inbound, err error) InjectOutcome {
	switch {
	case err == nil:
		return Applied
	case ctx.Err() != nil, errors.Is(err, context.Canceled), errors.Is(err, ygows.ErrServerShutdown):
		return Stop
	case errors.Is(err, ygows.ErrInvalidUpdate), errors.Is(err, ygows.ErrUpdateTooLarge),
		errors.Is(err, ygows.ErrInvalidRoomName):
		return Skip
	case payloadInvalid(in):
		return Skip
	default:
		return Retry
	}
}

// payloadInvalid reports whether in's payload fails to apply even to an empty
// document or awareness state, so no room can ever take it. It runs only after
// an Inject has failed, so the happy path pays nothing for it.
func payloadInvalid(in cluster.Inbound) bool {
	switch in.Kind {
	case cluster.KindSync:
		// No pending cap, as ygo's VersionedPersistence.AppendUpdate asks of a
		// scratch check: without the room's state an incremental update parks
		// every dependent item, and a bare document refuses past 100,000 of them,
		// which would skip a large update the room can take.
		scratch := crdt.New(crdt.WithMaxPendingItems(math.MaxInt))
		_, _, err := crdt.ApplyUpdateV1Held(scratch, in.Data, nil)
		return err != nil
	case cluster.KindAwareness:
		_, err := awareness.New(0).ApplyUpdateWith(in.Data, nil, awareness.ApplyOptions{Foreign: true})
		return err != nil
	default:
		return false
	}
}
