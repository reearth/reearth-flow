package gqlmodel

import (
	"encoding/json"
	"testing"
	"time"

	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/userfacinglog"
	"github.com/stretchr/testify/assert"
)

func TestToUserFacingLog(t *testing.T) {
	t.Run("nil input returns nil", func(t *testing.T) {
		result := ToUserFacingLog(nil)
		assert.Nil(t, result)
	})

	t.Run("converts log without metadata", func(t *testing.T) {
		jobID := id.NewJobID()
		timestamp := time.Now()
		message := "Test message"

		log := userfacinglog.NewUserFacingLog(jobID, timestamp, message, nil)
		result := ToUserFacingLog(log)

		assert.NotNil(t, result)
		assert.Equal(t, ID(jobID.String()), result.JobID)
		assert.Equal(t, timestamp, result.Timestamp)
		assert.Equal(t, UserFacingLogLevelInfo, result.Level)
		assert.Nil(t, result.NodeID)
		assert.Nil(t, result.NodeName)
		assert.Equal(t, message, result.Message)
		assert.Equal(t, JSON(nil), result.Metadata)
	})

	t.Run("converts log with metadata", func(t *testing.T) {
		jobID := id.NewJobID()
		timestamp := time.Now()
		message := "Test message with metadata"
		metadata := json.RawMessage(`{"key": "value", "count": 42}`)

		log := userfacinglog.NewUserFacingLog(jobID, timestamp, message, metadata)
		result := ToUserFacingLog(log)

		assert.NotNil(t, result)
		assert.Equal(t, ID(jobID.String()), result.JobID)
		assert.Equal(t, timestamp, result.Timestamp)
		assert.Equal(t, UserFacingLogLevelInfo, result.Level)
		assert.Nil(t, result.NodeID)
		assert.Nil(t, result.NodeName)
		assert.Equal(t, message, result.Message)

		expectedMetadata := JSON(map[string]interface{}{
			"key":   "value",
			"count": float64(42), // JSON numbers are float64
		})
		assert.Equal(t, expectedMetadata, result.Metadata)
	})

	t.Run("handles invalid metadata gracefully", func(t *testing.T) {
		jobID := id.NewJobID()
		timestamp := time.Now()
		message := "Test message with invalid metadata"
		invalidMetadata := json.RawMessage(`{invalid json}`)

		log := userfacinglog.NewUserFacingLog(jobID, timestamp, message, invalidMetadata)
		result := ToUserFacingLog(log)

		assert.NotNil(t, result)
		assert.Equal(t, ID(jobID.String()), result.JobID)
		assert.Equal(t, timestamp, result.Timestamp)
		assert.Equal(t, UserFacingLogLevelInfo, result.Level)
		assert.Nil(t, result.NodeID)
		assert.Nil(t, result.NodeName)
		assert.Equal(t, message, result.Message)
		assert.Equal(t, JSON(nil), result.Metadata) // Invalid JSON results in nil metadata
	})

	t.Run("converts log with all details", func(t *testing.T) {
		jobID := id.NewJobID()
		timestamp := time.Now()
		nodeID := "node-123"
		nodeName := "Process Data"
		message := "Processing completed successfully"
		metadata := json.RawMessage(`{"records": 100}`)

		log := userfacinglog.NewUserFacingLogWithDetails(
			jobID, timestamp, userfacinglog.LogLevelSuccess,
			&nodeID, &nodeName, message, metadata,
		)
		result := ToUserFacingLog(log)

		assert.NotNil(t, result)
		assert.Equal(t, ID(jobID.String()), result.JobID)
		assert.Equal(t, timestamp, result.Timestamp)
		assert.Equal(t, UserFacingLogLevelSuccess, result.Level)
		assert.NotNil(t, result.NodeID)
		assert.Equal(t, ID(nodeID), *result.NodeID)
		assert.NotNil(t, result.NodeName)
		assert.Equal(t, nodeName, *result.NodeName)
		assert.Equal(t, message, result.Message)
		assert.NotNil(t, result.Metadata)
	})
}
