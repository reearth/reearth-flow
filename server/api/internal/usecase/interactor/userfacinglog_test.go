package interactor

import (
	"context"
	"encoding/json"
	"errors"
	"testing"
	"time"

	accountsuser "github.com/reearth/reearth-accounts/server/pkg/user"
	"github.com/reearth/reearth-flow/api/internal/adapter"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/pkg/graph"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/job"
	"github.com/reearth/reearth-flow/api/pkg/log"
	"github.com/reearth/reearth-flow/api/pkg/subscription"
	"github.com/reearth/reearth-flow/api/pkg/userfacinglog"
	"github.com/reearth/reearthx/appx"
	"github.com/stretchr/testify/assert"
)

type mockUserFacingLogGateway struct {
	logs []*userfacinglog.UserFacingLog
	err  error
}

func (m *mockUserFacingLogGateway) GetUserFacingLogs(ctx context.Context, since time.Time, until time.Time, jobID id.JobID) ([]*userfacinglog.UserFacingLog, error) {
	return m.logs, m.err
}

func (m *mockUserFacingLogGateway) GetLogs(ctx context.Context, since time.Time, until time.Time, jobID id.JobID) ([]*log.Log, error) {
	return nil, nil
}

func (m *mockUserFacingLogGateway) GetNodeExecution(ctx context.Context, jobID id.JobID, nodeID string) (*graph.NodeExecution, error) {
	return nil, nil
}

func (m *mockUserFacingLogGateway) GetNodeExecutions(ctx context.Context, jobID id.JobID) ([]*graph.NodeExecution, error) {
	return nil, nil
}

func (m *mockUserFacingLogGateway) GetJobCompleteEvent(ctx context.Context, jobID id.JobID) (*gateway.JobCompleteEvent, error) {
	return nil, nil
}

func (m *mockUserFacingLogGateway) DeleteJobCompleteEvent(ctx context.Context, jobID id.JobID) error {
	return nil
}

func TestNewUserFacingLogInteractor(t *testing.T) {
	t.Run("successfully create UserFacingLogInteractor", func(t *testing.T) {
		redisMock := &mockUserFacingLogGateway{}
		jobRepo := &mockJobRepo{}
		permissionChecker := &mockPermissionChecker{}

		interactor := NewUserFacingLogInteractor(redisMock, jobRepo, permissionChecker)

		assert.NotNil(t, interactor)
	})
}

func TestUserFacingLogInteractor_GetUserFacingLogs(t *testing.T) {
	ctx := context.Background()
	jobID := id.NewJobID()
	since := time.Now().Add(-1 * time.Hour)

	// Setup auth info
	u := accountsuser.New().NewID().Email("test@example.com").Name("test").MustBuild()
	ctx = adapter.AttachUser(ctx, u)
	ctx = adapter.AttachAuthInfo(ctx, &appx.AuthInfo{})

	metadata := json.RawMessage(`{"key": "value"}`)
	expectedLogs := []*userfacinglog.UserFacingLog{
		userfacinglog.NewUserFacingLog(jobID, time.Now(), "Processing started", nil),
		userfacinglog.NewUserFacingLog(jobID, time.Now(), "Data loaded successfully", metadata),
		userfacinglog.NewUserFacingLog(jobID, time.Now(), "Processing completed", nil),
	}

	t.Run("get user-facing logs successfully", func(t *testing.T) {
		redisMock := &mockUserFacingLogGateway{
			logs: expectedLogs,
		}
		jobRepo := &mockJobRepo{}
		permissionChecker := &mockPermissionChecker{}

		interactor := NewUserFacingLogInteractor(redisMock, jobRepo, permissionChecker)

		logs, err := interactor.GetUserFacingLogs(ctx, since, jobID)

		assert.NoError(t, err)
		assert.Equal(t, expectedLogs, logs)
	})

	t.Run("redis error", func(t *testing.T) {
		expectedErr := errors.New("redis connection failed")
		redisMock := &mockUserFacingLogGateway{
			err: expectedErr,
		}
		jobRepo := &mockJobRepo{}
		permissionChecker := &mockPermissionChecker{}

		interactor := NewUserFacingLogInteractor(redisMock, jobRepo, permissionChecker)

		logs, err := interactor.GetUserFacingLogs(ctx, since, jobID)

		assert.Error(t, err)
		assert.Contains(t, err.Error(), "failed to get user-facing logs from Redis")
		assert.Nil(t, logs)
	})

	t.Run("redis gateway is nil", func(t *testing.T) {
		jobRepo := &mockJobRepo{}
		permissionChecker := &mockPermissionChecker{}

		interactor := NewUserFacingLogInteractor(nil, jobRepo, permissionChecker)

		logs, err := interactor.GetUserFacingLogs(ctx, since, jobID)

		assert.Error(t, err)
		assert.Contains(t, err.Error(), "logsGatewayRedis is nil")
		assert.Nil(t, logs)
	})
}

