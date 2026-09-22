package otel

import (
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestIsLoopback(t *testing.T) {
	tests := []struct {
		endpoint string
		want     bool
	}{
		{"localhost:4317", true},
		{"127.0.0.1:4317", true},
		{"[::1]:4317", true},
		{"localhost", true},
		{"collector.example.com:4317", false},
		{"10.0.0.5:4317", false},
	}

	for _, tt := range tests {
		t.Run(tt.endpoint, func(t *testing.T) {
			assert.Equal(t, tt.want, isLoopback(tt.endpoint))
		})
	}
}
