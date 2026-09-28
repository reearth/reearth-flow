package pg

import (
	"context"
	"log/slog"
	"time"
)

// Retention windows. These replace the Redis key TTLs the backend no longer has:
// the 6h stream EXPIRE, and the 120s heartbeat EXPIRE.
const (
	// streamRetention matches the Redis stream's 6h TTL. Rows older than this
	// cannot be needed: any document still being edited has been flushed to GCS
	// and reloaded long before.
	streamRetention = 6 * time.Hour
	// instanceRetention matches the Redis heartbeat TTL (120s). It is deliberately
	// longer than activeTimeout (60s): liveness is decided by seen_at, so reaping
	// early would only lose rows the election already treats as dead, and reaping
	// late changes no decision.
	instanceRetention = 2 * time.Minute
	// lockGrace keeps expired locks around briefly before deleting them, so a
	// diagnostic query can still see who last held one.
	lockGrace = 5 * time.Minute
	// janitorEvery is the sweep period. Nothing depends on its punctuality — every
	// correctness predicate is a timestamp comparison, never row presence — so this
	// is chosen to keep the table small, not to meet a deadline.
	janitorEvery = 1 * time.Minute
)

// Janitor deletes rows the Redis backend would have expired via TTL.
//
// Every instance runs one. They race harmlessly: the deletes are idempotent and
// commutative, so there is no need for leader election and therefore no risk of a
// document going unreaped because the leader died.
type Janitor struct {
	q     Querier
	log   *slog.Logger
	every time.Duration
}

// NewJanitor builds a Janitor over an existing handle.
func NewJanitor(q Querier, log *slog.Logger) *Janitor {
	if log == nil {
		log = slog.Default()
	}
	return &Janitor{q: q, log: log, every: janitorEvery}
}

// Run sweeps until ctx is cancelled. It sweeps once immediately so a process that
// restarts often still makes progress, then on every tick.
func (j *Janitor) Run(ctx context.Context) {
	t := time.NewTicker(j.every)
	defer t.Stop()
	j.sweepOnce(ctx)
	for {
		select {
		case <-ctx.Done():
			return
		case <-t.C:
			j.sweepOnce(ctx)
		}
	}
}

// sweepOnce runs all three deletes. A failed sweep is a growing table, not a
// correctness problem, so it logs and the next tick retries.
func (j *Janitor) sweepOnce(ctx context.Context) {
	for _, s := range []struct {
		name string
		sql  string
		age  time.Duration
	}{
		{"ws_stream", reapStreamSQL, streamRetention},
		{"ws_instance", reapInstancesSQL, instanceRetention},
		{"ws_lock", reapLocksSQL, lockGrace},
	} {
		tag, err := j.q.Exec(ctx, s.sql, secs(s.age))
		if err != nil {
			if ctx.Err() == nil {
				j.log.Debug("janitor sweep failed", "table", s.name, "err", err)
			}
			continue
		}
		if n := tag.RowsAffected(); n > 0 {
			j.log.Debug("janitor reaped rows", "table", s.name, "rows", n)
		}
	}
}
