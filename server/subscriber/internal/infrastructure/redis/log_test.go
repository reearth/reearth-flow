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

type mockRedisClient struct {
	mock.Mock
}

// Expire implements RedisClient.
func (m *mockRedisClient) Expire(ctx context.Context, key string, expiration time.Duration) *redis.BoolCmd {
	panic("unimplemented")
}

// LPush implements RedisClient.
func (m *mockRedisClient) LPush(ctx context.Context, key string, values ...interface{}) *redis.IntCmd {
	panic("unimplemented")
}

func (m *mockRedisClient) Set(ctx context.Context, key string, value interface{}, expiration time.Duration) *redis.StatusCmd {
	args := m.Called(ctx, key, value, expiration)

	err, _ := args.Get(0).(error)
	statusCmd := redis.NewStatusCmd(ctx)
	statusCmd.SetErr(err)
	return statusCmd
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

	mClient.
		On("Set", ctx, expectedKey, expectedVal, 12*time.Hour).
		Return(nil)

	err := rStorage.SaveLogToRedis(ctx, event)
	assert.NoError(t, err)
	mClient.AssertExpectations(t)
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
}
