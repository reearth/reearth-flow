package main

import (
	"os/exec"
	"testing"
)

// TestBuilds is a smoke test asserting the whole module compiles, including
// this main package and its real Redis/GCS health adapters.
func TestBuilds(t *testing.T) {
	cmd := exec.Command("go", "build", "./...")
	cmd.Dir = "../.." // module root relative to cmd/websocket
	if out, err := cmd.CombinedOutput(); err != nil {
		t.Fatalf("go build ./... failed: %v\n%s", err, out)
	}
}
