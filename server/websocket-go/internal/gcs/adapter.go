package gcs

import (
	"bytes"
	"context"
	"encoding/binary"
	"errors"
	"fmt"
	"log/slog"
	"sort"
	"sync"

	"cloud.google.com/go/storage"
	"github.com/andybalholm/brotli"
	"github.com/reearth/ygo/crdt"
)

// brotliQuality / brotliWindow are the doc_v2 compression params (q=4, lgwin=22).
const (
	brotliQuality = 4
	brotliWindow  = 22
)

// oidGenerationLockKey is the global Phase-1 OID allocation lock.
const oidGenerationLockKey = "lock:oid_generation"

// gcsDocLockKey is the per-doc save lock taken by PruneAfter/Compact. The
// "gcs:lock:" infix reproduces the Rust double-prefix bug-for-bug.
func gcsDocLockKey(d DocID) string { return "lock:doc:gcs:lock:" + string(d) }

// Adapter implements persistence.VersionedPersistence over GCS, owning the
// reearth-flow byte layout, brotli, OID mechanics, and the double-hex.
type Adapter struct {
	store    kv
	layout   Layout
	fallback Layout // Phase-2 only: legacy-root, for dual-read backfill; nil in Phase 1.
	locker   Locker
	log      *slog.Logger // never log payloads — sizes/clocks/room only

	phase2 bool

	// crashAfterCheckpoint, when set and true, simulates a crash after the
	// durable checkpoint write but before deleting future updates in PruneAfter.
	mu                   sync.Mutex
	crashAfterCheckpoint func() bool
	// crashAfterRecoveryWrite, when set and true, simulates a crash in
	// AppendUpdate's recovery path right after the new update is durably
	// written (test injection only).
	crashAfterRecoveryWrite func() bool
	// beforeUpdatePut, when set, runs between choosing an update's clock and
	// writing it (test injection only).
	beforeUpdatePut func(clock uint32)
	// afterReadBase, when set, runs in readState between reading doc_v2 and
	// listing the update log (test injection only).
	afterReadBase func()
	// afterCheckMarks, when set, runs in readState between its marks check and
	// reading the update objects (test injection only).
	afterCheckMarks func()
	// beforeV2Write, when set, runs in writeV2DocIf before its conditional
	// doc_v2 write (test injection only).
	beforeV2Write func()
	// afterCompactSelect, when set, runs in Compact between choosing the cutoff
	// and reading the state it folds (test injection only).
	afterCompactSelect func()
	// afterCompactBase, when set, runs in Compact between the base write and the
	// checkpoint publish (test injection only).
	afterCompactBase func()
	// afterCompactPublish, when set, runs in Compact between the checkpoint
	// publish and the check before its deletes (test injection only).
	afterCompactPublish func()
	// beforeCompactDelete, when set, runs in Compact between that check and the
	// deletes (test injection only).
	beforeCompactDelete func()
	// beforeMirrorPut, when set, runs in writeV2DocIf between confirming doc_v2 is
	// still ours and writing the named v1 mirror (test injection only).
	beforeMirrorPut func(name string)
	// afterRollbackMark and afterRollbackDocV2, when set, run in PruneAfter after
	// marking the ceiling and after publishing doc_v2; returning true stops it
	// there, as a crash would (test injection only).
	afterRollbackMark  func() bool
	afterRollbackDocV2 func() bool
	// afterRollbackCheckpoint is the same, after the checkpoint write.
	afterRollbackCheckpoint func() bool

	// oidCache memoizes allocated OIDs per room. Phase 2 pins OID=0.
	oidCache map[string]uint32

	// transientKeys are map keys stripped from every doc_v2 snapshot (mapName →
	// keys). Used for ephemeral UI state (e.g. metadata.rollbackInProgress) that
	// rides the CRDT for live broadcast but must never persist to storage.
	transientKeys map[string][]string
	// keepSnapshots is Options.KeepSnapshots.
	keepSnapshots int
}

// Options configure a GCS Adapter.
type Options struct {
	// Client is an initialized GCS client.
	Client *storage.Client
	// Bucket is the GCS bucket name.
	Bucket string
	// Locker coordinates OID allocation and flush/prune. Defaults to NewNoLock().
	Locker Locker
	// Phase2 enables the {projectId}/ layout with dual-read + lazy backfill.
	Phase2 bool
	// Logger receives diagnostics (sizes/clocks/room only, never payloads).
	// Defaults to slog.Default().
	Logger *slog.Logger
	// TransientMapKeys are doc map keys stripped from every doc_v2 snapshot
	// (mapName → keys), for ephemeral UI state that must never be persisted.
	TransientMapKeys map[string][]string
	// KeepSnapshots bounds the snapshots SaveSnapshot keeps per retention
	// class (see trimSnapshots); zero keeps them all.
	KeepSnapshots int
}

