// Package pg implements ygo's cluster.Relay over Postgres unlogged tables, as a
// drop-in alternative to internal/redis.
//
// Deliberately a mirror of internal/redis/relay.go — same roomState, same bounded
// write queue, same three per-room goroutines — so the two diff cleanly and the
// four bugs already fixed there stay fixed here. Each is marked below.
//
// NOT wire-compatible with the Rust server: a Postgres-backed Go instance and a
// Redis-backed Rust instance serving the same document cannot see each other and
// will silently fork it. Only enable where no Rust instance is live.
package pg

import (
	"context"
	"errors"
	"log/slog"
	"sync"
	"sync/atomic"
	"time"

	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgxpool"
	"github.com/reearth/ygo/cluster"

	"github.com/reearth/reearth-flow/websocket-go/internal/latency"
)

// ErrRelayClosed is returned by Publish after Close.
var ErrRelayClosed = errors.New("pg: relay closed")

// Read bounds. readLimit matches the Redis reader's XREAD COUNT so one iteration
// holds a comparable number of entries in memory.
const readLimit = 512

// Write-queue bounds, identical to the Redis relay: Publish enqueues onto a
// bounded per-room channel and returns immediately, and a per-room writer drains
// it in batches. On overflow the enqueue drops and counts, which a later flush plus
// a reconnect catch-up recovers.
const (
	writeQueueCap  = 1024
	syncBatch      = 100
	awarenessBatch = 50
)

// evictTimeout bounds the last-instance durability I/O (heartbeat removal, active
// re-check, GCS flush, row delete).
const evictTimeout = 30 * time.Second

// activeTimeout is how long since a heartbeat an instance still counts as live. It
// mirrors the Redis relay's activeTimeoutSecs and is deliberately shorter than the
// janitor's reap window, so liveness is decided by seen_at and never by whether a
// row has been reaped yet.
const activeTimeout = 60 * time.Second

// heartbeatRefresh is how often each room re-stamps seen_at.
const heartbeatRefresh = 30 * time.Second

// Flusher is the relay's last-instance persistence seam. Mirrors redis.Flusher.
type Flusher interface {
	// FlushRoom persists the room's converged document to GCS.
	//
	// CONTRACT: FlushRoom MUST hold the room's read lock for its entire critical
	// section (it fences the row DELETE the relay runs immediately after) and MUST
	// NOT delete the rows itself. On error the relay skips the delete so the
	// un-persisted rows survive for a reconnect to replay.
	FlushRoom(ctx context.Context, room string) error
}

type noopFlusher struct{}

func (noopFlusher) FlushRoom(context.Context, string) error { return nil }

// Options configures a Relay.
type Options struct {
	// Q is the database handle, normally a *pgxpool.Pool.
	Q Querier
	// Logger MUST NEVER receive row payloads or secrets — only room id and error
	// class. Defaults to slog.Default().
	Logger *slog.Logger
	// Flusher is the GCS persistence seam; defaults to a no-op.
	Flusher Flusher
	// PollEvery is the reader's poll period. Zero disables polling entirely, which
	// is what the notify-only fan-out wants.
	PollEvery time.Duration
	// Notify holds a LISTEN connection and wakes readers on commit. Requires Q to
	// be a *pgxpool.Pool (it needs a dedicated connection); New rejects the
	// combination otherwise rather than silently never delivering.
	Notify bool
	// ReadLag makes the reader ignore rows younger than this. Zero (the default)
	// is correct: serializeInsertSQL already guarantees a document's ids commit in
	// allocation order, so there is no timing window to cover. It exists only as an
	// escape hatch for rows that could reach ws_stream without taking the insert
	// lock, and any non-zero value is added directly to delivery latency.
	ReadLag time.Duration
}

// Relay implements cluster.Relay over ws_stream / ws_instance.
type Relay struct {
	q        Querier
	clientID int64
	log      *slog.Logger
	flusher  Flusher
	poll     time.Duration
	readLag  time.Duration
	pool     *pgxpool.Pool // non-nil only when notify is enabled
	latency  *latency.Recorder

	// droppedWrites counts outbound updates dropped on a full write queue.
	droppedWrites atomic.Uint64

	// bgWG tracks the relay-scoped goroutines (LISTEN, latency reporting) and
	// bgCancel stops them. Close must be able to stop them ITSELF: waiting on
	// goroutines that only exit with the Start context deadlocks any caller that
	// closes before cancelling, which the cluster.Relay contract permits.
	bgWG     sync.WaitGroup
	bgCancel context.CancelFunc

	mu     sync.Mutex
	sink   cluster.Sink
	ctx    context.Context
	rooms  map[string]*roomState
	closed bool
}

