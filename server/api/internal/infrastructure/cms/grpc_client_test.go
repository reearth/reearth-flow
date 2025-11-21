package cms

import (
	"context"
	"errors"
	"sync"
	"testing"
	"time"

	cmspb "github.com/eukarya-inc/reearth-proto/gen/go/cms/v1"
	"github.com/reearth/reearth-flow/api/pkg/cms"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"
	"google.golang.org/grpc"
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
		useTLS   bool
		wantErr  bool
	}{
		{
			name:     "empty endpoint should fail",
			endpoint: "",
			token:    "test-token",
			useTLS:   false,
			wantErr:  true,
		},
		{
			name:     "valid endpoint should succeed",
			endpoint: "localhost:50051",
			token:    "test-token",
			useTLS:   false,
			wantErr:  false,
		},
		{
			name:     "valid endpoint with TLS",
			endpoint: "localhost:50051",
			token:    "test-token",
			useTLS:   true,
			wantErr:  false,
		},
		{
			name:     "no token should succeed",
			endpoint: "localhost:50051",
			token:    "",
			useTLS:   false,
			wantErr:  false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Reset global pool for clean testing
			globalPool = nil
			poolOnce = sync.Once{}

			client, err := NewGRPCClient(tt.endpoint, tt.token, tt.useTLS)

			if tt.wantErr {
				assert.Error(t, err)
				assert.Nil(t, client)
			} else {
				if err != nil {
					assert.Contains(t, err.Error(), "failed to connect")
				} else {
					assert.NotNil(t, client)
					if gc, ok := client.(*grpcClient); ok {
						assert.NoError(t, gc.Close())
					}
				}
			}
		})
	}
}

func TestConnectionPool_GetConnection(t *testing.T) {
	pool := &ConnectionPool{
		connections: make(map[string]*pooledConnection),
		maxSize:     10,
		maxIdleTime: 5 * time.Minute,
	}

	_, err := pool.getConnection("localhost:50051", "token", false)
	assert.Error(t, err)
	assert.Contains(t, err.Error(), "failed to connect")
}

func TestConnectionPool_ReleaseConnection(t *testing.T) {
	pool := &ConnectionPool{
		connections: make(map[string]*pooledConnection),
		maxSize:     10,
		maxIdleTime: 5 * time.Minute,
	}

	key := "localhost:50051|token|false"
	pool.connections[key] = &pooledConnection{
		refCount: 2,
		lastUsed: time.Now(),
	}

	pool.releaseConnection("localhost:50051", "token", false)

	assert.Equal(t, int32(1), pool.connections[key].refCount)
}

func TestTokenAuth(t *testing.T) {
	token := "test-token"
	auth := &tokenAuth{token: token}

	t.Run("GetRequestMetadata", func(t *testing.T) {
		ctx := context.Background()
		metadata, err := auth.GetRequestMetadata(ctx)

		assert.NoError(t, err)
		assert.Equal(t, "Bearer test-token", metadata["authorization"])
	})

	t.Run("RequireTransportSecurity", func(t *testing.T) {
		assert.True(t, auth.RequireTransportSecurity())
	})
}

func TestTrimPort(t *testing.T) {
	tests := []struct {
		name     string
		endpoint string
		expected string
	}{
		{
			name:     "endpoint with port",
			endpoint: "localhost:50051",
			expected: "localhost",
		},
		{
			name:     "endpoint without port",
			endpoint: "localhost",
			expected: "localhost",
		},
		{
			name:     "IP with port",
			endpoint: "127.0.0.1:8080",
			expected: "127.0.0.1",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result := trimPort(tt.endpoint)
			assert.Equal(t, tt.expected, result)
		})
	}
}

