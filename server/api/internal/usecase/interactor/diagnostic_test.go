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
	"github.com/stretchr/testify/require"
)

// mockDiagnosticsRedis is a dedicated gateway.Redis fake for these tests,
// since the shared mockLogGateway/mockUserFacingLogGateway always return
// nil for GetNodeDiagnostics/GetJobDiagnostics.
type mockDiagnosticsRedis struct {
	nodeDiagnosticsErr error
	jobDiagnosticsErr  error
	nodeDiagnostics    []*diagnostic.Diagnostic
	jobDiagnostics     []*diagnostic.Diagnostic
}

func (m *mockDiagnosticsRedis) GetLogs(ctx context.Context, since, until time.Time, jobID id.JobID) ([]*log.Log, error) {
	return nil, nil
}

func (m *mockDiagnosticsRedis) GetUserFacingLogs(ctx context.Context, since, until time.Time, jobID id.JobID) ([]*userfacinglog.UserFacingLog, error) {
	return nil, nil
}

func (m *mockDiagnosticsRedis) GetNodeExecutions(ctx context.Context, jobID id.JobID) ([]*graph.NodeExecution, error) {
	return nil, nil
}

func (m *mockDiagnosticsRedis) GetNodeExecution(ctx context.Context, jobID id.JobID, nodeID string) (*graph.NodeExecution, error) {
	return nil, nil
}

func (m *mockDiagnosticsRedis) GetJobCompleteEvent(ctx context.Context, jobID id.JobID) (*gateway.JobCompleteEvent, error) {
	return nil, nil
}

func (m *mockDiagnosticsRedis) DeleteJobCompleteEvent(ctx context.Context, jobID id.JobID) error {
	return nil
}

func (m *mockDiagnosticsRedis) GetNodeDiagnostics(ctx context.Context, jobID id.JobID, nodeID string) ([]*diagnostic.Diagnostic, error) {
	return m.nodeDiagnostics, m.nodeDiagnosticsErr
}

func (m *mockDiagnosticsRedis) GetJobDiagnostics(ctx context.Context, jobID id.JobID) ([]*diagnostic.Diagnostic, error) {
	return m.jobDiagnostics, m.jobDiagnosticsErr
}

// mockDiagnosticsRepo is a dedicated repo.NodeDiagnostics fake. It is also
// reused (not redefined) by job_test.go's checkJobStatus merge tests, which
// additionally inspect the lastX fields SaveTerminalDiagnostics records to
// assert on exactly what was persisted.
type mockDiagnosticsRepo struct {
	lastTimestamp   time.Time
	byNodeErr       error
	byJobErr        error
	saveErr         error
	summaryErr      error
	summary         *uint64
	lastDropped     *uint64
	byNode          []*diagnostic.Diagnostic
	lastFailedNodes []*diagnostic.Diagnostic
	lastAggregated  []*diagnostic.Diagnostic
	byJob           []*diagnostic.Diagnostic
	saveCalls       int
	lastJobID       id.JobID
}

func (m *mockDiagnosticsRepo) FindByJobNodeID(ctx context.Context, jobID id.JobID, nodeID string) ([]*diagnostic.Diagnostic, error) {
	return m.byNode, m.byNodeErr
}

func (m *mockDiagnosticsRepo) FindByJobID(ctx context.Context, jobID id.JobID) ([]*diagnostic.Diagnostic, error) {
	return m.byJob, m.byJobErr
}

func (m *mockDiagnosticsRepo) FindJobSummary(ctx context.Context, jobID id.JobID) (*uint64, error) {
	return m.summary, m.summaryErr
}

func (m *mockDiagnosticsRepo) SaveTerminalDiagnostics(
	ctx context.Context,
	jobID id.JobID,
	timestamp time.Time,
	failedNodes []*diagnostic.Diagnostic,
	aggregated []*diagnostic.Diagnostic,
	droppedEventCount *uint64,
) error {
	m.saveCalls++
	m.lastJobID = jobID
	m.lastTimestamp = timestamp
	m.lastFailedNodes = failedNodes
	m.lastAggregated = aggregated
	m.lastDropped = droppedEventCount
	return m.saveErr
}

func newTestDiagnostic(t *testing.T, jobID id.JobID, code string) *diagnostic.Diagnostic {
	t.Helper()
	d, err := diagnostic.NewBuilder().
		JobID(jobID).
		Timestamp(time.Now()).
		Code(code).
		Category("internal").
		Severity("warn").
		Message("test diagnostic").
		Build()
	require.NoError(t, err)
	return d
}