// New builds a GCS Adapter: LegacyRootLayout in Phase 1; ProjectFolderLayout
// primary with LegacyRootLayout dual-read fallback in Phase 2.
func New(opts Options) (*Adapter, error) {
	if opts.Client == nil {
		return nil, fmt.Errorf("gcs: nil client")
	}
	if opts.Bucket == "" {
		return nil, fmt.Errorf("gcs: empty bucket")
	}
	locker := opts.Locker
	if locker == nil {
		locker = NewNoLock()
	}
	log := opts.Logger
	if log == nil {
		log = slog.Default()
	}
	a := &Adapter{
		store:         kv{bucket: opts.Client.Bucket(opts.Bucket)},
		locker:        locker,
		log:           log,
		phase2:        opts.Phase2,
		oidCache:      make(map[string]uint32),
		transientKeys: opts.TransientMapKeys,
		keepSnapshots: opts.KeepSnapshots,
	}
	if opts.Phase2 {
		a.layout = ProjectFolderLayout{}
		a.fallback = LegacyRootLayout{}
	} else {
		a.layout = LegacyRootLayout{}
	}
	return a, nil
}

// validate enforces Phase-2 path-safety before D is used as a raw path prefix.
// Phase 1 hex-encodes D, so traversal is already neutralized and the check is
// skipped (Go must be no stricter than Rust on the shared legacy keyspace).
func (a *Adapter) validate(d DocID) error {
	if a.phase2 {
		return ValidateDocIDForPrefix(d)
	}
	return nil
}

// oidFor returns the OID for room. Phase 2 pins FolderOID. Phase 1 reads the OID
// index, allocating a monotonic u32 from system:last_oid under the global lock.
func (a *Adapter) oidFor(ctx context.Context, room DocID) (uint32, error) {
	if a.phase2 {
		return FolderOID, nil
	}
	a.mu.Lock()
	if oid, ok := a.oidCache[room]; ok {
		a.mu.Unlock()
		return oid, nil
	}
	a.mu.Unlock()

	if b, err := a.store.get(ctx, a.layout.OIDIndexName(room)); err == nil {
		if len(b) >= 4 {
			oid := binary.BigEndian.Uint32(b)
			a.mu.Lock()
			a.oidCache[room] = oid
			a.mu.Unlock()
			return oid, nil
		}
	} else if err != errNotFound {
		return 0, err
	}

	var allocated uint32
	err := a.locker.WithLock(ctx, oidGenerationLockKey, func(ctx context.Context) error {
		// Re-check inside the lock (another process may have allocated).
		if b, err := a.store.get(ctx, a.layout.OIDIndexName(room)); err == nil {
			if len(b) >= 4 {
				allocated = binary.BigEndian.Uint32(b)
				return nil
			}
		} else if err != errNotFound {
			return err
		}
		var last uint32
		if b, err := a.store.get(ctx, SystemLastOIDName()); err == nil {
			if len(b) >= 4 {
				last = binary.BigEndian.Uint32(b)
			}
		} else if err != errNotFound {
			return err
		}
		next := last + 1
		if err := a.store.put(ctx, SystemLastOIDName(), be32(next)); err != nil {
			return err
		}
		if err := a.store.put(ctx, a.layout.OIDIndexName(room), be32(next)); err != nil {
			return err
		}
		allocated = next
		return nil
	})
	if err != nil {
		return 0, err
	}
	a.mu.Lock()
	a.oidCache[room] = allocated
	a.mu.Unlock()
	return allocated, nil
}

// updateClock parses the v1 update clock from an object name. The trailing hex
// segment decodes to V1 ‖ KEYSPACE_DOC ‖ oid(4) ‖ SUB_UPDATE ‖ clock(4) ‖
// TERMINATOR (12 bytes); the clock is bytes [7..11] BE.
func updateClock(name string) (uint32, bool) {
	hexPart := name
	if i := lastSlash(name); i >= 0 {
		hexPart = name[i+1:]
	}
	kb, err := hexDecode(hexPart)
	if err != nil || len(kb) < 12 {
		return 0, false
	}
	if kb[0] != rsV1 || kb[1] != rsKeyspaceDoc || kb[6] != rsSubUpdate {
		return 0, false
	}
	return binary.BigEndian.Uint32(kb[7:11]), true
}

