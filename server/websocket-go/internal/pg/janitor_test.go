package pg

import (
	"context"
	"testing"
	"time"
)

// TestJanitorReapsOnlyStaleRows: the janitor replaces the Redis key TTLs, so it
// must delete what has aged out and nothing else — reaping a live document's rows
// would strand every instance whose cursor is already past them.
func TestJanitorReapsOnlyStaleRows(t *testing.T) {
	pool := newPool(t)
	ctx := context.Background()

	// One row past the retention window, one inside it.
	if _, err := pool.Exec(ctx, `
		INSERT INTO ws_stream (doc_id, kind, data, client_id, created_at)
		VALUES ($1, 0, $2, 1, now() - interval '7 hours'),
		       ($1, 0, $3, 1, now())`, room, []byte("old"), []byte("fresh")); err != nil {
		t.Fatalf("seed stream: %v", err)
	}

	// One heartbeat past the window, one inside it.
	if _, err := pool.Exec(ctx, `
		INSERT INTO ws_instance (doc_id, client_id, seen_at)
		VALUES ($1, 1, now() - interval '10 minutes'),
		       ($1, 2, now())`, room); err != nil {
		t.Fatalf("seed instances: %v", err)
	}

	NewJanitor(pool, testLogger(t)).sweepOnce(ctx)

	var streamRows int
	if err := pool.QueryRow(ctx,
		`SELECT count(*) FROM ws_stream WHERE doc_id = $1`, room).Scan(&streamRows); err != nil {
		t.Fatalf("count stream: %v", err)
	}
	if streamRows != 1 {
		t.Errorf("ws_stream holds %d rows, want 1 (the fresh one)", streamRows)
	}

	var instanceRows int
	if err := pool.QueryRow(ctx,
		`SELECT count(*) FROM ws_instance WHERE doc_id = $1`, room).Scan(&instanceRows); err != nil {
		t.Fatalf("count instances: %v", err)
	}
	if instanceRows != 1 {
		t.Errorf("ws_instance holds %d rows, want 1 (the live one)", instanceRows)
	}
}

// TestJanitorReapIsIdempotent: every instance runs its own janitor with no leader
// election, so concurrent and repeated sweeps must be harmless.
func TestJanitorReapIsIdempotent(t *testing.T) {
	pool := newPool(t)
	ctx := context.Background()
	j := NewJanitor(pool, testLogger(t))
	for i := 0; i < 3; i++ {
		j.sweepOnce(ctx)
	}
}

// TestJanitorDoesNotAffectLiveness: liveness is decided by the seen_at predicate,
// never by whether a row has been reaped. instanceRetention is therefore longer
// than activeTimeout, so a row the election already treats as dead lingers
// harmlessly rather than the reap window becoming load-bearing.
func TestJanitorDoesNotAffectLiveness(t *testing.T) {
	if instanceRetention <= activeTimeout {
		t.Fatalf("instanceRetention (%s) must exceed activeTimeout (%s), or reaping would decide liveness",
			instanceRetention, activeTimeout)
	}

	pool := newPool(t)
	ctx := context.Background()

	// A heartbeat older than activeTimeout but younger than instanceRetention: the
	// janitor leaves the row, and the election must still count it dead.
	stale := activeTimeout + (instanceRetention-activeTimeout)/2
	if _, err := pool.Exec(ctx, `
		INSERT INTO ws_instance (doc_id, client_id, seen_at)
		VALUES ($1, 1, now() - make_interval(secs => $2))`, room, secs(stale)); err != nil {
		t.Fatalf("seed: %v", err)
	}

	NewJanitor(pool, testLogger(t)).sweepOnce(ctx)

	var rows int
	if err := pool.QueryRow(ctx, `SELECT count(*) FROM ws_instance WHERE doc_id = $1`, room).Scan(&rows); err != nil {
		t.Fatalf("count: %v", err)
	}
	if rows != 1 {
		t.Errorf("janitor reaped a row inside the retention window (%d left, want 1)", rows)
	}

	var active int
	if err := pool.QueryRow(ctx, activeInstancesSQL, room, secs(activeTimeout)).Scan(&active); err != nil {
		t.Fatalf("active count: %v", err)
	}
	if active != 0 {
		t.Errorf("active = %d, want 0; a stale heartbeat was counted live", active)
	}
}

// TestJanitorRunStopsOnCancel: it runs for the process lifetime, so a leaked
// goroutine per restart in a test binary (or a reload) would accumulate.
func TestJanitorRunStopsOnCancel(t *testing.T) {
	pool := newPool(t)
	ctx, cancel := context.WithCancel(context.Background())

	done := make(chan struct{})
	go func() {
		NewJanitor(pool, testLogger(t)).Run(ctx)
		close(done)
	}()

	cancel()
	select {
	case <-done:
	case <-time.After(2 * time.Second):
		t.Error("Run did not return after its context was cancelled")
	}
}
