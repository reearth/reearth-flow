package gqlmodel

import (
	"testing"

	"github.com/reearth/reearth-flow/api/pkg/diagnostic"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestToDiagnostic(t *testing.T) {
	t.Run("nil input", func(t *testing.T) {
		assert.Nil(t, ToDiagnostic(nil))
	})

	t.Run("minimal fields", func(t *testing.T) {
		d, err := diagnostic.NewBuilder().
			JobID(id.NewJobID()).
			Code("internal.unclassified").
			Category("internal").
			Severity("fatal").
			Message("boom").
			Build()
		require.NoError(t, err)

		got := ToDiagnostic(d)
		require.NotNil(t, got)
		assert.Equal(t, "internal.unclassified", got.Code)
		assert.Equal(t, "internal", got.Category)
		assert.Equal(t, "fatal", got.Severity)
		assert.Equal(t, "boom", got.Message)
		assert.Nil(t, got.EffectiveDisposition)
		assert.Nil(t, got.NodeID)
		assert.Nil(t, got.ActionType)
		assert.Nil(t, got.FeatureID)
		assert.Nil(t, got.Help)
		assert.Nil(t, got.AggregatedCount)
		assert.Nil(t, got.SampleFeatureIds)
	})

	t.Run("full fields including aggregated + featureId pass through verbatim", func(t *testing.T) {
		fatal := "fatal"
		nodeID := "subgraph-a.node-4"
		actionType := "AttributeManager"
		featureID := "11111111-1111-1111-1111-111111111111"
		help := "see the docs"

		d, err := diagnostic.NewBuilder().
			JobID(id.NewJobID()).
			Code("gltf.zero_face_solid").
			Category("gltf").
			Severity("warn").
			EffectiveDisposition(&fatal).
			NodeID(&nodeID).
			ActionType(&actionType).
			FeatureID(&featureID).
			Message("solid has zero faces").
			Help(&help).
			Aggregated(diagnostic.NewAggregateInfo(5, []string{"f1", "f2"})).
			Build()
		require.NoError(t, err)

		got := ToDiagnostic(d)
		require.NotNil(t, got)
		require.NotNil(t, got.EffectiveDisposition)
		assert.Equal(t, "fatal", *got.EffectiveDisposition)
		require.NotNil(t, got.NodeID)
		assert.Equal(t, nodeID, *got.NodeID)
		require.NotNil(t, got.ActionType)
		assert.Equal(t, actionType, *got.ActionType)
		require.NotNil(t, got.FeatureID)
		assert.Equal(t, ID(featureID), *got.FeatureID)
		require.NotNil(t, got.Help)
		assert.Equal(t, help, *got.Help)
		require.NotNil(t, got.AggregatedCount)
		assert.Equal(t, 5, *got.AggregatedCount)
		assert.Equal(t, []ID{"f1", "f2"}, got.SampleFeatureIds)
	})

	t.Run("aggregated with no sample ids leaves SampleFeatureIds nil", func(t *testing.T) {
		d, err := diagnostic.NewBuilder().
			JobID(id.NewJobID()).
			Code("internal.diagnostics_overflow").
			Category("internal").
			Severity("warn").
			Message("overflow").
			Aggregated(diagnostic.NewAggregateInfo(3, nil)).
			Build()
		require.NoError(t, err)

		got := ToDiagnostic(d)
		require.NotNil(t, got)
		require.NotNil(t, got.AggregatedCount)
		assert.Equal(t, 3, *got.AggregatedCount)
		assert.Nil(t, got.SampleFeatureIds)
	})
}

func TestToDiagnostics(t *testing.T) {
	t.Run("nil input", func(t *testing.T) {
		assert.Nil(t, ToDiagnostics(nil))
	})

	t.Run("empty input", func(t *testing.T) {
		got := ToDiagnostics([]*diagnostic.Diagnostic{})
		assert.NotNil(t, got)
		assert.Empty(t, got)
	})

	t.Run("converts every entry, skipping nils", func(t *testing.T) {
		jobID := id.NewJobID()
		d1, err := diagnostic.NewBuilder().
			JobID(jobID).
			Code("a.code").
			Category("internal").
			Severity("warn").
			Message("m1").
			Build()
		require.NoError(t, err)
		d2, err := diagnostic.NewBuilder().
			JobID(jobID).
			Code("b.code").
			Category("internal").
			Severity("warn").
			Message("m2").
			Build()
		require.NoError(t, err)

		got := ToDiagnostics([]*diagnostic.Diagnostic{d1, nil, d2})
		require.Len(t, got, 2)
		assert.Equal(t, "a.code", got[0].Code)
		assert.Equal(t, "b.code", got[1].Code)
	})
}
