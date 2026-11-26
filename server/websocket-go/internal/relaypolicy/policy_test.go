package relaypolicy

import (
	"context"
	"errors"
	"fmt"
	"testing"

	ygows "github.com/reearth/ygo/provider/websocket"
)

// TestClassifyInject: a bad entry is skipped so it cannot wedge a room, a load
// or persistence failure is retried, and shutdown stops the reader. Errors are
// matched however deeply they are wrapped.
func TestClassifyInject(t *testing.T) {
	t.Parallel()

	cancelled, cancel := context.WithCancel(context.Background())
	cancel()
	wrap := func(err error) error { return fmt.Errorf("inject: %w", err) }

	cases := []struct {
		name string
		ctx  context.Context
		err  error
		want InjectOutcome
	}{
		{"applied", context.Background(), nil, Applied},
		{"invalid update", context.Background(), wrap(ygows.ErrInvalidUpdate), Skip},
		{"update too large", context.Background(), wrap(ygows.ErrUpdateTooLarge), Skip},
		{"invalid room name", context.Background(), wrap(ygows.ErrInvalidRoomName), Skip},
		{"load failure", context.Background(), errors.New("gcs: load failed"), Retry},
		{"server shutdown", context.Background(), wrap(ygows.ErrServerShutdown), Stop},
		{"cancelled context", cancelled, errors.New("anything"), Stop},
	}
	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			t.Parallel()
			if got := ClassifyInject(tc.ctx, tc.err); got != tc.want {
				t.Errorf("ClassifyInject(%v) = %d, want %d", tc.err, got, tc.want)
			}
		})
	}
}
