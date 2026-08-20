package gcs

import (
	"context"
	"errors"
	"time"

	goredis "github.com/redis/go-redis/v9"
)

// Locker is the adapter's mutual-exclusion primitive. WithLock runs fn while
// holding key, and MUST bound its own acquisition (retry cap + timeout) so the
// connect path can never deadlock.
type Locker interface {
	WithLock(ctx context.Context, key string, fn func(context.Context) error) error
}

// noLock is a single-process Locker that runs fn immediately.
type noLock struct{}

// NewNoLock returns a Locker that does no cross-process locking (single process).
func NewNoLock() Locker { return noLock{} }

func (noLock) WithLock(ctx context.Context, _ string, fn func(context.Context) error) error {
	return fn(ctx)
}

// RedisLocker is a SET NX PX-based lock, bounded by oidLockRetries × oidLockDelay.
// It never spins unbounded.
type RedisLocker struct {
	client  *goredis.Client
	value   string // per-process owner token
	ttl     time.Duration
	retries int
	delay   time.Duration
}

const (
	oidLockTTL     = 10 * time.Second
	oidLockRetries = 10
	oidLockDelay   = 500 * time.Millisecond
)

// NewRedisLocker builds a bounded redis Locker. owner is a per-process unique
// token (e.g. "instance-{clientId}") so only the holder releases the lock.
func NewRedisLocker(client *goredis.Client, owner string) *RedisLocker {
	return &RedisLocker{
		client:  client,
		value:   owner,
		ttl:     oidLockTTL,
		retries: oidLockRetries,
		delay:   oidLockDelay,
	}
}

// ErrLockTimeout is returned when the lock could not be acquired within the
// bounded retry budget.
var ErrLockTimeout = errors.New("gcs: lock acquisition timed out")

// releaseScript releases the lock only if we still own it (no foreign unlock).
var releaseScript = goredis.NewScript(`
if redis.call("get", KEYS[1]) == ARGV[1] then
  return redis.call("del", KEYS[1])
end
return 0`)

func (l *RedisLocker) WithLock(ctx context.Context, key string, fn func(context.Context) error) error {
	acquired := false
	for i := 0; i < l.retries; i++ {
		ok, err := l.client.SetNX(ctx, key, l.value, l.ttl).Result()
		if err != nil {
			return err
		}
		if ok {
			acquired = true
			break
		}
		select {
		case <-ctx.Done():
			return ctx.Err()
		case <-time.After(l.delay):
		}
	}
	if !acquired {
		return ErrLockTimeout
	}
	defer func() {
		// Best-effort release on a fresh context so a cancelled ctx does not leak
		// the lock until TTL.
		rctx, cancel := context.WithTimeout(context.Background(), 2*time.Second)
		defer cancel()
		_ = releaseScript.Run(rctx, l.client, []string{key}, l.value).Err()
	}()
	return fn(ctx)
}
