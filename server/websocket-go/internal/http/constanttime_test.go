package http

import "testing"

// Functional guard for the secret comparator. The actual hardening (timing
// independence from length) is not unit-testable, but this pins that hashing
// both sides preserves correct accept/reject behavior, including wrong-length
// secrets (the case the old subtle.ConstantTimeCompare short-circuited on).
func TestConstantTimeMatch(t *testing.T) {
	cases := []struct {
		name      string
		got, want string
		ok        bool
	}{
		{"equal", "s3cr3t-value", "s3cr3t-value", true},
		{"wrong same length", "s3cr3t-value", "wr0ng-value!", false},
		{"wrong longer", "s3cr3t-value", "s3cr3t-valueX", false},
		{"wrong shorter", "s3cr3t-valu", "s3cr3t-value", false},
		{"empty got", "", "s3cr3t-value", false},
		{"empty want", "s3cr3t-value", "", false},
	}
	for _, c := range cases {
		if got := constantTimeMatch([]byte(c.got), []byte(c.want)); got != c.ok {
			t.Errorf("%s: constantTimeMatch=%v want %v", c.name, got, c.ok)
		}
	}
}
