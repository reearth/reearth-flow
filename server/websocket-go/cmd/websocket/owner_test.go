package main

import (
	"fmt"
	"os"
	"testing"
)

// The lock-owner token must be unique per instance even when two instances
// share a PID (e.g. PID 1 in separate containers). A PID-derived token would
// let one instance release another instance's Redis/GCS locks (OID allocation,
// read-lock, prune lock), breaking cross-instance safety. Guard: two calls must
// differ, be non-empty, and must not equal a PID-only derivation.
func TestNewInstanceOwner_UniquePerInstance(t *testing.T) {
	a, err := newInstanceOwner()
	if err != nil {
		t.Fatalf("newInstanceOwner: %v", err)
	}
	b, err := newInstanceOwner()
	if err != nil {
		t.Fatalf("newInstanceOwner: %v", err)
	}
	if a == "" {
		t.Fatal("owner token must be non-empty")
	}
	if a == b {
		t.Fatalf("owner tokens must differ across instances; both = %q (PID-derived?)", a)
	}
	if a == fmt.Sprintf("instance-%d", os.Getpid()) {
		t.Fatalf("owner token must not be PID-derived: %q", a)
	}
}
