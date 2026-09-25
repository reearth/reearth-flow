package pg

import (
	"context"
	"log/slog"
	"time"

	"github.com/jackc/pgx/v5/pgxpool"
)

// Reconnect backoff: a blip must not end delivery, nor hot-loop a database that is
// down.
const (
	listenRetryMin = 200 * time.Millisecond
	listenRetryMax = 5 * time.Second
)

// listener holds one LISTEN connection and wakes the rooms resident here. One per
// PROCESS, not per room: the connection is checked out of the pool for good, so a
// per-room one would tie pool size to how many documents are open.
//
// CONSTRAINT: LISTEN does not survive a transaction-pooling proxy — behind pgbouncer
// in transaction mode notifications never arrive, silently.
type listener struct {
	pool *pgxpool.Pool
	log  *slog.Logger
	// wake takes a doc id that may have new rows. Must not block — it runs on the
	// single delivery goroutine.
	wake func(room string)
}

func newListener(pool *pgxpool.Pool, log *slog.Logger, wake func(room string)) *listener {
	if log == nil {
		log = slog.Default()
	}
	return &listener{pool: pool, log: log, wake: wake}
}

// run holds a LISTEN connection and dispatches notifications until ctx is
// cancelled, reconnecting with backoff on failure.
func (l *listener) run(ctx context.Context) {
	backoff := listenRetryMin
	for {
		if ctx.Err() != nil {
			return
		}
		if err := l.listenOnce(ctx); err != nil {
			if ctx.Err() != nil {
				return
			}
			l.log.Debug("listener connection lost", "err", err, "retry_in", backoff)
			select {
			case <-ctx.Done():
				return
			case <-time.After(backoff):
			}
			backoff = min(backoff*2, listenRetryMax)
			continue
		}
		// A clean return means ctx ended.
		return
	}
}

// listenOnce acquires a connection, issues LISTEN, and delivers until it fails or
// ctx ends. It wakes every resident room first: notifications published while
// disconnected are gone, and only a re-read recovers them.
func (l *listener) listenOnce(ctx context.Context) error {
	conn, err := l.pool.Acquire(ctx)
	if err != nil {
		return err
	}
	defer conn.Release()

	if _, err := conn.Exec(ctx, "LISTEN "+notifyChannel); err != nil {
		return err
	}
	l.log.Debug("listener attached", "channel", notifyChannel)

	// Recover anything missed while disconnected.
	l.wake("")

	for {
		n, err := conn.Conn().WaitForNotification(ctx)
		if err != nil {
			if ctx.Err() != nil {
				return nil
			}
			return err
		}
		l.wake(n.Payload)
	}
}
