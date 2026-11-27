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

	"github.com/reearth/reearth-flow/websocket-go/internal/latency"
	"github.com/reearth/reearth-flow/websocket-go/internal/relaypolicy"
)

// ErrRelayClosed is returned by Publish after Close.
var ErrRelayClosed = errors.New("websocket-go/redis: relay closed")

// XREAD COUNT 512 BLOCK 1000 bounds the per-iteration in-memory entries.
const (
	readCount = 512
	readBlock = 1000 * time.Millisecond
	// readBackoff paces both reader retries (catch-up and live) off a sick Redis.
	readBackoff = 200 * time.Millisecond
)

// Stream retention. An entry older than streamRetention behind every active
// reader's cursor costs memory on the next catch-up and nothing else, provided it
// is durable — which trimStream establishes with a flush rather than assuming it
// from age. Awareness entries are ephemeral anyway. Each active room trims every
// streamTrimEvery.
const (
	streamRetention = 10 * time.Minute
	streamTrimEvery = time.Minute
)

// selfReplayWindow is how long after a room is activated onto a live relay
// state, or a deactivation leaves the state in use, the live reader also applies
// this instance's own sync entries; see roomState.selfApplyUntil.
const selfReplayWindow = 30 * time.Second

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

	// droppedWrites counts outbound updates dropped on a full write queue, or
	// arriving for a room that is no longer active here and could not be written.
	droppedWrites atomic.Uint64
	// lateWrites counts updates written after their room stopped being active
	// here; see publishLate.
	lateWrites atomic.Uint64

	latency *latency.Recorder

	// bgWG tracks the relay-scoped latency reporter and bgCancel stops it. Close
	// must be able to stop it ITSELF: waiting on a goroutine that only exits with
	// the Start context deadlocks any caller that closes before cancelling.
	bgWG     sync.WaitGroup
	bgCancel context.CancelFunc

	// beforeEnqueue, when set, runs in Publish just before an update is queued.
	// Tests use it to interleave RoomDeactivated.
	beforeEnqueue func()
	// beforeFenceIO, when set, runs in LoadStarting between finding the room's fence
	// state and locking it (test injection only).
	beforeFenceIO func()
	// beforeRestore, when set, runs in restoreSuccessor between finding the
	// successor and writing its heartbeat (test injection only).
	beforeRestore func()

	// presenceMu makes a restore for a successor and that successor's own removal
	// of the heartbeat happen one after the other. It is taken before mu, never after.
	presenceMu sync.Mutex

	mu     sync.Mutex
	sink   cluster.Sink
	ctx    context.Context // delivery lifetime, set in Start
	rooms  map[string]*roomState
	closed bool

	// fences holds the room loads this instance has fenced (see loadfence.go),
	// and loadSeq numbers loads across rooms. Both under mu.
	fences  map[string]*loadFence
	loadSeq uint64
	// now is the relay's clock for load fences; tests replace it.
	now func() time.Time
}

type roomState struct {
	cancel context.CancelFunc // stops reader + heartbeat + writer goroutines
	wg     sync.WaitGroup

	// activations counts the ygo rooms using this state. ygo can activate a new
	// room under the same name before deactivating the old one.
	activations int
	// replay asks the reader to replay the stream: a room activated onto this
	// state loaded from storage, which can lack entries the reader already applied
	// to the room it replaces.
	replay bool
	// replayed is closed once a replay finishes with none asked for since it
	// began; nil when no replay is pending.
	replayed chan struct{}
	// replayAsks counts replay requests, so a trim can tell that one was asked
	// for and finished while it was flushing.
	replayAsks uint64
	// selfApplyUntil: until then the live reader applies this instance's own sync
	// entries too. A room activated onto this state, onto a fresh state replacing
	// an evicting one, or left on it by a deactivation, can lack updates the room
	// it replaces published after the replacement's catch-up read the stream: the retiring room's lane drains after
	// RoomDeactivated, and a write batch can be retried for about 2 s.
	selfApplyUntil time.Time

	// writeCh is the bounded outbound queue; Publish enqueues non-blockingly.
	writeCh chan writeItem

	// evicting refuses Publish so no XADD races the DEL.
	evicting bool

	// lastID is the per-reader stream cursor, advanced past self-filtered entries.
	lastID string

	// outboundGap is set once an update for this room failed to reach the stream,
	// so other instances may lack it until they reload the room. gapWarned keeps
	// the warning to one per room.
	outboundGap bool
	gapWarned   bool

	// caughtUp is closed once the reader has replayed the stream as it stood
	// when the room was activated; done when the reader stops.
	caughtUp chan struct{}
	done     chan struct{}
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
		latency:  latency.New(log, "redis"),
		rooms:    make(map[string]*roomState),
		fences:   make(map[string]*loadFence),
		now:      time.Now,
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

	// Derived so either cancelling ctx or calling Close stops the reporter.
	bgCtx, cancel := context.WithCancel(ctx)
	r.bgCancel = cancel
	r.bgWG.Go(func() { r.latency.Run(bgCtx) })
	return nil
}

