package pg

import "testing"

// TestReadLockKeyFormat pins the read-lock key shape.
//
// This is a cross-package contract with no compiler to enforce it: the flusher
// builds the same key via gcs.readLockName, and the election in safeDeleteSQL
// fences against whatever the flusher wrote. If the two drift, the election stops
// seeing in-flight flushes and will delete a document's rows mid-read — with every
// test in both packages still passing, because each side is individually
// consistent.
//
// gcs has the mirror of this test. Change one only by changing both.
func TestReadLockKeyFormat(t *testing.T) {
	const want = "read:lock:doc-1"
	if got := lockKeyFor("doc-1"); got != want {
		t.Errorf("lockKeyFor = %q, want %q; this key must match gcs.readLockName (see its TestReadLockKeyFormat)", got, want)
	}
}
