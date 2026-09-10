package featureview

import (
	"strings"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func row(n int) *int          { return &n }
func filter(s string) *string { return &s }

func gltfRequest(r int) Request {
	return Request{Shape: ShapeGLTF, Selection: Selection{Row: row(r)}, Options: DefaultOptions()}
}

func tilesRequest() Request {
	return Request{Shape: ShapeTiles, Options: DefaultOptions()}
}

// The key is the cache contract: a repeat request must reuse a rendered view,
// and any change to what would be rendered must not.
func TestKeyIsStableForTheSameRequest(t *testing.T) {
	assert.Equal(t, gltfRequest(42).Key(), gltfRequest(42).Key())
	assert.NotEqual(t, gltfRequest(42).Key(), gltfRequest(43).Key())
	assert.NotEqual(t, gltfRequest(42).Key(), tilesRequest().Key())

	withFilter := tilesRequest()
	withFilter.Selection.Filter = filter(`attributes["kind"] == "keep"`)
	assert.NotEqual(t, tilesRequest().Key(), withFilter.Key())

	other := withFilter
	other.Selection.Filter = filter(`attributes["kind"] == "drop"`)
	assert.NotEqual(t, withFilter.Key(), other.Key(), "the filter decides which features are rendered")
}

func TestKeyChangesWithEveryRenderOption(t *testing.T) {
	base := tilesRequest()
	baseKey := base.Key()

	for name, mutate := range map[string]func(*Options){
		"draco":          func(o *Options) { o.Draco = !o.Draco },
		"texelSize":      func(o *Options) { o.TexelSize = 0.5 },
		"textureCodec":   func(o *Options) { o.TextureCodec = TextureCodecPNG },
		"targetTileSize": func(o *Options) { o.TargetTileSize = 2 << 20 },
		"minZoom":        func(o *Options) { o.MinZoom = 9 },
		"maxZoom":        func(o *Options) { o.MaxZoom = 13 },
		"extent":         func(o *Options) { o.Extent = 2048 },
		"maxTileBytes":   func(o *Options) { o.MaxTileBytes = 250_000 },
	} {
		t.Run(name, func(t *testing.T) {
			changed := base
			mutate(&changed.Options)
			require.NoError(t, changed.Validate())
			assert.NotEqual(t, baseKey, changed.Key(),
				"%s changes the output, so it must change the key", name)
		})
	}
}

// A glb is not tiled, so folding the tile knobs into its key would invalidate a
// perfectly good cached glb whenever an unrelated 2D default moved.
func TestGltfKeyIgnoresTileOptions(t *testing.T) {
	base := gltfRequest(7)

	retiled := base
	retiled.Options.MinZoom = 9
	retiled.Options.MaxZoom = 12
	retiled.Options.Extent = 1024
	retiled.Options.MaxTileBytes = 111_111
	retiled.Options.TargetTileSize = 3 << 20
	require.NoError(t, retiled.Validate())

	assert.Equal(t, base.Key(), retiled.Key())

	// The options a glb does use still count.
	untextured := base
	untextured.Options.TextureCodec = TextureCodecUntextured
	assert.NotEqual(t, base.Key(), untextured.Key())
}

// The key is a path segment, so it has to survive being one.
func TestKeyIsPathSafeAndReadable(t *testing.T) {
	awkward := tilesRequest()
	awkward.Selection.Filter = filter(`attributes["../etc/passwd"] == "/x"`)

	key := awkward.Key()
	assert.NotContains(t, key, "/")
	assert.NotContains(t, key, "..")
	assert.True(t, strings.HasPrefix(key, "tiles-f-"), "got %q", key)

	assert.True(t, strings.HasPrefix(gltfRequest(42).Key(), "gltf-r42-"))
	assert.True(t, strings.HasPrefix(tilesRequest().Key(), "tiles-all-"))
}

// Each shape takes only the selection it can act on, mirroring the engine CLI
// where clap rejects the other's argument outright.
func TestAShapeTakesOnlyItsOwnSelection(t *testing.T) {
	assert.NoError(t, gltfRequest(1).Validate())
	assert.NoError(t, tilesRequest().Validate(), "a tileset may take every row")

	filtered := tilesRequest()
	filtered.Selection.Filter = filter("true")
	assert.NoError(t, filtered.Validate())

	noRow := Request{Shape: ShapeGLTF, Options: DefaultOptions()}
	assert.ErrorIs(t, noRow.Validate(), ErrInvalidRequest, "a glb needs the row to render")

	gltfFiltered := gltfRequest(1)
	gltfFiltered.Selection.Filter = filter("true")
	assert.ErrorIs(t, gltfFiltered.Validate(), ErrInvalidRequest)

	tilesRow := tilesRequest()
	tilesRow.Selection.Row = row(1)
	assert.ErrorIs(t, tilesRow.Validate(), ErrInvalidRequest)

	unknown := Request{Shape: Shape("obj"), Options: DefaultOptions()}
	assert.ErrorIs(t, unknown.Validate(), ErrInvalidRequest)
}

// The zoom span guard is the one that protects the renderer's memory: the
// engine holds every level's sliced geometry at once.
func TestZoomRangeIsBounded(t *testing.T) {
	tests := map[string]struct {
		min, max uint8
		ok       bool
	}{
		"server default":                {8, 14, true},
		"single level":                  {10, 10, true},
		"exactly the span cap":          {6, 14, true},
		"one past the cap":              {5, 14, false},
		"the engine's own 0-15 default": {0, 15, false},
		"inverted":                      {14, 8, false},
		"past the supported maximum":    {20, 25, false},
	}

	for name, tt := range tests {
		t.Run(name, func(t *testing.T) {
			r := tilesRequest()
			r.Options.MinZoom = tt.min
			r.Options.MaxZoom = tt.max

			err := r.Validate()
			if tt.ok {
				assert.NoError(t, err)
				return
			}
			assert.ErrorIs(t, err, ErrInvalidRequest)
		})
	}
}

func TestOptionsAreValidated(t *testing.T) {
	for name, mutate := range map[string]func(*Options){
		"unknown codec":    func(o *Options) { o.TextureCodec = TextureCodec("webp") },
		"negative texel":   func(o *Options) { o.TexelSize = -1 },
		"zero target size": func(o *Options) { o.TargetTileSize = 0 },
		"zero extent":      func(o *Options) { o.Extent = 0 },
		"negative extent":  func(o *Options) { o.Extent = -4096 },
		"zero tile bytes":  func(o *Options) { o.MaxTileBytes = 0 },
	} {
		t.Run(name, func(t *testing.T) {
			r := tilesRequest()
			mutate(&r.Options)
			assert.ErrorIs(t, r.Validate(), ErrInvalidRequest)
		})
	}
}

func TestBlankFilterIsRefused(t *testing.T) {
	r := tilesRequest()
	r.Selection.Filter = filter("   ")
	assert.ErrorIs(t, r.Validate(), ErrInvalidRequest,
		"a blank filter matches nothing; omitting it renders everything")
}

func TestEntryPointNameFollowsTheFormat(t *testing.T) {
	name, err := EntryPointName("gltf-r42-abcd1234", FormatGLB)
	require.NoError(t, err)
	assert.Equal(t, "gltf-r42-abcd1234.glb", name)

	name, err = EntryPointName("tiles-all-abcd1234", FormatCesium3DTiles)
	require.NoError(t, err)
	assert.Equal(t, "tiles-all-abcd1234/tileset.json", name)

	name, err = EntryPointName("tiles-all-abcd1234", FormatVectorTiles)
	require.NoError(t, err)
	assert.Equal(t, "tiles-all-abcd1234/tilejson.json", name)

	_, err = EntryPointName("k", Format("obj"))
	assert.Error(t, err)
}

// The report is probed to answer "already rendered?", so its name must not
// depend on a format the server cannot know until the render has run.
func TestReportNameDoesNotDependOnFormat(t *testing.T) {
	assert.Equal(t, "tiles-all-abcd1234.report.json", ReportName("tiles-all-abcd1234"))
}

// The default range has to reach a zoomed-out camera. The engine emits tiles
// only for min..=max and the tilejson advertises minzoom, so a default that
// starts at the detail levels renders the whole-port overview blank until the
// user has already zoomed in — which is the one thing a tiles view is for.
func TestDefaultOptionsCoverAZoomedOutCamera(t *testing.T) {
	o := DefaultOptions()

	assert.LessOrEqual(t, o.MinZoom, uint8(6), "the overview has to be visible before the user zooms in")
	assert.GreaterOrEqual(t, o.MaxZoom, uint8(14), "and still resolve detail at the top")
	assert.NoError(t, Request{Shape: ShapeTiles, Options: o}.Validate(), "the default must satisfy its own guards")
}
