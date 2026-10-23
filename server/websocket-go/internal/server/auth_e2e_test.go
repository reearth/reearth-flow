package server

import (
	"encoding/json"
	"errors"
	"net/http"
	"net/http/httptest"
	"testing"

	gws "github.com/gorilla/websocket"

	"github.com/reearth/reearth-flow/websocket-go/internal/auth"
)

// TestUpgradeIsAuthorized drives the real ygo server through the production
// Authorize hook: a connection the flow API refuses never upgrades, and one it
// accepts does. Closes coverage-map gap G3.
func TestUpgradeIsAuthorized(t *testing.T) {
	verify := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		var body struct {
			Token string `json:"token"`
		}
		_ = json.NewDecoder(r.Body).Decode(&body)
		if body.Token != "member" {
			w.WriteHeader(http.StatusForbidden)
			_ = json.NewEncoder(w).Encode(map[string]bool{"authorized": false})
			return
		}
		_ = json.NewEncoder(w).Encode(map[string]bool{"authorized": true})
	}))
	defer verify.Close()

	srv := New(testConfig())
	srv.WSProvider().Authorize = auth.NewAuthorize(auth.Config{AuthURL: verify.URL})
	ts := httptest.NewServer(srv.Handler())
	defer ts.Close()

	const room = "550e8400-e29b-41d4-a716-446655440000:main"
	for name, query := range map[string]string{
		"no credential":    "",
		"refused by flow":  "?token=outsider",
		"blank share link": "?shareToken=%20",
	} {
		t.Run(name, func(t *testing.T) {
			conn, resp, err := gws.DefaultDialer.Dial(wsURL(ts, room)+query, nil)
			if err == nil {
				_ = conn.Close()
				t.Fatal("upgrade succeeded; want it refused")
			}
			if !errors.Is(err, gws.ErrBadHandshake) || resp == nil || resp.StatusCode != http.StatusUnauthorized {
				t.Fatalf("dial err = %v, resp = %v; want a 401 bad handshake", err, resp)
			}
		})
	}

	conn, _, err := gws.DefaultDialer.Dial(wsURL(ts, room)+"?token=member", nil)
	if err != nil {
		t.Fatalf("member dial: %v", err)
	}
	_ = conn.Close()
}
