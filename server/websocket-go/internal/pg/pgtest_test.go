package pg

import (
	"context"
	"fmt"
	"math/rand"
	"os"
	"sync"
	"testing"
	"time"

	"github.com/jackc/pgx/v5/pgxpool"
	"github.com/reearth/ygo/awareness"
	"github.com/reearth/ygo/cluster"
	"github.com/reearth/ygo/crdt"
)

// envDSN names the Postgres these tests run against. Unset skips them, matching the
// pattern in server/api's pgtest: CI supplies a service container, and a developer
// without one still gets a green `go test ./...`.
const envDSN = "REEARTH_FLOW_WS_TEST_PG"

// newPool returns a pool scoped to a schema private to this test, with the
// coordination schema applied. Per-test schemas rather than shared tables because
// the janitor and election tests delete rows unqualified by doc_id, so they would
// otherwise reach into a concurrent test's data.
func newPool(t *testing.T) *pgxpool.Pool {
	t.Helper()
	dsn := os.Getenv(envDSN)
	if dsn == "" {
		t.Skipf("%s not set; skipping Postgres-backed tests", envDSN)
	}

	admin, err := pgxpool.New(context.Background(), dsn)
	if err != nil {
		t.Fatalf("connect: %v", err)
	}
	defer admin.Close()

	schema := fmt.Sprintf("wstest_%d_%d", time.Now().UnixNano(), rand.Intn(1<<16))
	if _, err := admin.Exec(context.Background(), "CREATE SCHEMA "+schema); err != nil {
		t.Fatalf("create schema: %v", err)
	}

	cfg, err := pgxpool.ParseConfig(dsn)
	if err != nil {
		t.Fatalf("parse dsn: %v", err)
	}
	// search_path on the connection config, not a one-off SET: a pool hands out
	// many connections and a SET would only bind whichever one ran it.
	cfg.ConnConfig.RuntimeParams["search_path"] = schema

	pool, err := pgxpool.NewWithConfig(context.Background(), cfg)
	if err != nil {
		t.Fatalf("open pool: %v", err)
	}
	if err := Migrate(context.Background(), pool); err != nil {
		pool.Close()
		t.Fatalf("migrate: %v", err)
	}

	t.Cleanup(func() {
		pool.Close()
		drop, err := pgxpool.New(context.Background(), dsn)
		if err != nil {
			return
		}
		defer drop.Close()
		_, _ = drop.Exec(context.Background(), "DROP SCHEMA IF EXISTS "+schema+" CASCADE")
	})
	return pool
}

// fakeSink records what a relay injects. Inject must be safe for concurrent calls
// on distinct rooms per the cluster.Sink contract, hence the mutex.
type fakeSink struct {
	mu   sync.Mutex
	got  []cluster.Inbound
	fail error
}

func (s *fakeSink) Inject(_ context.Context, in cluster.Inbound) error {
	s.mu.Lock()
	defer s.mu.Unlock()
	if s.fail != nil {
		return s.fail
	}
	// Copy: the relay may reuse the buffer after Inject returns.
	s.got = append(s.got, cluster.Inbound{
		Room: in.Room,
		Kind: in.Kind,
		Data: append([]byte(nil), in.Data...),
	})
	return nil
}

func (s *fakeSink) Rooms() []string                                  { return nil }
func (s *fakeSink) GetAwareness(string) (*awareness.Awareness, bool) { return nil, false }
func (s *fakeSink) GetDoc(string) *crdt.Doc                          { return nil }

func (s *fakeSink) count() int {
	s.mu.Lock()
	defer s.mu.Unlock()
	return len(s.got)
}

func (s *fakeSink) payloads() [][]byte {
	s.mu.Lock()
	defer s.mu.Unlock()
	out := make([][]byte, 0, len(s.got))
	for _, in := range s.got {
		out = append(out, in.Data)
	}
	return out
}

// countingFlusher records FlushRoom calls and can be made to fail.
type countingFlusher struct {
	mu    sync.Mutex
	calls []string
	fail  error
}

func (f *countingFlusher) FlushRoom(_ context.Context, room string) error {
	f.mu.Lock()
	defer f.mu.Unlock()
	f.calls = append(f.calls, room)
	return f.fail
}

func (f *countingFlusher) count() int {
	f.mu.Lock()
	defer f.mu.Unlock()
	return len(f.calls)
}

// startRelay builds a poll-mode relay with a fast tick, started against sink.
func startRelay(t *testing.T, pool *pgxpool.Pool, sink cluster.Sink, fl Flusher) *Relay {
	t.Helper()
	return startRelayWith(t, pool, sink, fl, Options{PollEvery: 5 * time.Millisecond})
}

// startNotifyRelay builds a notify-ONLY relay: no ticker at all, so any delivery it
// achieves is attributable to LISTEN and nothing else. That is what makes the
// notify tests meaningful rather than accidentally passing on a poll.
func startNotifyRelay(t *testing.T, pool *pgxpool.Pool, sink cluster.Sink, fl Flusher) *Relay {
	t.Helper()
	return startRelayWith(t, pool, sink, fl, Options{Notify: true})
}

// startRelayWith builds and starts a relay, filling in the handle, flusher and
// logger and registering cleanup.
func startRelayWith(t *testing.T, pool *pgxpool.Pool, sink cluster.Sink, fl Flusher, opts Options) *Relay {
	t.Helper()
	opts.Q = pool
	opts.Flusher = fl
	// Respect a caller-supplied logger: the latency tests capture the relay's own
	// output, and overwriting it here would silently discard what they assert on.
	if opts.Logger == nil {
		opts.Logger = testLogger(t)
	}
	r, err := New(opts)
	if err != nil {
		t.Fatalf("New: %v", err)
	}
	ctx, cancel := context.WithCancel(context.Background())
	t.Cleanup(func() { cancel(); _ = r.Close() })
	if err := r.Start(ctx, sink); err != nil {
		t.Fatalf("Start: %v", err)
	}
	return r
}

// eventually polls cond until it holds or the deadline passes. Used instead of a
// fixed sleep so a slow CI box does not turn a passing test red.
func eventually(t *testing.T, within time.Duration, what string, cond func() bool) {
	t.Helper()
	deadline := time.Now().Add(within)
	for time.Now().Before(deadline) {
		if cond() {
			return
		}
		time.Sleep(2 * time.Millisecond)
	}
	t.Fatalf("timed out after %s waiting for %s", within, what)
}

func rowCount(t *testing.T, pool *pgxpool.Pool, room string) int {
	t.Helper()
	var n int
	if err := pool.QueryRow(context.Background(),
		`SELECT count(*) FROM ws_stream WHERE doc_id = $1`, room).Scan(&n); err != nil {
		t.Fatalf("count rows: %v", err)
	}
	return n
}
