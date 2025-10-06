package redis

import (
	"context"
	"errors"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"

	"github.com/reearth/reearth-flow/subscriber/pkg/userfacinglog"
)

func TestRedisStorage_SaveUserFacingLogToRedis(t *testing.T) {
	ctx := context.Background()
	mClient := new(mockRedisClient)
	rStorage := NewRedisStorage(mClient)

	nodeName := "test-node"
	nodeID := "node-123"
	event := &userfacinglog.UserFacingLogEvent{
		WorkflowID: "wf-123",
		JobID:      "job-456",
		Timestamp:  time.Date(2025, 1, 11, 9, 12, 54, 487779000, time.UTC),
		Level:      userfacinglog.UserFacingLogLevelInfo,
		NodeName:   &nodeName,
		NodeID:     &nodeID,
		Message:    "Test user-facing log message",
	}

	expectedKey := "userfacinglog:wf-123:job-456:2025-01-11T09:12:54.487779Z"
	expectedVal := `{"workflowId":"wf-123","jobId":"job-456","timestamp":"2025-01-11T09:12:54.487779Z","level":"INFO","nodeName":"test-node","nodeId":"node-123","message":"Test user-facing log message"}`

	mClient.
		On("Set", ctx, expectedKey, expectedVal, 12*time.Hour).
		Return(nil)

	err := rStorage.SaveUserFacingLogToRedis(ctx, event)
	assert.NoError(t, err)
	mClient.AssertExpectations(t)
}

func TestRedisStorage_SaveUserFacingLogToRedis_Error(t *testing.T) {
	ctx := context.Background()
	mClient := new(mockRedisClient)
	rStorage := NewRedisStorage(mClient)

	event := &userfacinglog.UserFacingLogEvent{
		WorkflowID: "wf-123",
		JobID:      "job-456",
		Timestamp:  time.Now(),
		Level:      userfacinglog.UserFacingLogLevelError,
		Message:    "Error message",
	}

	mClient.
		On("Set", mock.Anything, mock.Anything, mock.Anything, 12*time.Hour).
		Return(errors.New("redis set error"))

	err := rStorage.SaveUserFacingLogToRedis(ctx, event)
	assert.Error(t, err)
	assert.Contains(t, err.Error(), "failed to save user facing log to redis")
}
