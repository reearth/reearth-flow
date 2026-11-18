// Package featureview holds the shared vocabulary, option validation and cache
// keys for rendering one output port's intermediate data with the engine's
// `render-view` subcommand. Intermediate files are keyed by the engine's
// port_file_id, `[subgraphPrefix.]nodeID.port`.
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
	// StatusEmpty means no feature matched, or every match was dropped for
	// lacking a CRS or drawable geometry. The report's counts say which.
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

// Zoom bounds for a 2D pyramid. MaxZoomSpan matters most because the engine
// holds every level in memory until all are sliced, and a level holds up to
// 4^z tiles.
const (
	MinSupportedZoom uint8 = 0
	MaxSupportedZoom uint8 = 24
	MaxZoomSpan      uint8 = 8
)

// Options carries both the 2D and 3D knobs because the output format is only
// known once the engine runs. The API always renders with DefaultOptions, so
// changing a default changes every Key.
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

// DefaultOptions matches the engine's ViewOptions::default() except for a
// narrower zoom range and four times its 500 KB MaxTileBytes. The engine
// silently drops a tile's smallest features to fit the cap, which in an
// inspection view would look like a workflow bug.
func DefaultOptions() Options {
	return Options{
		Draco:          true,
		TexelSize:      0,
		TextureCodec:   TextureCodecJPEG,
		TargetTileSize: 1 << 20,
		MinZoom:        6, // low enough that the zoomed-out overview has tiles
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
	// Filter is a Flow expression evaluated against each feature as written,
	// without workflow variables or the row index.
	Filter *string
}

// Request is everything that decides a view's content, and so everything that
// goes into its Key.
type Request struct {
	Shape Shape
	// ServedFrom is the public URL of the view's directory, or empty when
	// storage has none. It is content because a tilejson names its tiles by
	// absolute URL.
	ServedFrom string
	// EngineVersion is the rendering engine's version, or empty when unknown.
	// It is content because an engine fix can change any render.
	EngineVersion string
	Selection     Selection
	Options       Options
}

// Validate reports whether the engine can render the request. The
// shape/selection pairing mirrors the engine CLI, which accepts one selection
// argument per shape.
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
		// One 2D feature is shown by rendering the whole port and picking it in
		// the viewer, where each tile feature's id is its row. ShapeGLTF is the
		// per-feature view because a global 3D view is expensive.
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

// Key identifies a view by its content and doubles as the cache key and output
// path segment. Only options that apply to the shape are hashed, with
// ServedFrom counted as a tile option, so a 2D default change keeps cached glbs.
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
	// Omitted when unknown, which keeps keys from before versions were hashed
	// valid.
	if r.EngineVersion != "" {
		fmt.Fprintf(&b, "engine=%s\n", r.EngineVersion)
	}

	// 128 bits of the digest, because the selector prefix is only a hint and the
	// hash alone must keep distinct requests apart.
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

// VectorTilesURL builds the absolute tile URL template by concatenation,
// because path escaping would turn the braces into %7Bz%7D. A query or fragment
// on the directory URL stays after the template.
func VectorTilesURL(tilesetURL string) string {
	dir, suffix := tilesetURL, ""
	if i := strings.IndexAny(tilesetURL, "?#"); i >= 0 {
		dir, suffix = tilesetURL[:i], tilesetURL[i:]
	}
	return strings.TrimSuffix(dir, "/") + "/" + VectorTileTemplate + suffix
}

// MaxViewKeyLength bounds a view key so a hostile one cannot be used to build
// an unreasonable object name.
const MaxViewKeyLength = 128

// ValidateViewKey checks a client-supplied view key before it becomes a storage
// path segment. Allowing only letters, digits and hyphens stops `../..` from
// escaping the view's directory.
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
