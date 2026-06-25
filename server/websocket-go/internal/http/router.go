package http

import (
	"context"
	"encoding/json"
	"errors"
	"log/slog"
	"net/http"
	"strconv"
	"time"
)

// keepUpdates is the update-object retention count for the cleanup routes.
const keepUpdates = 10

// ErrNotFound signals a missing document. The Load handler maps it to 404.
var ErrNotFound = errors.New("document not found")

// LoadResult is the latest merged state of a room.
type LoadResult struct {
	Update    []byte
	Version   uint64
	UpdatedAt time.Time
}

// VersionInfo is one history entry's metadata.
type VersionInfo struct {
	Version   uint64
	UpdatedAt time.Time
}

// DocStore is the persistence surface the HTTP handlers consume; the production
// impl adapts the GCS adapter (see adapt.go).
type DocStore interface {
	// Load returns the latest merged state. ErrNotFound ⇒ 404.
	Load(ctx context.Context, room string) (LoadResult, error)
	// ListVersions returns history metadata newest-first.
	ListVersions(ctx context.Context, room string) ([]VersionInfo, error)
	// GetUpdate returns the single (non-cumulative) v1 update at version v.
	GetUpdate(ctx context.Context, room string, v uint64) ([]byte, VersionInfo, bool, error)
	// MaterializeAt rebuilds the full state at v.
	MaterializeAt(ctx context.Context, room string, v uint64) ([]byte, error)
	// PruneAfter persists the rolled-back state and prunes updates > target.
	PruneAfter(ctx context.Context, room string, target uint64, rolledBack []byte) error
	// Flush forces a GCS snapshot of the room's current state.
	Flush(ctx context.Context, room string) error
	// (No CaptureSnapshot: the snapshot endpoint is read-only.)
	// Copy duplicates src's persisted state into dst.
	Copy(ctx context.Context, dst, src string) error
	// Import applies a raw v1 update as a new version; returns the new version.
	Import(ctx context.Context, room string, data []byte) (uint64, error)
	// Compact keeps at most `keep` update generations.
	Compact(ctx context.Context, room string, keep int) (int, error)
	// Delete removes ALL persisted state for room.
	Delete(ctx context.Context, room string) error
	// CleanupAll runs Compact(keep) across every known document.
	CleanupAll(ctx context.Context, keep int) (int, error)
}

// Signaler toggles metadata.rollbackInProgress on a live room so UI clients hide
// the canvas during a rollback. Optional (nil ⇒ no-op).
type Signaler interface {
	SignalRollback(ctx context.Context, room string, inProgress bool) error
}

// Deps are the router's collaborators.
type Deps struct {
	Store    DocStore
	Signaler Signaler // optional
	Logger   *slog.Logger
}

type router struct {
	store    DocStore
	signaler Signaler
	log      *slog.Logger
}

// NewRouter builds the HTTP document API. It does NOT apply the X-API-Secret
// middleware — wrap the returned handler with RequireAPISecret at the edge.
func NewRouter(d Deps) http.Handler {
	r := &router{store: d.Store, signaler: d.Signaler, log: d.Logger}
	if r.log == nil {
		r.log = slog.Default()
	}
	mux := http.NewServeMux()
	mux.HandleFunc("GET /api/document/{id}", r.getLatest)
	mux.HandleFunc("GET /api/document/{id}/history", r.getHistory)
	mux.HandleFunc("GET /api/document/{id}/history/metadata", r.getHistoryMetadata)
	mux.HandleFunc("GET /api/document/{id}/history/version/{version}", r.getHistoryByVersion)
	mux.HandleFunc("POST /api/document/{id}/rollback", r.rollback)
	mux.HandleFunc("POST /api/document/{id}/flush", r.flush)
	mux.HandleFunc("POST /api/document/snapshot", r.createSnapshot)
	mux.HandleFunc("POST /api/document/{id}/{source}/copy", r.copyDocument)
	mux.HandleFunc("POST /api/document/{id}/import", r.importDocument)
	mux.HandleFunc("POST /api/document/{id}/cleanup", r.cleanupUpdates)
	mux.HandleFunc("DELETE /api/document/{id}", r.deleteDocument)
	mux.HandleFunc("POST /api/admin/cleanup", r.adminCleanup)
	return mux
}

