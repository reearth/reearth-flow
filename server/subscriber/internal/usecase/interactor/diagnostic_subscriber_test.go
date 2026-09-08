package interactor

import (
	"context"
	"errors"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"

	"github.com/reearth/reearth-flow/subscriber/pkg/diagnostic"
)

type mockDiagnosticStorage struct {
	mock.Mock
}

func (m *mockDiagnosticStorage) SaveToRedis(ctx context.Context, event *diagnostic.DiagnosticEvent) error {
	args := m.Called(ctx, event)
	return args.Error(0)
}

func (m *mockDiagnosticStorage) SaveToMongo(ctx context.Context, event *diagnostic.DiagnosticEvent) error {
	args := m.Called(ctx, event)
	return args.Error(0)
}

func validDiagnosticEvent() *diagnostic.DiagnosticEvent {
	nodeID := "node-1"
	return &diagnostic.DiagnosticEvent{
		Schema:     diagnostic.DiagnosticSchemaV1,
		WorkflowID: "wf-123",
		JobID:      "job-123",
		Timestamp:  time.Now(),
		WireDiagnostic: diagnostic.WireDiagnostic{
			Code:     "gltf.zero_face_solid",
			Category: "gltf",
			Severity: "warn",
			NodeID:   &nodeID,
			Message:  "solid has zero faces and was dropped",
		},
	}
}

func TestDiagnosticSubscriberUseCase_ProcessDiagnosticEvent(t *testing.T) {
	ctx := context.Background()

	t.Run("Success: Redis and Mongo both succeed", func(t *testing.T) {
		mockStorage := new(mockDiagnosticStorage)
		u := NewDiagnosticSubscriberUseCase(mockStorage)
		event := validDiagnosticEvent()

		mockStorage.On("SaveToRedis", ctx, event).Return(nil).Once()
		mockStorage.On("SaveToMongo", ctx, event).Return(nil).Once()

		err := u.ProcessDiagnosticEvent(ctx, event)
		assert.NoError(t, err)

		mockStorage.AssertExpectations(t)
	})

	t.Run("Success: Mongo failure is logged, not returned (not Nacked)", func(t *testing.T) {
		mockStorage := new(mockDiagnosticStorage)
		u := NewDiagnosticSubscriberUseCase(mockStorage)
		event := validDiagnosticEvent()

		mockStorage.On("SaveToRedis", ctx, event).Return(nil).Once()
		mockStorage.On("SaveToMongo", ctx, event).Return(errors.New("mongo down")).Once()

		err := u.ProcessDiagnosticEvent(ctx, event)
		assert.NoError(t, err)

		mockStorage.AssertExpectations(t)
	})

	t.Run("Error: event is nil", func(t *testing.T) {
		mockStorage := new(mockDiagnosticStorage)
		u := NewDiagnosticSubscriberUseCase(mockStorage)

		err := u.ProcessDiagnosticEvent(ctx, nil)
		assert.Error(t, err)
		assert.Contains(t, err.Error(), "diagnostic event is nil")

		mockStorage.AssertNotCalled(t, "SaveToRedis")
		mockStorage.AssertNotCalled(t, "SaveToMongo")
	})

	t.Run("Error: invalid schema", func(t *testing.T) {
		mockStorage := new(mockDiagnosticStorage)
		u := NewDiagnosticSubscriberUseCase(mockStorage)
		event := validDiagnosticEvent()
		event.Schema = "bogus.v1"

		err := u.ProcessDiagnosticEvent(ctx, event)
		assert.Error(t, err)
		assert.Contains(t, err.Error(), "unexpected schema")

		mockStorage.AssertNotCalled(t, "SaveToRedis")
		mockStorage.AssertNotCalled(t, "SaveToMongo")
	})

	t.Run("Error: missing jobId", func(t *testing.T) {
		mockStorage := new(mockDiagnosticStorage)
		u := NewDiagnosticSubscriberUseCase(mockStorage)
		event := validDiagnosticEvent()
		event.JobID = ""

		err := u.ProcessDiagnosticEvent(ctx, event)
		assert.Error(t, err)
		assert.Contains(t, err.Error(), "missing job ID")

		mockStorage.AssertNotCalled(t, "SaveToRedis")
		mockStorage.AssertNotCalled(t, "SaveToMongo")
	})

	t.Run("Error: storing to Redis fails", func(t *testing.T) {
		mockStorage := new(mockDiagnosticStorage)
		u := NewDiagnosticSubscriberUseCase(mockStorage)
		event := validDiagnosticEvent()

		mockStorage.On("SaveToRedis", ctx, event).Return(errors.New("redis connection error")).Once()

		err := u.ProcessDiagnosticEvent(ctx, event)
		assert.Error(t, err)
		assert.Contains(t, err.Error(), "failed to write diagnostic event to Redis")

		mockStorage.AssertExpectations(t)
		mockStorage.AssertNotCalled(t, "SaveToMongo")
	})

	t.Run("Success: unknown severity/category strings pass through unvalidated", func(t *testing.T) {
		mockStorage := new(mockDiagnosticStorage)
		u := NewDiagnosticSubscriberUseCase(mockStorage)
		event := validDiagnosticEvent()
		event.Severity = "future_severity"
		event.Category = "future_category"

		mockStorage.On("SaveToRedis", ctx, event).Return(nil).Once()
		mockStorage.On("SaveToMongo", ctx, event).Return(nil).Once()

		err := u.ProcessDiagnosticEvent(ctx, event)
		assert.NoError(t, err)

		mockStorage.AssertExpectations(t)
	})
}

func TestNewDiagnosticSubscriberUseCase(t *testing.T) {
	mockStorage := new(mockDiagnosticStorage)
	u := NewDiagnosticSubscriberUseCase(mockStorage)
	assert.NotNil(t, u)
}