// Publish enqueues one update onto the room's bounded write queue and returns
// immediately; it must NOT do the XADD inline (that would block the publishing
// goroutine on a slow Redis). An update for a room that is evicting or not active
// here is written directly (publishLate); a full queue drops + increments
// DroppedWrites. Returns ErrRelayClosed after Close.
func (r *Relay) Publish(_ context.Context, out cluster.Outbound) error {
	r.mu.Lock()
	if r.closed {
		r.mu.Unlock()
		return ErrRelayClosed
	}
	rs, ok := r.rooms[out.Room]
	if !ok || rs.evicting {
		r.mu.Unlock()
		r.publishLate(out)
		return nil
	}
	if r.beforeEnqueue != nil {
		r.beforeEnqueue()
	}
	// Enqueue while holding r.mu: RoomDeactivated sets evicting under it before
	// stopping the writer, so the writer's final drain always sees this item.
	// Copy the payload: the caller's slice may alias internal buffers.
	item := writeItem{kind: out.Kind, data: append([]byte(nil), out.Data...)}
	select {
	case rs.writeCh <- item:
		r.mu.Unlock()
	default:
		// Blocking here would stall ygo's lane worker, so the update is dropped and
		// the room flagged for repair from storage, where ygo has already persisted it.
		r.markGapLocked(out.Room, rs, "write queue full", nil)
		r.mu.Unlock()
		r.droppedWrites.Add(1)
	}
	return nil
}

// markGap records that an update for the room did not reach the stream. It warns
// once per room; droppedWrites counts every loss.
func (r *Relay) markGap(room string, rs *roomState, reason string, err error) {
	r.mu.Lock()
	defer r.mu.Unlock()
	r.markGapLocked(room, rs, reason, err)
}

func (r *Relay) markGapLocked(room string, rs *roomState, reason string, err error) {
	rs.outboundGap = true
	if rs.gapWarned {
		return
	}
	rs.gapWarned = true
	args := []any{"room", room, "reason", reason}
	if err != nil {
		args = append(args, "err", err)
	}
	r.log.Warn("relay lost an update; instances holding the room miss it until they reload the room", args...)
}

// OutboundGap reports whether an update for a room this instance holds failed to
// reach the other instances holding it.
func (r *Relay) OutboundGap(room string) bool {
	r.mu.Lock()
	defer r.mu.Unlock()
	rs, ok := r.rooms[room]
	return ok && rs.outboundGap
}

// DroppedWrites returns the cumulative count of outbound updates dropped on a full
// write queue, or for a room no longer active here that could not be written.
func (r *Relay) DroppedWrites() uint64 { return r.droppedWrites.Load() }

// LateWrites returns the cumulative count of updates written after their room
// stopped being active here.
func (r *Relay) LateWrites() uint64 { return r.lateWrites.Load() }

// lateWriteTimeout bounds one late XADD.
const lateWriteTimeout = 2 * time.Second

// writeTimeout bounds one batched XADD.
const writeTimeout = 2 * time.Second

