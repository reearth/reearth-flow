// Package featureview describes a viewable rendering of the intermediate data a
// job left on one output port.
//
// A run writes one intermediate-data file per (node, output port) pair, keyed by
// the engine's port_file_id: `[subgraphPrefix.]nodeID.port`. The engine still
// wraps that string in a type called EdgeId, but nothing here is keyed by an
// edge.
//
// The engine's `reearth-flow-worker render-view` subcommand does the rendering;
// this package holds the vocabulary both sides agree on, the option validation
// that keeps a render affordable, and the key derivation that makes a view
// cacheable. Nothing here talks to storage or to the worker.
package featureview

import (
	"crypto/sha256"
	"encoding/hex"
	"fmt"
	"math"
	"strconv"
	"strings"
)

// Shape is what the caller asks for. It is not the output format: a Shape of
// ShapeTiles renders 3D Tiles or vector tiles depending on the geometry the
// selected features carry, and only the engine can decide which.
type Shape string

const (
	// ShapeGLTF renders one row into a glb. Needs 3D geometry: the engine
	// refuses a purely 2D row.
	ShapeGLTF Shape = "gltf"
	// ShapeTiles renders a whole selection into a tile pyramid.
	ShapeTiles Shape = "tiles"
)

// Format is what a render produced, and is only known once it has run.
type Format string

const (
	FormatGLB           Format = "glb"
	FormatCesium3DTiles Format = "cesium_3d_tiles"
	FormatVectorTiles   Format = "vector_tiles"
)

// Status is the outcome of a view, as the report records it. The three
// non-Ready terminal values are properties of the data rather than faults, so
// they arrive as a status and not as an error.
type Status string

const (
	// StatusReady means the entry point is loadable.
	StatusReady Status = "ready"
	// StatusEmpty covers both ways a view ends up with nothing to show: a
	// selection that matched no features, and a selection whose every feature
	// was dropped for naming no CRS or carrying geometry the writer cannot
	// draw. The engine distinguishes them — an empty selection renders an empty
	// tileset while an all-dropped one is refused — but a client has the same
	// nothing to display either way, and the report's counts say which it was.
	StatusEmpty Status = "empty"
	// StatusUnsupportedGeometry means a glb was asked of a purely 2D row.
	StatusUnsupportedGeometry Status = "unsupported_geometry"
	StatusFailed              Status = "failed"
)

// TextureCodec values are the strings the engine's --texture-codec parser
// accepts, so they can be passed through unchanged.
type TextureCodec string

const (
	TextureCodecJPEG       TextureCodec = "jpeg"
	TextureCodecPNG        TextureCodec = "png"
	TextureCodecKTX2ETC1S  TextureCodec = "ktx2-etc1s"
	TextureCodecKTX2UASTC  TextureCodec = "ktx2-uastc"
	TextureCodecUntextured TextureCodec = "untextured"
)

func (c TextureCodec) valid() bool {
	switch c {
	case TextureCodecJPEG, TextureCodecPNG, TextureCodecKTX2ETC1S,
		TextureCodecKTX2UASTC, TextureCodecUntextured:
		return true
	}
	return false
}

// Zoom bounds for a 2D pyramid.
//
// MaxZoomSpan is the guard that matters. The engine slices every feature across
// the whole range into one in-memory map and only starts writing once every
// level is accumulated, so a wide range costs memory and tiles geometrically.
// The engine's own default range is 0-15; that is sixteen levels, which is far
// too generous for a view built on demand, so the server neither adopts it as a
// default nor accepts it as input.
//
// The cost is not spread evenly across the span: a level holds up to 4^z tiles,
// so the deep end dominates the tile count. The shallow end is cheap in tiles
// but not always in memory: a polygon too small to cover a pixel is skipped at
// slicing, so a level below a port's detail holds little of it, but points and
// lines are kept at every level. Widening a range upwards is what has to be paid
// for. DefaultOptions is chosen with that asymmetry in mind.
const (
	MinSupportedZoom uint8 = 0
	MaxSupportedZoom uint8 = 24
	MaxZoomSpan      uint8 = 8
)

