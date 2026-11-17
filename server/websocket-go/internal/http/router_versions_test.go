package http

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"net/http"
	"strings"
	"testing"

	"github.com/reearth/reearth-flow/websocket-go/internal/versions"
)

type fakeVersioner struct {
	res       versions.Result
	err       error
	gotRoom   string
	gotLabel  string
	callCount int
}

func (f *fakeVersioner) SaveVersion(_ context.Context, room, label string) (versions.Result, error) {
	f.gotRoom, f.gotLabel = room, label
	f.callCount++
	return f.res, f.err
}

func versionRouter(v Versioner) http.Handler {
	return NewRouter(Deps{Store: &fakeStore{}, Versioner: v})
}

func decodeVersion(t *testing.T, body string) SaveVersionResponse {
	t.Helper()
	var got SaveVersionResponse
	if err := json.Unmarshal([]byte(body), &got); err != nil {
		t.Fatalf("decode %q: %v", body, err)
	}
	return got
}

func TestPostVersion_Saved(t *testing.T) {
	t.Parallel()

	v := &fakeVersioner{res: versions.Result{Saved: true, SnapshotID: 7}}
	rec := do(t, versionRouter(v), "POST", "/api/document/proj1:main/versions", `{"label":"Saved"}`)
	if rec.Code != http.StatusOK {
		t.Fatalf("status = %d, body=%s", rec.Code, rec.Body.String())
	}
	if got := decodeVersion(t, rec.Body.String()); !got.Saved || got.SnapshotID != 7 {
		t.Fatalf("response = %+v", got)
	}
	if v.gotRoom != "proj1" || v.gotLabel != "Saved" {
		t.Fatalf("saved room %q label %q; the room is the normalized doc id", v.gotRoom, v.gotLabel)
	}
}

func TestPostVersion_NothingChanged(t *testing.T) {
	t.Parallel()

	rec := do(t, versionRouter(&fakeVersioner{}), "POST", "/api/document/proj1/versions", `{"label":"Saved"}`)
	if rec.Code != http.StatusOK {
		t.Fatalf("status = %d", rec.Code)
	}
	if body := strings.TrimSpace(rec.Body.String()); body != `{"saved":false}` {
		t.Fatalf("body = %s, want {\"saved\":false}", body)
	}
}

func TestPostVersion_LabelRequired(t *testing.T) {
	t.Parallel()

	v := &fakeVersioner{}
	for _, body := range []string{`{}`, `{"label":""}`, `not json`, ""} {
		rec := do(t, versionRouter(v), "POST", "/api/document/proj1/versions", body)
		if rec.Code != http.StatusBadRequest {
			t.Errorf("body %q: status = %d, want 400", body, rec.Code)
		}
	}
	if v.callCount != 0 {
		t.Fatal("a bad request reached the versioner")
	}
}

func TestPostVersion_OversizedLabelIs400(t *testing.T) {
	t.Parallel()

	body := fmt.Sprintf(`{"label":%q}`, strings.Repeat("x", maxSnapshotLabel+1))
	rec := do(t, versionRouter(&fakeVersioner{}), "POST", "/api/document/proj1/versions", body)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("status = %d, want 400", rec.Code)
	}
}

func TestPostVersion_FailureIs500(t *testing.T) {
	t.Parallel()

	v := &fakeVersioner{err: errors.New("gcs down")}
	rec := do(t, versionRouter(v), "POST", "/api/document/proj1/versions", `{"label":"Saved"}`)
	if rec.Code != http.StatusInternalServerError {
		t.Fatalf("status = %d, want 500", rec.Code)
	}
}

func TestPostVersion_CheckpointFailureStillReportsTheVersion(t *testing.T) {
	t.Parallel()

	v := &fakeVersioner{res: versions.Result{Saved: true, SnapshotID: 3}, err: versions.ErrCheckpoint}
	rec := do(t, versionRouter(v), "POST", "/api/document/proj1/versions", `{"label":"Saved"}`)
	if rec.Code != http.StatusOK {
		t.Fatalf("status = %d, want 200: the version was saved", rec.Code)
	}
	if got := decodeVersion(t, rec.Body.String()); !got.Saved || got.SnapshotID != 3 {
		t.Fatalf("response = %+v", got)
	}
}

func TestPostVersion_WithoutVersionerIs501(t *testing.T) {
	t.Parallel()

	rec := do(t, newTestRouter(&fakeStore{}), "POST", "/api/document/proj1/versions", `{"label":"Saved"}`)
	if rec.Code != http.StatusNotImplemented {
		t.Fatalf("status = %d, want 501", rec.Code)
	}
}

func TestGetSnapshots_CarriesContributors(t *testing.T) {
	t.Parallel()

	store := &fakeStore{snapshots: []SnapshotItem{
		{ID: 2, Label: "auto", Contributors: []string{"bob", "alice"}, ContributorCount: 2, LastEditor: "bob"},
	}}
	rec := do(t, newTestRouter(store), "GET", "/api/document/proj1/snapshots", "")
	var got []map[string]any
	if err := json.Unmarshal(rec.Body.Bytes(), &got); err != nil {
		t.Fatal(err)
	}
	if len(got) != 1 || got[0]["lastEditor"] != "bob" || got[0]["contributorCount"] != float64(2) {
		t.Fatalf("body = %s", rec.Body.String())
	}
	if c, _ := got[0]["contributors"].([]any); len(c) != 2 {
		t.Fatalf("contributors = %v", got[0]["contributors"])
	}
}
