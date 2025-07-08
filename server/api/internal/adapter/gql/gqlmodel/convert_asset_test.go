package gqlmodel

import (
	"testing"

	"github.com/reearth/reearth-flow/api/pkg/asset"
	"github.com/stretchr/testify/assert"
)

func TestToPreviewType(t *testing.T) {
	image := asset.PreviewType("image")
	geojson := asset.PreviewType("geojson")
	unknownGeo := asset.PreviewType("unknown_geo")
	geotiff := asset.PreviewType("geotiff")

	imageExpected := PreviewTypeImage
	geojsonExpected := PreviewTypeGeojson
	unknownGeoExpected := PreviewTypeUnknownGeo
	geotiffExpected := PreviewTypeGeotiff

	tests := []struct {
		name     string
		input    *asset.PreviewType
		expected *PreviewType
	}{
		{
			name:     "nil input",
			input:    nil,
			expected: nil,
		},
		{
			name:     "Image type",
			input:    &image,
			expected: &imageExpected,
		},
		{
			name:     "GeoJSON extended type",
			input:    &geojson,
			expected: &geojsonExpected,
		},
		{
			name:     "Unknown Geo extended type",
			input:    &unknownGeo,
			expected: &unknownGeoExpected,
		},
		{
			name:     "GeoTIFF extended type",
			input:    &geotiff,
			expected: &geotiffExpected,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result := ToPreviewType(tt.input)
			if tt.expected == nil {
				assert.Nil(t, result)
			} else {
				assert.NotNil(t, result)
				assert.Equal(t, *tt.expected, *result)
			}
		})
	}
}