// Options are the render knobs. Both groups are carried regardless of shape:
// for ShapeTiles the output format is not known until the render runs, so the
// engine is handed both and ignores whichever half does not apply.
//
// They are not caller-settable: the API always renders with DefaultOptions.
// The worker contract carries every knob regardless, so tuning a default is a
// change here alone, and exposing one later is an API change alone. A changed
// default also changes every Key, so views rendered under the old one are not
// reused.
//
// Fields are ordered for struct packing (govet fieldalignment) rather than by
// group, so each carries its group as a trailing comment instead.
type Options struct {
	TextureCodec   TextureCodec // 3D
	TexelSize      float64      // 3D
	TargetTileSize uint64       // 3D
	MaxTileBytes   uint64       // 2D
	Extent         int32        // 2D
	Draco          bool         // 3D
	MinZoom        uint8        // 2D
	MaxZoom        uint8        // 2D
}

// DefaultOptions matches the engine's ViewOptions::default() except in two
// places.
//
// MaxTileBytes is four times the engine's 500 KB. When a vector tile exceeds
// the cap the engine drops its smallest features until it fits, and the
// rendered count does not reflect it. The engine's figure suits a basemap,
// where thinning dense areas is an acceptable trade; a view exists to inspect
// the data, where a missing feature reads as a bug in the workflow. A larger
// cap costs viewer load time instead, which is why it is raised rather than
// removed.
//
// The zoom range is deliberately narrower than the engine's 0-15. See
// MaxZoomSpan.
//
// The low end is what a viewer sees first. The engine emits tiles only for
// min..=max and the tilejson advertises minzoom, so a layer is simply absent
// below it: starting the range at the detail levels leaves the whole-port
// overview — the thing a tiles view is for — blank until the user has already
// zoomed in. 6 puts a city-scale port comfortably inside the first tile a
// fitted camera loads, and costs almost nothing (see MaxZoomSpan).
func DefaultOptions() Options {
	return Options{
		Draco:          true,
		TexelSize:      0,
		TextureCodec:   TextureCodecJPEG,
		TargetTileSize: 1 << 20,
		MinZoom:        6,
		MaxZoom:        14,
		Extent:         4096,
		MaxTileBytes:   2_000_000,
	}
}

func (o Options) validate() error {
	if !o.TextureCodec.valid() {
		return fmt.Errorf("%w: unknown texture codec %q", ErrInvalidRequest, o.TextureCodec)
	}
	if o.TexelSize < 0 || math.IsNaN(o.TexelSize) || math.IsInf(o.TexelSize, 0) {
		return fmt.Errorf("%w: texelSize must be a non-negative number, got %v", ErrInvalidRequest, o.TexelSize)
	}
	if o.TargetTileSize == 0 {
		return fmt.Errorf("%w: targetTileSize must be positive", ErrInvalidRequest)
	}
	// The engine rejects a non-positive extent before writing any tile; say so
	// here instead, where the message can reach the caller.
	if o.Extent <= 0 {
		return fmt.Errorf("%w: extent must be positive, got %d", ErrInvalidRequest, o.Extent)
	}
	if o.MaxTileBytes == 0 {
		return fmt.Errorf("%w: maxTileBytes must be positive", ErrInvalidRequest)
	}
	if o.MinZoom > o.MaxZoom {
		return fmt.Errorf("%w: minZoom %d exceeds maxZoom %d", ErrInvalidRequest, o.MinZoom, o.MaxZoom)
	}
	if o.MaxZoom > MaxSupportedZoom {
		return fmt.Errorf("%w: maxZoom %d exceeds the supported maximum %d", ErrInvalidRequest, o.MaxZoom, MaxSupportedZoom)
	}
	// Refused rather than narrowed: silently rendering a shallower pyramid than
	// asked for would look like a rendering bug at the far zoom levels.
	if span := o.MaxZoom - o.MinZoom; span > MaxZoomSpan {
		return fmt.Errorf(
			"%w: zoom range %d-%d spans %d levels, more than the %d a view may cover",
			ErrInvalidRequest, o.MinZoom, o.MaxZoom, span, MaxZoomSpan,
		)
	}
	return nil
}

