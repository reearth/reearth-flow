package interactor

import (
	"context"
	"errors"
	"testing"
	"time"

	"github.com/reearth/reearth-flow/api/internal/adapter"
	"github.com/reearth/reearth-flow/api/pkg/edge"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/log"
	"github.com/reearth/reearthx/account/accountdomain/user"
	"github.com/reearth/reearthx/appx"
	"github.com/stretchr/testify/assert"
)

type mockLogGateway struct {
	logs []*log.Log
	err  error
}

// GetEdgeExecution implements gateway.Redis.
func (m *mockLogGateway) GetEdgeExecution(ctx context.Context, jobID id.JobID, edgeID string) (*edge.EdgeExecution, error) {
	panic("unimplemented")
}

// GetEdgeExecutions implements gateway.Redis.
func (m *mockLogGateway) GetEdgeExecutions(ctx context.Context, jobID id.JobID) ([]*edge.EdgeExecution, error) {
	panic("unimplemented")
}

// SubscribeToEdgeStatus implements gateway.Redis.
func (m *mockLogGateway) SubscribeToEdgeStatus(ctx context.Context, jobID id.JobID, edgeID string) (<-chan edge.Status, error) {
	panic("unimplemented")
}

func (m *mockLogGateway) GetLogs(ctx context.Context, since time.Time, until time.Time, jobID id.JobID) ([]*log.Log, error) {
	return m.logs, m.err
}

func TestNewLogInteractor(t *testing.T) {
	t.Run("successfully create LogInteractor", func(t *testing.T) {
		redisMock := &mockLogGateway{}
		mockPermissionCheckerTrue := NewMockPermissionChecker(func(ctx context.Context, authInfo *appx.AuthInfo, userId, resource, action string) (bool, error) {
			return true, nil
		})
		li := NewLogInteractor(redisMock, mockPermissionCheckerTrue)
		assert.NotNil(t, li)
	})
}

func TestLogInteractor_GetLogs(t *testing.T) {
	mockAuthInfo := &appx.AuthInfo{
		Token: "token",
	}
	mockUser := user.New().NewID().Name("hoge").Email("abc@bb.cc").MustBuild()

	ctx := context.Background()
	ctx = adapter.AttachAuthInfo(ctx, mockAuthInfo)
	ctx = adapter.AttachUser(ctx, mockUser)

	nodeID := log.NodeID(id.NewNodeID())
	jobID := id.NewJobID()
	redisLogs := []*log.Log{
		log.NewLog(jobID, &nodeID, time.Now(), log.LevelInfo, "redis log 1"),
		log.NewLog(jobID, &nodeID, time.Now(), log.LevelInfo, "redis log 2"),
	}
	redisMock := &mockLogGateway{logs: redisLogs}
	mockPermissionCheckerTrue := NewMockPermissionChecker(func(ctx context.Context, authInfo *appx.AuthInfo, userId, resource, action string) (bool, error) {
		return true, nil
	})

	t.Run("get Redis logs", func(t *testing.T) {
		li := NewLogInteractor(redisMock, mockPermissionCheckerTrue)

		since := time.Now().Add(-30 * time.Minute)
		out, err := li.GetLogs(ctx, since, id.NewJobID())
		assert.NoError(t, err)
		assert.Equal(t, redisLogs, out)
	})

	t.Run("redis error", func(t *testing.T) {
		brokenRedis := &mockLogGateway{err: errors.New("redis error")}
		li := NewLogInteractor(brokenRedis, mockPermissionCheckerTrue)

		since := time.Now()
		out, err := li.GetLogs(ctx, since, id.NewJobID())
		assert.Nil(t, out)
		assert.Error(t, err)
		assert.Contains(t, err.Error(), "failed to get logs from Redis")
	})

	t.Run("redis gateway is nil", func(t *testing.T) {
		li := NewLogInteractor(nil, mockPermissionCheckerTrue)
		since := time.Now().Add(-30 * time.Minute)
		out, err := li.GetLogs(ctx, since, jobID)
		assert.Nil(t, out)
		assert.Error(t, err)
		assert.Contains(t, err.Error(), "logsGatewayRedis is nil")
	})
}

func TestLogInteractor_SubscribeInitialLogs(t *testing.T) {
	jobID := id.NewJobID()
	nodeID := log.NodeID(id.NewNodeID())
	initialLog := log.NewLog(jobID, &nodeID, time.Now(), log.LevelInfo, "initial log")
	redisMock := &mockLogGateway{
		logs: []*log.Log{initialLog},
	}
	mockPermissionCheckerTrue := NewMockPermissionChecker(func(ctx context.Context, authInfo *appx.AuthInfo, userId, resource, action string) (bool, error) {
		return true, nil
	})
	li := NewLogInteractor(redisMock, mockPermissionCheckerTrue)

	mockAuthInfo := &appx.AuthInfo{
		Token: "token",
	}
	mockUser := user.New().NewID().Name("hoge").Email("abc@bb.cc").MustBuild()

	ctx := context.Background()
	ctx = adapter.AttachAuthInfo(ctx, mockAuthInfo)
	ctx = adapter.AttachUser(ctx, mockUser)

	ch, err := li.Subscribe(ctx, jobID)
	assert.NoError(t, err)

	select {
	case logEntry := <-ch:
		assert.Equal(t, initialLog, logEntry)
	case <-time.After(500 * time.Millisecond):
		t.Fatalf("Timeout waiting for initial log notification")
	}

	li.Unsubscribe(jobID, ch)
}

func TestLogInteractor_Unsubscribe(t *testing.T) {
	redisMock := &mockLogGateway{}
	mockPermissionCheckerTrue := NewMockPermissionChecker(func(ctx context.Context, authInfo *appx.AuthInfo, userId, resource, action string) (bool, error) {
		return true, nil
	})
	liInterface := NewLogInteractor(redisMock, mockPermissionCheckerTrue)
	li, ok := liInterface.(*LogInteractor)
	if !ok {
		t.Fatal("expected *LogInteractor")
	}
	jobID := id.NewJobID()

	mockAuthInfo := &appx.AuthInfo{
		Token: "token",
	}
	mockUser := user.New().NewID().Name("hoge").Email("abc@bb.cc").MustBuild()

	ctx := context.Background()
	ctx = adapter.AttachAuthInfo(ctx, mockAuthInfo)
	ctx = adapter.AttachUser(ctx, mockUser)

	ch, err := liInterface.Subscribe(ctx, jobID)
	if err != nil {
		t.Fatal(err)
	}

	li.Unsubscribe(jobID, ch)

	testLog2 := log.NewLog(jobID, nil, time.Now(), log.LevelInfo, "test log 2")
	li.subscriptions.Notify(jobID.String(), []*log.Log{testLog2})

	select {
	case l, ok := <-ch:
		if ok && l != nil {
			t.Fatalf("Channel received a non-nil log after unsubscription: %v", l)
		}
	case <-time.After(100 * time.Millisecond):
	}
}
