package app

import (
	"testing"

	"github.com/reearth/reearth-flow/api/internal/app/config"
	apiotel "github.com/reearth/reearth-flow/api/internal/app/otel"
	"github.com/stretchr/testify/assert"
)

func TestTracerConfig(t *testing.T) {
	tests := []struct {
		name         string
		tracer       string
		gcpProject   string
		wantExporter apiotel.ExporterType
		sample       float64
		wantRatio    float64
		wantEnabled  bool
	}{
		{name: "disabled by default", tracer: "", wantEnabled: false},
		{name: "gcp", tracer: "gcp", sample: 0.5, gcpProject: "my-project", wantEnabled: true, wantExporter: apiotel.ExporterTypeGCP, wantRatio: 0.5},
		{name: "jaeger", tracer: "jaeger", sample: 0.5, wantEnabled: true, wantExporter: apiotel.ExporterTypeJaeger, wantRatio: 0.5},
		{name: "jaeger defaults to full sampling", tracer: "jaeger", sample: 0, wantEnabled: true, wantExporter: apiotel.ExporterTypeJaeger, wantRatio: 1},
		{name: "unknown value disables tracing", tracer: "datadog", wantEnabled: false},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			conf := &config.Config{
				Tracer:          tt.tracer,
				TracerSample:    tt.sample,
				Tracer_Endpoint: "localhost:4317",
				GCPProject:      tt.gcpProject,
			}

			cfg := tracerConfig(conf)

			assert.Equal(t, tt.wantEnabled, cfg.Enabled)
			if tt.wantEnabled {
				assert.Equal(t, tt.wantExporter, cfg.ExporterType)
			}
			assert.Equal(t, tt.wantRatio, cfg.SamplingRatio)
			assert.Equal(t, tt.gcpProject, cfg.GCPProjectID)
			assert.Equal(t, "localhost:4317", cfg.Endpoint)
			assert.Equal(t, tracerServiceName, cfg.ServiceName)
		})
	}
}
