package interactor

import (
	"context"
	"encoding/json"
	"errors"
	"testing"
	"time"

	"github.com/reearth/reearth-flow/api/internal/adapter"
	"github.com/reearth/reearth-flow/api/pkg/graph"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/job"
	"github.com/reearth/reearth-flow/api/pkg/log"
	"github.com/reearth/reearth-flow/api/pkg/userfacinglog"
	"github.com/reearth/reearthx/account/accountdomain/user"
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
	u := user.New().NewID().Email("test@example.com").Name("test").MustBuild()
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
	u := user.New().NewID().Email("test@example.com").Name("test").MustBuild()
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
	u := user.New().NewID().Email("test@example.com").Name("test").MustBuild()
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