func TestConvertProtoToVisibility(t *testing.T) {
	tests := []struct {
		name  string
		input cmspb.Visibility
		want  cms.Visibility
	}{
		{
			name:  "public visibility",
			input: cmspb.Visibility_PUBLIC,
			want:  cms.VisibilityPublic,
		},
		{
			name:  "private visibility",
			input: cmspb.Visibility_PRIVATE,
			want:  cms.VisibilityPrivate,
		},
		{
			name:  "unknown visibility defaults to private",
			input: cmspb.Visibility(999),
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
	desc := "Test description"
	license := "MIT"
	readme := "# Test README"

	protoProject := &cmspb.Project{
		Id:          "project-123",
		Name:        "Test Project",
		Alias:       "test-project",
		Description: &desc,
		License:     &license,
		Readme:      &readme,
		WorkspaceId: "workspace-456",
		Visibility:  cmspb.Visibility_PUBLIC,
		Topics:      []string{"geospatial", "data"},
		StarCount:   42,
		CreatedAt:   timestamppb.New(now),
		UpdatedAt:   timestamppb.New(now),
	}

	result := convertProtoToProject(protoProject)

	assert.NotNil(t, result)
	assert.Equal(t, "project-123", result.ID)
	assert.Equal(t, "Test Project", result.Name)
	assert.Equal(t, "test-project", result.Alias)
	assert.Equal(t, &desc, result.Description)
	assert.Equal(t, &license, result.License)
	assert.Equal(t, &readme, result.Readme)
	assert.Equal(t, "workspace-456", result.WorkspaceID)
	assert.Equal(t, cms.VisibilityPublic, result.Visibility)
	assert.Equal(t, []string{"geospatial", "data"}, result.Topics)
	assert.Equal(t, int64(42), result.StarCount)
	assert.Equal(t, now.Unix(), result.CreatedAt.Unix())
	assert.Equal(t, now.Unix(), result.UpdatedAt.Unix())

	nilResult := convertProtoToProject(nil)
	assert.Nil(t, nilResult)
}

func TestConvertProtoToAsset(t *testing.T) {
	now := time.Now()
	previewType := "image"
	archiveStatus := "extracted"

	protoAsset := &cmspb.Asset{
		Id:                      "asset-123",
		Uuid:                    "uuid-456",
		ProjectId:               "project-789",
		Filename:                "test.jpg",
		Size:                    1024,
		PreviewType:             &previewType,
		Url:                     "https://example.com/test.jpg",
		ArchiveExtractionStatus: &archiveStatus,
		Public:                  true,
		CreatedAt:               timestamppb.New(now),
	}

	result := convertProtoToAsset(protoAsset)

	assert.NotNil(t, result)
	assert.Equal(t, "asset-123", result.ID)
	assert.Equal(t, "uuid-456", result.UUID)
	assert.Equal(t, "project-789", result.ProjectID)
	assert.Equal(t, "test.jpg", result.Filename)
	assert.Equal(t, uint64(1024), result.Size)
	assert.Equal(t, &previewType, result.PreviewType)
	assert.Equal(t, "https://example.com/test.jpg", result.URL)
	assert.Equal(t, &archiveStatus, result.ArchiveExtractionStatus)
	assert.True(t, result.Public)
	assert.Equal(t, now.Unix(), result.CreatedAt.Unix())

	nilResult := convertProtoToAsset(nil)
	assert.Nil(t, nilResult)
}

func TestConvertProtoToModel(t *testing.T) {
	now := time.Now()

	protoModel := &cmspb.Model{
		Id:          "model-123",
		ProjectId:   "project-456",
		Name:        "Test Model",
		Description: "Test model description",
		Key:         "test_model",
		Schema: &cmspb.Schema{
			SchemaId: "schema-789",
			Fields: []*cmspb.SchemaField{
				{
					FieldId:     "field-1",
					Name:        "Title",
					Type:        cmspb.SchemaField_Text,
					Key:         "title",
					Description: ptr("Title field"),
				},
			},
		},
		PublicApiEp: "/api/models/test_model",
		EditorUrl:   "/admin/models/test_model",
		CreatedAt:   timestamppb.New(now),
		UpdatedAt:   timestamppb.New(now),
	}

	result := convertProtoToModel(protoModel)

	assert.NotNil(t, result)
	assert.Equal(t, "model-123", result.ID)
	assert.Equal(t, "project-456", result.ProjectID)
	assert.Equal(t, "Test Model", result.Name)
	assert.Equal(t, "Test model description", result.Description)
	assert.Equal(t, "test_model", result.Key)
	assert.Equal(t, "schema-789", result.Schema.SchemaID)
	assert.Len(t, result.Schema.Fields, 1)
	assert.Equal(t, "field-1", result.Schema.Fields[0].FieldID)
	assert.Equal(t, "Title", result.Schema.Fields[0].Name)
	assert.Equal(t, "/api/models/test_model", result.PublicAPIEP)
	assert.Equal(t, "/admin/models/test_model", result.EditorURL)

	nilResult := convertProtoToModel(nil)
	assert.Nil(t, nilResult)
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
		{
			name: "unknown type returns raw bytes",
			setup: func() *anypb.Any {
				return &anypb.Any{
					TypeUrl: "type.googleapis.com/unknown.CustomType",
					Value:   []byte("raw data"),
				}
			},
			expected: []byte("raw data"),
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			input := tt.setup()
			result := convertAnyToInterface(input)

			if tt.name == "timestamp value" {
				assert.IsType(t, time.Time{}, result)
			} else {
				assert.Equal(t, tt.expected, result)
			}
		})
	}
}