// Selection is which features a view covers. Exactly one field is set, and
// which one is legal depends on the shape.
type Selection struct {
	// Row is a 0-based line in the intermediate-data file, which is the row the
	// table shows the feature at.
	Row *int
	// Filter is a Flow expression evaluated against each feature. It sees the
	// feature alone, not workflow variables — and it sees the feature as it was
	// written, before the row index is attached, so it cannot select by row.
	Filter *string
}

// Request is everything that decides a view's content, and so everything that
// goes into its Key.
type Request struct {
	Shape Shape
	// ServedFrom is the public URL of the directory the view is written under,
	// or empty when storage has none. A vector tileset's tilejson names its
	// tiles by absolute URL, so where a tiles view is served is part of what
	// it contains.
	ServedFrom string
	Selection  Selection
	Options    Options
}

// Validate reports whether the request describes a view the engine can render.
//
// The shape/selection pairing mirrors the engine CLI, where each shape declares
// only the selection argument it accepts and clap rejects the other's. The
// GraphQL API enforces it too, with one mutation per shape, but this package
// does not rely on its callers for that.
func (r Request) Validate() error {
	switch r.Shape {
	case ShapeGLTF:
		if r.Selection.Row == nil {
			return fmt.Errorf("%w: a %s view needs the row to render", ErrInvalidRequest, ShapeGLTF)
		}
		if r.Selection.Filter != nil {
			return fmt.Errorf("%w: a %s view renders one row and takes no filter", ErrInvalidRequest, ShapeGLTF)
		}
	case ShapeTiles:
		// A tiles view covers a whole selection and never a single row, by
		// design rather than by omission. There is no per-feature 2D view
		// because a global 2D view is always light-weight: to show one 2D
		// feature you render the whole port and select it in the viewer, where every
		// tile feature carries its row number as its id. ShapeGLTF is the
		// per-feature view, and it exists because a global 3D view is not cheap.
		if r.Selection.Row != nil {
			return fmt.Errorf("%w: a %s view renders a whole selection and takes no row", ErrInvalidRequest, ShapeTiles)
		}
	default:
		return fmt.Errorf("%w: unknown shape %q", ErrInvalidRequest, r.Shape)
	}

	if r.Selection.Row != nil && *r.Selection.Row < 0 {
		return fmt.Errorf("%w: row must not be negative, got %d", ErrInvalidRequest, *r.Selection.Row)
	}
	if r.Selection.Filter != nil && strings.TrimSpace(*r.Selection.Filter) == "" {
		return fmt.Errorf("%w: filter must not be blank; omit it to render every feature", ErrInvalidRequest)
	}

	return r.Options.validate()
}

// Key identifies a view by its content: the same request always yields the same
// key, and a different one always yields a different key. It is the cache
// contract and the output path segment, so it stays short and path-safe.
//
// Only the options that apply to the shape are hashed. A glb is not tiled, so
// folding the tile knobs in would invalidate a perfectly good cached glb every
// time an unrelated 2D default moved. ServedFrom is a tile knob in that sense:
// a glb never names its own URL.
func (r Request) Key() string {
	var b strings.Builder
	b.WriteString("v1\n")
	b.WriteString(string(r.Shape))
	b.WriteByte('\n')

	if r.Selection.Row != nil {
		b.WriteString("row=")
		b.WriteString(strconv.Itoa(*r.Selection.Row))
	}
	b.WriteByte('\n')
	if r.Selection.Filter != nil {
		b.WriteString("filter=")
		b.WriteString(*r.Selection.Filter)
	}
	b.WriteByte('\n')

	o := r.Options
	fmt.Fprintf(&b, "draco=%t\ntexel=%s\ncodec=%s\n",
		o.Draco, strconv.FormatFloat(o.TexelSize, 'g', -1, 64), o.TextureCodec)
	if r.Shape == ShapeTiles {
		fmt.Fprintf(&b, "target=%d\nzoom=%d-%d\nextent=%d\ntilebytes=%d\nserved=%s\n",
			o.TargetTileSize, o.MinZoom, o.MaxZoom, o.Extent, o.MaxTileBytes, r.ServedFrom)
	}

	// 128 bits of the digest. The selector in front is a human-readable hint,
	// not identity — every filtered request shares the same "f" — so the hash
	// alone has to carry the cache contract above. A 32-bit prefix would start
	// colliding, and reusing another request's render, in the tens of thousands.
	sum := sha256.Sum256([]byte(b.String()))
	return fmt.Sprintf("%s-%s-%s", r.Shape, r.selector(), hex.EncodeToString(sum[:16]))
}

