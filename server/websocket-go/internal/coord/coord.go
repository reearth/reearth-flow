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

// Flusher is the last-instance persistence seam.
type Flusher interface {
	FlushRoom(ctx context.Context, room string) error
}

// Probe reports backend liveness for /health.
type Probe func(ctx context.Context) error

// Locks is the configured backend's lock store, health probe and cleanup.
type Locks struct {
	Locker gcs.Locker
	Probe  Probe
	Close  func() error

	// handle carries the backend's client from NewLocker to NewRelay so both reach the same store.
	handle any
}

// NewLocker builds the lock store for cfg.CoordBackend. Call before gcs.New.
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
		// No external store, so nothing to be down. The probe must be always-healthy
		// rather than nil: nil reads as unhealthy and served 503 forever.
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

// NewRelay builds the relay for cfg.CoordBackend, after NewLocker (whose store it
// may share) and the Flusher. Callers attach it and must Close it on shutdown.
func NewRelay(cfg *config.Config, locks *Locks, flusher Flusher, log *slog.Logger) (cluster.Relay, error) {
	switch cfg.CoordBackend {
	case config.BackendRedis:
		return newRedisRelay(cfg, flusher, log)

	case config.BackendMemory:
		return cluster.NewMemRelay(), nil

	case config.BackendPostgres:
		return newPostgresRelay(cfg, locks, flusher, log)

	default:
		return nil, fmt.Errorf("coord: unhandled backend %q", cfg.CoordBackend)
	}
}

// Describe summarises the active backend for the startup log and /health
func Describe(cfg *config.Config) string {
	if cfg.CoordBackend != config.BackendPostgres {
		return cfg.CoordBackend
	}

	if cfg.PGFanout == config.FanoutNotify {
		return fmt.Sprintf("%s (fanout=%s)", cfg.CoordBackend, cfg.PGFanout)
	}
	return fmt.Sprintf("%s (fanout=%s, poll=%s)", cfg.CoordBackend, cfg.PGFanout, cfg.PGPollInterval)
}
