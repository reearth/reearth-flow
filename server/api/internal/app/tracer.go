package app

import (
	"context"
	"time"

	"github.com/reearth/reearth-flow/api/internal/app/config"
	apiotel "github.com/reearth/reearth-flow/api/internal/app/otel"
	"github.com/reearth/reearthx/log"
)

const (
	tracerServiceName = "reearth-flow-api"

	defaultMaxExportBatchSize = 512
	defaultBatchTimeout       = 5 * time.Second
	defaultMaxQueueSize       = 2048
)

func initTracer(ctx context.Context, conf *config.Config) apiotel.TracerProvider {
	cfg := tracerConfig(conf)

	tp, err := apiotel.InitTracer(ctx, cfg)
	if err != nil {
		log.Fatalf("failed to init tracer: %v", err)
	}
	return tp
}

func tracerConfig(conf *config.Config) *apiotel.Config {
	cfg := &apiotel.Config{
		Endpoint:           conf.Tracer_Endpoint,
		SamplingRatio:      conf.TracerSample,
		ServiceName:        tracerServiceName,
		MaxExportBatchSize: defaultMaxExportBatchSize,
		BatchTimeout:       defaultBatchTimeout,
		MaxQueueSize:       defaultMaxQueueSize,
	}

	switch conf.Tracer {
	case "gcp":
		cfg.Enabled = true
		cfg.ExporterType = apiotel.ExporterTypeGCP
		cfg.GCPProjectID = conf.GCPProject
	case "jaeger":
		cfg.Enabled = true
		cfg.ExporterType = apiotel.ExporterTypeJaeger
		// the previous Jaeger tracer always sampled; keep that when no ratio is set
		if conf.TracerSample == 0 {
			cfg.SamplingRatio = 1
		}
	case "otlp":
		cfg.Enabled = true
		cfg.ExporterType = apiotel.ExporterTypeOTLP
	default:
		cfg.Enabled = false
	}

	return cfg
}
