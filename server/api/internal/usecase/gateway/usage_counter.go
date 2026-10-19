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
}
