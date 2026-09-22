package gcs

import "testing"

// TestReadLockKeyFormat pins the read-lock key shape.
//
// This is a cross-package contract with no compiler to enforce it: the flusher
// takes this key, and a relay's last-instance election fences against it before
// deleting a document's rows. If the two drift, the election stops seeing in-flight
// flushes and deletes rows mid-read — with every test in both packages still
// passing, because each side is individually consistent.
//
// internal/pg has the mirror of this test (pg.lockKeyFor). Change one only by
// changing both.
func TestReadLockKeyFormat(t *testing.T) {
	const want = "read:lock:doc-1"
	if got := readLockName("doc-1"); got != want {
		t.Errorf("readLockName = %q, want %q; this key must match pg.lockKeyFor (see its TestReadLockKeyFormat)", got, want)
	}
}
