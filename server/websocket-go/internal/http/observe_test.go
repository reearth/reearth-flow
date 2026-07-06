package http

import (
	"bufio"
	"bytes"
	"errors"
	"log/slog"
	"net"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"
)

func newCapturingLogger() (*slog.Logger, *bytes.Buffer) {
	var buf bytes.Buffer
	return slog.New(slog.NewJSONHandler(&buf, &slog.HandlerOptions{Level: slog.LevelDebug})), &buf
}

func TestObserveLogsServerErrorAtError(t *testing.T) {
	log, buf := newCapturingLogger()
	h := ObserveRequests(log)(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusInternalServerError)
	}))
	rec := httptest.NewRecorder()
	h.ServeHTTP(rec, httptest.NewRequest("GET", "/api/document/proj1", nil))

	out := buf.String()
	if !strings.Contains(out, `"level":"ERROR"`) {
		t.Fatalf("5xx not logged at ERROR:\n%s", out)
	}
	if !strings.Contains(out, `"status":500`) || !strings.Contains(out, "/api/document/proj1") {
		t.Fatalf("request line missing status/path:\n%s", out)
	}
}

func TestObserveLogsOKAtInfo(t *testing.T) {
	log, buf := newCapturingLogger()
	h := ObserveRequests(log)(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusOK)
	}))
	rec := httptest.NewRecorder()
	h.ServeHTTP(rec, httptest.NewRequest("GET", "/api/document/proj1", nil))

	out := buf.String()
	if !strings.Contains(out, `"level":"INFO"`) || !strings.Contains(out, `"status":200`) {
		t.Fatalf("2xx not logged at INFO with status 200:\n%s", out)
	}
}

func TestObserveRecoversPanic(t *testing.T) {
	log, buf := newCapturingLogger()
	h := ObserveRequests(log)(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		panic("boom")
	}))
	rec := httptest.NewRecorder()
	h.ServeHTTP(rec, httptest.NewRequest("GET", "/api/document/proj1", nil))

	if rec.Code != http.StatusInternalServerError {
		t.Fatalf("panic did not become a 500: got %d", rec.Code)
	}
	out := buf.String()
	if !strings.Contains(out, `"level":"ERROR"`) || !strings.Contains(out, "boom") {
		t.Fatalf("panic not logged with its value at ERROR:\n%s", out)
	}
}

// hijackableRecorder is an httptest recorder that also satisfies http.Hijacker,
// standing in for the real server's ResponseWriter that the WS upgrade hijacks.
type hijackableRecorder struct {
	*httptest.ResponseRecorder
	hijackCalled bool
}

func (h *hijackableRecorder) Hijack() (net.Conn, *bufio.ReadWriter, error) {
	h.hijackCalled = true
	return nil, nil, errors.New("test: no real conn")
}

// TestObservePreservesHijacker is the WebSocket-safety guard: the wrapped
// ResponseWriter MUST still expose http.Hijacker, or the WS upgrade breaks.
func TestObservePreservesHijacker(t *testing.T) {
	log, _ := newCapturingLogger()
	var sawHijacker bool
	h := ObserveRequests(log)(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		_, sawHijacker = w.(http.Hijacker)
	}))
	rw := &hijackableRecorder{ResponseRecorder: httptest.NewRecorder()}
	h.ServeHTTP(rw, httptest.NewRequest("GET", "/room-uuid", nil))
	if !sawHijacker {
		t.Fatal("wrapped ResponseWriter dropped http.Hijacker — the WebSocket upgrade would fail")
	}
}

// TestObserveHealthProbeAtDebug keeps health-probe noise out of the INFO stream
// so it cannot bury the real errors an operator is hunting.
func TestObserveHealthProbeAtDebug(t *testing.T) {
	log, buf := newCapturingLogger()
	h := ObserveRequests(log)(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusOK)
	}))
	rec := httptest.NewRecorder()
	h.ServeHTTP(rec, httptest.NewRequest("GET", "/health", nil))

	out := buf.String()
	if !strings.Contains(out, `"level":"DEBUG"`) {
		t.Fatalf("healthy /health probe should log at DEBUG:\n%s", out)
	}
	if strings.Contains(out, `"level":"INFO"`) {
		t.Fatalf("healthy /health probe must not log at INFO:\n%s", out)
	}
}