func TestGrpcClient_Close(t *testing.T) {
	pool := &ConnectionPool{
		connections: make(map[string]*pooledConnection),
		maxSize:     10,
		maxIdleTime: 5 * time.Minute,
	}
	globalPool = pool

	client := &grpcClient{
		endpoint: "localhost:50051",
		token:    "test-token",
		useTLS:   false,
	}

	key := "localhost:50051|test-token|false"
	pool.connections[key] = &pooledConnection{
		refCount: 1,
		lastUsed: time.Now(),
	}

	err := client.Close()
	assert.NoError(t, err)
	assert.Equal(t, int32(0), pool.connections[key].refCount)
}

type MockReEarthCMSClient struct {
	mock.Mock
}

func (m *MockReEarthCMSClient) GetProject(ctx context.Context, req *cmspb.ProjectRequest, opts ...grpc.CallOption) (*cmspb.ProjectResponse, error) {
	args := m.Called(ctx, req)
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).(*cmspb.ProjectResponse), args.Error(1)
}

func (m *MockReEarthCMSClient) ListProjects(ctx context.Context, req *cmspb.ListProjectsRequest, opts ...grpc.CallOption) (*cmspb.ListProjectsResponse, error) {
	args := m.Called(ctx, req)
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).(*cmspb.ListProjectsResponse), args.Error(1)
}

func (m *MockReEarthCMSClient) GetAsset(ctx context.Context, req *cmspb.AssetRequest, opts ...grpc.CallOption) (*cmspb.AssetResponse, error) {
	args := m.Called(ctx, req)
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).(*cmspb.AssetResponse), args.Error(1)
}

func (m *MockReEarthCMSClient) ListAssets(ctx context.Context, req *cmspb.ListAssetsRequest, opts ...grpc.CallOption) (*cmspb.ListAssetsResponse, error) {
	args := m.Called(ctx, req)
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).(*cmspb.ListAssetsResponse), args.Error(1)
}

func (m *MockReEarthCMSClient) GetModel(ctx context.Context, req *cmspb.ModelRequest, opts ...grpc.CallOption) (*cmspb.ModelResponse, error) {
	args := m.Called(ctx, req)
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).(*cmspb.ModelResponse), args.Error(1)
}

func (m *MockReEarthCMSClient) ListModels(ctx context.Context, req *cmspb.ListModelsRequest, opts ...grpc.CallOption) (*cmspb.ListModelsResponse, error) {
	args := m.Called(ctx, req)
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).(*cmspb.ListModelsResponse), args.Error(1)
}

func (m *MockReEarthCMSClient) ListItems(ctx context.Context, req *cmspb.ListItemsRequest, opts ...grpc.CallOption) (*cmspb.ListItemsResponse, error) {
	args := m.Called(ctx, req)
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).(*cmspb.ListItemsResponse), args.Error(1)
}

func (m *MockReEarthCMSClient) GetModelExportURL(ctx context.Context, req *cmspb.ModelExportRequest, opts ...grpc.CallOption) (*cmspb.ExportURLResponse, error) {
	args := m.Called(ctx, req)
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).(*cmspb.ExportURLResponse), args.Error(1)
}

