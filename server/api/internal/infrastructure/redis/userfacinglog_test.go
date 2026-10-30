package redis_test

import (
	"context"
	"encoding/json"
	"errors"
	"testing"
	"time"

	"github.com/go-redis/redismock/v9"
	goredis "github.com/redis/go-redis/v9"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/redis"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestRedisLog_GetUserFacingLogs(t *testing.T) {
	t.Parallel()

	jobID := id.NewJobID()
	workflowID := id.NewWorkflowID()
	streamKey := "userfacinglog:" + jobID.String()
	now := time.Now().UTC()
	since := now.Add(-1 * time.Hour)
	until := now

	t.Run("get user-facing logs successfully", func(t *testing.T) {
		t.Parallel()

		client, mock := redismock.NewClientMock()
		redisLog, err := redis.NewRedisLog(client)
		require.NoError(t, err)

		mock.ExpectXRange(streamKey, streamStart(since), "+").SetVal([]goredis.XMessage{
			logMessage(t, "1-0", map[string]any{
				"workflowId": workflowID.String(),
				"jobId":      jobID.String(),
				"timestamp":  now.Add(-30 * time.Minute),
				"level":      "info",
				"message":    "Processing started",
				"metadata":   json.RawMessage(`{"step": 1}`),
			}),
			logMessage(t, "2-0", map[string]any{
				"workflowId": workflowID.String(),
				"jobId":      jobID.String(),
				"timestamp":  now.Add(-15 * time.Minute),
				"level":      "success",
				"message":    "Data loaded successfully",
				"metadata":   json.RawMessage(`{"records": 1000}`),
			}),
		})

		result, err := redisLog.GetUserFacingLogs(context.Background(), since, until, jobID)
		assert.NoError(t, err)
		require.Len(t, result, 2)
		assert.Equal(t, "Processing started", result[0].Message())
		assert.Equal(t, "Data loaded successfully", result[1].Message())
		assert.NoError(t, mock.ExpectationsWereMet())
	})

	t.Run("handle xrange error", func(t *testing.T) {
		t.Parallel()

		client, mock := redismock.NewClientMock()
		redisLog, err := redis.NewRedisLog(client)
		require.NoError(t, err)

		mock.ExpectXRange(streamKey, streamStart(since), "+").SetErr(errors.New("redis connection failed"))

		result, err := redisLog.GetUserFacingLogs(context.Background(), since, until, jobID)
		assert.Error(t, err)
		assert.Contains(t, err.Error(), "failed to xrange redis stream")
		assert.Nil(t, result)
		assert.NoError(t, mock.ExpectationsWereMet())
	})

	t.Run("handle empty result", func(t *testing.T) {
		t.Parallel()

		client, mock := redismock.NewClientMock()
		redisLog, err := redis.NewRedisLog(client)
		require.NoError(t, err)

		mock.ExpectXRange(streamKey, streamStart(since), "+").SetVal([]goredis.XMessage{})

		result, err := redisLog.GetUserFacingLogs(context.Background(), since, until, jobID)
		assert.NoError(t, err)
		assert.Empty(t, result)
		assert.NoError(t, mock.ExpectationsWereMet())
	})

	t.Run("filter logs by time range", func(t *testing.T) {
		t.Parallel()

		client, mock := redismock.NewClientMock()
		redisLog, err := redis.NewRedisLog(client)
		require.NoError(t, err)

		recentSince := now.Add(-30 * time.Minute)
		mock.ExpectXRange(streamKey, streamStart(recentSince), "+").SetVal([]goredis.XMessage{
			logMessage(t, "1-0", map[string]any{
				"workflowId": workflowID.String(),
				"jobId":      jobID.String(),
				"timestamp":  now.Add(-60 * time.Minute),
				"level":      "info",
				"message":    "Old log",
			}),
			logMessage(t, "2-0", map[string]any{
				"workflowId": workflowID.String(),
				"jobId":      jobID.String(),
				"timestamp":  now.Add(-10 * time.Minute),
				"level":      "info",
				"message":    "Recent log",
			}),
		})

		result, err := redisLog.GetUserFacingLogs(context.Background(), recentSince, until, jobID)
		assert.NoError(t, err)
		require.Len(t, result, 1)
		assert.Equal(t, "Recent log", result[0].Message())
		assert.NoError(t, mock.ExpectationsWereMet())
	})

	t.Run("skip entries without data", func(t *testing.T) {
		t.Parallel()

		client, mock := redismock.NewClientMock()
		redisLog, err := redis.NewRedisLog(client)
		require.NoError(t, err)

		mock.ExpectXRange(streamKey, streamStart(since), "+").SetVal([]goredis.XMessage{
			{ID: "1-0", Values: map[string]any{"timestampMs": "1"}},
		})

		result, err := redisLog.GetUserFacingLogs(context.Background(), since, until, jobID)
		assert.NoError(t, err)
		assert.Empty(t, result)
		assert.NoError(t, mock.ExpectationsWereMet())
	})
}
