package app

import (
	"bytes"
	"context"
	"errors"
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

// countingReader records how many bytes have been read from r.
type countingReader struct {
	r io.Reader
	n int
}

func (r *countingReader) Read(p []byte) (int, error) {
	n, err := r.r.Read(p)
	r.n += n
	return n, err
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

func TestGraphqlBodyLimitMiddleware(t *testing.T) {
	t.Parallel()

	const (
		multipart = "multipart/form-data; boundary=xyz"
		// Small limits keep the boundary cases cheap; the production cases
		// below use the real constants.
		smallUpload = 64
		smallJSON   = 16
	)

	tests := []struct {
		name        string
		method      string
		path        string
		contentType string
		size        int64
		uploadLimit int64
		jsonLimit   int64
		want        int
	}{
		{name: "json under limit", contentType: echo.MIMEApplicationJSON, size: smallJSON - 1, want: http.StatusOK},
		{name: "json at limit", contentType: echo.MIMEApplicationJSON, size: smallJSON, want: http.StatusOK},
		{name: "json over limit", contentType: echo.MIMEApplicationJSON, size: smallJSON + 1, want: http.StatusRequestEntityTooLarge},
		{name: "json with charset over limit", contentType: echo.MIMEApplicationJSONCharsetUTF8, size: smallJSON + 1, want: http.StatusRequestEntityTooLarge},
		{name: "no content type uses json limit", size: smallJSON + 1, want: http.StatusRequestEntityTooLarge},
		{name: "unparsable content type uses json limit", contentType: "multipart/form-data; boundary", size: smallJSON + 1, want: http.StatusRequestEntityTooLarge},
		{name: "other multipart type uses json limit", contentType: "multipart/mixed; boundary=xyz", size: smallJSON + 1, want: http.StatusRequestEntityTooLarge},
		{name: "multipart over json limit", contentType: multipart, size: smallJSON + 1, want: http.StatusOK},
		{name: "multipart at upload limit", contentType: multipart, size: smallUpload, want: http.StatusOK},
		{name: "multipart with uppercase type", contentType: "Multipart/Form-Data; boundary=xyz", size: smallUpload, want: http.StatusOK},
		{name: "multipart over upload limit", contentType: multipart, size: smallUpload + 1, want: http.StatusRequestEntityTooLarge},
		{name: "other path is not limited", path: "/api/other", contentType: echo.MIMEApplicationJSON, size: smallUpload + 1, want: http.StatusOK},
		{name: "non-POST is not limited", method: http.MethodPut, contentType: echo.MIMEApplicationJSON, size: smallUpload + 1, want: http.StatusOK},
		{
			name:        "production: 30MB json is accepted",
			contentType: echo.MIMEApplicationJSON,
			size:        30 * 1024 * 1024,
			uploadLimit: maxUploadSize,
			jsonLimit:   maxJSONBodySize,
			want:        http.StatusOK,
		},
		{
			name:        "production: json over maxJSONBodySize is rejected",
			contentType: echo.MIMEApplicationJSON,
			size:        maxJSONBodySize + 1,
			uploadLimit: maxUploadSize,
			jsonLimit:   maxJSONBodySize,
			want:        http.StatusRequestEntityTooLarge,
		},
		{
			name:        "production: multipart over maxJSONBodySize is accepted",
			contentType: multipart,
			size:        maxJSONBodySize + 1024,
			uploadLimit: maxUploadSize,
			jsonLimit:   maxJSONBodySize,
			want:        http.StatusOK,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			t.Parallel()

			method, path := tt.method, tt.path
			if method == "" {
				method = http.MethodPost
			}
			if path == "" {
				path = "/api/graphql"
			}
			uploadLimit, jsonLimit := tt.uploadLimit, tt.jsonLimit
			if uploadLimit == 0 {
				uploadLimit, jsonLimit = smallUpload, smallJSON
			}

			// Chain the same two middlewares in the same order as
			// newAuthMiddlewares. Non-multipart bodies are rejected with 413
			// by gqlOpNameMiddleware itself; multipart bodies reach the
			// handler unread (gqlgen in production), so the handler reports a
			// capped read as 413 to show the limit is enforced there too.
			e := echo.New()
			e.Add(method, path, func(c echo.Context) error {
				if _, err := io.Copy(io.Discard, c.Request().Body); err != nil {
					var tooLarge *http.MaxBytesError
					if errors.As(err, &tooLarge) {
						return echo.ErrStatusRequestEntityTooLarge
					}
					return err
				}
				return c.NoContent(http.StatusOK)
			}, graphqlBodyLimitMiddleware(uploadLimit, jsonLimit), gqlOpNameMiddleware())

			req := httptest.NewRequest(method, path, &fillReader{n: tt.size})
			if tt.contentType != "" {
				req.Header.Set(echo.HeaderContentType, tt.contentType)
			}
			rec := httptest.NewRecorder()
			e.ServeHTTP(rec, req)

			assert.Equal(t, tt.want, rec.Code)
		})
	}
}

func TestGqlOpNameMiddleware(t *testing.T) {
	t.Parallel()

	tests := []struct {
		name        string
		contentType string
		body        string
		wantOp      string
		// wantBuffered is whether the middleware consumed the original
		// stream (and replaced it with an in-memory copy).
		wantBuffered bool
	}{
		{
			name:         "json with operationName",
			contentType:  echo.MIMEApplicationJSON,
			body:         `{"operationName":"Signup","query":"mutation Signup { __typename }"}`,
			wantOp:       "Signup",
			wantBuffered: true,
		},
		{
			name:         "json without operationName",
			contentType:  echo.MIMEApplicationJSON,
			body:         `{"query":"{ __typename }"}`,
			wantBuffered: true,
		},
		{
			name:         "invalid json",
			contentType:  echo.MIMEApplicationJSON,
			body:         `not json`,
			wantBuffered: true,
		},
		{
			name:        "multipart is not buffered",
			contentType: "multipart/form-data; boundary=xyz",
			body:        `{"operationName":"Signup"}`,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			t.Parallel()

			src := &countingReader{r: strings.NewReader(tt.body)}
			var gotOp, gotBody string
			var readBeforeHandler int
			e := echo.New()
			e.POST("/api/graphql", func(c echo.Context) error {
				readBeforeHandler = src.n
				gotOp = adapter.GQLOperationName(c.Request().Context())
				b, err := io.ReadAll(c.Request().Body)
				if err != nil {
					return err
				}
				gotBody = string(b)
				return c.NoContent(http.StatusOK)
			}, gqlOpNameMiddleware())

			req := httptest.NewRequest(http.MethodPost, "/api/graphql", src)
			req.Header.Set(echo.HeaderContentType, tt.contentType)
			rec := httptest.NewRecorder()
			e.ServeHTTP(rec, req)

			assert.Equal(t, http.StatusOK, rec.Code)
			assert.Equal(t, tt.wantOp, gotOp)
			// The handler must always see the full, unchanged body.
			assert.Equal(t, tt.body, gotBody)
			if tt.wantBuffered {
				assert.Equal(t, len(tt.body), readBeforeHandler)
			} else {
				assert.Zero(t, readBeforeHandler)
			}
		})
	}
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
