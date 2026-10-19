package app

import (
	"testing"
	"time"

	"github.com/reearth/reearth-flow/api/internal/app/config"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/stretchr/testify/assert"
)

func TestWorkerJobLifetime(t *testing.T) {
	tests := []struct {
		name string
		conf config.Config
		want time.Duration
	}{
		{
			name: "default six-hour limit",
			conf: config.Config{Worker_MaxRunDurationSeconds: "21600", Worker_MaxRetries: "3"},
			want: 7 * time.Hour,
		},
		{
			name: "a twelve-hour limit outlives the default",
			conf: config.Config{Worker_MaxRunDurationSeconds: "43200", Worker_MaxRetries: "3"},
			want: 13 * time.Hour,
		},
		{
			name: "spot VM retries each get the full limit",
			conf: config.Config{Worker_MaxRunDurationSeconds: "21600", Worker_MaxRetries: "3", Worker_UseSpotVMs: true},
			want: 25 * time.Hour,
		},
		{
			name: "an invalid limit falls back to the default",
			conf: config.Config{Worker_MaxRunDurationSeconds: "x"},
			want: gateway.DefaultJobLifetime,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			assert.Equal(t, tt.want, workerJobLifetime(&tt.conf))
		})
	}
}
