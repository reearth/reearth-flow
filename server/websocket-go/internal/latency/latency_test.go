package latency

import (
	"bytes"
	"context"
	"encoding/json"
	"log/slog"
	"strings"
	"sync"
	"testing"
	"time"
)

func newTestRecorder(t *testing.T) (*Recorder, *bytes.Buffer) {
	t.Helper()
	var buf bytes.Buffer
	log := slog.New(slog.NewJSONHandler(&buf, nil))
	return New(log, "test"), &buf
}

// TestPercentilesUseNearestRank: interpolated percentiles can report a latency no
// update actually experienced, which is the wrong thing to hand someone choosing
// between backends.
func TestPercentilesUseNearestRank(t *testing.T) {
	r, buf := newTestRecorder(t)
	for i := 1; i <= 100; i++ {
		r.Observe(time.Duration(i) * time.Millisecond)
	}
	r.report()

	var rec map[string]any
	if err := json.Unmarshal(bytes.TrimSpace(buf.Bytes()), &rec); err != nil {
		t.Fatalf("parse log: %v (%s)", err, buf.String())
	}
	// Samples are 1..100ms, so nearest-rank puts p50 at 50ms, p95 at 95ms and p99
	// at 99ms — every reported figure is a value some update actually saw.
	for field, want := range map[string]float64{
		"p50_ms": 50, "p95_ms": 95, "p99_ms": 99, "max_ms": 100, "count": 100,
	} {
		if got, _ := rec[field].(float64); got != want {
			t.Errorf("%s = %v, want %v", field, rec[field], want)
		}
	}
}

// TestSubMillisecondIsNotFlooredToZero is the specific failure this package works
// around: the existing HTTP timing uses whole milliseconds, so every fast request
// logs 0 and the distribution is unreadable exactly where it matters.
func TestSubMillisecondIsNotFlooredToZero(t *testing.T) {
	r, buf := newTestRecorder(t)
	r.Observe(250 * time.Microsecond)
	r.report()

	if !strings.Contains(buf.String(), "0.25") {
		t.Errorf("250µs was not reported at sub-millisecond resolution: %s", buf.String())
	}
}

// TestIdleWindowLogsNothing: an idle document must be silent. A report every
// interval regardless would bury real numbers in zeroes across the many
// environments this runs in.
func TestIdleWindowLogsNothing(t *testing.T) {
	r, buf := newTestRecorder(t)
	r.report()
	if buf.Len() != 0 {
		t.Errorf("an empty window logged %q, want silence", buf.String())
	}
}

// TestReportResetsTheWindow: percentiles must describe the interval just elapsed,
// not everything since boot, or a latency regression would be masked by however
// much good history preceded it.
func TestReportResetsTheWindow(t *testing.T) {
	r, buf := newTestRecorder(t)
	r.Observe(time.Second)
	r.report()
	buf.Reset()
	r.report()
	if buf.Len() != 0 {
		t.Errorf("second report emitted %q, want silence after the reset", buf.String())
	}
}

// TestNegativeObservationsAreDropped: the Redis backend compares two clocks, so
// skew can produce a negative age. Recording it would drag a percentile below what
// any update actually experienced.
func TestNegativeObservationsAreDropped(t *testing.T) {
	r, buf := newTestRecorder(t)
	r.Observe(-5 * time.Millisecond)
	r.report()
	if buf.Len() != 0 {
		t.Errorf("a negative observation was recorded: %s", buf.String())
	}
}

// TestCountExceedsSamplesWhenReservoirFills: past maxSamples the recorder keeps a
// representative sample, not everything — but the reported count must still be the
// true total, or a busy window would look quiet.
func TestCountExceedsSamplesWhenReservoirFills(t *testing.T) {
	r, buf := newTestRecorder(t)
	total := maxSamples + 1000
	for i := 0; i < total; i++ {
		r.Observe(time.Millisecond)
	}
	r.report()

	var rec map[string]any
	if err := json.Unmarshal(bytes.TrimSpace(buf.Bytes()), &rec); err != nil {
		t.Fatalf("parse log: %v", err)
	}
	if got, _ := rec["count"].(float64); int(got) != total {
		t.Errorf("count = %v, want %d", rec["count"], total)
	}
	if got, _ := rec["sampled"].(float64); int(got) != maxSamples {
		t.Errorf("sampled = %v, want %d", rec["sampled"], maxSamples)
	}
}

// TestObserveIsConcurrencySafe: every room's reader goroutine reports into one
// recorder.
func TestObserveIsConcurrencySafe(t *testing.T) {
	r, buf := newTestRecorder(t)
	var wg sync.WaitGroup
	for i := 0; i < 8; i++ {
		wg.Go(func() {
			for j := 0; j < 500; j++ {
				r.Observe(time.Millisecond)
			}
		})
	}
	wg.Wait()
	r.report()

	var rec map[string]any
	if err := json.Unmarshal(bytes.TrimSpace(buf.Bytes()), &rec); err != nil {
		t.Fatalf("parse log: %v", err)
	}
	if got, _ := rec["count"].(float64); int(got) != 4000 {
		t.Errorf("count = %v, want 4000", rec["count"])
	}
}

// TestRunReportsOnShutdown: a process that exits before its first tick must still
// report, or a short benchmark run would produce no numbers at all.
func TestRunReportsOnShutdown(t *testing.T) {
	r, buf := newTestRecorder(t)
	r.Observe(3 * time.Millisecond)

	ctx, cancel := context.WithCancel(context.Background())
	done := make(chan struct{})
	go func() { r.Run(ctx); close(done) }()

	cancel()
	select {
	case <-done:
	case <-time.After(2 * time.Second):
		t.Fatal("Run did not return after cancellation")
	}
	if !strings.Contains(buf.String(), "relay latency") {
		t.Errorf("no final report on shutdown: %q", buf.String())
	}
}

