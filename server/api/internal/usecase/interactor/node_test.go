package interactor

import (
	"context"
	"errors"
	"testing"
	"time"

	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/pkg/diagnostic"
	"github.com/reearth/reearth-flow/api/pkg/graph"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/log"
	"github.com/reearth/reearth-flow/api/pkg/userfacinglog"
	"github.com/stretchr/testify/assert"
)

// mockNodeExecutionsRedis is a dedicated gateway.Redis fake for
// TestNodeExecution_GetNodeExecutions: the shared mocks in this package
// (mockDiagnosticsRedis, mockCheckStatusRedis) always return nil for
// GetNodeExecutions, so this one lets the result and error be configured.
type mockNodeExecutionsRedis struct {
	err            error
	nodeExecutions []*graph.NodeExecution
}

func (m *mockNodeExecutionsRedis) GetLogs(ctx context.Context, since, until time.Time, jobID id.JobID) ([]*log.Log, error) {
	return nil, nil
}

func (m *mockNodeExecutionsRedis) GetUserFacingLogs(ctx context.Context, since, until time.Time, jobID id.JobID) ([]*userfacinglog.UserFacingLog, error) {
	return nil, nil
}

func (m *mockNodeExecutionsRedis) GetNodeExecutions(ctx context.Context, jobID id.JobID) ([]*graph.NodeExecution, error) {
	return m.nodeExecutions, m.err
}

func (m *mockNodeExecutionsRedis) GetNodeExecution(ctx context.Context, jobID id.JobID, nodeID string) (*graph.NodeExecution, error) {
	return nil, nil
}

func (m *mockNodeExecutionsRedis) GetJobCompleteEvent(ctx context.Context, jobID id.JobID) (*gateway.JobCompleteEvent, error) {
	return nil, nil
}

func (m *mockNodeExecutionsRedis) DeleteJobCompleteEvent(ctx context.Context, jobID id.JobID) error {
	return nil
}

func (m *mockNodeExecutionsRedis) GetNodeDiagnostics(ctx context.Context, jobID id.JobID, nodeID string) ([]*diagnostic.Diagnostic, error) {
	return nil, nil
}

func (m *mockNodeExecutionsRedis) GetJobDiagnostics(ctx context.Context, jobID id.JobID) ([]*diagnostic.Diagnostic, error) {
	return nil, nil
}

func TestNodeExecution_GetNodeExecutions(t *testing.T) {
	ctx := context.Background()
	jobID := id.NewJobID()
	nodeID := id.NewNodeID()

	t.Run("returns node executions from redis", func(t *testing.T) {
		want := []*graph.NodeExecution{
			graph.NewNodeExecution("n1", jobID, nodeID, graph.StatusCompleted),
		}
		redisMock := &mockNodeExecutionsRedis{nodeExecutions: want}
		jobRepo := &mockJobRepo{}

		i := NewNodeExecution(nil, jobRepo, redisMock, alwaysAllowPermissionChecker())
		got, err := i.GetNodeExecutions(ctx, jobID)
		assert.NoError(t, err)
		assert.Equal(t, want, got)
	})

	t.Run("redis error is propagated", func(t *testing.T) {
		redisMock := &mockNodeExecutionsRedis{err: errors.New("redis down")}
		jobRepo := &mockJobRepo{}

		i := NewNodeExecution(nil, jobRepo, redisMock, alwaysAllowPermissionChecker())
		got, err := i.GetNodeExecutions(ctx, jobID)
		assert.Error(t, err)
		assert.Nil(t, got)
	})

	t.Run("permission denied", func(t *testing.T) {
		redisMock := &mockNodeExecutionsRedis{}
		jobRepo := &mockJobRepo{}
		denyChecker := NewMockPermissionChecker(func(ctx context.Context, resource, action string) (bool, error) {
			return false, nil
		})

		i := NewNodeExecution(nil, jobRepo, redisMock, denyChecker)
		got, err := i.GetNodeExecutions(ctx, jobID)
		assert.Error(t, err)
		assert.Nil(t, got)
	})
}
