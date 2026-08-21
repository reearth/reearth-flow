package diagnostic_test

import (
	"encoding/json"
	"os"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"github.com/reearth/reearth-flow/subscriber/pkg/diagnostic"
)

const diagnosticEventFullFixturePath = "../../../testdata/diagnostics/diagnostic_event.json"

func TestDiagnosticEvent_DecodesDiagnosticEventFixture(t *testing.T) {
	raw, err := os.ReadFile(diagnosticEventFullFixturePath)
	require.NoError(t, err)

	var evt diagnostic.DiagnosticEvent
	require.NoError(t, json.Unmarshal(raw, &evt))

	assert.Equal(t, diagnostic.DiagnosticSchemaV1, evt.Schema)
	assert.Equal(t, "11111111-1111-1111-1111-111111111111", evt.WorkflowID)
	assert.Equal(t, "22222222-2222-2222-2222-222222222222", evt.JobID)
	assert.Equal(t, "2026-07-16T09:31:10Z", evt.Timestamp.UTC().Format(time.RFC3339))
	assert.Equal(t, "gltf.zero_face_solid", evt.Code)
	assert.Equal(t, "gltf", evt.Category)
	assert.Equal(t, "warn", evt.Severity)
	require.NotNil(t, evt.EffectiveDisposition)
	assert.Equal(t, "warn_drop", *evt.EffectiveDisposition)
	require.NotNil(t, evt.NodeID)
	assert.Equal(t, "subgraph-a.node-4", *evt.NodeID)
	require.NotNil(t, evt.ActionType)
	assert.Equal(t, "Gltf Writer", *evt.ActionType)
	require.NotNil(t, evt.FeatureID)
	assert.Equal(t, "33333333-3333-3333-3333-333333333333", *evt.FeatureID)
	assert.Equal(t, "solid has zero faces and was dropped", evt.Message)
	require.NotNil(t, evt.Help)
	assert.Nil(t, evt.SourceSpan)
	require.NotNil(t, evt.Aggregated)
	assert.Equal(t, uint64(5), evt.Aggregated.Count)
	assert.Len(t, evt.Aggregated.SampleFeatureIds, 2)

	// Round trip: Marshal -> Unmarshal must survive byte-safely (this is
	// the hazard the spec calls out for the pubsub -> Redis -> api hop).
	data, err := json.Marshal(&evt)
	require.NoError(t, err)

	var roundTripped diagnostic.DiagnosticEvent
	require.NoError(t, json.Unmarshal(data, &roundTripped))
	assert.Equal(t, evt, roundTripped)
}

func TestNewDiagnosticEvent(t *testing.T) {
	ts := time.Date(2026, 7, 16, 9, 31, 10, 0, time.UTC)
	wire := diagnostic.WireDiagnostic{
		Code:     "gltf.zero_face_solid",
		Category: "gltf",
		Severity: "warn",
		Message:  "solid has zero faces and was dropped",
	}

	t.Run("valid event", func(t *testing.T) {
		evt, err := diagnostic.NewDiagnosticEvent(diagnostic.DiagnosticSchemaV1, "wf-1", "job-1", ts, wire)
		require.NoError(t, err)
		require.NotNil(t, evt)
		assert.Equal(t, diagnostic.DiagnosticSchemaV1, evt.Schema)
		assert.Equal(t, "wf-1", evt.WorkflowID)
		assert.Equal(t, "job-1", evt.JobID)
		assert.Equal(t, ts, evt.Timestamp)
		assert.Equal(t, "gltf.zero_face_solid", evt.Code)
	})

	t.Run("invalid schema", func(t *testing.T) {
		evt, err := diagnostic.NewDiagnosticEvent("bogus.v1", "wf-1", "job-1", ts, wire)
		assert.Nil(t, evt)
		assert.ErrorIs(t, err, diagnostic.ErrInvalidDiagnosticEvent)
	})

	t.Run("missing jobId", func(t *testing.T) {
		evt, err := diagnostic.NewDiagnosticEvent(diagnostic.DiagnosticSchemaV1, "wf-1", "", ts, wire)
		assert.Nil(t, evt)
		assert.ErrorIs(t, err, diagnostic.ErrInvalidDiagnosticEvent)
	})

	t.Run("unknown enum strings pass through verbatim (no validation)", func(t *testing.T) {
		exotic := diagnostic.WireDiagnostic{
			Code:     "future.new_code",
			Category: "future_category",
			Severity: "future_severity",
			Message:  "unknown values must survive",
		}
		evt, err := diagnostic.NewDiagnosticEvent(diagnostic.DiagnosticSchemaV1, "wf-1", "job-1", ts, exotic)
		require.NoError(t, err)
		assert.Equal(t, "future_category", evt.Category)
		assert.Equal(t, "future_severity", evt.Severity)
	})
}