// publishLate writes a document update that ygo hands over after deactivating
// its room here, which other instances holding the room still need. NOMKSTREAM
// skips it once no instance holds the room, when the update is already persisted.
func (r *Relay) publishLate(out cluster.Outbound) {
	if out.Kind != cluster.KindSync {
		return
	}
	ctx, cancel := context.WithTimeout(context.Background(), lateWriteTimeout)
	defer cancel()
	ts := strconv.FormatInt(time.Now().UnixMilli(), 10)
	err := r.client.XAdd(ctx, &goredis.XAddArgs{
		Stream:     streamKey(out.Room),
		NoMkStream: true,
		ID:         "*",
		Values:     xaddValuesTS(msgTypeSync, out.Data, r.clientID, ts),
	}).Err()
	switch {
	case errors.Is(err, goredis.Nil):
		// No stream: no instance holds the room.
	case err != nil:
		r.droppedWrites.Add(1)
		r.log.Warn("relay could not write a late update; instances holding the room miss it until they reload it",
			"room", out.Room, "err", err)
	default:
		r.lateWrites.Add(1)
	}
}

// ErrRoomNotActive is what WaitCaughtUp returns for a room this instance does
// not hold, or stops holding while waiting.
var ErrRoomNotActive = errors.New("websocket-go/redis: room not active on this instance")

// caughtUpPoll is how often WaitCaughtUp checks the reader's cursor.
const caughtUpPoll = 5 * time.Millisecond

// WaitCaughtUp waits until this instance's reader has applied every entry the
// room's stream held when it was called, so the room's document holds every
// update other instances had published by then.
func (r *Relay) WaitCaughtUp(ctx context.Context, room string) error {
	r.mu.Lock()
	rs, ok := r.rooms[room]
	active := ok && !rs.evicting
	var replayed chan struct{}
	if active {
		replayed = rs.replayed
	}
	r.mu.Unlock()
	if !active {
		return ErrRoomNotActive
	}
	head, err := r.client.XRevRangeN(ctx, streamKey(room), "+", "-", 1).Result()
	if err != nil {
		return err
	}
	for _, ch := range []chan struct{}{rs.caughtUp, replayed} {
		if ch == nil {
			continue
		}
		select {
		case <-ch:
		case <-rs.done:
			return ErrRoomNotActive
		case <-ctx.Done():
			return ctx.Err()
		}
	}
	if len(head) == 0 {
		return nil
	}
	target := head[0].ID
	for {
		r.mu.Lock()
		cur := rs.lastID
		r.mu.Unlock()
		if !streamIDBefore(cur, target) {
			return nil
		}
		select {
		case <-rs.done:
			return ErrRoomNotActive
		case <-ctx.Done():
			return ctx.Err()
		case <-time.After(caughtUpPoll):
		}
	}
}

// ActiveInstances counts the instances holding room, by their heartbeats.
func (r *Relay) ActiveInstances(ctx context.Context, room string) (int, error) {
	n, err := getActiveInstances(ctx, r.client, room, activeTimeoutSecs)
	return int(n), err
}

