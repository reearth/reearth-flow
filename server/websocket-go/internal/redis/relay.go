package redis

import (
	"context"
	"errors"
	"log/slog"
	"strconv"
	"sync"
	"sync/atomic"
	"time"

	goredis "github.com/redis/go-redis/v9"
	"github.com/reearth/ygo/cluster"
)

// ErrRelayClosed is returned by Publish after Close.
var ErrRelayClosed = errors.New("websocket-go/redis: relay closed")

// XREAD COUNT 512 BLOCK 1000 bounds the per-iteration in-memory entries.
const (
	readCount = 512
	readBlock = 1000 * time.Millisecond
)

// evictTimeout bounds the last-instance durability I/O (heartbeat removal, active
// re-check, GCS flush, stream delete). It runs on a fresh context so a cancelled
// relay delivery context (graceful shutdown) cannot abort the flush.
const evictTimeout = 30 * time.Second

// Write-queue bounds. Publish enqueues onto a bounded per-room channel and returns
// immediately; a per-room writer goroutine drains it and pipelines the XADDs in
// batches. On overflow the enqueue drops + counts (recoverable via flush +
// reconnect catch-up).
const (
	writeQueueCap  = 1024 // per-room buffered channel cap
	syncBatch      = 100  // max sync XADDs per pipelined flush
	awarenessBatch = 50   // max awareness XADDs per pipelined flush
)

// writeItem is one queued outbound update awaiting its XADD.
type writeItem struct {
	kind cluster.Kind
	data []byte
}

// Options configures a Relay.
type Options struct {
	// Addr is a host:port (used by tests / when URL is empty).
	Addr string
	// URL is a redis:// connection string. If set, it wins over Addr.
	URL string
	// Logger MUST NEVER receive stream payloads or secrets — only room id + error
	// class. Defaults to slog.Default().
	Logger *slog.Logger
	// Flusher is the GCS persistence seam; defaults to a no-op.
	Flusher Flusher
}

// Relay implements ygo's cluster.Relay over Redis Streams, byte- and
// semantics-compatible with the Rust server. One per-process clientId serves all
// rooms; echo correctness rides ygo's origin sentinel, not the wire clientId.
type Relay struct {
	client   goredis.UniversalClient
	clientID uint64 // self-filter only
	log      *slog.Logger
	flusher  Flusher

	// droppedWrites counts outbound updates dropped on a full write queue.
	droppedWrites atomic.Uint64

	mu     sync.Mutex
	sink   cluster.Sink
	ctx    context.Context // delivery lifetime, set in Start
	rooms  map[string]*roomState
	closed bool
}

type roomState struct {
	cancel context.CancelFunc // stops reader + heartbeat + writer goroutines
	wg     sync.WaitGroup

	// writeCh is the bounded outbound queue; Publish enqueues non-blockingly.
	writeCh chan writeItem

	// evicting refuses Publish so no XADD races the DEL.
	evicting bool

	// lastID is the per-reader stream cursor, advanced past self-filtered entries.
	lastID string
}

var _ interface {
	Publish(context.Context, cluster.Outbound) error
	Start(context.Context, cluster.Sink) error
	RoomActivated(string)
	RoomDeactivated(string)
	Close() error
} = (*Relay)(nil)

// New builds a Relay, drawing the per-process clientId and connecting to Redis.
func New(opts Options) (*Relay, error) {
	id, err := newInstanceID()
	if err != nil {
		return nil, err
	}
	var client goredis.UniversalClient
	if opts.URL != "" {
		o, err := goredis.ParseURL(opts.URL)
		if err != nil {
			return nil, err
		}
		client = goredis.NewClient(o)
	} else {
		client = goredis.NewClient(&goredis.Options{Addr: opts.Addr})
	}
	log := opts.Logger
	if log == nil {
		log = slog.Default()
	}
	fl := opts.Flusher
	if fl == nil {
		fl = noopFlusher{}
	}
	return &Relay{
		client:   client,
		clientID: id,
		log:      log,
		flusher:  fl,
		rooms:    make(map[string]*roomState),
	}, nil
}

