package gcs

import (
	"sync"
	"sync/atomic"
	"time"
)

// parkedLogEvery is how often Load logs, per room and instance, that it served a
// document with parked items.
const parkedLogEvery = time.Hour

// parkedLog rate limits Load's parked-items ERROR. A room is logged at most once
// per parkedLogEvery while its items stay parked, and is forgotten once a Load
// finds nothing parked, so a later recurrence logs at once.
type parkedLog struct {
	now func() time.Time // nil means time.Now

	mu        sync.Mutex
	rooms     map[string]*parkedRoom
	lastSweep time.Time

	// total counts every Load that served parked items, logged or not.
	total atomic.Uint64
}

type parkedRoom struct {
	logged     time.Time
	suppressed uint64
}

func (p *parkedLog) clock() time.Time {
	if p.now != nil {
		return p.now()
	}
	return time.Now()
}

// observe records a Load of room that served parked items. It reports whether
// to log, and how many such Loads went unlogged since the room's last line.
func (p *parkedLog) observe(room string) (bool, uint64) {
	p.total.Add(1)
	now := p.clock()
	p.mu.Lock()
	defer p.mu.Unlock()
	if p.rooms == nil {
		p.rooms = make(map[string]*parkedRoom)
	}
	p.sweep(now)
	r, ok := p.rooms[room]
	if !ok {
		p.rooms[room] = &parkedRoom{logged: now}
		return true, 0
	}
	if now.Sub(r.logged) < parkedLogEvery {
		r.suppressed++
		return false, 0
	}
	suppressed := r.suppressed
	r.logged, r.suppressed = now, 0
	return true, suppressed
}

// resolved forgets room after a Load found nothing parked in it.
func (p *parkedLog) resolved(room string) {
	p.mu.Lock()
	defer p.mu.Unlock()
	delete(p.rooms, room)
}

// sweep drops rooms logged a window ago with nothing suppressed since: their
// next parked Load logs anyway, so the entry only holds memory.
func (p *parkedLog) sweep(now time.Time) {
	if now.Sub(p.lastSweep) < parkedLogEvery {
		return
	}
	p.lastSweep = now
	for room, r := range p.rooms {
		if r.suppressed == 0 && now.Sub(r.logged) >= parkedLogEvery {
			delete(p.rooms, room)
		}
	}
}

// ParkedLoads returns how many Loads on this instance served a document with
// parked items, including those the rate limit kept out of the log.
func (a *Adapter) ParkedLoads() uint64 { return a.parkedLog.total.Load() }
