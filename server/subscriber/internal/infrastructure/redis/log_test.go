package redis

import (
	"context"
	"errors"
	"fmt"
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

// orderingRedisClient simulates real Redis' rejection of an explicit XADD ID
// that is equal to or smaller than a stream's current top entry. Auto IDs
// ("" or "*") are always accepted, matching real Redis behavior, so this only
// trips on the old explicit-`{ms}-*`-ID code path.
type orderingRedisClient struct {
	streamTopMs map[string]int64
}

func newOrderingRedisClient() *orderingRedisClient {
	return &orderingRedisClient{streamTopMs: make(map[string]int64)}
}

func (m *orderingRedisClient) LPush(ctx context.Context, key string, values ...interface{}) *redis.IntCmd {
	return redis.NewIntCmd(ctx)
}

func (m *orderingRedisClient) Expire(ctx context.Context, key string, expiration time.Duration) *redis.BoolCmd {
	return redis.NewBoolCmd(ctx)
}

func (m *orderingRedisClient) Set(ctx context.Context, key string, value interface{}, expiration time.Duration) *redis.StatusCmd {
	return redis.NewStatusCmd(ctx)
}

func (m *orderingRedisClient) XAdd(ctx context.Context, a *redis.XAddArgs) *redis.StringCmd {
	cmd := redis.NewStringCmd(ctx)
	if a.ID != "" && a.ID != "*" {
		var ms int64
		if _, err := fmt.Sscanf(a.ID, "%d-*", &ms); err == nil {
			if top, ok := m.streamTopMs[a.Stream]; ok && ms <= top {
				cmd.SetErr(errors.New("ERR The ID specified in XADD is equal or smaller than the target stream top item"))
				return cmd
			}
			m.streamTopMs[a.Stream] = ms
		}
	} else {
		m.streamTopMs[a.Stream]++
	}
	cmd.SetVal("0-0")
	return cmd
}

func parseStreamMinID(minID string) (int64, error) {
	var ms int64
	_, err := fmt.Sscanf(minID, "%d", &ms)
	return ms, err
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
	expectedMinID := time.Now().Add(-streamRetention).UnixMilli()

	mClient.
		On("Set", mock.Anything, expectedKey, expectedVal, 12*time.Hour).
		Return(nil)
	mClient.
		On("XAdd", mock.Anything, mock.MatchedBy(func(a redis.XAddArgs) bool {
			if a.Stream != expectedStreamKey || a.ID != "" || !a.Approx {
				return false
			}
			minIDMs, err := parseStreamMinID(a.MinID)
			if err != nil || minIDMs < expectedMinID-5000 || minIDMs > expectedMinID+5000 {
				return false
			}
			values, ok := a.Values.(map[string]interface{})
			if !ok {
				return false
			}
			return values["data"] == expectedVal && values["timestampMs"] == event.Timestamp.UnixMilli()
		})).
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

// A failure after a successful XAdd must not fail the whole save, or Pub/Sub
// redelivery would append a duplicate stream entry on every retry.
func TestRedisStorage_SaveLogToRedis_ExpireFailureIsNonFatal(t *testing.T) {
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
		Return(nil)
	mClient.
		On("Expire", mock.Anything, mock.Anything, mock.Anything).
		Return(errors.New("redis expire error"))

	err := rStorage.SaveLogToRedis(ctx, event)
	assert.NoError(t, err)
	assert.Equal(t, []string{"Set", "XAdd", "Expire"}, mClient.calls)
}

// Regression pin for the explicit `{ms}-*` XADD ID bug: two events for the same
// job whose timestamps go backwards (as Pub/Sub redelivery can produce) must
// both be written without error. Against the old explicit-ID code, the second
// XAdd deterministically fails because its ID is <= the stream's top entry.
func TestRedisStorage_SaveLogToRedis_OutOfOrderEvents(t *testing.T) {
	ctx := context.Background()
	client := newOrderingRedisClient()
	rStorage := NewRedisStorage(client)

	later := time.Date(2025, 1, 11, 9, 12, 54, 0, time.UTC)
	earlier := later.Add(-time.Second)

	eventLater := &domainLog.LogEvent{
		WorkflowID: "wf-123",
		JobID:      "job-ooo",
		Timestamp:  later,
		LogLevel:   domainLog.LogLevelInfo,
		Message:    "later",
	}
	eventEarlier := &domainLog.LogEvent{
		WorkflowID: "wf-123",
		JobID:      "job-ooo",
		Timestamp:  earlier,
		LogLevel:   domainLog.LogLevelInfo,
		Message:    "earlier",
	}

	assert.NoError(t, rStorage.SaveLogToRedis(ctx, eventLater))
	assert.NoError(t, rStorage.SaveLogToRedis(ctx, eventEarlier))
}
