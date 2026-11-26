package http

import (
	"bytes"
	"context"
	"encoding/json"
	"errors"
	"io"
	"log/slog"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"
	"time"

	"github.com/reearth/ygo/persistence"
)

// fakeStore implements DocStore for router tests, recording calls.
type fakeStore struct {
	loadUpdate   []byte
	loadVersion  uint64
	versions     []VersionInfo
	updateAt     map[uint64][]byte
	materialized []byte
	// materializeErr, when set, is returned by MaterializeAt instead of materialized.
	materializeErr error
	// unrebuildable marks versions RebuildableVersions reports as not rebuildable.
	unrebuildable map[uint64]bool
	// rebuildableErr, if set, is returned by RebuildableVersions.
	rebuildableErr error
	rolledBack     bool
	prunedTarget   uint64
	flushed        bool
	// captureSnapshotCalled must stay false: the snapshot endpoint is read-only.
	captureSnapshotCalled bool
	copied                [2]string // [dst, src]
	imported              []byte
	cleanupKeep           int
	deleted               bool
	cleanupAll            bool
	notFound              bool
	loadErr               error
	onPrune               func()
	// snapshots is returned verbatim by ListSnapshots.
	snapshots []SnapshotItem
	// snapshotErr, if set, is returned by ListSnapshots.
	snapshotErr error
	// snapshotState maps snapshot id -> state; a missing id yields ErrSnapshotNotFound.
	snapshotState map[int64][]byte
	// saveSnapshotID/saveSnapshotErr control SaveSnapshot's return value.
	saveSnapshotID  int64
	saveSnapshotErr error
	// savedRoom/savedLabel record the last SaveSnapshot call's arguments.
	savedRoom  string
	savedLabel string
}

func (f *fakeStore) Load(ctx context.Context, room string) (LoadResult, error) {
	if f.loadErr != nil {
		return LoadResult{}, f.loadErr
	}
	if f.notFound {
		return LoadResult{}, ErrNotFound
	}
	return LoadResult{Update: f.loadUpdate, Version: f.loadVersion, UpdatedAt: time.Unix(0, 0).UTC()}, nil
}
func (f *fakeStore) ListVersions(ctx context.Context, room string) ([]VersionInfo, error) {
	return f.versions, nil
}
func (f *fakeStore) GetUpdate(ctx context.Context, room string, v uint64) ([]byte, VersionInfo, bool, error) {
	b, ok := f.updateAt[v]
	if !ok {
		return nil, VersionInfo{}, false, nil
	}
	return b, VersionInfo{Version: v, UpdatedAt: time.Unix(0, 0).UTC()}, true, nil
}
func (f *fakeStore) MaterializeAt(ctx context.Context, room string, v uint64) ([]byte, error) {
	if f.materializeErr != nil {
		return nil, f.materializeErr
	}
	return f.materialized, nil
}
func (f *fakeStore) RebuildableVersions(ctx context.Context, room string, versions []uint64) (map[uint64]bool, error) {
	if f.rebuildableErr != nil {
		return nil, f.rebuildableErr
	}
	out := make(map[uint64]bool, len(versions))
	for _, v := range versions {
		out[v] = !f.unrebuildable[v]
	}
	return out, nil
}
func (f *fakeStore) PruneAfter(ctx context.Context, room string, target uint64, rolledBack []byte) error {
	f.rolledBack = true
	f.prunedTarget = target
	if f.onPrune != nil {
		f.onPrune()
	}
	return nil
}
func (f *fakeStore) Flush(ctx context.Context, room string) error { f.flushed = true; return nil }
func (f *fakeStore) Copy(ctx context.Context, dst, src string) error {
	f.copied = [2]string{dst, src}
	return nil
}
func (f *fakeStore) Import(ctx context.Context, room string, data []byte) (uint64, error) {
	f.imported = data
	return 1, nil
}
func (f *fakeStore) Compact(ctx context.Context, room string, keep int) (int, error) {
	f.cleanupKeep = keep
	return 0, nil
}
func (f *fakeStore) Delete(ctx context.Context, room string) error { f.deleted = true; return nil }
func (f *fakeStore) CleanupAll(ctx context.Context, keep int) (int, error) {
	f.cleanupAll = true
	f.cleanupKeep = keep
	return 0, nil
}
func (f *fakeStore) ListSnapshots(ctx context.Context, room string) ([]SnapshotItem, error) {
	if f.snapshotErr != nil {
		return nil, f.snapshotErr
	}
	return f.snapshots, nil
}
func (f *fakeStore) GetSnapshotState(ctx context.Context, room string, id int64) ([]byte, error) {
	b, ok := f.snapshotState[id]
	if !ok {
		return nil, persistence.ErrSnapshotNotFound
	}
	return b, nil
}
func (f *fakeStore) SaveSnapshot(ctx context.Context, room, label string) (int64, error) {
	f.savedRoom = room
	f.savedLabel = label
	return f.saveSnapshotID, f.saveSnapshotErr
}

