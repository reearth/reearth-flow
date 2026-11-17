package versions

import (
	"context"
	"log/slog"
	"sync"
	"time"

	"github.com/reearth/ygo/crdt"
)

// DefaultMarkEvery is how often a busy room's edits reach the Store.
const DefaultMarkEvery = time.Second

// maxMarkBackoff bounds how long a room waits between writes while the Store
// keeps failing.
const maxMarkBackoff = 30 * time.Second

// markTimeout bounds one write to the Store.
const markTimeout = 5 * time.Second

// Tracker reports the edits an instance receives to the Store without a write
// per edit: edits arrive many times a second. A room's first edit after a quiet
// spell is written at once, so a version requested right after it finds the
// room changed; edits after that are gathered and written once per interval.
type Tracker struct {
	store Store
	every time.Duration
	log   *slog.Logger
	now   func() time.Time

	mu    sync.Mutex
	rooms map[string]*roomEdits
}

// roomEdits are one room's edits not yet in the Store.
type roomEdits struct {
	batch Record
	// writing is closed when the write in progress ends; nil when none is.
	writing chan struct{}
	// timer, while set, holds the room's next write back until the interval
	// since the last one has passed. seq tells a fired timer whether it is
	// still the current one.
	timer *time.Timer
	seq   uint64
	// failures counts consecutive failed writes, which lengthen the wait.
	failures int
}

// NewTracker returns a Tracker writing to store at most once per every per
// room (DefaultMarkEvery when every <= 0).
func NewTracker(store Store, every time.Duration, log *slog.Logger) *Tracker {
	if every <= 0 {
		every = DefaultMarkEvery
	}
	if log == nil {
		log = slog.Default()
	}
	return &Tracker{store: store, every: every, log: log, now: time.Now, rooms: map[string]*roomEdits{}}
}

// Edit records that user's update, which touched ids, changed room's
// document. user is empty when the connection carries no identity. It does
// not wait for the Store.
func (t *Tracker) Edit(room, user string, ids crdt.ContentIDs) {
	t.mu.Lock()
	defer t.mu.Unlock()
	// Read the clock under the lock, so edits enter a batch in time order.
	now := t.now()
	re := t.rooms[room]
	if re == nil {
		re = &roomEdits{batch: Record{}}
		t.rooms[room] = re
	}
	e, ok := re.batch[user]
	if !ok {
		e = NewEdits()
	}
	e.add(ids, now)
	re.batch[user] = e
	if re.writing == nil && re.timer == nil {
		t.startWrite(room, re)
	}
}

// startWrite sends the room's batch to the Store on its own goroutine. The
// caller holds t.mu.
func (t *Tracker) startWrite(room string, re *roomEdits) {
	batch := re.batch
	re.batch = Record{}
	re.writing = make(chan struct{})
	go func() { _ = t.write(room, re, batch) }()
}

// Flush writes room's gathered edits now and waits for them, and for a write
// already in progress, to reach the Store.
func (t *Tracker) Flush(ctx context.Context, room string) error {
	for {
		t.mu.Lock()
		re := t.rooms[room]
		if re == nil {
			t.mu.Unlock()
			return nil
		}
		if w := re.writing; w != nil {
			t.mu.Unlock()
			select {
			case <-w:
				continue
			case <-ctx.Done():
				return ctx.Err()
			}
		}
		if len(re.batch) == 0 {
			t.mu.Unlock()
			return nil
		}
		if re.timer != nil {
			re.timer.Stop()
			re.timer = nil
		}
		batch := re.batch
		re.batch = Record{}
		re.writing = make(chan struct{})
		t.mu.Unlock()
		return t.write(room, re, batch)
	}
}

// write sends batch to the Store. On failure the batch rejoins the room's
// edits for the next write, which waits longer after each consecutive failure.
func (t *Tracker) write(room string, re *roomEdits, batch Record) error {
	ctx, cancel := context.WithTimeout(context.Background(), markTimeout)
	err := t.store.Mark(ctx, room, batch)
	cancel()

	t.mu.Lock()
	defer t.mu.Unlock()
	wait := t.every
	if err != nil {
		re.failures++
		if re.failures == 1 {
			t.log.Warn("recording a document change failed; retrying", "room", room, "err", err)
		} else {
			t.log.Debug("recording a document change failed again", "room", room, "failures", re.failures, "err", err)
		}
		batch.Merge(re.batch)
		re.batch = batch
		wait = t.backoff(re.failures)
	} else {
		re.failures = 0
	}
	close(re.writing)
	re.writing = nil
	re.seq++
	seq := re.seq
	re.timer = time.AfterFunc(wait, func() { t.tick(room, re, seq) })
	return err
}

// backoff doubles the interval for each consecutive failure, up to
// maxMarkBackoff.
func (t *Tracker) backoff(failures int) time.Duration {
	d := t.every
	for i := 1; i < failures && d < maxMarkBackoff; i++ {
		d *= 2
	}
	if d > maxMarkBackoff {
		d = maxMarkBackoff
	}
	return d
}

// tick ends a room's interval: it writes the edits gathered during it, or
// forgets a room that had none.
func (t *Tracker) tick(room string, re *roomEdits, seq uint64) {
	t.mu.Lock()
	defer t.mu.Unlock()
	if t.rooms[room] != re || re.seq != seq || re.timer == nil {
		return // superseded by a Flush
	}
	re.timer = nil
	if re.writing != nil {
		return // the write in progress sets the next timer
	}
	if len(re.batch) == 0 {
		delete(t.rooms, room)
		return
	}
	t.startWrite(room, re)
}