type roomState struct {
	cancel context.CancelFunc
	wg     sync.WaitGroup

	// writeCh is the bounded outbound queue; Publish enqueues non-blockingly.
	writeCh chan writeItem

	// evicting refuses Publish so no INSERT races the DELETE.
	evicting bool

	// cursor is the per-reader position, advanced past self-filtered rows too.
	cursor int64

	// wake nudges the reader out of its wait when a notification arrives. Capacity
	// 1 and sent non-blockingly, so wakeups coalesce rather than queue.
	wake chan struct{}
}

type writeItem struct {
	kind cluster.Kind
	data []byte
}

var _ cluster.Relay = (*Relay)(nil)

// New builds a Relay and draws the per-process client id. It does NOT apply the
// schema — call Migrate first.
func New(opts Options) (*Relay, error) {
	if opts.Q == nil {
		return nil, errors.New("pg: nil Querier")
	}
	id, err := newInstanceID()
	if err != nil {
		return nil, err
	}
	log := opts.Logger
	if log == nil {
		log = slog.Default()
	}
	fl := opts.Flusher
	if fl == nil {
		fl = noopFlusher{}
	}
	// A relay with neither a poll interval nor notify would activate rooms and
	// never read them: documents would look healthy and silently stop syncing.
	if opts.PollEvery <= 0 && !opts.Notify {
		return nil, errors.New("pg: relay needs PollEvery, Notify, or both")
	}

	var pool *pgxpool.Pool
	if opts.Notify {
		p, ok := opts.Q.(*pgxpool.Pool)
		if !ok {
			return nil, errors.New("pg: Notify requires Q to be a *pgxpool.Pool (LISTEN needs a dedicated connection)")
		}
		pool = p
	}

	return &Relay{
		q:        opts.Q,
		clientID: id,
		log:      log,
		flusher:  fl,
		poll:     opts.PollEvery,
		readLag:  opts.ReadLag,
		pool:     pool,
		latency:  latency.New(log, backendLabel(opts)),
		rooms:    make(map[string]*roomState),
	}, nil
}

// backendLabel names the arm in the latency log. The fan-out is part of the name
// because poll and notify are the two things being compared: a report that said
// only "postgres" could not be attributed to either.
func backendLabel(opts Options) string {
	switch {
	case opts.Notify && opts.PollEvery > 0:
		return "postgres/hybrid"
	case opts.Notify:
		return "postgres/notify"
	default:
		return "postgres/poll"
	}
}

// lockKeyFor is the read-lock key for a room. It must match the key the flusher
// passes to Locker.TryWithLock, or the election would fence against nothing.
func lockKeyFor(room string) string { return "read:lock:" + room }

// Start binds the Sink and records the delivery context. Per-room delivery begins
// at RoomActivated. Cancelling ctx (or calling Close) stops the relay.
func (r *Relay) Start(ctx context.Context, sink cluster.Sink) error {
	r.mu.Lock()
	defer r.mu.Unlock()
	if r.closed {
		return ErrRelayClosed
	}
	r.sink = sink
	r.ctx = ctx

	// Derived so either cancelling ctx or calling Close stops these.
	bgCtx, cancel := context.WithCancel(ctx)
	r.bgCancel = cancel

	// One listener per relay, started here rather than per room: LISTEN is
	// connection-scoped but the channel is shared, so rooms multiplex over it.
	if r.pool != nil {
		r.bgWG.Go(func() { newListener(r.pool, r.log, r.wakeRoom).run(bgCtx) })
	}
	r.bgWG.Go(func() { r.latency.Run(bgCtx) })
	return nil
}

