# CMS Schema Proto Design Document

## Overview

This document provides a comprehensive design specification for the `schema.proto` file located at `/Users/xy/work/eukarya/reearth-flow/server/api/pkg/cms/proto/schema.proto`. This protocol buffer definition serves as the interface contract between Re:Earth Flow and the Re:Earth CMS system, enabling Flow to interact with CMS projects, models, and data through a standardized gRPC API.

## File Reference

**Location**: `/Users/xy/work/eukarya/reearth-flow/server/api/pkg/cms/proto/schema.proto`  
**Package**: `reearth.cms.v1`  
**Go Package**: `github.com/reearth/reearth-flow/api/pkg/cms/proto`

## Service Definition

### ReEarthCMS Service

The `ReEarthCMS` service provides a comprehensive gRPC API for managing CMS projects, models, and content items:

```proto
service ReEarthCMS {
  // Project Management
  rpc CreateProject(CreateProjectRequest) returns (ProjectResponse) {}
  rpc UpdateProject(UpdateProjectRequest) returns (ProjectResponse) {}
  rpc DeleteProject(DeleteProjectRequest) returns (DeleteProjectResponse) {}
  rpc GetProject(ProjectRequest) returns (ProjectResponse) {}
  rpc ListProjects(ListProjectsRequest) returns (ListProjectsResponse) {}
  
  // Model Operations
  rpc ListModels(ListModelsRequest) returns (ListModelsResponse) {}
  
  // Content Operations
  rpc ListItems(ListItemsRequest) returns (ListItemsResponse) {}
  
  // Utility Operations
  rpc CheckAliasAvailability(AliasAvailabilityRequest) returns (AliasAvailabilityResponse) {}
  rpc GetModelGeoJSONExportURL(ExportRequest) returns (ExportURLResponse) {}
}
```

## Core Data Models

### Project Entity

The `Project` message represents a CMS project with comprehensive metadata:

```proto
message Project {
  string id = 1;                                    // Unique project identifier
  string name = 2;                                  // Human-readable project name
  string alias = 3;                                 // URL-friendly project alias
  optional string description = 4;                  // Project description
  optional string license = 5;                      // License information
  optional string readme = 6;                       // README content
  string workspace_id = 7;                          // Parent workspace ID
  Visibility visibility = 8;                        // Public/private visibility
  google.protobuf.Timestamp created_at = 9;         // Creation timestamp
  google.protobuf.Timestamp updated_at = 10;        // Last update timestamp
}
```

#### Visibility Enum

```proto
enum Visibility {
  PUBLIC = 0;   // Publicly accessible project
  PRIVATE = 1;  // Private project (workspace access only)
}
```

### Model Entity

The `Model` message represents a data model within a CMS project:

```proto
message Model {
  string id = 1;                                    // Unique model identifier
  string project_id = 2;                            // Parent project ID
  string name = 3;                                  // Model display name
  string description = 4;                           // Model description
  string key = 5;                                   // API key for model access
  Schema schema = 6;                                // Model schema definition
  string public_api_ep = 7;                         // Public API endpoint
  string editor_url = 8;                            // CMS editor URL
  google.protobuf.Timestamp created_at = 9;         // Creation timestamp
  google.protobuf.Timestamp updated_at = 10;        // Last update timestamp
}
```

### Item Entity

The `Item` message represents a content item within a model:

```proto
message Item {
  string id = 1;                                    // Unique item identifier
  map<string, google.protobuf.Any> fields = 2;     // Dynamic field values
  google.protobuf.Timestamp created_at = 5;        // Creation timestamp
  google.protobuf.Timestamp updated_at = 6;        // Last update timestamp
}
```

**Note**: The flexible `fields` map allows for dynamic content structures based on the model's schema definition.

## Schema System

### Schema Definition

```proto
message Schema {
  string schema_id = 1;                             // Unique schema identifier
  repeated SchemaField fields = 2;                  // List of field definitions
}
```

### Schema Field Definition

```proto
message SchemaField {
  string field_id = 1;                              // Unique field identifier
  string name = 2;                                  // Field display name
  SchemaFieldType type = 3;                         // Field data type
  string key = 4;                                   // API key for field access
  optional string description = 5;                  // Field description
}
```