// RoomActivated starts the per-room subscriber + heartbeat and refreshes the
// stream EXPIRE. Catch-up replay of the existing stream is performed by the
// reader goroutine (NOT inline here): ygo invokes RoomActivated while holding its
// rooms lock, so injecting during the callback re-enters the Server
// (Sink.Inject -> getOrCreateRoom) and deadlocks on that non-reentrant lock
// (ygo#133). The heartbeat is registered before the reader starts so a
// concurrent evictor sees this node active.
//
// Every call is matched by one RoomDeactivated, and the room stays active until
// the last of them. ygo can activate a room again before deactivating the room it
// replaces: a relay entry or a reconnect can re-create a room while the old one
// is still being torn down.
func (r *Relay) RoomActivated(room string) {
	r.mu.Lock()
	if r.closed || r.sink == nil {
		r.mu.Unlock()
		return
	}
	old, ok := r.rooms[room]
	if ok && !old.evicting {
		old.activations++
		old.replay = true
		old.replayAsks++
		old.selfApplyUntil = r.now().Add(selfReplayWindow)
		if old.replayed == nil {
			old.replayed = make(chan struct{})
		}
		r.mu.Unlock()
		return
	}
	// An evicting state is replaced at once rather than waited for: its eviction
	// may be waiting on the reader goroutine this call is running on.
	ctx, cancel := context.WithCancel(r.ctx)
	rs := &roomState{
		activations: 1,
		cancel:      cancel,
		lastID:      "0",
		writeCh:     make(chan writeItem, writeQueueCap),
		caughtUp:    make(chan struct{}),
		done:        make(chan struct{}),
	}
	if ok {
		// It replaces a room whose state is still evicting, so that room can still
		// publish after this state's catch-up; see roomState.selfApplyUntil.
		rs.selfApplyUntil = r.now().Add(selfReplayWindow)
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

	rs.wg.Add(4)
	go r.readLoop(ctx, room, rs, sink)
	go r.heartbeatLoop(ctx, room, rs)
	go r.writeLoop(ctx, room, rs)
	go r.trimLoop(ctx, room, rs)
}

// catchUp replays the whole stream into the sink, readCount entries per XRANGE so
// a long stream is never materialized at once, and returns the last entry id so
// the live reader does not re-apply replayed entries. This instance's own sync
// entries are applied as well, its own awareness entries are skipped, and both
// advance the cursor.
func (r *Relay) catchUp(ctx context.Context, room string, sink cluster.Sink) (string, error) {
	// A pass that completes clears the load fences of loads that began before it.
	covers := r.passBegin(room)
	lastID := "0"
	start := "-"
	for {
		r.passRefresh(ctx, room)
		msgs, err := r.client.XRangeN(ctx, streamKey(room), start, "+", readCount).Result()
		if err != nil {
			return "", err
		}
		for _, m := range msgs {
			e := parseEntry(m, r.clientID)
			// This instance's own sync entries are applied too: the room may have
			// loaded storage that lacks them. See applyOwn.
			if !e.isSelf || r.applyOwn(room, sink, e) {
				switch r.inject(ctx, room, sink, e, false) { // catch-up: replay depth, not latency
				case relaypolicy.Stop:
					return "", errStopReading
				case relaypolicy.Retry:
					return "", errInjectRetry
				}
			}
			lastID = e.id
		}
		if len(msgs) < readCount {
			r.passEnd(ctx, room, covers)
			return lastID, nil
		}
		next, ok := nextStreamID(lastID)
		if !ok {
			r.passEnd(ctx, room, covers)
			return lastID, nil
		}
		start = next
	}
}

// trimLoop trims the room's stream every streamTrimEvery.
func (r *Relay) trimLoop(ctx context.Context, room string, rs *roomState) {
	defer rs.wg.Done()
	t := time.NewTicker(streamTrimEvery)
	defer t.Stop()
	for {
		select {
		case <-ctx.Done():
			return
		case <-t.C:
			if err := r.trimStream(ctx, room, rs); err != nil && ctx.Err() == nil {
				r.log.Debug("relay stream trim failed", "room", room, "err", err)
			}
		}
	}
}

// trimStream drops the prefix of the stream that is provably safe to lose, which
// takes four independent bounds, all of which must hold:
//
//   - Age: more than streamRetention older than this reader's cursor. Anchoring on
//     the cursor (a Redis-clock id) keeps local clock skew out of the boundary and
//     never trims anything this reader has not yet applied. No-op until catch-up
//     has set a cursor.
//   - Readers: XTRIM mutates a stream every instance shares, so the boundary is
//     also held back to the lowest cursor advertised by any active instance. A
//     reader stuck in Sink.Inject keeps advertising its old cursor from its own
//     trim goroutine, so the stream cannot be trimmed out from under it; an active
//     instance with no advertised cursor (a coexisting Rust node) blocks the trim
//     entirely.
//   - Durability: age alone does not establish that anything reached GCS, because
//     the cursor advances on Inject while persistence is asynchronous. The trim is
//     therefore gated on a successful FlushRoom, whose completion means the whole
//     converged document — every entry at or below the boundary — is in GCS. On a
//     GCS outage the flush fails, no trim happens, and Redis keeps the only replay
//     copy until persistence recovers. A checkpoint rebuilt from storage, with no
//     resident document, vouches for nothing the reader applied and allows no trim.
//   - Loads: a room that loaded storage lacking an entry needs it from the stream
//     until a pass over the stream has given it to the room. No instance trims
//     while any instance holds a load fence (loadfence.go), and the fences and
//     this instance's load and replay counts are checked again after the flush.
//
// The checkpoint is skipped when the stream holds nothing below the boundary, so
// an idle room settles into cheap no-op ticks rather than a flush a minute.
func (r *Relay) trimStream(ctx context.Context, room string, rs *roomState) error {
	r.mu.Lock()
	cursor := rs.lastID
	replaying := rs.replayed != nil
	asks := rs.replayAsks
	r.mu.Unlock()
	// A replay asked for and not yet finished, or a load on this instance not yet
	// read through, can still need entries below the cursor that the resident
	// document lacks, so neither flush nor trim until it is.
	if replaying || r.localFenceLive(room) {
		return nil
	}
	gen := r.loadGen(room)
	ms, ok := streamIDMillis(cursor)
	if !ok {
		return nil
	}
	minMs := ms - streamRetention.Milliseconds()
	if minMs <= 0 {
		return nil
	}

	floorMs, ok, err := publishTrimFloor(ctx, r.client, room, r.clientID, ms)
	if err != nil {
		return err
	}
	if !ok {
		return nil // an active instance has not advertised a cursor, or one is loading
	}
	if floorMs < minMs {
		minMs = floorMs
	}
	if minMs <= 0 {
		return nil
	}

	key := streamKey(room)
	has, err := r.hasEntriesBelow(ctx, key, minMs)
	if err != nil || !has {
		return err
	}
	// Durable checkpoint before anything is discarded. Only a resident document
	// is known to hold every entry this reader applied.
	live, err := checkpointLive(ctx, r.flusher, room)
	if err != nil || !live {
		return err
	}
	// A load can start during the flush, and here a room can even load, replay
	// and clear its fence inside it, after the flush took a document lacking the
	// entries. So refuse if a load or replay began since the first check; a load
	// starting after this point reads what the flush wrote.
	if r.trimRaced(room, rs, gen, asks) {
		return nil
	}
	if fenced, err := loadFenced(ctx, r.client, room, nowSecs()); err != nil || fenced {
		return err
	}
	// Approximate: exact MINID costs time proportional to the entries removed, so
	// the first tick on an already-oversized stream would monopolize Redis. Matches
	// the Rust backend's `MINID ~`.
	return r.client.XTrimMinIDApprox(ctx, key, strconv.FormatInt(minMs, 10)+"-0", 0).Err()
}

// trimRaced reports whether, since a trim recorded gen and asks, a load or a
// replay began on this instance or one is still pending.
func (r *Relay) trimRaced(room string, rs *roomState, gen, asks uint64) bool {
	if r.localFenceLive(room) || r.loadGen(room) != gen {
		return true
	}
	r.mu.Lock()
	defer r.mu.Unlock()
	return rs.replayed != nil || rs.replayAsks != asks
}

// hasEntriesBelow reports whether the stream's oldest entry sits below minMs,
// i.e. whether a trim would remove anything at all.
func (r *Relay) hasEntriesBelow(ctx context.Context, key string, minMs int64) (bool, error) {
	first, err := r.client.XRangeN(ctx, key, "-", "+", 1).Result()
	if err != nil {
		return false, err
	}
	if len(first) == 0 {
		return false, nil
	}
	firstMs, ok := streamIDMillis(first[0].ID)
	if !ok {
		return false, nil
	}
	return firstMs < minMs, nil
}

// readLoop is the live subscriber: XREAD from the per-reader last-id, self-filter,
// route sync/awareness to the sink, and advance last-id past filtered entries too.
func (r *Relay) readLoop(ctx context.Context, room string, rs *roomState, sink cluster.Sink) {
	defer rs.wg.Done()
	defer close(rs.done)

	var lastID string
	for {
		var err error
		if lastID, err = r.catchUp(ctx, room, sink); err == nil {
			break
		}
		if ctx.Err() != nil || errors.Is(err, errStopReading) {
			return
		}
		r.log.Debug("relay catch-up XRANGE failed", "room", room, "err", err)
		select {
		case <-ctx.Done():
			return
		case <-time.After(readBackoff):
		}
	}
	r.mu.Lock()
	rs.lastID = lastID
	r.mu.Unlock()
	close(rs.caughtUp)

	for {
		if ctx.Err() != nil {
			return
		}
		if r.replayIfAsked(ctx, room, rs, sink) {
			return
		}
		r.mu.Lock()
		from := rs.lastID
		applyOwn := r.now().Before(rs.selfApplyUntil)
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
			case <-time.After(readBackoff):
			}
			continue
		}

		newLast := from
		held := false
		// Drain the batch in order; all injects complete before the cursor advances.
		// The cursor stops before an entry the room could not take yet: the stream
		// keeps it until every reader's advertised cursor has passed it.
	batch:
		for _, stream := range res {
			for _, m := range stream.Messages {
				e := parseEntry(m, r.clientID)
				if !e.isSelf || (applyOwn && r.applyOwn(room, sink, e)) {
					switch r.inject(ctx, room, sink, e, !e.isSelf) {
					case relaypolicy.Stop:
						return
					case relaypolicy.Retry:
						held = true
						break batch
					}
				}
				newLast = e.id // past applied, skipped and self/filtered entries
			}
		}
		r.mu.Lock()
		rs.lastID = newLast
		r.mu.Unlock()
		if held {
			select {
			case <-ctx.Done():
				return
			case <-time.After(readBackoff):
			}
		}
	}
}

