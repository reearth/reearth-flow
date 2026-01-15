package app

import (
	"context"
	"fmt"
	"io"
	"net/http"
	"net/url"
	"strings"
	"time"

	"github.com/hellofresh/health-go/v5"
	healthMongo "github.com/hellofresh/health-go/v5/checks/mongo"
	healthRedis "github.com/hellofresh/health-go/v5/checks/redis"
	"github.com/labstack/echo/v4"
	"github.com/reearth/reearth-flow/api/internal/app/config"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearthx/log"
)

type HealthChecker struct {
	health *health.Health
	config *config.Config
}

// Handler godoc
// @Summary      Health check endpoint
// @Description  Comprehensive health check that verifies connectivity to all dependencies (MongoDB, Redis, Auth servers, Storage)
// @Tags         health
// @Produce      json
// @Success      200  {object}  map[string]interface{}  "All health checks passed"
// @Failure      503  {object}  map[string]interface{}  "One or more health checks failed"
// @Failure      401  {object}  map[string]string       "Unauthorized (if basic auth is configured)"
// @Router       /api/health [get]
// @Security     BasicAuth
func (hc *HealthChecker) Handler() echo.HandlerFunc {
	return func(c echo.Context) error {
		// Optional HTTP Basic Auth
		if hc.config.HealthCheck.Username != "" && hc.config.HealthCheck.Password != "" {
			username, password, ok := c.Request().BasicAuth()
			if !ok || username != hc.config.HealthCheck.Username || password != hc.config.HealthCheck.Password {
				return c.JSON(http.StatusUnauthorized, map[string]string{
					"error": "unauthorized",
				})
			}
		}

		// Serve the health check
		hc.health.Handler().ServeHTTP(c.Response(), c.Request())
		return nil
	}
}

func NewHealthChecker(conf *config.Config, ver string, fileRepo gateway.File, redisGateway gateway.Redis) *HealthChecker {
	checks := []health.Config{
		{
			Name:      "db",
			Timeout:   time.Second * 5,
			SkipOnErr: false,
			Check:     healthMongo.New(healthMongo.Config{DSN: conf.DB}),
		},
	}

	// Add checks for additional DB users
	for _, u := range conf.DB_Users {
		checks = append(checks, health.Config{
			Name:      "db-" + u.Name,
			Timeout:   time.Second * 5,
			SkipOnErr: false,
			Check:     healthMongo.New(healthMongo.Config{DSN: u.URI}),
		})
	}

	// Redis check
	if conf.Redis_URL != "" {
		checks = append(checks, health.Config{
			Name:      "redis",
			Timeout:   time.Second * 5,
			SkipOnErr: false,
			Check:     healthRedis.New(healthRedis.Config{DSN: conf.Redis_URL}),
		})
	}

	// File storage check (GCS/local)
	if fileRepo != nil {
		checks = append(checks, health.Config{
			Name:      "storage",
			Timeout:   time.Second * 30,
			SkipOnErr: false,
			Check:     createStorageCheck(fileRepo),
		})
	}

	// Auth server checks
	for _, a := range conf.Auths() {
		if a.ISS != "" {
			u, err := url.Parse(a.ISS)
			if err != nil {
				log.Warnf("health: invalid issuer URL: %v", err)
				continue
			}
			issuerURL := u.JoinPath(".well-known/openid-configuration").String()
			checks = append(checks, health.Config{
				Name:      "auth:" + a.ISS,
				Timeout:   time.Second * 10,
				SkipOnErr: false,
				Check: func(ctx context.Context) error {
					return authServerPingCheck(issuerURL)
				},
			})
		}
	}

	h, err := health.New(health.WithComponent(health.Component{
		Name:    "reearth-flow",
		Version: ver,
	}), health.WithChecks(checks...))
	if err != nil {
		log.Fatalf("failed to create health check: %v", err)
	}

	return &HealthChecker{
		health: h,
		config: conf,
	}
}

func (hc *HealthChecker) Check(ctx context.Context) error {
	log.Infof("health check: running initial health checks...")
	result := hc.health.Measure(ctx)
	if len(result.Failures) > 0 {
		return fmt.Errorf("initial health check failed: %v", result.Failures)
	}
	log.Infof("health check: all checks passed")
	return nil
}

func authServerPingCheck(issuerURL string) (checkErr error) {
	client := http.Client{
		Timeout: 5 * time.Second,
	}
	resp, err := client.Get(issuerURL)
	if err != nil {
		return fmt.Errorf("auth server unreachable: %v", err)
	}
	defer func(Body io.ReadCloser) {
		err := Body.Close()
		if err != nil {
			checkErr = fmt.Errorf("failed to close response body: %v", err)
		}
	}(resp.Body)

	if resp.StatusCode < 200 || resp.StatusCode >= 300 {
		return fmt.Errorf("auth server unhealthy, status: %d", resp.StatusCode)
	}
	return nil
}

// createStorageCheck creates a health check function for file storage
// This is a simple connectivity check - we verify the storage is accessible
func createStorageCheck(fileRepo gateway.File) func(ctx context.Context) error {
	return func(ctx context.Context) error {
		// Try to read a non-existent file to verify connectivity
		// The storage should return a "not found" error rather than a connection error
		_, err := fileRepo.ReadAsset(ctx, "__health_check_test__")
		if err != nil {
			// Check if it's a "not found" error (expected) vs a connectivity error
			errStr := err.Error()
			// These are expected "not found" type errors - storage is working
			if contains(errStr, "not found", "no such file", "does not exist", "404", "NoSuchKey", "storage: object doesn't exist") {
				return nil
			}
			// Any other error indicates a problem with storage connectivity
			return fmt.Errorf("storage connectivity error: %v", err)
		}
		// If no error, file somehow exists (unlikely) - storage is working
		return nil
	}
}

func contains(s string, substrings ...string) bool {
	for _, sub := range substrings {
		if strings.Contains(s, sub) {
			return true
		}
	}
	return false
}
