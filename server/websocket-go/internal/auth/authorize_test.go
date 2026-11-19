package auth

import (
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"net/url"
	"testing"
)

type verifyBody struct {
	Token      string `json:"token"`
	Room       string `json:"room"`
	ShareToken string `json:"shareToken"`
}

// verifyServer answers /auth/verify with resp and records the last request body.
func verifyServer(t *testing.T, resp map[string]bool) (*httptest.Server, *verifyBody) {
	t.Helper()
	got := &verifyBody{}
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		_ = json.NewDecoder(r.Body).Decode(got)
		_ = json.NewEncoder(w).Encode(resp)
	}))
	t.Cleanup(srv.Close)
	return srv, got
}

func roomReq(room string, query map[string]string) *http.Request {
	u := &url.URL{Scheme: "ws", Host: "h", Path: "/" + room}
	q := u.Query()
	for k, v := range query {
		q.Set(k, v)
	}
	u.RawQuery = q.Encode()
	return httptest.NewRequest("GET", u.String(), nil)
}

func TestAuthorizeSendsRoomAndToken(t *testing.T) {
	srv, got := verifyServer(t, map[string]bool{"authorized": true})
	fn := NewAuthorize(Config{AuthURL: srv.URL})

	cfg, ok := fn(roomReq("proj-1:main", map[string]string{"token": "tok"}))
	if !ok {
		t.Fatal("expected accept")
	}
	if cfg.ReadOnly {
		t.Fatal("a read-write verdict must not produce a read-only connection")
	}
	if got.Room != "proj-1:main" || got.Token != "tok" {
		t.Fatalf("verify body = %+v, want room proj-1:main and token tok", *got)
	}
}

func TestAuthorizeHonoursReadOnlyVerdict(t *testing.T) {
	srv, _ := verifyServer(t, map[string]bool{"authorized": true, "readOnly": true})
	fn := NewAuthorize(Config{AuthURL: srv.URL})

	cfg, ok := fn(roomReq("proj-1:main", map[string]string{"token": "tok"}))
	if !ok || !cfg.ReadOnly {
		t.Fatalf("got ok=%v readOnly=%v, want an accepted read-only connection", ok, cfg.ReadOnly)
	}
}

func TestAuthorizeShareTokenIsAlwaysReadOnly(t *testing.T) {
	// Even if the verifier forgot to say so, a share-token connection is read-only.
	srv, got := verifyServer(t, map[string]bool{"authorized": true})
	fn := NewAuthorize(Config{AuthURL: srv.URL})

	cfg, ok := fn(roomReq("proj-1:main", map[string]string{"shareToken": "s1"}))
	if !ok || !cfg.ReadOnly {
		t.Fatalf("got ok=%v readOnly=%v, want an accepted read-only connection", ok, cfg.ReadOnly)
	}
	if got.ShareToken != "s1" || got.Token != "" || got.Room != "proj-1:main" {
		t.Fatalf("verify body = %+v, want shareToken s1 and no token", *got)
	}
}

func TestAuthorizeNoCredentialDeniesWithoutCalling(t *testing.T) {
	called := false
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		called = true
		_ = json.NewEncoder(w).Encode(map[string]bool{"authorized": true})
	}))
	defer srv.Close()
	fn := NewAuthorize(Config{AuthURL: srv.URL})

	if _, ok := fn(roomReq("proj-1:main", map[string]string{"shareToken": "  "})); ok {
		t.Fatal("a request with no credential must be denied")
	}
	if called {
		t.Fatal("must not call /auth/verify without a credential")
	}
}

func TestAuthorizeForbiddenDenies(t *testing.T) {
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusForbidden)
		_ = json.NewEncoder(w).Encode(map[string]bool{"authorized": false})
	}))
	defer srv.Close()
	fn := NewAuthorize(Config{AuthURL: srv.URL})

	if _, ok := fn(roomReq("victim:main", map[string]string{"token": "tok"})); ok {
		t.Fatal("403 must deny")
	}
}

// identityServer answers /auth/verify with an accepted verdict naming userID.
func identityServer(t *testing.T, userID string) *httptest.Server {
	t.Helper()
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		_ = json.NewEncoder(w).Encode(map[string]any{"authorized": true, "userId": userID})
	}))
	t.Cleanup(srv.Close)
	return srv
}

func TestAuthorizeAttachesTheTokenUser(t *testing.T) {
	t.Parallel()

	fn := NewAuthorize(Config{AuthURL: identityServer(t, "user-1").URL})

	cfg, ok := fn(roomReq("proj-1:main", map[string]string{"token": "tok"}))
	if !ok {
		t.Fatal("expected accept")
	}
	if got := UserIDOf(cfg.Identity); got != "user-1" {
		t.Fatalf("identity user = %q, want user-1", got)
	}
}

func TestAuthorizeShareTokenCarriesNoUser(t *testing.T) {
	t.Parallel()

	// The API names no user for a share token; a user ID in the verdict anyway
	// must not be attached to a connection that presented no user token.
	fn := NewAuthorize(Config{AuthURL: identityServer(t, "user-1").URL})

	cfg, ok := fn(roomReq("proj-1:main", map[string]string{"shareToken": "s1"}))
	if !ok {
		t.Fatal("expected accept")
	}
	if cfg.Identity != nil {
		t.Fatalf("identity = %#v, want none", cfg.Identity)
	}
}

func TestAuthorizeWithoutUserIDCarriesNoIdentity(t *testing.T) {
	t.Parallel()

	fn := NewAuthorize(Config{AuthURL: identityServer(t, "").URL})

	cfg, ok := fn(roomReq("proj-1:main", map[string]string{"token": "tok"}))
	if !ok {
		t.Fatal("expected accept")
	}
	if cfg.Identity != nil {
		t.Fatalf("identity = %#v, want none", cfg.Identity)
	}
}

func TestUserIDOfIgnoresForeignIdentities(t *testing.T) {
	t.Parallel()

	if got := UserIDOf(nil); got != "" {
		t.Fatalf("UserIDOf(nil) = %q", got)
	}
	if got := UserIDOf("user-1"); got != "" {
		t.Fatalf("UserIDOf(string) = %q, want empty: only an Identity names a user", got)
	}
}
