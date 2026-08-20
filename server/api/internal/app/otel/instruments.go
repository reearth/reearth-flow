package otel

import (
	"context"
	"fmt"
	"net/http"
	"runtime"
	"sync/atomic"
	"time"

	"go.opentelemetry.io/otel/attribute"
	"go.opentelemetry.io/otel/metric"
)

const meterName = "github.com/reearth/reearth-flow/api"

const (
	// InstrumentRequestDuration is a histogram (ms) of GraphQL request
	// latency, labelled by operation name.
	InstrumentRequestDuration = "reearth_flow_api.graphql.request.duration"
	// InstrumentAccountsCalls is a histogram of accounts-service calls made
	// while serving a single GraphQL request, labelled by operation name.
	InstrumentAccountsCalls = "reearth_flow_api.graphql.accounts_calls"
	// InstrumentRedisCommands is a histogram of Redis commands issued while
	// serving a single GraphQL request, labelled by operation name.
	InstrumentRedisCommands = "reearth_flow_api.graphql.redis_commands"
	// InstrumentActivePollers is a gauge of job-status polling loops
	// currently running.
	InstrumentActivePollers = "reearth_flow_api.jobs.pollers_active"
	// InstrumentMonitoredJobs is a gauge of jobs currently registered for
	// monitoring.
	InstrumentMonitoredJobs = "reearth_flow_api.jobs.monitored"
	// InstrumentGoroutines is a gauge of the current goroutine count.
	InstrumentGoroutines = "reearth_flow_api.process.goroutines"

	// AttrOperation is the bounded-cardinality GraphQL operation name label.
	AttrOperation = "graphql.operation"
)

// requestDurationBoundaries (ms) start below 1us so the hot paths this
// histogram exists to judge (sub-ms) land in non-zero buckets, not just the
// default SDK boundaries meant for whole-millisecond web request latency.
var requestDurationBoundaries = []float64{
	0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1, 2.5, 5, 10, 25, 50, 100, 250, 500, 1000, 2500, 5000,
}

// Instruments holds the deliberately small set of instruments the API
// server records. Operation name is the only per-request label used, to
// keep cardinality bounded.
type Instruments struct {
	RequestDuration metric.Float64Histogram
	AccountsCalls   metric.Int64Histogram
	RedisCommands   metric.Int64Histogram
}

// GaugeCallbacks bundles the read functions backing the observable gauges.
// All are polled at collection time, never cached, so they stay correct as
// the numbers they describe change between scrapes.
type GaugeCallbacks struct {
	ActivePollers func() int64
	MonitoredJobs func() int64
}

func NewInstruments(mp MeterProvider, gauges GaugeCallbacks) (*Instruments, error) {
	meter := mp.Meter(meterName)

	requestDuration, err := meter.Float64Histogram(
		InstrumentRequestDuration,
		metric.WithUnit("ms"),
		metric.WithDescription("GraphQL request latency by operation name"),
		metric.WithExplicitBucketBoundaries(requestDurationBoundaries...),
	)
	if err != nil {
		return nil, fmt.Errorf("otel: failed to create %s: %w", InstrumentRequestDuration, err)
	}

	accountsCalls, err := meter.Int64Histogram(
		InstrumentAccountsCalls,
		metric.WithUnit("{call}"),
		metric.WithDescription("accounts-service calls made per GraphQL request"),
	)
	if err != nil {
		return nil, fmt.Errorf("otel: failed to create %s: %w", InstrumentAccountsCalls, err)
	}

	redisCommands, err := meter.Int64Histogram(
		InstrumentRedisCommands,
		metric.WithUnit("{command}"),
		metric.WithDescription("Redis commands issued per GraphQL request"),
	)
	if err != nil {
		return nil, fmt.Errorf("otel: failed to create %s: %w", InstrumentRedisCommands, err)
	}

	if gauges.ActivePollers != nil {
		if _, err := meter.Int64ObservableGauge(
			InstrumentActivePollers,
			metric.WithDescription("job-status polling loops currently running"),
			metric.WithInt64Callback(func(_ context.Context, o metric.Int64Observer) error {
				o.Observe(gauges.ActivePollers())
				return nil
			}),
		); err != nil {
			return nil, fmt.Errorf("otel: failed to create %s: %w", InstrumentActivePollers, err)
		}
	}

	if gauges.MonitoredJobs != nil {
		if _, err := meter.Int64ObservableGauge(
			InstrumentMonitoredJobs,
			metric.WithDescription("jobs currently registered for monitoring"),
			metric.WithInt64Callback(func(_ context.Context, o metric.Int64Observer) error {
				o.Observe(gauges.MonitoredJobs())
				return nil
			}),
		); err != nil {
			return nil, fmt.Errorf("otel: failed to create %s: %w", InstrumentMonitoredJobs, err)
		}
	}

	if _, err := meter.Int64ObservableGauge(
		InstrumentGoroutines,
		metric.WithDescription("current goroutine count"),
		metric.WithInt64Callback(func(_ context.Context, o metric.Int64Observer) error {
			o.Observe(int64(runtime.NumGoroutine()))
			return nil
		}),
	); err != nil {
		return nil, fmt.Errorf("otel: failed to create %s: %w", InstrumentGoroutines, err)
	}

	return &Instruments{
		RequestDuration: requestDuration,
		AccountsCalls:   accountsCalls,
		RedisCommands:   redisCommands,
	}, nil
}

