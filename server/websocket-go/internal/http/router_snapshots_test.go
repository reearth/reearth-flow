package http

import (
	"encoding/json"
	"errors"
	"net/http"
	"strings"
	"testing"
	"time"
)

var errUnexpected = errors.New("unexpected store failure")

// TestGetSnapshots_ReturnsNewestFirstWithLabels: the router must pass the
// store's snapshot list through to JSON verbatim, preserving order and label.
// (Ordering itself is the store's contract — see adapt_snapshots_test.go for
// a test that exercises that against a real SnapshotStore implementation.)
func TestGetSnapshots_ReturnsNewestFirstWithLabels(t *testing.T) {
	newer := time.Date(2026, 7, 30, 12, 0, 0, 0, time.UTC)
	older := newer.Add(-time.Hour)
	store := &fakeStore{
		snapshots: []SnapshotItem{
			{ID: 2, Label: "second", Timestamp: newer.Format(time.RFC3339), Size: 42},
			{ID: 1, Label: "first", Timestamp: older.Format(time.RFC3339), Size: 10},
		},
	}
	h := newTestRouter(store)
	rec := do(t, h, "GET", "/api/document/proj1/snapshots", "")
	if rec.Code != http.StatusOK {
		t.Fatalf("status = %d, body=%s", rec.Code, rec.Body.String())
	}
	var got []SnapshotItem
	if err := json.Unmarshal(rec.Body.Bytes(), &got); err != nil {
		t.Fatalf("decode: %v", err)
	}
	if len(got) != 2 {
		t.Fatalf("len = %d, want 2", len(got))
	}
	if got[0].ID != 2 || got[0].Label != "second" {
		t.Fatalf("first entry = %+v, want the newest (id=2, label=second)", got[0])
	}
	if got[1].ID != 1 || got[1].Label != "first" {
		t.Fatalf("second entry = %+v, want the older one (id=1, label=first)", got[1])
	}
	if got[0].Timestamp == "" {
		t.Fatal("Timestamp must be populated (RFC3339)")
	}
	if got[0].Size != 42 {
		t.Fatalf("Size = %d, want 42 (field must round-trip)", got[0].Size)
	}
}

// TestGetSnapshots_UnknownRoomIsEmptyList: a room with no snapshots (or that
// doesn't exist) must render as an empty JSON array, never null and never an error.
func TestGetSnapshots_UnknownRoomIsEmptyList(t *testing.T) {
	store := &fakeStore{snapshots: nil}
	h := newTestRouter(store)
	rec := do(t, h, "GET", "/api/document/unknown-room/snapshots", "")
	if rec.Code != http.StatusOK {
		t.Fatalf("status = %d, want 200", rec.Code)
	}
	got := rec.Body.String()
	if got != "[]\n" && got != "[]" {
		t.Fatalf("body = %q, want an empty JSON array, not null", got)
	}
}

// TestGetSnapshots_StoreErrorIs500: a genuine store failure (not "no
// snapshots") must surface as a logged 500, matching every other list-style
// handler in this router.
func TestGetSnapshots_StoreErrorIs500(t *testing.T) {
	store := &fakeStore{snapshotErr: errUnexpected}
	h := newTestRouter(store)
	rec := do(t, h, "GET", "/api/document/proj1/snapshots", "")
	if rec.Code != http.StatusInternalServerError {
		t.Fatalf("status = %d, want 500, body=%s", rec.Code, rec.Body.String())
	}
}

// TestGetSnapshotState_BadIDIs400: a non-numeric snapshot id must not reach
// the store at all.
func TestGetSnapshotState_BadIDIs400(t *testing.T) {
	store := &fakeStore{}
	h := newTestRouter(store)
	rec := do(t, h, "GET", "/api/document/proj1/snapshots/not-a-number", "")
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("status = %d, want 400, body=%s", rec.Code, rec.Body.String())
	}
}

// TestGetSnapshotState_UnknownIDIs404: an id the store doesn't recognize
// (persistence.ErrSnapshotNotFound) must map to 404, not a generic 500.
func TestGetSnapshotState_UnknownIDIs404(t *testing.T) {
	store := &fakeStore{snapshotState: map[int64][]byte{}}
	h := newTestRouter(store)
	rec := do(t, h, "GET", "/api/document/proj1/snapshots/999", "")
	if rec.Code != http.StatusNotFound {
		t.Fatalf("status = %d, want 404, body=%s", rec.Code, rec.Body.String())
	}
}

