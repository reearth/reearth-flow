package main

import (
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestHealthChecker_Handler_NoAuth(t *testing.T) {
	t.Parallel()

	// Create a minimal config without auth
	conf := &Config{
		RedisURL: "redis://localhost:6379",
	}

	// Create health checker with nil clients (will skip their checks)
	hc := NewHealthChecker(conf, "test-version", nil, nil, nil)
	assert.NotNil(t, hc)

	// Create request
	req := httptest.NewRequest(http.MethodGet, "/health", nil)
	rec := httptest.NewRecorder()

	// Execute handler
	handler := hc.Handler()
	handler(rec, req)

	// Should return OK (200) since no actual checks are performed with nil clients
	assert.Equal(t, http.StatusOK, rec.Code)
}

func TestHealthChecker_Handler_WithAuth_Unauthorized(t *testing.T) {
	t.Parallel()

	// Create a config with auth
	conf := &Config{
		RedisURL:            "redis://localhost:6379",
		HealthCheckUsername: "admin",
		HealthCheckPassword: "secret",
	}

	hc := NewHealthChecker(conf, "test-version", nil, nil, nil)
	assert.NotNil(t, hc)

	// Create request without auth
	req := httptest.NewRequest(http.MethodGet, "/health", nil)
	rec := httptest.NewRecorder()

	// Execute handler
	handler := hc.Handler()
	handler(rec, req)

	// Should return unauthorized
	assert.Equal(t, http.StatusUnauthorized, rec.Code)
}

func TestHealthChecker_Handler_WithAuth_Authorized(t *testing.T) {
	t.Parallel()

	// Create a config with auth
	conf := &Config{
		RedisURL:            "redis://localhost:6379",
		HealthCheckUsername: "admin",
		HealthCheckPassword: "secret",
	}

	hc := NewHealthChecker(conf, "test-version", nil, nil, nil)
	assert.NotNil(t, hc)

	// Create request with correct auth
	req := httptest.NewRequest(http.MethodGet, "/health", nil)
	req.SetBasicAuth("admin", "secret")
	rec := httptest.NewRecorder()

	// Execute handler
	handler := hc.Handler()
	handler(rec, req)

	// Should succeed (200)
	assert.Equal(t, http.StatusOK, rec.Code)
}

func TestHealthChecker_Handler_WithAuth_WrongPassword(t *testing.T) {
	t.Parallel()

	// Create a config with auth
	conf := &Config{
		RedisURL:            "redis://localhost:6379",
		HealthCheckUsername: "admin",
		HealthCheckPassword: "secret",
	}

	hc := NewHealthChecker(conf, "test-version", nil, nil, nil)
	assert.NotNil(t, hc)

	// Create request with wrong password
	req := httptest.NewRequest(http.MethodGet, "/health", nil)
	req.SetBasicAuth("admin", "wrong-password")
	rec := httptest.NewRecorder()

	// Execute handler
	handler := hc.Handler()
	handler(rec, req)

	// Should return unauthorized
	assert.Equal(t, http.StatusUnauthorized, rec.Code)
}

func TestHealthCheckError_Error(t *testing.T) {
	t.Parallel()

	err := &HealthCheckError{
		Failures: map[string]string{
			"redis": "connection refused",
		},
	}

	assert.Equal(t, "health check failed", err.Error())
}
