package otel

import (
	"slices"
	"strings"

	"github.com/labstack/echo/v4"
	"go.opentelemetry.io/contrib/instrumentation/github.com/labstack/echo/otelecho"
)

var skipPaths = []string{"/", "/api/ping", "/api/health"}

func skipper(c echo.Context) bool {
	if slices.Contains(skipPaths, c.Path()) {
		return true
	}
	return strings.Contains(c.Request().UserAgent(), "GoogleStackdriverMonitoring")
}

// Middleware returns an Echo middleware that starts a span per request,
// skipping health/uptime-check traffic.
func Middleware(serviceName string) echo.MiddlewareFunc {
	return otelecho.Middleware(serviceName, otelecho.WithSkipper(skipper))
}
