package http

import (
	"net/http"
	"net/http/httptest"
	"testing"
)

func okHandler() http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusOK)
		_, _ = w.Write([]byte("ok"))
	})
}

func TestAPISecretAllowAllWhenUnset(t *testing.T) {
	mw, err := RequireAPISecret(APISecretConfig{Secret: "", AppEnv: "development"})
	if err != nil {
		t.Fatalf("build: %v", err)
	}
	h := mw(okHandler())
	req := httptest.NewRequest("GET", "/api/document/x", nil)
	rec := httptest.NewRecorder()
	h.ServeHTTP(rec, req)
	if rec.Code != 200 {
		t.Fatalf("status = %d, want 200 (allow-all when unset)", rec.Code)
	}
}

func TestAPISecretRejectsMissingHeader(t *testing.T) {
	mw, _ := RequireAPISecret(APISecretConfig{Secret: "topsecret", AppEnv: "development"})
	h := mw(okHandler())
	req := httptest.NewRequest("GET", "/api/document/x", nil)
	rec := httptest.NewRecorder()
	h.ServeHTTP(rec, req)
	if rec.Code != 401 {
		t.Fatalf("status = %d, want 401", rec.Code)
	}
}

func TestAPISecretRejectsMismatch(t *testing.T) {
	mw, _ := RequireAPISecret(APISecretConfig{Secret: "topsecret", AppEnv: "development"})
	h := mw(okHandler())
	req := httptest.NewRequest("GET", "/api/document/x", nil)
	req.Header.Set("X-API-Secret", "wrong")
	rec := httptest.NewRecorder()
	h.ServeHTTP(rec, req)
	if rec.Code != 401 {
		t.Fatalf("status = %d, want 401", rec.Code)
	}
}

func TestAPISecretAcceptsExactMatch(t *testing.T) {
	mw, _ := RequireAPISecret(APISecretConfig{Secret: "topsecret", AppEnv: "development"})
	h := mw(okHandler())
	req := httptest.NewRequest("GET", "/api/document/x", nil)
	req.Header.Set("X-API-Secret", "topsecret")
	rec := httptest.NewRecorder()
	h.ServeHTTP(rec, req)
	if rec.Code != 200 {
		t.Fatalf("status = %d, want 200", rec.Code)
	}
}

// TestAPISecretProdGuardFailsStartup: a non-dev env with an empty secret must
// not allow-all — either a build error or a middleware that refuses /api/* (503).
func TestAPISecretProdGuardFailsStartup(t *testing.T) {
	for _, env := range []string{"production", "prod", "staging"} {
		mw, err := RequireAPISecret(APISecretConfig{Secret: "", AppEnv: env})
		if err == nil {
			h := mw(okHandler())
			req := httptest.NewRequest("GET", "/api/document/x", nil)
			rec := httptest.NewRecorder()
			h.ServeHTTP(rec, req)
			if rec.Code == 200 {
				t.Fatalf("env=%s: empty secret silently allowed /api/* (status 200)", env)
			}
			if rec.Code != http.StatusServiceUnavailable {
				t.Fatalf("env=%s: expected build error or 503, got %d", env, rec.Code)
			}
		}
	}
}

func TestAPISecretDevAllowsEmpty(t *testing.T) {
	if _, err := RequireAPISecret(APISecretConfig{Secret: "", AppEnv: "development"}); err != nil {
		t.Fatalf("dev empty secret should not fail: %v", err)
	}
}