// instanceValue is the doc/GCS lock value, "instance-{clientId}".
func (r *Relay) instanceValue() string {
	return "instance-" + strconv.FormatUint(r.clientID, 10)
}

// Start binds the Sink and records the delivery context. Per-room delivery begins
// at RoomActivated. Cancelling ctx (or Close) stops the relay.
func (r *Relay) Start(ctx context.Context, sink cluster.Sink) error {
	r.mu.Lock()
	defer r.mu.Unlock()
	if r.closed {
		return ErrRelayClosed
	}
	r.sink = sink
	r.ctx = ctx
	return nil
}

// Publish enqueues one update onto the room's bounded write queue and returns
// immediately; it must NOT do the XADD inline (that would block the publishing
// goroutine on a slow Redis). Silent no-op while the room is evicting; a full
// queue drops + increments DroppedWrites. Returns ErrRelayClosed after Close.
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
	// XADD happens later on the writer goroutine.
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

// DroppedWrites returns the cumulative count of outbound updates dropped on a full
// write queue.
func (r *Relay) DroppedWrites() uint64 { return r.droppedWrites.Load() }

// RoomActivated starts the per-room subscriber + heartbeat and refreshes the
// stream EXPIRE. Catch-up replay of the existing stream is performed by the
// reader goroutine (NOT inline here): ygo invokes RoomActivated while holding its
// rooms lock, so injecting during the callback re-enters the Server
// (Sink.Inject -> getOrCreateRoom) and deadlocks on that non-reentrant lock
// (ygo#133). Idempotent. The heartbeat is registered before the reader starts so
// a concurrent evictor sees this node active.
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
		lastID:  "0",
		writeCh: make(chan writeItem, writeQueueCap),
	}
	r.rooms[room] = rs
	sink := r.sink
	r.mu.Unlock()

	// Heartbeat first so an evictor's HGETALL sees us active before catch-up.
	if err := updateHeartbeat(ctx, r.client, room, r.clientID); err != nil {
		r.log.Debug("relay heartbeat init failed", "room", room, "err", err)
	}
	if err := publishEmptyMarker(ctx, r.client, room, r.clientID); err != nil {
		r.log.Debug("relay empty marker failed", "room", room, "err", err)
	}

	rs.wg.Add(3)
	go r.readLoop(ctx, room, rs, sink)
	go r.heartbeatLoop(ctx, room, rs)
	go r.writeLoop(ctx, room, rs)
}

// catchUp replays the whole stream (XRANGE - +) into the sink and returns the last
// entry id so the live reader does not re-apply replayed entries. Self entries are
// skipped on apply but still advance the cursor.
func (r *Relay) catchUp(ctx context.Context, room string, sink cluster.Sink) string {
	msgs, err := r.client.XRange(ctx, streamKey(room), "-", "+").Result()
	if err != nil {
		r.log.Debug("relay catch-up XRANGE failed", "room", room, "err", err)
		return "0"
	}
	lastID := "0"
	for _, m := range msgs {
		e := parseEntry(m, r.clientID)
		lastID = e.id
		if e.isSelf {
			continue
		}
		r.inject(ctx, room, sink, e)
	}
	return lastID
}

