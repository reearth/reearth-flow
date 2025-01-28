package app

import (
	"io"
	"mime"
	"net/http"
	"path"

	"github.com/labstack/echo/v4"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
)

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
			ct := "application/octet-stream"
			if ext := path.Ext(filename); ext != "" {
				ct2 := mime.TypeByExtension(ext)
				if ct2 != "" {
					ct = ct2
				}
			}

			// For HEAD requests, just set headers without streaming body
			if ctx.Request().Method == "HEAD" {
				ctx.Response().Header().Set("Content-Type", ct)
				return ctx.NoContent(http.StatusOK)
			}

			return ctx.Stream(http.StatusOK, ct, reader)
		}
	}

	group := ec.Group("")

	group.Match([]string{"GET", "HEAD"}, "/artifacts/:filename",
		fileHandler(func(ctx echo.Context) (io.Reader, string, error) {
			filename := ctx.Param("filename")
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
