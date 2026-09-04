package app

import (
	"context"
	"testing"

	"github.com/99designs/gqlgen/graphql"
	apiotel "github.com/reearth/reearth-flow/api/internal/app/otel"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"go.opentelemetry.io/otel/attribute"
	sdkmetric "go.opentelemetry.io/otel/sdk/metric"
	"go.opentelemetry.io/otel/sdk/metric/metricdata"
)

func histogramDataPoint(t *testing.T, reader *sdkmetric.ManualReader, name string) (attrs map[string]string, sum float64, found bool) {
	t.Helper()
	var rm metricdata.ResourceMetrics
	require.NoError(t, reader.Collect(context.Background(), &rm))
	for _, sm := range rm.ScopeMetrics {
		for _, m := range sm.Metrics {
			if m.Name != name {
				continue
			}
			switch data := m.Data.(type) {
			case metricdata.Histogram[float64]:
				if len(data.DataPoints) == 0 {
					continue
				}
				dp := data.DataPoints[0]
				return attrsToMap(dp.Attributes), dp.Sum, true
			case metricdata.Histogram[int64]:
				if len(data.DataPoints) == 0 {
					continue
				}
				dp := data.DataPoints[0]
				return attrsToMap(dp.Attributes), float64(dp.Sum), true
			}
		}
	}
	return nil, 0, false
}

func attrsToMap(set attribute.Set) map[string]string {
	m := map[string]string{}
	iter := set.Iter()
	for iter.Next() {
		kv := iter.Attribute()
		m[string(kv.Key)] = kv.Value.Emit()
	}
	return m
}

func TestMetricsExtension_RecordsOperationNameOnce(t *testing.T) {
	reader := sdkmetric.NewManualReader()
	mp := sdkmetric.NewMeterProvider(sdkmetric.WithReader(reader))
	defer func() { assert.NoError(t, mp.Shutdown(context.Background())) }()

	in, err := apiotel.NewInstruments(mp, apiotel.GaugeCallbacks{})
	require.NoError(t, err)

	ext := &metricsExtension{instruments: in}

	// gqlgen's real DispatchOperation captures the ctx it hands to the
	// terminal OperationHandler and reuses it for every ResponseHandler
	// invocation, so requests carry the counters InterceptOperation attaches.
	var invocations int
	var enrichedCtx context.Context
	next := func(ctx context.Context) graphql.ResponseHandler {
		enrichedCtx = ctx
		return func(ctx context.Context) *graphql.Response {
			invocations++
			apiotel.IncAccountsCall(ctx)
			return &graphql.Response{}
		}
	}

	opCtx := &graphql.OperationContext{OperationName: "GetProject"}
	ctx := graphql.WithOperationContext(context.Background(), opCtx)

	handler := ext.InterceptOperation(ctx, next)
	// Simulate a subscription-style multi-invocation: metrics must be
	// recorded exactly once, not once per response.
	handler(enrichedCtx)
	handler(enrichedCtx)
	assert.Equal(t, 2, invocations)

	attrs, _, ok := histogramDataPoint(t, reader, apiotel.InstrumentRequestDuration)
	require.True(t, ok)
	assert.Equal(t, "GetProject", attrs[apiotel.AttrOperation])

	_, accountsSum, ok := histogramDataPoint(t, reader, apiotel.InstrumentAccountsCalls)
	require.True(t, ok)
	assert.Equal(t, float64(1), accountsSum)
}

func TestMetricsExtension_AnonymousOperationUsesBoundedLabel(t *testing.T) {
	reader := sdkmetric.NewManualReader()
	mp := sdkmetric.NewMeterProvider(sdkmetric.WithReader(reader))
	defer func() { assert.NoError(t, mp.Shutdown(context.Background())) }()

	in, err := apiotel.NewInstruments(mp, apiotel.GaugeCallbacks{})
	require.NoError(t, err)

	ext := &metricsExtension{instruments: in}
	next := func(ctx context.Context) graphql.ResponseHandler {
		return func(ctx context.Context) *graphql.Response { return &graphql.Response{} }
	}

	opCtx := &graphql.OperationContext{}
	ctx := graphql.WithOperationContext(context.Background(), opCtx)

	ext.InterceptOperation(ctx, next)(ctx)

	attrs, _, ok := histogramDataPoint(t, reader, apiotel.InstrumentRequestDuration)
	require.True(t, ok)
	assert.Equal(t, "unknown", attrs[apiotel.AttrOperation])
}

func TestMetricsExtension_NilInstrumentsPassesThrough(t *testing.T) {
	ext := &metricsExtension{}
	var called bool
	next := func(ctx context.Context) graphql.ResponseHandler {
		return func(ctx context.Context) *graphql.Response {
			called = true
			return &graphql.Response{}
		}
	}

	handler := ext.InterceptOperation(context.Background(), next)
	assert.NotPanics(t, func() { handler(context.Background()) })
	assert.True(t, called)
}
