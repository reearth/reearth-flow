package gcpbatch

import (
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
)

func TestBatchConfig_JobLifetime(t *testing.T) {
	tests := []struct {
		name string
		conf BatchConfig
		want time.Duration
	}{
		{
			name: "default six-hour limit",
			conf: BatchConfig{MaxRunDurationSeconds: 21600, MaxRetryCount: 3},
			want: 7 * time.Hour,
		},
		{
			name: "a twelve-hour limit outlives the default",
			conf: BatchConfig{MaxRunDurationSeconds: 43200, MaxRetryCount: 3},
			want: 13 * time.Hour,
		},
		{
			name: "spot VM retries each get the full limit",
			conf: BatchConfig{MaxRunDurationSeconds: 21600, MaxRetryCount: 3, UseSpotVMs: true},
			want: 25 * time.Hour,
		},
		{
			name: "no limit gets the six-hour default",
			conf: BatchConfig{MaxRunDurationSeconds: 0, MaxRetryCount: 3},
			want: 7 * time.Hour,
		},
		{
			name: "spot VM retries apply to the default limit too",
			conf: BatchConfig{MaxRunDurationSeconds: 0, MaxRetryCount: 3, UseSpotVMs: true},
			want: 25 * time.Hour,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			assert.Equal(t, tt.want, tt.conf.JobLifetime())
		})
	}
}
