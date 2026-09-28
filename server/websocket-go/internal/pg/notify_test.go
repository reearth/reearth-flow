package pg

import (
	"context"
	"os"
	"strings"
	"testing"
	"time"

	"github.com/jackc/pgx/v5/pgxpool"
	"github.com/reearth/ygo/cluster"
)

// TestNotifyDeliversWithNoPolling proves LISTEN actually works, and is written so
// it cannot pass by accident: the reader has NO ticker (PollEvery is zero), so the
// only thing that can move it is a notification.
func TestNotifyDeliversWithNoPolling(t *testing.T) {
	pool := newPool(t)

	readerSink := &fakeSink{}
	writer := startRelay(t, pool, &fakeSink{}, nil)      // poll-mode writer
	reader := startNotifyRelay(t, pool, readerSink, nil) // notify-only reader

	if reader.poll != 0 {
		t.Fatalf("reader poll = %s, want 0; this test must not be able to pass by polling", reader.poll)
	}

	writer.RoomActivated(room)
	reader.RoomActivated(room)

	// Let the listener attach before the write, so the wakeup path is what is
	// exercised rather than the activation catch-up.
	time.Sleep(200 * time.Millisecond)

	if err := writer.Publish(context.Background(), cluster.Outbound{
		Room: room, Kind: cluster.KindSync, Data: []byte("via-notify"),
	}); err != nil {
		t.Fatalf("Publish: %v", err)
	}

	eventually(t, 5*time.Second, "delivery via LISTEN with no ticker", func() bool {
		return readerSink.count() == 1
	})
	if got := string(readerSink.payloads()[0]); got != "via-notify" {
		t.Errorf("payload = %q, want %q", got, "via-notify")
	}
}

// TestNotifyOnlyStillReplaysHistory: with no ticker, a room that did not replay on
// activation would show a joining client an empty document until somebody happened
// to type. Catch-up runs on the reader goroutine regardless of fan-out, and this
// pins that.
func TestNotifyOnlyStillReplaysHistory(t *testing.T) {
	pool := newPool(t)

	writer := startRelay(t, pool, &fakeSink{}, nil)
	writer.RoomActivated(room)
	for i := 0; i < 3; i++ {
		if err := writer.Publish(context.Background(), cluster.Outbound{
			Room: room, Kind: cluster.KindSync, Data: []byte{byte(i)},
		}); err != nil {
			t.Fatalf("Publish: %v", err)
		}
	}
	eventually(t, 3*time.Second, "history to persist", func() bool {
		return rowCount(t, pool, room) == 3
	})

	// A notify-only instance joins after the fact and must still see all of it.
	lateSink := &fakeSink{}
	late := startNotifyRelay(t, pool, lateSink, nil)
	late.RoomActivated(room)

	eventually(t, 5*time.Second, "history replay on a notify-only reader", func() bool {
		return lateSink.count() == 3
	})
}

// TestNotifyIsSentRegardlessOfLocalFanout: a poll-mode instance that did not notify
// would leave notify-mode peers blind to its writes, making a mixed rollout
// silently lossy. The writer here polls; the reader only listens.
func TestNotifyIsSentRegardlessOfLocalFanout(t *testing.T) {
	pool := newPool(t)

	readerSink := &fakeSink{}
	pollWriter := startRelay(t, pool, &fakeSink{}, nil)
	notifyReader := startNotifyRelay(t, pool, readerSink, nil)

	pollWriter.RoomActivated(room)
	notifyReader.RoomActivated(room)
	time.Sleep(200 * time.Millisecond)

	if err := pollWriter.Publish(context.Background(), cluster.Outbound{
		Room: room, Kind: cluster.KindSync, Data: []byte("from-poller"),
	}); err != nil {
		t.Fatalf("Publish: %v", err)
	}

	eventually(t, 5*time.Second, "a poll-mode writer's update to wake a notify reader", func() bool {
		return readerSink.count() == 1
	})
}

// TestHybridDeliversWithBothMechanisms: hybrid must work when either path alone
// would, so a slow poll plus LISTEN still delivers promptly.
func TestHybridDeliversWithBothMechanisms(t *testing.T) {
	pool := newPool(t)

	readerSink := &fakeSink{}
	writer := startRelay(t, pool, &fakeSink{}, nil)
	// A deliberately slow poll: prompt delivery must come from the notification.
	reader := startRelayWith(t, pool, readerSink, nil, Options{
		PollEvery: 30 * time.Second,
		Notify:    true,
	})

	writer.RoomActivated(room)
	reader.RoomActivated(room)
	time.Sleep(200 * time.Millisecond)

	if err := writer.Publish(context.Background(), cluster.Outbound{
		Room: room, Kind: cluster.KindSync, Data: []byte("hybrid"),
	}); err != nil {
		t.Fatalf("Publish: %v", err)
	}

	eventually(t, 5*time.Second, "hybrid delivery ahead of its 30s poll", func() bool {
		return readerSink.count() == 1
	})
}

