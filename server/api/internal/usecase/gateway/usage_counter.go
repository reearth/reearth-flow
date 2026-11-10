package gateway

import (
	"context"
	"time"
)

// UsageCounter counts uses in windows that end at a fixed time, such as calls
// per day.
type UsageCounter interface {
	// Increment adds one to key and returns the new count. The count is
	// dropped at expireAt, which starts the next window from zero.
	Increment(ctx context.Context, key string, expireAt time.Time) (int64, error)
	// Decrement takes back one Increment of key, with the same expireAt.
	Decrement(ctx context.Context, key string, expireAt time.Time) error
	// AcquireLease holds a lease named member under key until expireAt, and
	// returns how many unexpired leases key holds, counting the new one. Leases
	// that expired by now no longer count.
	AcquireLease(ctx context.Context, key, member string, now, expireAt time.Time) (int64, error)
	// ReleaseLease drops member's lease under key. It leaves every other lease
	// as it is, so a late release cannot free a slot another use holds.
	ReleaseLease(ctx context.Context, key, member string) error
}
