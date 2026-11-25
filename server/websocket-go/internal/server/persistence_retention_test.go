package server

import (
	"context"
	"testing"

	"github.com/reearth/ygo/persistence"

	"github.com/reearth/reearth-flow/websocket-go/internal/config"
)

// TestKeepUpdatesOf_NeverReturnsZero: ygo reads KeepVersions 0 as "keep every
// update", so the unload compaction would never trim. A config built without
// the field, which every test literal is, must still get a real retention.
func TestKeepUpdatesOf_NeverReturnsZero(t *testing.T) {
	t.Parallel()

	t.Run("configured value wins", func(t *testing.T) {
		t.Parallel()

		if got := keepUpdatesOf(&config.Config{KeepUpdates: 42}); got != 42 {
			t.Errorf("keepUpdatesOf = %d, want 42", got)
		}
	})

	t.Run("unset falls back to the default", func(t *testing.T) {
		t.Parallel()

		if got := keepUpdatesOf(&config.Config{}); got != config.DefaultKeepUpdates {
			t.Errorf("keepUpdatesOf = %d, want %d", got, config.DefaultKeepUpdates)
		}
	})

	t.Run("nil config falls back to the default", func(t *testing.T) {
		t.Parallel()

		if got := keepUpdatesOf(nil); got != config.DefaultKeepUpdates {
			t.Errorf("keepUpdatesOf = %d, want %d", got, config.DefaultKeepUpdates)
		}
	})
}

// TestLegacyAdapterCarriesTheConfiguredRetention: ygo compacts a room on every
// unload with the adapter's KeepVersions, so the configured retention has to
// reach that field. It also must never arrive as 0, which ygo reads as "keep
// every update" and which is the bug #2759 fixed on the other side.
func TestLegacyAdapterCarriesTheConfiguredRetention(t *testing.T) {
	t.Parallel()

	t.Run("configured value reaches ygo", func(t *testing.T) {
		t.Parallel()

		la := legacyAdapter(context.Background(), &config.Config{KeepUpdates: 42}, persistence.NewMemoryPersistence())
		if la.KeepVersions != 42 {
			t.Errorf("KeepVersions = %d, want 42", la.KeepVersions)
		}
	})

	t.Run("a config without the field still trims", func(t *testing.T) {
		t.Parallel()

		la := legacyAdapter(context.Background(), &config.Config{}, persistence.NewMemoryPersistence())
		if la.KeepVersions != config.DefaultKeepUpdates {
			t.Errorf("KeepVersions = %d, want %d", la.KeepVersions, config.DefaultKeepUpdates)
		}
	})
}
