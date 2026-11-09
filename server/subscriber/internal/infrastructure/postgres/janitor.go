package postgres

import (
	"context"
	"fmt"
	"log"
	"time"
)

// Retention windows. These replace the Redis TTLs the Postgres backend doesn't
// have, and match them: 12h for the log streams, 24h for diagnostics and the
// job-complete key.
const (
	logRetention         = 12 * time.Hour
	diagnosticRetention  = 24 * time.Hour
	jobCompleteRetention = 24 * time.Hour

	// janitorEvery is the sweep period. Nothing reads row presence as a signal,
	// so this only bounds how far past its window a row can linger.
	janitorEvery = time.Minute
	// reapBatchSize bounds each DELETE so a large backlog never holds locks or
	// bloats a single statement; reapMaxBatches bounds one sweep, and the next
	// tick picks up whatever is left.
	reapBatchSize  = 5000
	reapMaxBatches = 100
)

// reapSQL deletes up to $2 rows of a table older than $1 seconds. SKIP LOCKED
// lets the janitors of several instances sweep the same table concurrently
// without blocking on each other's batches.
const reapSQL = `DELETE FROM %[1]s WHERE ctid = ANY(ARRAY(
  SELECT ctid FROM %[1]s
  WHERE created_at < now() - make_interval(secs => $1)
  LIMIT $2
  FOR UPDATE SKIP LOCKED
))`

// reapDiagnosticsSQL deletes stale diagnostics of jobs with no fresh one, as each LPUSH refreshed the Redis list's TTL.
const reapDiagnosticsSQL = `DELETE FROM job_diagnostics WHERE ctid = ANY(ARRAY(
  SELECT d.ctid FROM job_diagnostics d
  WHERE d.created_at < now() - make_interval(secs => $1)
    AND NOT EXISTS (
      SELECT 1 FROM job_diagnostics f
      WHERE f.job_id = d.job_id AND f.created_at >= now() - make_interval(secs => $1))
  LIMIT $2
  FOR UPDATE SKIP LOCKED
))`

type reapTarget struct {
	table string
	age   time.Duration
	sql   string // overrides reapSQL when set
}

var reapTargets = []reapTarget{
	{table: "job_logs", age: logRetention},
	{table: "job_user_facing_logs", age: logRetention},
	{table: "job_diagnostics", age: diagnosticRetention, sql: reapDiagnosticsSQL},
	{table: "job_complete_events", age: jobCompleteRetention},
}

// Janitor deletes rows the Redis backend would have expired via TTL.
//
// Every instance runs one. They race harmlessly: the deletes are idempotent and
// commutative, so there is no leader election and no risk of data going
// unreaped because a leader died.
type Janitor struct {
	q         Querier
	every     time.Duration
	batchSize int
}

func NewJanitor(q Querier) *Janitor {
	return &Janitor{q: q, every: janitorEvery, batchSize: reapBatchSize}
}

// Run sweeps until ctx is cancelled. It sweeps once immediately so a process
// that restarts often still makes progress, then on every tick.
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

// sweepOnce reaps every table. A failed sweep is a growing table, not a
// correctness problem, so it logs and the next tick retries.
func (j *Janitor) sweepOnce(ctx context.Context) {
	for _, target := range reapTargets {
		n, err := j.reap(ctx, target)
		if err != nil {
			if ctx.Err() == nil {
				log.Printf("WARN: janitor sweep of %s failed: %v", target.table, err)
			}
			continue
		}
		if n > 0 {
			log.Printf("DEBUG: janitor reaped %d rows from %s", n, target.table)
		}
	}
}

func (j *Janitor) reap(ctx context.Context, target reapTarget) (int64, error) {
	sql := target.sql
	if sql == "" {
		sql = fmt.Sprintf(reapSQL, target.table)
	}
	var total int64
	for range reapMaxBatches {
		tag, err := j.q.Exec(ctx, sql, target.age.Seconds(), j.batchSize)
		if err != nil {
			return total, err
		}
		total += tag.RowsAffected()
		if tag.RowsAffected() < int64(j.batchSize) {
			break
		}
	}
	return total, nil
}
