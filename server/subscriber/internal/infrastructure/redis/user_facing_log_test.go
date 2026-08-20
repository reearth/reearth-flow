package redis

import (
	"context"
	"errors"
	"testing"
	"time"

	"github.com/redis/go-redis/v9"
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
	expectedStreamKey := "userfacinglog:job-456"
	expectedMinID := time.Now().Add(-userFacingLogStreamRetention).UnixMilli()

	mClient.
		On("Set", mock.Anything, expectedKey, expectedVal, 12*time.Hour).
		Return(nil)
	mClient.
		On("XAdd", mock.Anything, mock.MatchedBy(func(a redis.XAddArgs) bool {
			if a.Stream != expectedStreamKey || a.ID != "" || !a.Approx {
				return false
			}
			minIDMs, err := parseStreamMinID(a.MinID)
			if err != nil || minIDMs < expectedMinID-5000 || minIDMs > expectedMinID+5000 {
				return false
			}
			values, ok := a.Values.(map[string]interface{})
			if !ok {
				return false
			}
			return values["data"] == expectedVal && values["timestampMs"] == event.Timestamp.UnixMilli()
		})).
		Return(nil)
	mClient.
		On("Expire", mock.Anything, expectedStreamKey, 12*time.Hour).
		Return(nil)

	err := rStorage.SaveUserFacingLogToRedis(ctx, event)
	assert.NoError(t, err)
	mClient.AssertExpectations(t)
	assert.Equal(t, []string{"Set", "XAdd", "Expire"}, mClient.calls)
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
	assert.Equal(t, []string{"Set"}, mClient.calls)
}

func TestRedisStorage_SaveUserFacingLogToRedis_StreamError(t *testing.T) {
	ctx := context.Background()
	mClient := new(mockRedisClient)
	rStorage := NewRedisStorage(mClient)

	event := &userfacinglog.UserFacingLogEvent{
		WorkflowID: "wf-123",
		JobID:      "job-456",
		Timestamp:  time.Now(),
		Level:      userfacinglog.UserFacingLogLevelInfo,
		Message:    "Test message",
	}

	mClient.
		On("Set", mock.Anything, mock.Anything, mock.Anything, 12*time.Hour).
		Return(nil)
	mClient.
		On("XAdd", mock.Anything, mock.Anything).
		Return(errors.New("redis xadd error"))

	err := rStorage.SaveUserFacingLogToRedis(ctx, event)
	assert.Error(t, err)
	assert.Contains(t, err.Error(), "failed to add user facing log to redis stream")
	assert.Equal(t, []string{"Set", "XAdd"}, mClient.calls)
}

// A failure after a successful XAdd must not fail the whole save, or Pub/Sub
// redelivery would append a duplicate stream entry on every retry.
func TestRedisStorage_SaveUserFacingLogToRedis_ExpireFailureIsNonFatal(t *testing.T) {
	ctx := context.Background()
	mClient := new(mockRedisClient)
	rStorage := NewRedisStorage(mClient)

	event := &userfacinglog.UserFacingLogEvent{
		WorkflowID: "wf-123",
		JobID:      "job-456",
		Timestamp:  time.Now(),
		Level:      userfacinglog.UserFacingLogLevelInfo,
		Message:    "Test message",
	}

	mClient.
		On("Set", mock.Anything, mock.Anything, mock.Anything, 12*time.Hour).
		Return(nil)
	mClient.
		On("XAdd", mock.Anything, mock.Anything).
		Return(nil)
	mClient.
		On("Expire", mock.Anything, mock.Anything, mock.Anything).
		Return(errors.New("redis expire error"))

	err := rStorage.SaveUserFacingLogToRedis(ctx, event)
	assert.NoError(t, err)
	assert.Equal(t, []string{"Set", "XAdd", "Expire"}, mClient.calls)
}

// Regression pin for the explicit `{ms}-*` XADD ID bug: two events for the same
// job whose timestamps go backwards (as Pub/Sub redelivery can produce) must
// both be written without error.
func TestRedisStorage_SaveUserFacingLogToRedis_OutOfOrderEvents(t *testing.T) {
	ctx := context.Background()
	client := newOrderingRedisClient()
	rStorage := NewRedisStorage(client)

	later := time.Date(2025, 1, 11, 9, 12, 54, 0, time.UTC)
	earlier := later.Add(-time.Second)

	eventLater := &userfacinglog.UserFacingLogEvent{
		WorkflowID: "wf-123",
		JobID:      "job-ooo",
		Timestamp:  later,
		Level:      userfacinglog.UserFacingLogLevelInfo,
		Message:    "later",
	}
	eventEarlier := &userfacinglog.UserFacingLogEvent{
		WorkflowID: "wf-123",
		JobID:      "job-ooo",
		Timestamp:  earlier,
		Level:      userfacinglog.UserFacingLogLevelInfo,
		Message:    "earlier",
	}

	assert.NoError(t, rStorage.SaveUserFacingLogToRedis(ctx, eventLater))
	assert.NoError(t, rStorage.SaveUserFacingLogToRedis(ctx, eventEarlier))
}
