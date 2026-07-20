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
		// Pins Important-3's fix: previously a non-empty Redis result
		// short-circuited Mongo entirely, hiding a Mongo-only terminal row
		// (e.g. a node's terminal fatal, which is never written to Redis —
		// see interactor/job.go's persistTerminalDiagnostics) until the
		// Redis event's 24h TTL expired. Both sources must now always be
		// consulted and merged.
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
		// This is the real duplicate dedupeDiagnostics fixes (see the package
		// doc comment): emit_summaries publishes each finish()-time
		// WarnDrop/Reject/WarnContinue summary live AND folds the same
		// summary into RunSummary.aggregated_diagnostics, persisted again as
		// a terminal row at job completion. Fatal diagnostics never take this
		// path — they're never published live in the first place.
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

	t.Run("two fatal rows sharing a key still dedupe to one, preferring terminal", func(t *testing.T) {
		// Fatal diagnostics are never published live (see the package doc
		// comment), so this exact pairing isn't reachable in production —
		// GetFailedNodes' dedupe here is a defensive backstop, not the
		// live/terminal merge GetNodeDiagnostics/GetJobDiagnostics rely on.
		// This pins that the backstop still resolves a same-key collision
		// deterministically (terminal wins) rather than dropping one at
		// random, should two rows ever land on the same key.
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
		// [] not null: Job.failedNodes and NodeExecution.diagnostics both
		// normalize their no-data state to an empty list (see
		// gqlmodel.ToDiagnostics), so this usecase must return a non-nil
		// empty slice too, not a nil one.
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

// TestDedupeDiagnostics exercises dedupeDiagnostics directly rather than
// through a NodeDiagnostics method, since the case it pins — a failedNodes
// row and an aggregatedDiagnostics row sharing (nodeId, code) — can only
// reach dedupeDiagnostics through FindByJobID's undifferentiated mix, which
// GetFailedNodes deliberately filters away before deduping.
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
		// The bug the widened key fixes: with the old (nodeId, code)-only
		// key, these two structurally different rows — this node/code
		// failed the run fatally, AND the same node/code was separately
		// warn-dropped some number of times — would collapse into one via
		// preferOver's Terminal+Timestamp tie-break, nondeterministically
		// dropping whichever lost. Both are real, distinct information and
		// must both survive.
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