// replayIfAsked replays the stream into the room when an activation asked for it,
// moving the cursor only forward and asking again after a failure. It reports
// true when the server is shutting down and the reader should exit.
func (r *Relay) replayIfAsked(ctx context.Context, room string, rs *roomState, sink cluster.Sink) bool {
	r.mu.Lock()
	asked := rs.replay
	rs.replay = false
	r.mu.Unlock()
	if !asked {
		return false
	}
	last, err := r.catchUp(ctx, room, sink)
	r.mu.Lock()
	defer r.mu.Unlock()
	if err != nil {
		rs.replay = true
		rs.replayAsks++
		if errors.Is(err, errStopReading) {
			return true
		}
		if ctx.Err() == nil {
			r.log.Debug("relay replay failed", "room", room, "err", err)
		}
		return false
	}
	if streamIDBefore(rs.lastID, last) {
		rs.lastID = last
	}
	if !rs.replay && rs.replayed != nil {
		close(rs.replayed)
		rs.replayed = nil
	}
	return false
}

// applyOwn reports whether a reader applies e, one of this instance's own
// entries: only sync, since own awareness would bring back clients that left
// with the replaced room. And only into a resident room, since Inject would
// reload a room being torn down; a room loaded later gets e from its catch-up.
func (r *Relay) applyOwn(room string, sink cluster.Sink, e parsedEntry) bool {
	return e.kind == kindSync && sink.GetDoc(room) != nil
}

