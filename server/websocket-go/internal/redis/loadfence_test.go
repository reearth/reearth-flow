package redis

import (
	"context"
	"errors"
	"strconv"
	"sync"
	"testing"
	"time"
)

// fakeClock is a settable relay clock.
type fakeClock struct {
	mu  sync.Mutex
	cur time.Time
}

func (c *fakeClock) now() time.Time {
	c.mu.Lock()
	defer c.mu.Unlock()
	return c.cur
}

func (c *fakeClock) advance(d time.Duration) {
	c.mu.Lock()
	c.cur = c.cur.Add(d)
	c.mu.Unlock()
}

func clockedRelay(t *testing.T, addr string) (*Relay, *fakeClock) {
	t.Helper()
	r := mustRelay(t, addr)
	t.Cleanup(func() { _ = r.Close() })
	clk := &fakeClock{cur: time.Unix(1_800_000_000, 0)}
	r.now = clk.now
	return r, clk
}

func fenceField(t *testing.T, r *Relay, room string) (string, bool) {
	t.Helper()
	v, err := r.client.HGet(context.Background(), loadsKey(room), strconv.FormatUint(r.clientID, 10)).Result()
	if err != nil {
		return "", false
	}
	return v, true
}

func TestLoadFence_SetsAndClearsRedisField(t *testing.T) {
	t.Parallel()
	_, mr := newTestClient(t)
	r, clk := clockedRelay(t, mr.Addr())
	ctx := context.Background()
	const room = "fence-set"

	done := r.LoadStarting(ctx, room)
	v, ok := fenceField(t, r, room)
	if !ok {
		t.Fatal("a load did not write its fence")
	}
	// While the load reads storage its fence runs to the cap; once it has, the
	// fence lasts loadFenceTTL unless a pass refreshes it.
	if want := strconv.FormatInt(clk.now().Add(loadFenceMax).Unix(), 10); v != want {
		t.Errorf("fence deadline during the load %s, want %s", v, want)
	}
	done(nil)
	if v, _ := fenceField(t, r, room); v != strconv.FormatInt(clk.now().Add(loadFenceTTL).Unix(), 10) {
		t.Errorf("fence deadline after the load %s, want now + loadFenceTTL", v)
	}
	if !r.localFenceLive(room) {
		t.Fatal("a load that succeeded stopped fencing before any pass")
	}

	snap := r.passBegin(room)
	r.passEnd(ctx, room, snap)
	if r.localFenceLive(room) {
		t.Error("a completed pass did not clear the local fence")
	}
	if _, ok := fenceField(t, r, room); ok {
		t.Error("a completed pass did not clear the redis fence")
	}
}

func TestLoadFence_LoadErrorDropsFence(t *testing.T) {
	t.Parallel()
	_, mr := newTestClient(t)
	r, _ := clockedRelay(t, mr.Addr())
	const room = "fence-err"

	r.LoadStarting(context.Background(), room)(errors.New("gcs: load failed"))
	if r.localFenceLive(room) {
		t.Error("a failed load kept its local fence")
	}
	if _, ok := fenceField(t, r, room); ok {
		t.Error("a failed load kept its redis fence")
	}
}

func TestLoadFence_ExpiresWithoutPass(t *testing.T) {
	t.Parallel()
	c, mr := newTestClient(t)
	r, clk := clockedRelay(t, mr.Addr())
	ctx := context.Background()
	const room = "fence-expire"

	r.LoadStarting(ctx, room)(nil)
	clk.advance(loadFenceTTL - time.Second)
	if !r.localFenceLive(room) {
		t.Fatal("the fence expired early")
	}
	if fenced, err := loadFenced(ctx, c, room, clk.now().Unix()); err != nil || !fenced {
		t.Fatalf("redis fence before its deadline: fenced=%v err=%v", fenced, err)
	}
	clk.advance(2 * time.Second)
	if r.localFenceLive(room) {
		t.Error("a load whose room never activated kept fencing past its deadline")
	}
	if fenced, err := loadFenced(ctx, c, room, clk.now().Unix()); err != nil || fenced {
		t.Errorf("redis fence past its deadline: fenced=%v err=%v", fenced, err)
	}
}

func TestLoadFence_RefreshStopsAtCap(t *testing.T) {
	t.Parallel()
	c, mr := newTestClient(t)
	r, clk := clockedRelay(t, mr.Addr())
	ctx := context.Background()
	const room = "fence-cap"

	r.LoadStarting(ctx, room)(nil)
	for elapsed := time.Minute; elapsed <= 9*time.Minute; elapsed += time.Minute {
		clk.advance(time.Minute)
		r.passRefresh(ctx, room)
	}
	if !r.localFenceLive(room) {
		t.Fatal("a refreshed fence lapsed during a long pass")
	}
	if fenced, err := loadFenced(ctx, c, room, clk.now().Unix()); err != nil || !fenced {
		t.Fatalf("refreshed redis fence: fenced=%v err=%v", fenced, err)
	}
	clk.advance(time.Minute + time.Second) // past start + loadFenceMax
	r.passRefresh(ctx, room)
	if r.localFenceLive(room) {
		t.Error("a pass that never ends kept the fence past its cap")
	}
	if fenced, err := loadFenced(ctx, c, room, clk.now().Unix()); err != nil || fenced {
		t.Errorf("redis fence past its cap: fenced=%v err=%v", fenced, err)
	}
}

