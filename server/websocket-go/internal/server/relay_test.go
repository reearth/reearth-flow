package server

import (
	"context"
	"testing"

	"github.com/alicebob/miniredis/v2"

	"github.com/reearth/reearth-flow/websocket-go/internal/config"
	redisrelay "github.com/reearth/reearth-flow/websocket-go/internal/redis"
)

// TestAttachRedisRelay wires a Redis relay onto the server's ygo provider and
// verifies a second relay is rejected.
func TestAttachRedisRelay(t *testing.T) {
	mr, err := miniredis.Run()
	if err != nil {
		t.Fatalf("miniredis: %v", err)
	}
	defer mr.Close()

	cfg := &config.Config{Origins: []string{"http://localhost:3000"}}
	srv := New(cfg)

	relay, err := redisrelay.New(redisrelay.Options{Addr: mr.Addr()})
	if err != nil {
		t.Fatalf("relay.New: %v", err)
	}
	defer relay.Close()

	if err := srv.AttachRedisRelay(context.Background(), relay); err != nil {
		t.Fatalf("AttachRedisRelay: %v", err)
	}

	// A second relay must be rejected.
	relay2, err := redisrelay.New(redisrelay.Options{Addr: mr.Addr()})
	if err != nil {
		t.Fatalf("relay2.New: %v", err)
	}
	defer relay2.Close()
	if err := srv.AttachRedisRelay(context.Background(), relay2); err == nil {
		t.Fatal("attaching a second relay returned nil, want already-attached error")
	}
}

func TestAttachRedisRelayNil(t *testing.T) {
	srv := New(&config.Config{})
	if err := srv.AttachRedisRelay(context.Background(), nil); err == nil {
		t.Fatal("AttachRedisRelay(nil) returned nil, want error")
	}
}
