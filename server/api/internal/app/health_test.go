package app

import (
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/labstack/echo/v4"
	"github.com/reearth/reearth-flow/api/internal/app/config"
	"github.com/stretchr/testify/assert"
)

func TestHealthChecker_Handler_NoAuth(t *testing.T) {
	t.Parallel()

	// Create a minimal config without auth
	conf := &config.Config{
		DB: "mongodb://localhost:27017",
	}

	// Create health checker (will skip Redis and file checks since they're nil)
	hc := NewHealthChecker(conf, "test-version", nil, nil)
	assert.NotNil(t, hc)

	// Create echo context
	e := echo.New()
	req := httptest.NewRequest(http.MethodGet, "/api/health", nil)
	rec := httptest.NewRecorder()
	c := e.NewContext(req, rec)

	// Execute handler (note: will fail if MongoDB is not running, but tests the handler flow)
	handler := hc.Handler()
	_ = handler(c) // Don't check error as MongoDB may not be running

	// The handler should at least respond
	assert.True(t, rec.Code == http.StatusOK || rec.Code == http.StatusServiceUnavailable)
}

func TestHealthChecker_Handler_WithAuth_Unauthorized(t *testing.T) {
	t.Parallel()

	// Create a config with auth
	conf := &config.Config{
		DB: "mongodb://localhost:27017",
		HealthCheck: config.HealthCheckConfig{
			Username: "admin",
			Password: "secret",
		},
	}

	hc := NewHealthChecker(conf, "test-version", nil, nil)
	assert.NotNil(t, hc)

	// Create echo context without auth
	e := echo.New()
	req := httptest.NewRequest(http.MethodGet, "/api/health", nil)
	rec := httptest.NewRecorder()
	c := e.NewContext(req, rec)

	// Execute handler
	handler := hc.Handler()
	err := handler(c)
	assert.NoError(t, err)

	// Should return unauthorized
	assert.Equal(t, http.StatusUnauthorized, rec.Code)
}

func TestHealthChecker_Handler_WithAuth_Authorized(t *testing.T) {
	t.Parallel()

	// Create a config with auth
	conf := &config.Config{
		DB: "mongodb://localhost:27017",
		HealthCheck: config.HealthCheckConfig{
			Username: "admin",
			Password: "secret",
		},
	}

	hc := NewHealthChecker(conf, "test-version", nil, nil)
	assert.NotNil(t, hc)

	// Create echo context with correct auth
	e := echo.New()
	req := httptest.NewRequest(http.MethodGet, "/api/health", nil)
	req.SetBasicAuth("admin", "secret")
	rec := httptest.NewRecorder()
	c := e.NewContext(req, rec)

	// Execute handler
	handler := hc.Handler()
	_ = handler(c)

	// Should either succeed or fail due to DB, but not due to auth
	assert.True(t, rec.Code == http.StatusOK || rec.Code == http.StatusServiceUnavailable)
}

func TestHealthChecker_Handler_WithAuth_WrongPassword(t *testing.T) {
	t.Parallel()

	// Create a config with auth
	conf := &config.Config{
		DB: "mongodb://localhost:27017",
		HealthCheck: config.HealthCheckConfig{
			Username: "admin",
			Password: "secret",
		},
	}

	hc := NewHealthChecker(conf, "test-version", nil, nil)
	assert.NotNil(t, hc)

	// Create echo context with wrong password
	e := echo.New()
	req := httptest.NewRequest(http.MethodGet, "/api/health", nil)
	req.SetBasicAuth("admin", "wrong-password")
	rec := httptest.NewRecorder()
	c := e.NewContext(req, rec)

	// Execute handler
	handler := hc.Handler()
	err := handler(c)
	assert.NoError(t, err)

	// Should return unauthorized
	assert.Equal(t, http.StatusUnauthorized, rec.Code)
}

func TestContains(t *testing.T) {
	t.Parallel()

	tests := []struct {
		name       string
		s          string
		substrings []string
		want       bool
	}{
		{
			name:       "contains one match",
			s:          "file not found",
			substrings: []string{"not found", "error"},
			want:       true,
		},
		{
			name:       "contains no match",
			s:          "connection refused",
			substrings: []string{"not found", "404"},
			want:       false,
		},
		{
			name:       "empty string",
			s:          "",
			substrings: []string{"test"},
			want:       false,
		},
		{
			name:       "empty substrings",
			s:          "test",
			substrings: []string{},
			want:       false,
		},
		{
			name:       "404 error",
			s:          "storage: returned HTTP 404",
			substrings: []string{"404"},
			want:       true,
		},
	}

	for _, tt := range tests {
		tt := tt
		t.Run(tt.name, func(t *testing.T) {
			t.Parallel()
			got := contains(tt.s, tt.substrings...)
			assert.Equal(t, tt.want, got)
		})
	}
}
