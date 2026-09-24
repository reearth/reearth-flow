package app

import (
	"context"

	"github.com/reearth/reearth-flow/api/internal/app/config"
	apiotel "github.com/reearth/reearth-flow/api/internal/app/otel"
	"github.com/reearth/reearth-flow/api/internal/usecase/interactor"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearthx/log"
)

func initMeter(ctx context.Context, conf *config.Config) apiotel.MeterProvider {
	cfg := metricsConfig(conf)

	mp, err := apiotel.InitMeter(ctx, cfg)
	if err != nil {
		log.Fatalf("failed to init meter: %v", err)
	}
	return mp
}

func metricsConfig(conf *config.Config) *apiotel.Config {
	cfg := &apiotel.Config{
		MetricsEndpoint: conf.Metrics_Endpoint,
		PrometheusAddr:  conf.Metrics_PrometheusAddr,
		ServiceName:     tracerServiceName,
	}

	switch conf.Metrics {
	case "gcp":
		cfg.MetricsEnabled = true
		cfg.MetricsExporterType = apiotel.ExporterTypeGCP
		cfg.GCPProjectID = conf.GCPProject
	case "otlp":
		cfg.MetricsEnabled = true
		cfg.MetricsExporterType = apiotel.ExporterTypeOTLP
	case "prometheus":
		cfg.MetricsEnabled = true
		cfg.MetricsExporterType = apiotel.ExporterTypePrometheus
	default:
		cfg.MetricsEnabled = false
	}

	return cfg
}

// newInstruments builds the request/gauge instruments for the API server.
// sharedJob backs the active-pollers/monitored-jobs gauges when it's the
// concrete interactor type; mp defaults to a noop provider when nil, e.g.
// in tests that build a partial ServerConfig.
func newInstruments(ctx context.Context, mp apiotel.MeterProvider, sharedJob interfaces.Job) *apiotel.Instruments {
	if mp == nil {
		var err error
		mp, err = apiotel.InitMeter(ctx, &apiotel.Config{})
		if err != nil {
			log.Errorf("failed to init noop meter: %v", err)
			return nil
		}
	}

	var gauges apiotel.GaugeCallbacks
	if j, ok := sharedJob.(*interactor.Job); ok {
		gauges.ActivePollers = func() int64 { return int64(j.ActivePollerCount()) }
		gauges.MonitoredJobs = func() int64 { return int64(j.MonitoredJobCount()) }
	}

	instruments, err := apiotel.NewInstruments(mp, gauges)
	if err != nil {
		log.Errorf("failed to init metrics instruments: %v", err)
		return nil
	}
	return instruments
}