// Publish enqueues one update onto the room's bounded write queue and returns
// immediately. It must NOT insert inline: that would block the publishing
// goroutine on database latency. Silent no-op while the room is evicting; a full
// queue drops and counts. Returns ErrRelayClosed after Close.
func (r *Relay) Publish(_ context.Context, out cluster.Outbound) error {
	r.mu.Lock()
	if r.closed {
		r.mu.Unlock()
		return ErrRelayClosed
	}
	rs, ok := r.rooms[out.Room]
	if !ok || rs.evicting {
		r.mu.Unlock()
		return nil
	}
	ch := rs.writeCh
	r.mu.Unlock()

	// Copy the payload: the caller's slice may alias internal buffers, and the
	// INSERT happens later on the writer goroutine.
	item := writeItem{kind: out.Kind, data: append([]byte(nil), out.Data...)}
	select {
	case ch <- item:
		return nil
	default:
		r.droppedWrites.Add(1)
		r.log.Debug("relay write queue full, dropping update", "room", out.Room, "kind", out.Kind.String())
		return nil
	}
}

// DroppedWrites returns the cumulative count of updates dropped on a full queue.
func (r *Relay) DroppedWrites() uint64 { return r.droppedWrites.Load() }

// RoomActivated starts the room's reader, heartbeat and writer, and registers this
// instance as live. Idempotent.
//
// Catch-up replay is performed by the reader goroutine, NOT inline here: ygo invokes
// RoomActivated while holding its rooms lock, so injecting during the callback
// re-enters the Server (Sink.Inject -> getOrCreateRoom) and deadlocks on that
// non-reentrant lock (ygo#133). The heartbeat is registered before the reader starts
// so a concurrent evictor sees this node active.
func (r *Relay) RoomActivated(room string) {
	r.mu.Lock()
	if r.closed || r.sink == nil {
		r.mu.Unlock()
		return
	}
	if rs, ok := r.rooms[room]; ok && !rs.evicting {
		r.mu.Unlock()
		return // already active
	}
	ctx, cancel := context.WithCancel(r.ctx)
	rs := &roomState{
		cancel:  cancel,
		writeCh: make(chan writeItem, writeQueueCap),
		wake:    make(chan struct{}, 1),
	}
	r.rooms[room] = rs
	sink := r.sink
	r.mu.Unlock()

	if err := r.heartbeat(ctx, room); err != nil {
		r.log.Debug("relay heartbeat init failed", "room", room, "err", err)
	}

	rs.wg.Add(3)
	go r.readLoop(ctx, room, rs, sink)
	go r.heartbeatLoop(ctx, room, rs)
	go r.writeLoop(ctx, room, rs)
}

// RoomDeactivated stops the room's goroutines and runs the locked evict path: if
// this is the last active instance, flush to GCS then safe-delete the rows.
// Idempotent.
func (r *Relay) RoomDeactivated(room string) {
	r.mu.Lock()
	rs, ok := r.rooms[room]
	if !ok || r.closed {
		r.mu.Unlock()
		return
	}
	// Refuse Publish for the whole critical section so no INSERT races the DELETE.
	rs.evicting = true
	r.mu.Unlock()

	rs.cancel()
	rs.wg.Wait()

	r.evict(room)

	r.mu.Lock()
	// Only delete if the map still holds THIS roomState. A client reconnecting into
	// the eviction window makes ygo fire RoomActivated (off-lock), which installs a
	// fresh roomState under the same key; an unconditional delete would drop that
	// live entry, leaking its goroutines and silently no-op'ing every future
	// Publish for the room.
	if r.rooms[room] == rs {
		delete(r.rooms, room)
	}
	r.mu.Unlock()
}

// Close stops every room's goroutines. It does NOT close the Querier: the pool is
// shared with the locker and the health probe, and is owned by the caller.
func (r *Relay) Close() error {
	r.mu.Lock()
	if r.closed {
		r.mu.Unlock()
		return nil
	}
	r.closed = true
	rooms := make([]*roomState, 0, len(r.rooms))
	for _, rs := range r.rooms {
		rs.cancel()
		rooms = append(rooms, rs)
	}
	r.rooms = make(map[string]*roomState)
	bgCancel := r.bgCancel
	r.mu.Unlock()

	for _, rs := range rooms {
		rs.wg.Wait()
	}
	// Stop the relay-scoped goroutines rather than waiting for the caller to cancel
	// the Start context, then wait so the listener's pooled connection is released
	// before the pool is closed. nil when Start was never called.
	if bgCancel != nil {
		bgCancel()
	}
	r.bgWG.Wait()
	return nil
}

