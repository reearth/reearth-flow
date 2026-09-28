// Package otel wires up OpenTelemetry tracing for the API server: OTLP-gRPC
// exporters (GCP Cloud Trace, Jaeger, or a generic OTLP collector), resource
// detection, sampling and propagation.
package otel

import (
	"context"
	"fmt"
	"net"
	"time"

	"cloud.google.com/go/compute/metadata"
	"github.com/reearth/reearthx/log"
	"go.opentelemetry.io/contrib/detectors/gcp"
	"go.opentelemetry.io/otel"
	"go.opentelemetry.io/otel/attribute"
	"go.opentelemetry.io/otel/exporters/otlp/otlptrace/otlptracegrpc"
	"go.opentelemetry.io/otel/propagation"
	"go.opentelemetry.io/otel/sdk/resource"
	sdktrace "go.opentelemetry.io/otel/sdk/trace"
	semconv "go.opentelemetry.io/otel/semconv/v1.21.0"
	"go.opentelemetry.io/otel/trace"
	"go.opentelemetry.io/otel/trace/noop"
	"golang.org/x/oauth2/google"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials"
	"google.golang.org/grpc/credentials/oauth"
)

type ExporterType string

const (
	ExporterTypeJaeger ExporterType = "jaeger"
	ExporterTypeGCP    ExporterType = "gcp"
	ExporterTypeOTLP   ExporterType = "otlp"
)

const (
	gcpProjectIDAttribute = "gcp.project_id"
	gcpCloudTraceEndpoint = "telemetry.googleapis.com:443"

	defaultServiceName = "reearth-flow-api"
)

type Config struct {
	Endpoint     string
	ExporterType ExporterType

	// GCPProjectID overrides the project id resource attribute for the gcp
	// exporter. Left empty, it is looked up from the GCE metadata server.
	GCPProjectID string

	ServiceName string

	MaxExportBatchSize int
	BatchTimeout       time.Duration
	MaxQueueSize       int
	SamplingRatio      float64

	Enabled bool
}

func (c *Config) serviceName() string {
	if c.ServiceName != "" {
		return c.ServiceName
	}
	return defaultServiceName
}

type TracerProvider interface {
	Tracer(name string, options ...trace.TracerOption) trace.Tracer
	Shutdown(ctx context.Context) error
}

type noopTracerProvider struct {
	trace.TracerProvider
}

func (n *noopTracerProvider) Shutdown(ctx context.Context) error {
	return nil
}

func InitTracer(ctx context.Context, cfg *Config) (TracerProvider, error) {
	if !cfg.Enabled {
		log.Infoc(ctx, "otel: tracing is disabled")
		return &noopTracerProvider{TracerProvider: noop.NewTracerProvider()}, nil
	}

	if (cfg.ExporterType == ExporterTypeOTLP || cfg.ExporterType == ExporterTypeJaeger) && cfg.Endpoint == "" {
		return nil, fmt.Errorf("otel: endpoint is required for exporter type %s", cfg.ExporterType)
	}

	var exporter sdktrace.SpanExporter
	var err error

	switch cfg.ExporterType {
	case ExporterTypeGCP:
		log.Infoc(ctx, "otel: initializing GCP Cloud Trace exporter via OTLP")
		cfg.Endpoint = gcpCloudTraceEndpoint
		exporter, err = createGCPExporter(ctx)
	case ExporterTypeJaeger, ExporterTypeOTLP:
		log.Infoc(ctx, "otel: initializing OTLP exporter", "endpoint", cfg.Endpoint)
		exporter, err = createOTLPExporter(ctx, cfg, false)
	default:
		return nil, fmt.Errorf("otel: unknown exporter type %q", cfg.ExporterType)
	}
	if err != nil {
		return nil, fmt.Errorf("otel: failed to create trace exporter: %w", err)
	}

	res, err := createResource(ctx, cfg)
	if err != nil {
		return nil, fmt.Errorf("otel: failed to create resource: %w", err)
	}

	// Enabled with a zero ratio exports nothing. That is a valid way to switch
	// sampling off, but it is indistinguishable from an unset or mis-spelled
	// env var, so say so rather than start up looking healthy and emit nothing.
	if cfg.SamplingRatio == 0 {
		log.Warnfc(ctx, "otel: tracing is enabled but the sampling ratio is 0, so no spans will be exported; set the sampling ratio to collect traces")
	}

	sampler := createSampler(cfg)

	tp := sdktrace.NewTracerProvider(
		sdktrace.WithBatcher(
			exporter,
			sdktrace.WithMaxExportBatchSize(cfg.MaxExportBatchSize),
			sdktrace.WithBatchTimeout(cfg.BatchTimeout),
			sdktrace.WithMaxQueueSize(cfg.MaxQueueSize),
		),
		sdktrace.WithResource(res),
		sdktrace.WithSampler(sampler),
	)

	otel.SetTracerProvider(tp)
	otel.SetTextMapPropagator(propagation.NewCompositeTextMapPropagator(
		propagation.TraceContext{},
		propagation.Baggage{},
	))

	log.Infoc(ctx, "otel: tracing initialized",
		"endpoint", cfg.Endpoint,
		"exporter", cfg.ExporterType,
		"service", cfg.serviceName(),
		"sampling_ratio", cfg.SamplingRatio)

	return tp, nil
}