### Supported Field Types

The schema system supports 17 different field types for flexible content modeling:

```proto
enum SchemaFieldType {
  // Text Types
  Text = 0;              // Single-line text input
  TextArea = 1;          // Multi-line text input
  RichText = 2;          // Rich text editor
  MarkdownText = 3;      // Markdown text editor
  
  // Media Types
  Asset = 4;             // File/image upload
  URL = 13;              // URL input
  
  // Data Types
  Date = 5;              // Date/time picker
  Bool = 6;              // Boolean checkbox
  Integer = 9;           // Integer number input
  Number = 10;           // Decimal number input
  
  // Selection Types
  Select = 7;            // Dropdown selection
  Tag = 8;               // Tag selection
  Checkbox = 12;         // Checkbox selection
  
  // Relationship Types
  Reference = 11;        // Reference to other items
  Group = 14;            // Group of fields
  
  // Geometry Types
  GeometryObject = 15;   // Static geometry object
  GeometryEditor = 16;   // Interactive geometry editor
}
```

## Request/Response Messages

### Project Operations

#### Create Project Request
```proto
message CreateProjectRequest {
  string workspace_id = 1;                          // Target workspace ID
  string name = 2;                                  // Project name
  optional string description = 3;                  // Project description
  optional string license = 4;                      // License information
  optional string readme = 5;                       // README content
  string alias = 6;                                 // Unique project alias
  Visibility visibility = 7;                        // Visibility setting
}
```

#### Update Project Request
```proto
message UpdateProjectRequest {
  string project_id = 1;                            // Target project ID
  optional string name = 2;                         // Updated name
  optional string description = 3;                  // Updated description
  optional string license = 4;                      // Updated license
  optional string readme = 5;                       // Updated README
  optional string alias = 6;                        // Updated alias
  optional Visibility visibility = 7;               // Updated visibility
}
```

#### List Projects Request
```proto
message ListProjectsRequest {
  string workspace_id = 1;                          // Target workspace ID
  bool public_only = 2;                             // Filter for public projects only
}
```

### Model Operations

#### List Models Request
```proto
message ListModelsRequest {
  string project_id = 1;                            // Target project ID
}
```

### Content Operations

#### List Items Request
```proto
message ListItemsRequest {
  string model_id = 1;                              // Target model ID
  string project_id = 2;                            // Target project ID
  optional int32 page = 3;                          // Page number (pagination)
  optional int32 page_size = 4;                     // Items per page
}
```

### Utility Operations

#### Alias Availability Request
```proto
message AliasAvailabilityRequest {
  string alias = 1;                                 // Alias to check
}
```

#### Export Request
```proto
message ExportRequest {
  string project_id = 1;                            // Target project ID
  string model_id = 2;                              // Target model ID
}
```

## Authentication and Authorization

### Authentication Implementation

The protocol buffer definition includes implementation notes for authentication:

```proto
// Implementation note:
// Authentication should be implemented using gRPC interceptors
// M2M tokens should be passed in metadata with key "authorization"
// Format: "Bearer <token>"
// UserId should be passed in metadata with key "user-id"
```

### Authentication Pattern

```go
// Example authentication metadata
ctx = metadata.NewOutgoingContext(ctx, metadata.New(map[string]string{
    "authorization": "Bearer <M2M_TOKEN>",
    "user-id": "<USER_ID>",
}))
```

## Dashboard Integration Patterns

### Project Creation Workflow

The Dashboard integrates with the CMS service through the following workflow:

1. **User Input**: User fills out project creation form in Dashboard UI
2. **Validation**: Dashboard validates input and checks alias availability
3. **Project Creation**: Dashboard calls `CreateProject` gRPC method
4. **Response Handling**: Dashboard processes response and updates UI

#### Dashboard Implementation Example

```go
// Dashboard CMS Gateway Implementation
func (c *Client) CreateProject(ctx context.Context, req *cms.CreateProjectRequest, usr *user.User) (*cms.Project, error) {
    // Add authentication metadata
    ctx = c.addAuth(ctx, usr)
    
    // Call CMS service
    resp, err := c.client.CreateProject(ctx, req)
    if err != nil {
        return nil, err
    }
    
    return resp.Project, nil
}
```