func newTestRouter(store DocStore) http.Handler {
	return NewRouter(Deps{Store: store})
}

func do(t *testing.T, h http.Handler, method, path, body string) *httptest.ResponseRecorder {
	t.Helper()
	var r io.Reader
	if body != "" {
		r = strings.NewReader(body)
	}
	req := httptest.NewRequest(method, path, r)
	rec := httptest.NewRecorder()
	h.ServeHTTP(rec, req)
	return rec
}

func TestGetLatest(t *testing.T) {
	store := &fakeStore{loadUpdate: []byte{1, 2, 3}, loadVersion: 7}
	h := newTestRouter(store)
	rec := do(t, h, "GET", "/api/document/proj1", "")
	if rec.Code != 200 {
		t.Fatalf("status = %d, body=%s", rec.Code, rec.Body.String())
	}
	var resp DocumentResponse
	if err := json.Unmarshal(rec.Body.Bytes(), &resp); err != nil {
		t.Fatalf("decode: %v", err)
	}
	if resp.ID != "proj1" || resp.Version != 7 || string(resp.Updates) != "\x01\x02\x03" {
		t.Fatalf("resp = %+v", resp)
	}
	if rec.Body.String() == "" || !strings.Contains(rec.Body.String(), "[1,2,3]") {
		t.Fatalf("updates not int-array: %s", rec.Body.String())
	}
}

// TestServerErrorIsLogged: a 500 from the store must log the underlying error
// (with the doc id) at ERROR, not just return a generic message. This is the
// regression guard for "500 with nothing in the logs".
func TestServerErrorIsLogged(t *testing.T) {
	var buf bytes.Buffer
	log := slog.New(slog.NewJSONHandler(&buf, &slog.HandlerOptions{Level: slog.LevelDebug}))
	store := &fakeStore{loadErr: errors.New("gcs permission denied")}
	h := NewRouter(Deps{Store: store, Logger: log})

	rec := do(t, h, "GET", "/api/document/proj1", "")
	if rec.Code != http.StatusInternalServerError {
		t.Fatalf("status = %d, want 500", rec.Code)
	}
	out := buf.String()
	if !strings.Contains(out, "gcs permission denied") {
		t.Fatalf("underlying error not logged:\n%s", out)
	}
	if !strings.Contains(out, "proj1") {
		t.Fatalf("doc id not logged:\n%s", out)
	}
	if !strings.Contains(out, `"level":"ERROR"`) {
		t.Fatalf("500 not logged at ERROR level:\n%s", out)
	}
}

func TestGetLatestNotFound(t *testing.T) {
	h := newTestRouter(&fakeStore{notFound: true})
	rec := do(t, h, "GET", "/api/document/proj1", "")
	if rec.Code != 404 {
		t.Fatalf("status = %d, want 404", rec.Code)
	}
}

