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

// Also reused by job_test.go's checkJobStatus merge tests; keep lastX fields in sync.
type mockDiagnosticsRepo struct {
	lastTimestamp   time.Time
	byNodeErr       error
	byJobErr        error
	saveErr         error
	summaryErr      error
	lastDropped     *uint64
	summary         *uint64
	lastWorkflowID  string
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
	workflowID string,
	timestamp time.Time,
	failedNodes []*diagnostic.Diagnostic,
	aggregated []*diagnostic.Diagnostic,
	droppedEventCount *uint64,
) error {
	m.saveCalls++
	m.lastJobID = jobID
	m.lastWorkflowID = workflowID
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

	t.Run("Redis and Mongo rows are merged, not short-circuited", func(t *testing.T) {
		// A Mongo-only terminal row must not be hidden until Redis's TTL expires.
		redisRows := []*diagnostic.Diagnostic{newTestDiagnostic(t, jobID, "redis.code")}
		redisMock := &mockDiagnosticsRedis{nodeDiagnostics: redisRows}
		repoMock := &mockDiagnosticsRepo{byNode: []*diagnostic.Diagnostic{newTestDiagnostic(t, jobID, "mongo.code")}}
		jobRepo := &mockJobRepo{}

		i := NewNodeDiagnostics(repoMock, jobRepo, redisMock, alwaysAllowPermissionChecker())
		got, err := i.GetNodeDiagnostics(ctx, jobID, "node-1")
		assert.NoError(t, err)
		require.Len(t, got, 2)
		codes := []string{got[0].Code(), got[1].Code()}
		assert.ElementsMatch(t, []string{"redis.code", "mongo.code"}, codes)
	})

	t.Run("an aggregated summary that rode both the live and terminal paths dedupes to its terminal copy", func(t *testing.T) {
		// A summary published live is folded into aggregated_diagnostics again at completion; must dedupe to one row.
		nodeID := "node-1"
		warnDrop := "warn_drop"
		olderTimestamp := time.Now().Add(-time.Hour)
		newerTimestamp := time.Now()

		liveRow, err := diagnostic.NewBuilder().
			JobID(jobID).
			NodeID(&nodeID).
			Timestamp(olderTimestamp).
			Code("gltf.zero_face_solid").
			Category("gltf").
			Severity("warn").
			EffectiveDisposition(&warnDrop).
			Message("live copy").
			Build()
		require.NoError(t, err)

		terminalRow, err := diagnostic.NewBuilder().
			JobID(jobID).
			NodeID(&nodeID).
			Timestamp(newerTimestamp).
			Code("gltf.zero_face_solid").
			Category("gltf").
			Severity("warn").
			EffectiveDisposition(&warnDrop).
			Terminal(true).
			Message("terminal copy").
			Build()
		require.NoError(t, err)

		redisMock := &mockDiagnosticsRedis{nodeDiagnostics: []*diagnostic.Diagnostic{liveRow}}
		repoMock := &mockDiagnosticsRepo{byNode: []*diagnostic.Diagnostic{terminalRow}}
		jobRepo := &mockJobRepo{}

		i := NewNodeDiagnostics(repoMock, jobRepo, redisMock, alwaysAllowPermissionChecker())
		got, err := i.GetNodeDiagnostics(ctx, jobID, nodeID)
		assert.NoError(t, err)
		require.Len(t, got, 1)
		assert.True(t, got[0].Terminal())
		assert.Equal(t, "terminal copy", got[0].Message())
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

	t.Run("Redis and Mongo rows are merged, not short-circuited", func(t *testing.T) {
		redisRows := []*diagnostic.Diagnostic{newTestDiagnostic(t, jobID, "redis.code")}
		redisMock := &mockDiagnosticsRedis{jobDiagnostics: redisRows}
		repoMock := &mockDiagnosticsRepo{byJob: []*diagnostic.Diagnostic{newTestDiagnostic(t, jobID, "mongo.code")}}
		jobRepo := &mockJobRepo{}

		i := NewNodeDiagnostics(repoMock, jobRepo, redisMock, alwaysAllowPermissionChecker())
		got, err := i.GetJobDiagnostics(ctx, jobID)
		assert.NoError(t, err)
		require.Len(t, got, 2)
		codes := []string{got[0].Code(), got[1].Code()}
		assert.ElementsMatch(t, []string{"redis.code", "mongo.code"}, codes)
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
		nonFatalRow := newTestDiagnostic(t, jobID, "gltf.zero_face_solid")
		repoMock := &mockDiagnosticsRepo{byJob: []*diagnostic.Diagnostic{fatalRow, nonFatalRow}}
		jobRepo := &mockJobRepo{}

		i := NewNodeDiagnostics(repoMock, jobRepo, nil, alwaysAllowPermissionChecker())
		got, err := i.GetFailedNodes(ctx, jobID)
		assert.NoError(t, err)
		require.Len(t, got, 1)
		assert.Equal(t, "internal.invariant_violation", got[0].Code())
	})

	t.Run("two fatal rows sharing a key still dedupe to one, preferring terminal", func(t *testing.T) {
		// Unreachable in production (fatals never publish live); pins that the dedupe backstop still resolves same-key collisions deterministically.
		nodeID := "node-1"
		fatal := fatalEffectiveDisposition

		liveRow, err := diagnostic.NewBuilder().
			JobID(jobID).
			NodeID(&nodeID).
			Timestamp(time.Now()).
			Code("internal.invariant_violation").
			Category("internal").
			Severity("fatal").
			EffectiveDisposition(&fatal).
			Message("live copy").
			Build()
		require.NoError(t, err)

		terminalRow, err := diagnostic.NewBuilder().
			JobID(jobID).
			NodeID(&nodeID).
			Timestamp(time.Now()).
			Code("internal.invariant_violation").
			Category("internal").
			Severity("fatal").
			EffectiveDisposition(&fatal).
			Terminal(true).
			Message("terminal copy").
			Build()
		require.NoError(t, err)

		repoMock := &mockDiagnosticsRepo{byJob: []*diagnostic.Diagnostic{liveRow, terminalRow}}
		jobRepo := &mockJobRepo{}

		i := NewNodeDiagnostics(repoMock, jobRepo, nil, alwaysAllowPermissionChecker())
		got, err := i.GetFailedNodes(ctx, jobID)
		assert.NoError(t, err)
		require.Len(t, got, 1)
		assert.True(t, got[0].Terminal())
	})

	t.Run("no rows: empty, not error", func(t *testing.T) {
		repoMock := &mockDiagnosticsRepo{}
		jobRepo := &mockJobRepo{}

		i := NewNodeDiagnostics(repoMock, jobRepo, nil, alwaysAllowPermissionChecker())
		got, err := i.GetFailedNodes(ctx, jobID)
		assert.NoError(t, err)
		assert.Empty(t, got)
	})

	t.Run("nil repo: empty slice, not nil, not error", func(t *testing.T) {
		// GraphQL normalizes no-data to [] (gqlmodel.ToDiagnostics); must return non-nil empty, not nil.
		jobRepo := &mockJobRepo{}

		i := NewNodeDiagnostics(nil, jobRepo, nil, alwaysAllowPermissionChecker())
		got, err := i.GetFailedNodes(ctx, jobID)
		assert.NoError(t, err)
		assert.NotNil(t, got)
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

// Pins a case only reachable via FindByJobID's mix; GetFailedNodes filters it out first.
func TestDedupeDiagnostics(t *testing.T) {
	jobID := id.NewJobID()
	nodeID := "node-1"

	t.Run("same node/code/disposition collapses to the terminal copy", func(t *testing.T) {
		warnDrop := "warn_drop"

		liveRow, err := diagnostic.NewBuilder().
			JobID(jobID).
			NodeID(&nodeID).
			Timestamp(time.Now().Add(-time.Hour)).
			Code("gltf.zero_face_solid").
			Category("gltf").
			Severity("warn").
			EffectiveDisposition(&warnDrop).
			Message("live copy").
			Build()
		require.NoError(t, err)

		terminalRow, err := diagnostic.NewBuilder().
			JobID(jobID).
			NodeID(&nodeID).
			Timestamp(time.Now()).
			Code("gltf.zero_face_solid").
			Category("gltf").
			Severity("warn").
			EffectiveDisposition(&warnDrop).
			Terminal(true).
			Message("terminal copy").
			Build()
		require.NoError(t, err)

		got := dedupeDiagnostics([]*diagnostic.Diagnostic{liveRow, terminalRow})
		require.Len(t, got, 1)
		assert.True(t, got[0].Terminal())
		assert.Equal(t, "terminal copy", got[0].Message())
	})

	t.Run("a failedNodes row and an aggregatedDiagnostics row sharing (nodeId, code) both survive", func(t *testing.T) {
		// Regression guard: the old (nodeId, code)-only key let these nondeterministically collapse into one via preferOver's tie-break.
		fatal := fatalEffectiveDisposition
		warnDrop := "warn_drop"

		failedRow, err := diagnostic.NewBuilder().
			JobID(jobID).
			NodeID(&nodeID).
			Timestamp(time.Now()).
			Code("internal.unclassified").
			Category("internal").
			Severity("fatal").
			EffectiveDisposition(&fatal).
			Terminal(true).
			Message("node failed").
			Build()
		require.NoError(t, err)

		aggregatedRow, err := diagnostic.NewBuilder().
			JobID(jobID).
			NodeID(&nodeID).
			Timestamp(time.Now()).
			Code("internal.unclassified").
			Category("internal").
			Severity("warn").
			EffectiveDisposition(&warnDrop).
			Terminal(true).
			Message("features dropped").
			Build()
		require.NoError(t, err)

		got := dedupeDiagnostics([]*diagnostic.Diagnostic{failedRow, aggregatedRow})
		require.Len(t, got, 2)
		messages := []string{got[0].Message(), got[1].Message()}
		assert.ElementsMatch(t, []string{"node failed", "features dropped"}, messages)
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
