package otel

import (
	"context"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	sdkmetric "go.opentelemetry.io/otel/sdk/metric"
	"go.opentelemetry.io/otel/sdk/metric/metricdata"
)

func collect(t *testing.T, reader *sdkmetric.ManualReader) metricdata.ResourceMetrics {
	t.Helper()
	var rm metricdata.ResourceMetrics
	require.NoError(t, reader.Collect(context.Background(), &rm))
	return rm
}

func gaugeValue(rm metricdata.ResourceMetrics, name string) (int64, bool) {
	for _, sm := range rm.ScopeMetrics {
		for _, m := range sm.Metrics {
			if m.Name != name {
				continue
			}
			if data, ok := m.Data.(metricdata.Gauge[int64]); ok && len(data.DataPoints) > 0 {
				return data.DataPoints[0].Value, true
			}
		}
	}
	return 0, false
}

func histogramSum(rm metricdata.ResourceMetrics, name string) (int64, bool) {
	for _, sm := range rm.ScopeMetrics {
		for _, m := range sm.Metrics {
			if m.Name != name {
				continue
			}
			if data, ok := m.Data.(metricdata.Histogram[int64]); ok && len(data.DataPoints) > 0 {
				return data.DataPoints[0].Sum, true
			}
		}
	}
	return 0, false
}

func TestNewInstrumentsGaugesReflectCallbacks(t *testing.T) {
	reader := sdkmetric.NewManualReader()
	mp := sdkmetric.NewMeterProvider(sdkmetric.WithReader(reader))
	defer func() { assert.NoError(t, mp.Shutdown(context.Background())) }()

	pollers, jobs := int64(3), int64(5)
	_, err := NewInstruments(mp, GaugeCallbacks{
		ActivePollers: func() int64 { return pollers },
		MonitoredJobs: func() int64 { return jobs },
	})
	require.NoError(t, err)

	rm := collect(t, reader)

	got, ok := gaugeValue(rm, InstrumentActivePollers)
	require.True(t, ok, "%s not reported", InstrumentActivePollers)
	assert.Equal(t, pollers, got)

	got, ok = gaugeValue(rm, InstrumentMonitoredJobs)
	require.True(t, ok, "%s not reported", InstrumentMonitoredJobs)
	assert.Equal(t, jobs, got)

	_, ok = gaugeValue(rm, InstrumentGoroutines)
	assert.True(t, ok, "%s not reported", InstrumentGoroutines)
}

func TestNewInstrumentsGaugesZeroIsReportedNotOmitted(t *testing.T) {
	reader := sdkmetric.NewManualReader()
	mp := sdkmetric.NewMeterProvider(sdkmetric.WithReader(reader))
	defer func() { assert.NoError(t, mp.Shutdown(context.Background())) }()

	_, err := NewInstruments(mp, GaugeCallbacks{
		ActivePollers: func() int64 { return 0 },
		MonitoredJobs: func() int64 { return 0 },
	})
	require.NoError(t, err)

	rm := collect(t, reader)

	got, ok := gaugeValue(rm, InstrumentActivePollers)
	require.True(t, ok, "gauge with zero value must still be reported")
	assert.Equal(t, int64(0), got)
}

func TestNewInstrumentsNilCallbacksSkipGaugeRegistration(t *testing.T) {
	reader := sdkmetric.NewManualReader()
	mp := sdkmetric.NewMeterProvider(sdkmetric.WithReader(reader))
	defer func() { assert.NoError(t, mp.Shutdown(context.Background())) }()

	_, err := NewInstruments(mp, GaugeCallbacks{})
	require.NoError(t, err)

	rm := collect(t, reader)
	_, ok := gaugeValue(rm, InstrumentActivePollers)
	assert.False(t, ok)
	_, ok = gaugeValue(rm, InstrumentMonitoredJobs)
	assert.False(t, ok)
}

func TestRequestCountersRoundTrip(t *testing.T) {
	ctx := WithRequestCounters(context.Background())
	IncAccountsCall(ctx)
	IncAccountsCall(ctx)
	IncRedisCommand(ctx)

	accounts, redis := requestCountersFrom(ctx)
	assert.Equal(t, int64(2), accounts)
	assert.Equal(t, int64(1), redis)
}

func TestRequestCountersNoopWithoutAttachedContext(t *testing.T) {
	ctx := context.Background()

	assert.NotPanics(t, func() {
		IncAccountsCall(ctx)
		IncRedisCommand(ctx)
	})

	accounts, redis := requestCountersFrom(ctx)
	assert.Zero(t, accounts)
	assert.Zero(t, redis)
}

// A request that made zero accounts/Redis calls must still record an
// explicit zero observation, not silently skip the metric point.
func TestRecordRequestZeroCallsAreRecordedNotSkipped(t *testing.T) {
	reader := sdkmetric.NewManualReader()
	mp := sdkmetric.NewMeterProvider(sdkmetric.WithReader(reader))
	defer func() { assert.NoError(t, mp.Shutdown(context.Background())) }()

	in, err := NewInstruments(mp, GaugeCallbacks{})
	require.NoError(t, err)

	ctx := WithRequestCounters(context.Background())
	in.RecordRequest(ctx, "GetProject", 12*time.Millisecond)

	rm := collect(t, reader)

	sum, ok := histogramSum(rm, InstrumentAccountsCalls)
	require.True(t, ok, "%s data point missing for zero-call request", InstrumentAccountsCalls)
	assert.Equal(t, int64(0), sum)

	sum, ok = histogramSum(rm, InstrumentRedisCommands)
	require.True(t, ok, "%s data point missing for zero-call request", InstrumentRedisCommands)
	assert.Equal(t, int64(0), sum)

	durationSum, ok := func() (float64, bool) {
		for _, sm := range rm.ScopeMetrics {
			for _, m := range sm.Metrics {
				if m.Name != InstrumentRequestDuration {
					continue
				}
				if data, ok := m.Data.(metricdata.Histogram[float64]); ok && len(data.DataPoints) > 0 {
					return data.DataPoints[0].Sum, true
				}
			}
		}
		return 0, false
	}()
	require.True(t, ok)
	assert.InDelta(t, 12, durationSum, 0.001)
}

func TestRecordRequestCountsCallsMade(t *testing.T) {
	reader := sdkmetric.NewManualReader()
	mp := sdkmetric.NewMeterProvider(sdkmetric.WithReader(reader))
	defer func() { assert.NoError(t, mp.Shutdown(context.Background())) }()

	in, err := NewInstruments(mp, GaugeCallbacks{})
	require.NoError(t, err)

	ctx := WithRequestCounters(context.Background())
	IncAccountsCall(ctx)
	IncAccountsCall(ctx)
	IncAccountsCall(ctx)
	IncRedisCommand(ctx)
	in.RecordRequest(ctx, "ListDeployments", time.Millisecond)

	rm := collect(t, reader)

	sum, ok := histogramSum(rm, InstrumentAccountsCalls)
	require.True(t, ok)
	assert.Equal(t, int64(3), sum)

	sum, ok = histogramSum(rm, InstrumentRedisCommands)
	require.True(t, ok)
	assert.Equal(t, int64(1), sum)
}

func TestRecordRequestNilInstrumentsIsNoop(t *testing.T) {
	var in *Instruments
	assert.NotPanics(t, func() {
		in.RecordRequest(context.Background(), "op", time.Millisecond)
	})
}
