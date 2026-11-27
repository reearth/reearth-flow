package server

import (
	"context"
	"runtime"
	"strings"
	"sync/atomic"
	"testing"
	"time"

	"github.com/reearth/ygo/cluster"
	"github.com/reearth/ygo/crdt"
	ygsync "github.com/reearth/ygo/sync"

	"github.com/reearth/reearth-flow/websocket-go/internal/pg"
	redisrelay "github.com/reearth/reearth-flow/websocket-go/internal/redis"
)

// Both production relays report lost updates: AttachRedisRelay finds this by
// assertion.
var (
	_ gapTaker = (*redisrelay.Relay)(nil)
	_ gapTaker = (*pg.Relay)(nil)
)

// lossyRelay is the real Redis relay with chosen outbound sync updates dropped,
// so the other instances miss an edit the stream never had.
type lossyRelay struct {
	*redisrelay.Relay
	dropOut atomic.Int32 // outbound sync updates still to drop
	gap     atomic.Bool  // a lost update to report, as the relay reports its own
}

func lossy(out **lossyRelay) func(*redisrelay.Relay) cluster.Relay {
	return func(r *redisrelay.Relay) cluster.Relay {
		*out = &lossyRelay{Relay: r}
		return *out
	}
}

// takeOne consumes one of n, reporting whether there was one.
func takeOne(n *atomic.Int32) bool {
	for {
		v := n.Load()
		if v <= 0 {
			return false
		}
		if n.CompareAndSwap(v, v-1) {
			return true
		}
	}
}

func (l *lossyRelay) Publish(ctx context.Context, out cluster.Outbound) error {
	if out.Kind == cluster.KindSync && takeOne(&l.dropOut) {
		return nil
	}
	return l.Relay.Publish(ctx, out)
}

func (l *lossyRelay) TakeOutboundGap(room string) bool {
	return l.gap.Swap(false) || l.Relay.TakeOutboundGap(room)
}

func (p *lcInstance) parked() bool {
	doc := p.s.ws.GetDoc(lifecycleRoom)
	return doc != nil && isParked(doc)
}

// heal runs one heal pass, as the periodic sync does, and waits for it.
func (p *lcInstance) heal(t *testing.T) {
	t.Helper()
	ctx, cancel := context.WithTimeout(context.Background(), 20*time.Second)
	defer cancel()
	p.s.healRooms(ctx)
	p.s.heals.waitIdle(ctx)
	if ctx.Err() != nil {
		t.Fatal("a heal never finished")
	}
}

// countLoads counts the stored-state reads s makes from now on.
func countLoads(s *Server) *atomic.Int32 {
	var n atomic.Int32
	inner := s.stored
	s.stored = func(ctx context.Context, room string) ([]byte, error) {
		n.Add(1)
		return inner(ctx, room)
	}
	return &n
}

// storedText is the room's text as storage holds it.
func storedText(t *testing.T, s *Server) string {
	t.Helper()
	state, err := s.stored(context.Background(), lifecycleRoom)
	if err != nil {
		t.Fatal(err)
	}
	doc := crdt.New()
	if len(state) > 0 {
		if err := crdt.ApplyUpdateV1(doc, state, nil); err != nil {
			t.Fatal(err)
		}
	}
	return doc.GetText("content").ToString()
}

// missedEdit is two instances where B's relay never wrote x, so only storage
// has it, and a later edit that builds on x parks on A and in alice's tab.
type missedEdit struct {
	A, B  *lcInstance
	alice *lcTab
	loads *atomic.Int32
}

func missEdit(t *testing.T) missedEdit {
	t.Helper()
	e := newLCEnv(t)
	var lr *lossyRelay
	A := e.instance("A", testConfig())
	B := e.instanceWith("B", testConfig(), lossy(&lr))
	alice := openLCTab(t, A, "alice", 101)
	bob := openLCTab(t, B, "bob", 202)
	t.Cleanup(func() { runtime.KeepAlive(bob) })
	bob.insertText("b1 ")
	waitFor(t, "b1 on A", func() bool { return strings.Contains(A.text(), "b1 ") })

	lr.dropOut.Store(1)
	bob.insertText("x")
	waitFor(t, "B's relay to drop x", func() bool { return lr.dropOut.Load() == 0 })
	waitFor(t, "x in storage", func() bool { return strings.Contains(storedText(t, A.s), "x") })
	bob.insertText("y")
	waitFor(t, "y to park on A", A.parked)
	if strings.Contains(A.text(), "x") {
		t.Fatal("A's copy has x; the test exercises nothing")
	}
	return missedEdit{A: A, B: B, alice: alice, loads: countLoads(A.s)}
}

// healed checks that A's copy and alice's tab match B's copy.
func (m missedEdit) healed(t *testing.T) {
	t.Helper()
	if m.A.parked() {
		t.Fatal("A's copy is still parked after the heal")
	}
	want := m.B.text()
	if got := m.A.text(); got != want {
		t.Fatalf("A's copy = %q, want %q", got, want)
	}
	if !waitForText(t, m.alice.e2eClient, "content", want) {
		t.Fatalf("alice's tab = %q, want %q", m.alice.doc.GetText("content").ToString(), want)
	}
}

