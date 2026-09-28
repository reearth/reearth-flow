package app

import (
	"context"
	"testing"

	"github.com/reearth/reearth-flow/api/internal/app/config"
	apiotel "github.com/reearth/reearth-flow/api/internal/app/otel"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/internal/usecase/interactor"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	sdkmetric "go.opentelemetry.io/otel/sdk/metric"
	"go.opentelemetry.io/otel/sdk/metric/metricdata"
)

func TestMetricsConfig(t *testing.T) {
	tests := []struct {
		name         string
		metrics      string
		gcpProject   string
		wantExporter apiotel.ExporterType
		wantEnabled  bool
	}{
		{name: "disabled by default", metrics: "", wantEnabled: false},
		{name: "gcp", metrics: "gcp", gcpProject: "my-project", wantEnabled: true, wantExporter: apiotel.ExporterTypeGCP},
		{name: "otlp", metrics: "otlp", wantEnabled: true, wantExporter: apiotel.ExporterTypeOTLP},
		{name: "prometheus", metrics: "prometheus", wantEnabled: true, wantExporter: apiotel.ExporterTypePrometheus},
		{name: "unknown value disables metrics", metrics: "datadog", wantEnabled: false},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			conf := &config.Config{
				Metrics:                tt.metrics,
				Metrics_Endpoint:       "localhost:4317",
				Metrics_PrometheusAddr: ":9464",
				GCPProject:             tt.gcpProject,
			}

			cfg := metricsConfig(conf)

			assert.Equal(t, tt.wantEnabled, cfg.MetricsEnabled)
			if tt.wantEnabled {
				assert.Equal(t, tt.wantExporter, cfg.MetricsExporterType)
			}
			assert.Equal(t, tt.gcpProject, cfg.GCPProjectID)
			assert.Equal(t, "localhost:4317", cfg.MetricsEndpoint)
			assert.Equal(t, ":9464", cfg.PrometheusAddr)
			assert.Equal(t, tracerServiceName, cfg.ServiceName)
		})
	}
}

func TestNewInstrumentsFallsBackToNoopWhenMeterProviderNil(t *testing.T) {
	in := newInstruments(context.Background(), nil, nil)
	require.NotNil(t, in)
	assert.NotPanics(t, func() {
		in.RecordRequest(context.Background(), "op", 0)
	})
}

func TestNewInstrumentsWiresJobGauges(t *testing.T) {
	reader := sdkmetric.NewManualReader()
	mp := sdkmetric.NewMeterProvider(sdkmetric.WithReader(reader))
	defer func() { assert.NoError(t, mp.Shutdown(context.Background())) }()

	sharedJob := interactor.NewJob(&repo.Container{}, &gateway.Container{}, nil)

	in := newInstruments(context.Background(), mp, sharedJob)
	require.NotNil(t, in)

	var rm metricdata.ResourceMetrics
	require.NoError(t, reader.Collect(context.Background(), &rm))

	found := map[string]bool{}
	for _, sm := range rm.ScopeMetrics {
		for _, m := range sm.Metrics {
			found[m.Name] = true
		}
	}
	assert.True(t, found[apiotel.InstrumentActivePollers], "%s not registered from sharedJob", apiotel.InstrumentActivePollers)
	assert.True(t, found[apiotel.InstrumentMonitoredJobs], "%s not registered from sharedJob", apiotel.InstrumentMonitoredJobs)
}