// errInjectRetry reports an entry the room could not take yet; the reader keeps
// its cursor before it and reads it again.
var errInjectRetry = errors.New("relay: an entry could not be injected yet")

// errStopReading reports that the server is shutting down, so the reader exits
// rather than recording a catch-up or replay it did not finish.
var errStopReading = errors.New("relay: the server is shutting down")

// inject routes a parsed sync/awareness entry to the sink and reports what the
// reader should do with it.
func (r *Relay) inject(ctx context.Context, room string, sink cluster.Sink, e parsedEntry, measure bool) relaypolicy.InjectOutcome {
	var kind cluster.Kind
	switch e.kind {
	case kindSync:
		kind = cluster.KindSync
	case kindAwareness:
		kind = cluster.KindAwareness
	default:
		return relaypolicy.Applied // empty markers / unknown types are no-ops
	}
	if len(e.data) == 0 {
		return relaypolicy.Applied
	}
	in := cluster.Inbound{Room: room, Kind: kind, Data: e.data}
	err := sink.Inject(ctx, in)
	outcome := relaypolicy.ClassifyInject(ctx, in, err)
	switch outcome {
	case relaypolicy.Applied:
		// Measure only live deliveries. Catch-up replays entries that may be hours
		// old, and their age is replay depth, not the latency a remote editor saw.
		if measure {
			r.latency.Observe(e.age)
		}
	case relaypolicy.Skip:
		r.log.Warn("relay skipped an entry the room cannot apply", "room", room, "kind", kind.String(), "err", err)
	case relaypolicy.Retry:
		r.log.Debug("relay inject failed; reading the entry again", "room", room, "kind", kind.String(), "err", err)
	}
	return outcome
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
			batch := r.collectBatch(room, rs, first)
			r.writeWithRetry(ctx, room, key, rs, batch)
		}
	}
}