func TestGetHistory(t *testing.T) {
	store := &fakeStore{
		versions: []VersionInfo{{Version: 2, UpdatedAt: time.Unix(0, 0).UTC()}, {Version: 1, UpdatedAt: time.Unix(0, 0).UTC()}},
		updateAt: map[uint64][]byte{1: {9}, 2: {8}},
	}
	h := newTestRouter(store)
	rec := do(t, h, "GET", "/api/document/proj1/history", "")
	if rec.Code != 200 {
		t.Fatalf("status = %d, body=%s", rec.Code, rec.Body.String())
	}
	var items []DocumentResponse
	if err := json.Unmarshal(rec.Body.Bytes(), &items); err != nil {
		t.Fatalf("decode: %v", err)
	}
	if len(items) != 2 || items[0].Version != 2 {
		t.Fatalf("items = %+v", items)
	}
}

// TestGetHistory_ListsOnlyRebuildableVersions: the recovery view used to list
// entries the store could not rebuild, and reverting to one wiped the project.
// Both lists now carry only what preview and rollback will accept.
func TestGetHistory_ListsOnlyRebuildableVersions(t *testing.T) {
	t.Parallel()

	newStore := func() *fakeStore {
		return &fakeStore{
			versions: []VersionInfo{
				{Version: 3, UpdatedAt: time.Unix(0, 0).UTC()},
				{Version: 2, UpdatedAt: time.Unix(0, 0).UTC()},
				{Version: 1, UpdatedAt: time.Unix(0, 0).UTC()},
			},
			updateAt:      map[uint64][]byte{1: {9}, 2: {8}, 3: {7}},
			unrebuildable: map[uint64]bool{1: true, 2: true},
		}
	}

	t.Run("history", func(t *testing.T) {
		t.Parallel()
		rec := do(t, newTestRouter(newStore()), "GET", "/api/document/proj1/history", "")
		if rec.Code != 200 {
			t.Fatalf("status = %d, body=%s", rec.Code, rec.Body.String())
		}
		var items []DocumentResponse
		if err := json.Unmarshal(rec.Body.Bytes(), &items); err != nil {
			t.Fatalf("decode: %v", err)
		}
		if len(items) != 1 || items[0].Version != 3 {
			t.Errorf("items = %+v, want only version 3", items)
		}
	})

	t.Run("metadata", func(t *testing.T) {
		t.Parallel()
		rec := do(t, newTestRouter(newStore()), "GET", "/api/document/proj1/history/metadata", "")
		if rec.Code != 200 {
			t.Fatalf("status = %d, body=%s", rec.Code, rec.Body.String())
		}
		var items []HistoryMetadataItem
		if err := json.Unmarshal(rec.Body.Bytes(), &items); err != nil {
			t.Fatalf("decode: %v", err)
		}
		if len(items) != 1 || items[0].Version != 3 {
			t.Errorf("items = %+v, want only version 3", items)
		}
	})
}

func TestGetHistoryMetadata(t *testing.T) {
	store := &fakeStore{versions: []VersionInfo{{Version: 5, UpdatedAt: time.Unix(0, 0).UTC()}}}
	h := newTestRouter(store)
	rec := do(t, h, "GET", "/api/document/proj1/history/metadata", "")
	if rec.Code != 200 {
		t.Fatalf("status = %d", rec.Code)
	}
	var items []HistoryMetadataItem
	if err := json.Unmarshal(rec.Body.Bytes(), &items); err != nil {
		t.Fatalf("decode: %v", err)
	}
	if len(items) != 1 || items[0].Version != 5 {
		t.Fatalf("items = %+v", items)
	}
}

