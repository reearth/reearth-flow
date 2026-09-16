package pubsub_test

import (
	"context"
	"encoding/json"
	"errors"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"

	. "github.com/reearth/reearth-flow/subscriber/internal/adapter/pubsub"
	"github.com/reearth/reearth-flow/subscriber/pkg/diagnostic"
)

type mockDiagnosticUseCase struct {
	mock.Mock
}

func (m *mockDiagnosticUseCase) ProcessDiagnosticEvent(ctx context.Context, event *diagnostic.DiagnosticEvent) error {
	args := m.Called(ctx, event)
	return args.Error(0)
}

func testDiagnosticEvent() diagnostic.DiagnosticEvent {
	nodeID := "subgraph-a.node-4"
	return diagnostic.DiagnosticEvent{
		Schema:     diagnostic.DiagnosticSchemaV1,
		WorkflowID: "workflow-123",
		JobID:      "job-abc",
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

func TestDiagnosticSubscriber_StartListening_Success(t *testing.T) {
	ctx := context.Background()

	mSub := new(mockSubscription)
	mUseCase := new(mockDiagnosticUseCase)
	mMsg := new(mockMessage)

	testEvent := testDiagnosticEvent()
	data, _ := json.Marshal(testEvent)
	mMsg.data = data

	mMsg.On("Ack").Return().Once()
	mMsg.On("Nack").Return().Maybe()

	mUseCase.On("ProcessDiagnosticEvent", mock.Anything, mock.AnythingOfType("*diagnostic.DiagnosticEvent")).
		Return(nil).Once()

	mSub.On("Receive", ctx, mock.Anything).
		Run(func(args mock.Arguments) {
			cb := args.Get(1).(func(context.Context, Message))
			cb(ctx, mMsg)
		}).
		Return(nil).Once()

	s := NewDiagnosticSubscriber(mSub, mUseCase)
	err := s.StartListening(ctx)
	assert.NoError(t, err)

	mMsg.AssertExpectations(t)
	mUseCase.AssertExpectations(t)
	mSub.AssertExpectations(t)
}

func TestDiagnosticSubscriber_StartListening_ProcessError(t *testing.T) {
	ctx := context.Background()

	mSub := new(mockSubscription)
	mUseCase := new(mockDiagnosticUseCase)
	mMsg := new(mockMessage)

	testEvent := testDiagnosticEvent()
	data, _ := json.Marshal(testEvent)
	mMsg.data = data

	mUseCase.
		On("ProcessDiagnosticEvent", mock.Anything, mock.AnythingOfType("*diagnostic.DiagnosticEvent")).
		Return(errors.New("processing failed")).Once()

	mMsg.On("Ack").Return().Maybe()
	mMsg.On("Nack").Return().Once()

	mSub.On("Receive", ctx, mock.Anything).
		Run(func(args mock.Arguments) {
			cb := args.Get(1).(func(context.Context, Message))
			cb(ctx, mMsg)
		}).
		Return(nil).Once()

	s := NewDiagnosticSubscriber(mSub, mUseCase)
	err := s.StartListening(ctx)
	assert.NoError(t, err)

	mMsg.AssertExpectations(t)
	mUseCase.AssertExpectations(t)
	mSub.AssertExpectations(t)
}

func TestDiagnosticSubscriber_StartListening_InvalidJSON(t *testing.T) {
	ctx := context.Background()

	mSub := new(mockSubscription)
	mUseCase := new(mockDiagnosticUseCase)
	mMsg := new(mockMessage)

	mMsg.data = []byte(`{ "invalid": ??? }`)

	mMsg.On("Ack").Return().Maybe()
	mMsg.On("Nack").Return().Once()

	mSub.On("Receive", ctx, mock.Anything).
		Run(func(args mock.Arguments) {
			cb := args.Get(1).(func(context.Context, Message))
			cb(ctx, mMsg)
		}).
		Return(nil).Once()

	s := NewDiagnosticSubscriber(mSub, mUseCase)
	err := s.StartListening(ctx)
	assert.NoError(t, err)

	mUseCase.AssertNotCalled(t, "ProcessDiagnosticEvent")

	mMsg.AssertExpectations(t)
	mUseCase.AssertExpectations(t)
	mSub.AssertExpectations(t)
}