// nolint:staticcheck // SA1019: Deprecated in proto; required only to satisfy generated interface in tests.
func (m *MockReEarthCMSClient) GetModelGeoJSONExportURL(ctx context.Context, req *cmspb.ExportRequest, opts ...grpc.CallOption) (*cmspb.ExportURLResponse, error) {
	args := m.Called(ctx, req)
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).(*cmspb.ExportURLResponse), args.Error(1)
}

func (m *MockReEarthCMSClient) CreateProject(ctx context.Context, req *cmspb.CreateProjectRequest, opts ...grpc.CallOption) (*cmspb.ProjectResponse, error) {
	args := m.Called(ctx, req)
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).(*cmspb.ProjectResponse), args.Error(1)
}

func (m *MockReEarthCMSClient) UpdateProject(ctx context.Context, req *cmspb.UpdateProjectRequest, opts ...grpc.CallOption) (*cmspb.ProjectResponse, error) {
	args := m.Called(ctx, req)
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).(*cmspb.ProjectResponse), args.Error(1)
}

func (m *MockReEarthCMSClient) DeleteProject(ctx context.Context, req *cmspb.DeleteProjectRequest, opts ...grpc.CallOption) (*cmspb.DeleteProjectResponse, error) {
	args := m.Called(ctx, req)
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).(*cmspb.DeleteProjectResponse), args.Error(1)
}

func (m *MockReEarthCMSClient) CheckAliasAvailability(ctx context.Context, req *cmspb.AliasAvailabilityRequest, opts ...grpc.CallOption) (*cmspb.AliasAvailabilityResponse, error) {
	args := m.Called(ctx, req)
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).(*cmspb.AliasAvailabilityResponse), args.Error(1)
}

func (m *MockReEarthCMSClient) StarProject(ctx context.Context, req *cmspb.StarRequest, opts ...grpc.CallOption) (*cmspb.StarResponse, error) {
	args := m.Called(ctx, req)
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).(*cmspb.StarResponse), args.Error(1)
}

func TestGrpcClient_GetAsset(t *testing.T) {
	mockClient := &MockReEarthCMSClient{}
	client := &grpcClient{
		client: mockClient,
	}

	t.Run("successful get asset", func(t *testing.T) {
		now := time.Now()
		previewType := "image"

		expectedReq := &cmspb.AssetRequest{
			AssetId: "asset-123",
		}

		mockResponse := &cmspb.AssetResponse{
			Asset: &cmspb.Asset{
				Id:          "asset-123",
				Uuid:        "uuid-456",
				ProjectId:   "project-789",
				Filename:    "test.jpg",
				Size:        1024,
				PreviewType: &previewType,
				Url:         "https://example.com/test.jpg",
				Public:      true,
				CreatedAt:   timestamppb.New(now),
			},
		}

		mockClient.On("GetAsset", mock.Anything, expectedReq).Return(mockResponse, nil)

		result, err := client.GetAsset(context.Background(), cms.GetAssetInput{
			AssetID: "asset-123",
		})

		assert.NoError(t, err)
		assert.NotNil(t, result)
		assert.Equal(t, "asset-123", result.ID)
		assert.Equal(t, "uuid-456", result.UUID)
		assert.Equal(t, "project-789", result.ProjectID)
		assert.Equal(t, "test.jpg", result.Filename)
		assert.Equal(t, uint64(1024), result.Size)
		assert.Equal(t, &previewType, result.PreviewType)
		assert.Equal(t, "https://example.com/test.jpg", result.URL)
		assert.True(t, result.Public)

		mockClient.AssertExpectations(t)
	})

	t.Run("error getting asset", func(t *testing.T) {
		mockClient.ExpectedCalls = nil
		expectedReq := &cmspb.AssetRequest{
			AssetId: "asset-404",
		}

		mockClient.On("GetAsset", mock.Anything, expectedReq).Return(nil, errors.New("asset not found"))

		result, err := client.GetAsset(context.Background(), cms.GetAssetInput{
			AssetID: "asset-404",
		})

		assert.Error(t, err)
		assert.Nil(t, result)
		assert.Contains(t, err.Error(), "failed to get asset")

		mockClient.AssertExpectations(t)
	})
}

