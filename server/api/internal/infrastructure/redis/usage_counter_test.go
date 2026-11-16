package redis_test

import (
	"context"
	"errors"
	"testing"
	"time"

	"github.com/go-redis/redismock/v9"
	goredis "github.com/redis/go-redis/v9"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/redis"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestUsageCounter_Increment(t *testing.T) {
	ctx := context.Background()
	key := "flow:plan-limits:api-trigger-calls:ws:2026-10-19"
	expireAt := time.Date(2026, 10, 20, 0, 0, 0, 0, time.FixedZone("JST", 9*60*60))

	t.Run("increments and sets the expiry in one transaction", func(t *testing.T) {
		client, mock := redismock.NewClientMock()
		mock.ExpectTxPipeline()
		mock.ExpectIncr(key).SetVal(3)
		mock.ExpectExpireAt(key, expireAt).SetVal(true)
		mock.ExpectTxPipelineExec()

		n, err := redis.NewUsageCounter(client).Increment(ctx, key, expireAt)

		require.NoError(t, err)
		assert.Equal(t, int64(3), n)
		assert.NoError(t, mock.ExpectationsWereMet())
	})

	t.Run("returns the error when the transaction fails", func(t *testing.T) {
		client, mock := redismock.NewClientMock()
		mock.ExpectTxPipeline()
		mock.ExpectIncr(key).SetErr(errors.New("connection refused"))

		_, err := redis.NewUsageCounter(client).Increment(ctx, key, expireAt)

		assert.Error(t, err)
	})
}

func TestUsageCounter_Decrement(t *testing.T) {
	ctx := context.Background()
	key := "flow:plan-limits:api-trigger-calls:ws:2026-10-19"
	expireAt := time.Date(2026, 10, 20, 0, 0, 0, 0, time.FixedZone("JST", 9*60*60))

	client, mock := redismock.NewClientMock()
	mock.ExpectTxPipeline()
	mock.ExpectDecr(key).SetVal(2)
	mock.ExpectExpireAt(key, expireAt).SetVal(true)
	mock.ExpectTxPipelineExec()

	require.NoError(t, redis.NewUsageCounter(client).Decrement(ctx, key, expireAt))
	assert.NoError(t, mock.ExpectationsWereMet())
}

func TestUsageCounter_AcquireLease(t *testing.T) {
	t.Parallel()

	ctx := context.Background()
	key := "flow:plan-limits:render-leases:ws"
	now := time.Date(2026, 10, 20, 0, 0, 0, 0, time.UTC)
	expireAt := now.Add(5 * time.Minute)

	t.Run("drops expired leases, adds its own and counts them in one transaction", func(t *testing.T) {
		t.Parallel()

		client, mock := redismock.NewClientMock()
		mock.ExpectTxPipeline()
		mock.ExpectZRemRangeByScore(key, "-inf", "1792454400000").SetVal(1)
		mock.ExpectZAdd(key, goredis.Z{Score: float64(expireAt.UnixMilli()), Member: "render-1"}).SetVal(1)
		mock.ExpectZCard(key).SetVal(2)
		mock.ExpectExpireAt(key, expireAt).SetVal(true)
		mock.ExpectTxPipelineExec()

		n, err := redis.NewUsageCounter(client).AcquireLease(ctx, key, "render-1", now, expireAt)

		require.NoError(t, err)
		assert.Equal(t, int64(2), n)
		assert.NoError(t, mock.ExpectationsWereMet())
	})

	t.Run("returns the error when the transaction fails", func(t *testing.T) {
		t.Parallel()

		client, mock := redismock.NewClientMock()
		mock.ExpectTxPipeline()
		mock.ExpectZRemRangeByScore(key, "-inf", "1792454400000").SetErr(errors.New("connection refused"))

		_, err := redis.NewUsageCounter(client).AcquireLease(ctx, key, "render-1", now, expireAt)

		assert.Error(t, err)
	})
}

func TestUsageCounter_ReleaseLease(t *testing.T) {
	t.Parallel()

	client, mock := redismock.NewClientMock()
	mock.ExpectZRem("flow:plan-limits:render-leases:ws", "render-1").SetVal(1)

	require.NoError(t, redis.NewUsageCounter(client).ReleaseLease(context.Background(), "flow:plan-limits:render-leases:ws", "render-1"))
	assert.NoError(t, mock.ExpectationsWereMet())
}
