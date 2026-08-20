package redis

import (
	"context"
	"errors"
	"testing"
	"time"

	"github.com/redis/go-redis/v9"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"

	domainLog "github.com/reearth/reearth-flow/subscriber/pkg/log"
)

// mockRedisClient records the sequence of Redis commands issued so tests can
// assert exact command sequences, not just "no error".
type mockRedisClient struct {
	mock.Mock
	calls []string
}

func (m *mockRedisClient) LPush(ctx context.Context, key string, values ...interface{}) *redis.IntCmd {
	m.calls = append(m.calls, "LPush")
	args := m.Called(ctx, key, values)
	cmd := redis.NewIntCmd(ctx)
	cmd.SetErr(args.Error(0))
	return cmd
}

func (m *mockRedisClient) Expire(ctx context.Context, key string, expiration time.Duration) *redis.BoolCmd {
	m.calls = append(m.calls, "Expire")
	args := m.Called(ctx, key, expiration)
	cmd := redis.NewBoolCmd(ctx)
	cmd.SetErr(args.Error(0))
	return cmd
}

func (m *mockRedisClient) Set(ctx context.Context, key string, value interface{}, expiration time.Duration) *redis.StatusCmd {
	m.calls = append(m.calls, "Set")
	args := m.Called(ctx, key, value, expiration)
	cmd := redis.NewStatusCmd(ctx)
	cmd.SetErr(args.Error(0))
	return cmd
}

func (m *mockRedisClient) XAdd(ctx context.Context, a *redis.XAddArgs) *redis.StringCmd {
	m.calls = append(m.calls, "XAdd")
	args := m.Called(ctx, *a)
	cmd := redis.NewStringCmd(ctx)
	cmd.SetErr(args.Error(0))
	return cmd
}

func (m *mockRedisClient) HSet(ctx context.Context, key string, values ...interface{}) *redis.IntCmd {
	m.calls = append(m.calls, "HSet")
	args := m.Called(ctx, key, values)
	cmd := redis.NewIntCmd(ctx)
	cmd.SetErr(args.Error(0))
	return cmd
}

func TestRedisStorage_SaveLogToRedis(t *testing.T) {
	ctx := context.Background()
	mClient := new(mockRedisClient)
	rStorage := NewRedisStorage(mClient)

	event := &domainLog.LogEvent{
		WorkflowID: "wf-123",
		JobID:      "job-123",
		Timestamp:  time.Date(2025, 1, 11, 9, 12, 54, 487779000, time.UTC),
		LogLevel:   domainLog.LogLevelInfo,
		Message:    "Hello from test",
		NodeID:     nil,
	}

	expectedKey := "log:wf-123:job-123:2025-01-11T09:12:54.487779Z"
	expectedVal := `{"workflowId":"wf-123","jobId":"job-123","timestamp":"2025-01-11T09:12:54.487779Z","logLevel":"INFO","message":"Hello from test"}`
	expectedStreamKey := "log:job-123"
	expectedID := "1736586774487-*"

	mClient.
		On("Set", mock.Anything, expectedKey, expectedVal, 12*time.Hour).
		Return(nil)
	mClient.
		On("XAdd", mock.Anything, redis.XAddArgs{
			Stream: expectedStreamKey,
			ID:     expectedID,
			MaxLen: logStreamMaxLen,
			Approx: true,
			Values: map[string]interface{}{"data": expectedVal},
		}).
		Return(nil)
	mClient.
		On("Expire", mock.Anything, expectedStreamKey, 12*time.Hour).
		Return(nil)

	err := rStorage.SaveLogToRedis(ctx, event)
	assert.NoError(t, err)
	mClient.AssertExpectations(t)
	assert.Equal(t, []string{"Set", "XAdd", "Expire"}, mClient.calls)
}

func TestRedisStorage_SaveLogToRedis_Error(t *testing.T) {
	ctx := context.Background()
	mClient := new(mockRedisClient)
	rStorage := NewRedisStorage(mClient)

	event := &domainLog.LogEvent{
		WorkflowID: "wf-123",
		JobID:      "job-123",
		Timestamp:  time.Now(),
		LogLevel:   domainLog.LogLevelInfo,
		Message:    "Hello from test",
	}

	mClient.
		On("Set", mock.Anything, mock.Anything, mock.Anything, 12*time.Hour).
		Return(errors.New("redis set error"))

	err := rStorage.SaveLogToRedis(ctx, event)
	assert.EqualError(t, err, "redis set error")
	assert.Equal(t, []string{"Set"}, mClient.calls)
}

func TestRedisStorage_SaveLogToRedis_StreamError(t *testing.T) {
	ctx := context.Background()
	mClient := new(mockRedisClient)
	rStorage := NewRedisStorage(mClient)

	event := &domainLog.LogEvent{
		WorkflowID: "wf-123",
		JobID:      "job-123",
		Timestamp:  time.Now(),
		LogLevel:   domainLog.LogLevelInfo,
		Message:    "Hello from test",
	}

	mClient.
		On("Set", mock.Anything, mock.Anything, mock.Anything, 12*time.Hour).
		Return(nil)
	mClient.
		On("XAdd", mock.Anything, mock.Anything).
		Return(errors.New("redis xadd error"))

	err := rStorage.SaveLogToRedis(ctx, event)
	assert.Error(t, err)
	assert.Contains(t, err.Error(), "failed to add log to redis stream")
	assert.Equal(t, []string{"Set", "XAdd"}, mClient.calls)
}
