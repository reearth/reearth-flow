package redis

import "testing"

// TestKeyStrings asserts the exact Redis key strings the Rust server uses, so Go
// and Rust coexist on one Redis during the blue-green rollout.
func TestKeyStrings(t *testing.T) {
	const d = "proj1"
	cases := []struct {
		name string
		got  string
		want string
	}{
		{"stream", streamKey(d), "yjs:stream:proj1"},
		{"instances", instancesKey(d), "doc:instances:proj1"},
		{"lock", lockKey(d), "lock:doc:proj1"},
		{"readLock", readLockKey(d), "read:lock:proj1"},
		// GCS-save lock reproduces the Rust doubled-prefix quirk bug-for-bug.
		{"gcsLock", gcsLockKey(d), "lock:doc:gcs:lock:proj1"},
		{"oidLock", oidLockKey, "lock:oid_generation"},
	}
	for _, c := range cases {
		if c.got != c.want {
			t.Errorf("%s: got %q want %q", c.name, c.got, c.want)
		}
	}
}

func TestKeyStringsUUID(t *testing.T) {
	const d = "01234567-89ab-cdef-0123-456789abcdef"
	if got, want := streamKey(d), "yjs:stream:"+d; got != want {
		t.Errorf("stream: got %q want %q", got, want)
	}
}

func TestMessageTypeConstants(t *testing.T) {
	if msgTypeSync != "sync" {
		t.Errorf("msgTypeSync = %q, want sync", msgTypeSync)
	}
	if msgTypeAwareness != "awareness" {
		t.Errorf("msgTypeAwareness = %q, want awareness", msgTypeAwareness)
	}
}
