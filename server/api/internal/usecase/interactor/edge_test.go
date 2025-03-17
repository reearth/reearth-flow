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

type mockRedisGateway struct {
	edgeExecution  *edge.EdgeExecution
	edgeExecutions []*edge.EdgeExecution
	err            error
}

// GetLogs implements gateway.Redis.
func (m *mockRedisGateway) GetLogs(ctx context.Context, since time.Time, until time.Time, jobID id.JobID) ([]*log.Log, error) {
	panic("unimplemented")
}

func (m *mockRedisGateway) GetEdgeExecution(ctx context.Context, jobID id.JobID, edgeID string) (*edge.EdgeExecution, error) {
	return m.edgeExecution, m.err
}

func (m *mockRedisGateway) GetEdgeExecutions(ctx context.Context, jobID id.JobID) ([]*edge.EdgeExecution, error) {
	return m.edgeExecutions, m.err
}

// Mock interface for PermissionChecker
type MockPermissionChecker struct {
	result bool
	err    error
}

func (m *MockPermissionChecker) CheckPermission(ctx context.Context, authInfo *appx.AuthInfo, userId, resource, action string) (bool, error) {
	return m.result, m.err
}

func TestEdgeExecution_GetEdgeExecutions(t *testing.T) {
	jobID := id.NewJobID()
	now := time.Now()
	edges := []*edge.EdgeExecution{
		edge.NewEdgeExecution("random1", "edge1", jobID, "workflow1", edge.StatusInProgress, &now, nil, nil, nil),
		edge.NewEdgeExecution("random2", "edge2", jobID, "workflow1", edge.StatusCompleted, &now, &now, nil, nil),
	}

	// Create auth context
	mockAuthInfo := &appx.AuthInfo{
		Token: "token",
	}
	mockUser := user.New().NewID().Name("test").Email("test@example.com").MustBuild()
	ctx := context.Background()
	ctx = adapter.AttachAuthInfo(ctx, mockAuthInfo)
	ctx = adapter.AttachUser(ctx, mockUser)

	t.Run("redis error", func(t *testing.T) {
		redis := &mockRedisGateway{
			err: errors.New("redis error"),
		}
		mockPermissionChecker := &MockPermissionChecker{result: true}

		ei := &EdgeExecution{
			redisGateway:      redis,
			permissionChecker: mockPermissionChecker,
		}

		result, err := ei.GetEdgeExecutions(ctx, jobID)
		assert.Error(t, err)
		assert.Nil(t, result)
		assert.Contains(t, err.Error(), "failed to get edge executions from Redis")
	})

	t.Run("get edge executions success", func(t *testing.T) {
		redis := &mockRedisGateway{
			edgeExecutions: edges,
		}
		mockPermissionChecker := &MockPermissionChecker{result: true}

		ei := &EdgeExecution{
			redisGateway:      redis,
			permissionChecker: mockPermissionChecker,
		}

		result, err := ei.GetEdgeExecutions(ctx, jobID)
		assert.NoError(t, err)
		assert.Equal(t, edges, result)
	})
}

func TestEdgeExecution_GetEdgeExecution(t *testing.T) {
	jobID := id.NewJobID()
	edgeID := "edge1"
	now := time.Now()
	edgeExec := edge.NewEdgeExecution("random", edgeID, jobID, "workflow1", edge.StatusInProgress, &now, nil, nil, nil)

	// Create auth context
	mockAuthInfo := &appx.AuthInfo{
		Token: "token",
	}
	mockUser := user.New().NewID().Name("test").Email("test@example.com").MustBuild()
	ctx := context.Background()
	ctx = adapter.AttachAuthInfo(ctx, mockAuthInfo)
	ctx = adapter.AttachUser(ctx, mockUser)

	t.Run("redis gateway is nil", func(t *testing.T) {
		mockPermissionChecker := &MockPermissionChecker{result: true}

		ei := &EdgeExecution{
			redisGateway:      nil,
			permissionChecker: mockPermissionChecker,
		}

		result, err := ei.GetEdgeExecution(ctx, jobID, edgeID)
		assert.Error(t, err)
		assert.Nil(t, result)
		assert.Contains(t, err.Error(), "redisGateway is nil")
	})

	t.Run("get edge execution success", func(t *testing.T) {
		redis := &mockRedisGateway{
			edgeExecution: edgeExec,
		}
		mockPermissionChecker := &MockPermissionChecker{result: true}

		ei := &EdgeExecution{
			redisGateway:      redis,
			permissionChecker: mockPermissionChecker,
		}

		result, err := ei.GetEdgeExecution(ctx, jobID, edgeID)
		assert.NoError(t, err)
		assert.Equal(t, edgeExec, result)
	})
}
