package http

import (
	"encoding/json"
	"errors"
	"net/http"
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
	var resp DocumentResponse
	if err := json.Unmarshal(rec.Body.Bytes(), &resp); err != nil {
		t.Fatalf("decode: %v", err)
	}
	if string(resp.Updates) != "\x01\x02\x03" {
		t.Fatalf("Updates = %v, want the snapshot's state bytes", []byte(resp.Updates))
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
