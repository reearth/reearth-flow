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
	"github.com/reearth/reearth-flow/subscriber/pkg/edge"
)

type mockEdgeSubscription struct {
	mock.Mock
}

func (m *mockEdgeSubscription) Receive(ctx context.Context, f func(context.Context, Message)) error {
	args := m.Called(ctx, f)
	return args.Error(0)
}

type mockEdgeMessage struct {
	mock.Mock
	data []byte
}

func (m *mockEdgeMessage) Data() []byte {
	return m.data
}
func (m *mockEdgeMessage) Ack() {
	m.Called()
}
func (m *mockEdgeMessage) Nack() {
	m.Called()
}

type mockEdgeUseCase struct {
	mock.Mock
}

func (m *mockEdgeUseCase) ProcessEdgeEvent(ctx context.Context, event *edge.PassThroughEvent) error {
	args := m.Called(ctx, event)
	return args.Error(0)
}

func TestEdgeSubscriber_StartListening_Success(t *testing.T) {
	ctx := context.Background()

	mSub := new(mockEdgeSubscription)
	mUseCase := new(mockEdgeUseCase)
	mMsg := new(mockEdgeMessage)

	timestamp := time.Now()
	testEvent := edge.PassThroughEvent{
		WorkflowID:   "workflow-123",
		JobID:        "job-xyz",
		Status:       edge.StatusCompleted,
		Timestamp:    timestamp,
		UpdatedEdges: []edge.UpdatedEdge{{ID: "edge-1", Status: edge.StatusInProgress}},
	}
	data, _ := json.Marshal(testEvent)
	mMsg.data = data

	mMsg.On("Ack").Return().Once()
	mMsg.On("Nack").Return().Maybe()

	mUseCase.On("ProcessEdgeEvent", mock.Anything, mock.AnythingOfType("*edge.PassThroughEvent")).
		Return(nil).Once()

	mSub.On("Receive", ctx, mock.Anything).
		Run(func(args mock.Arguments) {
			cb := args.Get(1).(func(context.Context, Message))
			cb(ctx, mMsg)
		}).
		Return(nil).Once()

	s := NewEdgeSubscriber(mSub, mUseCase)
	err := s.StartListening(ctx)
	assert.NoError(t, err)

	mMsg.AssertExpectations(t)
	mUseCase.AssertExpectations(t)
	mSub.AssertExpectations(t)
}

func TestEdgeSubscriber_StartListening_ProcessError(t *testing.T) {
	ctx := context.Background()

	mSub := new(mockEdgeSubscription)
	mUseCase := new(mockEdgeUseCase)
	mMsg := new(mockEdgeMessage)

	testEvent := edge.PassThroughEvent{
		WorkflowID: "workflow-123",
		JobID:      "job-123",
		Status:     edge.StatusFailed,
	}
	data, _ := json.Marshal(testEvent)
	mMsg.data = data

	mUseCase.On("ProcessEdgeEvent", mock.Anything, mock.AnythingOfType("*edge.PassThroughEvent")).
		Return(errors.New("processing failed")).Once()

	mMsg.On("Ack").Return().Maybe()
	mMsg.On("Nack").Return().Once()

	mSub.On("Receive", ctx, mock.Anything).
		Run(func(args mock.Arguments) {
			cb := args.Get(1).(func(context.Context, Message))
			cb(ctx, mMsg)
		}).
		Return(nil).Once()

	s := NewEdgeSubscriber(mSub, mUseCase)
	err := s.StartListening(ctx)
	assert.NoError(t, err)

	mMsg.AssertExpectations(t)
	mUseCase.AssertExpectations(t)
	mSub.AssertExpectations(t)
}

func TestEdgeSubscriber_StartListening_InvalidJSON(t *testing.T) {
	ctx := context.Background()

	mSub := new(mockEdgeSubscription)
	mUseCase := new(mockEdgeUseCase)
	mMsg := new(mockEdgeMessage)

	mMsg.data = []byte(`{ "invalid": ??? }`)

	mMsg.On("Ack").Return().Maybe()
	mMsg.On("Nack").Return().Once()

	mSub.On("Receive", ctx, mock.Anything).
		Run(func(args mock.Arguments) {
			cb := args.Get(1).(func(context.Context, Message))
			cb(ctx, mMsg)
		}).
		Return(nil).Once()

	s := NewEdgeSubscriber(mSub, mUseCase)
	err := s.StartListening(ctx)
	assert.NoError(t, err)

	mUseCase.AssertNotCalled(t, "ProcessEdgeEvent")

	mMsg.AssertExpectations(t)
	mUseCase.AssertExpectations(t)
	mSub.AssertExpectations(t)
}
