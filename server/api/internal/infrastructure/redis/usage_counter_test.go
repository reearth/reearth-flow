package redis_test

import (
	"context"
	"errors"
	"testing"
	"time"

	"github.com/go-redis/redismock/v9"
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
