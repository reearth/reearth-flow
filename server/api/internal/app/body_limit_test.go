package app

import (
	"bytes"
	"io"
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/labstack/echo/v4"
	"github.com/labstack/echo/v4/middleware"
	"github.com/stretchr/testify/assert"
)

func newBodyLimitTestEcho() *echo.Echo {
	e := echo.New()
	e.Use(middleware.BodyLimitWithConfig(middleware.BodyLimitConfig{
		Skipper: func(c echo.Context) bool {
			return c.Path() == "/api/graphql"
		},
		Limit: nonGraphQLBodyLimit,
	}))
	echoBody := func(c echo.Context) error {
		if _, err := io.Copy(io.Discard, c.Request().Body); err != nil {
			return err
		}
		return c.NoContent(http.StatusOK)
	}
	e.POST("/api/signup", echoBody)
	e.POST("/api/graphql", echoBody)
	return e
}

func TestBodyLimitMiddleware_RejectsOversizedNonGraphQLBody(t *testing.T) {
	t.Parallel()
	e := newBodyLimitTestEcho()

	body := bytes.Repeat([]byte("a"), 32*1024*1024+1)
	req := httptest.NewRequest(http.MethodPost, "/api/signup", bytes.NewReader(body))
	rec := httptest.NewRecorder()
	e.ServeHTTP(rec, req)

	assert.Equal(t, http.StatusRequestEntityTooLarge, rec.Code)
}

func TestBodyLimitMiddleware_AllowsBodyUnderLimit(t *testing.T) {
	t.Parallel()
	e := newBodyLimitTestEcho()

	body := bytes.Repeat([]byte("a"), 1024)
	req := httptest.NewRequest(http.MethodPost, "/api/signup", bytes.NewReader(body))
	rec := httptest.NewRecorder()
	e.ServeHTTP(rec, req)

	assert.Equal(t, http.StatusOK, rec.Code)
}

func TestBodyLimitMiddleware_SkipsGraphQLRoute(t *testing.T) {
	t.Parallel()
	e := newBodyLimitTestEcho()

	// Larger than the 32M non-GraphQL cap, but the GraphQL route must not be
	// bounded by this middleware (it enforces maxUploadSize separately).
	body := bytes.Repeat([]byte("a"), 33*1024*1024)
	req := httptest.NewRequest(http.MethodPost, "/api/graphql", bytes.NewReader(body))
	rec := httptest.NewRecorder()
	e.ServeHTTP(rec, req)

	assert.Equal(t, http.StatusOK, rec.Code)
}

func TestGraphqlBodyLimitMiddleware_EnforcesLimitAtReadTime(t *testing.T) {
	t.Parallel()
	const limit = 16

	e := echo.New()
	e.POST("/api/graphql", func(c echo.Context) error {
		if _, err := io.Copy(io.Discard, c.Request().Body); err != nil {
			return c.String(http.StatusRequestEntityTooLarge, err.Error())
		}
		return c.NoContent(http.StatusOK)
	}, graphqlBodyLimitMiddleware(limit))

	oversized := bytes.Repeat([]byte("a"), limit+1)
	req := httptest.NewRequest(http.MethodPost, "/api/graphql", bytes.NewReader(oversized))
	rec := httptest.NewRecorder()
	e.ServeHTTP(rec, req)
	assert.Equal(t, http.StatusRequestEntityTooLarge, rec.Code)

	undersized := bytes.Repeat([]byte("a"), limit)
	req2 := httptest.NewRequest(http.MethodPost, "/api/graphql", bytes.NewReader(undersized))
	rec2 := httptest.NewRecorder()
	e.ServeHTTP(rec2, req2)
	assert.Equal(t, http.StatusOK, rec2.Code)
}

func TestGraphqlBodyLimitMiddleware_UsesConfiguredMaxUploadSize(t *testing.T) {
	t.Parallel()

	e := echo.New()
	e.POST("/api/graphql", func(c echo.Context) error {
		if _, err := io.Copy(io.Discard, c.Request().Body); err != nil {
			return err
		}
		return c.NoContent(http.StatusOK)
	}, graphqlBodyLimitMiddleware(maxUploadSize))

	// Bigger than the 32M non-GraphQL cap, well within maxUploadSize (10G).
	body := bytes.Repeat([]byte("a"), 33*1024*1024)
	req := httptest.NewRequest(http.MethodPost, "/api/graphql", bytes.NewReader(body))
	rec := httptest.NewRecorder()
	e.ServeHTTP(rec, req)

	assert.Equal(t, http.StatusOK, rec.Code)
}
