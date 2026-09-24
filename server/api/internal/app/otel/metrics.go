package otel

import (
	"context"
	"fmt"
	"net"
	"net/http"
	"time"

	"github.com/prometheus/client_golang/prometheus"
	"github.com/prometheus/client_golang/prometheus/promhttp"
	"github.com/reearth/reearthx/log"
	"go.opentelemetry.io/otel"
	"go.opentelemetry.io/otel/exporters/otlp/otlpmetric/otlpmetricgrpc"
	otelprom "go.opentelemetry.io/otel/exporters/prometheus"
	"go.opentelemetry.io/otel/metric"
	"go.opentelemetry.io/otel/metric/noop"
	sdkmetric "go.opentelemetry.io/otel/sdk/metric"
	"go.opentelemetry.io/otel/sdk/resource"
	"golang.org/x/oauth2/google"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials"
	"google.golang.org/grpc/credentials/oauth"
)

type MeterProvider interface {
	Meter(name string, options ...metric.MeterOption) metric.Meter
	Shutdown(ctx context.Context) error
}

type noopMeterProvider struct {
	metric.MeterProvider
}

func (n *noopMeterProvider) Shutdown(ctx context.Context) error {
	return nil
}

// promServer wraps a sdkmetric.MeterProvider with the HTTP server serving the
// Prometheus scrape endpoint, so both are torn down together on Shutdown.
type promServer struct {
	*sdkmetric.MeterProvider
	server *http.Server
}

func (p *promServer) Shutdown(ctx context.Context) error {
	err := p.MeterProvider.Shutdown(ctx)
	if p.server != nil {
		if serr := p.server.Shutdown(ctx); serr != nil && err == nil {
			err = serr
		}
	}
	return err
}

func InitMeter(ctx context.Context, cfg *Config) (MeterProvider, error) {
	if !cfg.MetricsEnabled {
		log.Infoc(ctx, "otel: metrics are disabled")
		return &noopMeterProvider{MeterProvider: noop.NewMeterProvider()}, nil
	}

	if (cfg.MetricsExporterType == ExporterTypeOTLP) && cfg.MetricsEndpoint == "" {
		return nil, fmt.Errorf("otel: metrics endpoint is required for exporter type %s", cfg.MetricsExporterType)
	}

	res, err := createResource(ctx, cfg.serviceName(), cfg.GCPProjectID, cfg.MetricsExporterType == ExporterTypeGCP)
	if err != nil {
		return nil, fmt.Errorf("otel: failed to create resource: %w", err)
	}

	interval := cfg.MetricsExportInterval
	if interval <= 0 {
		interval = defaultMetricsExportEvery
	}

	switch cfg.MetricsExporterType {
	case ExporterTypePrometheus:
		log.Infoc(ctx, "otel: initializing Prometheus exporter", "addr", cfg.PrometheusAddr)
		return initPrometheusMeter(ctx, cfg, res)
	case ExporterTypeGCP:
		log.Infoc(ctx, "otel: initializing GCP Cloud Monitoring exporter via OTLP")
		cfg.MetricsEndpoint = gcpCloudTraceEndpoint
		exporter, err := createOTLPMetricExporter(ctx, cfg, true)
		if err != nil {
			return nil, fmt.Errorf("otel: failed to create metrics exporter: %w", err)
		}
		return newPeriodicMeterProvider(exporter, res, interval), nil
	case ExporterTypeOTLP:
		log.Infoc(ctx, "otel: initializing OTLP metrics exporter", "endpoint", cfg.MetricsEndpoint)
		exporter, err := createOTLPMetricExporter(ctx, cfg, false)
		if err != nil {
			return nil, fmt.Errorf("otel: failed to create metrics exporter: %w", err)
		}
		return newPeriodicMeterProvider(exporter, res, interval), nil
	default:
		return nil, fmt.Errorf("otel: unknown metrics exporter type %q", cfg.MetricsExporterType)
	}
}

func newPeriodicMeterProvider(exporter sdkmetric.Exporter, res *resource.Resource, interval time.Duration) MeterProvider {
	mp := sdkmetric.NewMeterProvider(
		sdkmetric.WithReader(sdkmetric.NewPeriodicReader(exporter, sdkmetric.WithInterval(interval))),
		sdkmetric.WithResource(res),
	)
	otel.SetMeterProvider(mp)
	return mp
}

func createOTLPMetricExporter(ctx context.Context, cfg *Config, useGCPAuth bool) (sdkmetric.Exporter, error) {
	opts := []otlpmetricgrpc.Option{
		otlpmetricgrpc.WithEndpoint(cfg.MetricsEndpoint),
	}

	if useGCPAuth {
		creds, err := google.FindDefaultCredentials(ctx, gcpCloudMonitoringScope)
		if err != nil {
			return nil, fmt.Errorf("failed to get GCP credentials: %w", err)
		}

		opts = append(opts,
			otlpmetricgrpc.WithTLSCredentials(credentials.NewClientTLSFromCert(nil, "")),
			otlpmetricgrpc.WithDialOption(grpc.WithPerRPCCredentials(oauth.TokenSource{TokenSource: creds.TokenSource})),
		)
	} else if isLoopback(cfg.MetricsEndpoint) {
		opts = append(opts, otlpmetricgrpc.WithInsecure())
	} else {
		opts = append(opts, otlpmetricgrpc.WithTLSCredentials(credentials.NewClientTLSFromCert(nil, "")))
	}

	return otlpmetricgrpc.New(ctx, opts...)
}

func initPrometheusMeter(ctx context.Context, cfg *Config, res *resource.Resource) (MeterProvider, error) {
	registry := prometheus.NewRegistry()

	exporter, err := otelprom.New(otelprom.WithRegisterer(registry))
	if err != nil {
		return nil, fmt.Errorf("failed to create prometheus exporter: %w", err)
	}

	mp := sdkmetric.NewMeterProvider(
		sdkmetric.WithReader(exporter),
		sdkmetric.WithResource(res),
	)

	addr := cfg.PrometheusAddr
	if addr == "" {
		addr = defaultPrometheusAddr
	}

	ln, err := net.Listen("tcp", addr)
	if err != nil {
		return nil, fmt.Errorf("failed to listen on %s for prometheus scrape endpoint: %w", addr, err)
	}

	mux := http.NewServeMux()
	mux.Handle("/metrics", promhttp.HandlerFor(registry, promhttp.HandlerOpts{}))
	server := &http.Server{Handler: mux}

	go func() {
		if err := server.Serve(ln); err != nil && err != http.ErrServerClosed {
			log.Errorc(ctx, "otel: prometheus scrape server stopped", "error", err)
		}
	}()

	log.Infoc(ctx, "otel: prometheus scrape endpoint listening", "addr", ln.Addr().String())

	otel.SetMeterProvider(mp)

	return &promServer{MeterProvider: mp, server: server}, nil
}
