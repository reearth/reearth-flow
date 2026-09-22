package pg

import (
	"context"
	"strings"
	"testing"
	"time"

	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgconn"
)

// TestMigrateWaitsForTheLock proves Migrate blocks while another session holds the
// migration lock. Note this does NOT distinguish the current implementation from the
// broken two-Exec one — both block on ACQUIRING the lock. What the two-Exec form got
// wrong was releasing it immediately afterwards, which
// TestMigrateHoldsLockAndDDLInOneTransaction is the test for.
func TestMigrateWaitsForTheLock(t *testing.T) {
	pool := newPool(t)
	ctx := context.Background()

	blocker, err := pool.Begin(ctx)
	if err != nil {
		t.Fatalf("begin blocker: %v", err)
	}
	defer func() { _ = blocker.Rollback(ctx) }()
	if _, err := blocker.Exec(ctx, lockMigrationSQL); err != nil {
		t.Fatalf("blocker take lock: %v", err)
	}

	blocked, cancel := context.WithTimeout(ctx, 750*time.Millisecond)
	defer cancel()

	if err := Migrate(blocked, pool); err == nil {
		t.Fatal("Migrate completed while another session held the migration lock; the lock is not serialising startups")
	}

	// Releasing the blocker must let it through, so the failure above is the lock
	// and not something incidental about the DDL.
	if err := blocker.Rollback(ctx); err != nil {
		t.Fatalf("release blocker: %v", err)
	}
	if err := Migrate(ctx, pool); err != nil {
		t.Fatalf("Migrate after releasing the lock: %v", err)
	}
}

// TestConcurrentMigrateSucceeds mirrors several instances starting at once, which is
// the situation the lock exists for: IF NOT EXISTS alone does not make concurrent
// CREATE TABLE safe, as they can still collide on pg_type's unique index.
func TestConcurrentMigrateSucceeds(t *testing.T) {
	pool := newPool(t)
	ctx := context.Background()

	errs := make(chan error, 8)
	for i := 0; i < 8; i++ {
		go func() { errs <- Migrate(ctx, pool) }()
	}
	for i := 0; i < 8; i++ {
		if err := <-errs; err != nil {
			t.Errorf("concurrent Migrate: %v", err)
		}
	}
}

// recordingTx captures the statements Migrate runs inside its transaction. Only the
// methods Migrate touches are implemented; the embedded nil interface satisfies the
// rest and would panic loudly if anything else were called.
type recordingTx struct {
	pgx.Tx
	execs *[]string
	done  *string
}

func (t recordingTx) Exec(_ context.Context, sql string, _ ...any) (pgconn.CommandTag, error) {
	*t.execs = append(*t.execs, sql)
	return pgconn.CommandTag{}, nil
}
func (t recordingTx) Commit(context.Context) error   { *t.done = "commit"; return nil }
func (t recordingTx) Rollback(context.Context) error { return nil }

type recordingQuerier struct {
	Querier
	begun int
	execs []string
	done  string
}

func (q *recordingQuerier) Begin(context.Context) (pgx.Tx, error) {
	q.begun++
	return recordingTx{execs: &q.execs, done: &q.done}, nil
}

// TestMigrateHoldsLockAndDDLInOneTransaction is the regression test for the bug the
// two-Exec form had: pg_advisory_xact_lock is released at commit, so issuing it as
// its own Exec on a pool released it before the schema was applied — and the two
// calls could even land on different pooled connections. Concurrent startups were
// not serialised at all, despite the comment claiming they were.
//
// That interleaving is impractical to reproduce deterministically, so this pins the
// structure instead: exactly one transaction, the lock first, the DDL second, and
// both on the SAME transaction handle rather than the pool.
func TestMigrateHoldsLockAndDDLInOneTransaction(t *testing.T) {
	q := &recordingQuerier{}
	if err := Migrate(context.Background(), q); err != nil {
		t.Fatalf("Migrate: %v", err)
	}

	if q.begun != 1 {
		t.Errorf("Begin called %d times, want exactly 1", q.begun)
	}
	if len(q.execs) != 2 {
		t.Fatalf("statements in transaction = %d (%v), want 2", len(q.execs), q.execs)
	}
	if q.execs[0] != lockMigrationSQL {
		t.Errorf("first statement = %q, want the migration lock; a lock taken after the DDL protects nothing", q.execs[0])
	}
	if !strings.Contains(q.execs[1], "CREATE UNLOGGED TABLE") {
		t.Errorf("second statement does not look like the schema: %q", q.execs[1])
	}
	if q.done != "commit" {
		t.Error("transaction was not committed")
	}
}