func lastSlash(s string) int {
	for i := len(s) - 1; i >= 0; i-- {
		if s[i] == '/' {
			return i
		}
	}
	return -1
}

// listUpdates returns (clock → object name) for every update object of room.
func (a *Adapter) listUpdates(ctx context.Context, room DocID, oid uint32) (map[uint32]string, error) {
	prefix := a.layout.UpdatePrefix(room, oid)
	names, err := a.store.list(ctx, prefix)
	if err != nil {
		return nil, err
	}
	out := make(map[uint32]string, len(names))
	for _, n := range names {
		if c, ok := updateClock(n); ok {
			out[c] = n
		}
	}
	return out, nil
}

func (a *Adapter) listUpdatesAttrs(ctx context.Context, room DocID, oid uint32) (map[uint32]*storage.ObjectAttrs, error) {
	prefix := a.layout.UpdatePrefix(room, oid)
	attrs, err := a.store.listAttrs(ctx, prefix)
	if err != nil {
		return nil, err
	}
	out := make(map[uint32]*storage.ObjectAttrs, len(attrs))
	for _, at := range attrs {
		if c, ok := updateClock(at.Name); ok {
			out[c] = at
		}
	}
	return out, nil
}

// checkpoint reads the compaction checkpoint clock (0 if absent).
func (a *Adapter) checkpoint(ctx context.Context, room DocID) (uint32, error) {
	b, err := a.store.get(ctx, a.layout.CheckpointName(room))
	if err == errNotFound {
		return 0, nil
	}
	if err != nil {
		return 0, err
	}
	if len(b) < 4 {
		return 0, nil
	}
	return binary.BigEndian.Uint32(b), nil
}

func compressBrotli(data []byte) []byte {
	var buf bytes.Buffer
	w := brotli.NewWriterOptions(&buf, brotli.WriterOptions{Quality: brotliQuality, LGWin: brotliWindow})
	_, _ = w.Write(data)
	_ = w.Close()
	return buf.Bytes()
}

func decompressBrotli(data []byte) ([]byte, error) {
	r := brotli.NewReader(bytes.NewReader(data))
	var out bytes.Buffer
	if _, err := out.ReadFrom(r); err != nil {
		return nil, err
	}
	return out.Bytes(), nil
}

// ErrPartialV2Decode reports that a stored doc_v2 payload decoded only partially.
var ErrPartialV2Decode = errors.New("gcs: doc_v2 decoded only partially")

// docFromV2 decodes a stored doc_v2 payload and REFUSES a partial decode.
//
// A doc_v2 object is a complete, self-contained state snapshot: every item it
// references is inside it, so a correct decoder never has anything left over.
// Items still parked in the pending queue therefore mean the decoder could not
// integrate part of the document and the doc we are holding is silently smaller
// than the bytes on disk (reearth/ygo#231 — ApplyUpdateV2 drops content when the
// update contains GC structs, and returns nil while doing it).
//
// Returning the truncated doc would hand a partial document to the room, and the
// next flush would write that truncation back over the original, destroying data
// that is still intact in storage. Failing the load is the safe half of that
// trade: the room refuses to open, loudly, and the stored bytes stay untouched.
func docFromV2(v2 []byte) (*crdt.Doc, error) {
	doc := crdt.New()
	if err := crdt.ApplyUpdateV2(doc, v2, nil); err != nil {
		return nil, err
	}
	if ps := doc.PendingStats(); ps.Items > 0 {
		return nil, fmt.Errorf("%w: %d items left unintegrated, awaiting %d clients; refusing to serve a truncated document",
			ErrPartialV2Decode, ps.Items, len(ps.MissingFor))
	}
	return doc, nil
}

// stateFromV2 decodes a stored doc_v2 payload into a V1 update, keeping any
// parked items (the V1 encode carries them along, as Yjs does).
func stateFromV2(v2 []byte) ([]byte, error) {
	doc := crdt.New()
	if err := crdt.ApplyUpdateV2(doc, v2, nil); err != nil {
		return nil, err
	}
	return crdt.EncodeStateAsUpdateV1(doc, nil), nil
}

