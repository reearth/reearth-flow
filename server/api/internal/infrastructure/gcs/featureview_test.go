package gcs

import (
	"net/url"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func viewRepo(t *testing.T) *fileRepo {
	t.Helper()
	assetBase, err := url.Parse("https://api.example/assets")
	require.NoError(t, err)
	artifactBase, err := url.Parse("https://api.example/artifacts")
	require.NoError(t, err)
	return &fileRepo{bucketName: "flow-bucket", base: assetBase, artifactBase: artifactBase}
}

// The renderer is handed gs:// because the engine's http storage backend is
// read-only and unauthenticated.
func TestFeatureViewUploadURIsAreGCSURIs(t *testing.T) {
	f := viewRepo(t)

	assert.Equal(t,
		"gs://flow-bucket/artifacts/JOB/feature-view/node.default",
		f.GetFeatureViewUploadURI("JOB", "node.default"))

	assert.Equal(t,
		"gs://flow-bucket/artifacts/JOB/feature-view/node.default/gltf-r42-abcd1234.report.json",
		f.GetFeatureViewReportUploadURI("JOB", "node.default", "gltf-r42-abcd1234"))
}

// A view is served by the /artifacts route, which already reads under the
// artifacts prefix — so the public URL must not repeat it, and must not be
// built from the asset base.
func TestFeatureViewURLResolvesAgainstTheArtifactBase(t *testing.T) {
	f := viewRepo(t)

	got := f.GetFeatureViewURL("JOB", "node.default", "tiles-all-abcd1234/tileset.json")

	assert.Equal(t,
		"https://api.example/artifacts/JOB/feature-view/node.default/tiles-all-abcd1234/tileset.json",
		got)
	assert.NotContains(t, got, "/assets/")
	assert.NotContains(t, got, "artifacts/artifacts")
}

func TestFeatureViewURLIsEmptyWithoutAnArtifactBase(t *testing.T) {
	f := viewRepo(t)
	f.artifactBase = nil

	assert.Empty(t, f.GetFeatureViewURL("JOB", "node.default", "k.glb"),
		"an unconfigured base yields no URL rather than a wrong one")
}

// A port name is the author's own, so a view URL has to carry spaces and
// non-ASCII intact: escaped on the way out, and decoded by the artifacts route
// back to the object name the renderer wrote.
func TestFeatureViewURLEscapesAuthorNamedPorts(t *testing.T) {
	f := viewRepo(t)

	for _, fileID := range []string{"node.high rise", "node.建物"} {
		got := f.GetFeatureViewURL("JOB", fileID, "k/tileset.json")

		u, err := url.Parse(got)
		require.NoError(t, err, got)
		assert.NotContains(t, got, " ", "the URL itself must be escaped")
		assert.Equal(t, "/artifacts/JOB/feature-view/"+fileID+"/k/tileset.json", u.Path)
	}
}
