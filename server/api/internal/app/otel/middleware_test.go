package otel

import (
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/labstack/echo/v4"
	"github.com/stretchr/testify/assert"
)

func TestSkipper(t *testing.T) {
	e := echo.New()
	e.GET("/", func(c echo.Context) error { return nil })
	e.GET("/api/ping", func(c echo.Context) error { return nil })
	e.GET("/api/health", func(c echo.Context) error { return nil })
	e.GET("/api/graphql", func(c echo.Context) error { return nil })

	tests := []struct {
		name      string
		path      string
		userAgent string
		want      bool
	}{
		{name: "root", path: "/", want: true},
		{name: "ping", path: "/api/ping", want: true},
		{name: "health", path: "/api/health", want: true},
		{name: "graphql", path: "/api/graphql", want: false},
		{name: "graphql with uptime UA", path: "/api/graphql", userAgent: "GoogleStackdriverMonitoring-UptimeChecks(https://cloud.google.com/monitoring)", want: true},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			req := httptest.NewRequest(http.MethodGet, tt.path, nil)
			if tt.userAgent != "" {
				req.Header.Set("User-Agent", tt.userAgent)
			}
			rec := httptest.NewRecorder()
			c := e.NewContext(req, rec)
			e.Router().Find(req.Method, tt.path, c)

			assert.Equal(t, tt.want, skipper(c))
		})
	}
}
