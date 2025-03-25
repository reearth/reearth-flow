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

type mockLogStorage struct {
	mock.Mock
}

func (m *mockLogStorage) SaveToRedis(ctx context.Context, event *domainLog.LogEvent) error {
	args := m.Called(ctx, event)
	return args.Error(0)
}

func TestLogSubscriberUseCase_ProcessLogEvent(t *testing.T) {
	ctx := context.Background()
	mockStorage := new(mockLogStorage)

	u := NewLogSubscriberUseCase(mockStorage)

	t.Run("Success: storing to Redis succeed", func(t *testing.T) {
		event := &domainLog.LogEvent{
			WorkflowID: "wf-123",
			JobID:      "job-123",
			Timestamp:  time.Now(),
			LogLevel:   domainLog.LogLevelInfo,
			Message:    "Test message",
		}

		mockStorage.
			On("SaveToRedis", ctx, event).
			Return(nil)

		err := u.ProcessLogEvent(ctx, event)
		assert.NoError(t, err)

		mockStorage.AssertExpectations(t)
	})

	t.Run("Error: event is nil", func(t *testing.T) {
		err := u.ProcessLogEvent(ctx, nil)
		assert.Error(t, err, "event is nil")
	})

	t.Run("Error: storing to Redis fails", func(t *testing.T) {
		event := &domainLog.LogEvent{
			WorkflowID: "wf-123",
			JobID:      "job-123",
			Timestamp:  time.Now(),
			LogLevel:   domainLog.LogLevelInfo,
			Message:    "Test message",
		}

		mockStorage.
			On("SaveToRedis", ctx, event).
			Return(errors.New("redis error"))

		err := u.ProcessLogEvent(ctx, event)
		assert.ErrorContains(t, err, "failed to write to Redis: redis error")
	})
}