// ForceEvict is the rollback entry point: stop the room's goroutines and
// unconditionally delete its rows WITHOUT a GCS flush, under the evicting guard so
// a concurrent Publish or reconnect cannot revive rolled-back state. Idempotent;
// safe even if the room is not resident.
func (r *Relay) ForceEvict(ctx context.Context, room string) error {
	r.mu.Lock()
	if r.closed {
		r.mu.Unlock()
		return ErrRelayClosed
	}
	rs, ok := r.rooms[room]
	if ok {
		rs.evicting = true
	}
	r.mu.Unlock()

	if ok {
		rs.cancel()
		rs.wg.Wait()
	}

	_, _ = r.q.Exec(ctx, removeHeartbeatSQL, room, r.clientID)
	if _, err := r.q.Exec(ctx, forceDeleteSQL, room); err != nil {
		r.log.Debug("relay force-evict delete failed", "room", room, "err", err)
		return err
	}

	if ok {
		r.mu.Lock()
		// Only delete if the map still holds the roomState we evicted: a concurrent
		// re-activation may have installed a fresh one under the same key.
		if r.rooms[room] == rs {
			delete(r.rooms, room)
		}
		r.mu.Unlock()
	}
	return nil
}

// evict removes this node's heartbeat; if no instance remains active it flushes to
// GCS then safe-deletes the rows under the election's atomic re-check.
//
// It runs on a fresh bounded context, NOT the relay delivery context: on graceful
// shutdown ygo cancels the delivery context BEFORE closing peers (which triggers
// this eviction), so reusing it would abort the last-instance flush and silently
// lose the room's latest state.
func (r *Relay) evict(room string) {
	ctx, cancel := context.WithTimeout(context.WithoutCancel(context.Background()), evictTimeout)
	defer cancel()

	if _, err := r.q.Exec(ctx, removeHeartbeatSQL, room, r.clientID); err != nil {
		r.log.Debug("relay remove heartbeat failed", "room", room, "err", err)
	}

	active, err := r.activeInstances(ctx, room)
	if err != nil {
		r.log.Debug("relay active-count failed", "room", room, "err", err)
		return
	}
	if active > 0 {
		return
	}

	// Last instance: flush to GCS first, then safe-delete.
	if err := r.flusher.FlushRoom(ctx, room); err != nil {
		r.log.Debug("relay GCS flush failed", "room", room, "err", err)
		// Do NOT delete the rows on flush failure — un-persisted updates would be
		// lost; a reconnect re-replays them.
		return
	}

	// The lock and the DELETE go in one batch so they share an implicit transaction
	// and execute in this order — see electionLockSQL for why the lock cannot be a
	// CTE alongside the DELETE.
	b := &pgx.Batch{}
	b.Queue(electionLockSQL, room)
	b.Queue(safeDeleteSQL, room, lockKeyFor(room), secs(activeTimeout))
	br := r.q.SendBatch(ctx, b)
	if _, err := br.Exec(); err != nil {
		_ = br.Close()
		r.log.Debug("relay election lock failed", "room", room, "err", err)
		return
	}
	var deleted int64
	if err := br.QueryRow().Scan(&deleted); err != nil {
		_ = br.Close()
		r.log.Debug("relay safe-delete failed", "room", room, "err", err)
		return
	}
	if err := br.Close(); err != nil {
		r.log.Debug("relay safe-delete close failed", "room", room, "err", err)
		return
	}
	r.log.Debug("relay evicted room", "room", room, "rows", deleted)
}

// activeInstances counts instances whose heartbeat is within activeTimeout.
func (r *Relay) activeInstances(ctx context.Context, room string) (int, error) {
	var n int
	if err := r.q.QueryRow(ctx, activeInstancesSQL, room, secs(activeTimeout)).Scan(&n); err != nil {
		return 0, err
	}
	return n, nil
}

// heartbeat stamps this instance live for a room.
func (r *Relay) heartbeat(ctx context.Context, room string) error {
	_, err := r.q.Exec(ctx, heartbeatSQL, room, r.clientID)
	return err
}

