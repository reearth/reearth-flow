package server

import (
	"context"
	"encoding/json"
	"net/http"
)

// pinger probes a dependency for liveness (real impl: Redis PING or Postgres
// SELECT 1, depending on the configured coordination backend).
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
	pinger  pinger
	lister  lister
	backend string
}

// SetHealthChecks attaches the coordination-store pinger and GCS lister used by
// /health, plus the name of the backend being probed. Until set, /health fails
// closed (503). A func-typed nil is normalized to a nil probe so the fail-closed
// checks behave correctly.
//
// backend is reported verbatim in the response so an operator can confirm which
// store the service is actually coordinating through — during the Redis/Postgres
// comparison that is the difference between a meaningful measurement and a wasted
// run.
func (s *Server) SetHealthChecks(p pinger, l lister, backend string) {
	if pf, ok := p.(PingerFunc); ok && pf == nil {
		p = nil
	}
	if lf, ok := l.(ListerFunc); ok && lf == nil {
		l = nil
	}
	s.health.pinger = p
	s.health.lister = l
	s.health.backend = backend
}

func (s *Server) registerHealth(mux *http.ServeMux) {
	mux.HandleFunc("GET /health", s.healthHandler)
}

// healthHandler returns 200 when both the coordination-store probe and the GCS
// list succeed, 503 otherwise, with per-component statuses in the JSON body.
//
// The coordination component is NOT named after a specific store: it was "redis"
// when Redis was the only option, which then reported "redis":"ok" for a service
// coordinating through Postgres.
func (s *Server) healthHandler(w http.ResponseWriter, r *http.Request) {
	ctx := r.Context()
	components := map[string]string{
		"coordination": s.checkCoordination(ctx),
		"gcs":          s.checkGCS(ctx),
	}
	healthy := components["coordination"] == "ok" && components["gcs"] == "ok"

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
		"backend":    s.health.backend,
		"components": components,
	})
}

func (s *Server) checkCoordination(ctx context.Context) string {
	if s.health.pinger == nil {
		// The memory backend has no store to probe, which is healthy for it —
		// callers distinguish it from a failure by the "unconfigured" value.
		return "unconfigured"
	}
	if err := s.health.pinger.Ping(ctx); err != nil {
		// Never leak the error detail to the response; log only.
		s.log.Warn("health: coordination probe failed", "backend", s.health.backend, "err", err)
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
