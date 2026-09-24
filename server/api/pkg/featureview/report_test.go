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
	} {
		t.Run(name, func(t *testing.T) {
			_, err := ParseReport(strings.NewReader(body))
			assert.ErrorIs(t, err, ErrInvalidReport)
		})
	}
}
