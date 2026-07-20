package mongodoc

import (
	"encoding/json"
	"os"
	"testing"
	"time"

	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/pkg/diagnostic"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// diagnosticEventFixturePath is the shared wire-shape fixture also used by
// the api gateway package's own diagnostic tests (see
// internal/usecase/gateway/diagnostic_test.go) and by the subscriber module.
const diagnosticEventFixturePath = "../../../../../testdata/diagnostics/diagnostic_event.json"

func TestDiagnosticDocument_Model_RoundTripsFixture(t *testing.T) {
	raw, err := os.ReadFile(diagnosticEventFixturePath)
	require.NoError(t, err)

	var wire gateway.WireDiagnostic
	require.NoError(t, json.Unmarshal(raw, &wire))

	jobID := id.MustJobID("22222222-2222-2222-2222-222222222222")
	timestamp := time.Date(2026, 7, 16, 9, 31, 10, 0, time.UTC)

	domainDiagnostic, err := wire.ToDomain(jobID, timestamp)
	require.NoError(t, err)

	doc := NewFailedNodeDocument(jobID, domainDiagnostic)
	assert.Equal(t, jobID.String()+":subgraph-a.node-4:failed:gltf.zero_face_solid", doc.ID)
	assert.Equal(t, "gltf.zero_face_solid", doc.Code)
	assert.Equal(t, jobID.String(), doc.JobID)

	modelDiagnostic, err := doc.Model()
	require.NoError(t, err)
	assert.Equal(t, jobID, modelDiagnostic.JobID())
	assert.Equal(t, "gltf.zero_face_solid", modelDiagnostic.Code())
	assert.Equal(t, "gltf", modelDiagnostic.Category())
	assert.Equal(t, "warn", modelDiagnostic.Severity())
	require.NotNil(t, modelDiagnostic.NodeID())
	assert.Equal(t, "subgraph-a.node-4", *modelDiagnostic.NodeID())
	require.NotNil(t, modelDiagnostic.Aggregated())
	assert.Equal(t, uint64(5), modelDiagnostic.Aggregated().Count())
}

func TestNewFailedNodeDocument_FallsBackToJobSegment(t *testing.T) {
	jobID := id.NewJobID()
	d, err := diagnostic.NewBuilder().
		JobID(jobID).
		Timestamp(time.Now()).
		Code("internal.unclassified").
		Category("internal").
		Severity("warn").
		Message("no node context").
		Build()
	require.NoError(t, err)

	doc := NewFailedNodeDocument(jobID, d)
	assert.Equal(t, jobID.String()+":_job:failed:internal.unclassified", doc.ID)
	// The nodeId bson field carries the same "_job" sentinel as the ID
	// segment (T5 normalization fix), not nil/the raw empty string.
	require.NotNil(t, doc.NodeID)
	assert.Equal(t, JobDiagnosticNodeSegment, *doc.NodeID)

	// Round-tripping through Model() must strip the sentinel back to nil:
	// the domain/GraphQL layer's nil-means-job-level semantics must not see
	// the internal storage convention.
	modelDiagnostic, err := doc.Model()
	require.NoError(t, err)
	assert.Nil(t, modelDiagnostic.NodeID())
}

func TestDiagnosticDocument_Model_NilReceiver(t *testing.T) {
	var doc *DiagnosticDocument
	model, err := doc.Model()
	assert.NoError(t, err)
	assert.Nil(t, model)
}

func TestNewJobDiagnosticsSummaryDocument(t *testing.T) {
	jobID := id.NewJobID()
	now := time.Now()
	dropped := uint64(2)

	doc := NewJobDiagnosticsSummaryDocument(jobID, now, &dropped)

	assert.Equal(t, JobDiagnosticsSummaryID(jobID), doc.ID)
	assert.Equal(t, jobID.String()+":_job:summary", doc.ID)
	assert.Equal(t, jobID.String(), doc.JobID)
	require.NotNil(t, doc.DroppedEventCount)
	assert.Equal(t, uint64(2), *doc.DroppedEventCount)
}

func TestJobDiagnosticsSummaryDocument_Model(t *testing.T) {
	t.Run("returns the droppedEventCount pointer", func(t *testing.T) {
		dropped := uint64(7)
		doc := JobDiagnosticsSummaryDocument{DroppedEventCount: &dropped}

		got, err := doc.Model()
		require.NoError(t, err)
		require.NotNil(t, got)
		assert.Equal(t, uint64(7), *got)
	})

	t.Run("nil receiver", func(t *testing.T) {
		var doc *JobDiagnosticsSummaryDocument
		got, err := doc.Model()
		assert.NoError(t, err)
		assert.Nil(t, got)
	})
}

func TestNewAggregatedDiagnosticDocument(t *testing.T) {
	jobID := id.NewJobID()
	nodeID := "subgraph-a.node-4"

	d, err := diagnostic.NewBuilder().
		JobID(jobID).
		NodeID(&nodeID).
		Timestamp(time.Now()).
		Code("gltf.zero_face_solid").
		Category("gltf").
		Severity("warn").
		Message("solid has zero faces").
		Aggregated(diagnostic.NewAggregateInfo(5, []string{"f1", "f2"})).
		Build()
	require.NoError(t, err)

	doc := NewAggregatedDiagnosticDocument(jobID, d)
	assert.Equal(t, jobID.String()+":subgraph-a.node-4:aggregated:gltf.zero_face_solid", doc.ID)
	assert.Equal(t, "gltf.zero_face_solid", doc.Code)
	require.NotNil(t, doc.Aggregated)
	assert.Equal(t, uint64(5), doc.Aggregated.Count)
}
