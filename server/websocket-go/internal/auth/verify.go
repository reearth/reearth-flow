// Package auth implements WebSocket per-connection token verification for
// protected mode. It is config-gated and defaults OFF.
package auth

import (
	"bytes"
	"context"
	"encoding/json"
	"net/http"
	"path"
	"strconv"
	"strings"
	"time"

	ygws "github.com/reearth/ygo/provider/websocket"
)

const defaultTimeout = 5 * time.Second

const verifyPath = "/auth/verify"

// Config configures the WS Authorize hook.
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
	ReadOnly   bool `json:"readOnly"`
}

// verifyEndpoint joins the AuthURL base with verifyPath, trimming any trailing
// slash on the base so a configured "https://auth/" cannot produce a
// double-slash path that some routers 404.
func verifyEndpoint(authURL string) string {
	return strings.TrimRight(authURL, "/") + verifyPath
}

// NewAuthorize returns the WS provider's Authorize hook, run before the upgrade.
// The caller presents either a user JWT (?token=) or a project's public share
// token (?shareToken=); both are checked against the room's project, and a
// share-token connection is always read-only. Fail-closed: when enabled, the
// only accept path is a 2xx {"authorized":true}; every other outcome (no
// credential, transport error, timeout, non-2xx, decode error,
// authorized:false) denies.
func NewAuthorize(cfg Config) func(*http.Request) (ygws.ConnectionConfig, bool) {
	if !cfg.Enabled {
		return func(*http.Request) (ygws.ConnectionConfig, bool) { return ygws.ConnectionConfig{}, true }
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

	deny := ygws.ConnectionConfig{}
	return func(r *http.Request) (ygws.ConnectionConfig, bool) {
		token := strings.TrimSpace(r.URL.Query().Get("token"))
		shareToken := strings.TrimSpace(r.URL.Query().Get("shareToken"))
		// A blank credential is rejected without contacting the backend.
		if token == "" && shareToken == "" {
			return deny, false
		}
		req := map[string]string{"room": path.Base(r.URL.Path)} // the room name ygo derives
		if token != "" {
			req["token"] = token
		} else {
			req["shareToken"] = shareToken
		}

		ctx := r.Context()
		ctx, cancel := context.WithTimeout(ctx, timeout)
		defer cancel()

		body, err := json.Marshal(req)
		if err != nil {
			return deny, false
		}
		hreq, err := http.NewRequestWithContext(ctx, http.MethodPost, endpoint, bytes.NewReader(body))
		if err != nil {
			return deny, false
		}
		hreq.Header.Set("Content-Type", "application/json")
		hreq.Header.Set("Accept", "application/json")
		hreq.Header.Set("Cache-Control", "no-cache, no-store, must-revalidate")
		hreq.Header.Set("Pragma", "no-cache")
		hreq.Header.Set("X-Request-Time", strconv.FormatInt(time.Now().UnixMilli(), 10))

		resp, err := client.Do(hreq)
		if err != nil {
			return deny, false
		}
		defer func() { _ = resp.Body.Close() }()

		if resp.StatusCode < 200 || resp.StatusCode >= 300 {
			return deny, false
		}
		var vr tokenVerifyResponse
		if err := json.NewDecoder(resp.Body).Decode(&vr); err != nil {
			return deny, false
		}
		if !vr.Authorized {
			return deny, false
		}
		return ygws.ConnectionConfig{ReadOnly: vr.ReadOnly || token == ""}, true
	}
}