func TestGetHistoryByVersion(t *testing.T) {
	store := &fakeStore{updateAt: map[uint64][]byte{3: {7, 7}}}
	h := newTestRouter(store)
	rec := do(t, h, "GET", "/api/document/proj1/history/version/3", "")
	if rec.Code != 200 {
		t.Fatalf("status = %d", rec.Code)
	}
	var resp DocumentResponse
	if err := json.Unmarshal(rec.Body.Bytes(), &resp); err != nil {
		t.Fatalf("decode: %v", err)
	}
	if resp.Version != 3 || string(resp.Updates) != "\x07\x07" {
		t.Fatalf("resp = %+v", resp)
	}
}

func TestGetHistoryByVersionNotFound(t *testing.T) {
	h := newTestRouter(&fakeStore{updateAt: map[uint64][]byte{}})
	rec := do(t, h, "GET", "/api/document/proj1/history/version/99", "")
	if rec.Code != 404 {
		t.Fatalf("status = %d, want 404", rec.Code)
	}
}

func TestGetHistoryByVersionBadVersion(t *testing.T) {
	h := newTestRouter(&fakeStore{})
	rec := do(t, h, "GET", "/api/document/proj1/history/version/notanint", "")
	if rec.Code != 400 {
		t.Fatalf("status = %d, want 400", rec.Code)
	}
}

func TestRollback(t *testing.T) {
	store := &fakeStore{materialized: []byte{1}}
	h := newTestRouter(store)
	rec := do(t, h, "POST", "/api/document/proj1/rollback", `{"doc_id":"proj1","version":4}`)
	if rec.Code != 200 {
		t.Fatalf("status = %d, body=%s", rec.Code, rec.Body.String())
	}
	if !store.rolledBack || store.prunedTarget != 4 {
		t.Fatalf("rollback not invoked with target 4: %+v", store)
	}
}

// TestRollbackReturnsMaterializedDocument: the rollback handler must return the
// full DocumentResponse (id, updates, version, timestamp) like the Rust server,
// not a bare {status,version}. The API-server client decodes `updates` into the
// rolled-back document; omitting it silently no-ops the user's rollback in the UI.
func TestRollbackReturnsMaterializedDocument(t *testing.T) {
	store := &fakeStore{materialized: []byte{7, 8, 9}}
	h := newTestRouter(store)
	rec := do(t, h, "POST", "/api/document/proj1/rollback", `{"doc_id":"proj1","version":4}`)
	if rec.Code != 200 {
		t.Fatalf("status = %d, body=%s", rec.Code, rec.Body.String())
	}
	var resp DocumentResponse
	if err := json.Unmarshal(rec.Body.Bytes(), &resp); err != nil {
		t.Fatalf("decode: %v (body=%s)", err, rec.Body.String())
	}
	if resp.ID != "proj1" {
		t.Errorf("ID = %q, want proj1", resp.ID)
	}
	if resp.Version != 4 {
		t.Errorf("Version = %d, want 4", resp.Version)
	}
	if string(resp.Updates) != "\x07\x08\x09" {
		t.Errorf("Updates = %v, want the materialized rolled-back bytes", []byte(resp.Updates))
	}
	if resp.Timestamp == "" {
		t.Errorf("Timestamp is empty; client falls back to time.Now() and loses the rolled-back timestamp")
	}
}

func TestFlush(t *testing.T) {
	store := &fakeStore{}
	h := newTestRouter(store)
	rec := do(t, h, "POST", "/api/document/proj1/flush", "")
	if rec.Code != 200 || !store.flushed {
		t.Fatalf("status=%d flushed=%v", rec.Code, store.flushed)
	}
}

