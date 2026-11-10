package featureview

import (
	"strings"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestParseReportReadsAReadyView(t *testing.T) {
	report, err := ParseReport(strings.NewReader(`{
	  "version": 1,
	  "status": "ready",
	  "shape": "tiles",
	  "format": "vector_tiles",
	  "filter": "attributes[\"kind\"] == \"keep\"",
	  "selectedFeatures": 900,
	  "renderedFeatures": 812,
	  "scanned": 900,
	  "entryPoint": "tiles-f-abcd1234/tilejson.json",
	  "written": ["tiles-f-abcd1234/8/227/101.mvt", "tiles-f-abcd1234/tilejson.json"]
	}`))
	require.NoError(t, err)

	assert.Equal(t, StatusReady, report.Status)
	require.NotNil(t, report.Format)
	assert.Equal(t, FormatVectorTiles, *report.Format)
	assert.Equal(t, "tiles-f-abcd1234/tilejson.json", report.EntryPoint)
	assert.Equal(t, 900, report.SelectedFeatures)
	assert.Equal(t, 812, report.RenderedFeatures)
	assert.Equal(t, 900, report.Scanned)
	assert.Less(t, report.RenderedFeatures, report.SelectedFeatures,
		"a partial drop is intended behaviour and still a READY view")
}

// A vector-tile report says what the size cap left out; no other format has a
// size cap, so its report carries neither count.
func TestParseReportReadsTheSizeLimitedCounts(t *testing.T) {
	report, err := ParseReport(strings.NewReader(`{
	  "version": 1, "status": "ready", "shape": "tiles", "format": "vector_tiles",
	  "selectedFeatures": 900, "renderedFeatures": 812, "scanned": 900,
	  "sizeLimitedFeatures": 88, "sizeLimitedTiles": 3,
	  "entryPoint": "tiles-f-abcd1234/tilejson.json"
	}`))
	require.NoError(t, err)
	require.NotNil(t, report.SizeLimitedFeatures)
	require.NotNil(t, report.SizeLimitedTiles)
	assert.Equal(t, 88, *report.SizeLimitedFeatures)
	assert.Equal(t, 3, *report.SizeLimitedTiles)

	report, err = ParseReport(strings.NewReader(`{
	  "version": 1, "status": "ready", "shape": "tiles", "format": "cesium_3d_tiles",
	  "selectedFeatures": 2, "renderedFeatures": 2, "scanned": 2,
	  "entryPoint": "tiles-f-abcd1234/tileset.json"
	}`))
	require.NoError(t, err)
	assert.Nil(t, report.SizeLimitedFeatures)
	assert.Nil(t, report.SizeLimitedTiles)
}

// The outcomes that wrote no view are the reason the report exists: without it
// "this row holds 2D geometry" would reach the user as an exit code.
func TestParseReportReadsTheOutcomesThatWroteNothing(t *testing.T) {
	for _, status := range []Status{StatusEmpty, StatusUnsupportedGeometry, StatusFailed} {
		t.Run(string(status), func(t *testing.T) {
			report, err := ParseReport(strings.NewReader(`{
			  "version": 1,
			  "status": "` + string(status) + `",
			  "shape": "gltf",
			  "row": 42,
			  "renderedFeatures": 0,
			  "scanned": 43,
			  "error": "A glTF view needs 3D geometry, and this row holds 2D geometry"
			}`))
			require.NoError(t, err)

			assert.Equal(t, status, report.Status)
			assert.Nil(t, report.Format)
			assert.Empty(t, report.EntryPoint)
			require.NotNil(t, report.Error)
		})
	}
}

// A truncated or self-contradicting report must not be served to a client as a
// usable view.
func TestParseReportRefusesAnUnusableReport(t *testing.T) {
	for name, body := range map[string]string{
		"not json":               `{`,
		"wrong version":          `{"version": 2, "status": "ready", "shape": "gltf", "format": "glb", "entryPoint": "k.glb"}`,
		"missing version":        `{"status": "ready", "shape": "gltf", "format": "glb", "entryPoint": "k.glb"}`,
		"unknown status":         `{"version": 1, "status": "done", "shape": "gltf"}`,
		"ready without format":   `{"version": 1, "status": "ready", "shape": "gltf", "entryPoint": "k.glb"}`,
		"ready without entry":    `{"version": 1, "status": "ready", "shape": "gltf", "format": "glb"}`,
		"ready with blank entry": `{"version": 1, "status": "ready", "shape": "gltf", "format": "glb", "entryPoint": "  "}`,
		"non-terminal status":    `{"version": 1, "status": "rendering", "shape": "gltf"}`,
		"top level is not the first layer": `{"version": 1, "status": "ready", "shape": "tiles",
		  "format": "vector_tiles", "entryPoint": "k/tilejson.json",
		  "layers": [{"format": "cesium_3d_tiles", "entryPoint": "k/tileset.json"},
		             {"format": "vector_tiles", "entryPoint": "k/tilejson.json"}]}`,
		"layer the shape cannot produce": `{"version": 1, "status": "ready", "shape": "tiles",
		  "format": "cesium_3d_tiles", "entryPoint": "k/tileset.json",
		  "layers": [{"format": "cesium_3d_tiles", "entryPoint": "k/tileset.json"},
		             {"format": "glb", "entryPoint": "k.glb"}]}`,
		"layer without entry": `{"version": 1, "status": "ready", "shape": "tiles",
		  "format": "cesium_3d_tiles", "entryPoint": "k/tileset.json",
		  "layers": [{"format": "cesium_3d_tiles", "entryPoint": "k/tileset.json"},
		             {"format": "vector_tiles", "entryPoint": " "}]}`,
		"two layers of one format": `{"version": 1, "status": "ready", "shape": "tiles",
		  "format": "cesium_3d_tiles", "entryPoint": "k/tileset.json",
		  "layers": [{"format": "cesium_3d_tiles", "entryPoint": "k/tileset.json"},
		             {"format": "cesium_3d_tiles", "entryPoint": "k/tileset.json"}]}`,
	} {
		t.Run(name, func(t *testing.T) {
			_, err := ParseReport(strings.NewReader(body))
			assert.ErrorIs(t, err, ErrInvalidReport)
		})
	}
}

// A parsed report carries counts the renderer measured; the stand-in for an
// unreadable one carries none, so its zeros must not be served as a count.
func TestUnreadableReportHasNoKnownCounts(t *testing.T) {
	stand := UnreadableReport("could not be read")
	assert.Equal(t, StatusFailed, stand.Status)
	assert.False(t, stand.CountsKnown())
	assert.Empty(t, stand.Shape)

	parsed, err := ParseReport(strings.NewReader(`{"version":1,"status":"empty","shape":"tiles",
	  "selectedFeatures":0,"renderedFeatures":0,"scanned":0}`))
	require.NoError(t, err)
	assert.True(t, parsed.CountsKnown(), "zero is a measured count on a report the renderer wrote")
}

// The report names the engine that wrote it, so a view keyed for one engine
// but rendered by another can be noticed.
func TestParseReportReadsTheEngineVersion(t *testing.T) {
	r, err := ParseReport(strings.NewReader(`{"version":1,"engineVersion":"0.0.583","status":"empty","shape":"tiles",
	  "selectedFeatures":0,"renderedFeatures":0,"scanned":3}`))
	require.NoError(t, err)
	assert.Equal(t, "0.0.583", r.EngineVersion)

	r, err = ParseReport(strings.NewReader(`{"version":1,"status":"empty","shape":"tiles",
	  "selectedFeatures":0,"renderedFeatures":0,"scanned":3}`))
	require.NoError(t, err, "a report from before engines named themselves still parses")
	assert.Empty(t, r.EngineVersion)
}

// A view of geometry holding both 3D and 2D lists both of its layers, each
// with its own counts, and the top level repeats the first.
func TestParseReportReadsTheLayers(t *testing.T) {
	t.Parallel()

	report, err := ParseReport(strings.NewReader(`{
	  "version": 1, "status": "ready", "shape": "tiles", "format": "cesium_3d_tiles",
	  "selectedFeatures": 16, "renderedFeatures": 16, "scanned": 16,
	  "entryPoint": "k/tileset.json",
	  "layers": [
	    {"format": "cesium_3d_tiles", "entryPoint": "k/tileset.json",
	     "selectedFeatures": 8, "renderedFeatures": 8},
	    {"format": "vector_tiles", "entryPoint": "k/tilejson.json",
	     "selectedFeatures": 9, "renderedFeatures": 7,
	     "sizeLimitedFeatures": 2, "sizeLimitedTiles": 1}
	  ]
	}`))
	require.NoError(t, err)

	layers := report.ViewLayers()
	require.Len(t, layers, 2)
	assert.Equal(t, FormatCesium3DTiles, layers[0].Format)
	assert.Equal(t, "k/tileset.json", layers[0].EntryPoint)
	assert.Equal(t, 8, layers[0].RenderedFeatures)
	assert.Nil(t, layers[0].SizeLimitedFeatures)
	assert.Equal(t, FormatVectorTiles, layers[1].Format)
	assert.Equal(t, 9, layers[1].SelectedFeatures)
	assert.Equal(t, 7, layers[1].RenderedFeatures)
	require.NotNil(t, layers[1].SizeLimitedFeatures)
	assert.Equal(t, 2, *layers[1].SizeLimitedFeatures)
}

// A report from an engine that wrote a single entry point names it at the top
// level, and that is the view's only layer. A view that is not ready has none.
func TestReportViewLayersOfASingleEntryPoint(t *testing.T) {
	t.Parallel()

	t.Run("ready", func(t *testing.T) {
		t.Parallel()

		report, err := ParseReport(strings.NewReader(`{
		  "version": 1, "status": "ready", "shape": "tiles", "format": "vector_tiles",
		  "selectedFeatures": 900, "renderedFeatures": 812, "scanned": 900,
		  "sizeLimitedFeatures": 88, "sizeLimitedTiles": 3,
		  "entryPoint": "k/tilejson.json"
		}`))
		require.NoError(t, err)

		layers := report.ViewLayers()
		require.Len(t, layers, 1)
		assert.Equal(t, FormatVectorTiles, layers[0].Format)
		assert.Equal(t, "k/tilejson.json", layers[0].EntryPoint)
		assert.Equal(t, 900, layers[0].SelectedFeatures)
		assert.Equal(t, 812, layers[0].RenderedFeatures)
		require.NotNil(t, layers[0].SizeLimitedFeatures)
		assert.Equal(t, 88, *layers[0].SizeLimitedFeatures)
	})

	t.Run("not ready", func(t *testing.T) {
		t.Parallel()

		report, err := ParseReport(strings.NewReader(`{"version":1,"status":"empty","shape":"tiles",
		  "selectedFeatures":0,"renderedFeatures":0,"scanned":3}`))
		require.NoError(t, err)
		assert.Empty(t, report.ViewLayers())
	})
}
