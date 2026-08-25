package interactor

import (
	"context"
	"errors"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"

	"github.com/reearth/reearth-flow/subscriber/pkg/node"
)

type mockNodeStorage struct {
	mock.Mock
}

func (m *mockNodeStorage) SaveToRedis(ctx context.Context, event *node.NodeStatusEvent) error {
	args := m.Called(ctx, event)
	return args.Error(0)
}

func (m *mockNodeStorage) SaveNodeExecution(ctx context.Context, jobID string, nodeExecution *node.NodeExecution) error {
	args := m.Called(ctx, jobID, nodeExecution)
	return args.Error(0)
}

func TestNodeSubscriberUseCase_ProcessNodeEvent(t *testing.T) {
	ctx := context.Background()

	t.Run("Success: terminal status is persisted under jobID:nodeID", func(t *testing.T) {
		mockStorage := new(mockNodeStorage)
		u := NewNodeSubscriberUseCase(mockStorage)

		event := &node.NodeStatusEvent{
			JobID:  "job-123",
			NodeID: "node-456",
			Status: node.StatusCompleted,
		}

		mockStorage.On("SaveToRedis", ctx, event).Return(nil)
		mockStorage.
			On("SaveNodeExecution", ctx, "job-123", mock.MatchedBy(func(e *node.NodeExecution) bool {
				return e.ID == "job-123:node-456" &&
					e.JobID == "job-123" &&
					e.NodeID == "node-456" &&
					e.Status == node.StatusCompleted &&
					e.CompletedAt != nil
			})).
			Return(nil)

		assert.NoError(t, u.ProcessNodeEvent(ctx, event))
		mockStorage.AssertExpectations(t)
	})

	t.Run("Success: non-terminal status is only cached", func(t *testing.T) {
		mockStorage := new(mockNodeStorage)
		u := NewNodeSubscriberUseCase(mockStorage)

		event := &node.NodeStatusEvent{
			JobID:  "job-123",
			NodeID: "node-456",
			Status: node.StatusProcessing,
		}

		mockStorage.On("SaveToRedis", ctx, event).Return(nil)

		assert.NoError(t, u.ProcessNodeEvent(ctx, event))
		mockStorage.AssertExpectations(t)
		mockStorage.AssertNotCalled(t, "SaveNodeExecution")
	})

	t.Run("Error: event is nil", func(t *testing.T) {
		mockStorage := new(mockNodeStorage)
		u := NewNodeSubscriberUseCase(mockStorage)

		assert.Error(t, u.ProcessNodeEvent(ctx, nil))
	})

	t.Run("Error: SaveNodeExecution failure is propagated so the message is nacked", func(t *testing.T) {
		mockStorage := new(mockNodeStorage)
		u := NewNodeSubscriberUseCase(mockStorage)

		event := &node.NodeStatusEvent{
			JobID:  "job-123",
			NodeID: "node-456",
			Status: node.StatusCompleted,
		}

		saveErr := errors.New("storage unavailable")
		mockStorage.On("SaveToRedis", ctx, event).Return(nil)
		mockStorage.On("SaveNodeExecution", ctx, "job-123", mock.Anything).Return(saveErr)

		err := u.ProcessNodeEvent(ctx, event)
		assert.Error(t, err)
		assert.ErrorIs(t, err, saveErr)
		mockStorage.AssertExpectations(t)
	})
}
