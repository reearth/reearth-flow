package pg

import (
	"context"
	"errors"
	"time"

	"github.com/jackc/pgx/v5"
)

// Locker is the Postgres lock store for the GCS adapter and flusher (satisfies
// gcs.Locker structurally, so neither package imports the other).
//
// Uses the ws_lock table, not pg_advisory_lock: both callers wrap non-transactional
// work — the flusher's critical section is a GCS round trip — so an advisory lock
// would hold a transaction open across that I/O, or be session-scoped and not
// survive a pooling proxy.
type Locker struct {
	q       Querier
	owner   string
	ttl     time.Duration
	retries int
	delay   time.Duration
}

// Lock acquisition bounds, mirroring the Redis locker so the connect path cannot
// deadlock: ten tries, 500ms apart, against a 10s lease.
const (
	lockTTL     = 10 * time.Second
	lockRetries = 10
	lockDelay   = 500 * time.Millisecond
)

// ErrLockTimeout is returned when the lock could not be taken within the bounded
// retry budget.
var ErrLockTimeout = errors.New("pg: lock acquisition timed out")

// NewLocker builds a bounded Postgres Locker. owner must be unique per process so
// only the holder can release.
func NewLocker(q Querier, owner string) *Locker {
	return &Locker{q: q, owner: owner, ttl: lockTTL, retries: lockRetries, delay: lockDelay}
}

// WithLock runs fn while holding key, retrying a bounded number of times and
// returning ErrLockTimeout rather than blocking forever.
func (l *Locker) WithLock(ctx context.Context, key string, fn func(context.Context) error) error {
	var acquired bool
	for i := 0; i < l.retries; i++ {
		ok, err := l.tryAcquire(ctx, key, l.ttl)
		if err != nil {
			return err
		}
		if ok {
			acquired = true
			break
		}
		select {
		case <-ctx.Done():
			return ctx.Err()
		case <-time.After(l.delay):
		}
	}
	if !acquired {
		return ErrLockTimeout
	}
	defer l.release(key)
	return fn(ctx)
}

// TryWithLock takes key in one attempt and runs fn either way: a fence, not mutual
// exclusion. A held lock or an unreachable database must not stop a flush — the relay
// re-checks active instances before deleting, and AppendUpdate is idempotent.
func (l *Locker) TryWithLock(ctx context.Context, key string, ttl time.Duration, fn func(context.Context) error) error {
	ok, err := l.tryAcquire(ctx, key, ttl)
	if err != nil || !ok {
		return fn(ctx)
	}
	defer l.release(key)
	return fn(ctx)
}

// tryAcquire takes key for ttl in one statement. ON CONFLICT ... WHERE expires_at
// <= now() means a live holder yields no row and an expired one is taken over.
func (l *Locker) tryAcquire(ctx context.Context, key string, ttl time.Duration) (bool, error) {
	var one int
	err := l.q.QueryRow(ctx, acquireLockSQL, key, l.owner, secs(ttl)).Scan(&one)
	if errors.Is(err, pgx.ErrNoRows) {
		return false, nil
	}
	if err != nil {
		return false, err
	}
	return true, nil
}

// release drops our own lock, on a fresh bounded context so a cancelled caller
// context does not leave the lock held until its TTL.
func (l *Locker) release(key string) {
	ctx, cancel := context.WithTimeout(context.WithoutCancel(context.Background()), 2*time.Second)
	defer cancel()
	_, _ = l.q.Exec(ctx, releaseLockSQL, key, l.owner)
}

// Pinger reports database liveness for /health.
type Pinger struct{ q Querier }

// NewPinger builds a /health probe over an existing pool.
func NewPinger(q Querier) *Pinger { return &Pinger{q: q} }

// Ping issues the cheapest possible round trip. It deliberately does NOT touch
// ws_stream: /health must report connectivity, and an unlogged table emptied by a
// crash is a healthy state, not a failure.
func (p *Pinger) Ping(ctx context.Context) error {
	var one int
	return p.q.QueryRow(ctx, `SELECT 1`).Scan(&one)
}
