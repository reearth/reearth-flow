package gateway_test

import (
	"encoding/json"
	"os"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
)

const diagnosticEventFixturePath = "../../../../testdata/diagnostics/diagnostic_event.json"

func TestWireDiagnostic_DecodesDiagnosticEventFixture(t *testing.T) {
	raw, err := os.ReadFile(diagnosticEventFixturePath)
	require.NoError(t, err)

	var wire gateway.WireDiagnostic
	require.NoError(t, json.Unmarshal(raw, &wire))

	assert.Equal(t, "gltf.zero_face_solid", wire.Code)
	assert.Equal(t, "gltf", wire.Category)
	assert.Equal(t, "warn", wire.Severity)
	require.NotNil(t, wire.EffectiveDisposition)
	assert.Equal(t, "warn_drop", *wire.EffectiveDisposition)
	require.NotNil(t, wire.NodeID)
	assert.Equal(t, "subgraph-a.node-4", *wire.NodeID)
	assert.Nil(t, wire.SourceSpan)
	require.NotNil(t, wire.Aggregated)
	assert.Equal(t, uint64(5), wire.Aggregated.Count)
	assert.Len(t, wire.Aggregated.SampleFeatureIds, 2)
}