// TestCreateSnapshotIsReadOnly: the snapshot endpoint returns the materialized
// state at the requested version and must not persist anything.
func TestCreateSnapshotIsReadOnly(t *testing.T) {
	store := &fakeStore{materialized: []byte{4, 5, 6}, loadVersion: 9}
	h := newTestRouter(store)
	rec := do(t, h, "POST", "/api/document/snapshot", `{"doc_id":"proj1","name":"snap","version":9}`)
	if rec.Code != 200 {
		t.Fatalf("status=%d body=%s", rec.Code, rec.Body.String())
	}
	if store.captureSnapshotCalled {
		t.Fatalf("snapshot endpoint persisted (CaptureSnapshot called) — must be read-only")
	}
	var resp DocumentResponse
	if err := json.Unmarshal(rec.Body.Bytes(), &resp); err != nil {
		t.Fatalf("decode: %v", err)
	}
	if resp.ID != "proj1" || resp.Version != 9 || string(resp.Updates) != "\x04\x05\x06" {
		t.Fatalf("resp = %+v", resp)
	}
}

func TestCopyDocument(t *testing.T) {
	store := &fakeStore{}
	h := newTestRouter(store)
	rec := do(t, h, "POST", "/api/document/dst1/src1/copy", "")
	if rec.Code != 200 {
		t.Fatalf("status=%d body=%s", rec.Code, rec.Body.String())
	}
	if store.copied != [2]string{"dst1", "src1"} {
		t.Fatalf("copied = %v", store.copied)
	}
}

func TestImportDocument(t *testing.T) {
	store := &fakeStore{}
	h := newTestRouter(store)
	rec := do(t, h, "POST", "/api/document/proj1/import", `{"data":[0,1,255]}`)
	if rec.Code != 200 {
		t.Fatalf("status=%d body=%s", rec.Code, rec.Body.String())
	}
	if string(store.imported) != "\x00\x01\xff" {
		t.Fatalf("imported = %v", store.imported)
	}
}

func TestCleanupUpdates(t *testing.T) {
	store := &fakeStore{}
	h := newTestRouter(store)
	rec := do(t, h, "POST", "/api/document/proj1/cleanup", "")
	// Was 10, now the retention shared with ygo's unload compaction: a cleanup
	// that kept fewer would discard entries the recovery view lists.
	if rec.Code != 200 || store.cleanupKeep != defaultKeepUpdates {
		t.Fatalf("status=%d keep=%d, want keep=%d", rec.Code, store.cleanupKeep, defaultKeepUpdates)
	}
}

func TestDeleteDocument(t *testing.T) {
	store := &fakeStore{}
	h := newTestRouter(store)
	rec := do(t, h, "DELETE", "/api/document/proj1", "")
	if rec.Code != 204 && rec.Code != 200 {
		t.Fatalf("status = %d, want 204 or 200", rec.Code)
	}
	if !store.deleted {
		t.Fatalf("not deleted")
	}
}

func TestAdminCleanup(t *testing.T) {
	store := &fakeStore{}
	h := newTestRouter(store)
	rec := do(t, h, "POST", "/api/admin/cleanup", "")
	if rec.Code != 200 || !store.cleanupAll || store.cleanupKeep != defaultKeepUpdates {
		t.Fatalf("status=%d all=%v keep=%d, want keep=%d", rec.Code, store.cleanupAll, store.cleanupKeep, defaultKeepUpdates)
	}
}

// TestRollbackSignalsLiveRoom: the rollback handler toggles rollbackInProgress
// via the signaler, with set before prune and clear after.
func TestRollbackSignalsLiveRoom(t *testing.T) {
	store := &fakeStore{materialized: []byte{1}}
	sig := &fakeSignaler{}
	store.onPrune = func() { sig.pruneSeen = true }
	h := NewRouter(Deps{Store: store, Signaler: sig})
	rec := do(t, h, "POST", "/api/document/proj1/rollback", `{"doc_id":"proj1","version":2}`)
	if rec.Code != 200 {
		t.Fatalf("status=%d body=%s", rec.Code, rec.Body.String())
	}
	if sig.room != "proj1" || !sig.set || !sig.cleared {
		t.Fatalf("signaler not toggled: %+v", sig)
	}
	if !sig.setBeforePrune || !sig.clearAfterPrune {
		t.Fatalf("ordering wrong: set-before-prune=%v clear-after-prune=%v", sig.setBeforePrune, sig.clearAfterPrune)
	}
}

