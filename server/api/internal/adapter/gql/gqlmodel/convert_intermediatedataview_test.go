package gqlmodel

import (
	"testing"

	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/pkg/featureview"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestToIntermediateDataView_SizeLimitedCounts(t *testing.T) {
	sizeLimitedFeatures, sizeLimitedTiles := 88, 3

	t.Run("passed through for vector tiles", func(t *testing.T) {
		got := ToIntermediateDataView(&interfaces.IntermediateDataViewResult{
			Status:              featureview.StatusReady,
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

func TestToIntermediateDataView_Layers(t *testing.T) {
	t.Parallel()

	t.Run("each layer converts", func(t *testing.T) {
		t.Parallel()

		sizeLimitedFeatures, sizeLimitedTiles := 2, 1
		got := ToIntermediateDataView(&interfaces.IntermediateDataViewResult{
			Status: featureview.StatusReady,
			Layers: []interfaces.IntermediateDataViewLayer{
				{
					Format:           featureview.FormatCesium3DTiles,
					EntryPointURL:    "https://api.example/v/tileset.json",
					SelectedFeatures: 8,
					RenderedFeatures: 8,
				},
				{
					Format:              featureview.FormatVectorTiles,
					EntryPointURL:       "https://api.example/v/tilejson.json",
					SelectedFeatures:    9,
					RenderedFeatures:    7,
					SizeLimitedFeatures: &sizeLimitedFeatures,
					SizeLimitedTiles:    &sizeLimitedTiles,
				},
			},
		})
		require.NotNil(t, got)
		require.Len(t, got.Layers, 2)
		assert.Equal(t, IntermediateDataViewFormatCesium3dTiles, got.Layers[0].Format)
		assert.Equal(t, "https://api.example/v/tileset.json", got.Layers[0].EntryPointURL)
		assert.Nil(t, got.Layers[0].SizeLimitedFeatures)
		assert.Equal(t, IntermediateDataViewFormatVectorTiles, got.Layers[1].Format)
		assert.Equal(t, 9, got.Layers[1].SelectedFeatures)
		assert.Equal(t, 7, got.Layers[1].RenderedFeatures)
		require.NotNil(t, got.Layers[1].SizeLimitedFeatures)
		assert.Equal(t, 2, *got.Layers[1].SizeLimitedFeatures)
	})

	// The list is non-null in the schema.
	t.Run("a view with no layers has an empty list", func(t *testing.T) {
		t.Parallel()

		got := ToIntermediateDataView(&interfaces.IntermediateDataViewResult{
			Status: featureview.StatusEmpty,
		})
		require.NotNil(t, got)
		assert.NotNil(t, got.Layers)
		assert.Empty(t, got.Layers)
	})
}
