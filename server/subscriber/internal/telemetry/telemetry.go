package telemetry

import (
	"context"
	"fmt"
	"time"

	texporter "github.com/GoogleCloudPlatform/opentelemetry-operations-go/exporter/trace"
	"go.opentelemetry.io/otel"
	"go.opentelemetry.io/otel/exporters/otlp/otlptrace/otlptracegrpc"
	"go.opentelemetry.io/otel/propagation"
	"go.opentelemetry.io/otel/sdk/resource"
	sdktrace "go.opentelemetry.io/otel/sdk/trace"
	semconv "go.opentelemetry.io/otel/semconv/v1.38.0"
	"go.opentelemetry.io/otel/trace"
)

const (
	serviceName    = "reearth-flow-subscriber"
	serviceVersion = "1.0.0"

	TracerTypeGCP  = "gcp"
	TracerTypeOTLP = "otlp"
)

type Config struct {
	Enabled      bool
	TracerType   string // "gcp" or "otlp"
	GCPProjectID string
	OTLPEndpoint string
	Insecure     bool
}

type Telemetry struct {
	tracerProvider *sdktrace.TracerProvider
	tracer         trace.Tracer
}

func New(ctx context.Context, cfg Config) (*Telemetry, error) {
	if !cfg.Enabled {
		return &Telemetry{
			tracer: otel.Tracer(serviceName),
		}, nil
	}

	res, err := resource.Merge(
		resource.Default(),
		resource.NewSchemaless(
			semconv.ServiceName(serviceName),
			semconv.ServiceVersion(serviceVersion),
		),
	)
	if err != nil {
		return nil, fmt.Errorf("failed to create resource: %w", err)
	}

	var exporter sdktrace.SpanExporter

	switch cfg.TracerType {
	case TracerTypeGCP:
		exporter, err = createGCPExporter(ctx, cfg.GCPProjectID)
	case TracerTypeOTLP:
		exporter, err = createOTLPExporter(ctx, cfg.OTLPEndpoint, cfg.Insecure)
	default:
		// Auto-detect: use GCP if project ID is set, otherwise OTLP
		if cfg.GCPProjectID != "" {
			exporter, err = createGCPExporter(ctx, cfg.GCPProjectID)
		} else if cfg.OTLPEndpoint != "" {
			exporter, err = createOTLPExporter(ctx, cfg.OTLPEndpoint, cfg.Insecure)
		} else {
			return &Telemetry{
				tracer: otel.Tracer(serviceName),
			}, nil
		}
	}

	if err != nil {
		return nil, err
	}

	tp := sdktrace.NewTracerProvider(
		sdktrace.WithBatcher(exporter,
			sdktrace.WithBatchTimeout(5*time.Second),
		),
		sdktrace.WithResource(res),
		sdktrace.WithSampler(sdktrace.AlwaysSample()),
	)

	otel.SetTracerProvider(tp)
	otel.SetTextMapPropagator(propagation.NewCompositeTextMapPropagator(
		propagation.TraceContext{},
		propagation.Baggage{},
	))

	return &Telemetry{
		tracerProvider: tp,
		tracer:         tp.Tracer(serviceName),
	}, nil
}

func createGCPExporter(ctx context.Context, projectID string) (sdktrace.SpanExporter, error) {
	exporter, err := texporter.New(texporter.WithProjectID(projectID))
	if err != nil {
		return nil, fmt.Errorf("failed to create GCP trace exporter: %w", err)
	}
	return exporter, nil
}

func createOTLPExporter(ctx context.Context, endpoint string, insecure bool) (sdktrace.SpanExporter, error) {
	opts := []otlptracegrpc.Option{
		otlptracegrpc.WithEndpoint(endpoint),
	}
	if insecure {
		opts = append(opts, otlptracegrpc.WithInsecure())
	}

	exporter, err := otlptracegrpc.New(ctx, opts...)
	if err != nil {
		return nil, fmt.Errorf("failed to create OTLP exporter: %w", err)
	}
	return exporter, nil
}

func (t *Telemetry) Tracer() trace.Tracer {
	return t.tracer
}

func (t *Telemetry) Shutdown(ctx context.Context) error {
	if t.tracerProvider == nil {
		return nil
	}
	return t.tracerProvider.Shutdown(ctx)
}
