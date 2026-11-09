package interactor

import (
	"context"
	"errors"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"

	domainLog "github.com/reearth/reearth-flow/subscriber/pkg/log"
)

// Valid IDs for the interactor fixtures.
const (
	testWorkflowID = "5b0f6a1e-3c2d-4e8f-9a7b-1c2d3e4f5a6b"
	testNodeID     = "7e1d2c3b-4a59-4687-b9a0-c1d2e3f4a5b6"
)

type mockLogStorage struct {
	mock.Mock
}

func (m *mockLogStorage) Save(ctx context.Context, event *domainLog.LogEvent) error {
	args := m.Called(ctx, event)
	return args.Error(0)
}

func TestLogSubscriberUseCase_ProcessLogEvent(t *testing.T) {
	t.Parallel()

	t.Run("Success: storing to the live store succeeds", func(t *testing.T) {
		t.Parallel()
		ctx := context.Background()
		mockStorage := new(mockLogStorage)
		u := NewLogSubscriberUseCase(mockStorage)

		event := &domainLog.LogEvent{
			WorkflowID: testWorkflowID,
			JobID:      "job-123",
			Timestamp:  time.Now(),
			LogLevel:   domainLog.LogLevelInfo,
			Message:    "Test message",
		}

		mockStorage.
			On("Save", ctx, event).
			Return(nil)

		err := u.ProcessLogEvent(ctx, event)
		assert.NoError(t, err)

		mockStorage.AssertExpectations(t)
	})

	t.Run("Error: event is nil", func(t *testing.T) {
		t.Parallel()
		u := NewLogSubscriberUseCase(new(mockLogStorage))

		err := u.ProcessLogEvent(context.Background(), nil)
		assert.Error(t, err, "event is nil")
	})

	t.Run("Error: storing to the live store fails", func(t *testing.T) {
		t.Parallel()
		ctx := context.Background()
		mockStorage := new(mockLogStorage)
		u := NewLogSubscriberUseCase(mockStorage)

		event := &domainLog.LogEvent{
			WorkflowID: testWorkflowID,
			JobID:      "job-123",
			Timestamp:  time.Now(),
			LogLevel:   domainLog.LogLevelInfo,
			Message:    "Test message",
		}

		mockStorage.
			On("Save", ctx, event).
			Return(errors.New("redis error"))

		err := u.ProcessLogEvent(ctx, event)
		assert.ErrorContains(t, err, "failed to write log to live store: redis error")
	})
}
