package gcs

import (
	"bufio"
	"bytes"
	"context"
	"encoding/json"
	"log/slog"
	"strings"
	"testing"
	"time"
)

// fakeClock is a settable clock for parkedLog.
type fakeClock struct{ t time.Time }

func (c *fakeClock) now() time.Time          { return c.t }
func (c *fakeClock) advance(d time.Duration) { c.t = c.t.Add(d) }

func TestParkedLog_LogsARoomOncePerWindow(t *testing.T) {
	t.Parallel()
	clk := &fakeClock{t: time.Unix(0, 0)}
	p := &parkedLog{now: clk.now}

	if ok, _ := p.observe("a"); !ok {
		t.Fatal("first parked Load not logged")
	}
	for i := 0; i < 3; i++ {
		clk.advance(10 * time.Minute)
		if ok, _ := p.observe("a"); ok {
			t.Fatalf("parked Load %d within the hour was logged", i+2)
		}
	}
	if ok, _ := p.observe("b"); !ok {
		t.Error("another room's first parked Load not logged")
	}
	clk.advance(parkedLogEvery)
	ok, suppressed := p.observe("a")
	if !ok || suppressed != 3 {
		t.Errorf("after the hour: logged=%v suppressed=%d, want true and 3", ok, suppressed)
	}
	if got := p.total.Load(); got != 6 {
		t.Errorf("total = %d, want every parked Load counted, 6", got)
	}
}

func TestParkedLog_LogsAtOnceAfterItsItemsResolved(t *testing.T) {
	t.Parallel()
	clk := &fakeClock{t: time.Unix(0, 0)}
	p := &parkedLog{now: clk.now}

	p.observe("a")
	p.resolved("a")
	clk.advance(time.Minute)
	if ok, suppressed := p.observe("a"); !ok || suppressed != 0 {
		t.Errorf("parked again after resolving: logged=%v suppressed=%d, want true and 0", ok, suppressed)
	}
}

func TestParkedLog_SweepKeepsOnlyRoomsWithUnloggedLoads(t *testing.T) {
	t.Parallel()
	clk := &fakeClock{t: time.Unix(0, 0)}
	p := &parkedLog{now: clk.now}

	p.observe("quiet")
	p.observe("busy")
	p.observe("busy")
	clk.advance(parkedLogEvery)
	p.observe("trigger") // sweeps
	if _, ok := p.rooms["quiet"]; ok {
		t.Error("a room with nothing unlogged was kept past the window")
	}
	if r, ok := p.rooms["busy"]; !ok || r.suppressed != 1 {
		t.Errorf("a room with an unlogged Load: kept=%v, want it kept with its count", ok)
	}
}

// parkedErrors returns the suppressed_loads value of each parked-items ERROR in
// buf's JSON log lines.
func parkedErrors(t *testing.T, buf *bytes.Buffer) []float64 {
	t.Helper()
	var got []float64
	sc := bufio.NewScanner(bytes.NewReader(buf.Bytes()))
	for sc.Scan() {
		var rec map[string]any
		if err := json.Unmarshal(sc.Bytes(), &rec); err != nil {
			t.Fatalf("log line %q: %v", sc.Text(), err)
		}
		msg, _ := rec["msg"].(string)
		if rec["level"] == "ERROR" && strings.Contains(msg, "parked items") {
			n, _ := rec["suppressed_loads"].(float64)
			got = append(got, n)
		}
	}
	return got
}

// TestLoad_RateLimitsTheParkedItemsError: a room served with parked items logs
// one ERROR per hour, the next line says how many Loads went unlogged, and once
// the missing edit arrives the room is forgotten.
func TestLoad_RateLimitsTheParkedItemsError(t *testing.T) {
	t.Parallel()
	ctx := context.Background()
	client, bucket := newFakeGCS(t)
	var buf bytes.Buffer
	log := slog.New(slog.NewJSONHandler(&buf, nil))
	a, err := New(Options{Client: client, Bucket: bucket, Locker: NewNoLock(), Logger: log})
	if err != nil {
		t.Fatal(err)
	}
	clk := &fakeClock{t: time.Unix(0, 0)}
	a.parkedLog.now = clk.now
	first, second := editPair(t)
	writeOldCheckpoint(t, a, "room", parkedState(t, second))

	for i := 0; i < 3; i++ {
		if _, parked := loaded(t, mustLoad(t, a), "t"); parked == 0 {
			t.Fatal("setup: Load served nothing parked")
		}
	}
	if got := parkedErrors(t, &buf); len(got) != 1 {
		t.Fatalf("3 Loads within the hour logged %d errors, want 1", len(got))
	}
	clk.advance(parkedLogEvery)
	mustLoad(t, a)
	if got := parkedErrors(t, &buf); len(got) != 2 || got[1] != 2 {
		t.Fatalf("after the hour: errors %v, want a second one reporting 2 unlogged Loads", got)
	}
	if got := a.ParkedLoads(); got != 4 {
		t.Errorf("ParkedLoads = %d, want 4", got)
	}

	if _, err := a.AppendUpdate(ctx, "room", first); err != nil {
		t.Fatal(err)
	}
	if _, parked := loaded(t, mustLoad(t, a), "t"); parked != 0 {
		t.Fatal("setup: the missing edit did not resolve the parked items")
	}
	a.parkedLog.mu.Lock()
	_, kept := a.parkedLog.rooms["room"]
	a.parkedLog.mu.Unlock()
	if kept {
		t.Error("the room was not forgotten after a Load found nothing parked")
	}
}
