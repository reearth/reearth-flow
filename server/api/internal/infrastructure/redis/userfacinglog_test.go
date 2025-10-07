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
	ctx := context.Background()
	jobID := id.NewJobID()
	workflowID := id.NewWorkflowID()
	now := time.Now().UTC()

	t.Run("get user-facing logs successfully", func(t *testing.T) {
		client, mock := redismock.NewClientMock()
		redisLog, err := redis.NewRedisLog(client)
		require.NoError(t, err)

		// Setup test data - using map for JSON marshaling since UserFacingLogEntry is not exported
		entry1 := map[string]interface{}{
			"workflowId": workflowID.String(),
			"jobId":      jobID.String(),
			"timestamp":  now.Add(-30 * time.Minute),
			"level":      "info",
			"message":    "Processing started",
			"metadata":   json.RawMessage(`{"step": 1}`),
		}
		entry2 := map[string]interface{}{
			"workflowId": workflowID.String(),
			"jobId":      jobID.String(),
			"timestamp":  now.Add(-15 * time.Minute),
			"level":      "success",
			"message":    "Data loaded successfully",
			"metadata":   json.RawMessage(`{"records": 1000}`),
		}

		data1, _ := json.Marshal(entry1)
		data2, _ := json.Marshal(entry2)

		pattern := "userfacinglog:*:" + jobID.String() + ":*"

		// Mock SCAN command
		mock.ExpectScan(0, pattern, 100).SetVal([]string{
			"userfacinglog:" + workflowID.String() + ":" + jobID.String() + ":1",
			"userfacinglog:" + workflowID.String() + ":" + jobID.String() + ":2",
		}, 0)

		// Mock GET commands
		mock.ExpectGet("userfacinglog:" + workflowID.String() + ":" + jobID.String() + ":1").SetVal(string(data1))
		mock.ExpectGet("userfacinglog:" + workflowID.String() + ":" + jobID.String() + ":2").SetVal(string(data2))

		// Test
		since := now.Add(-1 * time.Hour)
		until := now
		result, err := redisLog.GetUserFacingLogs(ctx, since, until, jobID)

		assert.NoError(t, err)
		assert.Len(t, result, 2)
		assert.Equal(t, "Processing started", result[0].Message())
		assert.Equal(t, "Data loaded successfully", result[1].Message())

		assert.NoError(t, mock.ExpectationsWereMet())
	})

	t.Run("handle scan error", func(t *testing.T) {
		client, mock := redismock.NewClientMock()
		redisLog, err := redis.NewRedisLog(client)
		require.NoError(t, err)

		pattern := "userfacinglog:*:" + jobID.String() + ":*"

		// Mock SCAN command to return error
		mock.ExpectScan(0, pattern, 100).SetErr(errors.New("redis connection failed"))

		// Test
		since := now.Add(-1 * time.Hour)
		until := now
		result, err := redisLog.GetUserFacingLogs(ctx, since, until, jobID)

		assert.Error(t, err)
		assert.Contains(t, err.Error(), "failed to scan redis keys")
		assert.Nil(t, result)

		assert.NoError(t, mock.ExpectationsWereMet())
	})

	t.Run("handle empty result", func(t *testing.T) {
		client, mock := redismock.NewClientMock()
		redisLog, err := redis.NewRedisLog(client)
		require.NoError(t, err)

		pattern := "userfacinglog:*:" + jobID.String() + ":*"

		// Mock SCAN command to return no keys
		mock.ExpectScan(0, pattern, 100).SetVal([]string{}, 0)

		// Test
		since := now.Add(-1 * time.Hour)
		until := now
		result, err := redisLog.GetUserFacingLogs(ctx, since, until, jobID)

		assert.NoError(t, err)
		assert.Empty(t, result)

		assert.NoError(t, mock.ExpectationsWereMet())
	})

	t.Run("filter logs by time range", func(t *testing.T) {
		client, mock := redismock.NewClientMock()
		redisLog, err := redis.NewRedisLog(client)
		require.NoError(t, err)

		// Setup test data - one old log and one recent log
		oldEntry := map[string]interface{}{
			"workflowId": workflowID.String(),
			"jobId":      jobID.String(),
			"timestamp":  now.Add(-60 * time.Minute), // Old - should be excluded
			"level":      "info",
			"message":    "Old log",
		}
		recentEntry := map[string]interface{}{
			"workflowId": workflowID.String(),
			"jobId":      jobID.String(),
			"timestamp":  now.Add(-10 * time.Minute), // Recent - should be included
			"level":      "info",
			"message":    "Recent log",
		}

		oldData, _ := json.Marshal(oldEntry)
		recentData, _ := json.Marshal(recentEntry)

		pattern := "userfacinglog:*:" + jobID.String() + ":*"

		// Mock SCAN command
		mock.ExpectScan(0, pattern, 100).SetVal([]string{
			"userfacinglog:" + workflowID.String() + ":" + jobID.String() + ":1",
			"userfacinglog:" + workflowID.String() + ":" + jobID.String() + ":2",
		}, 0)

		// Mock GET commands
		mock.ExpectGet("userfacinglog:" + workflowID.String() + ":" + jobID.String() + ":1").SetVal(string(oldData))
		mock.ExpectGet("userfacinglog:" + workflowID.String() + ":" + jobID.String() + ":2").SetVal(string(recentData))

		// Test: Get logs from last 30 minutes only
		since := now.Add(-30 * time.Minute)
		until := now
		result, err := redisLog.GetUserFacingLogs(ctx, since, until, jobID)

		assert.NoError(t, err)
		assert.Len(t, result, 1)
		assert.Equal(t, "Recent log", result[0].Message())

		assert.NoError(t, mock.ExpectationsWereMet())
	})

	t.Run("handle nil redis key", func(t *testing.T) {
		client, mock := redismock.NewClientMock()
		redisLog, err := redis.NewRedisLog(client)
		require.NoError(t, err)

		pattern := "userfacinglog:*:" + jobID.String() + ":*"

		// Mock SCAN command
		mock.ExpectScan(0, pattern, 100).SetVal([]string{
			"userfacinglog:" + workflowID.String() + ":" + jobID.String() + ":1",
		}, 0)

		// Mock GET command to return Redis Nil error
		mock.ExpectGet("userfacinglog:" + workflowID.String() + ":" + jobID.String() + ":1").SetErr(goredis.Nil)

		// Test
		since := now.Add(-1 * time.Hour)
		until := now
		result, err := redisLog.GetUserFacingLogs(ctx, since, until, jobID)

		assert.NoError(t, err)
		assert.Empty(t, result)

		assert.NoError(t, mock.ExpectationsWereMet())
	})
}
