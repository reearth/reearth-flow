package pg

import (
	"context"
	"testing"
)

// TestTablesAreUnlogged is the premise of the whole backend: if a table came back
// logged, every write would be paying WAL cost and the latency comparison against
// Redis would be measuring the wrong thing.
//
// relpersistence: 'u' = unlogged, 'p' = permanent, 't' = temporary.
func TestTablesAreUnlogged(t *testing.T) {
	pool := newPool(t)
	for _, table := range []string{"ws_stream", "ws_instance", "ws_lock"} {
		var persistence string
		err := pool.QueryRow(context.Background(), `
			SELECT c.relpersistence
			  FROM pg_class c
			  JOIN pg_namespace n ON n.oid = c.relnamespace
			 WHERE c.relname = $1
			   AND n.nspname = current_schema()`, table).Scan(&persistence)
		if err != nil {
			t.Fatalf("%s: %v", table, err)
		}
		if persistence != "u" {
			t.Errorf("%s relpersistence = %q, want \"u\" (unlogged)", table, persistence)
		}
	}
}

// TestMigrateIsIdempotent: every instance applies the schema at startup, so a
// second application must be a no-op rather than an error that crash-loops the
// service.
func TestMigrateIsIdempotent(t *testing.T) {
	pool := newPool(t) // already migrated once
	for i := 0; i < 2; i++ {
		if err := Migrate(context.Background(), pool); err != nil {
			t.Fatalf("re-apply %d: %v", i, err)
		}
	}
}
