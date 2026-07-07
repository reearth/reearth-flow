package otel

import (
	"context"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"

	sdktrace "go.opentelemetry.io/otel/sdk/trace"
	"go.opentelemetry.io/otel/sdk/trace/tracetest"
)

// TestDisabledReturnsNoopProvider: disabled config yields a noop provider with a
// clean Shutdown.
func TestDisabledReturnsNoopProvider(t *testing.T) {
	tp, err := InitTracer(context.Background(), Config{Enabled: false})
	if err != nil {
		t.Fatalf("init: %v", err)
	}
	if tp == nil {
		t.Fatalf("nil provider")
	}
	if err := tp.Shutdown(context.Background()); err != nil {
		t.Fatalf("shutdown: %v", err)
	}
}

// TestSamplerFromRatio pins the ratio→sampler mapping.
func TestSamplerFromRatio(t *testing.T) {
	cases := []struct {
		ratio float64
		want  string
	}{
		{-1, "AlwaysOnSampler"},
		{0, "AlwaysOffSampler"},
		{1, "AlwaysOnSampler"},
		{2, "AlwaysOnSampler"},
		{0.25, "TraceIDRatioBased{0.25}"},
	}
	for _, c := range cases {
		got := samplerFor(c.ratio).Description()
		if got != c.want {
			t.Fatalf("ratio %v ⇒ %q, want %q", c.ratio, got, c.want)
		}
	}
}

// TestNoSecretLeakInSpanAttributes asserts the wrapped handler captures neither
// the ?token= query, the X-API-Secret header, nor any payload into span name or
// attributes.
func TestNoSecretLeakInSpanAttributes(t *testing.T) {
	rec := tracetest.NewSpanRecorder()
	tp := sdktrace.NewTracerProvider(sdktrace.WithSpanProcessor(rec))
	defer func() { _ = tp.Shutdown(context.Background()) }()

	const secret = "SUPER-SECRET-VALUE"
	const token = "JWT-TOKEN-LEAK-CANARY"

	inner := http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusOK)
	})
	h := WrapHandler(inner, WrapOptions{TracerProvider: tp, SpanName: "doc"})

	srv := httptest.NewServer(h)
	defer srv.Close()

	req, _ := http.NewRequest("GET", srv.URL+"/api/document/proj1?token="+token, nil)
	req.Header.Set("X-API-Secret", secret)
	req.Header.Set("Authorization", "Bearer "+token)
	resp, err := http.DefaultClient.Do(req)
	if err != nil {
		t.Fatalf("request: %v", err)
	}
	_ = resp.Body.Close()

	spans := rec.Ended()
	if len(spans) == 0 {
		t.Fatalf("no spans recorded — instrumentation not active")
	}
	for _, sp := range spans {
		hay := sp.Name()
		for _, kv := range sp.Attributes() {
			hay += " " + string(kv.Key) + "=" + kv.Value.String()
		}
		for _, canary := range []string{secret, token} {
			if strings.Contains(hay, canary) {
				t.Fatalf("span leaks secret/token: span=%q contains %q", hay, canary)
			}
		}
		// The full URL with query must not appear either.
		if strings.Contains(hay, "?token=") {
			t.Fatalf("span leaks query string: %q", hay)
		}
	}
}

// TestWrapHandlerNilProvider: a nil provider must not panic; it wraps with the
// global provider.
func TestWrapHandlerNilProvider(t *testing.T) {
	called := false
	inner := http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) { called = true })
	h := WrapHandler(inner, WrapOptions{})
	req := httptest.NewRequest("GET", "/api/document/x", nil)
	rec := httptest.NewRecorder()
	h.ServeHTTP(rec, req)
	if !called {
		t.Fatalf("inner handler not called")
	}
}