// TestHeal_FoldsAnEditTheStreamNeverHad: the stored state heals A's copy and
// reaches alice's tab, with one storage read.
func TestHeal_FoldsAnEditTheStreamNeverHad(t *testing.T) {
	t.Parallel()
	m := missEdit(t)
	m.A.heal(t)
	m.healed(t)
	if n := m.loads.Load(); n != 1 {
		t.Fatalf("the heal read storage %d times, want 1", n)
	}
}

// TestPeriodicSync_HealsAParkedCopy: the periodic sync heals, without a call
// from anywhere else.
func TestPeriodicSync_HealsAParkedCopy(t *testing.T) {
	t.Parallel()
	m := missEdit(t)
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	go m.A.s.StartPeriodicSync(ctx, 50*time.Millisecond)
	if !within(10*time.Second, func() bool { return !m.A.parked() }) {
		t.Fatal("the periodic sync never healed A's copy")
	}
	m.healed(t)
}

// TestHeal_BacksOffWhileTheMissingEditExistsNowhere: carol's tab sends an edit
// that builds on one it never sent, so neither the relay nor storage can heal
// the copy. Heals back off from 30 s to 15 min and start afresh once healed.
func TestHeal_BacksOffWhileTheMissingEditExistsNowhere(t *testing.T) {
	t.Parallel()
	e := newLCEnv(t)
	A := e.instance("A", testConfig())
	var clock atomic.Int64
	clock.Store(time.Date(2026, 11, 26, 0, 0, 0, 0, time.UTC).UnixNano())
	A.s.heals.now = func() time.Time { return time.Unix(0, clock.Load()) }
	advance := func(d time.Duration) { clock.Add(int64(d)) }
	loads := countLoads(A.s)

	carol := openLCTab(t, A, "carol", 303)
	defer runtime.KeepAlive(carol)
	carol.insertText("c1 ")
	waitFor(t, "c1 on A", func() bool { return strings.Contains(A.text(), "c1 ") })
	txt := carol.doc.GetText("content")
	carol.doc.Transact(func(txn *crdt.Transaction) { txt.Insert(txn, txt.Len(), "m", nil) }) // never sent
	carol.insertText("n")
	waitFor(t, "n to park on A", A.parked)

	expect := func(when string, want int32) {
		t.Helper()
		A.heal(t)
		if n := loads.Load(); n != want {
			t.Fatalf("%s: %d storage reads, want %d", when, n, want)
		}
	}
	expect("first parked tick", 1)
	expect("same instant", 1)
	expect("same instant again", 1)
	for _, retry := range []time.Duration{
		30 * time.Second, time.Minute, 2 * time.Minute, 4 * time.Minute, 8 * time.Minute,
		15 * time.Minute, 15 * time.Minute,
	} {
		before := loads.Load()
		advance(retry - time.Second)
		expect("a second before the retry "+retry.String(), before)
		advance(time.Second)
		expect("at the retry "+retry.String(), before+1)
	}
	if !A.parked() {
		t.Fatal("the copy healed although its missing edit exists nowhere")
	}

	// The tab reconnects in effect and sends what it holds, m included.
	carol.writeSync(ygsync.EncodeUpdate(crdt.EncodeStateAsUpdateV1(carol.doc, nil)))
	waitFor(t, "m to unpark n on A", func() bool { return !A.parked() })
	before := loads.Load()
	expect("after the copy healed", before)
	A.s.heals.mu.Lock()
	_, tracked := A.s.heals.rooms[lifecycleRoom]
	A.s.heals.mu.Unlock()
	if tracked {
		t.Fatal("a healed room keeps its backoff")
	}
}

// TestHeal_RepublishesARoomTheRelayLostAnUpdateFor: B's relay lost x and nothing
// builds on it, so nothing parks on A. B republishes its full state and A and
// alice's tab get x.
func TestHeal_RepublishesARoomTheRelayLostAnUpdateFor(t *testing.T) {
	t.Parallel()
	e := newLCEnv(t)
	var lr *lossyRelay
	A := e.instance("A", testConfig())
	B := e.instanceWith("B", testConfig(), lossy(&lr))
	alice := openLCTab(t, A, "alice", 101)
	defer runtime.KeepAlive(alice)
	bob := openLCTab(t, B, "bob", 202)
	defer runtime.KeepAlive(bob)
	bob.insertText("b1 ")
	waitFor(t, "b1 on A", func() bool { return strings.Contains(A.text(), "b1 ") })

	lr.dropOut.Store(1)
	lr.gap.Store(true)
	bob.insertText("x")
	waitFor(t, "B's relay to drop x", func() bool { return lr.dropOut.Load() == 0 })
	if within(300*time.Millisecond, func() bool { return strings.Contains(A.text(), "x") }) {
		t.Fatal("A got x although B's relay dropped it; the test exercises nothing")
	}

	B.heal(t)
	want := B.text()
	if !within(3*time.Second, func() bool { return A.text() == want }) {
		t.Fatalf("A's copy = %q, want %q", A.text(), want)
	}
	if !waitForText(t, alice.e2eClient, "content", want) {
		t.Fatalf("alice's tab = %q, want %q", alice.doc.GetText("content").ToString(), want)
	}
}