// selector is the human-readable middle of a key: enough to tell two views of
// the same port apart in a bucket listing without carrying the whole request.
func (r Request) selector() string {
	switch {
	case r.Selection.Row != nil:
		return "r" + strconv.Itoa(*r.Selection.Row)
	case r.Selection.Filter != nil:
		return "f"
	default:
		return "all"
	}
}

// ShapeFromKey recovers the shape a key was made for from its leading
// segment, which Key always writes. It is how a view whose report says nothing
// usable is still described as the kind of view it is.
func ShapeFromKey(key string) (Shape, bool) {
	for _, s := range []Shape{ShapeGLTF, ShapeTiles} {
		if strings.HasPrefix(key, string(s)+"-") {
			return s, true
		}
	}
	return "", false
}

// EntryPointName is the file the caller opens for a format, relative to the
// view's own directory. A glb is a single file beside the report; a tile
// pyramid is a directory named for the key.
func EntryPointName(key string, format Format) (string, error) {
	switch format {
	case FormatGLB:
		return key + ".glb", nil
	case FormatCesium3DTiles:
		return key + "/tileset.json", nil
	case FormatVectorTiles:
		return key + "/tilejson.json", nil
	default:
		return "", fmt.Errorf("unknown view format %q", format)
	}
}

// VectorTileTemplate is where the engine writes a vector tileset's tiles,
// relative to the tileset's directory.
const VectorTileTemplate = "{z}/{x}/{y}.mvt"

// VectorTilesURL is the absolute tile URL template a vector tileset's tilejson
// names, given the URL of the tileset's directory. It is built by
// concatenation: escaping it as a URL path would turn the braces into
// %7Bz%7D, which no viewer substitutes.
func VectorTilesURL(tilesetURL string) string {
	return strings.TrimSuffix(tilesetURL, "/") + "/" + VectorTileTemplate
}

// MaxViewKeyLength bounds a view key so a hostile one cannot be used to build
// an unreasonable object name.
const MaxViewKeyLength = 128

// ValidateViewKey checks a view key before it is used to build a storage path.
//
// Key() only ever produces `shape-selector-digest`, so letters, digits and the
// separating hyphen are the whole accepted set. The check exists because a key
// also arrives from the client on the read path, where it reaches path.Join as
// a segment: refusing dots and separators outright is what stops `../..` from
// addressing a file outside the view's own directory.
func ValidateViewKey(key string) error {
	if key == "" {
		return fmt.Errorf("%w: must not be empty", ErrInvalidViewKey)
	}
	if len(key) > MaxViewKeyLength {
		return fmt.Errorf("%w: longer than the %d character maximum", ErrInvalidViewKey, MaxViewKeyLength)
	}

	for i := 0; i < len(key); i++ {
		c := key[i]
		switch {
		case c >= 'a' && c <= 'z',
			c >= 'A' && c <= 'Z',
			c >= '0' && c <= '9',
			c == '-':
		default:
			return fmt.Errorf("%w: unexpected character %q", ErrInvalidViewKey, string(c))
		}
	}

	return nil
}

// ReportName is the report for a key, relative to the view's own directory. It
// is the cache probe: unlike the entry point, its name does not depend on a
// format the server cannot know before the render runs.
func ReportName(key string) string {
	return key + ".report.json"
}