// TestGetSnapshotState_ReturnsState: a known id returns its state as the
// document's int-array update payload.
func TestGetSnapshotState_ReturnsState(t *testing.T) {
	store := &fakeStore{snapshotState: map[int64][]byte{7: {1, 2, 3}}}
	h := newTestRouter(store)
	rec := do(t, h, "GET", "/api/document/proj1/snapshots/7", "")
	if rec.Code != http.StatusOK {
		t.Fatalf("status = %d, body=%s", rec.Code, rec.Body.String())
	}
	var resp SnapshotStateResponse
	if err := json.Unmarshal(rec.Body.Bytes(), &resp); err != nil {
		t.Fatalf("decode: %v", err)
	}
	if string(resp.Updates) != "\x01\x02\x03" {
		t.Fatalf("Updates = %v, want the snapshot's state bytes", []byte(resp.Updates))
	}
	if resp.SnapshotID != 7 {
		t.Fatalf("SnapshotID = %d, want 7", resp.SnapshotID)
	}
}

// TestPostSnapshot_SavesWithLabel: posting a label must reach the store
// unchanged and the created id must round-trip in the response.
func TestPostSnapshot_SavesWithLabel(t *testing.T) {
	store := &fakeStore{saveSnapshotID: 5}
	h := newTestRouter(store)
	rec := do(t, h, "POST", "/api/document/proj1/snapshots", `{"label":"before-migration"}`)
	if rec.Code != http.StatusOK {
		t.Fatalf("status = %d, body=%s", rec.Code, rec.Body.String())
	}
	if store.savedRoom != "proj1" || store.savedLabel != "before-migration" {
		t.Fatalf("SaveSnapshot called with room=%q label=%q, want proj1/before-migration", store.savedRoom, store.savedLabel)
	}
	var resp SnapshotItem
	if err := json.Unmarshal(rec.Body.Bytes(), &resp); err != nil {
		t.Fatalf("decode: %v", err)
	}
	if resp.ID != 5 || resp.Label != "before-migration" {
		t.Fatalf("resp = %+v", resp)
	}
}

// TestPostSnapshot_NoBodyIsOptionalLabel: an empty POST body (no label) must
// not be treated as a bad request; label is optional.
func TestPostSnapshot_NoBodyIsOptionalLabel(t *testing.T) {
	store := &fakeStore{saveSnapshotID: 1}
	h := newTestRouter(store)
	rec := do(t, h, "POST", "/api/document/proj1/snapshots", "")
	if rec.Code != http.StatusOK {
		t.Fatalf("status = %d, want 200 (label optional), body=%s", rec.Code, rec.Body.String())
	}
	if store.savedLabel != "" {
		t.Fatalf("savedLabel = %q, want empty", store.savedLabel)
	}
}

// TestPostSnapshot_StoreErrorIs500: a save failure must be a logged 500, not
// silently swallowed.
func TestPostSnapshot_StoreErrorIs500(t *testing.T) {
	store := &fakeStore{saveSnapshotErr: errUnexpected}
	h := newTestRouter(store)
	rec := do(t, h, "POST", "/api/document/proj1/snapshots", `{"label":"x"}`)
	if rec.Code != http.StatusInternalServerError {
		t.Fatalf("status = %d, want 500, body=%s", rec.Code, rec.Body.String())
	}
}

// TestPostSnapshot_EmptyDocumentIsConflict: SaveSnapshot reports "nothing to
// version" as (0, nil), and ids start at 1, so a 200 with id 0 would advertise a
// snapshot that does not exist. Reachable by saving a version on a fresh project.
func TestPostSnapshot_EmptyDocumentIsConflict(t *testing.T) {
	store := &fakeStore{saveSnapshotID: 0} // the (0, nil) no-op
	h := newTestRouter(store)
	rec := do(t, h, "POST", "/api/document/proj1/snapshots", `{"label":"x"}`)
	if rec.Code != http.StatusConflict {
		t.Fatalf("status = %d, want 409, body=%s", rec.Code, rec.Body.String())
	}
	// The response must not carry a snapshot id at all.
	var resp SnapshotItem
	if err := json.Unmarshal(rec.Body.Bytes(), &resp); err == nil && resp.ID != 0 {
		t.Fatalf("a snapshot id leaked into the refusal: %+v", resp)
	}
}

// TestPostSnapshot_MalformedBodyIs400: the label is optional, so an ABSENT body
// is fine, but a malformed one is not. Silently ignoring the decode error drops
// the label the user typed and still reports success, leaving them with an
// unnamed version and no indication anything went wrong.
func TestPostSnapshot_MalformedBodyIs400(t *testing.T) {
	for _, body := range []string{`{"label":`, `{"label":123}`, `not json`} {
		store := &fakeStore{saveSnapshotID: 7}
		h := newTestRouter(store)
		rec := do(t, h, "POST", "/api/document/proj1/snapshots", body)
		if rec.Code != http.StatusBadRequest {
			t.Fatalf("body %q: status = %d, want 400, body=%s", body, rec.Code, rec.Body.String())
		}
		if store.savedRoom != "" {
			t.Fatalf("body %q: SaveSnapshot was called despite a bad body", body)
		}
	}
}

