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
	s, seen := r.drain()
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
		"max_ms", ms(s[len(s)-1]),
	)
}

// drain takes the window's samples and resets the accumulator.
func (r *Recorder) drain() ([]time.Duration, int64) {
	r.mu.Lock()
	defer r.mu.Unlock()
	s, seen := r.samples, r.seen
	r.samples, r.seen = nil, 0
	return s, seen
}

// percentile returns the p-th percentile of a sorted slice using nearest-rank,
// which needs no interpolation and cannot report a value no sample achieved.
func percentile(sorted []time.Duration, p float64) time.Duration {
	if len(sorted) == 0 {
		return 0
	}
	i := int(float64(len(sorted)-1) * p)
	if i < 0 {
		i = 0
	}
	if i >= len(sorted) {
		i = len(sorted) - 1
	}
	return sorted[i]
}

// ms renders a duration in milliseconds with microsecond resolution. Whole
// milliseconds would floor most of this service's latencies to 0, which is the
// exact failure of the existing HTTP timing this package works around.
func ms(d time.Duration) float64 {
	return float64(d.Microseconds()) / 1000
}
