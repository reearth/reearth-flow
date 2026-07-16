package redis_test

import (
	"context"
	"encoding/json"
	"os"
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

		result, err := r.GetJobCompleteEvent(ctx, jobID)
		assert.NoError(t, err)
		assert.NotNil(t, result)
		assert.Equal(t, "success", result.Result)

		assert.NoError(t, mock.ExpectationsWereMet())
	})
}

// fixturePath is the shared wire-shape fixture also used by the subscriber
// module's own round-trip test (pkg/job/job_complete_event_test.go) and by
// gateway.TestJobCompleteEvent_RoundTripsDiagnosticsFixture. The subscriber
// and api Go modules are independent (no shared package, no cross-module
// import), so this test simulates the subscriber -> Redis -> api hop by
// treating the fixture bytes as "what the subscriber wrote to Redis" —
// the subscriber's own tests separately lock that its Marshal output
// matches this fixture's shape.
const fixturePath = "../../../../testdata/diagnostics/job_complete_with_diagnostics.json"

func TestGetJobCompleteEvent_DiagnosticsSurviveSubscriberRedisHop(t *testing.T) {
	ctx := context.Background()

	client, mock := redismock.NewClientMock()
	r, err := redis.NewRedisLog(client)
	require.NoError(t, err)

	raw, err := os.ReadFile(fixturePath)
	require.NoError(t, err)

	jobID := id.NewJobID()
	key := "job_complete:" + jobID.String()
	mock.ExpectGet(key).SetVal(string(raw))

	result, err := r.GetJobCompleteEvent(ctx, jobID)
	assert.NoError(t, err)
	require.NotNil(t, result)
	assert.Equal(t, "failed", result.Result)

	require.Len(t, result.FailedNodes, 2)
	assert.Equal(t, "internal.invariant_violation", result.FailedNodes[0].Code)
	assert.Equal(t, "internal.unclassified", result.FailedNodes[1].Code)
	require.NotNil(t, result.FailedNodes[1].NodeID)
	assert.Equal(t, "subgraph-a.sink-writer-2", *result.FailedNodes[1].NodeID)

	require.Len(t, result.AggregatedDiagnostics, 1)
	assert.Equal(t, "gltf.zero_face_solid", result.AggregatedDiagnostics[0].Code)
	require.NotNil(t, result.AggregatedDiagnostics[0].Aggregated)
	assert.Equal(t, uint64(5), result.AggregatedDiagnostics[0].Aggregated.Count)

	require.NotNil(t, result.DroppedEventCount)
	assert.Equal(t, uint64(2), *result.DroppedEventCount)

	assert.NoError(t, mock.ExpectationsWereMet())
}

func TestGetJobCompleteEvent_LegacyWireCompat(t *testing.T) {
	ctx := context.Background()

	client, mock := redismock.NewClientMock()
	r, err := redis.NewRedisLog(client)
	require.NoError(t, err)

	jobID := id.NewJobID()
	key := "job_complete:" + jobID.String()
	legacy := `{"workflowId":"11111111-1111-1111-1111-111111111111","jobId":"` + jobID.String() + `","result":"success","timestamp":"2026-01-01T00:00:00Z"}`
	mock.ExpectGet(key).SetVal(legacy)

	result, err := r.GetJobCompleteEvent(ctx, jobID)
	assert.NoError(t, err)
	require.NotNil(t, result)
	assert.Equal(t, "success", result.Result)
	assert.Nil(t, result.FailedNodes)
	assert.Nil(t, result.AggregatedDiagnostics)
	assert.Nil(t, result.DroppedEventCount)

	assert.NoError(t, mock.ExpectationsWereMet())
}

func TestDeleteJobCompleteEvent(t *testing.T) {
	ctx := context.Background()

	t.Run("Success - delete event", func(t *testing.T) {
		client, mock := redismock.NewClientMock()
		r, err := redis.NewRedisLog(client)
		require.NoError(t, err)

		jobID := id.NewJobID()
		key := "job_complete:" + jobID.String()

		// Mock Redis Del
		mock.ExpectDel(key).SetVal(1)

		err = r.DeleteJobCompleteEvent(ctx, jobID)
		assert.NoError(t, err)

		// Verify all expectations were met
		assert.NoError(t, mock.ExpectationsWereMet())
	})

	t.Run("Error - delete fails", func(t *testing.T) {
		client, mock := redismock.NewClientMock()
		r, err := redis.NewRedisLog(client)
		require.NoError(t, err)

		jobID := id.NewJobID()
		key := "job_complete:" + jobID.String()

		// Mock Redis Del returning error
		mock.ExpectDel(key).SetErr(assert.AnError)

		err = r.DeleteJobCompleteEvent(ctx, jobID)
		assert.Error(t, err)

		assert.NoError(t, mock.ExpectationsWereMet())
	})
}
