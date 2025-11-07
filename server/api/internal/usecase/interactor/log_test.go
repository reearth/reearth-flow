package interactor

import (
	"context"
	"errors"
	"testing"
	"time"

	"github.com/reearth/reearth-flow/api/internal/adapter"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/graph"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/job"
	"github.com/reearth/reearth-flow/api/pkg/log"
	"github.com/reearth/reearth-flow/api/pkg/user"
	"github.com/reearth/reearth-flow/api/pkg/userfacinglog"
	"github.com/reearth/reearthx/appx"
	"github.com/stretchr/testify/assert"
)

type mockLogGateway struct {
	logs []*log.Log
	err  error
}

// GetNodeExecution implements gateway.Redis.
func (m *mockLogGateway) GetNodeExecution(ctx context.Context, jobID id.JobID, edgeID string) (*graph.NodeExecution, error) {
	panic("unimplemented")
}

// GetNodeExecutions implements gateway.Redis.
func (m *mockLogGateway) GetNodeExecutions(ctx context.Context, jobID id.JobID) ([]*graph.NodeExecution, error) {
	panic("unimplemented")
}

func (m *mockLogGateway) GetLogs(ctx context.Context, since time.Time, until time.Time, jobID id.JobID) ([]*log.Log, error) {
	return m.logs, m.err
}

func (m *mockLogGateway) GetUserFacingLogs(ctx context.Context, since time.Time, until time.Time, jobID id.JobID) ([]*userfacinglog.UserFacingLog, error) {
	return []*userfacinglog.UserFacingLog{}, nil
}

func (m *mockLogGateway) GetJobCompleteEvent(ctx context.Context, jobID id.JobID) (*gateway.JobCompleteEvent, error) {
	return nil, nil
}

func (m *mockLogGateway) DeleteJobCompleteEvent(ctx context.Context, jobID id.JobID) error {
	return nil
}

type mockJobRepo struct {
	job *job.Job
	err error
}

func (m *mockJobRepo) FindByID(ctx context.Context, jobID id.JobID) (*job.Job, error) {
	return m.job, m.err
}

func (m *mockJobRepo) FindByIDs(ctx context.Context, jobIDs id.JobIDList) ([]*job.Job, error) {
	panic("unimplemented")
}

func (m *mockJobRepo) FindByWorkspace(ctx context.Context, workspaceID id.WorkspaceID, p *interfaces.PaginationParam, keyword *string) ([]*job.Job, *interfaces.PageBasedInfo, error) {
	panic("unimplemented")
}

func (m *mockJobRepo) Save(ctx context.Context, job *job.Job) error {
	panic("unimplemented")
}

func (m *mockJobRepo) Filtered(filter repo.WorkspaceFilter) repo.Job {
	return m
}

func (m *mockJobRepo) Remove(ctx context.Context, jobID id.JobID) error {
	panic("unimplemented")
}

func TestNewLogInteractor(t *testing.T) {
	t.Run("successfully create LogInteractor", func(t *testing.T) {
		redisMock := &mockLogGateway{}
		jobRepoMock := &mockJobRepo{}
		mockPermissionCheckerTrue := NewMockPermissionChecker(func(ctx context.Context, authInfo *appx.AuthInfo, userId, resource, action string) (bool, error) {
			return true, nil
		})
		li := NewLogInteractor(redisMock, jobRepoMock, mockPermissionCheckerTrue)
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
	jobRepoMock := &mockJobRepo{}
	mockPermissionCheckerTrue := NewMockPermissionChecker(func(ctx context.Context, authInfo *appx.AuthInfo, userId, resource, action string) (bool, error) {
		return true, nil
	})

	t.Run("get Redis logs", func(t *testing.T) {
		li := NewLogInteractor(redisMock, jobRepoMock, mockPermissionCheckerTrue)

		since := time.Now().Add(-30 * time.Minute)
		out, err := li.GetLogs(ctx, since, id.NewJobID())
		assert.NoError(t, err)
		assert.Equal(t, redisLogs, out)
	})

	t.Run("redis error", func(t *testing.T) {
		brokenRedis := &mockLogGateway{err: errors.New("redis error")}
		li := NewLogInteractor(brokenRedis, jobRepoMock, mockPermissionCheckerTrue)

		since := time.Now()
		out, err := li.GetLogs(ctx, since, id.NewJobID())
		assert.Nil(t, out)
		assert.Error(t, err)
		assert.Contains(t, err.Error(), "failed to get logs from Redis")
	})

	t.Run("redis gateway is nil", func(t *testing.T) {
		li := NewLogInteractor(nil, jobRepoMock, mockPermissionCheckerTrue)
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
	jobRepoMock := &mockJobRepo{}
	mockPermissionCheckerTrue := NewMockPermissionChecker(func(ctx context.Context, authInfo *appx.AuthInfo, userId, resource, action string) (bool, error) {
		return true, nil
	})
	li := NewLogInteractor(redisMock, jobRepoMock, mockPermissionCheckerTrue)

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
	jobRepoMock := &mockJobRepo{}
	mockPermissionCheckerTrue := NewMockPermissionChecker(func(ctx context.Context, authInfo *appx.AuthInfo, userId, resource, action string) (bool, error) {
		return true, nil
	})
	liInterface := NewLogInteractor(redisMock, jobRepoMock, mockPermissionCheckerTrue)
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

func TestLogInteractor_StopsMonitoringWhenJobCompleted(t *testing.T) {
	completedJob, err := job.New().
		NewID().
		Deployment(id.NewDeploymentID()).
		Workspace(id.WorkspaceID(id.NewWorkspaceID())).
		Status(job.StatusCompleted).
		StartedAt(time.Now()).
		Build()
	if err != nil {
		t.Fatal(err)
	}

	redisMock := &mockLogGateway{
		logs: []*log.Log{
			log.NewLog(completedJob.ID(), nil, time.Now(), log.LevelInfo, "test log"),
		},
	}

	jobRepoMock := &mockJobRepo{
		job: completedJob,
	}

	mockPermissionCheckerTrue := NewMockPermissionChecker(func(ctx context.Context, authInfo *appx.AuthInfo, userId, resource, action string) (bool, error) {
		return true, nil
	})

	li := NewLogInteractor(redisMock, jobRepoMock, mockPermissionCheckerTrue)

	mockAuthInfo := &appx.AuthInfo{
		Token: "token",
	}
	mockUser := user.New().NewID().Name("test").Email("test@example.com").MustBuild()

	ctx := context.Background()
	ctx = adapter.AttachAuthInfo(ctx, mockAuthInfo)
	ctx = adapter.AttachUser(ctx, mockUser)

	t.Run("monitoring stops when job is completed", func(t *testing.T) {
		ctx, cancel := context.WithTimeout(ctx, 2*time.Second)
		defer cancel()

		done := make(chan bool)
		go func() {
			li.(*LogInteractor).runLogMonitoringLoop(ctx, completedJob.ID(), time.Now().Add(-1*time.Hour))
			done <- true
		}()

		select {
		case <-done:
		case <-time.After(5 * time.Second):
			t.Fatal("Log monitoring loop did not stop when job was completed")
		}
	})
}
