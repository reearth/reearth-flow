// Package auth implements WebSocket per-connection token verification for
// protected mode. It is config-gated and defaults OFF.
package auth

import (
	"bytes"
	"context"
	"encoding/json"
	"net/http"
	"strconv"
	"strings"
	"time"
)

const defaultTimeout = 5 * time.Second

const verifyPath = "/auth/verify"

// Config configures the WS AuthFunc.
type Config struct {
	// Enabled gates protected mode; default OFF (no-op allow, endpoint never called).
	Enabled bool
	// AuthURL is the base URL for POST /auth/verify (plain JSON HTTP).
	AuthURL string
	// Timeout bounds the verify round-trip. <=0 ⇒ defaultTimeout.
	Timeout time.Duration
	// Client overrides the HTTP client (tests). nil ⇒ a client with Timeout.
	Client *http.Client
}

type tokenVerifyResponse struct {
	Authorized bool `json:"authorized"`
}

// verifyEndpoint joins the AuthURL base with verifyPath, trimming any trailing
// slash on the base so a configured "https://auth/" cannot produce a
// double-slash path that some routers 404.
func verifyEndpoint(authURL string) string {
	return strings.TrimRight(authURL, "/") + verifyPath
}

// NewAuthFunc returns the WS provider's AuthFunc, run before the upgrade.
// Fail-closed: when enabled, the only accept path is a 2xx {"authorized":true};
// every other outcome (empty token, transport error, timeout, non-2xx, decode
// error, authorized:false) denies.
func NewAuthFunc(cfg Config) func(*http.Request) bool {
	if !cfg.Enabled {
		return func(*http.Request) bool { return true }
	}

	timeout := cfg.Timeout
	if timeout <= 0 {
		timeout = defaultTimeout
	}
	client := cfg.Client
	if client == nil {
		client = &http.Client{Timeout: timeout}
	}
	endpoint := verifyEndpoint(cfg.AuthURL)

	return func(r *http.Request) bool {
		token := r.URL.Query().Get("token")
		if token == "" {
			return false
		}

		ctx := r.Context()
		ctx, cancel := context.WithTimeout(ctx, timeout)
		defer cancel()

		body, err := json.Marshal(map[string]string{"token": token})
		if err != nil {
			return false
		}
		req, err := http.NewRequestWithContext(ctx, http.MethodPost, endpoint, bytes.NewReader(body))
		if err != nil {
			return false
		}
		req.Header.Set("Content-Type", "application/json")
		req.Header.Set("Accept", "application/json")
		req.Header.Set("Cache-Control", "no-cache, no-store, must-revalidate")
		req.Header.Set("Pragma", "no-cache")
		req.Header.Set("X-Request-Time", strconv.FormatInt(time.Now().UnixMilli(), 10))

		resp, err := client.Do(req)
		if err != nil {
			return false
		}
		defer func() { _ = resp.Body.Close() }()

		if resp.StatusCode < 200 || resp.StatusCode >= 300 {
			return false
		}
		var vr tokenVerifyResponse
		if err := json.NewDecoder(resp.Body).Decode(&vr); err != nil {
			return false
		}
		return vr.Authorized
	}
}
