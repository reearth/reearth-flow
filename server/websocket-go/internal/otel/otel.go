// Package otel provides OTLP-over-gRPC tracing for the websocket-go service:
// exporter select, GCP resource detector, ratio sampler, batcher, and graceful
// Shutdown, wired to net/http via otelhttp.
//
// SECURITY: tokens (?token=), X-API-Secret, the full URL with query, and document
// payloads MUST NEVER become span attributes. WrapHandler names spans by a static
// name only and adds no header/query attributes.
package otel

import (
	"context"
	"fmt"
	"log/slog"
	"net/http"
	"time"

	"cloud.google.com/go/compute/metadata"
	"go.opentelemetry.io/contrib/detectors/gcp"
	"go.opentelemetry.io/contrib/instrumentation/net/http/otelhttp"
	"go.opentelemetry.io/otel"
	"go.opentelemetry.io/otel/attribute"
	"go.opentelemetry.io/otel/exporters/otlp/otlptrace/otlptracegrpc"
	"go.opentelemetry.io/otel/propagation"
	"go.opentelemetry.io/otel/sdk/resource"
	sdktrace "go.opentelemetry.io/otel/sdk/trace"
	semconv "go.opentelemetry.io/otel/semconv/v1.21.0"
	"go.opentelemetry.io/otel/trace"
	"go.opentelemetry.io/otel/trace/noop"
)

// ExporterType selects the span exporter backend.
type ExporterType string

const (
	ExporterTypeOTLP ExporterType = "otlp"
	ExporterTypeGCP  ExporterType = "gcp"
)

const (
	gcpProjectIDAttribute = "gcp.project_id"
	gcpCloudTraceEndpoint = "telemetry.googleapis.com:443"

	defaultServiceName = "reearth-flow-websocket"
)

// Config configures the tracer.
type Config struct {
	Enabled      bool
	Endpoint     string
	ExporterType ExporterType
	GCPProjectID string
	ServiceName  string

	MaxExportBatchSize int
	BatchTimeout       time.Duration
	MaxQueueSize       int
	SamplingRatio      float64
}

// TracerProvider adds a Shutdown to flush spans on graceful drain.
type TracerProvider interface {
	trace.TracerProvider
	Shutdown(ctx context.Context) error
}

type noopTracerProvider struct{ trace.TracerProvider }

func (n *noopTracerProvider) Shutdown(context.Context) error { return nil }

// InitTracer builds the tracer provider. Disabled yields a noop provider; enabled
// yields a batched otlptracegrpc provider (GCP resource detector, ratio sampler,
// W3C tracecontext + baggage propagation), set as the global provider.
func InitTracer(ctx context.Context, cfg Config) (TracerProvider, error) {
	if !cfg.Enabled {
		slog.Default().Info("OpenTelemetry tracing disabled")
		return &noopTracerProvider{TracerProvider: noop.NewTracerProvider()}, nil
	}

	if cfg.ExporterType == ExporterTypeOTLP && cfg.Endpoint == "" {
		return nil, fmt.Errorf("otel: OTLP endpoint required for exporter type %q", cfg.ExporterType)
	}

	exporter, err := newExporter(ctx, cfg)
	if err != nil {
		return nil, fmt.Errorf("otel: create exporter: %w", err)
	}
	res, err := newResource(ctx, cfg)
	if err != nil {
		return nil, fmt.Errorf("otel: create resource: %w", err)
	}

	tp := sdktrace.NewTracerProvider(
		sdktrace.WithBatcher(exporter,
			sdktrace.WithMaxExportBatchSize(cfg.MaxExportBatchSize),
			sdktrace.WithBatchTimeout(cfg.BatchTimeout),
			sdktrace.WithMaxQueueSize(cfg.MaxQueueSize),
		),
		sdktrace.WithResource(res),
		sdktrace.WithSampler(samplerFor(cfg.SamplingRatio)),
	)
	otel.SetTracerProvider(tp)
	otel.SetTextMapPropagator(propagation.NewCompositeTextMapPropagator(
		propagation.TraceContext{},
		propagation.Baggage{},
	))
	slog.Default().Info("OpenTelemetry tracing initialized",
		"exporter", cfg.ExporterType,
		"endpoint", cfg.Endpoint,
		"service", serviceName(cfg),
		"sampling_ratio", cfg.SamplingRatio)
	return tp, nil
}

func serviceName(cfg Config) string {
	if cfg.ServiceName != "" {
		return cfg.ServiceName
	}
	return defaultServiceName
}

func newExporter(ctx context.Context, cfg Config) (sdktrace.SpanExporter, error) {
	switch cfg.ExporterType {
	case ExporterTypeGCP:
		// GCP Cloud Trace via OTLP (TLS endpoint, ADC credentials).
		return otlptracegrpc.New(ctx, otlptracegrpc.WithEndpoint(gcpCloudTraceEndpoint))
	default: // OTLP (collector / Jaeger) over insecure gRPC.
		return otlptracegrpc.New(ctx,
			otlptracegrpc.WithEndpoint(cfg.Endpoint),
			otlptracegrpc.WithInsecure(),
		)
	}
}

func newResource(ctx context.Context, cfg Config) (*resource.Resource, error) {
	opts := []resource.Option{}
	if cfg.ExporterType == ExporterTypeGCP {
		opts = append(opts, resource.WithDetectors(gcp.NewDetector()))
		if cfg.GCPProjectID != "" {
			opts = append(opts, resource.WithAttributes(
				attribute.String(gcpProjectIDAttribute, cfg.GCPProjectID),
			))
		} else if metadata.OnGCE() {
			if id, err := metadata.ProjectIDWithContext(ctx); err == nil {
				opts = append(opts, resource.WithAttributes(
					attribute.String(gcpProjectIDAttribute, id),
				))
			}
		}
	}
	opts = append(opts,
		resource.WithTelemetrySDK(),
		resource.WithAttributes(semconv.ServiceName(serviceName(cfg))),
	)
	return resource.New(ctx, opts...)
}

// samplerFor maps a ratio to a sampler: <0 always, 0 never, >=1 always, else
// ratio-based.
func samplerFor(ratio float64) sdktrace.Sampler {
	switch {
	case ratio < 0:
		return sdktrace.AlwaysSample()
	case ratio == 0:
		return sdktrace.NeverSample()
	case ratio >= 1:
		return sdktrace.AlwaysSample()
	default:
		return sdktrace.TraceIDRatioBased(ratio)
	}
}

// WrapOptions configure WrapHandler.
type WrapOptions struct {
	// TracerProvider to use. nil ⇒ the global provider (noop unless InitTracer ran).
	TracerProvider trace.TracerProvider
	// SpanName is the static operation name for the request span (defaults to
	// "http.request"). The span is NEVER named from the raw URL, so ?token= cannot
	// reach a span name.
	SpanName string
}

// WrapHandler instruments next with otelhttp request spans, hardened so no
// secret/token/payload can leak into spans: spans use a static name and no header
// or query attributes are attached.
func WrapHandler(next http.Handler, opts WrapOptions) http.Handler {
	name := opts.SpanName
	if name == "" {
		name = "http.request"
	}
	otelOpts := []otelhttp.Option{
		// Pin the span name: never derive it from r.URL (which carries ?token=).
		otelhttp.WithSpanNameFormatter(func(operation string, r *http.Request) string {
			return name
		}),
	}
	if opts.TracerProvider != nil {
		otelOpts = append(otelOpts, otelhttp.WithTracerProvider(opts.TracerProvider))
	}
	return otelhttp.NewHandler(next, name, otelOpts...)
}