#### Dashboard UI Integration

```typescript
// Dashboard Project Creation Form
const createProject = useCallback(
  (workspaceAlias: string, projectType: ProjectType, body: ProjectCreateInput) => {
    mutate({
      path: { workspace_alias: workspaceAlias, project_type: projectType },
      body: {
        name: body.name,
        alias: body.alias,
        description: body.description,
        license: body.license,
        readme: body.readme,
        scope: body.scope, // Maps to Visibility enum
      },
      headers: authHeaders,
    });
  },
  [mutate, authHeaders]
);
```

### Data Display Integration

The Dashboard displays CMS project data through specialized components:

```typescript
// CMS Project Data Display
interface ProjectCms {
  editor_url?: string;
  models?: Array<{
    name?: string;
    endpoint?: string;
    download_url?: string;
    items?: Array<ProjectCmsItems>;
    schema?: Array<ProjectCmsSchema>;
  }>;
}
```

## CMS Service Usage Patterns

### Model and Schema Management

#### Schema Creation Pattern

```go
// Schema field creation with validation
sf1 := schema.NewField(schema.NewText(nil).TypeProperty()).
    ID(fieldID).
    Key(fieldKey).
    Name("Title").
    Description("Title field").
    MustBuild()

sf2 := schema.NewField(schema.NewGeometryObject(supportedTypes).TypeProperty()).
    ID(geoFieldID).
    Key(geoFieldKey).
    Name("Location").
    Description("Geometry field").
    MustBuild()

// Schema creation with fields
s := schema.New().
    ID(schemaID).
    Workspace(workspaceID).
    Project(projectID).
    Fields([]*schema.Field{sf1, sf2}).
    TitleField(sf1.ID().Ref()).
    MustBuild()
```

#### Model Creation Pattern

```go
// Model creation with schema reference
m := model.New().
    ID(modelID).
    Name("Sample Model").
    Description("Sample model description").
    Public(true).
    Key("sample-model").
    Project(projectID).
    Schema(schemaID).
    Order(1).
    MustBuild()
```

### Content Item Management

#### Item Creation Pattern

```go
// Item creation with field values
item := item.New().
    ID(itemID).
    Schema(schemaID).
    Model(modelID).
    Project(projectID).
    Fields([]*item.Field{
        item.NewField(textFieldID, value.TypeText.Value("Sample Title"), nil),
        item.NewField(geoFieldID, value.TypeGeometryObject.Value(geometry), nil),
    }).
    MustBuild()
```

#### Item Query Pattern

```go
// Item search with pagination
items, pageInfo, err := uc.Item.Search(ctx, schema, query, pagination, operator)
if err != nil {
    return nil, err
}

// Process items for API response
result := make([]ItemResponse, len(items))
for i, item := range items {
    result[i] = convertItemToResponse(item, schema)
}
```

### Export Functionality

#### GeoJSON Export Pattern

```go
// GeoJSON export for spatial data
func (c *Client) GetModelGeoJSONExportURL(ctx context.Context, projectID, modelID string) (string, error) {
    ctx = c.addAuth(ctx, user)
    
    resp, err := c.client.GetModelGeoJSONExportURL(ctx, &cms.ExportRequest{
        ProjectId: projectID,
        ModelId:   modelID,
    })
    if err != nil {
        return "", err
    }
    
    return resp.Url, nil
}
```

#### CSV Export Pattern

```go
// CSV export with field headers
func BuildCSVHeaders(s *schema.Schema) []string {
    headers := []string{"id", "location_lat", "location_lng"}
    
    for _, field := range s.Fields() {
        if !field.IsGeometryField() {
            headers = append(headers, field.Name())
        }
    }
    
    return headers
}
```

## Error Handling

### Common Error Scenarios

