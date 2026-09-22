package pg

import (
	"context"
	"log/slog"
	"time"

	"github.com/jackc/pgx/v5/pgxpool"
)

// Reconnect backoff for the LISTEN connection. A dedicated connection is held for
// the process lifetime, so a transient database blip must not end delivery — but it
// must also not hot-loop against a database that is down.
const (
	listenRetryMin = 200 * time.Millisecond
	listenRetryMax = 5 * time.Second
)

// listener holds one dedicated connection running LISTEN and fans wakeups out to
// the rooms resident on this instance.
//
// One connection per PROCESS, not per room: LISTEN is connection-scoped but the
// channel is shared. The connection is checked out of the pool for good, so a
// per-room one would tie pool size to how many documents are open.
//
// CONSTRAINT: LISTEN does not survive a transaction-pooling proxy. Behind pgbouncer
// in transaction mode notifications never arrive, and the relay silently degrades to
// whatever polling is configured — which for the notify fan-out is none.
type listener struct {
	pool *pgxpool.Pool
	log  *slog.Logger
	// wake is called with a doc id when that document may have new rows. It must
	// not block: it is invoked from the single delivery goroutine.
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

// listenOnce acquires a connection, issues LISTEN, and delivers notifications until
// the connection fails or ctx ends.
//
// On reconnect, every resident room is woken unconditionally before waiting again:
// notifications published while this instance had no listener are gone, and only a
// re-read can recover them. Without this a blip would silently strand every room
// until its next local write.
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
