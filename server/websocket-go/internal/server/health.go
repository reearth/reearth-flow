package server

import (
	"context"
	"encoding/json"
	"net/http"
)

// pinger probes a dependency for liveness (real impl: Redis PING).
type pinger interface {
	Ping(ctx context.Context) error
}

// lister probes a dependency by listing (real impl: a GCS bucket list).
type lister interface {
	List(ctx context.Context) error
}

// PingerFunc adapts a function to the pinger interface; a nil value is a nil pinger.
type PingerFunc func(ctx context.Context) error

// Ping implements pinger.
func (f PingerFunc) Ping(ctx context.Context) error { return f(ctx) }

// ListerFunc adapts a function to the lister interface; a nil value is a nil lister.
type ListerFunc func(ctx context.Context) error

// List implements lister.
func (f ListerFunc) List(ctx context.Context) error { return f(ctx) }

type healthDeps struct {
	pinger pinger
	lister lister
}

// SetHealthChecks attaches the Redis pinger and GCS lister used by /health.
// Until set, /health fails closed (503). A func-typed nil is normalized to a
// nil probe so the fail-closed checks behave correctly.
func (s *Server) SetHealthChecks(p pinger, l lister) {
	if pf, ok := p.(PingerFunc); ok && pf == nil {
		p = nil
	}
	if lf, ok := l.(ListerFunc); ok && lf == nil {
		l = nil
	}
	s.health.pinger = p
	s.health.lister = l
}

func (s *Server) registerHealth(mux *http.ServeMux) {
	mux.HandleFunc("GET /health", s.healthHandler)
}

// healthHandler returns 200 when both the Redis PING and the GCS list succeed,
// 503 otherwise, with per-component statuses in the JSON body.
func (s *Server) healthHandler(w http.ResponseWriter, r *http.Request) {
	ctx := r.Context()
	components := map[string]string{
		"redis": s.checkRedis(ctx),
		"gcs":   s.checkGCS(ctx),
	}
	healthy := components["redis"] == "ok" && components["gcs"] == "ok"

	status := "ok"
	code := http.StatusOK
	if !healthy {
		status = "unavailable"
		code = http.StatusServiceUnavailable
	}

	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(code)
	_ = json.NewEncoder(w).Encode(map[string]any{
		"status":     status,
		"components": components,
	})
}

func (s *Server) checkRedis(ctx context.Context) string {
	if s.health.pinger == nil {
		return "unconfigured"
	}
	if err := s.health.pinger.Ping(ctx); err != nil {
		// Never leak the error detail to the response; log only.
		s.log.Warn("health: redis probe failed", "err", err)
		return "error"
	}
	return "ok"
}

func (s *Server) checkGCS(ctx context.Context) string {
	if s.health.lister == nil {
		return "unconfigured"
	}
	if err := s.health.lister.List(ctx); err != nil {
		s.log.Warn("health: gcs probe failed", "err", err)
		return "error"
	}
	return "ok"
}