```go
// Error handling patterns
var (
    ErrProjectNotFound       = errors.New("project not found")
    ErrModelNotFound        = errors.New("model not found")
    ErrUnauthorized         = errors.New("unauthorized")
    ErrAliasAlreadyExists   = errors.New("project alias already exists")
    ErrInvalidInput         = errors.New("invalid input")
)

// Error conversion from gRPC status codes
func convertGRPCError(err error) error {
    if err == nil {
        return nil
    }
    
    switch status.Code(err) {
    case codes.NotFound:
        return ErrProjectNotFound
    case codes.Unauthenticated:
        return ErrUnauthorized
    case codes.AlreadyExists:
        return ErrAliasAlreadyExists
    default:
        return fmt.Errorf("CMS service error: %w", err)
    }
}
```

### Field Validation

```go
// Field type validation
func (f *SchemaField) ValidateValue(v *value.Value) error {
    switch f.Type {
    case SchemaFieldType_Text:
        return validateTextValue(v, f.constraints)
    case SchemaFieldType_Integer:
        return validateIntegerValue(v, f.constraints)
    case SchemaFieldType_GeometryObject:
        return validateGeometryValue(v, f.constraints)
    default:
        return fmt.Errorf("unsupported field type: %v", f.Type)
    }
}
```

## Performance Considerations

### Pagination Implementation

```go
// Efficient pagination for large datasets
func (s *service) ListItems(ctx context.Context, req *ListItemsRequest) (*ListItemsResponse, error) {
    const defaultPageSize = 50
    const maxPageSize = 1000
    
    pageSize := req.PageSize
    if pageSize == 0 {
        pageSize = defaultPageSize
    }
    if pageSize > maxPageSize {
        pageSize = maxPageSize
    }
    
    // Use cursor-based pagination for better performance
    offset := (req.Page - 1) * pageSize
    
    items, totalCount, err := s.itemRepo.FindByModel(ctx, req.ModelId, offset, pageSize)
    if err != nil {
        return nil, err
    }
    
    return &ListItemsResponse{
        Items:      items,
        TotalCount: totalCount,
    }, nil
}
```

### Field Indexing

```go
// Index management for efficient queries
type FieldIndex struct {
    FieldID   string
    FieldType SchemaFieldType
    Indexed   bool
    Unique    bool
}

// Create indexes for frequently queried fields
func (s *service) CreateFieldIndexes(ctx context.Context, schema *Schema) error {
    for _, field := range schema.Fields {
        if shouldIndex(field) {
            err := s.indexRepo.CreateIndex(ctx, field.FieldId, field.Type)
            if err != nil {
                return err
            }
        }
    }
    return nil
}
```

## Security Considerations

### Input Validation

```go
// Comprehensive input validation
func validateCreateProjectRequest(req *CreateProjectRequest) error {
    if req.Name == "" {
        return errors.New("project name is required")
    }
    
    if req.Alias == "" {
        return errors.New("project alias is required")
    }
    
    if !isValidAlias(req.Alias) {
        return errors.New("invalid alias format")
    }
    
    if req.WorkspaceId == "" {
        return errors.New("workspace ID is required")
    }
    
    return nil
}
```

### Permission Checks

```go
// Permission validation
func (s *service) checkProjectPermission(ctx context.Context, projectID string, user *User, action string) error {
    project, err := s.projectRepo.FindByID(ctx, projectID)
    if err != nil {
        return err
    }
    
    // Check workspace membership
    if !user.HasWorkspaceAccess(project.WorkspaceId) {
        return ErrUnauthorized
    }
    
    // Check specific permission
    return s.authz.CheckPermission(ctx, user, project, action)
}
```

## Flow Integration Examples

### Flow Workflow Integration

```go
// Flow accessing CMS data for processing
func (f *flowProcessor) ProcessCMSData(ctx context.Context, projectID, modelID string) error {
    // List items from CMS
    items, err := f.cmsClient.ListItems(ctx, &cms.ListItemsRequest{
        ProjectId: projectID,
        ModelId:   modelID,
        PageSize:  100,
    })
    if err != nil {
        return err
    }
    
    // Process items through Flow workflow
    for _, item := range items.Items {
        processedData, err := f.processItem(item)
        if err != nil {
            return err
        }
        
        // Send to next workflow node
        err = f.sendToNextNode(processedData)
        if err != nil {
            return err
        }
    }
    
    return nil
}
```

