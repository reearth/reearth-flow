package coord

import (
	"context"
	"os"
	"strings"
	"testing"
	"time"

	"github.com/reearth/reearth-flow/websocket-go/internal/config"
)

const envDSN = "REEARTH_FLOW_WS_TEST_PG"

func pgCfg(t *testing.T, fanout string) *config.Config {
	t.Helper()
	dsn := os.Getenv(envDSN)
	if dsn == "" {
		t.Skipf("%s not set; skipping Postgres-backed tests", envDSN)
	}
	return &config.Config{
		CoordBackend:   config.BackendPostgres,
		PGURL:          dsn,
		PGFanout:       fanout,
		PGPollInterval: 20 * time.Millisecond,
	}
}

// TestPostgresBackendWiresEndToEnd walks the real construction order main.go uses,
// which is the only place the two factories' coupling through Locks.handle is
// exercised.
func TestPostgresBackendWiresEndToEnd(t *testing.T) {
	cfg := pgCfg(t, config.FanoutPoll)

	locks, err := NewLocker(cfg, "instance-test")
	if err != nil {
		t.Fatalf("NewLocker: %v", err)
	}
	defer func() { _ = locks.Close() }()

	if locks.Locker == nil {
		t.Fatal("Locker is nil")
	}
	if locks.Probe == nil {
		t.Fatal("Probe is nil")
	}
	if err := locks.Probe(context.Background()); err != nil {
		t.Errorf("Probe = %v, want nil", err)
	}

	relay, err := NewRelay(cfg, locks, nil, nil)
	if err != nil {
		t.Fatalf("NewRelay: %v", err)
	}
	defer func() { _ = relay.Close() }()
}

// TestPostgresRelayNeedsTheLockStorePool: NewRelay depends on NewLocker having run,
// and that ordering is enforced by a parameter rather than documentation. Passing
// the wrong Locks must be a clear error, not a nil-pointer panic at the first
// update.
func TestPostgresRelayNeedsTheLockStorePool(t *testing.T) {
	cfg := pgCfg(t, config.FanoutPoll)

	if _, err := NewRelay(cfg, nil, nil, nil); err == nil {
		t.Error("NewRelay with nil Locks = nil error, want a wiring error")
	}

	// Locks from a different backend carry no pool.
	memLocks, err := NewLocker(&config.Config{CoordBackend: config.BackendMemory}, "instance-test")
	if err != nil {
		t.Fatalf("NewLocker(memory): %v", err)
	}
	defer func() { _ = memLocks.Close() }()

	if _, err := NewRelay(cfg, memLocks, nil, nil); err == nil {
		t.Error("NewRelay with memory Locks = nil error, want a wiring error")
	}
}

// TestEveryFanoutConstructs walks all three benchmark arms through the factory.
// The mapping is the part worth pinning: a fan-out that silently landed on the
// wrong combination of ticker and listener would still serve traffic, and the
// benchmark would attribute its numbers to the wrong mechanism.
func TestEveryFanoutConstructs(t *testing.T) {
	for _, fanout := range []string{config.FanoutPoll, config.FanoutNotify, config.FanoutHybrid} {
		t.Run(fanout, func(t *testing.T) {
			cfg := pgCfg(t, fanout)
			locks, err := NewLocker(cfg, "instance-test")
			if err != nil {
				t.Fatalf("NewLocker: %v", err)
			}
			defer func() { _ = locks.Close() }()

			relay, err := NewRelay(cfg, locks, nil, nil)
			if err != nil {
				t.Fatalf("NewRelay(%s): %v", fanout, err)
			}
			defer func() { _ = relay.Close() }()
		})
	}
}

// TestUnknownFanoutIsRefused: Validate() rejects an unknown name first, so reaching
// the factory with one means a fan-out was added to the enum without being mapped.
func TestUnknownFanoutIsRefused(t *testing.T) {
	cfg := pgCfg(t, "listen-only")
	locks, err := NewLocker(cfg, "instance-test")
	if err != nil {
		t.Fatalf("NewLocker: %v", err)
	}
	defer func() { _ = locks.Close() }()

	_, err = NewRelay(cfg, locks, nil, nil)
	if err == nil {
		t.Fatal("NewRelay with an unmapped fanout = nil error, want a refusal")
	}
	if !strings.Contains(err.Error(), "fanout") {
		t.Errorf("NewRelay = %v, want an error naming the fanout", err)
	}
}

// TestStartJanitorIsNoOpForOtherBackends: it is called unconditionally from main,
// so it must be inert for backends whose store expires its own keys.
func TestStartJanitorIsNoOpForOtherBackends(t *testing.T) {
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	// No panic and no goroutine for a backend with nothing to reap.
	StartJanitor(ctx, &config.Config{CoordBackend: config.BackendMemory}, nil, nil)
	StartJanitor(ctx, &config.Config{CoordBackend: config.BackendRedis}, nil, nil)
}
