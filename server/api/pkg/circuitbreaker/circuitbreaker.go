// Package circuitbreaker provides a thin wrapper around sony/gobreaker/v2
// so that outbound-dependency clients (gRPC, HTTP, etc.) can be short-circuited
// when a downstream service is unhealthy, instead of piling up doomed requests.
//
// The wrapper is intentionally minimal:
//
//   - a Config with sensible defaults suitable for RPC-style traffic,
//   - a NewCircuitBreaker constructor that logs state transitions,
//   - a generic Execute helper so callers can wrap a typed function without
//     importing gobreaker directly.
//
// Callers who need finer control can still build a *gobreaker.CircuitBreaker[T]
// themselves via gobreaker directly; this package's goal is a shared,
// consistent default across the codebase.
package circuitbreaker

import (
	"context"
	"errors"
	"time"

	"github.com/reearth/reearthx/log"
	"github.com/sony/gobreaker/v2"
)

// ErrOpen is returned to the caller when a request is rejected because the
// underlying breaker is open. It wraps gobreaker.ErrOpenState so callers can
// keep using errors.Is against either sentinel.
var ErrOpen = gobreaker.ErrOpenState

// ErrTooManyRequests mirrors gobreaker.ErrTooManyRequests for the half-open
// state and is re-exported so callers do not need to import gobreaker.
var ErrTooManyRequests = gobreaker.ErrTooManyRequests

// Config configures a circuit breaker. Zero values mean "use the default";
// see DefaultConfig for the values applied.
type Config struct {
	// Name identifies the breaker in logs and metrics. Required in practice
	// but not enforced — an empty name simply produces less useful log lines.
	Name string

	// MaxRequests is the maximum number of requests allowed to pass through
	// while the breaker is in the half-open state. Zero means 1.
	MaxRequests uint32

	// Interval is the cyclic period over which the closed-state counters are
	// cleared. Zero means the counters never reset while closed.
	Interval time.Duration

	// Timeout is how long the breaker stays open before transitioning to
	// half-open. Zero means 60 seconds (gobreaker's default).
	Timeout time.Duration

	// ConsecutiveFailures is the number of consecutive failures required to
	// trip the breaker. Zero means DefaultConsecutiveFailures.
	ConsecutiveFailures uint32

	// IsSuccessful reports whether the given error should count as a success.
	// A nil error is always a success; this hook lets callers exclude e.g.
	// context.Canceled / gRPC codes that represent client-side aborts rather
	// than real downstream failures. If nil, DefaultIsSuccessful is used.
	IsSuccessful func(err error) bool
}

// DefaultConsecutiveFailures is the default trip threshold. It is deliberately
// higher than gobreaker's stock value of 5 because a single flaky request
// tripping the breaker for a whole service is more disruptive than useful.
const DefaultConsecutiveFailures uint32 = 10

// DefaultTimeout is the default open→half-open cooldown.
const DefaultTimeout = 30 * time.Second

// DefaultInterval is the default rolling window over which closed-state
// counters are cleared, so a slow trickle of unrelated errors does not
// eventually trip the breaker.
const DefaultInterval = 60 * time.Second

// DefaultMaxRequests is the default number of probe requests permitted in the
// half-open state.
const DefaultMaxRequests uint32 = 1

// DefaultIsSuccessful treats context cancellation / deadline exceeded as
// successes, since they represent caller-side aborts rather than downstream
// failure. Everything else counts as a failure.
func DefaultIsSuccessful(err error) bool {
	if err == nil {
		return true
	}
	if errors.Is(err, context.Canceled) || errors.Is(err, context.DeadlineExceeded) {
		return true
	}
	return false
}

// DefaultConfig returns a Config populated with the package defaults for the
// given name. Callers can override individual fields on the returned value.
func DefaultConfig(name string) Config {
	return Config{
		Name:                name,
		MaxRequests:         DefaultMaxRequests,
		Interval:            DefaultInterval,
		Timeout:             DefaultTimeout,
		ConsecutiveFailures: DefaultConsecutiveFailures,
		IsSuccessful:        DefaultIsSuccessful,
	}
}

// New builds a *gobreaker.CircuitBreaker[T] from cfg. It fills in package
// defaults for zero-valued fields and installs a log-based OnStateChange hook.
func New[T any](cfg Config) *gobreaker.CircuitBreaker[T] {
	settings := settingsFrom(cfg)
	return gobreaker.NewCircuitBreaker[T](settings)
}

// Execute is a convenience helper that runs fn under cb and returns the
// typed result. It is equivalent to cb.Execute(fn) and exists so callers do
// not have to spell out the generic parameter twice.
func Execute[T any](cb *gobreaker.CircuitBreaker[T], fn func() (T, error)) (T, error) {
	return cb.Execute(fn)
}

func settingsFrom(cfg Config) gobreaker.Settings {
	name := cfg.Name
	maxRequests := cfg.MaxRequests
	if maxRequests == 0 {
		maxRequests = DefaultMaxRequests
	}
	interval := cfg.Interval
	if interval == 0 {
		interval = DefaultInterval
	}
	timeout := cfg.Timeout
	if timeout == 0 {
		timeout = DefaultTimeout
	}
	trip := cfg.ConsecutiveFailures
	if trip == 0 {
		trip = DefaultConsecutiveFailures
	}
	isSuccessful := cfg.IsSuccessful
	if isSuccessful == nil {
		isSuccessful = DefaultIsSuccessful
	}

	return gobreaker.Settings{
		Name:        name,
		MaxRequests: maxRequests,
		Interval:    interval,
		Timeout:     timeout,
		ReadyToTrip: func(counts gobreaker.Counts) bool {
			return counts.ConsecutiveFailures >= trip
		},
		OnStateChange: func(name string, from gobreaker.State, to gobreaker.State) {
			log.Warnf("circuitbreaker %q state change: %s -> %s", name, from, to)
		},
		IsSuccessful: isSuccessful,
	}
}
