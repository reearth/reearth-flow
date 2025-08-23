package userfacinglog

import (
	"encoding/json"
	"testing"
	"time"

	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/stretchr/testify/assert"
)

func TestNewUserFacingLog(t *testing.T) {
	t.Run("creates log with all fields", func(t *testing.T) {
		jobID := id.NewJobID()
		timestamp := time.Now()
		message := "Processing completed successfully"
		metadata := json.RawMessage(`{"records": 1000, "duration": "5m"}`)

		log := NewUserFacingLog(jobID, timestamp, message, metadata)

		assert.NotNil(t, log)
		assert.Equal(t, jobID, log.JobID())
		assert.Equal(t, timestamp, log.Timestamp())
		assert.Equal(t, message, log.Message())
		assert.Equal(t, metadata, log.Metadata())
	})

	t.Run("creates log without metadata", func(t *testing.T) {
		jobID := id.NewJobID()
		timestamp := time.Now()
		message := "Simple log message"

		log := NewUserFacingLog(jobID, timestamp, message, nil)

		assert.NotNil(t, log)
		assert.Equal(t, jobID, log.JobID())
		assert.Equal(t, timestamp, log.Timestamp())
		assert.Equal(t, message, log.Message())
		assert.Nil(t, log.Metadata())
	})

	t.Run("creates log with empty message", func(t *testing.T) {
		jobID := id.NewJobID()
		timestamp := time.Now()

		log := NewUserFacingLog(jobID, timestamp, "", nil)

		assert.NotNil(t, log)
		assert.Equal(t, jobID, log.JobID())
		assert.Equal(t, timestamp, log.Timestamp())
		assert.Equal(t, "", log.Message())
		assert.Nil(t, log.Metadata())
	})

	t.Run("creates log with complex metadata", func(t *testing.T) {
		jobID := id.NewJobID()
		timestamp := time.Now()
		message := "Data processing"

		// Complex nested JSON metadata
		metadata := json.RawMessage(`{
			"statistics": {
				"total": 5000,
				"processed": 4950,
				"failed": 50
			},
			"performance": {
				"avgProcessingTime": "2.5ms",
				"peakMemory": "512MB"
			},
			"tags": ["batch-1", "high-priority"]
		}`)

		log := NewUserFacingLog(jobID, timestamp, message, metadata)

		assert.NotNil(t, log)
		assert.Equal(t, jobID, log.JobID())
		assert.Equal(t, timestamp, log.Timestamp())
		assert.Equal(t, message, log.Message())
		assert.Equal(t, metadata, log.Metadata())
	})
}

func TestUserFacingLog_Getters(t *testing.T) {
	jobID := id.NewJobID()
	timestamp := time.Now()
	message := "Test message"
	metadata := json.RawMessage(`{"key": "value"}`)

	log := NewUserFacingLog(jobID, timestamp, message, metadata)

	t.Run("JobID returns correct value", func(t *testing.T) {
		assert.Equal(t, jobID, log.JobID())
	})

	t.Run("Timestamp returns correct value", func(t *testing.T) {
		assert.Equal(t, timestamp, log.Timestamp())
	})

	t.Run("Message returns correct value", func(t *testing.T) {
		assert.Equal(t, message, log.Message())
	})

	t.Run("Metadata returns correct value", func(t *testing.T) {
		assert.Equal(t, metadata, log.Metadata())
	})
}

func TestUserFacingLog_Immutability(t *testing.T) {
	jobID := id.NewJobID()
	timestamp := time.Now()
	message := "Original message"
	metadata := json.RawMessage(`{"original": true}`)

	log := NewUserFacingLog(jobID, timestamp, message, metadata)

	// Verify that modifying the original metadata doesn't affect the log
	metadata = json.RawMessage(`{"modified": true}`)
	assert.NotEqual(t, metadata, log.Metadata())
	assert.Equal(t, json.RawMessage(`{"original": true}`), log.Metadata())
}

func TestUserFacingLog_EmptyMetadata(t *testing.T) {
	jobID := id.NewJobID()
	timestamp := time.Now()
	message := "Log without metadata"

	t.Run("nil metadata", func(t *testing.T) {
		log := NewUserFacingLog(jobID, timestamp, message, nil)
		assert.Nil(t, log.Metadata())
	})

	t.Run("empty json metadata", func(t *testing.T) {
		emptyMetadata := json.RawMessage(`{}`)
		log := NewUserFacingLog(jobID, timestamp, message, emptyMetadata)
		assert.Equal(t, emptyMetadata, log.Metadata())
	})

	t.Run("empty array metadata", func(t *testing.T) {
		emptyArrayMetadata := json.RawMessage(`[]`)
		log := NewUserFacingLog(jobID, timestamp, message, emptyArrayMetadata)
		assert.Equal(t, emptyArrayMetadata, log.Metadata())
	})
}

func TestUserFacingLog_TimestampPrecision(t *testing.T) {
	jobID := id.NewJobID()
	message := "Timestamp precision test"

	// Test with different timestamp precisions
	t.Run("microsecond precision", func(t *testing.T) {
		timestamp := time.Now().Truncate(time.Microsecond)
		log := NewUserFacingLog(jobID, timestamp, message, nil)
		assert.Equal(t, timestamp, log.Timestamp())
	})

	t.Run("nanosecond precision", func(t *testing.T) {
		timestamp := time.Now()
		log := NewUserFacingLog(jobID, timestamp, message, nil)
		assert.Equal(t, timestamp, log.Timestamp())
	})

	t.Run("second precision", func(t *testing.T) {
		timestamp := time.Now().Truncate(time.Second)
		log := NewUserFacingLog(jobID, timestamp, message, nil)
		assert.Equal(t, timestamp, log.Timestamp())
	})
}

func TestUserFacingLog_LargeMessage(t *testing.T) {
	jobID := id.NewJobID()
	timestamp := time.Now()

	// Create a large message
	largeMessage := ""
	for i := 0; i < 1000; i++ {
		largeMessage += "This is a long message that simulates verbose logging output. "
	}

	log := NewUserFacingLog(jobID, timestamp, largeMessage, nil)

	assert.NotNil(t, log)
	assert.Equal(t, largeMessage, log.Message())
	assert.Greater(t, len(log.Message()), 60000) // Should be > 60k characters
}

func TestUserFacingLog_InvalidMetadata(t *testing.T) {
	// Note: json.RawMessage doesn't validate JSON on creation,
	// so we can store invalid JSON. Validation happens on unmarshal.
	jobID := id.NewJobID()
	timestamp := time.Now()
	message := "Log with potentially invalid metadata"

	t.Run("malformed json metadata", func(t *testing.T) {
		invalidMetadata := json.RawMessage(`{invalid json}`)
		log := NewUserFacingLog(jobID, timestamp, message, invalidMetadata)

		// The log should still be created
		assert.NotNil(t, log)
		assert.Equal(t, invalidMetadata, log.Metadata())

		// But unmarshaling should fail
		var result map[string]interface{}
		err := json.Unmarshal(log.Metadata(), &result)
		assert.Error(t, err)
	})

	t.Run("valid json string metadata", func(t *testing.T) {
		stringMetadata := json.RawMessage(`"just a string"`)
		log := NewUserFacingLog(jobID, timestamp, message, stringMetadata)

		assert.NotNil(t, log)
		assert.Equal(t, stringMetadata, log.Metadata())

		// Should unmarshal as a string
		var result string
		err := json.Unmarshal(log.Metadata(), &result)
		assert.NoError(t, err)
		assert.Equal(t, "just a string", result)
	})
}
