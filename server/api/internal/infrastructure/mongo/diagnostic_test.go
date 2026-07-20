package mongo

import (
	"context"
	"testing"
	"time"

	"github.com/reearth/reearth-flow/api/pkg/diagnostic"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearthx/mongox"
	"github.com/reearth/reearthx/mongox/mongotest"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// TestNodeDiagnostics_FindByJobNodeID_And_FindByJobID exercises the read
// path against a real MongoDB (via mongotest.Connect, which self-skips when
// no test DB URI is configured — see mongotest.Env, only wired in e2e/
// common.go for e2e runs, so this test skips under plain `make test` and
// runs under CI's `ci-api-test` job which starts a mongo service).
func TestNodeDiagnostics_FindByJobNodeID_And_FindByJobID(t *testing.T) {
	c := mongotest.Connect(t)(t)
	ctx := context.Background()

	r := NewNodeDiagnostics(mongox.NewClientWithDatabase(c))

	jobID := id.NewJobID()
	now := time.Now().UTC().Truncate(time.Millisecond)

	failedNode, err := diagnostic.NewBuilder().
		JobID(jobID).
		Timestamp(now).
		Code("internal.invariant_violation").
		Category("internal").
		Severity("fatal").
		Message("invariant violation").
		Build()
	require.NoError(t, err)

	nodeID := "subgraph-a.sink-writer-2"
	cascadeFailedNode, err := diagnostic.NewBuilder().
		JobID(jobID).
		NodeID(&nodeID).
		Timestamp(now).
		Code("internal.unclassified").
		Category("internal").
		Severity("warn").
		Message("downstream sink terminated").
		Build()
	require.NoError(t, err)

	aggregated, err := diagnostic.NewBuilder().
		JobID(jobID).
		NodeID(&nodeID).
		Timestamp(now).
		Code("gltf.zero_face_solid").
		Category("gltf").
		Severity("warn").
		Message("solid has zero faces").
		Aggregated(diagnostic.NewAggregateInfo(5, []string{"f1", "f2"})).
		Build()
	require.NoError(t, err)

	dropped := uint64(2)
	require.NoError(t, r.SaveTerminalDiagnostics(
		ctx, jobID, now,
		[]*diagnostic.Diagnostic{failedNode, cascadeFailedNode},
		[]*diagnostic.Diagnostic{aggregated},
		&dropped,
	))

	t.Run("FindByJobNodeID scopes to one node", func(t *testing.T) {
		got, err := r.FindByJobNodeID(ctx, jobID, nodeID)
		assert.NoError(t, err)
		require.Len(t, got, 2)
	})

	t.Run("FindByJobID returns all code-bearing rows, excludes the summary row", func(t *testing.T) {
		got, err := r.FindByJobID(ctx, jobID)
		assert.NoError(t, err)
		// failedNode + cascadeFailedNode + aggregated == 3; the _job:summary
		// row (droppedEventCount) has no "code" field and must be excluded.
		require.Len(t, got, 3)
		for _, d := range got {
			assert.NotEmpty(t, d.Code())
		}
	})

	t.Run("FindByJobNodeID unknown node is empty, not error", func(t *testing.T) {
		got, err := r.FindByJobNodeID(ctx, jobID, "no-such-node")
		assert.NoError(t, err)
		assert.Empty(t, got)
	})

	t.Run("SaveTerminalDiagnostics upserts the same failed-node row idempotently", func(t *testing.T) {
		// Re-persisting the same failedNode (simulating a JobCompleteEvent
		// redelivery after an earlier persist failure) must not duplicate
		// rows: the deterministic ID upserts in place.
		require.NoError(t, r.SaveTerminalDiagnostics(
			ctx, jobID, now,
			[]*diagnostic.Diagnostic{failedNode},
			nil, nil,
		))

		got, err := r.FindByJobID(ctx, jobID)
		assert.NoError(t, err)
		require.Len(t, got, 3)
	})

	t.Run("FindJobSummary reads the droppedEventCount persisted above", func(t *testing.T) {
		got, err := r.FindJobSummary(ctx, jobID)
		assert.NoError(t, err)
		require.NotNil(t, got)
		assert.Equal(t, uint64(2), *got)
	})

	t.Run("FindJobSummary is nil, not error, for a job with no summary row", func(t *testing.T) {
		got, err := r.FindJobSummary(ctx, id.NewJobID())
		assert.NoError(t, err)
		assert.Nil(t, got)
	})
}
