package coord

import (
	"context"
	"strings"
	"testing"
	"time"

	"github.com/alicebob/miniredis/v2"
	"github.com/reearth/ygo/cluster"

	"github.com/reearth/reearth-flow/websocket-go/internal/config"
	"github.com/reearth/reearth-flow/websocket-go/internal/gcs"
	redisrelay "github.com/reearth/reearth-flow/websocket-go/internal/redis"
)

func redisCfg(t *testing.T) *config.Config {
	t.Helper()
	mr := miniredis.RunT(t)
	return &config.Config{CoordBackend: config.BackendRedis, RedisURL: "redis://" + mr.Addr()}
}

// TestRedisLockerProbesTheStoreItLocks is the point of routing /health through the
// lock store rather than rebuilding a client: the probe and the locks can never
// end up pointed at different stores.
func TestRedisLockerProbesTheStoreItLocks(t *testing.T) {
	locks, err := NewLocker(redisCfg(t), "instance-test")
	if err != nil {
		t.Fatalf("NewLocker: %v", err)
	}
	defer func() { _ = locks.Close() }()

	if locks.Locker == nil {
		t.Fatal("Locker is nil")
	}
	if locks.Probe == nil {
		t.Fatal("Probe is nil; /health would report the backend unconfigured")
	}
	if err := locks.Probe(context.Background()); err != nil {
		t.Errorf("Probe() = %v, want nil against a live miniredis", err)
	}
}

// TestMemoryBackendHasNoStore: the memory backend's whole purpose is that there is
// nothing external to lock against or probe, so a non-nil probe here would mean it
// had quietly acquired a dependency.
func TestMemoryBackendHasNoStore(t *testing.T) {
	locks, err := NewLocker(&config.Config{CoordBackend: config.BackendMemory}, "instance-test")
	if err != nil {
		t.Fatalf("NewLocker: %v", err)
	}
	defer func() { _ = locks.Close() }()

	if locks.Locker == nil {
		t.Fatal("Locker is nil; the GCS adapter requires one")
	}
	// Probe must be non-nil even though there is no store: a nil one reports
	// "unconfigured" on /health, which the handler treats as 503. See
	// TestMemoryBackendIsHealthy.
	if locks.Probe == nil {
		t.Error("Probe is nil; the memory backend would serve 503 forever")
	}

	// A no-op locker must still run the critical section, or every flush and OID
	// allocation would silently stop happening.
	var ran bool
	if err := locks.Locker.WithLock(context.Background(), "k", func(context.Context) error {
		ran = true
		return nil
	}); err != nil {
		t.Errorf("WithLock = %v", err)
	}
	if !ran {
		t.Error("WithLock did not run fn")
	}
	ran = false
	if err := locks.Locker.TryWithLock(context.Background(), "k", time.Second, func(context.Context) error {
		ran = true
		return nil
	}); err != nil {
		t.Errorf("TryWithLock = %v", err)
	}
	if !ran {
		t.Error("TryWithLock did not run fn")
	}
}

func TestRelayForBackend(t *testing.T) {
	t.Run("redis", func(t *testing.T) {
		r, err := NewRelay(redisCfg(t), nil, nil, nil)
		if err != nil {
			t.Fatalf("NewRelay: %v", err)
		}
		defer func() { _ = r.Close() }()
		if _, ok := r.(*redisrelay.Relay); !ok {
			t.Errorf("redis backend returned %T, want *redis.Relay", r)
		}
	})

	t.Run("memory", func(t *testing.T) {
		r, err := NewRelay(&config.Config{CoordBackend: config.BackendMemory}, nil, nil, nil)
		if err != nil {
			t.Fatalf("NewRelay: %v", err)
		}
		defer func() { _ = r.Close() }()
		if _, ok := r.(*cluster.MemRelay); !ok {
			t.Errorf("memory backend returned %T, want *cluster.MemRelay", r)
		}
	})
}

// TestUnknownBackendIsRefused: Validate() already rejects an unknown name, so
// reaching the factory with one means a new backend was added to the enum without
// being wired. That must fail loudly rather than silently serving with no relay.
func TestUnknownBackendIsRefused(t *testing.T) {
	cfg := &config.Config{CoordBackend: "memcached"}
	if _, err := NewLocker(cfg, "instance-test"); err == nil {
		t.Error("NewLocker returned nil error for an unknown backend")
	}
	if _, err := NewRelay(cfg, nil, nil, nil); err == nil {
		t.Error("NewRelay returned nil error for an unknown backend")
	}
}

// TestDescribeNamesTheFanout: picking the wrong arm is the easiest way to waste a
// benchmark run, so the postgres description must distinguish poll from notify.
func TestDescribeNamesTheFanout(t *testing.T) {
	if got := Describe(&config.Config{CoordBackend: config.BackendRedis}); got != "redis" {
		t.Errorf("Describe(redis) = %q", got)
	}
	got := Describe(&config.Config{
		CoordBackend:   config.BackendPostgres,
		PGFanout:       config.FanoutPoll,
		PGPollInterval: 50 * time.Millisecond,
	})
	for _, want := range []string{"postgres", "poll", "50ms"} {
		if !strings.Contains(got, want) {
			t.Errorf("Describe(postgres) = %q, want it to mention %q", got, want)
		}
	}
}

// Compile-time proof that the no-op and Redis lockers both satisfy the interface
// the GCS adapter and flusher consume.
var _ gcs.Locker = gcs.NewNoLock()

// TestDescribeOmitsPollForNotify: notify holds no ticker, so reporting a poll
// interval for it is wrong in the one place an operator looks to confirm which arm
// is live. It also invites tuning a value the running config ignores.
func TestDescribeOmitsPollForNotify(t *testing.T) {
	for _, tc := range []struct {
		fanout   string
		wantPoll bool
	}{
		{config.FanoutNotify, false},
		{config.FanoutPoll, true},
		{config.FanoutHybrid, true},
	} {
		t.Run(tc.fanout, func(t *testing.T) {
			got := Describe(&config.Config{
				CoordBackend:   config.BackendPostgres,
				PGFanout:       tc.fanout,
				PGPollInterval: 50 * time.Millisecond,
			})
			if strings.Contains(got, "poll=") != tc.wantPoll {
				t.Errorf("Describe(%s) = %q, poll interval present = %v, want %v",
					tc.fanout, got, !tc.wantPoll, tc.wantPoll)
			}
			if !strings.Contains(got, tc.fanout) {
				t.Errorf("Describe(%s) = %q, want the fanout named", tc.fanout, got)
			}
		})
	}
}

// TestMemoryBackendIsHealthy guards a bug that made the memory backend
// undeployable: it supplied no probe, /health reported the coordination component
// "unconfigured", and the handler treats anything but "ok" as 503. The service
// returned 503 forever with GCS perfectly healthy, so Cloud Run would never have
// marked it ready — and the startup log carried a "probe unavailable" warning for a
// backend that has nothing to probe by design.
func TestMemoryBackendIsHealthy(t *testing.T) {
	cfg := &config.Config{CoordBackend: config.BackendMemory}
	locks, err := NewLocker(cfg, "instance-test")
	if err != nil {
		t.Fatalf("NewLocker: %v", err)
	}
	defer func() { _ = locks.Close() }()

	if locks.Probe == nil {
		t.Fatal("memory backend supplied a nil probe; /health reports it unconfigured and the service serves 503")
	}
	if err := locks.Probe(context.Background()); err != nil {
		t.Errorf("memory probe = %v, want healthy", err)
	}
}
