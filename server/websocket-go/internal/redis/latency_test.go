package redis

import (
	"bytes"
	"context"
	"log/slog"
	"strconv"
	"strings"
	"testing"
	"time"

	goredis "github.com/redis/go-redis/v9"
)

// seedAgedEntries writes n entries to room's stream carrying ids two hours in the
// past. Age is derived from the id's millisecond component, so backdating the id is
// what makes these look like a deep replay rather than a fresh delivery.
func seedAgedEntries(t *testing.T, c *goredis.Client, room string, n int) {
	t.Helper()
	base := time.Now().Add(-2 * time.Hour).UnixMilli()
	for i := 0; i < n; i++ {
		id := strconv.FormatInt(base+int64(i), 10) + "-0"
		err := c.XAdd(context.Background(), &goredis.XAddArgs{
			Stream: streamKey(room),
			ID:     id,
			// clientId 4242 is not the relay's own id, so these are not self-filtered.
			Values: xaddValuesTS(msgTypeSync, []byte("old"), 4242, "0"),
		}).Err()
		if err != nil {
			t.Fatalf("seed %d: %v", i, err)
		}
	}
}

// relayWithLog builds a relay that logs JSON into buf so the test can assert on
// what the latency recorder did or did not report.
func relayWithLog(t *testing.T, addr string, buf *bytes.Buffer) *Relay {
	t.Helper()
	r, err := New(Options{Addr: addr, Logger: slog.New(slog.NewJSONHandler(buf, nil))})
	if err != nil {
		t.Fatalf("New relay: %v", err)
	}
	return r
}

// TestCatchUpIsNotMeasured is the Redis twin of the Postgres test of the same name.
// Catch-up replays entries that may be hours old; recording their age as delivery
// latency is what put p50 values of 188s and 296s into the first deployed benchmark.
// That age is replay depth, not what any editor experienced.
func TestCatchUpIsNotMeasured(t *testing.T) {
	const room = "proj-catchup"
	c, mr := newTestClient(t)
	seedAgedEntries(t, c, room, 5)

	var buf bytes.Buffer
	sink := &fakeSink{}
	r := relayWithLog(t, mr.Addr(), &buf)
	defer r.Close()

	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	if err := r.Start(ctx, sink); err != nil {
		t.Fatalf("Start: %v", err)
	}
	r.RoomActivated(room)

	waitFor(t, 5*time.Second, func() bool { return sink.count() == 5 })

	if err := r.Close(); err != nil { // flushes a final report
		t.Fatalf("Close: %v", err)
	}
	if strings.Contains(buf.String(), "relay latency") {
		t.Errorf("catch-up was recorded as delivery latency; a 2h-old entry reports a 7200000ms sample:\n%s", buf.String())
	}
}

// TestCatchUpFailureDoesNotPoisonLatency covers the path the twin above misses. A
// failed XRANGE used to return cursor "0", handing the whole stream to the live
// XREAD — which measures. The replay then landed in the percentiles anyway, so the
// guard held only while Redis was healthy. Catch-up is retried instead.
func TestCatchUpFailureDoesNotPoisonLatency(t *testing.T) {
	const room = "proj-catchup-fail"
	c, mr := newTestClient(t)
	seedAgedEntries(t, c, room, 5)

	// Fail every command before the reader starts, so the first catch-up cannot
	// succeed and the retry path is the one under test.
	mr.SetError("LOADING redis is loading the dataset in memory")

	var buf bytes.Buffer
	sink := &fakeSink{}
	r := relayWithLog(t, mr.Addr(), &buf)
	defer r.Close()

	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	if err := r.Start(ctx, sink); err != nil {
		t.Fatalf("Start: %v", err)
	}
	r.RoomActivated(room)

	// Recover after at least one backoff so the reader has certainly failed once.
	time.Sleep(readBackoff + 100*time.Millisecond)
	mr.SetError("")

	waitFor(t, 5*time.Second, func() bool { return sink.count() == 5 })

	if err := r.Close(); err != nil {
		t.Fatalf("Close: %v", err)
	}
	if strings.Contains(buf.String(), "relay latency") {
		t.Errorf("replay after a failed catch-up was measured; falling back to cursor %q is what caused this:\n%s", "0", buf.String())
	}
}
