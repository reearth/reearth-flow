package redis

import (
	"context"
	"encoding/json"
	"errors"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"

	"github.com/reearth/reearth-flow/subscriber/pkg/node"
)

func TestRedisStorage_SaveNodeEventToRedis(t *testing.T) {
	ctx := context.Background()
	mClient := new(mockRedisClient)
	rStorage := NewRedisStorage(mClient)

	ts := time.Date(2025, 1, 11, 9, 12, 54, 487779000, time.UTC)
	event := &node.NodeStatusEvent{
		WorkflowID: "wf-123",
		JobID:      "job-123",
		NodeID:     "node-1",
		Status:     node.StatusCompleted,
		Timestamp:  ts,
	}

	expectedNodeKey := "node:job-123:node-1"
	expectedHashKey := "node:job-123"

	nodeData := map[string]interface{}{
		"id":         "node-1",
		"jobId":      "job-123",
		"nodeId":     "node-1",
		"status":     "COMPLETED",
		"timestamp":  ts,
		"workflowId": "wf-123",
	}
	expectedValBytes, err := json.Marshal(nodeData)
	assert.NoError(t, err)
	expectedVal := string(expectedValBytes)

	mClient.
		On("Set", mock.Anything, expectedNodeKey, expectedVal, 12*time.Hour).
		Return(nil)
	mClient.
		On("HSet", mock.Anything, expectedHashKey, []interface{}{"node-1", expectedVal}).
		Return(nil)
	mClient.
		On("Expire", mock.Anything, expectedHashKey, 12*time.Hour).
		Return(nil)

	err = rStorage.SaveNodeEventToRedis(ctx, event)
	assert.NoError(t, err)
	mClient.AssertExpectations(t)

	assert.Equal(t, []string{"Set", "HSet", "Expire"}, mClient.calls)
}

func TestRedisStorage_SaveNodeEventToRedis_NoNodeEventsListWrite(t *testing.T) {
	ctx := context.Background()
	mClient := new(mockRedisClient)
	rStorage := NewRedisStorage(mClient)

	event := &node.NodeStatusEvent{
		WorkflowID: "wf-123",
		JobID:      "job-123",
		NodeID:     "node-1",
		Status:     node.StatusPending,
		Timestamp:  time.Now(),
	}

	mClient.On("Set", mock.Anything, mock.Anything, mock.Anything, mock.Anything).Return(nil)
	mClient.On("HSet", mock.Anything, mock.Anything, mock.Anything).Return(nil)
	mClient.On("Expire", mock.Anything, mock.Anything, mock.Anything).Return(nil)

	err := rStorage.SaveNodeEventToRedis(ctx, event)
	assert.NoError(t, err)

	mClient.AssertNotCalled(t, "LPush", mock.Anything, mock.Anything, mock.Anything)
	assert.Equal(t, []string{"Set", "HSet", "Expire"}, mClient.calls)
}

func TestRedisStorage_SaveNodeEventToRedis_HashError(t *testing.T) {
	ctx := context.Background()
	mClient := new(mockRedisClient)
	rStorage := NewRedisStorage(mClient)

	event := &node.NodeStatusEvent{
		WorkflowID: "wf-123",
		JobID:      "job-123",
		NodeID:     "node-1",
		Status:     node.StatusFailed,
		Timestamp:  time.Now(),
	}

	mClient.On("Set", mock.Anything, mock.Anything, mock.Anything, mock.Anything).Return(nil)
	mClient.On("HSet", mock.Anything, mock.Anything, mock.Anything).Return(errors.New("redis hset error"))

	err := rStorage.SaveNodeEventToRedis(ctx, event)
	assert.Error(t, err)
	assert.Contains(t, err.Error(), "failed to set node status in redis hash")
	assert.Equal(t, []string{"Set", "HSet"}, mClient.calls)
}
