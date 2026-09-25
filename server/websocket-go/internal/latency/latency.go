// Package latency records relay delivery latency and logs percentiles every 30s.
// Logs, not metrics: the service has no /metrics endpoint. Both relays report here
// so their arms are comparable — filter on the backend field.
package latency

import (
	"context"
	"log/slog"
	"math"
	"math/rand"
	"sort"
	"sync"
	"time"
)

// maxSamples bounds retained samples; beyond it we reservoir-sample, so a busy
// window stays representative rather than truncating to its first arrivals.
const maxSamples = 8192

// defaultEvery is the report interval.
const defaultEvery = 30 * time.Second

// Recorder accumulates observations and logs percentiles. Call New; the zero value
// is not usable.
type Recorder struct {
	log     *slog.Logger
	backend string
	every   time.Duration

	mu      sync.Mutex
	samples []time.Duration
	seen    int64 // total observed this window, including samples not retained
	// max is tracked separately: percentiles survive reservoir sampling, an extreme
	// does not — a sampled max under-reports the worst the backend actually did.
	max time.Duration
}

// New builds a Recorder for one backend. A nil logger disables reporting but keeps
// Observe safe, so callers need no nil checks.
func New(log *slog.Logger, backend string) *Recorder {
	return &Recorder{log: log, backend: backend, every: defaultEvery}
}

// Observe records one delivery latency. Concurrency-safe. Negatives are dropped:
// they can only be clock skew, and would pull a percentile below any real sample.
func (r *Recorder) Observe(d time.Duration) {
	if r == nil || d < 0 {
		return
	}
	r.mu.Lock()
	defer r.mu.Unlock()

	r.seen++
	if d > r.max {
		r.max = d
	}
	if len(r.samples) < maxSamples {
		r.samples = append(r.samples, d)
		return
	}
	// Reservoir sampling: keep each observation with probability maxSamples/seen.
	if i := rand.Int63n(r.seen); i < maxSamples {
		r.samples[i] = d
	}
}

// Run reports every interval until ctx ends, plus a final one on the way out.
func (r *Recorder) Run(ctx context.Context) {
	if r == nil || r.log == nil {
		return
	}
	t := time.NewTicker(r.every)
	defer t.Stop()
	for {
		select {
		case <-ctx.Done():
			r.report()
			return
		case <-t.C:
			r.report()
		}
	}
}

// report logs the window's percentiles and resets. An empty window logs nothing.
func (r *Recorder) report() {
	s, seen, max := r.drain()
	if len(s) == 0 {
		return
	}
	sort.Slice(s, func(i, j int) bool { return s[i] < s[j] })

	r.log.Info("relay latency",
		"backend", r.backend,
		"count", seen,
		"sampled", len(s),
		"p50_ms", ms(percentile(s, 0.50)),
		"p95_ms", ms(percentile(s, 0.95)),
		"p99_ms", ms(percentile(s, 0.99)),
		"max_ms", ms(max),
	)
}

// drain takes the window's samples, total and maximum, and resets.
func (r *Recorder) drain() ([]time.Duration, int64, time.Duration) {
	r.mu.Lock()
	defer r.mu.Unlock()
	s, seen, max := r.samples, r.seen, r.max
	r.samples, r.seen, r.max = nil, 0, 0
	return s, seen, max
}

// percentile by nearest rank: ceil(n*p) as a 1-based position. Scaling by n-1
// instead rounds the tail down and hides it on small windows.
func percentile(sorted []time.Duration, p float64) time.Duration {
	if len(sorted) == 0 {
		return 0
	}
	rank := int(math.Ceil(float64(len(sorted)) * p))
	if rank < 1 {
		rank = 1
	}
	if rank > len(sorted) {
		rank = len(sorted)
	}
	return sorted[rank-1]
}

// ms renders milliseconds at microsecond resolution; whole ms would floor most of
// this service's latencies to 0.
func ms(d time.Duration) float64 {
	return float64(d.Microseconds()) / 1000
}