type fakeSignaler struct {
	room            string
	set             bool
	cleared         bool
	pruneSeen       bool
	setBeforePrune  bool
	clearAfterPrune bool
	// clearCtxErr records ctx.Err() seen during the clear call.
	clearCtxErr error
}

func (f *fakeSignaler) SignalLock(context.Context, string, bool, time.Time) error { return nil }

func (f *fakeSignaler) SignalRollback(ctx context.Context, room string, inProgress bool) error {
	f.room = room
	if inProgress {
		f.set = true
		f.setBeforePrune = !f.pruneSeen
	} else {
		f.cleared = true
		f.clearAfterPrune = f.pruneSeen
		f.clearCtxErr = ctx.Err()
	}
	return nil
}

// TestRollbackClearSurvivesClientDisconnect: if the client disconnects
// mid-rollback, the deferred clear must still fire on a non-cancelled context.
func TestRollbackClearSurvivesClientDisconnect(t *testing.T) {
	ctx, cancel := context.WithCancel(context.Background())
	store := &fakeStore{materialized: []byte{1}}
	sig := &fakeSignaler{}
	store.onPrune = func() {
		sig.pruneSeen = true
		cancel()
	}
	h := NewRouter(Deps{Store: store, Signaler: sig})

	req := httptest.NewRequest("POST", "/api/document/proj1/rollback", strings.NewReader(`{"doc_id":"proj1","version":2}`)).WithContext(ctx)
	rec := httptest.NewRecorder()
	h.ServeHTTP(rec, req)

	if !sig.cleared {
		t.Fatalf("deferred clear did not fire after client disconnect")
	}
	if sig.clearCtxErr != nil {
		t.Fatalf("clear ran on a cancelled context (err=%v) — unhiding the canvas must not depend on client liveness", sig.clearCtxErr)
	}
}

// refusedRebuild models a store that cannot reconstruct a version because its
// update log was compacted past it. It reports itself by method, the way the
// GCS adapter's error does.
type refusedRebuild struct{}

func (refusedRebuild) Error() string        { return "the update log was compacted past this version" }
func (refusedRebuild) NotRebuildable() bool { return true }

// rollbackRunning is a store error for a rollback whose writer holds the lock.
type rollbackRunning struct{}

func (rollbackRunning) Error() string            { return "a rollback of this room is in progress" }
func (rollbackRunning) RollbackInProgress() bool { return true }

// TestRecoveryRoutesAskForARetryDuringARollback: the state is unreadable only
// until the rollback's writer finishes, so the routes answer 503, not the 500 of
// a failure or the 409 of a final refusal.
func TestRecoveryRoutesAskForARetryDuringARollback(t *testing.T) {
	t.Parallel()

	t.Run("rollback", func(t *testing.T) {
		t.Parallel()
		store := &fakeStore{materializeErr: rollbackRunning{}}
		rec := do(t, newTestRouter(store), "POST", "/api/document/proj1/rollback", `{"doc_id":"proj1","version":4}`)
		if rec.Code != http.StatusServiceUnavailable {
			t.Errorf("status = %d, want %d", rec.Code, http.StatusServiceUnavailable)
		}
		if store.rolledBack {
			t.Error("PruneAfter ran while a rollback was in progress")
		}
	})

	t.Run("snapshot", func(t *testing.T) {
		t.Parallel()
		store := &fakeStore{materializeErr: rollbackRunning{}}
		rec := do(t, newTestRouter(store), "POST", "/api/document/snapshot", `{"doc_id":"proj1","version":4}`)
		if rec.Code != http.StatusServiceUnavailable {
			t.Errorf("status = %d, want %d", rec.Code, http.StatusServiceUnavailable)
		}
	})

	t.Run("history", func(t *testing.T) {
		t.Parallel()
		store := &fakeStore{versions: []VersionInfo{{Version: 1}}, rebuildableErr: rollbackRunning{}}
		rec := do(t, newTestRouter(store), "GET", "/api/document/proj1/history/metadata", "")
		if rec.Code != http.StatusServiceUnavailable {
			t.Errorf("status = %d, want %d", rec.Code, http.StatusServiceUnavailable)
		}
	})
}

