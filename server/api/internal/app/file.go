package app

import (
	"io"
	"mime"
	"net/http"
	"path"

	"github.com/labstack/echo/v4"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearthx/log"
)

// Content types Go's mime table lacks, registered so a view artifact is not
// served as application/octet-stream to a viewer that needs the real type.
func init() {
	for ext, ct := range map[string]string{
		".glb": "model/gltf-binary",
		".mvt": "application/vnd.mapbox-vector-tile",
	} {
		if err := mime.AddExtensionType(ext, ct); err != nil {
			log.Warnf("app: could not register the %s content type: %v", ext, err)
		}
	}
}

func serveFiles(
	ec *echo.Echo,
	repo gateway.File,
) {
	if repo == nil {
		return
	}

	fileHandler := func(handler func(echo.Context) (io.Reader, string, error)) echo.HandlerFunc {
		return func(ctx echo.Context) error {
			reader, filename, err := handler(ctx)
			if err != nil {
				return err
			}
			// Close the object reader even for HEAD, which never reads the body.
			// Each one holds a storage connection until it is closed.
			if c, ok := reader.(io.Closer); ok {
				defer func() {
					if err := c.Close(); err != nil {
						log.Warnfc(ctx.Request().Context(), "app: closing %s: %v", filename, err)
					}
				}()
			}
			ct := "application/octet-stream"
			if ext := path.Ext(filename); ext != "" {
				ct2 := mime.TypeByExtension(ext)
				if ct2 != "" {
					ct = ct2
				}
			}

			if ctx.Request().Method == "HEAD" {
				ctx.Response().Header().Set("Content-Type", ct)
				return ctx.NoContent(http.StatusOK)
			}

			return ctx.Stream(http.StatusOK, ct, reader)
		}
	}

	group := ec.Group("")

	// A wildcard, because feature-view paths are nested
	// (`<job>/feature-view/<file>/<key>/tileset.json`).
	group.Match([]string{"GET", "HEAD"}, "/artifacts/*",
		fileHandler(func(ctx echo.Context) (io.Reader, string, error) {
			filename := ctx.Param("*")
			r, err := repo.ReadArtifact(ctx.Request().Context(), filename)
			return r, filename, err
		}),
	)

	group.Match([]string{"GET", "HEAD"}, "/assets/:filename",
		fileHandler(func(ctx echo.Context) (io.Reader, string, error) {
			filename := ctx.Param("filename")
			r, err := repo.ReadAsset(ctx.Request().Context(), filename)
			return r, filename, err
		}),
	)

	group.Match([]string{"GET", "HEAD"}, "/workflows/:filename",
		fileHandler(func(ctx echo.Context) (io.Reader, string, error) {
			filename := ctx.Param("filename")
			r, err := repo.ReadWorkflow(ctx.Request().Context(), filename)
			return r, filename, err
		}),
	)

	group.Match([]string{"GET", "HEAD"}, "/metadata/:filename",
		fileHandler(func(ctx echo.Context) (io.Reader, string, error) {
			filename := ctx.Param("filename")
			r, err := repo.ReadMetadata(ctx.Request().Context(), filename)
			return r, filename, err
		}),
	)
}