// TestPostSnapshot_OversizedLabelIs400: the label lands in GCS custom object
// metadata, which caps at 8 KiB per object. Without a bound at the boundary an
// oversized label fails deep in the storage layer as an opaque 500 — after the
// id counter has already been consumed, burning an id per attempt.
func TestPostSnapshot_OversizedLabelIs400(t *testing.T) {
	long := strings.Repeat("a", maxSnapshotLabel+1)
	store := &fakeStore{saveSnapshotID: 7}
	h := newTestRouter(store)
	rec := do(t, h, "POST", "/api/document/proj1/snapshots", `{"label":"`+long+`"}`)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("status = %d, want 400, body=%s", rec.Code, rec.Body.String())
	}
	if store.savedRoom != "" {
		t.Fatal("SaveSnapshot was called with an oversized label")
	}
	// A label exactly at the limit must still be accepted.
	store2 := &fakeStore{saveSnapshotID: 8}
	h2 := newTestRouter(store2)
	rec2 := do(t, h2, "POST", "/api/document/proj1/snapshots", `{"label":"`+strings.Repeat("a", maxSnapshotLabel)+`"}`)
	if rec2.Code != http.StatusOK {
		t.Fatalf("label at the limit: status = %d, want 200, body=%s", rec2.Code, rec2.Body.String())
	}
}

// TestSnapshots_UnsupportedBackendIs501: an unsupported backend is a deployment
// fact, not a server fault, so it must not be a 500 or a misleading 404.
func TestSnapshots_UnsupportedBackendIs501(t *testing.T) {
	st := NewStoreAdapter(StoreAdapterDeps{P: &memPersist{}}) // no SnapshotStore
	h := newTestRouter(st)

	rec := do(t, h, "POST", "/api/document/proj1/snapshots", `{"label":"x"}`)
	if rec.Code != http.StatusNotImplemented {
		t.Fatalf("POST status = %d, want 501, body=%s", rec.Code, rec.Body.String())
	}
	rec = do(t, h, "GET", "/api/document/proj1/snapshots/1", "")
	if rec.Code != http.StatusNotImplemented {
		t.Fatalf("GET state status = %d, want 501, body=%s", rec.Code, rec.Body.String())
	}
	// Listing stays lenient on purpose: the panel renders empty, not an error.
	rec = do(t, h, "GET", "/api/document/proj1/snapshots", "")
	if rec.Code != http.StatusOK {
		t.Fatalf("GET list status = %d, want 200 (empty list), body=%s", rec.Code, rec.Body.String())
	}
}

// TestGetSnapshotState_NonPositiveIDIs400: ids are allocated from 1, so 0 and
// negatives can never match. Rejecting them here avoids a misleading 404 that
// reads as "your snapshot was deleted".
func TestGetSnapshotState_NonPositiveIDIs400(t *testing.T) {
	for _, sid := range []string{"0", "-1"} {
		store := &fakeStore{snapshotState: map[int64][]byte{1: {1, 2, 3}}}
		h := newTestRouter(store)
		rec := do(t, h, "GET", "/api/document/proj1/snapshots/"+sid, "")
		if rec.Code != http.StatusBadRequest {
			t.Fatalf("sid %s: status = %d, want 400, body=%s", sid, rec.Code, rec.Body.String())
		}
	}
}

// TestGetSnapshotState_ResponseCarriesNoUpdateLogVersion pins the response shape.
// The old one shipped `"version":0`, and rollback with 0 prunes the whole log.
func TestGetSnapshotState_ResponseCarriesNoUpdateLogVersion(t *testing.T) {
	store := &fakeStore{snapshotState: map[int64][]byte{7: {1, 2, 3}}}
	h := newTestRouter(store)
	rec := do(t, h, "GET", "/api/document/proj1/snapshots/7", "")
	if rec.Code != http.StatusOK {
		t.Fatalf("status = %d, body=%s", rec.Code, rec.Body.String())
	}
	var raw map[string]any
	if err := json.Unmarshal(rec.Body.Bytes(), &raw); err != nil {
		t.Fatalf("decode: %v", err)
	}
	if _, present := raw["version"]; present {
		t.Fatalf("response must not carry a `version` field (update-log clock space): %v", raw)
	}
	if got := raw["snapshot_id"]; got != float64(7) {
		t.Fatalf("snapshot_id = %v, want 7", got)
	}
}
