package relaypolicy

import (
	"context"
	"errors"
	"fmt"
	"math"
	"strconv"
	"testing"

	"github.com/reearth/ygo/awareness"
	"github.com/reearth/ygo/cluster"
	"github.com/reearth/ygo/crdt"
	ygows "github.com/reearth/ygo/provider/websocket"
)

// Payloads the server can take: an empty V1 update and an empty awareness update.
var (
	validSync      = []byte{0, 0}
	validAwareness = []byte{0}
	malformed      = []byte{1}
)

func syncIn(data []byte) cluster.Inbound {
	return cluster.Inbound{Room: "r", Kind: cluster.KindSync, Data: data}
}

func awareIn(data []byte) cluster.Inbound {
	return cluster.Inbound{Room: "r", Kind: cluster.KindAwareness, Data: data}
}

// TestClassifyInject: an entry whose payload can never apply is skipped so it
// cannot wedge a room, a load or persistence failure is retried even when it
// carries a decoder error, and shutdown stops the reader.
func TestClassifyInject(t *testing.T) {
	t.Parallel()

	t.Run("payloads the empty state takes", func(t *testing.T) {
		t.Parallel()
		if payloadInvalid(syncIn(validSync)) || payloadInvalid(awareIn(validAwareness)) {
			t.Fatal("setup: a valid payload was judged invalid")
		}
	})

	t.Run("applied", func(t *testing.T) {
		t.Parallel()
		if got := ClassifyInject(context.Background(), syncIn(validSync), nil); got != Applied {
			t.Errorf("got %d, want Applied", got)
		}
	})

	t.Run("malformed sync entry is skipped", func(t *testing.T) {
		t.Parallel()
		// What the server's Inject returns for it: the decoder's own error.
		_, _, err := crdt.ApplyUpdateV1Held(crdt.New(), malformed, nil)
		if err == nil {
			t.Fatal("setup: a malformed update decoded")
		}
		if got := ClassifyInject(context.Background(), syncIn(malformed), err); got != Skip {
			t.Errorf("got %d, want Skip", got)
		}
	})

	t.Run("malformed awareness entry is skipped", func(t *testing.T) {
		t.Parallel()
		// What the server's Inject returns for it: the awareness decoder's error,
		// which matches none of the server's sentinels.
		_, err := awareness.New(1).ApplyUpdateWith(malformed, nil, awareness.ApplyOptions{Foreign: true})
		if err == nil {
			t.Fatal("setup: a malformed awareness update decoded")
		}
		if got := ClassifyInject(context.Background(), awareIn(malformed), err); got != Skip {
			t.Errorf("got %d, want Skip", got)
		}
	})

	t.Run("stored document that fails to decode is retried", func(t *testing.T) {
		t.Parallel()
		// A room whose stored document fails to decode returns the decoder's
		// error too, wrapped, for a payload that is itself fine.
		_, _, decodeErr := crdt.ApplyUpdateV1Held(crdt.New(), malformed, nil)
		loadErr := fmt.Errorf("bootstrapping room %q: %w", "r", decodeErr)
		if got := ClassifyInject(context.Background(), syncIn(validSync), loadErr); got != Retry {
			t.Errorf("sync: got %d, want Retry", got)
		}
		if got := ClassifyInject(context.Background(), awareIn(validAwareness), loadErr); got != Retry {
			t.Errorf("awareness: got %d, want Retry", got)
		}
	})

	t.Run("load failure is retried", func(t *testing.T) {
		t.Parallel()
		err := errors.New("gcs: load failed")
		if got := ClassifyInject(context.Background(), syncIn(validSync), err); got != Retry {
			t.Errorf("sync: got %d, want Retry", got)
		}
		if got := ClassifyInject(context.Background(), awareIn(validAwareness), err); got != Retry {
			t.Errorf("awareness: got %d, want Retry", got)
		}
	})

	t.Run("server sentinels are skipped", func(t *testing.T) {
		t.Parallel()
		for _, sentinel := range []error{ygows.ErrInvalidUpdate, ygows.ErrUpdateTooLarge, ygows.ErrInvalidRoomName} {
			if got := ClassifyInject(context.Background(), syncIn(validSync), fmt.Errorf("inject: %w", sentinel)); got != Skip {
				t.Errorf("%v: got %d, want Skip", sentinel, got)
			}
		}
	})

	t.Run("shutdown stops", func(t *testing.T) {
		t.Parallel()
		err := fmt.Errorf("inject: %w", ygows.ErrServerShutdown)
		if got := ClassifyInject(context.Background(), syncIn(malformed), err); got != Stop {
			t.Errorf("got %d, want Stop", got)
		}
	})

	t.Run("cancelled context stops", func(t *testing.T) {
		t.Parallel()
		ctx, cancel := context.WithCancel(context.Background())
		cancel()
		if got := ClassifyInject(ctx, syncIn(malformed), errors.New("anything")); got != Stop {
			t.Errorf("got %d, want Stop", got)
		}
	})
}

// TestPayloadInvalid_LargeDependentUpdateIsValid: without the room's state an
// incremental update parks every item that depends on it. A bare document caps
// parked items at 100,000 and refuses a larger update the room can take, so the
// check must not judge such an update invalid.
func TestPayloadInvalid_LargeDependentUpdateIsValid(t *testing.T) {
	t.Parallel()

	base := crdt.New(crdt.WithClientID(1))
	root := base.GetMap("m") // GetMap takes the doc lock: never inside Transact
	base.Transact(func(txn *crdt.Transaction) { root.Set(txn, "inner", crdt.NewMapPrelim()) })
	baseSV, err := crdt.DecodeStateVectorV1(crdt.EncodeStateVectorV1(base))
	if err != nil {
		t.Fatalf("state vector: %v", err)
	}

	editor := crdt.New(crdt.WithClientID(2), crdt.WithMaxPendingItems(math.MaxInt))
	if err := crdt.ApplyUpdateV1(editor, crdt.EncodeStateAsUpdateV1(base, nil), nil); err != nil {
		t.Fatalf("sync editor: %v", err)
	}
	v, ok := editor.GetMap("m").Get("inner")
	inner, isMap := v.(*crdt.YMap)
	if !ok || !isMap {
		t.Fatalf("setup: inner map missing: %T", v)
	}
	// Every key is its own item, parented by a map only the base document holds.
	editor.Transact(func(txn *crdt.Transaction) {
		for i := 0; i < 100_001; i++ {
			inner.Set(txn, strconv.Itoa(i), i)
		}
	})
	diff := crdt.EncodeStateAsUpdateV1(editor, baseSV)

	if payloadInvalid(syncIn(diff)) {
		t.Error("a valid update that parks more than 100,000 items was judged invalid")
	}
}