func TestGrpcClient_ListModels(t *testing.T) {
	mockClient := &MockReEarthCMSClient{}
	client := &grpcClient{
		client: mockClient,
	}

	t.Run("successful list models", func(t *testing.T) {
		now := time.Now()

		expectedReq := &cmspb.ListModelsRequest{
			ProjectId: "project-123",
			PageInfo: &cmspb.PageInfo{
				Page:     1,
				PageSize: 10,
			},
			SortInfo: &cmspb.SortInfo{
				Key:      "name",
				Reverted: false,
			},
		}

		mockResponse := &cmspb.ListModelsResponse{
			Models: []*cmspb.Model{
				{
					Id:          "model-1",
					ProjectId:   "project-123",
					Name:        "Test Model 1",
					Description: "Description 1",
					Key:         "test_model_1",
					Schema: &cmspb.Schema{
						SchemaId: "schema-1",
						Fields: []*cmspb.SchemaField{
							{
								FieldId:     "field-1",
								Name:        "Title",
								Type:        cmspb.SchemaField_Text,
								Key:         "title",
								Description: ptr("Title field"),
							},
						},
					},
					PublicApiEp: "/api/models/test_model_1",
					EditorUrl:   "/admin/models/test_model_1",
					CreatedAt:   timestamppb.New(now),
					UpdatedAt:   timestamppb.New(now),
				},
			},
			TotalCount: 1,
			PageInfo: &cmspb.PageInfo{
				Page:     1,
				PageSize: 10,
			},
		}

		mockClient.On("ListModels", mock.Anything, expectedReq).Return(mockResponse, nil)

		result, err := client.ListModels(context.Background(), cms.ListModelsInput{
			ProjectID: "project-123",
			PageInfo: &cms.PageInfo{
				Page:     1,
				PageSize: 10,
			},
			SortInfo: &cms.SortInfo{
				Key:      "name",
				Reverted: false,
			},
		})

		assert.NoError(t, err)
		assert.NotNil(t, result)
		assert.Len(t, result.Models, 1)
		assert.Equal(t, int64(1), result.TotalCount)
		assert.Equal(t, "model-1", result.Models[0].ID)
		assert.Equal(t, "Test Model 1", result.Models[0].Name)
		assert.NotNil(t, result.PageInfo)
		assert.Equal(t, int32(1), result.PageInfo.Page)
		assert.Equal(t, int32(10), result.PageInfo.PageSize)

		mockClient.AssertExpectations(t)
	})

	t.Run("list models without pagination", func(t *testing.T) {
		mockClient.ExpectedCalls = nil
		expectedReq := &cmspb.ListModelsRequest{
			ProjectId: "project-456",
		}

		mockResponse := &cmspb.ListModelsResponse{
			Models:     []*cmspb.Model{},
			TotalCount: 0,
		}

		mockClient.On("ListModels", mock.Anything, expectedReq).Return(mockResponse, nil)

		result, err := client.ListModels(context.Background(), cms.ListModelsInput{
			ProjectID: "project-456",
		})

		assert.NoError(t, err)
		assert.NotNil(t, result)
		assert.Len(t, result.Models, 0)
		assert.Equal(t, int64(0), result.TotalCount)
		assert.Nil(t, result.PageInfo)

		mockClient.AssertExpectations(t)
	})

	t.Run("error listing models", func(t *testing.T) {
		mockClient.ExpectedCalls = nil // Reset mock
		expectedReq := &cmspb.ListModelsRequest{
			ProjectId: "project-error",
		}

		mockClient.On("ListModels", mock.Anything, expectedReq).Return(nil, errors.New("project not found"))

		result, err := client.ListModels(context.Background(), cms.ListModelsInput{
			ProjectID: "project-error",
		})

		assert.Error(t, err)
		assert.Nil(t, result)
		assert.Contains(t, err.Error(), "failed to list models")

		mockClient.AssertExpectations(t)
	})
}

