package main

import (
	"context"
	"errors"
	"fmt"
	"testing"
	"time"

	"github.com/reearth/reearth-flow/websocket-go/internal/gcs"
	"github.com/reearth/reearth-flow/websocket-go/internal/versions"
)

// busyLocker times out its first busy acquisitions, then runs fn.
type busyLocker struct {
	busy  int
	tries int
}

func (l *busyLocker) WithLock(ctx context.Context, _ string, fn func(context.Context) error) error {
	l.tries++
	if l.tries <= l.busy {
		return gcs.ErrLockTimeout
	}
	return fn(ctx)
}

func (l *busyLocker) TryWithLock(ctx context.Context, _ string, _ time.Duration, fn func(context.Context) error) error {
	return fn(ctx)
}

func TestLockWithin_WaitsPastTheLockStoresBudget(t *testing.T) {
	t.Parallel()

	l := &busyLocker{busy: 3}
	ran := 0
	err := lockWithin(context.Background(), l, "k", time.Minute, func(context.Context) error { ran++; return nil })
	if err != nil || ran != 1 || l.tries != 4 {
		t.Fatalf("err %v, ran %d, tries %d; want one run after three timeouts", err, ran, l.tries)
	}
}

func TestLockWithin_ReportsBusyAfterItsWait(t *testing.T) {
	t.Parallel()

	l := &busyLocker{busy: 1 << 30}
	err := lockWithin(context.Background(), l, "k", 0, func(context.Context) error {
		t.Fatal("fn ran without the lock")
		return nil
	})
	if !errors.Is(err, versions.ErrBusy) || !errors.Is(err, gcs.ErrLockTimeout) {
		t.Fatalf("err = %v, want ErrBusy wrapping the timeout", err)
	}
}

func TestLockWithin_NeverRetriesWhatFnReturned(t *testing.T) {
	t.Parallel()

	l := &busyLocker{}
	ran := 0
	err := lockWithin(context.Background(), l, "k", time.Minute, func(context.Context) error {
		ran++
		return fmt.Errorf("checkpoint: %w", gcs.ErrLockTimeout) // fn's own lock, not the version lock
	})
	if ran != 1 || errors.Is(err, versions.ErrBusy) || !errors.Is(err, gcs.ErrLockTimeout) {
		t.Fatalf("ran %d, err %v; fn must run once and its error come back as is", ran, err)
	}
}
