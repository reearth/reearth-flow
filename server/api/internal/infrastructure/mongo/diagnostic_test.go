package mongo

import (
	"context"
	"testing"
	"time"

	"github.com/reearth/reearth-flow/api/internal/infrastructure/mongo/mongodoc"
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

	// T5 normalization fix pin: simulates a row the subscriber's own
	// mongodoc.NewDiagnosticDocument writes for a DiagnosticEvent with no
	// nodeId — the nodeId bson field carries the "_job" sentinel, mirroring
	// the document ID's "_job" segment (both mongodoc.DiagnosticDocument
	// shapes are kept in lockstep). Before the fix, the field stayed
	// nil/raw-empty while only the ID got the sentinel, so this exact
	// FindByJobNodeID("") lookup could never match.
	t.Run("FindByJobNodeID with empty nodeID finds a subscriber-written job-level row", func(t *testing.T) {
		jobLevelJobID := id.NewJobID()
		nodeSegment := mongodoc.JobDiagnosticNodeSegment
		subscriberWrittenDoc := mongodoc.DiagnosticDocument{
			Timestamp:  now,
			NodeID:     &nodeSegment,
			ID:         jobLevelJobID.String() + ":_job:507f1f77bcf86cd799439011",
			JobID:      jobLevelJobID.String(),
			WorkflowID: "11111111-1111-1111-1111-111111111111",
			Schema:     "diagnostic.v1",
			Code:       "internal.unclassified",
			Category:   "internal",
			Severity:   "warn",
			Message:    "job-level diagnostic with no node context",
		}
		impl, ok := r.(*NodeDiagnostics)
		require.True(t, ok)
		require.NoError(t, impl.client.SaveOne(ctx, subscriberWrittenDoc.ID, subscriberWrittenDoc))

		got, err := r.FindByJobNodeID(ctx, jobLevelJobID, "")
		assert.NoError(t, err)
		require.Len(t, got, 1)
		assert.Equal(t, "internal.unclassified", got[0].Code())
		assert.Nil(t, got[0].NodeID(), "the _job sentinel must not leak into the domain model")
	})
}