func alwaysAllowPermissionChecker() *mockPermissionChecker {
	return NewMockPermissionChecker(func(ctx context.Context, resource, action string) (bool, error) {
		return true, nil
	})
}

func TestNodeDiagnostics_GetNodeDiagnostics(t *testing.T) {
	ctx := context.Background()
	jobID := id.NewJobID()

	t.Run("Redis has rows: Mongo is never consulted", func(t *testing.T) {
		redisRows := []*diagnostic.Diagnostic{newTestDiagnostic(t, jobID, "redis.code")}
		redisMock := &mockDiagnosticsRedis{nodeDiagnostics: redisRows}
		repoMock := &mockDiagnosticsRepo{byNode: []*diagnostic.Diagnostic{newTestDiagnostic(t, jobID, "mongo.code")}}
		jobRepo := &mockJobRepo{}

		i := NewNodeDiagnostics(repoMock, jobRepo, redisMock, alwaysAllowPermissionChecker())
		got, err := i.GetNodeDiagnostics(ctx, jobID, "node-1")
		assert.NoError(t, err)
		require.Len(t, got, 1)
		assert.Equal(t, "redis.code", got[0].Code())
	})

	t.Run("Redis empty: falls back to Mongo", func(t *testing.T) {
		redisMock := &mockDiagnosticsRedis{}
		mongoRows := []*diagnostic.Diagnostic{newTestDiagnostic(t, jobID, "mongo.code")}
		repoMock := &mockDiagnosticsRepo{byNode: mongoRows}
		jobRepo := &mockJobRepo{}

		i := NewNodeDiagnostics(repoMock, jobRepo, redisMock, alwaysAllowPermissionChecker())
		got, err := i.GetNodeDiagnostics(ctx, jobID, "node-1")
		assert.NoError(t, err)
		require.Len(t, got, 1)
		assert.Equal(t, "mongo.code", got[0].Code())
	})

	t.Run("Redis errors: falls back to Mongo instead of failing", func(t *testing.T) {
		redisMock := &mockDiagnosticsRedis{nodeDiagnosticsErr: errors.New("redis down")}
		mongoRows := []*diagnostic.Diagnostic{newTestDiagnostic(t, jobID, "mongo.code")}
		repoMock := &mockDiagnosticsRepo{byNode: mongoRows}
		jobRepo := &mockJobRepo{}

		i := NewNodeDiagnostics(repoMock, jobRepo, redisMock, alwaysAllowPermissionChecker())
		got, err := i.GetNodeDiagnostics(ctx, jobID, "node-1")
		assert.NoError(t, err)
		require.Len(t, got, 1)
		assert.Equal(t, "mongo.code", got[0].Code())
	})

	t.Run("both empty: empty, not error", func(t *testing.T) {
		redisMock := &mockDiagnosticsRedis{}
		repoMock := &mockDiagnosticsRepo{}
		jobRepo := &mockJobRepo{}

		i := NewNodeDiagnostics(repoMock, jobRepo, redisMock, alwaysAllowPermissionChecker())
		got, err := i.GetNodeDiagnostics(ctx, jobID, "node-1")
		assert.NoError(t, err)
		assert.Empty(t, got)
	})

	t.Run("permission denied", func(t *testing.T) {
		redisMock := &mockDiagnosticsRedis{}
		repoMock := &mockDiagnosticsRepo{}
		jobRepo := &mockJobRepo{}
		denyChecker := NewMockPermissionChecker(func(ctx context.Context, resource, action string) (bool, error) {
			return false, nil
		})

		i := NewNodeDiagnostics(repoMock, jobRepo, redisMock, denyChecker)
		got, err := i.GetNodeDiagnostics(ctx, jobID, "node-1")
		assert.Error(t, err)
		assert.Nil(t, got)
	})
}

