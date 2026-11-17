package versions

import (
	"context"
	"log/slog"
	"sync"
	"time"
)

// DefaultMarkEvery is how often a busy room's edits reach the Store.
const DefaultMarkEvery = time.Second

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
	batch   Mark
	editors map[string]struct{}
	pending bool
	// writing is closed when the write in progress ends; nil when none is.
	writing chan struct{}
	// timer, while set, holds the room's next write back until the interval
	// since the last one has passed. seq tells a fired timer whether it is
	// still the current one.
	timer *time.Timer
	seq   uint64
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

// Edit records that user changed room's document. user is empty when the
// connection carries no identity. It does not wait for the Store.
func (t *Tracker) Edit(room, user string) {
	now := t.now()
	t.mu.Lock()
	defer t.mu.Unlock()
	re := t.rooms[room]
	if re == nil {
		re = &roomEdits{editors: map[string]struct{}{}}
		t.rooms[room] = re
	}
	re.add(user, now)
	if re.writing == nil && re.timer == nil {
		batch := re.take()
		re.writing = make(chan struct{})
		go func() { _ = t.write(room, re, batch) }()
	}
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
		if !re.pending {
			t.mu.Unlock()
			return nil
		}
		if re.timer != nil {
			re.timer.Stop()
			re.timer = nil
		}
		batch := re.take()
		re.writing = make(chan struct{})
		t.mu.Unlock()
		return t.write(room, re, batch)
	}
}

// write sends batch to the Store. On failure the batch rejoins the room's
// edits for the next write. Either way the next write waits out the interval.
func (t *Tracker) write(room string, re *roomEdits, batch Mark) error {
	ctx, cancel := context.WithTimeout(context.Background(), markTimeout)
	err := t.store.Mark(ctx, room, batch)
	cancel()

	t.mu.Lock()
	defer t.mu.Unlock()
	if err != nil {
		t.log.Warn("recording a document change failed; retrying", "room", room, "err", err)
		re.restore(batch)
	}
	close(re.writing)
	re.writing = nil
	re.seq++
	seq := re.seq
	re.timer = time.AfterFunc(t.every, func() { t.tick(room, re, seq) })
	return err
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
	if !re.pending {
		delete(t.rooms, room)
		return
	}
	batch := re.take()
	re.writing = make(chan struct{})
	go func() { _ = t.write(room, re, batch) }()
}

func (re *roomEdits) add(user string, at time.Time) {
	if !re.pending {
		re.batch.FirstAt = at
	}
	re.pending = true
	re.batch.LastAt = at
	if user != "" {
		re.editors[user] = struct{}{}
		re.batch.LastEditor = user
		re.batch.LastEditorAt = at
	}
}

// take returns the gathered edits and starts a new batch.
func (re *roomEdits) take() Mark {
	b := re.batch
	b.Editors = make([]string, 0, len(re.editors))
	for u := range re.editors {
		b.Editors = append(b.Editors, u)
	}
	re.batch = Mark{}
	re.editors = map[string]struct{}{}
	re.pending = false
	return b
}

// restore puts back a batch whose write failed, merged with edits gathered
// since.
func (re *roomEdits) restore(b Mark) {
	for _, u := range b.Editors {
		re.editors[u] = struct{}{}
	}
	if !re.pending || b.FirstAt.Before(re.batch.FirstAt) {
		re.batch.FirstAt = b.FirstAt
	}
	if b.LastAt.After(re.batch.LastAt) {
		re.batch.LastAt = b.LastAt
	}
	if b.LastEditor != "" && (re.batch.LastEditor == "" || b.LastEditorAt.After(re.batch.LastEditorAt)) {
		re.batch.LastEditor = b.LastEditor
		re.batch.LastEditorAt = b.LastEditorAt
	}
	re.pending = true
}