// heartbeatLoop re-stamps this node's liveness every heartbeatRefresh.
func (r *Relay) heartbeatLoop(ctx context.Context, room string, rs *roomState) {
	defer rs.wg.Done()
	t := time.NewTicker(heartbeatRefresh)
	defer t.Stop()
	for {
		select {
		case <-ctx.Done():
			return
		case <-t.C:
			if err := r.heartbeat(ctx, room); err != nil {
				r.log.Debug("relay heartbeat refresh failed", "room", room, "err", err)
			}
		}
	}
}

// readLoop replays the room's history, then waits for a reason to read again —
// a poll tick, a LISTEN notification, or both, per the configured fan-out.
//
// Catch-up runs here rather than in RoomActivated because ygo holds its rooms lock
// across that callback and an inject would re-enter the Server (ygo#133). Running it
// here preserves catch-up-before-live ordering and the cursor while keeping the
// activation callback re-entrancy-free.
func (r *Relay) readLoop(ctx context.Context, room string, rs *roomState, sink cluster.Sink) {
	defer rs.wg.Done()

	// A nil channel blocks forever in a select, which is exactly the behaviour the
	// notify-only fan-out wants: no ticker at all, wake only on notification.
	var tick <-chan time.Time
	if r.poll > 0 {
		t := time.NewTicker(r.poll)
		defer t.Stop()
		tick = t.C
	}

	for {
		if ctx.Err() != nil {
			return
		}
		if err := r.drain(ctx, room, rs, sink); err != nil {
			if ctx.Err() != nil {
				return
			}
			r.log.Debug("relay read failed", "room", room, "err", err)
			// Back off to avoid a hot error loop against a sick database.
			select {
			case <-ctx.Done():
				return
			case <-time.After(200 * time.Millisecond):
			}
			continue
		}
		select {
		case <-ctx.Done():
			return
		case <-tick:
		case <-rs.wake:
		}
	}
}

// wakeRoom signals one room's reader, or every resident room when room is empty
// (used after a listener reconnect, where missed notifications are unrecoverable
// except by re-reading).
//
// The wake channel has capacity 1 and the send is non-blocking: a pending wakeup
// already means "read again", so coalescing several into one is correct and keeps
// this safe to call from the single listener goroutine.
func (r *Relay) wakeRoom(room string) {
	r.mu.Lock()
	defer r.mu.Unlock()
	if room != "" {
		if rs, ok := r.rooms[room]; ok {
			select {
			case rs.wake <- struct{}{}:
			default:
			}
		}
		return
	}
	for _, rs := range r.rooms {
		select {
		case rs.wake <- struct{}{}:
		default:
		}
	}
}

// drain reads and injects every eligible row after the cursor, in batches, until it
// catches up. Returns on the first error so readLoop can back off.
func (r *Relay) drain(ctx context.Context, room string, rs *roomState, sink cluster.Sink) error {
	for {
		r.mu.Lock()
		from := rs.cursor
		r.mu.Unlock()

		rows, err := r.read(ctx, room, from)
		if err != nil {
			return err
		}
		if len(rows) == 0 {
			return nil
		}

		last := from
		for _, e := range rows {
			// Advance past self-originated and unknown-kind rows too, or a burst of
			// our own writes would stall the cursor and we would re-read forever.
			last = e.id
			if e.isSelf || !e.known || len(e.data) == 0 {
				continue
			}
			// Measure only rows we actually deliver: a self-originated row is never
			// injected anywhere, so counting it would dilute the number that
			// describes what a remote editor experiences.
			r.latency.Observe(e.age)
			if err := sink.Inject(ctx, cluster.Inbound{Room: room, Kind: e.kind, Data: e.data}); err != nil {
				r.log.Debug("relay inject failed", "room", room, "kind", e.kind.String(), "err", err)
			}
		}

		r.mu.Lock()
		// Never move the cursor backwards: a concurrent re-activation may have
		// installed a fresh roomState, and this goroutine's batch is then stale.
		if last > rs.cursor {
			rs.cursor = last
		}
		r.mu.Unlock()

		// A short batch means we have caught up; wait for the next tick.
		if len(rows) < readLimit {
			return nil
		}
	}
}

