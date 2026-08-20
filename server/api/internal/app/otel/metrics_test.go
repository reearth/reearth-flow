package otel

import (
	"context"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestInitMeterDisabledByDefault(t *testing.T) {
	mp, err := InitMeter(context.Background(), &Config{})
	require.NoError(t, err)
	require.NotNil(t, mp)

	// A noop provider must still hand out a usable meter/instrument, never nil.
	meter := mp.Meter("test")
	h, err := meter.Float64Histogram("test.histogram")
	require.NoError(t, err)
	h.Record(context.Background(), 1)

	assert.NoError(t, mp.Shutdown(context.Background()))
}

func TestInitMeterUnknownExporter(t *testing.T) {
	_, err := InitMeter(context.Background(), &Config{
		MetricsEnabled:      true,
		MetricsExporterType: ExporterType("datadog"),
	})
	assert.Error(t, err)
}

func TestInitMeterOTLPRequiresEndpoint(t *testing.T) {
	_, err := InitMeter(context.Background(), &Config{
		MetricsEnabled:      true,
		MetricsExporterType: ExporterTypeOTLP,
	})
	assert.Error(t, err)
}

func TestInitMeterPrometheus(t *testing.T) {
	mp, err := InitMeter(context.Background(), &Config{
		MetricsEnabled:      true,
		MetricsExporterType: ExporterTypePrometheus,
		PrometheusAddr:      "127.0.0.1:0",
	})
	require.NoError(t, err)
	require.NotNil(t, mp)

	meter := mp.Meter("test")
	c, err := meter.Int64Counter("test.counter")
	require.NoError(t, err)
	c.Add(context.Background(), 1)

	assert.NoError(t, mp.Shutdown(context.Background()))
}
