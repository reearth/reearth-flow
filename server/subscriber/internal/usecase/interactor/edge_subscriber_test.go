package interactor

import (
	"context"
	"errors"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"

	"github.com/reearth/reearth-flow/subscriber/pkg/edge"
)

type mockEdgeStorage struct {
	mock.Mock
}

func (m *mockEdgeStorage) SaveToRedis(ctx context.Context, event *edge.PassThroughEvent) error {
	args := m.Called(ctx, event)
	return args.Error(0)
}

func (m *mockEdgeStorage) UpdateEdgeStatusInMongo(ctx context.Context, jobID string, exec *edge.EdgeExecution) error {
	args := m.Called(ctx, jobID, exec)
	return args.Error(0)
}

func (m *mockEdgeStorage) ConstructIntermediateDataURL(jobID, edgeID string) string {
	args := m.Called(jobID, edgeID)
	return args.String(0)
}

func TestEdgeSubscriberUseCase_ProcessEdgeEvent(t *testing.T) {
	ctx := context.Background()
	mockStorage := new(mockEdgeStorage)
	u := NewEdgeSubscriberUseCase(mockStorage)

	t.Run("Success: storing event and updating edges", func(t *testing.T) {
		event := &edge.PassThroughEvent{
			WorkflowID: "wf-123",
			JobID:      "job-123",
			Status:     edge.StatusInProgress,
			Timestamp:  time.Now(),
			UpdatedEdges: []edge.UpdatedEdge{
				{ID: "edge-1", Status: edge.StatusInProgress},
				{ID: "edge-2", Status: edge.StatusCompleted},
			},
		}

		mockStorage.On("SaveToRedis", ctx, event).Return(nil)
		mockStorage.On("ConstructIntermediateDataURL", event.JobID, "edge-1").Return("http://example.com/edge-1")
		mockStorage.On("ConstructIntermediateDataURL", event.JobID, "edge-2").Return("http://example.com/edge-2")

		for range event.UpdatedEdges {
			mockStorage.On("UpdateEdgeStatusInMongo", ctx, event.JobID, mock.Anything).Return(nil)
		}

		err := u.ProcessEdgeEvent(ctx, event)
		assert.NoError(t, err)
		mockStorage.AssertExpectations(t)
	})

	t.Run("Error: event is nil", func(t *testing.T) {
		err := u.ProcessEdgeEvent(ctx, nil)
		assert.Error(t, err)
		assert.Equal(t, "event is nil", err.Error())
	})

	t.Run("Error: storing to Redis fails", func(t *testing.T) {
		event := &edge.PassThroughEvent{
			WorkflowID: "wf-123",
			JobID:      "job-123",
			Status:     edge.StatusInProgress,
			Timestamp:  time.Now(),
		}

		mockStorage.On("SaveToRedis", ctx, event).Return(errors.New("redis error"))
		err := u.ProcessEdgeEvent(ctx, event)
		assert.Error(t, err)
		assert.Contains(t, err.Error(), "failed to write to Redis: redis error")
	})
}
