package auth

import (
	"context"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"net/url"
	"testing"
	"time"
)

func reqWithToken(token string) *http.Request {
	u := &url.URL{Scheme: "ws", Host: "h", Path: "/room"}
	if token != "" {
		q := u.Query()
		q.Set("token", token)
		u.RawQuery = q.Encode()
	}
	r := httptest.NewRequest("GET", u.String(), nil)
	return r
}

// TestDisabledIsNoOpAllow: default OFF ⇒ always allow, verify endpoint never called.
func TestDisabledIsNoOpAllow(t *testing.T) {
	called := false
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		called = true
		_ = json.NewEncoder(w).Encode(map[string]bool{"authorized": true})
	}))
	defer srv.Close()

	fn := NewAuthFunc(Config{Enabled: false, AuthURL: srv.URL})
	if !fn(reqWithToken("anything")) {
		t.Fatalf("disabled mode must allow")
	}
	if !fn(reqWithToken("")) {
		t.Fatalf("disabled mode must allow even empty token")
	}
	if called {
		t.Fatalf("disabled mode must NOT call /auth/verify")
	}
}

func TestEnabledAccepts(t *testing.T) {
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.Method != "POST" || r.URL.Path != "/auth/verify" {
			t.Errorf("got %s %s", r.Method, r.URL.Path)
		}
		if r.Header.Get("Content-Type") != "application/json" || r.Header.Get("Accept") != "application/json" {
			t.Errorf("missing json headers")
		}
		if r.Header.Get("X-Request-Time") == "" {
			t.Errorf("missing X-Request-Time")
		}
		var body struct {
			Token string `json:"token"`
		}
		_ = json.NewDecoder(r.Body).Decode(&body)
		if body.Token != "good-token" {
			t.Errorf("token = %q", body.Token)
		}
		_ = json.NewEncoder(w).Encode(map[string]bool{"authorized": true})
	}))
	defer srv.Close()

	fn := NewAuthFunc(Config{Enabled: true, AuthURL: srv.URL})
	if !fn(reqWithToken("good-token")) {
		t.Fatalf("expected accept")
	}
}

func TestEnabledRejectsAuthorizedFalse(t *testing.T) {
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		_ = json.NewEncoder(w).Encode(map[string]bool{"authorized": false})
	}))
	defer srv.Close()
	fn := NewAuthFunc(Config{Enabled: true, AuthURL: srv.URL})
	if fn(reqWithToken("tok")) {
		t.Fatalf("authorized:false must reject")
	}
}

func TestEnabledEmptyTokenDenies(t *testing.T) {
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		t.Errorf("must not call verify for empty token")
	}))
	defer srv.Close()
	fn := NewAuthFunc(Config{Enabled: true, AuthURL: srv.URL})
	if fn(reqWithToken("")) {
		t.Fatalf("empty token must deny")
	}
}

func TestEnabled401Denies(t *testing.T) {
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusUnauthorized)
	}))
	defer srv.Close()
	fn := NewAuthFunc(Config{Enabled: true, AuthURL: srv.URL})
	if fn(reqWithToken("tok")) {
		t.Fatalf("401 must deny")
	}
}

func TestEnabled500Denies(t *testing.T) {
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusInternalServerError)
	}))
	defer srv.Close()
	fn := NewAuthFunc(Config{Enabled: true, AuthURL: srv.URL})
	if fn(reqWithToken("tok")) {
		t.Fatalf("500 must deny")
	}
}

func TestEnabledGarbageBodyDenies(t *testing.T) {
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		_, _ = w.Write([]byte("not json"))
	}))
	defer srv.Close()
	fn := NewAuthFunc(Config{Enabled: true, AuthURL: srv.URL})
	if fn(reqWithToken("tok")) {
		t.Fatalf("garbage body must deny")
	}
}

func TestEnabledEmptyBodyDenies(t *testing.T) {
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusOK)
	}))
	defer srv.Close()
	fn := NewAuthFunc(Config{Enabled: true, AuthURL: srv.URL})
	if fn(reqWithToken("tok")) {
		t.Fatalf("empty 2xx body must deny")
	}
}

func TestEnabledTransportErrorDenies(t *testing.T) {
	fn := NewAuthFunc(Config{Enabled: true, AuthURL: "http://127.0.0.1:1"}) // nothing listening
	if fn(reqWithToken("tok")) {
		t.Fatalf("transport error must deny")
	}
}

func TestEnabledTimeoutDenies(t *testing.T) {
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		time.Sleep(500 * time.Millisecond)
		_ = json.NewEncoder(w).Encode(map[string]bool{"authorized": true})
	}))
	defer srv.Close()
	fn := NewAuthFunc(Config{Enabled: true, AuthURL: srv.URL, Timeout: 50 * time.Millisecond})
	if fn(reqWithToken("tok")) {
		t.Fatalf("timeout must deny")
	}
}

func TestRequestContextCancellationDenies(t *testing.T) {
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		time.Sleep(500 * time.Millisecond)
		_ = json.NewEncoder(w).Encode(map[string]bool{"authorized": true})
	}))
	defer srv.Close()
	fn := NewAuthFunc(Config{Enabled: true, AuthURL: srv.URL})
	ctx, cancel := context.WithCancel(context.Background())
	cancel()
	r := reqWithToken("tok").WithContext(ctx)
	if fn(r) {
		t.Fatalf("cancelled context must deny")
	}
}
