package app

import (
	"context"
	"testing"

	"github.com/99designs/gqlgen/graphql/handler/transport"
	"github.com/reearth/reearth-flow/api/internal/adapter"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestWsInitFunc(t *testing.T) {
	t.Parallel()

	tests := []struct {
		name      string
		payload   transport.InitPayload
		wantToken string
	}{
		{
			name:      "no auth in payload",
			payload:   transport.InitPayload{},
			wantToken: "",
		},
		{
			name:      "top-level Authorization key",
			payload:   transport.InitPayload{"Authorization": "Bearer top-level-token"},
			wantToken: "top-level-token",
		},
		{
			name:      "top-level authorization key (lowercase)",
			payload:   transport.InitPayload{"authorization": "Bearer lowercase-token"},
			wantToken: "lowercase-token",
		},
		{
			name: "nested headers.authorization (graphql-ws connectionParams)",
			payload: transport.InitPayload{
				"headers": map[string]any{
					"authorization": "Bearer nested-token",
				},
			},
			wantToken: "nested-token",
		},
		{
			name: "nested headers.Authorization (uppercase)",
			payload: transport.InitPayload{
				"headers": map[string]any{
					"Authorization": "Bearer nested-upper-token",
				},
			},
			wantToken: "nested-upper-token",
		},
		{
			name: "top-level takes precedence over nested headers",
			payload: transport.InitPayload{
				"Authorization": "Bearer top-token",
				"headers": map[string]any{
					"authorization": "Bearer nested-token",
				},
			},
			wantToken: "top-token",
		},
		{
			name:      "token without Bearer prefix",
			payload:   transport.InitPayload{"Authorization": "raw-token"},
			wantToken: "raw-token",
		},
		{
			name:      "bare Bearer with no token",
			payload:   transport.InitPayload{"Authorization": "Bearer "},
			wantToken: "",
		},
		{
			name:      "nil payload",
			payload:   nil,
			wantToken: "",
		},
	}

	for _, tt := range tests {
		tt := tt
		t.Run(tt.name, func(t *testing.T) {
			t.Parallel()

			ctx, payload, err := wsInitFunc(context.Background(), tt.payload)

			require.NoError(t, err)
			require.NotNil(t, payload)
			assert.Equal(t, tt.wantToken, adapter.JWT(ctx))
		})
	}
}
