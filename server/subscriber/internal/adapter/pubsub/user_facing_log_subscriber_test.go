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
	"github.com/reearth/reearth-flow/subscriber/pkg/userfacinglog"
)

type mockUserFacingLogUseCase struct {
	mock.Mock
}

func (m *mockUserFacingLogUseCase) ProcessUserFacingLogEvent(ctx context.Context, event *userfacinglog.UserFacingLogEvent) error {
	args := m.Called(ctx, event)
	return args.Error(0)
}

func TestUserFacingLogSubscriber_StartListening_Success(t *testing.T) {
	ctx := context.Background()

	mSub := new(mockSubscription)
	mUseCase := new(mockUserFacingLogUseCase)
	mMsg := new(mockMessage)

	nodeName := "test-node"
	nodeID := "node-123"
	testEvent := userfacinglog.UserFacingLogEvent{
		WorkflowID:     "workflow-123",
		JobID:          "job-abc",
		Timestamp:      time.Now(),
		Level:          userfacinglog.UserFacingLogLevelInfo,
		NodeName:       &nodeName,
		NodeID:         &nodeID,
		DisplayMessage: "User-facing log message",
	}
	data, _ := json.Marshal(testEvent)
	mMsg.data = data

	mMsg.On("Ack").Return().Once()
	mMsg.On("Nack").Return().Maybe()

	mUseCase.On("ProcessUserFacingLogEvent", mock.Anything, mock.AnythingOfType("*userfacinglog.UserFacingLogEvent")).
		Return(nil).Once()

	mSub.On("Receive", ctx, mock.Anything).
		Run(func(args mock.Arguments) {
			cb := args.Get(1).(func(context.Context, Message))
			cb(ctx, mMsg)
		}).
		Return(nil).Once()

	s := NewUserFacingLogSubscriber(mSub, mUseCase)
	err := s.StartListening(ctx)
	assert.NoError(t, err)

	mMsg.AssertExpectations(t)
	mUseCase.AssertExpectations(t)
	mSub.AssertExpectations(t)
}

func TestUserFacingLogSubscriber_StartListening_ProcessError(t *testing.T) {
	ctx := context.Background()

	mSub := new(mockSubscription)
	mUseCase := new(mockUserFacingLogUseCase)
	mMsg := new(mockMessage)

	testEvent := userfacinglog.UserFacingLogEvent{
		WorkflowID:     "wf",
		JobID:          "job",
		Timestamp:      time.Now(),
		Level:          userfacinglog.UserFacingLogLevelError,
		DisplayMessage: "Error occurred",
	}
	data, _ := json.Marshal(testEvent)
	mMsg.data = data

	mUseCase.
		On("ProcessUserFacingLogEvent", mock.Anything, mock.AnythingOfType("*userfacinglog.UserFacingLogEvent")).
		Return(errors.New("processing failed")).Once()

	mMsg.On("Ack").Return().Maybe()
	mMsg.On("Nack").Return().Once()

	mSub.On("Receive", ctx, mock.Anything).
		Run(func(args mock.Arguments) {
			cb := args.Get(1).(func(context.Context, Message))
			cb(ctx, mMsg)
		}).
		Return(nil).Once()

	s := NewUserFacingLogSubscriber(mSub, mUseCase)
	err := s.StartListening(ctx)
	assert.NoError(t, err)

	mMsg.AssertExpectations(t)
	mUseCase.AssertExpectations(t)
	mSub.AssertExpectations(t)
}

func TestUserFacingLogSubscriber_StartListening_InvalidJSON(t *testing.T) {
	ctx := context.Background()

	mSub := new(mockSubscription)
	mUseCase := new(mockUserFacingLogUseCase)
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

	s := NewUserFacingLogSubscriber(mSub, mUseCase)
	err := s.StartListening(ctx)
	assert.NoError(t, err)

	mUseCase.AssertNotCalled(t, "ProcessUserFacingLogEvent")

	mMsg.AssertExpectations(t)
	mUseCase.AssertExpectations(t)
	mSub.AssertExpectations(t)
}

func TestUserFacingLogSubscriber_StartListening_WithSuccessLevel(t *testing.T) {
	ctx := context.Background()

	mSub := new(mockSubscription)
	mUseCase := new(mockUserFacingLogUseCase)
	mMsg := new(mockMessage)

	testEvent := userfacinglog.UserFacingLogEvent{
		WorkflowID:     "workflow-456",
		JobID:          "job-def",
		Timestamp:      time.Now(),
		Level:          userfacinglog.UserFacingLogLevelSuccess,
		DisplayMessage: "Operation completed successfully",
	}
	data, _ := json.Marshal(testEvent)
	mMsg.data = data

	mMsg.On("Ack").Return().Once()
	mMsg.On("Nack").Return().Maybe()

	mUseCase.On("ProcessUserFacingLogEvent", mock.Anything, mock.AnythingOfType("*userfacinglog.UserFacingLogEvent")).
		Return(nil).Once()

	mSub.On("Receive", ctx, mock.Anything).
		Run(func(args mock.Arguments) {
			cb := args.Get(1).(func(context.Context, Message))
			cb(ctx, mMsg)
		}).
		Return(nil).Once()

	s := NewUserFacingLogSubscriber(mSub, mUseCase)
	err := s.StartListening(ctx)
	assert.NoError(t, err)

	mMsg.AssertExpectations(t)
	mUseCase.AssertExpectations(t)
	mSub.AssertExpectations(t)
}

func TestUserFacingLogSubscriber_StartListening_PanicRecovery(t *testing.T) {
	ctx := context.Background()

	mSub := new(mockSubscription)
	mUseCase := new(mockUserFacingLogUseCase)
	mMsg := new(mockMessage)

	testEvent := userfacinglog.UserFacingLogEvent{
		WorkflowID:     "workflow-789",
		JobID:          "job-ghi",
		Timestamp:      time.Now(),
		Level:          userfacinglog.UserFacingLogLevelInfo,
		DisplayMessage: "Test message",
	}
	data, _ := json.Marshal(testEvent)
	mMsg.data = data

	// Simulate a panic in the use case
	mUseCase.On("ProcessUserFacingLogEvent", mock.Anything, mock.AnythingOfType("*userfacinglog.UserFacingLogEvent")).
		Run(func(args mock.Arguments) {
			panic("test panic")
		}).Once()

	// Message should not be acknowledged or nacked when panic occurs
	// The panic is recovered, so no Ack or Nack is called

	mSub.On("Receive", ctx, mock.Anything).
		Run(func(args mock.Arguments) {
			cb := args.Get(1).(func(context.Context, Message))
			cb(ctx, mMsg)
		}).
		Return(nil).Once()

	s := NewUserFacingLogSubscriber(mSub, mUseCase)
	err := s.StartListening(ctx)
	assert.NoError(t, err)

	mMsg.AssertNotCalled(t, "Ack")
	mMsg.AssertNotCalled(t, "Nack")
	mUseCase.AssertExpectations(t)
	mSub.AssertExpectations(t)
}
