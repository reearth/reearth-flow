package diagnostic

import (
	"testing"
	"time"

	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestBuilder_Build(t *testing.T) {
	jobID := id.NewJobID()
	now := time.Now()
	nodeID := "subgraph-a.sink-writer-2"
	disposition := "fatal"
	actionType := "Cesium 3D Tiles Writer"
	featureID := "33333333-3333-3333-3333-333333333333"
	help := "check the input"
	agg := NewAggregateInfo(5, []string{"f1", "f2"})
	span := NewSourceSpan(10, nil)

	d, err := NewBuilder().
		JobID(jobID).
		Timestamp(now).
		Code("internal.unclassified").
		Category("internal").
		Severity("warn").
		EffectiveDisposition(&disposition).
		NodeID(&nodeID).
		ActionType(&actionType).
		FeatureID(&featureID).
		Message("downstream sink terminated").
		Help(&help).
		Aggregated(agg).
		SourceSpan(span).
		Build()

	require.NoError(t, err)
	assert.Equal(t, jobID, d.JobID())
	assert.True(t, now.Equal(d.Timestamp()))
	assert.Equal(t, "internal.unclassified", d.Code())
	assert.Equal(t, "internal", d.Category())
	assert.Equal(t, "warn", d.Severity())
	require.NotNil(t, d.EffectiveDisposition())
	assert.Equal(t, "fatal", *d.EffectiveDisposition())
	require.NotNil(t, d.NodeID())
	assert.Equal(t, "subgraph-a.sink-writer-2", *d.NodeID())
	require.NotNil(t, d.ActionType())
	assert.Equal(t, "Cesium 3D Tiles Writer", *d.ActionType())
	require.NotNil(t, d.FeatureID())
	assert.Equal(t, featureID, *d.FeatureID())
	assert.Equal(t, "downstream sink terminated", d.Message())
	require.NotNil(t, d.Help())
	assert.Equal(t, "check the input", *d.Help())
	require.NotNil(t, d.Aggregated())
	assert.Equal(t, uint64(5), d.Aggregated().Count())
	assert.Equal(t, []string{"f1", "f2"}, d.Aggregated().SampleFeatureIDs())
	require.NotNil(t, d.SourceSpan())
	assert.Equal(t, uint(10), d.SourceSpan().Offset())
	assert.Nil(t, d.SourceSpan().Length())
}

func TestBuilder_Build_RequiresCode(t *testing.T) {
	_, err := NewBuilder().Category("internal").Build()
	assert.ErrorIs(t, err, ErrInvalidDiagnostic)
}

func TestBuilder_MustBuild_PanicsOnInvalid(t *testing.T) {
	assert.Panics(t, func() {
		NewBuilder().MustBuild()
	})
}
