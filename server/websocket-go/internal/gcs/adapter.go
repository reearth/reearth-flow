package gcs

import (
	"bytes"
	"context"
	"encoding/binary"
	"fmt"
	"log/slog"
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

	// oidCache memoizes allocated OIDs per room. Phase 2 pins OID=0.
	oidCache map[string]uint32
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
		store:    kv{bucket: opts.Client.Bucket(opts.Bucket)},
		locker:   locker,
		log:      log,
		phase2:   opts.Phase2,
		oidCache: make(map[string]uint32),
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
// refreshes the v1 SUB_DOC + SUB_STATE_VEC so both representations agree.
func (a *Adapter) writeV2Snapshot(ctx context.Context, room DocID, oid uint32, v1State []byte) error {
	doc, err := docFromV1(v1State)
	if err != nil {
		return err
	}
	v2 := crdt.EncodeStateAsUpdateV2(doc, nil)
	if err := a.store.put(ctx, a.layout.DocV2Name(room), compressBrotli(v2)); err != nil {
		return err
	}
	if err := a.store.put(ctx, a.layout.DocStateName(room, oid), crdt.EncodeStateAsUpdateV1(doc, nil)); err != nil {
		return err
	}
	if err := a.store.put(ctx, a.layout.StateVectorName(room, oid), crdt.EncodeStateVectorV1(doc)); err != nil {
		return err
	}
	return nil
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
	doc := crdt.New()
	if err := crdt.ApplyUpdateV2(doc, v2, nil); err != nil {
		return nil, false, err
	}
	return crdt.EncodeStateAsUpdateV1(doc, nil), true, nil
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