func writeJSON(w http.ResponseWriter, code int, v any) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(code)
	_ = json.NewEncoder(w).Encode(v)
}

func writeErr(w http.ResponseWriter, code int, msg string) {
	writeJSON(w, code, errorResponse{Error: msg})
}

// fail logs the underlying cause of a 500 at ERROR (so it is never lost behind
// the generic client message) and writes the response. args are extra slog
// key/value context, e.g. "doc", id.
func (r *router) fail(w http.ResponseWriter, msg string, err error, args ...any) {
	r.log.Error(msg, append(args, "err", err)...)
	writeErr(w, http.StatusInternalServerError, msg)
}

func (r *router) getLatest(w http.ResponseWriter, req *http.Request) {
	id := req.PathValue("id")
	res, err := r.store.Load(req.Context(), id)
	if errors.Is(err, ErrNotFound) {
		writeErr(w, http.StatusNotFound, "document not found")
		return
	}
	if err != nil {
		r.fail(w, "load failed", err, "doc", id)
		return
	}
	writeJSON(w, http.StatusOK, DocumentResponse{
		ID:        id,
		Timestamp: res.UpdatedAt.Format(time.RFC3339),
		Updates:   res.Update,
		Version:   res.Version,
	})
}

func (r *router) getHistory(w http.ResponseWriter, req *http.Request) {
	id := req.PathValue("id")
	versions, err := r.store.ListVersions(req.Context(), id)
	if err != nil {
		r.fail(w, "history failed", err, "doc", id)
		return
	}
	items := make([]DocumentResponse, 0, len(versions))
	for _, v := range versions {
		b, meta, ok, err := r.store.GetUpdate(req.Context(), id, v.Version)
		if err != nil {
			r.fail(w, "history failed", err, "doc", id)
			return
		}
		if !ok {
			continue
		}
		items = append(items, DocumentResponse{
			ID:        id,
			Timestamp: meta.UpdatedAt.Format(time.RFC3339),
			Updates:   b,
			Version:   v.Version,
		})
	}
	writeJSON(w, http.StatusOK, items)
}

func (r *router) getHistoryMetadata(w http.ResponseWriter, req *http.Request) {
	id := req.PathValue("id")
	versions, err := r.store.ListVersions(req.Context(), id)
	if err != nil {
		r.fail(w, "history metadata failed", err, "doc", id)
		return
	}
	items := make([]HistoryMetadataItem, 0, len(versions))
	for _, v := range versions {
		items = append(items, HistoryMetadataItem{
			Version:   v.Version,
			Timestamp: v.UpdatedAt.Format(time.RFC3339),
		})
	}
	writeJSON(w, http.StatusOK, items)
}

func (r *router) getHistoryByVersion(w http.ResponseWriter, req *http.Request) {
	id := req.PathValue("id")
	v, err := strconv.ParseUint(req.PathValue("version"), 10, 64)
	if err != nil {
		writeErr(w, http.StatusBadRequest, "invalid version")
		return
	}
	b, meta, ok, err := r.store.GetUpdate(req.Context(), id, v)
	if err != nil {
		r.fail(w, "history version failed", err, "doc", id, "version", v)
		return
	}
	if !ok {
		writeErr(w, http.StatusNotFound, "version not found")
		return
	}
	writeJSON(w, http.StatusOK, DocumentResponse{
		ID:        id,
		Timestamp: meta.UpdatedAt.Format(time.RFC3339),
		Updates:   b,
		Version:   v,
	})
}

// rollback rebuilds state at the target version and prunes updates > target,
// bracketing the prune with rollbackInProgress (set → prune → clear). The PATH
// id and BODY version are authoritative.
func (r *router) rollback(w http.ResponseWriter, req *http.Request) {
	id := req.PathValue("id")
	var body RollbackRequest
	if err := json.NewDecoder(req.Body).Decode(&body); err != nil {
		writeErr(w, http.StatusBadRequest, "invalid body")
		return
	}
	ctx := req.Context()
	rolledBack, err := r.store.MaterializeAt(ctx, id, body.Version)
	if err != nil {
		r.fail(w, "rollback materialize failed", err, "doc", id, "version", body.Version)
		return
	}
	r.signal(ctx, id, true)
	// Always clear the flag, even on prune failure or client disconnect:
	// WithoutCancel keeps the clear off client liveness (else the canvas stays
	// hidden until the room next reloads).
	clearCtx := context.WithoutCancel(ctx)
	defer r.signal(clearCtx, id, false)
	if err := r.store.PruneAfter(ctx, id, body.Version, rolledBack); err != nil {
		r.fail(w, "rollback prune failed", err, "doc", id, "version", body.Version)
		return
	}
	writeJSON(w, http.StatusOK, map[string]any{"status": "ok", "version": body.Version})
}