// read fetches one batch of eligible rows after the cursor.
func (r *Relay) read(ctx context.Context, room string, after int64) ([]entry, error) {
	rows, err := r.q.Query(ctx, readSQL, room, after, secs(r.readLag), readLimit)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	out := make([]entry, 0, 16)
	for rows.Next() {
		var (
			id       int64
			rawKind  int16
			data     []byte
			clientID int64
			ageSecs  float64
		)
		if err := rows.Scan(&id, &rawKind, &data, &clientID, &ageSecs); err != nil {
			return nil, err
		}
		kind, known := decodeKind(rawKind)
		out = append(out, entry{
			id:     id,
			kind:   kind,
			data:   data,
			known:  known,
			isSelf: clientID == r.clientID,
			age:    time.Duration(ageSecs * float64(time.Second)),
		})
	}
	return out, rows.Err()
}

// writeLoop drains the room's bounded queue and inserts each batch in one round
// trip. On ctx cancellation it best-effort drains whatever is buffered, then
// returns.
func (r *Relay) writeLoop(ctx context.Context, room string, rs *roomState) {
	defer rs.wg.Done()
	for {
		select {
		case <-ctx.Done():
			r.drainRemaining(room, rs)
			return
		case first := <-rs.writeCh:
			batch := r.collectBatch(rs, first)
			if err := r.append(ctx, room, batch); err != nil {
				if ctx.Err() == nil {
					r.log.Debug("relay batched insert failed", "room", room, "count", len(batch), "err", err)
				}
			}
		}
	}
}

// collectBatch accumulates queued items of the same kind as first, up to that
// kind's cap, stopping at a kind boundary or when the queue drains.
func (r *Relay) collectBatch(rs *roomState, first writeItem) []writeItem {
	limit := batchCap(first.kind)
	batch := make([]writeItem, 0, limit)
	batch = append(batch, first)
	for len(batch) < limit {
		select {
		case it := <-rs.writeCh:
			if it.kind != first.kind {
				out := batch
				r.requeue(rs, it)
				return out
			}
			batch = append(batch, it)
		default:
			return batch
		}
	}
	return batch
}

// requeue puts an item back at the tail, non-blocking; a full channel drops+counts.
func (r *Relay) requeue(rs *roomState, it writeItem) {
	select {
	case rs.writeCh <- it:
	default:
		r.droppedWrites.Add(1)
	}
}

// drainRemaining flushes items buffered at shutdown, best-effort, on a short
// background context so the already-cancelled ctx does not abort the inserts.
func (r *Relay) drainRemaining(room string, rs *roomState) {
	n := len(rs.writeCh)
	if n == 0 {
		return
	}
	ctx, cancel := context.WithTimeout(context.WithoutCancel(context.Background()), 2*time.Second)
	defer cancel()

	pending := make([]writeItem, 0, n)
	for i := 0; i < n; i++ {
		select {
		case it := <-rs.writeCh:
			pending = append(pending, it)
		default:
			i = n
		}
	}
	for len(pending) > 0 {
		limit := batchCap(pending[0].kind)
		end := 1
		for end < len(pending) && end < limit && pending[end].kind == pending[0].kind {
			end++
		}
		if err := r.append(ctx, room, pending[:end]); err != nil {
			r.log.Debug("relay drain insert failed", "room", room, "count", end, "err", err)
		}
		pending = pending[end:]
	}
}

// append inserts every item for a room in one batched round trip — the equivalent
// of the Redis pipeline.
func (r *Relay) append(ctx context.Context, room string, items []writeItem) error {
	if len(items) == 0 {
		return nil
	}
	b := &pgx.Batch{}
	// FIRST: serialize this document's inserts so its ids commit in allocation
	// order. See serializeInsertSQL — the reader's cursor depends on it.
	b.Queue(serializeInsertSQL, room)
	for i := range items {
		b.Queue(appendSQL, room, encodeKind(items[i].kind), items[i].data, r.clientID)
	}
	// LAST: wake listening instances. Delivered at commit, so it can never arrive
	// before the rows it announces.
	b.Queue(notifySQL, notifyChannel, room)

	// Close reports the first error from any queued statement, so the batch does
	// not need to be stepped through result by result.
	return r.q.SendBatch(ctx, b).Close()
}

// batchCap returns the per-flush cap for a kind (100 sync / 50 awareness).
func batchCap(k cluster.Kind) int {
	if k == cluster.KindAwareness {
		return awarenessBatch
	}
	return syncBatch
}