// readLoop is the live subscriber: XREAD from the per-reader last-id, self-filter,
// route sync/awareness to the sink, and advance last-id past filtered entries too.
func (r *Relay) readLoop(ctx context.Context, room string, rs *roomState, sink cluster.Sink) {
	defer rs.wg.Done()

	// Replay the existing stream history BEFORE the live loop. This runs on the
	// reader goroutine, not in RoomActivated, because ygo calls RoomActivated
	// under its rooms lock and a catch-up inject re-enters the Server
	// (Sink.Inject -> getOrCreateRoom), deadlocking on that non-reentrant lock
	// (ygo#133). Running it here preserves catch-up-before-live ordering and the
	// self-filter cursor while keeping the activation callback re-entrancy-free.
	lastID := r.catchUp(ctx, room, sink)
	r.mu.Lock()
	rs.lastID = lastID
	r.mu.Unlock()

	for {
		if ctx.Err() != nil {
			return
		}
		r.mu.Lock()
		from := rs.lastID
		r.mu.Unlock()

		res, err := r.client.XRead(ctx, &goredis.XReadArgs{
			Streams: []string{streamKey(room), from},
			Count:   readCount,
			Block:   readBlock,
		}).Result()
		if err != nil {
			if ctx.Err() != nil || errors.Is(err, goredis.Nil) {
				// goredis.Nil = BLOCK timeout with no entries; re-block.
				continue
			}
			r.log.Debug("relay XREAD failed", "room", room, "err", err)
			// Backoff to avoid a hot error loop against a sick Redis.
			select {
			case <-ctx.Done():
				return
			case <-time.After(200 * time.Millisecond):
			}
			continue
		}

		newLast := from
		// Drain the batch in order; all injects complete before the cursor advances.
		for _, stream := range res {
			for _, m := range stream.Messages {
				e := parseEntry(m, r.clientID)
				newLast = e.id // advance past self/filtered too
				if e.isSelf {
					continue
				}
				r.inject(ctx, room, sink, e)
			}
		}
		r.mu.Lock()
		rs.lastID = newLast
		r.mu.Unlock()
	}
}

// inject routes a parsed sync/awareness entry to the sink.
func (r *Relay) inject(ctx context.Context, room string, sink cluster.Sink, e parsedEntry) {
	var kind cluster.Kind
	switch e.kind {
	case kindSync:
		kind = cluster.KindSync
	case kindAwareness:
		kind = cluster.KindAwareness
	default:
		return // empty markers / unknown types are no-ops
	}
	if len(e.data) == 0 {
		return
	}
	if err := sink.Inject(ctx, cluster.Inbound{Room: room, Kind: kind, Data: e.data}); err != nil {
		r.log.Debug("relay inject failed", "room", room, "kind", kind.String(), "err", err)
	}
}

// heartbeatLoop refreshes this node's liveness every 30s.
func (r *Relay) heartbeatLoop(ctx context.Context, room string, rs *roomState) {
	defer rs.wg.Done()
	t := time.NewTicker(heartbeatRefresh)
	defer t.Stop()
	for {
		select {
		case <-ctx.Done():
			return
		case <-t.C:
			if err := updateHeartbeat(ctx, r.client, room, r.clientID); err != nil {
				r.log.Debug("relay heartbeat refresh failed", "room", room, "err", err)
			}
		}
	}
}

