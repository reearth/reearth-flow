package docid

import "testing"

func TestNormalize(t *testing.T) {
	cases := []struct {
		name string
		in   string
		want string
	}{
		// Strip trailing ":main" THEN trim; trailing space defeats the strip.
		{"trailing space defeats strip", "p:main ", "p:main"},
		{"strips main suffix", "p:main", "p"},
		{"bare id unchanged", "p", "p"},
		{"trims surrounding whitespace", "  p  ", "p"},
		{"empty stays empty", "", ""},
		{"only main suffix on uuid", "550e8400-e29b-41d4-a716-446655440000:main", "550e8400-e29b-41d4-a716-446655440000"},
		{"inner colon preserved", "a:b:main", "a:b"},
		{"leading space then main", " p:main", "p"},
	}
	for _, c := range cases {
		t.Run(c.name, func(t *testing.T) {
			if got := Normalize(c.in); got != c.want {
				t.Fatalf("Normalize(%q) = %q, want %q", c.in, got, c.want)
			}
		})
	}
}

func TestCanonicalEqualsNormalize(t *testing.T) {
	for _, in := range []string{"p:main", "p:main ", "  p  ", "p", "a:b:main"} {
		if Canonical(in) != Normalize(in) {
			t.Fatalf("Canonical(%q)=%q != Normalize=%q", in, Canonical(in), Normalize(in))
		}
	}
}