// writeWithRetry writes a batch, retrying a failure before the writer takes
// anything else off the queue so this instance's order holds; once every attempt
// fails, the batch is dropped and the room flagged. It writes even after
// RoomDeactivated cancels ctx, since a room's last edits are published just
// before it deactivates and every later edit builds on them.
func (r *Relay) writeWithRetry(ctx context.Context, room, key string, rs *roomState, batch []writeItem) {
	for attempt := 0; ; attempt++ {
		wctx, cancel := context.WithTimeout(context.WithoutCancel(ctx), writeTimeout)
		err := pipelineXAdd(wctx, r.client, key, r.clientID, batch)
		cancel()
		if err == nil {
			return
		}
		if attempt == len(relaypolicy.WriteRetryDelays) {
			r.droppedWrites.Add(uint64(len(batch)))
			r.markGap(room, rs, "batched XADD failed after retries", err)
			return
		}
		time.Sleep(relaypolicy.WriteRetryDelays[attempt])
	}
}

// collectBatch accumulates queued items of the same kind as first, up to that
// kind's cap, stopping at a kind boundary or when the queue drains.
func (r *Relay) collectBatch(room string, rs *roomState, first writeItem) []writeItem {
	limit := batchCap(first.kind)
	batch := make([]writeItem, 0, limit)
	batch = append(batch, first)
	for len(batch) < limit {
		select {
		case it := <-rs.writeCh:
			if it.kind != first.kind {
				// Kind boundary: flush what we have, requeue this for its own batch.
				out := batch
				r.requeue(room, rs, it)
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
func (r *Relay) requeue(room string, rs *roomState, it writeItem) {
	select {
	case rs.writeCh <- it:
	default:
		r.droppedWrites.Add(1)
		r.markGap(room, rs, "write queue full", nil)
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
			r.droppedWrites.Add(uint64(end))
			r.log.Warn("relay drain XADD failed; instances holding the room miss these updates until they reload it",
				"room", room, "count", end, "err", err)
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

// RoomDeactivated ends one RoomActivated. On the last, it stops the room's
// goroutines and runs the locked evict path: if this is the last active instance,
// flush GCS then safe-delete the stream.
func (r *Relay) RoomDeactivated(room string) {
	r.mu.Lock()
	rs, ok := r.rooms[room]
	if !ok || r.closed || rs.evicting {
		r.mu.Unlock()
		return
	}
	rs.activations--
	if rs.activations > 0 {
		// The room that left may still publish: its lane drains after this call.
		rs.selfApplyUntil = r.now().Add(selfReplayWindow)
		r.mu.Unlock()
		return
	}
	// Refuse Publish for the whole critical section so no XADD races the DEL.
	rs.evicting = true
	r.mu.Unlock()

	rs.cancel()
	rs.wg.Wait()

	r.evict(room, rs)

	r.mu.Lock()
	// A RoomActivated during the eviction installed a successor under the same
	// key, which must stay.
	if r.rooms[room] == rs {
		delete(r.rooms, room)
		r.dropFenceIfIdleLocked(room)
	}
	r.mu.Unlock()
}

// successor returns the live state that replaced rs while rs was evicting.
func (r *Relay) successor(room string, rs *roomState) (*roomState, bool) {
	r.mu.Lock()
	defer r.mu.Unlock()
	next, ok := r.rooms[room]
	if !ok || next == rs || next.evicting {
		return nil, false
	}
	return next, true
}

// restoreSuccessor re-registers the heartbeat and cursor of a state that
// replaced rs, which rs's eviction removed because both share this instance's id.
// Holding presenceMu throughout means a successor that leaves removes them after this.
func (r *Relay) restoreSuccessor(ctx context.Context, room string, rs *roomState) {
	r.presenceMu.Lock()
	defer r.presenceMu.Unlock()
	next, ok := r.successor(room, rs)
	if !ok {
		return
	}
	if r.beforeRestore != nil {
		r.beforeRestore()
	}
	if err := updateHeartbeat(ctx, r.client, room, r.clientID); err != nil {
		r.log.Warn("relay could not restore the heartbeat of a reactivated room", "room", room, "err", err)
	}
	r.mu.Lock()
	cursor := next.lastID
	r.mu.Unlock()
	if ms, ok := streamIDMillis(cursor); ok {
		if _, _, err := publishTrimFloor(ctx, r.client, room, r.clientID, ms); err != nil {
			r.log.Debug("relay could not restore the cursor of a reactivated room", "room", room, "err", err)
		}
	}
}

// evict removes this node's heartbeat; if no instance remains active it flushes
// GCS then safe-deletes the stream under the doc lock's atomic re-check.
//
// It runs on a fresh bounded context, NOT the relay delivery context: on graceful
// shutdown ygo cancels the delivery context BEFORE closing peers (which triggers
// this eviction), so reusing it would abort the last-instance flush and silently
// lose the room's latest state for a coexisting Rust cold-load.
//
// A state that replaced rs during the eviction keeps this instance live: the
// stream is kept, and its heartbeat and cursor are restored at the end.
func (r *Relay) evict(room string, rs *roomState) {
	ctx, cancel := context.WithTimeout(context.Background(), evictTimeout)
	defer cancel()
	defer r.restoreSuccessor(ctx, room, rs)
	r.presenceMu.Lock()
	last, err := removeHeartbeat(ctx, r.client, room, r.clientID)
	if err != nil {
		r.log.Debug("relay remove heartbeat failed", "room", room, "err", err)
	}
	// Stop advertising a cursor: a departed reader must not hold another
	// instance's trim floor down until the hash expires.
	if err := dropCursor(ctx, r.client, room, r.clientID); err != nil {
		r.log.Debug("relay drop cursor failed", "room", room, "err", err)
	}
	r.presenceMu.Unlock()
	if !last {
		return
	}
	if _, ok := r.successor(room, rs); ok {
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
	if err := flushLastInstance(ctx, r.flusher, room); err != nil {
		r.log.Debug("relay GCS flush failed", "room", room, "err", err)
		// Do NOT delete the stream on flush failure — un-persisted updates would be
		// lost; a reconnect re-replays.
		return
	}
	if _, ok := r.successor(room, rs); ok {
		return
	}
	// A load here whose fence never reached Redis still needs the stream.
	if r.localFenceLive(room) {
		return
	}
	if err := safeDeleteStream(ctx, r.client, room, r.instanceValue()); err != nil {
		r.log.Debug("relay safe-delete failed", "room", room, "err", err)
		return
	}
	if err := r.client.Del(ctx, cursorsKey(room)).Err(); err != nil {
		r.log.Debug("relay cursors cleanup failed", "room", room, "err", err)
	}
}

// ForceEvict is the rollback entry point: stop the room's goroutines and
// unconditionally delete the stream WITHOUT a GCS flush. Idempotent; safe even if
// the room is not resident.
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

	r.presenceMu.Lock()
	_, _ = removeHeartbeat(ctx, r.client, room, r.clientID)
	if err := r.client.Del(ctx, cursorsKey(room)).Err(); err != nil {
		r.log.Debug("relay force-evict cursors DEL failed", "room", room, "err", err)
	}
	r.presenceMu.Unlock()
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
	bgCancel := r.bgCancel
	r.mu.Unlock()

	for _, rs := range rooms {
		rs.wg.Wait()
	}
	// Stop the reporter rather than waiting for the caller to cancel the Start
	// context; nil when Start was never called.
	if bgCancel != nil {
		bgCancel()
	}
	r.bgWG.Wait()
	return r.client.Close()
}
