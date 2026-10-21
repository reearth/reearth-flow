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
	"github.com/reearth/reearth-accounts/server/pkg/gqlclient"
	"github.com/reearth/reearth-flow/api/internal/adapter"
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
	}, graphqlBodyLimitMiddleware(maxUploadSize, limit), gqlOpNameMiddleware())

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

func TestGraphqlBodyLimitMiddleware_MultipartUsesMaxUploadSize(t *testing.T) {
	t.Parallel()

	e := echo.New()
	e.POST("/api/graphql", func(c echo.Context) error {
		if _, err := io.Copy(io.Discard, c.Request().Body); err != nil {
			return err
		}
		return c.NoContent(http.StatusOK)
	}, graphqlBodyLimitMiddleware(maxUploadSize, maxJSONBodySize), gqlOpNameMiddleware())

	// Bigger than maxJSONBodySize, well within maxUploadSize (10G).
	req := httptest.NewRequest(http.MethodPost, "/api/graphql", &fillReader{n: maxJSONBodySize + 1024})
	req.Header.Set(echo.HeaderContentType, "multipart/form-data; boundary=xyz")
	rec := httptest.NewRecorder()
	e.ServeHTTP(rec, req)

	assert.Equal(t, http.StatusOK, rec.Code)
}

func TestGraphqlBodyLimitMiddleware_NonMultipartUsesMaxJSONBodySize(t *testing.T) {
	t.Parallel()

	e := echo.New()
	e.POST("/api/graphql", func(c echo.Context) error {
		return c.NoContent(http.StatusOK)
	}, graphqlBodyLimitMiddleware(maxUploadSize, maxJSONBodySize), gqlOpNameMiddleware())

	req := httptest.NewRequest(http.MethodPost, "/api/graphql", &fillReader{n: maxJSONBodySize + 1})
	req.Header.Set(echo.HeaderContentType, echo.MIMEApplicationJSON)
	rec := httptest.NewRecorder()
	e.ServeHTTP(rec, req)

	assert.Equal(t, http.StatusRequestEntityTooLarge, rec.Code)
}

func TestGqlOpNameMiddleware_DoesNotBufferMultipart(t *testing.T) {
	t.Parallel()

	body := &fillReader{n: 1024}
	e := echo.New()
	e.POST("/api/graphql", func(c echo.Context) error {
		return c.NoContent(http.StatusOK)
	}, gqlOpNameMiddleware())

	req := httptest.NewRequest(http.MethodPost, "/api/graphql", body)
	req.Header.Set(echo.HeaderContentType, "multipart/form-data; boundary=xyz")
	rec := httptest.NewRecorder()
	e.ServeHTTP(rec, req)

	assert.Equal(t, http.StatusOK, rec.Code)
	// Nothing was read, so the handler sees the original stream untouched.
	assert.Equal(t, int64(1024), body.n)
}

func TestGqlOpNameMiddleware_AttachesOperationNameForJSON(t *testing.T) {
	t.Parallel()

	var gotOp, gotBody string
	e := echo.New()
	e.POST("/api/graphql", func(c echo.Context) error {
		gotOp = adapter.GQLOperationName(c.Request().Context())
		b, err := io.ReadAll(c.Request().Body)
		if err != nil {
			return err
		}
		gotBody = string(b)
		return c.NoContent(http.StatusOK)
	}, gqlOpNameMiddleware())

	payload := `{"operationName":"Signup","query":"mutation Signup { __typename }"}`
	req := httptest.NewRequest(http.MethodPost, "/api/graphql", strings.NewReader(payload))
	req.Header.Set(echo.HeaderContentType, echo.MIMEApplicationJSON)
	rec := httptest.NewRecorder()
	e.ServeHTTP(rec, req)

	assert.Equal(t, http.StatusOK, rec.Code)
	assert.Equal(t, "Signup", gotOp)
	assert.Equal(t, payload, gotBody)
}

// TestInitEcho_EnforcesNonGraphQLBodyLimit drives a request through the real
// echo instance built by initEcho, not a hand-built one, to prove the
// BodyLimit middleware is actually installed there.
func TestInitEcho_EnforcesNonGraphQLBodyLimit(t *testing.T) {
	t.Parallel()

	// initEcho builds the usecase container at boot, so the client it reads
	// repos off must be present even though no request here reaches it.
	cfg := &ServerConfig{
		Config: &config.Config{
			Web_Disabled: true,
			AuthSrv:      config.AuthSrvConfig{Disabled: true},
		},
		Repos:            &repo.Container{},
		Gateways:         &gateway.Container{},
		AccountGQLClient: gqlclient.NewClient("http://localhost", 1, nil),
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
