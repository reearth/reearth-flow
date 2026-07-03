package http

import (
	"context"
	"errors"
	"fmt"
	"log/slog"

	"github.com/reearth/ygo/persistence"
)

// Persistence is the subset of the GCS adapter's VersionedPersistence surface
// the HTTP layer needs (production value: *gcs.Adapter).
type Persistence interface {
	Load(ctx context.Context, room string) (persistence.LoadResult, error)
	AppendUpdate(ctx context.Context, room string, update []byte) (persistence.Version, error)
	ListVersions(ctx context.Context, room string) ([]persistence.VersionMeta, error)
	GetUpdate(ctx context.Context, room string, v persistence.Version) ([]byte, persistence.VersionMeta, bool, error)
	MaterializeAt(ctx context.Context, room string, v persistence.Version) ([]byte, error)
	PruneAfter(ctx context.Context, room string, target persistence.Version, rolledBack []byte) error
	CaptureSnapshot(ctx context.Context, room, name string, state []byte) (persistence.Version, error)
	Delete(ctx context.Context, room string) error
	Compact(ctx context.Context, room string, keep int) (int, error)
}

// StoreAdapterDeps configure StoreAdapter.
type StoreAdapterDeps struct {
	// P is the GCS-backed persistence.
	P Persistence
	// FlushFn forces a GCS snapshot of a live room. nil ⇒ Flush falls back to
	// CaptureSnapshot of the current Load state.
	FlushFn func(ctx context.Context, room string) error
	// ListRooms enumerates rooms for admin cleanup when P does not implement
	// DocLister. nil ⇒ no-op.
	ListRooms func() []string
	// Logger receives per-doc CleanupAll warnings (room + err only, never a
	// payload). nil ⇒ slog.Default().
	Logger *slog.Logger
}

// DocLister optionally enumerates every persisted doc id. When Persistence
// satisfies it, CleanupAll sweeps the full bucket instead of resident rooms.
type DocLister interface {
	ListAllDocs(ctx context.Context) ([]string, error)
}

// StoreAdapter bridges the GCS persistence to the HTTP DocStore interface.
type StoreAdapter struct {
	p         Persistence
	flushFn   func(ctx context.Context, room string) error
	listRooms func() []string
	log       *slog.Logger
}

// NewStoreAdapter builds the bridge. The result satisfies DocStore.
func NewStoreAdapter(d StoreAdapterDeps) *StoreAdapter {
	log := d.Logger
	if log == nil {
		log = slog.Default()
	}
	return &StoreAdapter{p: d.P, flushFn: d.FlushFn, listRooms: d.ListRooms, log: log}
}

var _ DocStore = (*StoreAdapter)(nil)

func (s *StoreAdapter) Load(ctx context.Context, room string) (LoadResult, error) {
	res, err := s.p.Load(ctx, room)
	if err != nil {
		return LoadResult{}, err
	}
	// LoadResult carries no timestamp; take the head version's time from the
	// newest history entry (ListVersions is newest-first).
	out := LoadResult{Update: res.Update, Version: uint64(res.Version)}
	if metas, verr := s.p.ListVersions(ctx, room); verr == nil && len(metas) > 0 {
		out.UpdatedAt = metas[0].UpdatedAt
	}
	return out, nil
}

func (s *StoreAdapter) ListVersions(ctx context.Context, room string) ([]VersionInfo, error) {
	metas, err := s.p.ListVersions(ctx, room)
	if err != nil {
		return nil, err
	}
	out := make([]VersionInfo, len(metas))
	for i, m := range metas {
		out[i] = VersionInfo{Version: uint64(m.Version), UpdatedAt: m.UpdatedAt}
	}
	return out, nil
}

func (s *StoreAdapter) GetUpdate(ctx context.Context, room string, v uint64) ([]byte, VersionInfo, bool, error) {
	b, m, ok, err := s.p.GetUpdate(ctx, room, persistence.Version(v))
	if err != nil || !ok {
		return nil, VersionInfo{}, ok, err
	}
	return b, VersionInfo{Version: uint64(m.Version), UpdatedAt: m.UpdatedAt}, true, nil
}

func (s *StoreAdapter) MaterializeAt(ctx context.Context, room string, v uint64) ([]byte, error) {
	return s.p.MaterializeAt(ctx, room, persistence.Version(v))
}

func (s *StoreAdapter) PruneAfter(ctx context.Context, room string, target uint64, rolledBack []byte) error {
	return s.p.PruneAfter(ctx, room, persistence.Version(target), rolledBack)
}

// Flush forces a durable GCS snapshot of the room via FlushFn. When FlushFn is
// nil (tests), it falls back to a name-less CaptureSnapshot of the loaded state.
func (s *StoreAdapter) Flush(ctx context.Context, room string) error {
	if s.flushFn != nil {
		return s.flushFn(ctx, room)
	}
	res, err := s.p.Load(ctx, room)
	if err != nil {
		return err
	}
	_, err = s.p.CaptureSnapshot(ctx, room, "", res.Update)
	return err
}

func (s *StoreAdapter) CaptureSnapshot(ctx context.Context, room, name string, state []byte) (uint64, error) {
	v, err := s.p.CaptureSnapshot(ctx, room, name, state)
	return uint64(v), err
}

// Copy duplicates src's persisted state into dst by materializing src's full
// state and appending it as dst's next update.
func (s *StoreAdapter) Copy(ctx context.Context, dst, src string) error {
	state, err := s.p.Load(ctx, src)
	if err != nil {
		return err
	}
	if len(state.Update) == 0 {
		return nil // nothing to copy
	}
	_, err = s.p.AppendUpdate(ctx, dst, state.Update)
	return err
}

// Import applies a raw v1 update as the room's next version.
func (s *StoreAdapter) Import(ctx context.Context, room string, data []byte) (uint64, error) {
	v, err := s.p.AppendUpdate(ctx, room, data)
	return uint64(v), err
}

// Compact keeps at most `keep` update generations for one room.
func (s *StoreAdapter) Compact(ctx context.Context, room string, keep int) (int, error) {
	return s.p.Compact(ctx, room, keep)
}

func (s *StoreAdapter) Delete(ctx context.Context, room string) error {
	return s.p.Delete(ctx, room)
}

// CleanupAll compacts every persisted doc (keeping at most `keep` generations
// each) and returns the total update objects deleted. A per-doc Compact error is
// warn-logged (room + err only) and the loop continues; errors are aggregated
// via errors.Join. A failure to enumerate the doc set is fatal.
func (s *StoreAdapter) CleanupAll(ctx context.Context, keep int) (int, error) {
	docs, err := s.cleanupRooms(ctx)
	if err != nil {
		return 0, err
	}
	total := 0
	var errs []error
	for _, room := range docs {
		n, cerr := s.p.Compact(ctx, room, keep)
		if cerr != nil {
			// room + err only — never the doc payload.
			s.log.Warn("admin cleanup: compact failed for doc, continuing", "room", room, "err", cerr)
			errs = append(errs, fmt.Errorf("compact %q: %w", room, cerr))
			continue
		}
		total += n
	}
	return total, errors.Join(errs...)
}

// cleanupRooms picks the doc set CleanupAll compacts: full-bucket ListAllDocs
// when supported, else the resident-room fallback.
func (s *StoreAdapter) cleanupRooms(ctx context.Context) ([]string, error) {
	if dl, ok := s.p.(DocLister); ok {
		return dl.ListAllDocs(ctx)
	}
	if s.listRooms == nil {
		return nil, nil
	}
	return s.listRooms(), nil
}
