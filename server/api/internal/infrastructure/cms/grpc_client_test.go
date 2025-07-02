package cms

import (
	"context"
	"testing"

	"github.com/reearth/reearth-flow/api/pkg/cms"
	"github.com/reearth/reearth-flow/api/pkg/cms/proto"
	"github.com/stretchr/testify/assert"
)

func TestNewGRPCClient(t *testing.T) {
	tests := []struct {
		name     string
		endpoint string
		token    string
		userID   string
		wantErr  bool
	}{
		{
			name:     "valid endpoint",
			endpoint: "localhost:50051",
			token:    "test-token",
			userID:   "user-123",
			wantErr:  false,
		},
		{
			name:     "empty endpoint",
			endpoint: "",
			token:    "test-token",
			userID:   "user-123",
			wantErr:  true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			client, err := NewGRPCClient(tt.endpoint, tt.token, tt.userID)
			if tt.wantErr {
				assert.Error(t, err)
				assert.Nil(t, client)
			} else {
				assert.NoError(t, err)
				assert.NotNil(t, client)
				// Clean up
				if client != nil {
					gc := client.(*grpcClient)
					gc.Close()
				}
			}
		})
	}
}

func TestAddAuthMetadata(t *testing.T) {
	client := &grpcClient{
		token:  "test-token",
		userID: "user-123",
	}

	ctx := context.Background()
	newCtx := client.addAuthMetadata(ctx)

	// Verify that context has metadata
	assert.NotEqual(t, ctx, newCtx)
}

func TestConvertProtoToVisibility(t *testing.T) {
	tests := []struct {
		name  string
		input proto.Visibility
		want  cms.Visibility
	}{
		{
			name:  "public visibility",
			input: proto.Visibility_PUBLIC,
			want:  cms.VisibilityPublic,
		},
		{
			name:  "private visibility",
			input: proto.Visibility_PRIVATE,
			want:  cms.VisibilityPrivate,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result := convertProtoToVisibility(tt.input)
			assert.Equal(t, tt.want, result)
		})
	}
}