// foldParked applies the update objects at clocks to doc when doc holds parked
// items. The checkpoint claims those objects are already in doc_v2, but a doc_v2
// written from a copy that missed one of them is not.
func (a *Adapter) foldParked(ctx context.Context, room DocID, doc *crdt.Doc, updates map[uint32]string, clocks []uint32) (bool, error) {
	before := doc.PendingStats().Items
	if before == 0 || len(clocks) == 0 {
		return false, nil
	}
	for _, c := range clocks {
		if err := a.applyObject(ctx, doc, updates[c]); err != nil {
			return false, err
		}
	}
	a.log.Info("gcs: folded checkpointed update objects into a state with parked items",
		"room", room, "parked_before", before, "parked_after", doc.PendingStats().Items)
	return true, nil
}

// clocksUpTo returns the leading clocks of the ascending slice that are <= max.
func clocksUpTo(sorted []uint32, max uint32) []uint32 {
	n := sort.Search(len(sorted), func(i int) bool { return sorted[i] > max })
	return sorted[:n]
}

// docFromV1 applies a V1 update to a fresh doc and returns it.
func docFromV1(v1 []byte) (*crdt.Doc, error) {
	d := crdt.New()
	if len(v1) == 0 {
		return d, nil
	}
	if err := crdt.ApplyUpdateV1(d, v1, nil); err != nil {
		return nil, err
	}
	return d, nil
}

// writeV2Snapshot writes state (a V1 update) as the brotli(V2) doc_v2 object and
// refreshes the v1 SUB_DOC + SUB_STATE_VEC so both representations agree. It
// replaces doc_v2 outright; only rollback and backfill write this way.
func (a *Adapter) writeV2Snapshot(ctx context.Context, room DocID, oid uint32, v1State []byte, cov coverage) error {
	doc, err := docFromV1(v1State)
	if err != nil {
		return err
	}
	return a.writeV2Doc(ctx, room, oid, doc, cov)
}

// writeV2Doc writes doc as doc_v2, SUB_DOC and SUB_STATE_VEC. It strips the
// transient keys from doc first.
func (a *Adapter) writeV2Doc(ctx context.Context, room DocID, oid uint32, doc *crdt.Doc, cov coverage) error {
	return a.writeV2DocWith(ctx, room, oid, doc, cov, func(name string, b []byte, meta map[string]string) (int64, error) {
		// Unconditional: rollback and backfill hold the lock and publish doc_v2
		// outright. The generation still fences their mirrors, since a rollback
		// whose lease lapsed can resume after a newer one published.
		return a.store.putGen(ctx, name, b, meta)
	})
}

// writeV2DocIf is writeV2Doc, but writes nothing and returns errGenerationMoved
// once doc_v2 is no longer at generation gen (0 for absent).
func (a *Adapter) writeV2DocIf(ctx context.Context, room DocID, oid uint32, doc *crdt.Doc, gen int64, cov coverage) (int64, error) {
	var wrote int64
	err := a.writeV2DocWith(ctx, room, oid, doc, cov, func(name string, b []byte, meta map[string]string) (int64, error) {
		if a.beforeV2Write != nil {
			a.beforeV2Write()
		}
		g, err := a.store.putIfGenerationMeta(ctx, name, b, gen, meta)
		wrote = g
		return g, err
	})
	return wrote, err
}

func (a *Adapter) writeV2DocWith(ctx context.Context, room DocID, oid uint32, doc *crdt.Doc, cov coverage, putV2 func(name string, b []byte, meta map[string]string) (int64, error)) error {
	// Strip ephemeral UI keys (e.g. metadata.rollbackInProgress) before persisting:
	// they ride the CRDT for live broadcast but must never survive in doc_v2, which
	// the Rust reader loads on its own with no tail updates to clear them.
	a.stripTransient(doc)
	v2 := crdt.EncodeStateAsUpdateV2(doc, nil)
	gen, err := putV2(a.layout.DocV2Name(room), compressBrotli(v2), cov.meta())
	if err != nil {
		return err
	}
	docState := crdt.EncodeStateAsUpdateV1(doc, nil)
	stateVec := crdt.EncodeStateVectorV1(doc)
	// Without a generation to tie them to, publish the mirrors outright.
	if gen == 0 {
		if err := a.store.put(ctx, a.layout.DocStateName(room, oid), docState); err != nil {
			return err
		}
		return a.store.put(ctx, a.layout.StateVectorName(room, oid), stateVec)
	}
	// The V1 mirrors belong to the doc_v2 just written. A writer whose lease
	// lapsed can reach them after a newer doc_v2 was published, so each mirror is
	// fenced the same way as doc_v2 itself.
	for _, m := range []struct {
		name string
		data []byte
	}{
		{a.layout.DocStateName(room, oid), docState},
		{a.layout.StateVectorName(room, oid), stateVec},
	} {
		ours, err := a.putMirror(ctx, room, m.name, m.data, gen)
		if err != nil || !ours {
			return err
		}
	}
	return nil
}

