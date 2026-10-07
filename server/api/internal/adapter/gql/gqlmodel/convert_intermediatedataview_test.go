package gqlmodel

import (
	"testing"

	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/pkg/featureview"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestToIntermediateDataView_SizeLimitedCounts(t *testing.T) {
	format := featureview.FormatVectorTiles
	sizeLimitedFeatures, sizeLimitedTiles := 88, 3

	t.Run("passed through for vector tiles", func(t *testing.T) {
		got := ToIntermediateDataView(&interfaces.IntermediateDataViewResult{
			Status:              featureview.StatusReady,
			Format:              &format,
			SizeLimitedFeatures: &sizeLimitedFeatures,
			SizeLimitedTiles:    &sizeLimitedTiles,
		})
		require.NotNil(t, got)
		require.NotNil(t, got.SizeLimitedFeatures)
		require.NotNil(t, got.SizeLimitedTiles)
		assert.Equal(t, 88, *got.SizeLimitedFeatures)
		assert.Equal(t, 3, *got.SizeLimitedTiles)
	})

	t.Run("null stays null", func(t *testing.T) {
		got := ToIntermediateDataView(&interfaces.IntermediateDataViewResult{
			Status: featureview.StatusReady,
		})
		require.NotNil(t, got)
		assert.Nil(t, got.SizeLimitedFeatures)
		assert.Nil(t, got.SizeLimitedTiles)
	})
}