// TestRollbackRefusesAVersionTheStoreCannotRebuild: a rebuild the store refuses
// must never reach PruneAfter. PruneAfter replaces doc_v2 with what it is given
// and deletes the log above the target, so pruning on a fragment destroys the
// history above the version while restoring nothing. The caller gets 409, not
// 500: the stored history conflicts with the request, the server is fine.
func TestRollbackRefusesAVersionTheStoreCannotRebuild(t *testing.T) {
	t.Parallel()

	store := &fakeStore{materializeErr: refusedRebuild{}}
	h := newTestRouter(store)
	rec := do(t, h, "POST", "/api/document/proj1/rollback", `{"doc_id":"proj1","version":4}`)

	if rec.Code != http.StatusConflict {
		t.Errorf("status = %d, want %d", rec.Code, http.StatusConflict)
	}
	if store.rolledBack {
		t.Error("PruneAfter ran on a rebuild the store refused")
	}
}

// TestCreateSnapshotRefusesAVersionTheStoreCannotRebuild: the same refusal, so a
// fragment is not stored as if it were that version.
func TestCreateSnapshotRefusesAVersionTheStoreCannotRebuild(t *testing.T) {
	t.Parallel()

	store := &fakeStore{materializeErr: refusedRebuild{}}
	h := newTestRouter(store)
	rec := do(t, h, "POST", "/api/document/snapshot", `{"doc_id":"proj1","version":4}`)

	if rec.Code != http.StatusConflict {
		t.Errorf("status = %d, want %d", rec.Code, http.StatusConflict)
	}
}

// TestCleanupRoutesTrimToTheConfiguredRetention: the cleanup routes and ygo's
// unload compaction must trim a room's update log to the same number. They used
// to differ, 10 here against 300 on unload, so one admin cleanup would have
// discarded all but 10 of the entries the recovery view lists.
func TestCleanupRoutesTrimToTheConfiguredRetention(t *testing.T) {
	t.Parallel()

	t.Run("per document", func(t *testing.T) {
		t.Parallel()

		store := &fakeStore{}
		h := NewRouter(Deps{Store: store, KeepUpdates: 42})
		if rec := do(t, h, "POST", "/api/document/proj1/cleanup", ""); rec.Code != http.StatusOK {
			t.Fatalf("status = %d, want 200", rec.Code)
		}
		if store.cleanupKeep != 42 {
			t.Errorf("Compact keep = %d, want 42", store.cleanupKeep)
		}
	})

	t.Run("admin cleanup", func(t *testing.T) {
		t.Parallel()

		store := &fakeStore{}
		h := NewRouter(Deps{Store: store, KeepUpdates: 42})
		if rec := do(t, h, "POST", "/api/admin/cleanup", ""); rec.Code != http.StatusOK {
			t.Fatalf("status = %d, want 200", rec.Code)
		}
		if store.cleanupKeep != 42 {
			t.Errorf("CleanupAll keep = %d, want 42", store.cleanupKeep)
		}
	})

	t.Run("unset falls back to the shared default, never zero", func(t *testing.T) {
		t.Parallel()

		store := &fakeStore{}
		h := NewRouter(Deps{Store: store})
		if rec := do(t, h, "POST", "/api/document/proj1/cleanup", ""); rec.Code != http.StatusOK {
			t.Fatalf("status = %d, want 200", rec.Code)
		}
		// Zero would reach the store as "keep every update" and never trim.
		if store.cleanupKeep != defaultKeepUpdates {
			t.Errorf("Compact keep = %d, want %d", store.cleanupKeep, defaultKeepUpdates)
		}
	})
}