// signal toggles rollbackInProgress on the live room. A nil signaler is a no-op;
// errors are logged, not surfaced.
func (r *router) signal(ctx context.Context, room string, inProgress bool) {
	if r.signaler == nil {
		return
	}
	if err := r.signaler.SignalRollback(ctx, room, inProgress); err != nil {
		r.log.Warn("rollback signal failed", "room", room, "inProgress", inProgress, "err", err)
	}
}

func (r *router) flush(w http.ResponseWriter, req *http.Request) {
	id := req.PathValue("id")
	if err := r.store.Flush(req.Context(), id); err != nil {
		r.fail(w, "flush failed", err, "doc", id)
		return
	}
	writeJSON(w, http.StatusOK, map[string]string{"status": "ok"})
}

// createSnapshot is READ-ONLY: it materializes the state at the requested
// version and returns it without persisting. The `name` field is accepted but unused.
func (r *router) createSnapshot(w http.ResponseWriter, req *http.Request) {
	var body CreateSnapshotRequest
	if err := json.NewDecoder(req.Body).Decode(&body); err != nil {
		writeErr(w, http.StatusBadRequest, "invalid body")
		return
	}
	if body.DocID == "" {
		writeErr(w, http.StatusBadRequest, "doc_id required")
		return
	}
	state, err := r.store.MaterializeAt(req.Context(), body.DocID, body.Version)
	if err != nil {
		r.fail(w, "snapshot materialize failed", err, "doc", body.DocID, "version", body.Version)
		return
	}
	writeJSON(w, http.StatusOK, DocumentResponse{
		ID:        body.DocID,
		Timestamp: time.Now().UTC().Format(time.RFC3339),
		Updates:   state,
		Version:   body.Version,
	})
}

func (r *router) copyDocument(w http.ResponseWriter, req *http.Request) {
	dst := req.PathValue("id")
	src := req.PathValue("source")
	if err := r.store.Copy(req.Context(), dst, src); err != nil {
		r.fail(w, "copy failed", err, "dst", dst, "src", src)
		return
	}
	writeJSON(w, http.StatusOK, map[string]string{"status": "ok"})
}

func (r *router) importDocument(w http.ResponseWriter, req *http.Request) {
	id := req.PathValue("id")
	var body ImportRequest
	if err := json.NewDecoder(req.Body).Decode(&body); err != nil {
		writeErr(w, http.StatusBadRequest, "invalid body")
		return
	}
	version, err := r.store.Import(req.Context(), id, body.Data)
	if err != nil {
		r.fail(w, "import failed", err, "doc", id)
		return
	}
	writeJSON(w, http.StatusOK, map[string]any{"status": "ok", "version": version})
}

func (r *router) cleanupUpdates(w http.ResponseWriter, req *http.Request) {
	id := req.PathValue("id")
	deleted, err := r.store.Compact(req.Context(), id, keepUpdates)
	if err != nil {
		r.fail(w, "cleanup failed", err, "doc", id)
		return
	}
	writeJSON(w, http.StatusOK, map[string]any{"status": "ok", "deleted": deleted})
}

func (r *router) deleteDocument(w http.ResponseWriter, req *http.Request) {
	id := req.PathValue("id")
	if err := r.store.Delete(req.Context(), id); err != nil {
		r.fail(w, "delete failed", err, "doc", id)
		return
	}
	w.WriteHeader(http.StatusNoContent)
}

func (r *router) adminCleanup(w http.ResponseWriter, req *http.Request) {
	deleted, err := r.store.CleanupAll(req.Context(), keepUpdates)
	if err != nil {
		r.fail(w, "admin cleanup failed", err)
		return
	}
	writeJSON(w, http.StatusOK, map[string]any{"status": "ok", "deleted": deleted})
}
