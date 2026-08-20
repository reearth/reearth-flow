// Package docid normalizes document identifiers to their canonical form,
// collapsing the WS room "{projectId}:main" and the HTTP "{projectId}" to the
// same key. Only a trailing ":main" is stripped; ids are otherwise opaque.
package docid

import "strings"

const mainSuffix = ":main"

// Normalize strips a trailing ":main" THEN trims whitespace. Order matters: a
// trailing space defeats the strip ("p:main " ⇒ "p:main").
func Normalize(s string) string {
	s = strings.TrimSuffix(s, mainSuffix)
	return strings.TrimSpace(s)
}

// Canonical returns the canonical doc id for s; an alias for Normalize.
func Canonical(s string) string {
	return Normalize(s)
}
