package config

import (
	"strings"
	"testing"
	"time"
)

// TestCoordDefaultsToRedis pins the default backend. Defaulting anywhere else
// would move cross-instance document traffic to a store the environment has not
// provisioned, and — while the Rust server still runs — silently fork documents
// between the two implementations.
func TestCoordDefaultsToRedis(t *testing.T) {
	clearEnv(t)
	c := Load()
	if c.CoordBackend != BackendRedis {
		t.Errorf("CoordBackend = %q, want %q", c.CoordBackend, BackendRedis)
	}
	if err := c.Validate(); err != nil {
		t.Errorf("Validate() on defaults = %v, want nil", err)
	}
}

// TestCoordBackendNormalized: an enum-valued setting must not be case- or
// whitespace-sensitive, or "Postgres" from a Terraform variable fails startup
// for no good reason.
func TestCoordBackendNormalized(t *testing.T) {
	clearEnv(t)
	t.Setenv("REEARTH_FLOW_COORD_BACKEND", "  Postgres ")
	t.Setenv("REEARTH_FLOW_PG_URL", "postgres://localhost/ws")
	c := Load()
	if c.CoordBackend != BackendPostgres {
		t.Errorf("CoordBackend = %q, want %q", c.CoordBackend, BackendPostgres)
	}
	if err := c.Validate(); err != nil {
		t.Errorf("Validate() = %v, want nil", err)
	}
}

func TestCoordValidateRejects(t *testing.T) {
	tests := []struct {
		name string
		env  map[string]string
		want string
	}{
		{
			name: "unknown backend",
			env:  map[string]string{"REEARTH_FLOW_COORD_BACKEND": "memcached"},
			want: "REEARTH_FLOW_COORD_BACKEND",
		},
		{
			name: "postgres without a dsn",
			env:  map[string]string{"REEARTH_FLOW_COORD_BACKEND": "postgres"},
			want: "REEARTH_FLOW_PG_URL",
		},
		{
			name: "postgres with a blank dsn",
			env: map[string]string{
				"REEARTH_FLOW_COORD_BACKEND": "postgres",
				"REEARTH_FLOW_PG_URL":        "   ",
			},
			want: "REEARTH_FLOW_PG_URL",
		},
		{
			name: "unknown fanout",
			env: map[string]string{
				"REEARTH_FLOW_COORD_BACKEND": "postgres",
				"REEARTH_FLOW_PG_URL":        "postgres://localhost/ws",
				"REEARTH_FLOW_PG_FANOUT":     "listen",
			},
			want: "REEARTH_FLOW_PG_FANOUT",
		},
		{
			name: "unparseable poll interval",
			env: map[string]string{
				"REEARTH_FLOW_COORD_BACKEND":    "postgres",
				"REEARTH_FLOW_PG_URL":           "postgres://localhost/ws",
				"REEARTH_FLOW_PG_FANOUT":        "poll",
				"REEARTH_FLOW_PG_POLL_INTERVAL": "50",
			},
			want: "REEARTH_FLOW_PG_POLL_INTERVAL",
		},
		{
			name: "poll interval below the floor",
			env: map[string]string{
				"REEARTH_FLOW_COORD_BACKEND":    "postgres",
				"REEARTH_FLOW_PG_URL":           "postgres://localhost/ws",
				"REEARTH_FLOW_PG_FANOUT":        "poll",
				"REEARTH_FLOW_PG_POLL_INTERVAL": "1ms",
			},
			want: "REEARTH_FLOW_PG_POLL_INTERVAL",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			clearEnv(t)
			for k, v := range tt.env {
				t.Setenv(k, v)
			}
			err := Load().Validate()
			if err == nil {
				t.Fatalf("Validate() = nil, want an error naming %s", tt.want)
			}
			if !strings.Contains(err.Error(), tt.want) {
				t.Errorf("Validate() = %v, want an error naming %s", err, tt.want)
			}
		})
	}
}

// TestHybridPollsSlowly: under hybrid, LISTEN carries the latency and the poll is
// only a backstop for notifications lost with a dropped connection. Defaulting it
// to the fast poll period would spend queries for nothing.
func TestHybridPollsSlowly(t *testing.T) {
	clearEnv(t)
	t.Setenv("REEARTH_FLOW_PG_FANOUT", "hybrid")
	if got := Load().PGPollInterval; got != time.Second {
		t.Errorf("hybrid PGPollInterval = %s, want %s", got, time.Second)
	}

	clearEnv(t)
	t.Setenv("REEARTH_FLOW_PG_FANOUT", "poll")
	if got := Load().PGPollInterval; got != 50*time.Millisecond {
		t.Errorf("poll PGPollInterval = %s, want 50ms", got)
	}
}

// TestNotifyIgnoresPollInterval: notify never reads the interval, so a stale or
// malformed value left in the environment must not block startup.
func TestNotifyIgnoresPollInterval(t *testing.T) {
	clearEnv(t)
	t.Setenv("REEARTH_FLOW_COORD_BACKEND", "postgres")
	t.Setenv("REEARTH_FLOW_PG_URL", "postgres://localhost/ws")
	t.Setenv("REEARTH_FLOW_PG_FANOUT", "notify")
	t.Setenv("REEARTH_FLOW_PG_POLL_INTERVAL", "not-a-duration")
	if err := Load().Validate(); err != nil {
		t.Errorf("Validate() = %v, want nil (notify does not poll)", err)
	}
}

// TestMemoryNeedsNoStore: the memory backend exists to remove the external store
// entirely, so it must validate with neither a Redis URL nor a DSN.
func TestMemoryNeedsNoStore(t *testing.T) {
	clearEnv(t)
	t.Setenv("REEARTH_FLOW_COORD_BACKEND", "memory")
	if err := Load().Validate(); err != nil {
		t.Errorf("Validate() = %v, want nil", err)
	}
}