// mirrorPutAttempts bounds how often putMirror retries after losing a mirror
// write to another writer while doc_v2 is still ours.
const mirrorPutAttempts = 4

// putMirror writes one v1 mirror of the doc_v2 at generation ours. The mirror's
// generation is read before confirming doc_v2 is still ours, and the write is
// conditional on it: a newer doc_v2 is always published before its mirrors, so
// any newer writer's mirror write lands after that read and makes this one fail.
// It returns false, writing nothing, once doc_v2 has moved on.
func (a *Adapter) putMirror(ctx context.Context, room DocID, name string, data []byte, ours int64) (bool, error) {
	for attempt := 1; ; attempt++ {
		mg, err := a.store.generation(ctx, name)
		if err != nil {
			return false, err
		}
		cur, err := a.store.generation(ctx, a.layout.DocV2Name(room))
		if err != nil {
			return false, err
		}
		if cur != ours {
			a.log.Warn("gcs: doc_v2 moved on after our write, leaving the v1 mirrors to the newer writer",
				"room", room, "wrote", ours, "current", cur)
			return false, nil
		}
		if a.beforeMirrorPut != nil {
			a.beforeMirrorPut(name)
		}
		_, err = a.store.putIfGeneration(ctx, name, data, mg)
		if err == nil {
			return true, nil
		}
		// Lost to another writer's mirror. While doc_v2 is still ours that writer
		// is older, so read again and take the mirror back.
		if !errors.Is(err, errGenerationMoved) || attempt == mirrorPutAttempts {
			return false, err
		}
	}
}

// stripTransient deletes the configured ephemeral map keys from doc. No-op when
// none are configured or the keys are absent.
//
// GetMap and Has acquire the doc lock, so they must run OUTSIDE Transact (which
// already holds it) — calling them inside would deadlock a non-reentrant lock.
// Only the actual deletion runs in the transaction, and only for present keys so
// an all-absent doc is never perturbed.
func (a *Adapter) stripTransient(doc *crdt.Doc) {
	if len(a.transientKeys) == 0 {
		return
	}
	type target struct {
		m    *crdt.YMap
		keys []string
	}
	var targets []target
	for mapName, keys := range a.transientKeys {
		m := doc.GetMap(mapName)
		var present []string
		for _, k := range keys {
			if m.Has(k) {
				present = append(present, k)
			}
		}
		if len(present) > 0 {
			targets = append(targets, target{m: m, keys: present})
		}
	}
	if len(targets) == 0 {
		return
	}
	doc.Transact(func(txn *crdt.Transaction) {
		for _, t := range targets {
			for _, k := range t.keys {
				t.m.Delete(txn, k)
			}
		}
	})
}

// loadV2 reads + brotli-decodes + V2-decodes the doc_v2 snapshot into a V1
// update. Returns (nil, false, nil) when absent.
func (a *Adapter) loadV2(ctx context.Context, room DocID) ([]byte, bool, error) {
	b, err := a.getDual(ctx, func(l Layout) string { return l.DocV2Name(room) })
	if err == errNotFound {
		return nil, false, nil
	}
	if err != nil {
		return nil, false, err
	}
	v2, err := decompressBrotli(b)
	if err != nil {
		return nil, false, err
	}
	v1, err := stateFromV2(v2)
	if err != nil {
		return nil, false, err
	}
	return v1, true, nil
}

// getDual reads name(primaryLayout); on a Phase-2 miss it falls back to
// name(legacyLayout) for the same room. In Phase 1 fallback is nil (plain read).
func (a *Adapter) getDual(ctx context.Context, nameFn func(Layout) string) ([]byte, error) {
	b, err := a.store.get(ctx, nameFn(a.layout))
	if err == nil {
		return b, nil
	}
	if err != errNotFound || a.fallback == nil {
		return nil, err
	}
	return a.store.get(ctx, nameFn(a.fallback))
}
