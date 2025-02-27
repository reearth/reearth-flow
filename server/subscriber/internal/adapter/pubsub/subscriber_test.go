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
	domainLog "github.com/reearth/reearth-flow/subscriber/pkg/log"
)

type mockSubscription struct {
	mock.Mock
}

func (m *mockSubscription) Receive(ctx context.Context, f func(context.Context, Message)) error {
	args := m.Called(ctx, f)
	return args.Error(0)
}

type mockMessage struct {
	mock.Mock
	data []byte
}

func (m *mockMessage) Data() []byte {
	return m.data
}
func (m *mockMessage) Ack() {
	m.Called()
}
func (m *mockMessage) Nack() {
	m.Called()
}

type mockUseCase struct {
	mock.Mock
}

func (m *mockUseCase) ProcessLogEvent(ctx context.Context, event *domainLog.LogEvent) error {
	args := m.Called(ctx, event)
	return args.Error(0)
}

func TestSubscriber_StartListening_Success(t *testing.T) {
	ctx := context.Background()

	mSub := new(mockSubscription)
	mUseCase := new(mockUseCase)
	mMsg := new(mockMessage)

	testEvent := domainLog.LogEvent{
		WorkflowID: "workflow-123",
		JobID:      "job-abc",
		Timestamp:  time.Now(),
		LogLevel:   domainLog.LogLevelInfo,
		Message:    "Hello from test",
	}
	data, _ := json.Marshal(testEvent)
	mMsg.data = data

	mMsg.On("Ack").Return().Once()
	mMsg.On("Nack").Return().Maybe()

	mUseCase.On("ProcessLogEvent", mock.Anything, mock.AnythingOfType("*log.LogEvent")).
		Return(nil).Once()

	mSub.On("Receive", ctx, mock.Anything).
		Run(func(args mock.Arguments) {
			cb := args.Get(1).(func(context.Context, Message))
			cb(ctx, mMsg)
		}).
		Return(nil).Once()

	s := NewSubscriber(mSub, mUseCase)
	err := s.StartListening(ctx)
	assert.NoError(t, err)

	mMsg.AssertExpectations(t)
	mUseCase.AssertExpectations(t)
	mSub.AssertExpectations(t)
}

func TestSubscriber_StartListening_ProcessError(t *testing.T) {
	ctx := context.Background()

	mSub := new(mockSubscription)
	mUseCase := new(mockUseCase)
	mMsg := new(mockMessage)

	testEvent := domainLog.LogEvent{
		WorkflowID: "wf",
		JobID:      "job",
	}
	data, _ := json.Marshal(testEvent)
	mMsg.data = data

	mUseCase.
		On("ProcessLogEvent", mock.Anything, mock.AnythingOfType("*log.LogEvent")).
		Return(errors.New("something failed")).Once()

	mMsg.On("Ack").Return().Maybe()
	mMsg.On("Nack").Return().Once()

	mSub.On("Receive", ctx, mock.Anything).
		Run(func(args mock.Arguments) {
			cb := args.Get(1).(func(context.Context, Message))
			cb(ctx, mMsg)
		}).
		Return(nil).Once()

	s := NewSubscriber(mSub, mUseCase)
	err := s.StartListening(ctx)
	assert.NoError(t, err)

	mMsg.AssertExpectations(t)
	mUseCase.AssertExpectations(t)
	mSub.AssertExpectations(t)
}

func TestSubscriber_StartListening_InvalidJSON(t *testing.T) {
	ctx := context.Background()

	mSub := new(mockSubscription)
	mUseCase := new(mockUseCase)
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

	s := NewSubscriber(mSub, mUseCase)
	err := s.StartListening(ctx)
	assert.NoError(t, err)

	mUseCase.AssertNotCalled(t, "ProcessLogEvent")

	mMsg.AssertExpectations(t)
	mUseCase.AssertExpectations(t)
	mSub.AssertExpectations(t)
}
