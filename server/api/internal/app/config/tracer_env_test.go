package config

import (
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// TestTracerSampleBindsToTracerSampleEnv pins the env var name the deployed
// services actually set. Without the explicit envconfig tag the field binds to
// REEARTH_FLOW_TRACERSAMPLE, so REEARTH_FLOW_TRACER_SAMPLE was silently
// ignored, the ratio stayed 0, and createSampler turned that into
// NeverSample — tracing initialised cleanly and exported nothing.
func TestTracerSampleBindsToTracerSampleEnv(t *testing.T) {
	t.Setenv("REEARTH_FLOW_TRACER", "gcp")
	t.Setenv("REEARTH_FLOW_TRACER_SAMPLE", "0.25")

	c, err := ReadConfig(false)
	require.NoError(t, err)

	assert.Equal(t, "gcp", c.Tracer)
	assert.Equal(t, 0.25, c.TracerSample,
		"REEARTH_FLOW_TRACER_SAMPLE must bind; a zero ratio silently disables all tracing")
}

// TestTracerEndpointStillBinds guards the sibling field, which relies on the
// underscore in its Go field name rather than a tag.
func TestTracerEndpointStillBinds(t *testing.T) {
	t.Setenv("REEARTH_FLOW_TRACER_ENDPOINT", "collector:4317")

	c, err := ReadConfig(false)
	require.NoError(t, err)

	assert.Equal(t, "collector:4317", c.Tracer_Endpoint)
}
