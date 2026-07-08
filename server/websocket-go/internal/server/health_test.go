package server

import (
	"context"
	"encoding/json"
	"errors"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"
)

type fakePinger struct{ err error }

func (f fakePinger) Ping(context.Context) error { return f.err }

type fakeLister struct{ err error }

func (f fakeLister) List(context.Context) error { return f.err }

func doHealth(t *testing.T, srv *Server) (int, map[string]any) {
	t.Helper()
	rec := httptest.NewRecorder()
	req := httptest.NewRequest(http.MethodGet, "/health", nil)
	srv.Handler().ServeHTTP(rec, req)
	var body map[string]any
	if err := json.Unmarshal(rec.Body.Bytes(), &body); err != nil {
		t.Fatalf("decode health body %q: %v", rec.Body.String(), err)
	}
	return rec.Code, body
}

func TestHealthOKWhenBothSucceed(t *testing.T) {
	srv := New(testConfig())
	srv.SetHealthChecks(fakePinger{}, fakeLister{})
	code, body := doHealth(t, srv)
	if code != http.StatusOK {
		t.Fatalf("status = %d, want 200", code)
	}
	if body["status"] != "ok" {
		t.Fatalf("status field = %v", body["status"])
	}
	comps, _ := body["components"].(map[string]any)
	if comps["redis"] != "ok" || comps["gcs"] != "ok" {
		t.Fatalf("components = %v", comps)
	}
}

func TestHealth503WhenRedisFails(t *testing.T) {
	srv := New(testConfig())
	srv.SetHealthChecks(fakePinger{err: errors.New("conn refused")}, fakeLister{})
	code, body := doHealth(t, srv)
	if code != http.StatusServiceUnavailable {
		t.Fatalf("status = %d, want 503", code)
	}
	comps, _ := body["components"].(map[string]any)
	if comps["redis"] != "error" {
		t.Fatalf("redis = %v, want generic %q", comps["redis"], "error")
	}
	if comps["gcs"] != "ok" {
		t.Fatalf("gcs should be ok: %v", comps)
	}
}

// TestHealthDoesNotLeakErrorDetail asserts the /health body never echoes the underlying error string.
func TestHealthDoesNotLeakErrorDetail(t *testing.T) {
	const secret = "redis-internal.svc.cluster.local:6379"
	srv := New(testConfig())
	srv.SetHealthChecks(
		fakePinger{err: errors.New("dial tcp " + secret + ": connection refused")},
		fakeLister{err: errors.New("bucket gs://" + secret + " forbidden")},
	)
	rec := httptest.NewRecorder()
	req := httptest.NewRequest(http.MethodGet, "/health", nil)
	srv.Handler().ServeHTTP(rec, req)
	if strings.Contains(rec.Body.String(), secret) {
		t.Fatalf("health body leaked internal detail: %s", rec.Body.String())
	}
}

func TestHealth503WhenGCSFails(t *testing.T) {
	srv := New(testConfig())
	srv.SetHealthChecks(fakePinger{}, fakeLister{err: errors.New("no bucket")})
	code, _ := doHealth(t, srv)
	if code != http.StatusServiceUnavailable {
		t.Fatalf("status = %d, want 503", code)
	}
}

func TestHealth503WhenUnconfigured(t *testing.T) {
	// No SetHealthChecks call: deps are nil, so health must fail closed.
	srv := New(testConfig())
	code, _ := doHealth(t, srv)
	if code != http.StatusServiceUnavailable {
		t.Fatalf("status = %d, want 503 when health deps unset", code)
	}
}
