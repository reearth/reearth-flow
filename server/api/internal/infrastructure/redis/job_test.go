package redis_test

import (
	"context"
	"encoding/json"
	"testing"
	"time"

	"github.com/go-redis/redismock/v9"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/redis"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestGetJobCompleteEvent(t *testing.T) {
	ctx := context.Background()

	t.Run("Success - event exists", func(t *testing.T) {
		client, mock := redismock.NewClientMock()
		r, err := redis.NewRedisLog(client)
		require.NoError(t, err)

		jobID := id.NewJobID()
		event := gateway.JobCompleteEvent{
			WorkflowID: "workflow-123",
			JobID:      jobID.String(),
			Result:     "failed",
			Timestamp:  time.Now(),
		}

		key := "job_complete:" + jobID.String()
		data, err := json.Marshal(event)
		require.NoError(t, err)

		// Mock Redis Get
		mock.ExpectGet(key).SetVal(string(data))
		// Mock Redis Del
		mock.ExpectDel(key).SetVal(1)

		// Read event
		result, err := r.GetJobCompleteEvent(ctx, jobID)
		assert.NoError(t, err)
		assert.NotNil(t, result)
		assert.Equal(t, "failed", result.Result)
		assert.Equal(t, jobID.String(), result.JobID)

		// Verify all expectations were met
		assert.NoError(t, mock.ExpectationsWereMet())
	})

	t.Run("Success - event does not exist", func(t *testing.T) {
		client, mock := redismock.NewClientMock()
		r, err := redis.NewRedisLog(client)
		require.NoError(t, err)

		jobID := id.NewJobID()
		key := "job_complete:" + jobID.String()

		// Mock Redis Get returning nil (key doesn't exist)
		mock.ExpectGet(key).RedisNil()

		result, err := r.GetJobCompleteEvent(ctx, jobID)
		assert.NoError(t, err)
		assert.Nil(t, result)

		assert.NoError(t, mock.ExpectationsWereMet())
	})

	t.Run("Success - handles 'success' result", func(t *testing.T) {
		client, mock := redismock.NewClientMock()
		r, err := redis.NewRedisLog(client)
		require.NoError(t, err)

		jobID := id.NewJobID()
		event := gateway.JobCompleteEvent{
			WorkflowID: "workflow-456",
			JobID:      jobID.String(),
			Result:     "success",
			Timestamp:  time.Now(),
		}

		key := "job_complete:" + jobID.String()
		data, err := json.Marshal(event)
		require.NoError(t, err)

		mock.ExpectGet(key).SetVal(string(data))
		mock.ExpectDel(key).SetVal(1)

		result, err := r.GetJobCompleteEvent(ctx, jobID)
		assert.NoError(t, err)
		assert.NotNil(t, result)
		assert.Equal(t, "success", result.Result)

		assert.NoError(t, mock.ExpectationsWereMet())
	})
}
