package postgres

import (
	"context"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// The janitor stands in for the Redis TTLs, so it must delete what has aged out
// of each table's window and nothing else.
func TestJanitor_ReapsOnlyStaleRows(t *testing.T) {
	t.Parallel()
	pool := newPool(t)
	ctx := context.Background()

	_, err := pool.Exec(ctx, `
		INSERT INTO job_logs (job_id, workflow_id, ts, level, message, payload_hash, created_at) VALUES
		  ('j', '5b0f6a1e-3c2d-4e8f-9a7b-1c2d3e4f5a6b', now(), 'INFO', 'old',   'h1', now() - interval '13 hours'),
		  ('j', '5b0f6a1e-3c2d-4e8f-9a7b-1c2d3e4f5a6b', now(), 'INFO', 'fresh', 'h2', now() - interval '11 hours');
		INSERT INTO job_user_facing_logs (job_id, workflow_id, ts, level, message, payload_hash, created_at) VALUES
		  ('j', '5b0f6a1e-3c2d-4e8f-9a7b-1c2d3e4f5a6b', now(), 'INFO', 'old',   'h1', now() - interval '13 hours'),
		  ('j', '5b0f6a1e-3c2d-4e8f-9a7b-1c2d3e4f5a6b', now(), 'INFO', 'fresh', 'h2', now() - interval '11 hours');
		INSERT INTO job_diagnostics (job_id, data, created_at) VALUES
		  ('j-old', '{"code":"old"}',   now() - interval '25 hours'),
		  ('j',     '{"code":"fresh"}', now() - interval '13 hours');
		INSERT INTO job_complete_events (job_id, data, created_at) VALUES
		  ('old',   '{}', now() - interval '25 hours'),
		  ('fresh', '{}', now() - interval '13 hours');`)
	require.NoError(t, err)

	NewJanitor(pool).sweepOnce(ctx)

	assert.Equal(t, 1, count(t, pool, `SELECT count(*) FROM job_logs WHERE message = 'fresh'`))
	assert.Equal(t, 1, count(t, pool, `SELECT count(*) FROM job_logs`))
	assert.Equal(t, 1, count(t, pool, `SELECT count(*) FROM job_user_facing_logs WHERE message = 'fresh'`))
	assert.Equal(t, 1, count(t, pool, `SELECT count(*) FROM job_user_facing_logs`))
	// 13h is past the log window but inside the 24h diagnostic window.
	assert.Equal(t, 1, count(t, pool, `SELECT count(*) FROM job_diagnostics WHERE data->>'code' = 'fresh'`))
	assert.Equal(t, 1, count(t, pool, `SELECT count(*) FROM job_diagnostics`))
	assert.Equal(t, 1, count(t, pool, `SELECT count(*) FROM job_complete_events WHERE job_id = 'fresh'`))
	assert.Equal(t, 1, count(t, pool, `SELECT count(*) FROM job_complete_events`))
}

// A job's stale diagnostics stay while it has a fresh one.
func TestJanitor_KeepsDiagnosticsOfJobWithFreshEntry(t *testing.T) {
	t.Parallel()
	pool := newPool(t)
	ctx := context.Background()

	_, err := pool.Exec(ctx, `
		INSERT INTO job_diagnostics (job_id, data, created_at) VALUES
		  ('active', '{"code":"old"}',   now() - interval '25 hours'),
		  ('active', '{"code":"fresh"}', now() - interval '1 hour'),
		  ('stale',  '{"code":"old"}',   now() - interval '26 hours'),
		  ('stale',  '{"code":"older"}', now() - interval '25 hours');`)
	require.NoError(t, err)

	NewJanitor(pool).sweepOnce(ctx)

	assert.Equal(t, 2, count(t, pool, `SELECT count(*) FROM job_diagnostics WHERE job_id = 'active'`))
	assert.Equal(t, 0, count(t, pool, `SELECT count(*) FROM job_diagnostics WHERE job_id = 'stale'`))
}

// A backlog larger than one batch is drained across several bounded deletes in
// a single sweep.
func TestJanitor_ReapsInBatches(t *testing.T) {
	t.Parallel()
	pool := newPool(t)
	ctx := context.Background()

	_, err := pool.Exec(ctx, `
		INSERT INTO job_logs (job_id, workflow_id, ts, level, message, payload_hash, created_at)
		SELECT 'j', '5b0f6a1e-3c2d-4e8f-9a7b-1c2d3e4f5a6b', now(), 'INFO', 'old', int8send(g), now() - interval '13 hours'
		FROM generate_series(1, 25) g`)
	require.NoError(t, err)

	j := NewJanitor(pool)
	j.batchSize = 10
	n, err := j.reap(ctx, reapTarget{table: "job_logs", age: logRetention})
	require.NoError(t, err)
	assert.Equal(t, int64(25), n)
	assert.Equal(t, 0, count(t, pool, `SELECT count(*) FROM job_logs`))
}

// Every instance runs a janitor with no leader election, so concurrent and
// repeated sweeps must be harmless.
func TestJanitor_ConcurrentSweepsAreHarmless(t *testing.T) {
	t.Parallel()
	pool := newPool(t)
	ctx := context.Background()

	_, err := pool.Exec(ctx, `
		INSERT INTO job_logs (job_id, workflow_id, ts, level, message, payload_hash, created_at)
		SELECT 'j', '5b0f6a1e-3c2d-4e8f-9a7b-1c2d3e4f5a6b', now(), 'INFO', 'old', int8send(g), now() - interval '13 hours'
		FROM generate_series(1, 200) g`)
	require.NoError(t, err)

	done := make(chan struct{})
	for range 3 {
		go func() {
			j := NewJanitor(pool)
			j.batchSize = 7
			j.sweepOnce(ctx)
			done <- struct{}{}
		}()
	}
	for range 3 {
		<-done
	}
	assert.Equal(t, 0, count(t, pool, `SELECT count(*) FROM job_logs`))
}

func TestJanitor_RunStopsOnCancel(t *testing.T) {
	t.Parallel()
	pool := newPool(t)

	ctx, cancel := context.WithCancel(context.Background())
	stopped := make(chan struct{})
	go func() {
		NewJanitor(pool).Run(ctx)
		close(stopped)
	}()
	cancel()

	select {
	case <-stopped:
	case <-time.After(10 * time.Second):
		t.Fatal("Run did not return after cancel")
	}
}
