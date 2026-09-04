package app

import (
	"context"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"

	"github.com/redis/go-redis/v9"
	apiotel "github.com/reearth/reearth-flow/api/internal/app/otel"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	sdkmetric "go.opentelemetry.io/otel/sdk/metric"
	"go.opentelemetry.io/otel/sdk/metric/metricdata"
)

func newTestInstruments(t *testing.T) (*apiotel.Instruments, *sdkmetric.ManualReader) {
	t.Helper()
	reader := sdkmetric.NewManualReader()
	mp := sdkmetric.NewMeterProvider(sdkmetric.WithReader(reader))
	t.Cleanup(func() { assert.NoError(t, mp.Shutdown(context.Background())) })

	in, err := apiotel.NewInstruments(mp, apiotel.GaugeCallbacks{})
	require.NoError(t, err)
	return in, reader
}

func histogramSum(t *testing.T, reader *sdkmetric.ManualReader, name string) (int64, bool) {
	t.Helper()
	var rm metricdata.ResourceMetrics
	require.NoError(t, reader.Collect(context.Background(), &rm))
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

func TestRedisCommandCounterHook_ProcessHookCountsPerCommand(t *testing.T) {
	in, reader := newTestInstruments(t)
	hook := redisCommandCounterHook{}

	var calls int
	next := hook.ProcessHook(func(ctx context.Context, cmd redis.Cmder) error {
		calls++
		return nil
	})

	ctx := apiotel.WithRequestCounters(context.Background())
	require.NoError(t, next(ctx, redis.NewStatusCmd(ctx)))
	require.NoError(t, next(ctx, redis.NewStatusCmd(ctx)))
	assert.Equal(t, 2, calls)

	in.RecordRequest(ctx, "op", time.Millisecond)
	sum, ok := histogramSum(t, reader, apiotel.InstrumentRedisCommands)
	require.True(t, ok)
	assert.Equal(t, int64(2), sum)
}

func TestRedisCommandCounterHook_ProcessPipelineHookCountsEachCommand(t *testing.T) {
	in, reader := newTestInstruments(t)
	hook := redisCommandCounterHook{}

	var seen int
	next := hook.ProcessPipelineHook(func(ctx context.Context, cmds []redis.Cmder) error {
		seen = len(cmds)
		return nil
	})

	ctx := apiotel.WithRequestCounters(context.Background())
	cmds := []redis.Cmder{redis.NewStatusCmd(ctx), redis.NewStatusCmd(ctx), redis.NewStatusCmd(ctx)}
	require.NoError(t, next(ctx, cmds))
	assert.Equal(t, 3, seen)

	in.RecordRequest(ctx, "op", time.Millisecond)
	sum, ok := histogramSum(t, reader, apiotel.InstrumentRedisCommands)
	require.True(t, ok)
	assert.Equal(t, int64(3), sum)
}

// An empty pipeline must record zero commands explicitly, not skip the
// metric point for the request.
func TestRedisCommandCounterHook_EmptyPipelineRecordsZero(t *testing.T) {
	in, reader := newTestInstruments(t)
	hook := redisCommandCounterHook{}

	next := hook.ProcessPipelineHook(func(ctx context.Context, cmds []redis.Cmder) error {
		return nil
	})

	ctx := apiotel.WithRequestCounters(context.Background())
	require.NoError(t, next(ctx, nil))

	in.RecordRequest(ctx, "op", time.Millisecond)
	sum, ok := histogramSum(t, reader, apiotel.InstrumentRedisCommands)
	require.True(t, ok, "redis commands metric point must exist even with zero commands")
	assert.Equal(t, int64(0), sum)
}

func TestCountingTransport_CountsAccountsCalls(t *testing.T) {
	in, reader := newTestInstruments(t)

	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusOK)
	}))
	defer srv.Close()

	client := &http.Client{Transport: apiotel.CountingTransport(http.DefaultTransport)}

	ctx := apiotel.WithRequestCounters(context.Background())
	for i := 0; i < 2; i++ {
		req, err := http.NewRequestWithContext(ctx, http.MethodGet, srv.URL, nil)
		require.NoError(t, err)
		resp, err := client.Do(req)
		require.NoError(t, err)
		_ = resp.Body.Close()
	}

	in.RecordRequest(ctx, "op", time.Millisecond)
	sum, ok := histogramSum(t, reader, apiotel.InstrumentAccountsCalls)
	require.True(t, ok)
	assert.Equal(t, int64(2), sum)
}
