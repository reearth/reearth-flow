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

func (m *mockUserFacingLogStorage) Save(ctx context.Context, event *userfacinglog.UserFacingLogEvent) error {
	args := m.Called(ctx, event)
	return args.Error(0)
}

func TestUserFacingLogSubscriberUseCase_ProcessUserFacingLogEvent(t *testing.T) {
	t.Parallel()

	t.Run("Success: storing to the live store succeeds with all fields", func(t *testing.T) {
		t.Parallel()
		ctx := context.Background()
		mockStorage := new(mockUserFacingLogStorage)
		u := NewUserFacingLogSubscriberUseCase(mockStorage)

		nodeName := "test-node"
		nodeID := testNodeID
		event := &userfacinglog.UserFacingLogEvent{
			WorkflowID: testWorkflowID,
			JobID:      "job-123",
			Timestamp:  time.Now(),
			Level:      userfacinglog.UserFacingLogLevelInfo,
			NodeName:   &nodeName,
			NodeID:     &nodeID,
			Message:    "Test user-facing message",
		}

		mockStorage.
			On("Save", ctx, event).
			Return(nil).Once()

		err := u.ProcessUserFacingLogEvent(ctx, event)
		assert.NoError(t, err)

		mockStorage.AssertExpectations(t)
	})

	t.Run("Success: storing to the live store succeeds without optional fields", func(t *testing.T) {
		t.Parallel()
		ctx := context.Background()
		mockStorage := new(mockUserFacingLogStorage)
		u := NewUserFacingLogSubscriberUseCase(mockStorage)

		event := &userfacinglog.UserFacingLogEvent{
			WorkflowID: testWorkflowID,
			JobID:      "job-456",
			Timestamp:  time.Now(),
			Level:      userfacinglog.UserFacingLogLevelSuccess,
			NodeName:   nil,
			NodeID:     nil,
			Message:    "Operation completed successfully",
		}

		mockStorage.
			On("Save", ctx, event).
			Return(nil).Once()

		err := u.ProcessUserFacingLogEvent(ctx, event)
		assert.NoError(t, err)

		mockStorage.AssertExpectations(t)
	})

	t.Run("Success: storing error level log", func(t *testing.T) {
		t.Parallel()
		ctx := context.Background()
		mockStorage := new(mockUserFacingLogStorage)
		u := NewUserFacingLogSubscriberUseCase(mockStorage)

		nodeName := "error-node"
		event := &userfacinglog.UserFacingLogEvent{
			WorkflowID: testWorkflowID,
			JobID:      "job-789",
			Timestamp:  time.Now(),
			Level:      userfacinglog.UserFacingLogLevelError,
			NodeName:   &nodeName,
			NodeID:     nil,
			Message:    "An error occurred during processing",
		}

		mockStorage.
			On("Save", ctx, event).
			Return(nil).Once()

		err := u.ProcessUserFacingLogEvent(ctx, event)
		assert.NoError(t, err)

		mockStorage.AssertExpectations(t)
	})

	t.Run("Error: event is nil", func(t *testing.T) {
		t.Parallel()
		ctx := context.Background()
		mockStorage := new(mockUserFacingLogStorage)
		u := NewUserFacingLogSubscriberUseCase(mockStorage)

		err := u.ProcessUserFacingLogEvent(ctx, nil)
		assert.Error(t, err)
		assert.Contains(t, err.Error(), "user facing log event is nil")
	})

	t.Run("Error: missing workflow ID", func(t *testing.T) {
		t.Parallel()
		ctx := context.Background()
		mockStorage := new(mockUserFacingLogStorage)
		u := NewUserFacingLogSubscriberUseCase(mockStorage)

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

		mockStorage.AssertNotCalled(t, "Save")
	})

	t.Run("Error: missing job ID", func(t *testing.T) {
		t.Parallel()
		ctx := context.Background()
		mockStorage := new(mockUserFacingLogStorage)
		u := NewUserFacingLogSubscriberUseCase(mockStorage)

		event := &userfacinglog.UserFacingLogEvent{
			WorkflowID: testWorkflowID,
			JobID:      "",
			Timestamp:  time.Now(),
			Level:      userfacinglog.UserFacingLogLevelInfo,
			Message:    "Test message",
		}

		err := u.ProcessUserFacingLogEvent(ctx, event)
		assert.Error(t, err)
		assert.Contains(t, err.Error(), "invalid event: missing workflow ID or job ID")

		mockStorage.AssertNotCalled(t, "Save")
	})

	t.Run("Error: storing to the live store fails", func(t *testing.T) {
		t.Parallel()
		ctx := context.Background()
		mockStorage := new(mockUserFacingLogStorage)
		u := NewUserFacingLogSubscriberUseCase(mockStorage)

		event := &userfacinglog.UserFacingLogEvent{
			WorkflowID: testWorkflowID,
			JobID:      "job-123",
			Timestamp:  time.Now(),
			Level:      userfacinglog.UserFacingLogLevelInfo,
			Message:    "Test message",
		}

		mockStorage.
			On("Save", ctx, event).
			Return(errors.New("redis connection error")).Once()

		err := u.ProcessUserFacingLogEvent(ctx, event)
		assert.Error(t, err)
		assert.Contains(t, err.Error(), "failed to write user facing log to live store")
		assert.Contains(t, err.Error(), "redis connection error")

		mockStorage.AssertExpectations(t)
	})

	t.Run("Success: empty display message is allowed", func(t *testing.T) {
		t.Parallel()
		ctx := context.Background()
		mockStorage := new(mockUserFacingLogStorage)
		u := NewUserFacingLogSubscriberUseCase(mockStorage)

		event := &userfacinglog.UserFacingLogEvent{
			WorkflowID: testWorkflowID,
			JobID:      "job-empty",
			Timestamp:  time.Now(),
			Level:      userfacinglog.UserFacingLogLevelInfo,
			Message:    "",
		}

		mockStorage.
			On("Save", ctx, event).
			Return(nil).Once()

		err := u.ProcessUserFacingLogEvent(ctx, event)
		assert.NoError(t, err)

		mockStorage.AssertExpectations(t)
	})
}

func TestNewUserFacingLogSubscriberUseCase(t *testing.T) {
	t.Parallel()
	mockStorage := new(mockUserFacingLogStorage)
	u := NewUserFacingLogSubscriberUseCase(mockStorage)
	assert.NotNil(t, u)
}
