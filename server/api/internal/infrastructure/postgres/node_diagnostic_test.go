package postgres_test

import (
	"context"
	"testing"
	"time"

	"github.com/reearth/reearth-flow/api/internal/infrastructure/postgres"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/postgres/pgtest"
	"github.com/reearth/reearth-flow/api/pkg/diagnostic"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearthx/pgxx"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

const diagnosticTestWorkflowID = "11111111-1111-1111-1111-111111111111"

func TestNodeDiagnostics_SaveTerminalAndFind(t *testing.T) {
	t.Parallel()

	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	r := postgres.NewNodeDiagnostics(pgxx.NewClient(pool))

	jobID := id.NewJobID()
	now := time.Now().UTC().Truncate(time.Microsecond)
	nodeID := "subgraph-a.sink-writer-2"
	fatal := "fatal"

	jobLevel := diagnostic.NewBuilder().
		JobID(jobID).
		Timestamp(now).
		Code("internal.invariant_violation").
		Category("internal").
		Severity("fatal").
		EffectiveDisposition(&fatal).
		Message("invariant violation").
		MustBuild()
	failedNode := diagnostic.NewBuilder().
		JobID(jobID).
		NodeID(&nodeID).
		Timestamp(now.Add(time.Second)).
		Code("internal.unclassified").
		Category("internal").
		Severity("warn").
		EffectiveDisposition(&fatal).
		Message("downstream sink terminated\x00").
		MustBuild()
	length := uint(4)
	aggregated := diagnostic.NewBuilder().
		JobID(jobID).
		NodeID(&nodeID).
		Timestamp(now).
		Code("gltf.zero_face_solid").
		Category("gltf").
		Severity("warn").
		Message("solid has zero faces").
		Aggregated(diagnostic.NewAggregateInfo(5, []string{"f1", "f2"})).
		SourceSpan(diagnostic.NewSourceSpan(10, &length)).
		MustBuild()

	dropped := uint64(2)
	require.NoError(t, r.SaveTerminalDiagnostics(ctx, jobID, diagnosticTestWorkflowID, now,
		[]*diagnostic.Diagnostic{jobLevel, failedNode},
		[]*diagnostic.Diagnostic{aggregated},
		&dropped,
	))

	t.Run("FindByJobNodeID scopes to one node, oldest first", func(t *testing.T) {
		t.Parallel()

		got, err := r.FindByJobNodeID(ctx, jobID, nodeID)
		require.NoError(t, err)
		require.Len(t, got, 2)
		assert.Equal(t, "gltf.zero_face_solid", got[0].Code())
		assert.Equal(t, "internal.unclassified", got[1].Code())
		assert.Equal(t, "downstream sink terminated\x00", got[1].Message())
		for _, d := range got {
			assert.True(t, d.Terminal())
			require.NotNil(t, d.NodeID())
			assert.Equal(t, nodeID, *d.NodeID())
		}
		require.NotNil(t, got[0].Aggregated())
		assert.Equal(t, uint64(5), got[0].Aggregated().Count())
		assert.Equal(t, []string{"f1", "f2"}, got[0].Aggregated().SampleFeatureIDs())
		require.NotNil(t, got[0].SourceSpan())
		assert.Equal(t, uint(10), got[0].SourceSpan().Offset())
		assert.Equal(t, &length, got[0].SourceSpan().Length())
		assert.True(t, got[0].Timestamp().Equal(now))
	})

	t.Run("FindByJobNodeID with an empty node ID reads the job-level bucket", func(t *testing.T) {
		t.Parallel()

		got, err := r.FindByJobNodeID(ctx, jobID, "")
		require.NoError(t, err)
		require.Len(t, got, 1)
		assert.Equal(t, "internal.invariant_violation", got[0].Code())
		assert.Nil(t, got[0].NodeID())
		require.NotNil(t, got[0].EffectiveDisposition())
		assert.Equal(t, fatal, *got[0].EffectiveDisposition())
	})

	t.Run("FindByJobID returns every row in timestamp order", func(t *testing.T) {
		t.Parallel()

		got, err := r.FindByJobID(ctx, jobID)
		require.NoError(t, err)
		require.Len(t, got, 3)
		assert.Equal(t, "internal.unclassified", got[2].Code())
	})

	t.Run("FindJobSummary returns the dropped event count", func(t *testing.T) {
		t.Parallel()

		got, err := r.FindJobSummary(ctx, jobID)
		require.NoError(t, err)
		require.NotNil(t, got)
		assert.Equal(t, uint64(2), *got)
	})

	t.Run("other jobs see nothing", func(t *testing.T) {
		t.Parallel()

		other := id.NewJobID()
		rows, err := r.FindByJobID(ctx, other)
		require.NoError(t, err)
		assert.Empty(t, rows)
		summary, err := r.FindJobSummary(ctx, other)
		require.NoError(t, err)
		assert.Nil(t, summary)
	})
}

func TestNodeDiagnostics_SaveTerminalDiagnostics_IsIdempotent(t *testing.T) {
	t.Parallel()

	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	r := postgres.NewNodeDiagnostics(pgxx.NewClient(pool))

	jobID := id.NewJobID()
	now := time.Now().UTC().Truncate(time.Microsecond)
	nodeID := "node-1"
	build := func(message string) *diagnostic.Diagnostic {
		return diagnostic.NewBuilder().
			JobID(jobID).
			NodeID(&nodeID).
			Timestamp(now).
			Code("geometry.invalid").
			Category("geometry").
			Severity("warn").
			Message(message).
			MustBuild()
	}

	first := uint64(1)
	require.NoError(t, r.SaveTerminalDiagnostics(ctx, jobID, diagnosticTestWorkflowID, now, nil, []*diagnostic.Diagnostic{build("first")}, &first))
	second := uint64(3)
	require.NoError(t, r.SaveTerminalDiagnostics(ctx, jobID, diagnosticTestWorkflowID, now, nil, []*diagnostic.Diagnostic{build("second")}, &second))

	got, err := r.FindByJobID(ctx, jobID)
	require.NoError(t, err)
	require.Len(t, got, 1)
	assert.Equal(t, "second", got[0].Message())

	summary, err := r.FindJobSummary(ctx, jobID)
	require.NoError(t, err)
	require.NotNil(t, summary)
	assert.Equal(t, uint64(3), *summary)
}

func TestNodeDiagnostics_SaveTerminalDiagnostics_WithoutDroppedCount(t *testing.T) {
	t.Parallel()

	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	r := postgres.NewNodeDiagnostics(pgxx.NewClient(pool))

	jobID := id.NewJobID()
	require.NoError(t, r.SaveTerminalDiagnostics(ctx, jobID, diagnosticTestWorkflowID, time.Now(), nil, nil, nil))

	summary, err := r.FindJobSummary(ctx, jobID)
	require.NoError(t, err)
	assert.Nil(t, summary)
}

func TestNodeDiagnostics_ReadsSubscriberRows(t *testing.T) {
	t.Parallel()

	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	r := postgres.NewNodeDiagnostics(pgxx.NewClient(pool))

	jobID := id.NewJobID()
	now := time.Now().UTC().Truncate(time.Microsecond)
	_, err := pool.Exec(ctx, `INSERT INTO node_diagnostics
		(id, job_id, workflow_id, node_id, ts, schema, code, category, severity, message)
		VALUES ($1, $2, $3, 'node-1', $4, 'diagnostic.v1', 'geometry.invalid', 'geometry', 'warn', 'bad ring')`,
		jobID.String()+":node-1:abc", jobID.String(), diagnosticTestWorkflowID, now)
	require.NoError(t, err)

	got, err := r.FindByJobNodeID(ctx, jobID, "node-1")
	require.NoError(t, err)
	require.Len(t, got, 1)
	assert.False(t, got[0].Terminal())
	assert.Equal(t, "bad ring", got[0].Message())
	assert.Nil(t, got[0].Aggregated())
	assert.Nil(t, got[0].SourceSpan())
}
