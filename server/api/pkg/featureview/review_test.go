package featureview

import (
	"strings"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// A key becomes a storage path segment, and on the read path it arrives from
// the client. Anything that could climb out of the view's directory, or name a
// sibling, has to be refused before it reaches path.Join.
func TestValidateViewKeyRefusesTraversal(t *testing.T) {
	for _, bad := range []string{
		"", "..", "../..", "../../secret", "a/b", "a.b",
		"./a", `a\b`, "a%2Fb", strings.Repeat("a", MaxViewKeyLength+1),
	} {
		assert.ErrorIs(t, ValidateViewKey(bad), ErrInvalidViewKey, "key %q must be refused", bad)
	}
}

// Every key Key() produces has to survive its own validator.
func TestValidateViewKeyAcceptsGeneratedKeys(t *testing.T) {
	row := 42
	filter := "attributes.x > 1"
	for _, r := range []Request{
		{Shape: ShapeGLTF, Selection: Selection{Row: &row}, Options: DefaultOptions()},
		{Shape: ShapeTiles, Options: DefaultOptions()},
		{Shape: ShapeTiles, Selection: Selection{Filter: &filter}, Options: DefaultOptions()},
	} {
		key := r.Key()
		assert.NoError(t, ValidateViewKey(key), "generated key %q must validate", key)
	}
}

// The selector in a key is a hint, not identity: every filtered tiles request
// shares "f", so the digest alone carries the cache contract. 32 bits would
// start reusing another request's render in the tens of thousands.
func TestKeyCarriesAWideDigest(t *testing.T) {
	key := Request{Shape: ShapeTiles, Options: DefaultOptions()}.Key()
	digest := key[strings.LastIndexByte(key, '-')+1:]
	assert.Len(t, digest, 32, "expected a 128-bit hex digest, got %q", digest)
}

// ParseReport is the validation boundary for the renderer's only record of an
// outcome, so a document with anything after the object is not a usable report.
func TestParseReportRejectsTrailingData(t *testing.T) {
	valid := `{"version":1,"status":"empty","shape":"tiles","selectedFeatures":0,"renderedFeatures":0,"scanned":9}`

	_, err := ParseReport(strings.NewReader(valid))
	require.NoError(t, err, "the bare object must still parse")

	for _, trailing := range []string{valid + `{"version":1}`, valid + "\nnot json", valid + " []"} {
		_, err := ParseReport(strings.NewReader(trailing))
		assert.ErrorIs(t, err, ErrInvalidReport, "trailing data must be refused: %q", trailing)
	}
}

// The engine picks the format from the geometry it finds, but only within the
// shape it was asked for. A report pairing them otherwise would hand the caller
// a tilejson URL for a glb request.
func TestParseReportRejectsShapeFormatMismatch(t *testing.T) {
	report := func(shape, format string) string {
		return `{"version":1,"status":"ready","shape":"` + shape + `","format":"` + format +
			`","entryPoint":"k.glb","selectedFeatures":1,"renderedFeatures":1,"scanned":1}`
	}

	for _, bad := range [][2]string{
		{"gltf", "vector_tiles"},
		{"gltf", "cesium_3d_tiles"},
		{"tiles", "glb"},
		{"nonsense", "glb"},
	} {
		_, err := ParseReport(strings.NewReader(report(bad[0], bad[1])))
		assert.ErrorIs(t, err, ErrInvalidReport, "shape %q with format %q must be refused", bad[0], bad[1])
	}

	for _, ok := range [][2]string{
		{"gltf", "glb"},
		{"tiles", "cesium_3d_tiles"},
		{"tiles", "vector_tiles"},
	} {
		_, err := ParseReport(strings.NewReader(report(ok[0], ok[1])))
		assert.NoError(t, err, "shape %q with format %q is what the engine produces", ok[0], ok[1])
	}
}