// countingTransport increments the accounts-calls counter for the request
// in ctx before delegating to next. Wrap the accounts client's transport
// with it via CountingTransport.
type countingTransport struct {
	next http.RoundTripper
}

func (t *countingTransport) RoundTrip(req *http.Request) (*http.Response, error) {
	IncAccountsCall(req.Context())
	return t.next.RoundTrip(req)
}

// CountingTransport wraps next so every request made through it counts
// against InstrumentAccountsCalls for the calling GraphQL request.
func CountingTransport(next http.RoundTripper) http.RoundTripper {
	return &countingTransport{next: next}
}

// requestCounters accumulates accounts/redis calls for a single in-flight
// request. It is attached to the request context (never to a long-lived
// struct) so counts never leak between requests or get shared process-wide.
type requestCounters struct {
	accountsCalls atomic.Int64
	redisCommands atomic.Int64
}

type requestCountersKey struct{}

// WithRequestCounters attaches a fresh per-request counter set to ctx.
func WithRequestCounters(ctx context.Context) context.Context {
	return context.WithValue(ctx, requestCountersKey{}, &requestCounters{})
}

// IncAccountsCall records one accounts-service call against the request in
// ctx. A no-op if ctx carries no counters (e.g. calls made outside a
// GraphQL request).
func IncAccountsCall(ctx context.Context) {
	if c, ok := ctx.Value(requestCountersKey{}).(*requestCounters); ok {
		c.accountsCalls.Add(1)
	}
}

// IncRedisCommand records one Redis command against the request in ctx.
func IncRedisCommand(ctx context.Context) {
	if c, ok := ctx.Value(requestCountersKey{}).(*requestCounters); ok {
		c.redisCommands.Add(1)
	}
}

// requestCountersFrom reads the accumulated counts without resetting them;
// the counter set is discarded with the request context.
func requestCountersFrom(ctx context.Context) (accountsCalls, redisCommands int64) {
	c, ok := ctx.Value(requestCountersKey{}).(*requestCounters)
	if !ok {
		return 0, 0
	}
	return c.accountsCalls.Load(), c.redisCommands.Load()
}

// RecordRequest records the latency, accounts-call count and Redis-command
// count observed for one GraphQL request/operation. Call once per request,
// after the operation has finished.
func (in *Instruments) RecordRequest(ctx context.Context, operation string, duration time.Duration) {
	if in == nil {
		return
	}
	attrs := metric.WithAttributes(attribute.String(AttrOperation, operation))
	in.RequestDuration.Record(ctx, float64(duration)/float64(time.Millisecond), attrs)

	accountsCalls, redisCommands := requestCountersFrom(ctx)
	in.AccountsCalls.Record(ctx, accountsCalls, attrs)
	in.RedisCommands.Record(ctx, redisCommands, attrs)
}
