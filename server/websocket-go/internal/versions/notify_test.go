package versions

import (
	"context"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"sync/atomic"
	"testing"
	"time"
)

func noBackoff(int) time.Duration { return 0 }

func TestNotifier_PostsTheVersionWithTheSecret(t *testing.T) {
	t.Parallel()

	var got Saved
	var secret, path string
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		path, secret = r.URL.Path, r.Header.Get("X-API-Secret")
		_ = json.NewDecoder(r.Body).Decode(&got)
	}))
	defer srv.Close()

	n := NewNotifier(NotifierConfig{APIURL: srv.URL + "/", Secret: "s3cret", Backoff: noBackoff})
	want := Saved{
		ProjectID: "p1", VersionID: 7, Label: "Saved",
		SavedAt: at(10).UTC(), LastEditedAt: at(5).UTC(), LastEditorID: "bob",
		ContributorIDs: []string{"alice", "bob"},
	}
	n.Notify(want)
	if err := n.Wait(context.Background()); err != nil {
		t.Fatal(err)
	}
	if path != CallbackPath || secret != "s3cret" {
		t.Fatalf("path %q secret %q", path, secret)
	}
	if got.ProjectID != "p1" || got.VersionID != 7 || got.LastEditorID != "bob" || len(got.ContributorIDs) != 2 ||
		!got.SavedAt.Equal(want.SavedAt) || !got.LastEditedAt.Equal(want.LastEditedAt) {
		t.Fatalf("body = %+v", got)
	}
}

func TestNotifier_RetriesAServerError(t *testing.T) {
	t.Parallel()

	var calls atomic.Int32
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if calls.Add(1) < 3 {
			w.WriteHeader(http.StatusBadGateway)
		}
	}))
	defer srv.Close()

	n := NewNotifier(NotifierConfig{APIURL: srv.URL, Backoff: noBackoff})
	n.Notify(Saved{ProjectID: "p1", VersionID: 1})
	if err := n.Wait(context.Background()); err != nil {
		t.Fatal(err)
	}
	if c := calls.Load(); c != 3 {
		t.Fatalf("calls = %d, want 3 (two failures, then success)", c)
	}
}

func TestNotifier_DoesNotRetryARefusal(t *testing.T) {
	t.Parallel()

	var calls atomic.Int32
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		calls.Add(1)
		w.WriteHeader(http.StatusUnauthorized)
	}))
	defer srv.Close()

	n := NewNotifier(NotifierConfig{APIURL: srv.URL, Backoff: noBackoff})
	n.Notify(Saved{ProjectID: "p1", VersionID: 1})
	if err := n.Wait(context.Background()); err != nil {
		t.Fatal(err)
	}
	if c := calls.Load(); c != 1 {
		t.Fatalf("calls = %d, want 1: a wrong secret will not get better", c)
	}
}

func TestNotifier_GivesUpAfterItsAttempts(t *testing.T) {
	t.Parallel()

	var calls atomic.Int32
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		calls.Add(1)
		w.WriteHeader(http.StatusServiceUnavailable)
	}))
	defer srv.Close()

	n := NewNotifier(NotifierConfig{APIURL: srv.URL, Attempts: 4, Backoff: noBackoff})
	n.Notify(Saved{ProjectID: "p1", VersionID: 1})
	if err := n.Wait(context.Background()); err != nil {
		t.Fatal(err)
	}
	if c := calls.Load(); c != 4 {
		t.Fatalf("calls = %d, want 4", c)
	}
}

func TestNotifier_NotifyDoesNotWaitForTheAPI(t *testing.T) {
	t.Parallel()

	release := make(chan struct{})
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) { <-release }))
	defer srv.Close()
	defer close(release)

	n := NewNotifier(NotifierConfig{APIURL: srv.URL, Backoff: noBackoff})
	done := make(chan struct{})
	go func() { n.Notify(Saved{ProjectID: "p1", VersionID: 1}); close(done) }()
	select {
	case <-done:
	case <-time.After(time.Second):
		t.Fatal("Notify blocked on the API")
	}
}

func TestDefaultBackoff_IsBounded(t *testing.T) {
	t.Parallel()

	if d := defaultBackoff(1); d != time.Second {
		t.Fatalf("first backoff = %v", d)
	}
	if d := defaultBackoff(100); d != notifyBackoffMax {
		t.Fatalf("backoff(100) = %v, want the cap", d)
	}
}
