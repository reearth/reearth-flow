package app

import (
	"bytes"
	"context"
	"fmt"
	"io"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"

	"github.com/labstack/echo/v4"
	"github.com/labstack/echo/v4/middleware"
	"github.com/reearth/reearth-flow/api/internal/app/config"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/stretchr/testify/assert"
)

// fillReader streams n bytes of filler data without materialising the whole
// body in memory, so large-body tests don't each hold ~32MiB at once.
type fillReader struct {
	n int64
}

func (r *fillReader) Read(p []byte) (int, error) {
	if r.n <= 0 {
		return 0, io.EOF
	}
	if int64(len(p)) > r.n {
		p = p[:r.n]
	}
	for i := range p {
		p[i] = 'a'
	}
	r.n -= int64(len(p))
	return len(p), nil
}

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

	req := httptest.NewRequest(http.MethodPost, "/api/signup", &fillReader{n: 32*1024*1024 + 1})
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
	req := httptest.NewRequest(http.MethodPost, "/api/graphql", &fillReader{n: 33 * 1024 * 1024})
	rec := httptest.NewRecorder()
	e.ServeHTTP(rec, req)

	assert.Equal(t, http.StatusOK, rec.Code)
}

func TestGraphqlBodyLimitMiddleware_EnforcesLimitAtReadTime(t *testing.T) {
	t.Parallel()
	const limit = 16

	// Chain the same two middlewares in the same order as newAuthMiddlewares,
	// so the asserted status is what production actually returns, not one the
	// test handler fabricates.
	e := echo.New()
	e.POST("/api/graphql", func(c echo.Context) error {
		if _, err := io.Copy(io.Discard, c.Request().Body); err != nil {
			return err
		}
		return c.NoContent(http.StatusOK)
	}, graphqlBodyLimitMiddleware(limit), gqlOpNameMiddleware())

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
	req := httptest.NewRequest(http.MethodPost, "/api/graphql", &fillReader{n: 33 * 1024 * 1024})
	rec := httptest.NewRecorder()
	e.ServeHTTP(rec, req)

	assert.Equal(t, http.StatusOK, rec.Code)
}

// TestInitEcho_EnforcesNonGraphQLBodyLimit drives a request through the real
// echo instance built by initEcho, not a hand-built one, to prove the
// BodyLimit middleware is actually installed there.
func TestInitEcho_EnforcesNonGraphQLBodyLimit(t *testing.T) {
	t.Parallel()

	cfg := &ServerConfig{
		Config: &config.Config{
			Web_Disabled: true,
			AuthSrv:      config.AuthSrvConfig{Disabled: true},
		},
		Repos:    &repo.Container{},
		Gateways: &gateway.Container{},
	}
	e := initEcho(context.Background(), cfg)

	// BodyLimit rejects based on Content-Length before reading the body, so
	// there's no need for an actual oversized payload here.
	req := httptest.NewRequest(http.MethodPost, "/api/signup", http.NoBody)
	req.ContentLength = 32*1024*1024 + 1
	rec := httptest.NewRecorder()
	e.ServeHTTP(rec, req)

	assert.Equal(t, http.StatusRequestEntityTooLarge, rec.Code)
}

// TestNewAuthMiddlewares_WrapsGraphQLBodyWithMaxBytesReader drives a request
// through the first middleware of the real slice newAuthMiddlewares wires
// into the GraphQL route, and asserts the body it hands downstream is
// actually bounded by http.MaxBytesReader (rather than only advisory).
func TestNewAuthMiddlewares_WrapsGraphQLBodyWithMaxBytesReader(t *testing.T) {
	t.Parallel()

	mws := newAuthMiddlewares(&authMiddlewaresParam{
		Cfg:     &ServerConfig{Config: &config.Config{}},
		SkipOps: map[string]struct{}{},
	})
	if len(mws) == 0 {
		t.Fatal("newAuthMiddlewares returned no middlewares")
	}

	var gotBodyType string
	probe := func(c echo.Context) error {
		gotBodyType = fmt.Sprintf("%T", c.Request().Body)
		return c.NoContent(http.StatusOK)
	}

	e := echo.New()
	req := httptest.NewRequest(http.MethodPost, "/api/graphql", strings.NewReader(`{"query":"{__typename}"}`))
	rec := httptest.NewRecorder()
	c := e.NewContext(req, rec)
	c.SetPath("/api/graphql")

	err := mws[0](probe)(c)
	assert.NoError(t, err)

	// Derive the expected type from the stdlib itself, since the concrete
	// type http.MaxBytesReader returns is unexported and could change.
	wantBodyType := fmt.Sprintf("%T", http.MaxBytesReader(httptest.NewRecorder(), http.NoBody, 1))
	assert.Equal(t, wantBodyType, gotBodyType)
}
