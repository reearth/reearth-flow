package circuitbreaker_test

import (
	"context"
	"errors"
	"testing"
	"time"

	"github.com/reearth/reearth-flow/api/pkg/circuitbreaker"
	"github.com/sony/gobreaker/v2"
	"github.com/stretchr/testify/assert"
)

func TestDefaultConfig(t *testing.T) {
	cfg := circuitbreaker.DefaultConfig("svc")
	assert.Equal(t, "svc", cfg.Name)
	assert.Equal(t, circuitbreaker.DefaultConsecutiveFailures, cfg.ConsecutiveFailures)
	assert.Equal(t, circuitbreaker.DefaultTimeout, cfg.Timeout)
	assert.Equal(t, circuitbreaker.DefaultInterval, cfg.Interval)
	assert.Equal(t, circuitbreaker.DefaultMaxRequests, cfg.MaxRequests)
	assert.NotNil(t, cfg.IsSuccessful)
}

func TestDefaultIsSuccessful(t *testing.T) {
	assert.True(t, circuitbreaker.DefaultIsSuccessful(nil))
	assert.True(t, circuitbreaker.DefaultIsSuccessful(context.Canceled))
	assert.True(t, circuitbreaker.DefaultIsSuccessful(context.DeadlineExceeded))
	assert.False(t, circuitbreaker.DefaultIsSuccessful(errors.New("boom")))
}

func TestNew_TripsOnConsecutiveFailures(t *testing.T) {
	cfg := circuitbreaker.DefaultConfig("test")
	cfg.ConsecutiveFailures = 3
	cfg.Timeout = 50 * time.Millisecond
	cfg.Interval = time.Hour // effectively never reset while closed
	cb := circuitbreaker.New[int](cfg)

	failing := func() (int, error) { return 0, errors.New("boom") }

	// First N-1 failures should propagate the original error, breaker stays closed.
	for i := 0; i < 2; i++ {
		_, err := cb.Execute(failing)
		assert.EqualError(t, err, "boom")
		assert.Equal(t, gobreaker.StateClosed, cb.State())
	}

	// N-th failure trips the breaker.
	_, err := cb.Execute(failing)
	assert.EqualError(t, err, "boom")
	assert.Equal(t, gobreaker.StateOpen, cb.State())

	// While open, calls short-circuit with ErrOpen without invoking fn.
	called := false
	_, err = cb.Execute(func() (int, error) {
		called = true
		return 0, nil
	})
	assert.ErrorIs(t, err, circuitbreaker.ErrOpen)
	assert.False(t, called)
}

func TestNew_HalfOpenRecovery(t *testing.T) {
	cfg := circuitbreaker.DefaultConfig("test")
	cfg.ConsecutiveFailures = 1
	cfg.Timeout = 20 * time.Millisecond
	cfg.MaxRequests = 1
	cb := circuitbreaker.New[string](cfg)

	// Trip the breaker.
	_, err := cb.Execute(func() (string, error) { return "", errors.New("boom") })
	assert.EqualError(t, err, "boom")
	assert.Equal(t, gobreaker.StateOpen, cb.State())

	// After Timeout, a probe request should be allowed (half-open) and close
	// the breaker on success.
	time.Sleep(30 * time.Millisecond)
	got, err := cb.Execute(func() (string, error) { return "ok", nil })
	assert.NoError(t, err)
	assert.Equal(t, "ok", got)
	assert.Equal(t, gobreaker.StateClosed, cb.State())
}

func TestNew_ContextCancellationDoesNotTrip(t *testing.T) {
	cfg := circuitbreaker.DefaultConfig("test")
	cfg.ConsecutiveFailures = 2
	cb := circuitbreaker.New[int](cfg)

	// Many context.Canceled results should not trip the breaker because
	// DefaultIsSuccessful classifies them as success.
	for i := 0; i < 10; i++ {
		_, err := cb.Execute(func() (int, error) { return 0, context.Canceled })
		assert.ErrorIs(t, err, context.Canceled)
	}
	assert.Equal(t, gobreaker.StateClosed, cb.State())
}

func TestExecute_ReturnsTypedResult(t *testing.T) {
	cb := circuitbreaker.New[int](circuitbreaker.DefaultConfig("test"))
	got, err := circuitbreaker.Execute(cb, func() (int, error) { return 42, nil })
	assert.NoError(t, err)
	assert.Equal(t, 42, got)
}
