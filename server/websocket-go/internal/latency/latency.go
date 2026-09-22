// Package latency records relay delivery latency and logs percentiles
// periodically.
//
// The service has no metrics instruments and no /metrics endpoint, so logs are the
// only channel every environment already collects. Both relays report through this
// recorder so their arms are comparable; filter on the backend field.
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

// maxSamples bounds retained samples per window. Beyond it we reservoir-sample so
// the distribution stays representative of the whole window instead of collapsing
// to whichever updates arrived first — which on a busy document would be the
// unrepresentative burst right after a flush.
const maxSamples = 8192

// defaultEvery is how often percentiles are emitted. Long enough that a quiet
// document does not spam the log, short enough to see a deploy's effect.
const defaultEvery = 30 * time.Second

// Recorder accumulates observations and logs percentiles on an interval. The zero
// value is not usable; call New.
type Recorder struct {
	log     *slog.Logger
	backend string
	every   time.Duration

	mu      sync.Mutex
	samples []time.Duration
	seen    int64 // total observed this window, including samples not retained
	// max is tracked separately from samples because it is an extreme order
	// statistic: once the reservoir is full a high outlier can be evicted, and the
	// maximum of the retained sample is not the maximum of the window. Percentiles
	// survive sampling; the maximum does not, and it is the field most likely to be
	// read as "the worst this backend did".
	max time.Duration
}

// New builds a Recorder labelled with the backend it measures. A nil logger
// disables reporting but keeps Observe safe to call, so callers need no nil checks.
func New(log *slog.Logger, backend string) *Recorder {
	return &Recorder{log: log, backend: backend, every: defaultEvery}
}

// Observe records one delivery latency. Safe for concurrent use; negative values
// are dropped, since they can only come from clock skew and would drag a percentile
// below what any update actually experienced.
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

// Run emits a report every interval until ctx is cancelled, and one final report on
// the way out so a short-lived process still reports what it saw.
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

// report logs the window's percentiles and resets. A window with no deliveries logs
// nothing: an idle document should be silent, not a stream of zeroes.
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

// drain takes the window's samples, total and maximum, and resets the accumulator.
func (r *Recorder) drain() ([]time.Duration, int64, time.Duration) {
	r.mu.Lock()
	defer r.mu.Unlock()
	s, seen, max := r.samples, r.seen, r.max
	r.samples, r.seen, r.max = nil, 0, 0
	return s, seen, max
}

// percentile returns the p-th percentile of a sorted slice by nearest rank:
// rank = ceil(n*p), taken as a 1-based position. It needs no interpolation and can
// only ever report a value some observation actually achieved.
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

// ms renders a duration in milliseconds with microsecond resolution. Whole
// milliseconds would floor most of this service's latencies to 0, which is the
// exact failure of the existing HTTP timing this package works around.
func ms(d time.Duration) float64 {
	return float64(d.Microseconds()) / 1000
}
