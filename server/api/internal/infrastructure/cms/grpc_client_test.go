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

func TestConvertProtoToProject(t *testing.T) {
	now := time.Now()
	protoProject := &proto.Project{
		Id:          "project-123",
		Name:        "Test Project",
		Alias:       "test-project",
		Description: stringPtr("Test description"),
		License:     stringPtr("MIT"),
		Readme:      stringPtr("Test readme"),
		WorkspaceId: "workspace-123",
		Visibility:  proto.Visibility_PUBLIC,
		CreatedAt:   timestamppb.New(now),
		UpdatedAt:   timestamppb.New(now),
	}

	expected := &cms.Project{
		ID:          "project-123",
		Name:        "Test Project",
		Alias:       "test-project",
		Description: stringPtr("Test description"),
		License:     stringPtr("MIT"),
		Readme:      stringPtr("Test readme"),
		WorkspaceID: "workspace-123",
		Visibility:  cms.VisibilityPublic,
		CreatedAt:   now,
		UpdatedAt:   now,
	}

	result := convertProtoToProject(protoProject)
	assert.Equal(t, expected.ID, result.ID)
	assert.Equal(t, expected.Name, result.Name)
	assert.Equal(t, expected.Alias, result.Alias)
	assert.Equal(t, expected.WorkspaceID, result.WorkspaceID)
	assert.Equal(t, expected.Visibility, result.Visibility)

	// Test nil input
	result = convertProtoToProject(nil)
	assert.Nil(t, result)
}

func TestConvertProtoToModel(t *testing.T) {
	now := time.Now()
	protoModel := &proto.Model{
		Id:          "model-123",
		ProjectId:   "project-123",
		Name:        "Test Model",
		Description: "Test model description",
		Key:         "test-model",
		Schema: &proto.Schema{
			SchemaId: "schema-123",
			Fields: []*proto.SchemaField{
				{
					FieldId:     "field-1",
					Name:        "Title",
					Type:        proto.SchemaFieldType_Text,
					Key:         "title",
					Description: stringPtr("Title field"),
				},
			},
		},
		PublicApiEp: "https://api.example.com",
		EditorUrl:   "https://editor.example.com",
		CreatedAt:   timestamppb.New(now),
		UpdatedAt:   timestamppb.New(now),
	}

	result := convertProtoToModel(protoModel)
	assert.Equal(t, "model-123", result.ID)
	assert.Equal(t, "project-123", result.ProjectID)
	assert.Equal(t, "Test Model", result.Name)
	assert.Equal(t, "Test model description", result.Description)
	assert.Equal(t, "test-model", result.Key)
	assert.Equal(t, "https://api.example.com", result.PublicAPIEP)
	assert.Equal(t, "https://editor.example.com", result.EditorURL)

	// Test schema conversion
	assert.Equal(t, "schema-123", result.Schema.SchemaID)
	assert.Len(t, result.Schema.Fields, 1)
	assert.Equal(t, "field-1", result.Schema.Fields[0].FieldID)
	assert.Equal(t, "Title", result.Schema.Fields[0].Name)
	assert.Equal(t, cms.SchemaFieldTypeText, result.Schema.Fields[0].Type)

	// Test nil input
	result = convertProtoToModel(nil)
	assert.Nil(t, result)
}