// TestNilRecorderIsSafe: callers observe on a hot path and must not need a nil
// check.
func TestNilRecorderIsSafe(t *testing.T) {
	var r *Recorder
	r.Observe(time.Millisecond)
	r.Run(context.Background())
}

// TestMaxIsTheWindowMaxNotTheSampleMax guards a bug that would quietly flatter a
// backend: max_ms used to be read off the retained samples, but once the reservoir
// is full a high outlier can be evicted. Percentiles tolerate sampling; a maximum
// does not, and it is exactly the field someone reads as "the worst this backend
// did".
//
// One large observation is made first, then far more than the reservoir holds, so
// it is very unlikely to survive in samples. The reported max must still be it.
func TestMaxIsTheWindowMaxNotTheSampleMax(t *testing.T) {
	r, buf := newTestRecorder(t)

	const outlier = 9 * time.Second
	r.Observe(outlier)
	for i := 0; i < maxSamples*12; i++ {
		r.Observe(time.Millisecond)
	}
	r.report()

	var rec map[string]any
	if err := json.Unmarshal(bytes.TrimSpace(buf.Bytes()), &rec); err != nil {
		t.Fatalf("parse log: %v", err)
	}

	// Confirm the reservoir actually engaged, or the test proves nothing.
	sampled, _ := rec["sampled"].(float64)
	count, _ := rec["count"].(float64)
	if int(sampled) >= int(count) {
		t.Fatalf("reservoir did not engage (sampled=%v count=%v); test is not exercising eviction", sampled, count)
	}

	if got, _ := rec["max_ms"].(float64); got != 9000 {
		t.Errorf("max_ms = %v, want 9000; an evicted outlier must still be reported", rec["max_ms"])
	}
}

// TestMaxResetsWithTheWindow: the maximum describes one interval, like the
// percentiles. Carrying it forward would pin every later report to the worst spike
// since boot.
func TestMaxResetsWithTheWindow(t *testing.T) {
	r, buf := newTestRecorder(t)

	r.Observe(5 * time.Second)
	r.report()
	buf.Reset()

	r.Observe(2 * time.Millisecond)
	r.report()

	var rec map[string]any
	if err := json.Unmarshal(bytes.TrimSpace(buf.Bytes()), &rec); err != nil {
		t.Fatalf("parse log: %v", err)
	}
	if got, _ := rec["max_ms"].(float64); got != 2 {
		t.Errorf("max_ms = %v, want 2; the previous window's spike leaked", rec["max_ms"])
	}
}

// TestPercentilesOnSmallWindows is the case the previous index formula got wrong.
// Scaling by n-1 put both p95 and p99 on the smaller of two samples, so a window
// whose worst delivery was 100ms reported a 1ms tail.
//
// Small windows are not an edge case here: the quiet environments this recorder
// exists to compare produce them constantly, and hiding the tail there defeats the
// measurement.
func TestPercentilesOnSmallWindows(t *testing.T) {
	for _, tc := range []struct {
		name    string
		samples []time.Duration
		p50     float64
		p95     float64
		p99     float64
	}{
		{"two samples", []time.Duration{time.Millisecond, 100 * time.Millisecond}, 1, 100, 100},
		{"one sample", []time.Duration{7 * time.Millisecond}, 7, 7, 7},
		{"three samples", []time.Duration{time.Millisecond, 2 * time.Millisecond, 90 * time.Millisecond}, 2, 90, 90},
		{"ten samples", []time.Duration{
			1 * time.Millisecond, 2 * time.Millisecond, 3 * time.Millisecond, 4 * time.Millisecond,
			5 * time.Millisecond, 6 * time.Millisecond, 7 * time.Millisecond, 8 * time.Millisecond,
			9 * time.Millisecond, 500 * time.Millisecond,
		}, 5, 500, 500},
	} {
		t.Run(tc.name, func(t *testing.T) {
			r, buf := newTestRecorder(t)
			for _, d := range tc.samples {
				r.Observe(d)
			}
			r.report()

			var rec map[string]any
			if err := json.Unmarshal(bytes.TrimSpace(buf.Bytes()), &rec); err != nil {
				t.Fatalf("parse log: %v", err)
			}
			for field, want := range map[string]float64{"p50_ms": tc.p50, "p95_ms": tc.p95, "p99_ms": tc.p99} {
				if got, _ := rec[field].(float64); got != want {
					t.Errorf("%s = %v, want %v", field, rec[field], want)
				}
			}
		})
	}
}

// TestPercentilesNeverExceedTheMax: nearest rank can only ever return an observed
// value, so no percentile may land above the window maximum.
func TestPercentilesNeverExceedTheMax(t *testing.T) {
	for n := 1; n <= 64; n++ {
		r, buf := newTestRecorder(t)
		for i := 1; i <= n; i++ {
			r.Observe(time.Duration(i) * time.Millisecond)
		}
		r.report()

		var rec map[string]any
		if err := json.Unmarshal(bytes.TrimSpace(buf.Bytes()), &rec); err != nil {
			t.Fatalf("n=%d parse log: %v", n, err)
		}
		max, _ := rec["max_ms"].(float64)
		for _, field := range []string{"p50_ms", "p95_ms", "p99_ms"} {
			got, _ := rec[field].(float64)
			if got > max {
				t.Fatalf("n=%d: %s = %v exceeds max_ms %v", n, field, got, max)
			}
			if got <= 0 {
				t.Fatalf("n=%d: %s = %v, want a real observation", n, field, got)
			}
		}
	}
}
