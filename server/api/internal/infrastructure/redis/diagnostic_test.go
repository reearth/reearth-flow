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

func diagnosticEntryJSON(t *testing.T, jobID, nodeID string, code string) string {
	t.Helper()
	entry := redis.DiagnosticEntry{
		Timestamp:  time.Now().UTC(),
		WorkflowID: "workflow-1",
		JobID:      jobID,
		Schema:     "diagnostic.v1",
		WireDiagnostic: gateway.WireDiagnostic{
			Code:     code,
			Category: "internal",
			Severity: "warn",
			Message:  "something happened",
		},
	}
	if nodeID != "" {
		entry.NodeID = &nodeID
	}
	data, err := json.Marshal(entry)
	require.NoError(t, err)
	return string(data)
}

func TestGetNodeDiagnostics(t *testing.T) {
	ctx := context.Background()

	t.Run("Success - entries exist", func(t *testing.T) {
		client, mock := redismock.NewClientMock()
		r, err := redis.NewRedisLog(client)
		require.NoError(t, err)

		jobID := id.NewJobID()
		key := "diagnostics:" + jobID.String() + ":node-1"
		entry1 := diagnosticEntryJSON(t, jobID.String(), "node-1", "internal.invariant_violation")
		entry2 := diagnosticEntryJSON(t, jobID.String(), "node-1", "internal.unclassified")

		mock.ExpectLRange(key, 0, -1).SetVal([]string{entry1, entry2})

		result, err := r.GetNodeDiagnostics(ctx, jobID, "node-1")
		assert.NoError(t, err)
		require.Len(t, result, 2)
		assert.Equal(t, "internal.invariant_violation", result[0].Code())
		assert.Equal(t, "internal.unclassified", result[1].Code())

		assert.NoError(t, mock.ExpectationsWereMet())
	})

	t.Run("Success - empty nodeID falls back to _job bucket", func(t *testing.T) {
		client, mock := redismock.NewClientMock()
		r, err := redis.NewRedisLog(client)
		require.NoError(t, err)

		jobID := id.NewJobID()
		key := "diagnostics:" + jobID.String() + ":_job"
		entry := diagnosticEntryJSON(t, jobID.String(), "", "gltf.zero_face_solid")

		mock.ExpectLRange(key, 0, -1).SetVal([]string{entry})

		result, err := r.GetNodeDiagnostics(ctx, jobID, "")
		assert.NoError(t, err)
		require.Len(t, result, 1)
		assert.Equal(t, "gltf.zero_face_solid", result[0].Code())
		assert.Nil(t, result[0].NodeID())

		assert.NoError(t, mock.ExpectationsWereMet())
	})

	t.Run("Success - missing key returns empty, not error", func(t *testing.T) {
		client, mock := redismock.NewClientMock()
		r, err := redis.NewRedisLog(client)
		require.NoError(t, err)

		jobID := id.NewJobID()
		key := "diagnostics:" + jobID.String() + ":node-1"

		mock.ExpectLRange(key, 0, -1).SetVal([]string{})

		result, err := r.GetNodeDiagnostics(ctx, jobID, "node-1")
		assert.NoError(t, err)
		assert.Empty(t, result)

		assert.NoError(t, mock.ExpectationsWereMet())
	})

	t.Run("Error - lrange fails", func(t *testing.T) {
		client, mock := redismock.NewClientMock()
		r, err := redis.NewRedisLog(client)
		require.NoError(t, err)

		jobID := id.NewJobID()
		key := "diagnostics:" + jobID.String() + ":node-1"

		mock.ExpectLRange(key, 0, -1).SetErr(assert.AnError)

		result, err := r.GetNodeDiagnostics(ctx, jobID, "node-1")
		assert.Error(t, err)
		assert.Nil(t, result)

		assert.NoError(t, mock.ExpectationsWereMet())
	})
}

func TestGetJobDiagnostics(t *testing.T) {
	ctx := context.Background()

	t.Run("Success - entries exist across nodes", func(t *testing.T) {
		client, mock := redismock.NewClientMock()
		r, err := redis.NewRedisLog(client)
		require.NoError(t, err)

		jobID := id.NewJobID()
		key := "diagnostics:" + jobID.String()
		entry1 := diagnosticEntryJSON(t, jobID.String(), "node-1", "internal.invariant_violation")
		entry2 := diagnosticEntryJSON(t, jobID.String(), "subgraph-a.sink-writer-2", "internal.unclassified")

		mock.ExpectLRange(key, 0, -1).SetVal([]string{entry1, entry2})

		result, err := r.GetJobDiagnostics(ctx, jobID)
		assert.NoError(t, err)
		require.Len(t, result, 2)
		require.NotNil(t, result[1].NodeID())
		assert.Equal(t, "subgraph-a.sink-writer-2", *result[1].NodeID())

		assert.NoError(t, mock.ExpectationsWereMet())
	})

	t.Run("Success - missing key returns empty, not error", func(t *testing.T) {
		client, mock := redismock.NewClientMock()
		r, err := redis.NewRedisLog(client)
		require.NoError(t, err)

		jobID := id.NewJobID()
		key := "diagnostics:" + jobID.String()

		mock.ExpectLRange(key, 0, -1).SetVal(nil)

		result, err := r.GetJobDiagnostics(ctx, jobID)
		assert.NoError(t, err)
		assert.Empty(t, result)

		assert.NoError(t, mock.ExpectationsWereMet())
	})

	t.Run("Success - malformed entry is skipped, not fatal", func(t *testing.T) {
		client, mock := redismock.NewClientMock()
		r, err := redis.NewRedisLog(client)
		require.NoError(t, err)

		jobID := id.NewJobID()
		key := "diagnostics:" + jobID.String()
		good := diagnosticEntryJSON(t, jobID.String(), "node-1", "internal.invariant_violation")

		mock.ExpectLRange(key, 0, -1).SetVal([]string{"not-json", good})

		result, err := r.GetJobDiagnostics(ctx, jobID)
		assert.NoError(t, err)
		require.Len(t, result, 1)
		assert.Equal(t, "internal.invariant_violation", result[0].Code())

		assert.NoError(t, mock.ExpectationsWereMet())
	})
}
