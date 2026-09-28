package app

import (
	"context"
	"io"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"

	"github.com/labstack/echo/v4"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/stretchr/testify/assert"
)

// artifactFakeFile records the name the route handed to ReadArtifact. The
// gateway is embedded so the methods this test never calls do not have to be
// written out; calling one would nil-panic and fail loudly.
type artifactFakeFile struct {
	gateway.File
	lastName string
}

func (f *artifactFakeFile) ReadArtifact(_ context.Context, name string) (io.ReadCloser, error) {
	f.lastName = name
	return io.NopCloser(strings.NewReader("{}")), nil
}

// A feature view's entry point is nested, so the artifacts route has to match
// more than one path segment and hand the whole remainder to the repo. This
// pins that: the URLs GetFeatureViewURL builds are the feature's only public
// entry point, and nothing else would notice if the route stopped serving them.
func TestServeFilesArtifactsMatchesNestedPaths(t *testing.T) {
	for _, tc := range []struct {
		name string
		path string
		want string
	}{
		{"single segment stays matched", "/artifacts/a.glb", "a.glb"},
		{"cesium entry point", "/artifacts/job1/feature-view/n1.out/tiles-all-abc/tileset.json", "job1/feature-view/n1.out/tiles-all-abc/tileset.json"},
		{"vector tile", "/artifacts/job1/feature-view/n1.out/tiles-f-abc/7/1/2.mvt", "job1/feature-view/n1.out/tiles-f-abc/7/1/2.mvt"},
	} {
		t.Run(tc.name, func(t *testing.T) {
			repo := &artifactFakeFile{}
			e := echo.New()
			serveFiles(e, repo)

			rec := httptest.NewRecorder()
			e.ServeHTTP(rec, httptest.NewRequest(http.MethodGet, tc.path, nil))

			assert.Equal(t, http.StatusOK, rec.Code)
			assert.Equal(t, tc.want, repo.lastName)
		})
	}
}

// The content type is derived from the name the handler reports, which for a
// nested path is the whole remainder rather than a bare file name.
func TestServeFilesArtifactsContentTypeFromNestedName(t *testing.T) {
	repo := &artifactFakeFile{}
	e := echo.New()
	serveFiles(e, repo)

	rec := httptest.NewRecorder()
	e.ServeHTTP(rec, httptest.NewRequest(http.MethodGet, "/artifacts/j/feature-view/f/k/model.glb", nil))

	assert.Equal(t, http.StatusOK, rec.Code)
	assert.Contains(t, rec.Header().Get("Content-Type"), "model/gltf-binary")
}