func TestGrpcClient_ListItems(t *testing.T) {
	mockClient := &MockReEarthCMSClient{}
	client := &grpcClient{
		client: mockClient,
	}

	t.Run("successful list items", func(t *testing.T) {
		now := time.Now()
		keyword := "search term"

		titleAny, _ := anypb.New(&wrapperspb.StringValue{Value: "Item Title"})
		countAny, _ := anypb.New(&wrapperspb.Int32Value{Value: 42})

		expectedReq := &cmspb.ListItemsRequest{
			ModelId:   "model-123",
			ProjectId: "project-456",
			Keyword:   &keyword,
			PageInfo: &cmspb.PageInfo{
				Page:     1,
				PageSize: 20,
			},
			SortInfo: &cmspb.SortInfo{
				Key:      "createdAt",
				Reverted: true,
			},
		}

		mockResponse := &cmspb.ListItemsResponse{
			Items: []*cmspb.Item{
				{
					Id: "item-1",
					Fields: map[string]*anypb.Any{
						"title": titleAny,
						"count": countAny,
					},
					CreatedAt: timestamppb.New(now),
					UpdatedAt: timestamppb.New(now),
				},
			},
			TotalCount: 1,
			PageInfo: &cmspb.PageInfo{
				Page:     1,
				PageSize: 20,
			},
		}

		mockClient.On("ListItems", mock.Anything, expectedReq).Return(mockResponse, nil)

		result, err := client.ListItems(context.Background(), cms.ListItemsInput{
			ModelID:   "model-123",
			ProjectID: "project-456",
			Keyword:   &keyword,
			PageInfo: &cms.PageInfo{
				Page:     1,
				PageSize: 20,
			},
			SortInfo: &cms.SortInfo{
				Key:      "createdAt",
				Reverted: true,
			},
		})

		assert.NoError(t, err)
		assert.NotNil(t, result)
		assert.Len(t, result.Items, 1)
		assert.Equal(t, int64(1), result.TotalCount)
		assert.Equal(t, "item-1", result.Items[0].ID)
		assert.Equal(t, "Item Title", result.Items[0].Fields["title"])
		assert.Equal(t, int32(42), result.Items[0].Fields["count"])
		assert.NotNil(t, result.PageInfo)
		assert.Equal(t, int32(1), result.PageInfo.Page)
		assert.Equal(t, int32(20), result.PageInfo.PageSize)

		mockClient.AssertExpectations(t)
	})

	t.Run("list items without keyword and pagination", func(t *testing.T) {
		mockClient.ExpectedCalls = nil
		expectedReq := &cmspb.ListItemsRequest{
			ModelId:   "model-789",
			ProjectId: "project-101",
		}

		mockResponse := &cmspb.ListItemsResponse{
			Items:      []*cmspb.Item{},
			TotalCount: 0,
		}

		mockClient.On("ListItems", mock.Anything, expectedReq).Return(mockResponse, nil)

		result, err := client.ListItems(context.Background(), cms.ListItemsInput{
			ModelID:   "model-789",
			ProjectID: "project-101",
		})

		assert.NoError(t, err)
		assert.NotNil(t, result)
		assert.Len(t, result.Items, 0)
		assert.Equal(t, int64(0), result.TotalCount)
		assert.Nil(t, result.PageInfo)

		mockClient.AssertExpectations(t)
	})

	t.Run("error listing items", func(t *testing.T) {
		mockClient.ExpectedCalls = nil
		expectedReq := &cmspb.ListItemsRequest{
			ModelId:   "model-invalid",
			ProjectId: "project-invalid",
		}

		mockClient.On("ListItems", mock.Anything, expectedReq).Return(nil, errors.New("model not found"))

		result, err := client.ListItems(context.Background(), cms.ListItemsInput{
			ModelID:   "model-invalid",
			ProjectID: "project-invalid",
		})

		assert.Error(t, err)
		assert.Nil(t, result)
		assert.Contains(t, err.Error(), "failed to list items")

		mockClient.AssertExpectations(t)
	})
}

func ptr(s string) *string {
	return &s
}