func TestNodeDiagnostics_GetJobDiagnostics(t *testing.T) {
	ctx := context.Background()
	jobID := id.NewJobID()

	t.Run("Redis has rows: Mongo is never consulted", func(t *testing.T) {
		redisRows := []*diagnostic.Diagnostic{newTestDiagnostic(t, jobID, "redis.code")}
		redisMock := &mockDiagnosticsRedis{jobDiagnostics: redisRows}
		repoMock := &mockDiagnosticsRepo{byJob: []*diagnostic.Diagnostic{newTestDiagnostic(t, jobID, "mongo.code")}}
		jobRepo := &mockJobRepo{}

		i := NewNodeDiagnostics(repoMock, jobRepo, redisMock, alwaysAllowPermissionChecker())
		got, err := i.GetJobDiagnostics(ctx, jobID)
		assert.NoError(t, err)
		require.Len(t, got, 1)
		assert.Equal(t, "redis.code", got[0].Code())
	})

	t.Run("Redis empty: falls back to Mongo", func(t *testing.T) {
		redisMock := &mockDiagnosticsRedis{}
		mongoRows := []*diagnostic.Diagnostic{newTestDiagnostic(t, jobID, "mongo.code")}
		repoMock := &mockDiagnosticsRepo{byJob: mongoRows}
		jobRepo := &mockJobRepo{}

		i := NewNodeDiagnostics(repoMock, jobRepo, redisMock, alwaysAllowPermissionChecker())
		got, err := i.GetJobDiagnostics(ctx, jobID)
		assert.NoError(t, err)
		require.Len(t, got, 1)
		assert.Equal(t, "mongo.code", got[0].Code())
	})

	t.Run("Mongo repo is nil: Redis result still returned", func(t *testing.T) {
		redisRows := []*diagnostic.Diagnostic{newTestDiagnostic(t, jobID, "redis.code")}
		redisMock := &mockDiagnosticsRedis{jobDiagnostics: redisRows}
		jobRepo := &mockJobRepo{}

		i := NewNodeDiagnostics(nil, jobRepo, redisMock, alwaysAllowPermissionChecker())
		got, err := i.GetJobDiagnostics(ctx, jobID)
		assert.NoError(t, err)
		require.Len(t, got, 1)
	})

	t.Run("both nil gateways: empty, not error", func(t *testing.T) {
		jobRepo := &mockJobRepo{}
		i := NewNodeDiagnostics(nil, jobRepo, nil, alwaysAllowPermissionChecker())
		got, err := i.GetJobDiagnostics(ctx, jobID)
		assert.NoError(t, err)
		assert.Empty(t, got)
	})

	t.Run("job repo lookup fails", func(t *testing.T) {
		redisMock := &mockDiagnosticsRedis{}
		repoMock := &mockDiagnosticsRepo{}
		jobRepo := &mockJobRepo{err: errors.New("job lookup failed")}

		i := NewNodeDiagnostics(repoMock, jobRepo, redisMock, alwaysAllowPermissionChecker())
		got, err := i.GetJobDiagnostics(ctx, jobID)
		assert.Error(t, err)
		assert.Nil(t, got)
	})
}

// fatalDiagnostic/nonFatalDiagnostic build test rows mirroring the two wire
// arrays FindByJobID mixes together in Mongo: a failedNodes-derived row
// (effectiveDisposition="fatal") and an aggregatedDiagnostics-derived row
// (no fatal effectiveDisposition — see fatalEffectiveDisposition's doc
// comment on the engine guarantee this rests on).
func fatalDiagnostic(t *testing.T, jobID id.JobID, code string) *diagnostic.Diagnostic {
	t.Helper()
	fatal := fatalEffectiveDisposition
	d, err := diagnostic.NewBuilder().
		JobID(jobID).
		Timestamp(time.Now()).
		Code(code).
		Category("internal").
		Severity("fatal").
		EffectiveDisposition(&fatal).
		Message("fatal test diagnostic").
		Build()
	require.NoError(t, err)
	return d
}

