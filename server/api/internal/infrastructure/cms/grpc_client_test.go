package cms

import (
	"context"
	"testing"
	"time"

	"github.com/reearth/reearth-flow/api/pkg/cms"
	"github.com/reearth/reearth-flow/api/pkg/cms/proto"
	"github.com/stretchr/testify/assert"
	"google.golang.org/protobuf/types/known/anypb"
	"google.golang.org/protobuf/types/known/structpb"
	"google.golang.org/protobuf/types/known/timestamppb"
	"google.golang.org/protobuf/types/known/wrapperspb"
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

func TestConvertAnyToInterface(t *testing.T) {
	tests := []struct {
		name     string
		setup    func() *anypb.Any
		expected interface{}
	}{
		{
			name: "nil input",
			setup: func() *anypb.Any {
				return nil
			},
			expected: nil,
		},
		{
			name: "string value",
			setup: func() *anypb.Any {
				sv := &wrapperspb.StringValue{Value: "test string"}
				any, _ := anypb.New(sv)
				return any
			},
			expected: "test string",
		},
		{
			name: "int32 value",
			setup: func() *anypb.Any {
				iv := &wrapperspb.Int32Value{Value: 42}
				any, _ := anypb.New(iv)
				return any
			},
			expected: int32(42),
		},
		{
			name: "bool value",
			setup: func() *anypb.Any {
				bv := &wrapperspb.BoolValue{Value: true}
				any, _ := anypb.New(bv)
				return any
			},
			expected: true,
		},
		{
			name: "timestamp value",
			setup: func() *anypb.Any {
				ts := timestamppb.Now()
				any, _ := anypb.New(ts)
				return any
			},
			expected: nil,
		},
		{
			name: "struct value",
			setup: func() *anypb.Any {
				s, _ := structpb.NewStruct(map[string]interface{}{
					"key": "value",
					"num": 123.0,
				})
				any, _ := anypb.New(s)
				return any
			},
			expected: map[string]interface{}{
				"key": "value",
				"num": 123.0,
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			input := tt.setup()
			result := convertAnyToInterface(input)

			// Special handling for timestamp test
			if tt.name == "timestamp value" {
				assert.IsType(t, time.Time{}, result)
			} else {
				assert.Equal(t, tt.expected, result)
			}
		})
	}
}
