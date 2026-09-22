// Package coord builds the cluster coordination backend selected by
// REEARTH_FLOW_COORD_BACKEND. It is the ONE place in the service that branches on
// the backend: everything downstream consumes the cluster.Relay and gcs.Locker
// interfaces, so no other package needs to know which store is in use.
//
// Construction is split in two because the dependency order forces it:
//
//	NewLocker  →  gcs.New  →  server.NewWithPersistence  →  gcs.NewFlusher  →  NewRelay
package coord

import (
	"context"
	"fmt"
	"log/slog"

	goredis "github.com/redis/go-redis/v9"
	"github.com/reearth/ygo/cluster"

	"github.com/reearth/reearth-flow/websocket-go/internal/config"
	"github.com/reearth/reearth-flow/websocket-go/internal/gcs"
	"github.com/reearth/reearth-flow/websocket-go/internal/health"
)

// Flusher is the relay's last-instance persistence seam. It mirrors
// redis.Flusher structurally, so *gcs.Flusher satisfies both and neither backend
// package needs to import the other.
type Flusher interface {
	FlushRoom(ctx context.Context, room string) error
}

// Probe reports backend liveness for /health. Nil means the backend has nothing
// to probe (the memory backend), which /health reports as unconfigured rather
// than unhealthy.
type Probe func(ctx context.Context) error

// Locks holds the lock store for the configured backend, plus its health probe
// and a cleanup that releases the underlying client.
type Locks struct {
	Locker gcs.Locker
	Probe  Probe
	Close  func() error

	// handle is the backend's shared client, carried from NewLocker to NewRelay so
	// both reach the same store without a package-level variable or a
	// backend-specific parameter on the agnostic signature. Opaque: only the
	// backend that set it interprets it.
	handle any
}

// NewLocker builds the lock store for cfg.CoordBackend. Call it before the GCS
// adapter, which takes the Locker.
//
// cfg.Validate() has already rejected an unknown backend and a postgres backend
// with no DSN, so the default arm here is unreachable in a running service and
// exists only to fail loudly if a new backend is added without wiring it.
func NewLocker(cfg *config.Config, owner string) (*Locks, error) {
	switch cfg.CoordBackend {
	case config.BackendRedis:
		opt, err := goredis.ParseURL(cfg.RedisURL)
		if err != nil {
			return nil, fmt.Errorf("parse redis url: %w", err)
		}
		client := goredis.NewClient(opt)
		pinger, err := health.NewRedisPinger(cfg.RedisURL)
		if err != nil {
			// The URL already parsed above, so this cannot be a bad-DSN error.
			// Leave the probe nil rather than failing startup: /health reports the
			// component unconfigured, which is strictly better than no service.
			return &Locks{
				Locker: gcs.NewRedisLocker(client, owner),
				Close:  client.Close,
			}, nil
		}
		return &Locks{
			Locker: gcs.NewRedisLocker(client, owner),
			Probe:  pinger.Ping,
			Close: func() error {
				_ = pinger.Close()
				return client.Close()
			},
		}, nil

	case config.BackendMemory:
		// Single-instance operation: no external store, so no store to be down.
		//
		// The probe is a real always-healthy one rather than nil ON PURPOSE. A nil
		// probe reports "unconfigured", which the handler treats as unhealthy — so
		// leaving it nil made this backend serve 503 forever and never pass a Cloud
		// Run health check. Keeping nil to mean "miswired" is what makes that
		// distinction useful.
		return &Locks{
			Locker: gcs.NewNoLock(),
			Probe:  func(context.Context) error { return nil },
			Close:  func() error { return nil },
		}, nil

	case config.BackendPostgres:
		return newPostgresLocks(cfg, owner)

	default:
		return nil, fmt.Errorf("coord: unhandled backend %q", cfg.CoordBackend)
	}
}

// NewRelay builds the cluster relay for cfg.CoordBackend. Call it after NewLocker
// (whose store it may share) and after the Flusher, which it needs for
// last-instance persistence.
//
// The returned relay is always non-nil on a nil error; callers attach it with
// server.AttachRelay and must Close it on shutdown.
func NewRelay(cfg *config.Config, locks *Locks, flusher Flusher, log *slog.Logger) (cluster.Relay, error) {
	switch cfg.CoordBackend {
	case config.BackendRedis:
		return newRedisRelay(cfg, flusher, log)

	case config.BackendMemory:
		// MemRelay fans out within this process only. Two instances on this
		// backend silently serve diverging copies of the same document, which is
		// why config.Validate documents it as single-instance-only.
		return cluster.NewMemRelay(), nil

	case config.BackendPostgres:
		return newPostgresRelay(cfg, locks, flusher, log)

	default:
		return nil, fmt.Errorf("coord: unhandled backend %q", cfg.CoordBackend)
	}
}

// Describe returns the operator-facing summary of the active backend, for the
// startup log and /health. Benchmarking the wrong arm is the easiest way to waste
// a run, so the active backend must always be visible without reading env vars.
func Describe(cfg *config.Config) string {
	if cfg.CoordBackend != config.BackendPostgres {
		return cfg.CoordBackend
	}
	// Report the poll interval only for fan-outs that actually tick. Notify does
	// not, and printing an interval it ignores invites someone to "tune" a number
	// with no effect — or to read a notify arm's latency as a poll result.
	if cfg.PGFanout == config.FanoutNotify {
		return fmt.Sprintf("%s (fanout=%s)", cfg.CoordBackend, cfg.PGFanout)
	}
	return fmt.Sprintf("%s (fanout=%s, poll=%s)", cfg.CoordBackend, cfg.PGFanout, cfg.PGPollInterval)
}
