package http

import (
	"crypto/subtle"
	"fmt"
	"net/http"
	"strings"
)

const apiSecretHeader = "X-API-Secret"

// APISecretConfig configures the X-API-Secret middleware.
type APISecretConfig struct {
	// Secret is the expected X-API-Secret value. Empty ⇒ allow-all in dev only.
	Secret string
	// AppEnv is REEARTH_FLOW_APP_ENV; only a dev env permits an empty secret.
	AppEnv string
}

// isDevEnv reports whether env is a development environment (the only place an
// empty API secret is allowed).
func isDevEnv(env string) bool {
	switch strings.ToLower(strings.TrimSpace(env)) {
	case "", "development", "dev", "local", "test":
		return true
	default:
		return false
	}
}

// RequireAPISecret builds middleware enforcing X-API-Secret on the wrapped
// handler (wrap /api/* only, not /health or the WS route). A set secret requires
// an exact constant-time match (else 401); empty+dev allows all. A non-dev env
// with an empty secret FAILS STARTUP (returns an error) and the returned
// middleware hard-refuses every request with 503.
func RequireAPISecret(cfg APISecretConfig) (func(http.Handler) http.Handler, error) {
	if cfg.Secret == "" && !isDevEnv(cfg.AppEnv) {
		refuse := func(next http.Handler) http.Handler {
			return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
				writeErr(w, http.StatusServiceUnavailable, "api secret not configured")
			})
		}
		return refuse, fmt.Errorf(
			"REEARTH_FLOW_API_SECRET must be set when REEARTH_FLOW_APP_ENV=%q (refusing silent allow-all)",
			cfg.AppEnv)
	}

	if cfg.Secret == "" {
		passthrough := func(next http.Handler) http.Handler { return next }
		return passthrough, nil
	}

	want := []byte(cfg.Secret)
	mw := func(next http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			got := r.Header.Get(apiSecretHeader)
			// Constant-time compare: no timing oracle.
			if subtle.ConstantTimeCompare([]byte(got), want) != 1 {
				writeErr(w, http.StatusUnauthorized, "unauthorized")
				return
			}
			next.ServeHTTP(w, r)
		})
	}
	return mw, nil
}