// TestRelayNeedsAWakeupMechanism: a relay with neither a ticker nor LISTEN would
// activate rooms, accept writes, and never deliver anything — a document that looks
// healthy and has silently stopped syncing. Refuse it at construction.
func TestRelayNeedsAWakeupMechanism(t *testing.T) {
	pool := newPool(t)
	_, err := New(Options{Q: pool})
	if err == nil {
		t.Fatal("New with no poll and no notify = nil error, want a refusal")
	}
	if !strings.Contains(err.Error(), "PollEvery") {
		t.Errorf("New = %v, want an error naming PollEvery", err)
	}
}

// TestNotifyRequiresAPool: LISTEN needs a connection it can hold, which a bare
// Querier cannot promise. Failing at construction beats accepting the config and
// never delivering a notification.
func TestNotifyRequiresAPool(t *testing.T) {
	pool := newPool(t)
	conn, err := pool.Acquire(context.Background())
	if err != nil {
		t.Fatalf("acquire: %v", err)
	}
	defer conn.Release()

	// *pgxpool.Conn satisfies Querier but is not a pool.
	if _, err := New(Options{Q: conn, Notify: true}); err == nil {
		t.Error("New with Notify and a non-pool handle = nil error, want a refusal")
	}
}

// TestWakeRoomBroadcastsToEveryRoom covers the listener-reconnect path: notifications
// published while this instance had no listener are gone, so on reattach every
// resident room must be woken. Without it a connection blip strands every room until
// its next local write.
func TestWakeRoomBroadcastsToEveryRoom(t *testing.T) {
	pool := newPool(t)
	r := startNotifyRelay(t, pool, &fakeSink{}, nil)

	rooms := []string{
		"11111111-1111-1111-1111-111111111111",
		"22222222-2222-2222-2222-222222222222",
	}
	for _, room := range rooms {
		r.RoomActivated(room)
	}

	// Drain any wakeup the listener's own attach already queued.
	r.mu.Lock()
	for _, room := range rooms {
		select {
		case <-r.rooms[room].wake:
		default:
		}
	}
	r.mu.Unlock()

	r.wakeRoom("") // the reconnect broadcast

	r.mu.Lock()
	defer r.mu.Unlock()
	for _, room := range rooms {
		select {
		case <-r.rooms[room].wake:
		default:
			t.Errorf("room %s was not woken by the reconnect broadcast", room)
		}
	}
}

// TestNotifyRefusesAPoolItWouldMonopolise: the listener holds one connection for the
// life of the process. On a pool of 1 that leaves nothing for reads, writes,
// heartbeats or the election, and every query blocks until its context expires —
// while /health still reports ok, because the probe never gets a connection either
// and simply times out with everything else.
//
// Measured before this guard existed: with pool_max_conns=1 and notify enabled, a
// plain SELECT 1 failed with "context deadline exceeded".
func TestNotifyRefusesAPoolItWouldMonopolise(t *testing.T) {
	cfg, err := pgxpool.ParseConfig(os.Getenv(envDSN) + "&pool_max_conns=1")
	if err != nil {
		t.Skipf("no test database: %v", err)
	}
	pool, err := pgxpool.NewWithConfig(context.Background(), cfg)
	if err != nil {
		t.Skipf("no test database: %v", err)
	}
	defer pool.Close()

	_, err = New(Options{Q: pool, Notify: true})
	if err == nil {
		t.Fatal("New accepted a one-connection pool with notify; the listener would starve every other query")
	}
	if !strings.Contains(err.Error(), "pool_max_conns") {
		t.Errorf("New = %v, want an error naming pool_max_conns", err)
	}
}

// TestNotifyAcceptsATwoConnectionPool: two is the hard floor — one for the listener,
// one for everything else. Rejecting it would make the guard stricter than the
// constraint.
func TestNotifyAcceptsATwoConnectionPool(t *testing.T) {
	cfg, err := pgxpool.ParseConfig(os.Getenv(envDSN) + "&pool_max_conns=2")
	if err != nil {
		t.Skipf("no test database: %v", err)
	}
	pool, err := pgxpool.NewWithConfig(context.Background(), cfg)
	if err != nil {
		t.Skipf("no test database: %v", err)
	}
	defer pool.Close()

	r, err := New(Options{Q: pool, Notify: true})
	if err != nil {
		t.Fatalf("New with a two-connection pool: %v", err)
	}
	_ = r.Close()
}