func TestLoadFence_PassCoversOnlyEarlierLoads(t *testing.T) {
	t.Parallel()
	_, mr := newTestClient(t)
	r, _ := clockedRelay(t, mr.Addr())
	ctx := context.Background()
	const room = "fence-cover"

	r.LoadStarting(ctx, room)(nil)
	first := r.passBegin(room)
	r.LoadStarting(ctx, room)(nil) // starts after the pass began
	r.passEnd(ctx, room, first)
	if !r.localFenceLive(room) {
		t.Fatal("a pass cleared a load that started after it began")
	}
	if _, ok := fenceField(t, r, room); !ok {
		t.Fatal("the redis fence was cleared while a load was still pending")
	}
	second := r.passBegin(room)
	r.passEnd(ctx, room, second)
	if r.localFenceLive(room) {
		t.Error("a pass that began after both loads left a fence")
	}
	if got := r.loadGen(room); got != 2 {
		t.Errorf("loadGen = %d, want 2", got)
	}
}

func TestLoadFence_RedisDownStillFencesLocally(t *testing.T) {
	t.Parallel()
	_, mr := newTestClient(t)
	r, _ := clockedRelay(t, mr.Addr())
	const room = "fence-down"
	mr.Close()

	start := time.Now()
	done := r.LoadStarting(context.Background(), room)
	if took := time.Since(start); took > loadFenceWriteTimeout+time.Second {
		t.Errorf("a fence write to a dead redis held the load %v", took)
	}
	done(nil)
	if !r.localFenceLive(room) {
		t.Error("a load with no redis lost its local fence")
	}
}

// TestLoadFence_LongLoadStaysFenced: a load that takes longer than the TTL is
// still reading storage, and nothing has covered it yet.
func TestLoadFence_LongLoadStaysFenced(t *testing.T) {
	t.Parallel()
	c, mr := newTestClient(t)
	r, clk := clockedRelay(t, mr.Addr())
	ctx := context.Background()
	const room = "fence-long-load"

	done := r.LoadStarting(ctx, room)
	clk.advance(3 * time.Minute)
	if !r.localFenceLive(room) {
		t.Fatal("a load still in progress lost its local fence")
	}
	if fenced, err := loadFenced(ctx, c, room, clk.now().Unix()); err != nil || !fenced {
		t.Fatalf("a load still in progress lost its redis fence: fenced=%v err=%v", fenced, err)
	}
	done(nil)
	clk.advance(time.Minute)
	if !r.localFenceLive(room) {
		t.Fatal("a load that just finished is not fenced until its pass")
	}
	clk.advance(loadFenceMax)
	if r.localFenceLive(room) {
		t.Error("a load's fence outlived its cap")
	}
}

// TestLoadFence_RunningPassKeepsALaterLoadFenced: a replacement's replay waits
// behind a pass the reader is already running; that pass must keep the newer
// load fenced, though it does not clear it.
func TestLoadFence_RunningPassKeepsALaterLoadFenced(t *testing.T) {
	t.Parallel()
	_, mr := newTestClient(t)
	r, clk := clockedRelay(t, mr.Addr())
	ctx := context.Background()
	const room = "fence-queued"

	r.LoadStarting(ctx, room)(nil)
	snap := r.passBegin(room)
	r.LoadStarting(ctx, room)(nil)
	for i := 0; i < 5; i++ {
		clk.advance(time.Minute)
		r.passRefresh(ctx, room)
	}
	r.passEnd(ctx, room, snap)
	if !r.localFenceLive(room) {
		t.Error("a load waiting behind a long pass lost its fence")
	}
}

// TestLoadFence_DropRacingALoadKeepsItsFence: a room's fence state can be
// dropped (its relay state torn down) just as a load of it begins.
func TestLoadFence_DropRacingALoadKeepsItsFence(t *testing.T) {
	t.Parallel()
	_, mr := newTestClient(t)
	r, _ := clockedRelay(t, mr.Addr())
	ctx := context.Background()
	const room = "fence-drop-race"
	r.LoadStarting(ctx, room)(errors.New("prime the fence state"))

	var once sync.Once
	r.beforeFenceIO = func() {
		once.Do(func() {
			r.mu.Lock()
			r.dropFenceIfIdleLocked(room)
			r.mu.Unlock()
		})
	}
	r.LoadStarting(ctx, room)(nil)
	if !r.localFenceLive(room) {
		t.Error("a load whose fence state was dropped as it began is not fenced")
	}
	if _, ok := fenceField(t, r, room); !ok {
		t.Error("a load whose fence state was dropped as it began has no redis fence")
	}
}
