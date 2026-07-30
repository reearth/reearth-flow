package http

import (
	"bufio"
	"context"
	"errors"
	"log/slog"
	"net"
	"net/http"
	"runtime/debug"
	"time"
)

// statusRecorder wraps an http.ResponseWriter to capture the response status for
// access logging. It forwards Hijack and Flush so the WebSocket upgrade and any
// streaming handlers keep working when this middleware is in the chain.
type statusRecorder struct {
	http.ResponseWriter
	status   int
	wrote    bool
	hijacked bool
}

func (s *statusRecorder) WriteHeader(code int) {
	if s.wrote {
		return
	}
	s.status = code
	s.wrote = true
	s.ResponseWriter.WriteHeader(code)
}

func (s *statusRecorder) Write(b []byte) (int, error) {
	if !s.wrote {
		// An implicit 200, mirroring net/http's behavior on first Write.
		s.status = http.StatusOK
		s.wrote = true
	}
	return s.ResponseWriter.Write(b)
}

// Hijack forwards to the underlying writer (required for the WebSocket upgrade)
// and records that the connection was taken over.
func (s *statusRecorder) Hijack() (net.Conn, *bufio.ReadWriter, error) {
	hj, ok := s.ResponseWriter.(http.Hijacker)
	if !ok {
		return nil, nil, errors.New("flowhttp: underlying ResponseWriter is not a http.Hijacker")
	}
	s.hijacked = true
	return hj.Hijack()
}

func (s *statusRecorder) Flush() {
	if f, ok := s.ResponseWriter.(http.Flusher); ok {
		f.Flush()
	}
}

// ObserveRequests returns middleware that emits one structured log line per
// request (method, path, status, duration) and recovers panics into a logged
// 500, so no request failure is silent. It is WebSocket-safe (Hijack is
// preserved) and should wrap the outermost handler so it observes every route,
// including the ygo WS upgrade whose own 500s would otherwise be invisible.
func ObserveRequests(log *slog.Logger) func(http.Handler) http.Handler {
	if log == nil {
		log = slog.Default()
	}
	return func(next http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			rec := &statusRecorder{ResponseWriter: w, status: http.StatusOK}
			start := time.Now()
			defer func() {
				if p := recover(); p != nil {
					log.Error("panic serving request",
						"method", r.Method, "path", r.URL.Path,
						"panic", p, "stack", string(debug.Stack()))
					if !rec.wrote && !rec.hijacked {
						rec.WriteHeader(http.StatusInternalServerError)
					}
				}
				status := rec.status
				if rec.hijacked {
					status = http.StatusSwitchingProtocols // 101: a WS upgrade
				}
				logRequest(r.Context(), log, r, status, time.Since(start))
			}()
			next.ServeHTTP(rec, r)
		})
	}
}

// logRequest picks a severity from the status (5xx ERROR, 4xx WARN, else INFO)
// and emits the access line. Healthy /health probes drop to DEBUG so frequent
// load-balancer checks do not bury real errors in the INFO stream.
func logRequest(ctx context.Context, log *slog.Logger, r *http.Request, status int, dur time.Duration) {
	level := slog.LevelInfo
	switch {
	case status >= 500:
		level = slog.LevelError
	case status >= 400:
		level = slog.LevelWarn
	case r.URL.Path == "/health":
		level = slog.LevelDebug
	}
	log.Log(ctx, level, "http request",
		"method", r.Method,
		"path", r.URL.Path,
		"status", status,
		"duration_ms", dur.Milliseconds(),
	)
}