func createResource(ctx context.Context, cfg *Config) (*resource.Resource, error) {
	opts := []resource.Option{
		resource.WithTelemetrySDK(),
		resource.WithAttributes(semconv.ServiceName(cfg.serviceName())),
	}

	if cfg.ExporterType == ExporterTypeGCP {
		if gcpDetector := gcp.NewDetector(); gcpDetector != nil {
			opts = append(opts, resource.WithDetectors(gcpDetector))
		}

		projectID := cfg.GCPProjectID
		if projectID == "" && metadata.OnGCE() {
			if id, err := metadata.ProjectIDWithContext(ctx); err == nil {
				projectID = id
			}
		}
		if projectID != "" {
			opts = append(opts, resource.WithAttributes(attribute.String(gcpProjectIDAttribute, projectID)))
		}
	}

	return resource.New(ctx, opts...)
}

func createSampler(cfg *Config) sdktrace.Sampler {
	switch {
	case cfg.SamplingRatio < 0, cfg.SamplingRatio >= 1:
		return sdktrace.AlwaysSample()
	case cfg.SamplingRatio == 0:
		return sdktrace.NeverSample()
	default:
		return sdktrace.TraceIDRatioBased(cfg.SamplingRatio)
	}
}

func createOTLPExporter(ctx context.Context, cfg *Config, useGCPAuth bool) (sdktrace.SpanExporter, error) {
	opts := []otlptracegrpc.Option{
		otlptracegrpc.WithEndpoint(cfg.Endpoint),
	}

	if useGCPAuth {
		// Cloud Trace's OTLP endpoint requires an authenticated gRPC channel:
		// TLS plus a bearer token from application default credentials.
		creds, err := google.FindDefaultCredentials(ctx, "https://www.googleapis.com/auth/trace.append")
		if err != nil {
			return nil, fmt.Errorf("failed to get GCP credentials: %w", err)
		}

		opts = append(opts,
			otlptracegrpc.WithTLSCredentials(credentials.NewClientTLSFromCert(nil, "")),
			otlptracegrpc.WithDialOption(grpc.WithPerRPCCredentials(oauth.TokenSource{TokenSource: creds.TokenSource})),
		)
	} else if isLoopback(cfg.Endpoint) {
		// local collectors (make run-jaeger) have no certificate
		opts = append(opts, otlptracegrpc.WithInsecure())
	} else {
		opts = append(opts, otlptracegrpc.WithTLSCredentials(credentials.NewClientTLSFromCert(nil, "")))
	}

	return otlptracegrpc.New(ctx, opts...)
}

func isLoopback(endpoint string) bool {
	host, _, err := net.SplitHostPort(endpoint)
	if err != nil {
		host = endpoint
	}
	if host == "localhost" {
		return true
	}
	ip := net.ParseIP(host)
	return ip != nil && ip.IsLoopback()
}

func createGCPExporter(ctx context.Context) (sdktrace.SpanExporter, error) {
	return createOTLPExporter(ctx, &Config{Endpoint: gcpCloudTraceEndpoint}, true)
}
