package interactor

import (
	"context"
	"errors"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"

	"github.com/reearth/reearth-flow/subscriber/pkg/userfacinglog"
)

type mockUserFacingLogStorage struct {
	mock.Mock
}

func (m *mockUserFacingLogStorage) SaveToRedis(ctx context.Context, event *userfacinglog.UserFacingLogEvent) error {
	args := m.Called(ctx, event)
	return args.Error(0)
}

func TestUserFacingLogSubscriberUseCase_ProcessUserFacingLogEvent(t *testing.T) {
	ctx := context.Background()
	mockStorage := new(mockUserFacingLogStorage)

	u := NewUserFacingLogSubscriberUseCase(mockStorage)

	t.Run("Success: storing to Redis succeed with all fields", func(t *testing.T) {
		nodeName := "test-node"
		nodeID := "node-123"
		event := &userfacinglog.UserFacingLogEvent{
			WorkflowID: "wf-123",
			JobID:      "job-123",
			Timestamp:  time.Now(),
			Level:      userfacinglog.UserFacingLogLevelInfo,
			NodeName:   &nodeName,
			NodeID:     &nodeID,
			Message:    "Test user-facing message",
		}

		mockStorage.
			On("SaveToRedis", ctx, event).
			Return(nil).Once()

		err := u.ProcessUserFacingLogEvent(ctx, event)
		assert.NoError(t, err)

		mockStorage.AssertExpectations(t)
	})

	t.Run("Success: storing to Redis succeed without optional fields", func(t *testing.T) {
		event := &userfacinglog.UserFacingLogEvent{
			WorkflowID: "wf-456",
			JobID:      "job-456",
			Timestamp:  time.Now(),
			Level:      userfacinglog.UserFacingLogLevelSuccess,
			NodeName:   nil,
			NodeID:     nil,
			Message:    "Operation completed successfully",
		}

		mockStorage.
			On("SaveToRedis", ctx, event).
			Return(nil).Once()

		err := u.ProcessUserFacingLogEvent(ctx, event)
		assert.NoError(t, err)

		mockStorage.AssertExpectations(t)
	})

	t.Run("Success: storing error level log", func(t *testing.T) {
		nodeName := "error-node"
		event := &userfacinglog.UserFacingLogEvent{
			WorkflowID: "wf-789",
			JobID:      "job-789",
			Timestamp:  time.Now(),
			Level:      userfacinglog.UserFacingLogLevelError,
			NodeName:   &nodeName,
			NodeID:     nil,
			Message:    "An error occurred during processing",
		}

		mockStorage.
			On("SaveToRedis", ctx, event).
			Return(nil).Once()

		err := u.ProcessUserFacingLogEvent(ctx, event)
		assert.NoError(t, err)

		mockStorage.AssertExpectations(t)
	})

	t.Run("Error: event is nil", func(t *testing.T) {
		err := u.ProcessUserFacingLogEvent(ctx, nil)
		assert.Error(t, err)
		assert.Contains(t, err.Error(), "user facing log event is nil")
	})

	t.Run("Error: missing workflow ID", func(t *testing.T) {
		event := &userfacinglog.UserFacingLogEvent{
			WorkflowID: "",
			JobID:      "job-123",
			Timestamp:  time.Now(),
			Level:      userfacinglog.UserFacingLogLevelInfo,
			Message:    "Test message",
		}

		err := u.ProcessUserFacingLogEvent(ctx, event)
		assert.Error(t, err)
		assert.Contains(t, err.Error(), "invalid event: missing workflow ID or job ID")

		mockStorage.AssertNotCalled(t, "SaveToRedis")
	})

	t.Run("Error: missing job ID", func(t *testing.T) {
		event := &userfacinglog.UserFacingLogEvent{
			WorkflowID: "wf-123",
			JobID:      "",
			Timestamp:  time.Now(),
			Level:      userfacinglog.UserFacingLogLevelInfo,
			Message:    "Test message",
		}

		err := u.ProcessUserFacingLogEvent(ctx, event)
		assert.Error(t, err)
		assert.Contains(t, err.Error(), "invalid event: missing workflow ID or job ID")

		mockStorage.AssertNotCalled(t, "SaveToRedis")
	})

	t.Run("Error: storing to Redis fails", func(t *testing.T) {
		event := &userfacinglog.UserFacingLogEvent{
			WorkflowID: "wf-123",
			JobID:      "job-123",
			Timestamp:  time.Now(),
			Level:      userfacinglog.UserFacingLogLevelInfo,
			Message:    "Test message",
		}

		mockStorage.
			On("SaveToRedis", ctx, event).
			Return(errors.New("redis connection error")).Once()

		err := u.ProcessUserFacingLogEvent(ctx, event)
		assert.Error(t, err)
		assert.Contains(t, err.Error(), "failed to write user facing log to Redis")
		assert.Contains(t, err.Error(), "redis connection error")

		mockStorage.AssertExpectations(t)
	})

	t.Run("Success: empty display message is allowed", func(t *testing.T) {
		event := &userfacinglog.UserFacingLogEvent{
			WorkflowID: "wf-empty",
			JobID:      "job-empty",
			Timestamp:  time.Now(),
			Level:      userfacinglog.UserFacingLogLevelInfo,
			Message:    "",
		}

		mockStorage.
			On("SaveToRedis", ctx, event).
			Return(nil).Once()

		err := u.ProcessUserFacingLogEvent(ctx, event)
		assert.NoError(t, err)

		mockStorage.AssertExpectations(t)
	})
}

func TestNewUserFacingLogSubscriberUseCase(t *testing.T) {
	mockStorage := new(mockUserFacingLogStorage)
	u := NewUserFacingLogSubscriberUseCase(mockStorage)
	assert.NotNil(t, u)
}