func TestNodeDiagnostics_GetFailedNodes(t *testing.T) {
	ctx := context.Background()
	jobID := id.NewJobID()

	t.Run("filters to fatal effectiveDisposition rows only, excluding non-fatal aggregated rows", func(t *testing.T) {
		fatalRow := fatalDiagnostic(t, jobID, "internal.invariant_violation")
		nonFatalRow := newTestDiagnostic(t, jobID, "gltf.zero_face_solid") // no effectiveDisposition set
		repoMock := &mockDiagnosticsRepo{byJob: []*diagnostic.Diagnostic{fatalRow, nonFatalRow}}
		jobRepo := &mockJobRepo{}

		i := NewNodeDiagnostics(repoMock, jobRepo, nil, alwaysAllowPermissionChecker())
		got, err := i.GetFailedNodes(ctx, jobID)
		assert.NoError(t, err)
		require.Len(t, got, 1)
		assert.Equal(t, "internal.invariant_violation", got[0].Code())
	})

	t.Run("no rows: empty, not error", func(t *testing.T) {
		repoMock := &mockDiagnosticsRepo{}
		jobRepo := &mockJobRepo{}

		i := NewNodeDiagnostics(repoMock, jobRepo, nil, alwaysAllowPermissionChecker())
		got, err := i.GetFailedNodes(ctx, jobID)
		assert.NoError(t, err)
		assert.Empty(t, got)
	})

	t.Run("nil repo: empty, not error", func(t *testing.T) {
		jobRepo := &mockJobRepo{}

		i := NewNodeDiagnostics(nil, jobRepo, nil, alwaysAllowPermissionChecker())
		got, err := i.GetFailedNodes(ctx, jobID)
		assert.NoError(t, err)
		assert.Empty(t, got)
	})

	t.Run("repo error propagates", func(t *testing.T) {
		repoMock := &mockDiagnosticsRepo{byJobErr: errors.New("mongo down")}
		jobRepo := &mockJobRepo{}

		i := NewNodeDiagnostics(repoMock, jobRepo, nil, alwaysAllowPermissionChecker())
		got, err := i.GetFailedNodes(ctx, jobID)
		assert.Error(t, err)
		assert.Nil(t, got)
	})

	t.Run("permission denied", func(t *testing.T) {
		repoMock := &mockDiagnosticsRepo{}
		jobRepo := &mockJobRepo{}
		denyChecker := NewMockPermissionChecker(func(ctx context.Context, resource, action string) (bool, error) {
			return false, nil
		})

		i := NewNodeDiagnostics(repoMock, jobRepo, nil, denyChecker)
		got, err := i.GetFailedNodes(ctx, jobID)
		assert.Error(t, err)
		assert.Nil(t, got)
	})
}

func TestNodeDiagnostics_GetDroppedEventCount(t *testing.T) {
	ctx := context.Background()
	jobID := id.NewJobID()

	t.Run("returns the persisted count", func(t *testing.T) {
		dropped := uint64(4)
		repoMock := &mockDiagnosticsRepo{summary: &dropped}
		jobRepo := &mockJobRepo{}

		i := NewNodeDiagnostics(repoMock, jobRepo, nil, alwaysAllowPermissionChecker())
		got, err := i.GetDroppedEventCount(ctx, jobID)
		assert.NoError(t, err)
		require.NotNil(t, got)
		assert.Equal(t, uint64(4), *got)
	})

	t.Run("no summary row: nil, not error", func(t *testing.T) {
		repoMock := &mockDiagnosticsRepo{}
		jobRepo := &mockJobRepo{}

		i := NewNodeDiagnostics(repoMock, jobRepo, nil, alwaysAllowPermissionChecker())
		got, err := i.GetDroppedEventCount(ctx, jobID)
		assert.NoError(t, err)
		assert.Nil(t, got)
	})

	t.Run("nil repo: nil, not error", func(t *testing.T) {
		jobRepo := &mockJobRepo{}

		i := NewNodeDiagnostics(nil, jobRepo, nil, alwaysAllowPermissionChecker())
		got, err := i.GetDroppedEventCount(ctx, jobID)
		assert.NoError(t, err)
		assert.Nil(t, got)
	})

	t.Run("repo error propagates", func(t *testing.T) {
		repoMock := &mockDiagnosticsRepo{summaryErr: errors.New("mongo down")}
		jobRepo := &mockJobRepo{}

		i := NewNodeDiagnostics(repoMock, jobRepo, nil, alwaysAllowPermissionChecker())
		got, err := i.GetDroppedEventCount(ctx, jobID)
		assert.Error(t, err)
		assert.Nil(t, got)
	})

	t.Run("permission denied", func(t *testing.T) {
		repoMock := &mockDiagnosticsRepo{}
		jobRepo := &mockJobRepo{}
		denyChecker := NewMockPermissionChecker(func(ctx context.Context, resource, action string) (bool, error) {
			return false, nil
		})

		i := NewNodeDiagnostics(repoMock, jobRepo, nil, denyChecker)
		got, err := i.GetDroppedEventCount(ctx, jobID)
		assert.Error(t, err)
		assert.Nil(t, got)
	})
}
