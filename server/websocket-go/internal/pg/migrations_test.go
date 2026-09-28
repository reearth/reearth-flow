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
// rest and panics loudly if anything else is called.
type recordingTx struct {
	pgx.Tx
	execs *[]string
	done  *string
}

func (t recordingTx) Exec(_ context.Context, sql string, _ ...any) (pgconn.CommandTag, error) {
	*t.execs = append(*t.execs, sql)
	return pgconn.CommandTag{}, nil
}

// Query answers the applied-versions lookup with an empty result, so every embedded
// migration looks unapplied.
func (t recordingTx) Query(context.Context, string, ...any) (pgx.Rows, error) {
	return emptyRows{}, nil
}

// QueryRow answers the baseline probe with "schema absent", so the fake exercises the
// fresh-database path rather than baselining.
func (t recordingTx) QueryRow(context.Context, string, ...any) pgx.Row { return falseRow{} }
func (t recordingTx) Commit(context.Context) error                     { *t.done = "commit"; return nil }
func (t recordingTx) Rollback(context.Context) error                   { return nil }

type falseRow struct{}

func (falseRow) Scan(dest ...any) error {
	if b, ok := dest[0].(*bool); ok {
		*b = false
	}
	return nil
}

type emptyRows struct{ pgx.Rows }

func (emptyRows) Next() bool { return false }
func (emptyRows) Close()     {}
func (emptyRows) Err() error { return nil }

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
// two-Exec form had: pg_advisory_xact_lock releases at commit, so issuing it as its
// own Exec on a pool released it before the schema was applied, and on a pool the
// two calls could land on different connections.
//
// The interleaving is impractical to reproduce deterministically, so this pins the
// structure: one transaction, the lock FIRST, every embedded migration applied and
// recorded within it, and a commit.
func TestMigrateHoldsLockAndDDLInOneTransaction(t *testing.T) {
	q := &recordingQuerier{}
	if err := Migrate(context.Background(), q); err != nil {
		t.Fatalf("Migrate: %v", err)
	}

	if q.begun != 1 {
		t.Errorf("Begin called %d times, want exactly 1", q.begun)
	}
	if len(q.execs) == 0 {
		t.Fatal("no statements ran inside the transaction")
	}
	if q.execs[0] != lockMigrationSQL {
		t.Errorf("first statement = %q, want the migration lock; a lock taken later protects nothing", q.execs[0])
	}
	if !strings.Contains(q.execs[1], "ws_schema_migrations") {
		t.Errorf("second statement = %q, want the revision table", q.execs[1])
	}

	names, err := migrationNames()
	if err != nil {
		t.Fatalf("migrationNames: %v", err)
	}
	joined := strings.Join(q.execs, "\n")
	if !strings.Contains(joined, "CREATE UNLOGGED TABLE") {
		t.Error("no UNLOGGED table was created; the vendored Atlas migration did not run")
	}
	// Each migration contributes its body plus a row in ws_schema_migrations.
	if want := 2 + 2*len(names); len(q.execs) != want {
		t.Errorf("statements = %d, want %d (lock + revision table + %d migrations, each applied and recorded)",
			len(q.execs), want, len(names))
	}
	if q.done != "commit" {
		t.Error("transaction was not committed")
	}
}

// TestMigrateBaselinesAPreAtlasDatabase covers the upgrade path: a database created
// by the previous hand-written schema already has the tables but no revision rows.
// The Atlas migration is bare CREATE TABLE, so re-running it fails with "relation
// already exists" — Migrate must record it as applied instead.
func TestMigrateBaselinesAPreAtlasDatabase(t *testing.T) {
	pool := newPool(t)
	ctx := context.Background()

	// Simulate the old world: tables present, revision tracking absent.
	if _, err := pool.Exec(ctx, `DROP TABLE IF EXISTS ws_schema_migrations`); err != nil {
		t.Fatalf("drop revision table: %v", err)
	}

	if err := Migrate(ctx, pool); err != nil {
		t.Fatalf("Migrate on a pre-Atlas database: %v", err)
	}

	var n int
	if err := pool.QueryRow(ctx, `SELECT count(*) FROM ws_schema_migrations`).Scan(&n); err != nil {
		t.Fatalf("count revisions: %v", err)
	}
	if n == 0 {
		t.Error("nothing was recorded; the next startup would try to create the tables again")
	}

	// And it stays idempotent afterwards.
	if err := Migrate(ctx, pool); err != nil {
		t.Fatalf("second Migrate: %v", err)
	}
}
