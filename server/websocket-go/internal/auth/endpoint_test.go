package auth

import (
	"strings"
	"testing"
)

// A trailing slash on AuthURL must not produce a double-slash path (e.g.
// "https://auth/" + "/auth/verify" => "https://auth//auth/verify"), which some
// routers 404 — rejecting every WS upgrade in protected mode.
func TestVerifyEndpoint_NoDoubleSlash(t *testing.T) {
	cases := []struct{ in, want string }{
		{"https://auth", "https://auth/auth/verify"},
		{"https://auth/", "https://auth/auth/verify"},
		{"https://auth//", "https://auth/auth/verify"},
		{"https://auth/base", "https://auth/base/auth/verify"},
		{"https://auth/base/", "https://auth/base/auth/verify"},
	}
	for _, c := range cases {
		got := verifyEndpoint(c.in)
		if got != c.want {
			t.Errorf("verifyEndpoint(%q) = %q, want %q", c.in, got, c.want)
		}
		if strings.Contains(strings.TrimPrefix(got, "https://"), "//") {
			t.Errorf("verifyEndpoint(%q) = %q has a double slash in the path", c.in, got)
		}
	}
}