// writeLoop drains the room's bounded write queue and pipelines the XADDs of a
// single kind per flush, sharing one timestamp per batch. On ctx cancellation it
// best-effort drains whatever is buffered, then returns.
func (r *Relay) writeLoop(ctx context.Context, room string, rs *roomState) {
	defer rs.wg.Done()
	key := streamKey(room)
	for {
		select {
		case <-ctx.Done():
			r.drainRemaining(room, key, rs)
			return
		case first := <-rs.writeCh:
			batch := r.collectBatch(rs, first)
			if err := pipelineXAdd(ctx, r.client, key, r.clientID, batch); err != nil {
				if ctx.Err() == nil {
					r.log.Debug("relay batched XADD failed", "room", room, "count", len(batch), "err", err)
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
				// Kind boundary: flush what we have, requeue this for its own batch.
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

// requeue puts an item back at the tail of the queue, non-blocking; a full channel
// drops + counts.
func (r *Relay) requeue(rs *roomState, it writeItem) {
	select {
	case rs.writeCh <- it:
	default:
		r.droppedWrites.Add(1)
	}
}

// drainRemaining flushes items buffered at shutdown, best-effort, on a short
// background context so the already-cancelled ctx does not abort the XADDs.
func (r *Relay) drainRemaining(room, key string, rs *roomState) {
	n := len(rs.writeCh)
	if n == 0 {
		return
	}
	ctx, cancel := context.WithTimeout(context.Background(), 2*time.Second)
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
	// Flush in kind-contiguous, cap-bounded batches.
	for len(pending) > 0 {
		first := pending[0]
		limit := batchCap(first.kind)
		end := 1
		for end < len(pending) && end < limit && pending[end].kind == first.kind {
			end++
		}
		if err := pipelineXAdd(ctx, r.client, key, r.clientID, pending[:end]); err != nil {
			r.log.Debug("relay drain XADD failed", "room", room, "count", end, "err", err)
		}
		pending = pending[end:]
	}
}

// batchCap returns the per-flush cap for a kind (100 sync / 50 awareness).
func batchCap(k cluster.Kind) int {
	if k == cluster.KindAwareness {
		return awarenessBatch
	}
	return syncBatch
}

// RoomDeactivated stops the room's goroutines and runs the locked evict path: if
// this is the last active instance, flush GCS then safe-delete the stream.
// Idempotent.
func (r *Relay) RoomDeactivated(room string) {
	r.mu.Lock()
	rs, ok := r.rooms[room]
	if !ok || r.closed {
		r.mu.Unlock()
		return
	}
	// Refuse Publish for the whole critical section so no XADD races the DEL.
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

// evict removes this node's heartbeat; if no instance remains active it flushes
// GCS then safe-deletes the stream under the doc lock's atomic re-check.
//
// It runs on a fresh bounded context, NOT the relay delivery context: on graceful
// shutdown ygo cancels the delivery context BEFORE closing peers (which triggers
// this eviction), so reusing it would abort the last-instance flush and silently
// lose the room's latest state for a coexisting Rust cold-load.
func (r *Relay) evict(room string) {
	ctx, cancel := context.WithTimeout(context.Background(), evictTimeout)
	defer cancel()
	last, err := removeHeartbeat(ctx, r.client, room, r.clientID)
	if err != nil {
		r.log.Debug("relay remove heartbeat failed", "room", room, "err", err)
	}
	if !last {
		return
	}
	// Re-confirm no other instance is active before flush+delete (defends against
	// a concurrent re-activation).
	active, err := getActiveInstances(ctx, r.client, room, activeTimeoutSecs)
	if err != nil {
		r.log.Debug("relay active-count failed", "room", room, "err", err)
		return
	}
	if active > 0 {
		return
	}
	// Last instance: flush GCS first, then safe-delete the stream.
	if err := r.flusher.FlushRoom(ctx, room); err != nil {
		r.log.Debug("relay GCS flush failed", "room", room, "err", err)
		// Do NOT delete the stream on flush failure — un-persisted updates would be
		// lost; a reconnect re-replays.
		return
	}
	if err := safeDeleteStream(ctx, r.client, room, r.instanceValue()); err != nil {
		r.log.Debug("relay safe-delete failed", "room", room, "err", err)
	}
}

// ForceEvict is the rollback entry point: stop the room's goroutines and
// unconditionally delete the stream WITHOUT a GCS flush, under the evicting guard
// so a concurrent Publish/reconnect cannot revive rolled-back state. Idempotent;
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

	_, _ = removeHeartbeat(ctx, r.client, room, r.clientID)
	if err := r.client.Del(ctx, streamKey(room)).Err(); err != nil {
		r.log.Debug("relay force-evict DEL failed", "room", room, "err", err)
		return err
	}

	if ok {
		r.mu.Lock()
		// Only delete if the map still holds the roomState we evicted: a concurrent
		// re-activation may have installed a fresh one under the same key, which must
		// not be dropped (leaked goroutines + silently dead Publish).
		if r.rooms[room] == rs {
			delete(r.rooms, room)
		}
		r.mu.Unlock()
	}
	return nil
}

// Close stops every room's goroutines and closes the Redis client. After Close,
// Publish returns ErrRelayClosed. Idempotent.
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
	r.mu.Unlock()

	for _, rs := range rooms {
		rs.wg.Wait()
	}
	return r.client.Close()
}
