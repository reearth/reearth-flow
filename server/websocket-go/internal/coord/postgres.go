package coord

import (
	"context"
	"fmt"
	"log/slog"
	"time"

	"github.com/jackc/pgx/v5/pgxpool"
	"github.com/reearth/ygo/cluster"

	"github.com/reearth/reearth-flow/websocket-go/internal/config"
	"github.com/reearth/reearth-flow/websocket-go/internal/pg"
)

// schemaTimeout bounds the startup migration. The DDL is a handful of IF NOT EXISTS
// statements behind an advisory lock, so exceeding this means the database is
// unreachable or the lock is held by a wedged peer — both worth failing startup for.
const schemaTimeout = 30 * time.Second

// newPostgresLocks opens the pool, applies the coordination schema, and returns the
// lock store plus its health probe.
//
// The pool is stashed on Locks.handle so NewRelay and the janitor reach the SAME
// database and share its connections: opening a pool per component would multiply
// the connection count against a Cloud SQL max_connections for no benefit.
func newPostgresLocks(cfg *config.Config, owner string) (*Locks, error) {
	pcfg, err := pgxpool.ParseConfig(cfg.PGURL)
	if err != nil {
		return nil, fmt.Errorf("parse postgres url: %w", err)
	}
	pool, err := pgxpool.NewWithConfig(context.Background(), pcfg)
	if err != nil {
		return nil, fmt.Errorf("open postgres pool: %w", err)
	}

	// Fail startup rather than serving with no schema: every relay query would
	// error, which surfaces to users as a document that silently stops syncing.
	ctx, cancel := context.WithTimeout(context.Background(), schemaTimeout)
	defer cancel()
	if err := pg.Migrate(ctx, pool); err != nil {
		pool.Close()
		return nil, err
	}

	return &Locks{
		Locker: pg.NewLocker(pool, owner),
		Probe:  pg.NewPinger(pool).Ping,
		Close:  func() error { pool.Close(); return nil },
		handle: pool,
	}, nil
}

// poolFrom recovers the pool newPostgresLocks opened. A nil result means the caller
// passed Locks built for a different backend, which is a wiring bug rather than a
// runtime condition.
func poolFrom(locks *Locks) *pgxpool.Pool {
	if locks == nil {
		return nil
	}
	pool, _ := locks.handle.(*pgxpool.Pool)
	return pool
}

// newPostgresRelay builds the Postgres relay over the pool the lock store opened.
func newPostgresRelay(cfg *config.Config, locks *Locks, flusher Flusher, log *slog.Logger) (cluster.Relay, error) {
	pool := poolFrom(locks)
	if pool == nil {
		return nil, fmt.Errorf("coord: postgres relay needs the pool from NewLocker; got none")
	}

	// The fan-out is expressed to the relay as two independent switches: whether to
	// tick, and whether to hold a LISTEN connection. Poll-only passes no notify,
	// notify-only passes no interval (a zero interval disables the ticker outright),
	// and hybrid passes both.
	var (
		poll   time.Duration
		notify bool
	)
	switch cfg.PGFanout {
	case config.FanoutPoll:
		poll = cfg.PGPollInterval
	case config.FanoutNotify:
		notify = true
	case config.FanoutHybrid:
		poll, notify = cfg.PGPollInterval, true
	default:
		return nil, fmt.Errorf("coord: unhandled fanout %q", cfg.PGFanout)
	}

	r, err := pg.New(pg.Options{
		Q:         pool,
		Logger:    log,
		Flusher:   flusher,
		PollEvery: poll,
		Notify:    notify,
	})
	if err != nil {
		return nil, fmt.Errorf("build postgres relay: %w", err)
	}
	return r, nil
}

// StartJanitor runs the retention sweep that replaces the Redis key TTLs, and is a
// no-op for every backend that expires its own keys. Returns immediately; the sweep
// runs until ctx is cancelled.
func StartJanitor(ctx context.Context, cfg *config.Config, locks *Locks, log *slog.Logger) {
	if cfg.CoordBackend != config.BackendPostgres {
		return
	}
	pool := poolFrom(locks)
	if pool == nil {
		return
	}
	go pg.NewJanitor(pool, log).Run(ctx)
}
