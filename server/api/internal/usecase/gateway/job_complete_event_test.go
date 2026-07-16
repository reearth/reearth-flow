package gateway_test

import (
	"encoding/json"
	"os"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
)

const fixturePath = "../../../../testdata/diagnostics/job_complete_with_diagnostics.json"

func TestJobCompleteEvent_RoundTripsDiagnosticsFixture(t *testing.T) {
	raw, err := os.ReadFile(fixturePath)
	require.NoError(t, err)

	var first gateway.JobCompleteEvent
	require.NoError(t, json.Unmarshal(raw, &first))

	remarshaled, err := json.Marshal(first)
	require.NoError(t, err)

	var second gateway.JobCompleteEvent
	require.NoError(t, json.Unmarshal(remarshaled, &second))

	assert.Equal(t, first, second)

	require.Len(t, second.FailedNodes, 2)
	assert.Equal(t, "internal.invariant_violation", second.FailedNodes[0].Code)
	assert.Equal(t, "internal.unclassified", second.FailedNodes[1].Code)
	require.NotNil(t, second.FailedNodes[1].NodeID)
	assert.Equal(t, "subgraph-a.sink-writer-2", *second.FailedNodes[1].NodeID)
	require.NotNil(t, second.FailedNodes[0].EffectiveDisposition)
	assert.Equal(t, "fatal", *second.FailedNodes[0].EffectiveDisposition)

	require.Len(t, second.AggregatedDiagnostics, 1)
	assert.Equal(t, "gltf.zero_face_solid", second.AggregatedDiagnostics[0].Code)
	require.NotNil(t, second.AggregatedDiagnostics[0].Aggregated)
	assert.Equal(t, uint64(5), second.AggregatedDiagnostics[0].Aggregated.Count)
	assert.Equal(t, []string{
		"33333333-3333-3333-3333-333333333333",
		"44444444-4444-4444-4444-444444444444",
	}, second.AggregatedDiagnostics[0].Aggregated.SampleFeatureIds)

	require.NotNil(t, second.DroppedEventCount)
	assert.Equal(t, uint64(2), *second.DroppedEventCount)
}

func TestJobCompleteEvent_LegacyWireCompat(t *testing.T) {
	legacy := `{"workflowId":"11111111-1111-1111-1111-111111111111","jobId":"22222222-2222-2222-2222-222222222222","result":"success","timestamp":"2026-01-01T00:00:00Z"}`

	var event gateway.JobCompleteEvent
	require.NoError(t, json.Unmarshal([]byte(legacy), &event))

	assert.Equal(t, "success", event.Result)
	assert.Nil(t, event.FailedNodes)
	assert.Nil(t, event.AggregatedDiagnostics)
	assert.Nil(t, event.DroppedEventCount)
}