### Real-time Data Processing

```go
// Flow real-time processing of CMS updates
func (f *flowProcessor) HandleCMSUpdate(ctx context.Context, update *CMSUpdateEvent) error {
    // Get updated item
    item, err := f.cmsClient.GetItem(ctx, update.ItemID)
    if err != nil {
        return err
    }
    
    // Process through Flow pipeline
    result, err := f.processItem(item)
    if err != nil {
        return err
    }
    
    // Publish result
    return f.publishResult(result)
}
```

## Testing Strategies

### Unit Testing

```go
// Unit test for CMS service
func TestCreateProject(t *testing.T) {
    ctrl := gomock.NewController(t)
    defer ctrl.Finish()
    
    mockRepo := mocks.NewMockProjectRepository(ctrl)
    service := NewCMSService(mockRepo)
    
    req := &CreateProjectRequest{
        Name:        "Test Project",
        Alias:       "test-project",
        WorkspaceId: "workspace-123",
        Visibility:  Visibility_PUBLIC,
    }
    
    mockRepo.EXPECT().
        Create(gomock.Any(), gomock.Any()).
        Return(nil)
    
    resp, err := service.CreateProject(context.Background(), req)
    
    assert.NoError(t, err)
    assert.NotNil(t, resp)
    assert.Equal(t, req.Name, resp.Project.Name)
}
```

### Integration Testing

```go
// Integration test with real CMS
func TestCMSIntegration(t *testing.T) {
    client := setupCMSClient(t)
    
    // Create project
    project, err := client.CreateProject(context.Background(), &CreateProjectRequest{
        Name:        "Integration Test",
        Alias:       "integration-test",
        WorkspaceId: testWorkspaceID,
        Visibility:  Visibility_PRIVATE,
    })
    require.NoError(t, err)
    
    // List projects
    projects, err := client.ListProjects(context.Background(), &ListProjectsRequest{
        WorkspaceId: testWorkspaceID,
    })
    require.NoError(t, err)
    assert.Contains(t, projects.Projects, project.Project)
    
    // Cleanup
    _, err = client.DeleteProject(context.Background(), &DeleteProjectRequest{
        ProjectId: project.Project.Id,
    })
    require.NoError(t, err)
}
```

## Future Enhancements

### Planned Features

1. **Streaming APIs**: Real-time data streaming for large datasets
2. **Batch Operations**: Bulk create/update/delete operations
3. **Advanced Filtering**: Complex query capabilities with filtering
4. **Schema Migration**: Automated schema version management
5. **Webhook Support**: Event-driven notifications for CMS changes
6. **Multi-language Support**: Localization for field names and descriptions

### Schema Evolution

```proto
// Future schema versioning support
message Schema {
  string schema_id = 1;
  repeated SchemaField fields = 2;
  int32 version = 3;                    // Schema version
  string migration_script = 4;          // Migration instructions
  repeated string deprecated_fields = 5; // Deprecated field IDs
}
```

### Enhanced Field Types

```proto
// Additional field types under consideration
enum SchemaFieldType {
  // Existing types...
  
  // New types
  JSON = 17;             // JSON object field
  Color = 18;            // Color picker field
  Rating = 19;           // Rating/stars field
  Slider = 20;           // Range slider field
  MultiSelect = 21;      // Multiple selection field
  Location = 22;         // Address/location field
  Phone = 23;            // Phone number field
  Email = 24;            // Email validation field
}
```

## Conclusion

The `schema.proto` file provides a comprehensive and well-structured gRPC interface for CMS operations within the Re:Earth ecosystem. The design emphasizes:

- **Type Safety**: Strong typing through protocol buffers
- **Flexibility**: Dynamic field system supporting diverse content types
- **Scalability**: Efficient pagination and indexing strategies
- **Security**: Comprehensive authentication and authorization
- **Integration**: Seamless integration with Dashboard and Flow components
- **Extensibility**: Well-designed for future enhancements

This protocol buffer definition serves as the foundation for reliable, type-safe communication between Re:Earth Flow and the CMS service, enabling powerful workflow processing of spatial and content management data.