func TestConvertProtoToItem(t *testing.T) {
	now := time.Now()

	// Create test fields
	stringField, _ := anypb.New(&wrapperspb.StringValue{Value: "test value"})
	intField, _ := anypb.New(&wrapperspb.Int32Value{Value: 42})

	protoItem := &proto.Item{
		Id: "item-123",
		Fields: map[string]*anypb.Any{
			"title":  stringField,
			"number": intField,
		},
		CreatedAt: timestamppb.New(now),
		UpdatedAt: timestamppb.New(now),
	}

	result := convertProtoToItem(protoItem)
	assert.Equal(t, "item-123", result.ID)
	assert.Equal(t, "test value", result.Fields["title"])
	assert.Equal(t, int32(42), result.Fields["number"])

	// Test nil input
	result = convertProtoToItem(nil)
	assert.Nil(t, result)
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
			expected: nil, // Will be checked separately as time comparison needs tolerance
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

// Test helper functions for new write operations

func TestGRPCClient_WriteOperations_InputValidation(t *testing.T) {
	client := &grpcClient{
		client: nil, // Nil client to test validation
		token:  "test-token",
		userID: "user-123",
	}

	ctx := context.Background()

	t.Run("CreateProject with nil client", func(t *testing.T) {
		input := cms.CreateProjectInput{
			WorkspaceID: "workspace-123",
			Name:        "Test Project",
			Alias:       "test-project",
			Visibility:  cms.VisibilityPublic,
		}

		result, err := client.CreateProject(ctx, input)
		assert.Error(t, err)
		assert.Nil(t, result)
		assert.Contains(t, err.Error(), "gRPC client not initialized")
	})

	t.Run("UpdateProject with nil client", func(t *testing.T) {
		input := cms.UpdateProjectInput{
			ProjectID: "project-123",
			Name:      stringPtr("Updated Name"),
		}

		result, err := client.UpdateProject(ctx, input)
		assert.Error(t, err)
		assert.Nil(t, result)
		assert.Contains(t, err.Error(), "gRPC client not initialized")
	})

	t.Run("DeleteProject with nil client", func(t *testing.T) {
		input := cms.DeleteProjectInput{
			ProjectID: "project-123",
		}

		result, err := client.DeleteProject(ctx, input)
		assert.Error(t, err)
		assert.Nil(t, result)
		assert.Contains(t, err.Error(), "gRPC client not initialized")
	})

	t.Run("CheckAliasAvailability with nil client", func(t *testing.T) {
		input := cms.CheckAliasAvailabilityInput{
			Alias: "test-alias",
		}

		result, err := client.CheckAliasAvailability(ctx, input)
		assert.Error(t, err)
		assert.Nil(t, result)
		assert.Contains(t, err.Error(), "gRPC client not initialized")
	})
}

func TestVisibilityConversion(t *testing.T) {
	tests := []struct {
		name            string
		cmsVisibility   cms.Visibility
		protoVisibility proto.Visibility
	}{
		{
			name:            "public visibility",
			cmsVisibility:   cms.VisibilityPublic,
			protoVisibility: proto.Visibility_PUBLIC,
		},
		{
			name:            "private visibility",
			cmsVisibility:   cms.VisibilityPrivate,
			protoVisibility: proto.Visibility_PRIVATE,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Test CMS to Proto conversion (implicit in CreateProject method)
			// This would be tested in integration tests with actual gRPC calls

			// Test Proto to CMS conversion
			result := convertProtoToVisibility(tt.protoVisibility)
			assert.Equal(t, tt.cmsVisibility, result)
		})
	}
}

func TestConvertProtoToSchemaFieldType(t *testing.T) {
	tests := []struct {
		name      string
		protoType proto.SchemaFieldType
		cmsType   cms.SchemaFieldType
	}{
		{
			name:      "text field",
			protoType: proto.SchemaFieldType_Text,
			cmsType:   cms.SchemaFieldTypeText,
		},
		{
			name:      "textarea field",
			protoType: proto.SchemaFieldType_TextArea,
			cmsType:   cms.SchemaFieldTypeTextArea,
		},
		{
			name:      "asset field",
			protoType: proto.SchemaFieldType_Asset,
			cmsType:   cms.SchemaFieldTypeAsset,
		},
		{
			name:      "geometry object field",
			protoType: proto.SchemaFieldType_GeometryObject,
			cmsType:   cms.SchemaFieldTypeGeometryObject,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result := convertProtoToSchemaFieldType(tt.protoType)
			assert.Equal(t, tt.cmsType, result)
		})
	}
}

// Helper function to create string pointers
func stringPtr(s string) *string {
	return &s
}
