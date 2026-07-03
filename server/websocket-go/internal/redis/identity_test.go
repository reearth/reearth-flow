package redis

import "testing"

func TestNewInstanceIDNonZeroAndDistinct(t *testing.T) {
	a, err := newInstanceID()
	if err != nil {
		t.Fatalf("newInstanceID: %v", err)
	}
	if a == 0 {
		t.Fatal("instance id is zero")
	}
	// Two draws are overwhelmingly likely to differ (64-bit crypto/rand).
	b, _ := newInstanceID()
	if a == b {
		t.Fatal("two crypto/rand instance ids collided")
	}
}
