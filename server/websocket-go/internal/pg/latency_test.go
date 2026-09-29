package pg

import (
	"bytes"
	"context"
	"encoding/json"
	"log/slog"
	"strings"
	"testing"
	"time"

	"github.com/reearth/ygo/cluster"
)

// TestDeliveryLatencyIsReported is the end-to-end check on the measurement the
// backend comparison will be decided from. It asserts the number is produced, is
// attributed to the right arm, and is physically plausible — a recorder that
// silently observed nothing, or reported zeroes, would look exactly like a fast
// backend.
func TestDeliveryLatencyIsReported(t *testing.T) {
	pool := newPool(t)

	var buf bytes.Buffer
	readerSink := &fakeSink{}
	writer := startRelay(t, pool, &fakeSink{}, nil)
	reader := startRelayWith(t, pool, readerSink, nil, Options{
		PollEvery: 5 * time.Millisecond,
		Logger:    slog.New(slog.NewJSONHandler(&buf, nil)),
	})

	writer.RoomActivated(room)
	reader.RoomActivated(room)
	// Let the reader finish catch-up before publishing. Rows replayed during catch-up
	// are deliberately not measured, so publishing into that window would under-count.
	time.Sleep(300 * time.Millisecond)

	const updates = 20
	for i := 0; i < updates; i++ {
		if err := writer.Publish(context.Background(), cluster.Outbound{
			Room: room, Kind: cluster.KindSync, Data: []byte("x"),
		}); err != nil {
			t.Fatalf("Publish: %v", err)
		}
		time.Sleep(3 * time.Millisecond)
	}
	eventually(t, 10*time.Second, "every update to be delivered", func() bool {
		return readerSink.count() == updates
	})

	// Close emits the final report without waiting for the 30s tick.
	if err := reader.Close(); err != nil {
		t.Fatalf("Close: %v", err)
	}

	line := lastLatencyLine(t, buf.String())
	if got, _ := line["backend"].(string); got != "postgres/poll" {
		t.Errorf("backend = %v, want postgres/poll; the arm must be identifiable in the log", line["backend"])
	}
	if got, _ := line["count"].(float64); int(got) != updates {
		t.Errorf("count = %v, want %d", line["count"], updates)
	}
	p50, _ := line["p50_ms"].(float64)
	if p50 <= 0 {
		t.Errorf("p50_ms = %v, want a positive value; a zero here means nothing was measured", line["p50_ms"])
	}
	if p50 > 5000 {
		t.Errorf("p50_ms = %v, implausibly large for a local delivery", p50)
	}
	t.Logf("postgres/poll p50=%vms p95=%vms max=%vms over %v deliveries",
		line["p50_ms"], line["p95_ms"], line["max_ms"], line["count"])
}

// lastLatencyLine returns the final "relay latency" record in a JSON log.
func lastLatencyLine(t *testing.T, logged string) map[string]any {
	t.Helper()
	var found map[string]any
	for _, l := range strings.Split(strings.TrimSpace(logged), "\n") {
		if !strings.Contains(l, "relay latency") {
			continue
		}
		var rec map[string]any
		if err := json.Unmarshal([]byte(l), &rec); err != nil {
			t.Fatalf("parse log line %q: %v", l, err)
		}
		found = rec
	}
	if found == nil {
		t.Fatalf("no \"relay latency\" record was logged; the measurement never ran:\n%s", logged)
	}
	return found
}

// TestCatchUpIsNotMeasured guards a flaw found in the first deployed benchmark run:
// catch-up replays rows that may be hours old, and recording their age as delivery
// latency put p50 values of 5s, 188s and 296s into the reported percentiles. That age
// is replay depth, not what any editor experienced.
//
// Rows are aged well beyond any plausible delivery time, then a relay activates the
// room and replays them. Nothing may be recorded.
func TestCatchUpIsNotMeasured(t *testing.T) {
	pool := newPool(t)
	ctx := context.Background()

	for i := 0; i < 5; i++ {
		if _, err := pool.Exec(ctx, appendSQL, room, kindSync, []byte("old"), int64(4242)); err != nil {
			t.Fatalf("seed: %v", err)
		}
	}
	// Backdate them: catch-up would otherwise record a near-zero age and pass
	// regardless of whether the guard works.
	if _, err := pool.Exec(ctx, `UPDATE ws_stream SET created_at = now() - interval '2 hours' WHERE doc_id = $1`, room); err != nil {
		t.Fatalf("backdate: %v", err)
	}

	var buf bytes.Buffer
	sink := &fakeSink{}
	r := startRelayWith(t, pool, sink, nil, Options{
		PollEvery: 5 * time.Millisecond,
		Logger:    slog.New(slog.NewJSONHandler(&buf, nil)),
	})
	r.RoomActivated(room)

	eventually(t, 10*time.Second, "catch-up to replay the backdated rows", func() bool {
		return sink.count() == 5
	})

	if err := r.Close(); err != nil { // flushes a final report
		t.Fatalf("Close: %v", err)
	}
	if strings.Contains(buf.String(), "relay latency") {
		t.Errorf("catch-up was recorded as delivery latency; a 2h-old row would report a 7200000ms sample:\n%s", buf.String())
	}
}