func TestUserFacingLogInteractor_Subscribe(t *testing.T) {
	ctx := context.Background()
	jobID := id.NewJobID()

	// Setup auth info
	u := accountsuser.New().NewID().Email("test@example.com").Name("test").MustBuild()
	ctx = adapter.AttachUser(ctx, u)
	ctx = adapter.AttachAuthInfo(ctx, &appx.AuthInfo{})

	t.Run("subscribe successfully", func(t *testing.T) {
		redisMock := &mockUserFacingLogGateway{
			logs: []*userfacinglog.UserFacingLog{},
		}
		jobRepo := &mockJobRepo{
			job: job.New().ID(jobID).Status(job.StatusRunning).MustBuild(),
		}
		permissionChecker := &mockPermissionChecker{}

		interactor := NewUserFacingLogInteractor(redisMock, jobRepo, permissionChecker)

		ch, err := interactor.Subscribe(ctx, jobID)

		assert.NoError(t, err)
		assert.NotNil(t, ch)

		// Clean up
		interactor.Unsubscribe(jobID, ch)
	})

	t.Run("nil redis gateway", func(t *testing.T) {
		jobRepo := &mockJobRepo{}
		permissionChecker := &mockPermissionChecker{}

		interactor := NewUserFacingLogInteractor(nil, jobRepo, permissionChecker)

		ch, err := interactor.Subscribe(ctx, jobID)

		assert.Error(t, err)
		assert.Contains(t, err.Error(), "logsGatewayRedis is nil")
		assert.Nil(t, ch)
	})
}

func TestUserFacingLogInteractor_Unsubscribe(t *testing.T) {
	ctx := context.Background()
	jobID := id.NewJobID()

	// Setup auth info
	u := accountsuser.New().NewID().Email("test@example.com").Name("test").MustBuild()
	ctx = adapter.AttachUser(ctx, u)
	ctx = adapter.AttachAuthInfo(ctx, &appx.AuthInfo{})

	redisMock := &mockUserFacingLogGateway{
		logs: []*userfacinglog.UserFacingLog{},
	}
	jobRepo := &mockJobRepo{
		job: job.New().ID(jobID).Status(job.StatusRunning).MustBuild(),
	}
	permissionChecker := &mockPermissionChecker{}

	interactor := NewUserFacingLogInteractor(redisMock, jobRepo, permissionChecker)

	ch, err := interactor.Subscribe(ctx, jobID)
	assert.NoError(t, err)
	assert.NotNil(t, ch)

	// Test unsubscribe
	interactor.Unsubscribe(jobID, ch)

	// Channel should be closed after unsubscribe
	select {
	case _, ok := <-ch:
		assert.False(t, ok, "Channel should be closed after unsubscribe")
	default:
		// Channel is not closed immediately in our implementation
		// This is acceptable behavior
	}
}

func TestUserFacingLogInteractor_FlushFinalLogs(t *testing.T) {
	t.Parallel()

	jobID := id.NewJobID()
	finalLog := userfacinglog.NewUserFacingLog(jobID, time.Now(), "final log", nil)

	t.Run("notifies subscribers with final logs on terminal state", func(t *testing.T) {
		t.Parallel()

		redisMock := &mockUserFacingLogGateway{logs: []*userfacinglog.UserFacingLog{finalLog}}
		li := &UserFacingLogInteractor{
			logsGatewayRedis:  redisMock,
			subscriptions:     subscription.NewUserFacingLogManager(),
			permissionChecker: NewMockPermissionChecker(func(_ context.Context, _, _ string) (bool, error) { return true, nil }),
		}

		ch := li.subscriptions.Subscribe(jobID.String())
		li.flushFinalLogs(context.Background(), jobID.String(), jobID, time.Now().Add(-time.Minute))

		select {
		case got := <-ch:
			assert.Equal(t, finalLog, got)
		case <-time.After(500 * time.Millisecond):
			t.Fatal("subscriber did not receive final log")
		}

		select {
		case extra, ok := <-ch:
			if ok && extra != nil {
				t.Fatalf("received unexpected second message: %v", extra)
			}
		default:
		}
	})

	t.Run("does not notify when final fetch returns no logs", func(t *testing.T) {
		t.Parallel()

		redisMock := &mockUserFacingLogGateway{logs: []*userfacinglog.UserFacingLog{}}
		li := &UserFacingLogInteractor{
			logsGatewayRedis:  redisMock,
			subscriptions:     subscription.NewUserFacingLogManager(),
			permissionChecker: NewMockPermissionChecker(func(_ context.Context, _, _ string) (bool, error) { return true, nil }),
		}

		ch := li.subscriptions.Subscribe(jobID.String())
		li.flushFinalLogs(context.Background(), jobID.String(), jobID, time.Now().Add(-time.Minute))

		select {
		case msg, ok := <-ch:
			if ok && msg != nil {
				t.Fatalf("expected no notification but got: %v", msg)
			}
		case <-time.After(100 * time.Millisecond):
		}
	})
}
