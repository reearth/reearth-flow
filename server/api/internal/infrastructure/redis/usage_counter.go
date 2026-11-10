package redis

import (
	"context"
	"strconv"
	"time"

	"github.com/redis/go-redis/v9"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
)

type usageCounter struct {
	client *redis.Client
}

func NewUsageCounter(client *redis.Client) gateway.UsageCounter {
	return &usageCounter{client: client}
}

// Increment runs INCR and EXPIREAT in one transaction, so a key never outlives
// its window even if the server stops between the two.
func (c *usageCounter) Increment(ctx context.Context, key string, expireAt time.Time) (int64, error) {
	pipe := c.client.TxPipeline()
	incr := pipe.Incr(ctx, key)
	pipe.ExpireAt(ctx, key, expireAt)
	if _, err := pipe.Exec(ctx); err != nil {
		return 0, err
	}
	return incr.Val(), nil
}

// Decrement also sets the expiry, so a key whose window ended in between, and
// that DECR would create again, is dropped at once.
func (c *usageCounter) Decrement(ctx context.Context, key string, expireAt time.Time) error {
	pipe := c.client.TxPipeline()
	pipe.Decr(ctx, key)
	pipe.ExpireAt(ctx, key, expireAt)
	_, err := pipe.Exec(ctx)
	return err
}

// AcquireLease keeps a key's leases in a sorted set scored by when each
// expires. In one transaction it drops the expired ones, adds member's, counts
// them, and keeps the key until the newest lease expires.
func (c *usageCounter) AcquireLease(ctx context.Context, key, member string, now, expireAt time.Time) (int64, error) {
	pipe := c.client.TxPipeline()
	pipe.ZRemRangeByScore(ctx, key, "-inf", strconv.FormatInt(now.UnixMilli(), 10))
	pipe.ZAdd(ctx, key, redis.Z{Score: float64(expireAt.UnixMilli()), Member: member})
	card := pipe.ZCard(ctx, key)
	pipe.ExpireAt(ctx, key, expireAt)
	if _, err := pipe.Exec(ctx); err != nil {
		return 0, err
	}
	return card.Val(), nil
}

// ReleaseLease removes member alone, leaving the key's expiry to the leases
// still held.
func (c *usageCounter) ReleaseLease(ctx context.Context, key, member string) error {
	return c.client.ZRem(ctx, key, member).Err()
}
