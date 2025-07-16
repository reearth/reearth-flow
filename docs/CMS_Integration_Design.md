# CMS Integration Design Document

## Overview

This document describes the design and architecture for integrating the Re:Earth CMS with the Re:Earth Flow platform. The integration enables Flow to interact with CMS projects, models, and data items through a standardized gRPC interface.

## Architecture

### System Components

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│                 │    │                  │    │                 │
│  Re:Earth Flow  │◄──►│  CMS Integration │◄──►│  Re:Earth CMS   │
│                 │    │     Gateway      │    │                 │
└─────────────────┘    └──────────────────┘    └─────────────────┘
         │                       │                       │
         │                       │                       │
    ┌────▼────┐             ┌────▼────┐             ┌────▼────┐
    │   UI    │             │  gRPC   │             │ CMS API │
    │ Layer   │             │ Client  │             │ Server  │
    └─────────┘             └─────────┘             └─────────┘
```

### Key Architecture Patterns

1. **Gateway Pattern**: CMS integration through dedicated gateway service
2. **Clean Architecture**: Domain-driven design with clear layer separation
3. **Protocol Buffers**: Type-safe communication via gRPC
4. **Authentication**: JWT-based authentication with M2M token support

## Data Model

### Core Entities

#### Project
```proto
message Project {
  string id = 1;
  string name = 2;
  string alias = 3;
  optional string description = 4;
  optional string license = 5;
  optional string readme = 6;
  string workspace_id = 7;
  Visibility visibility = 8;
  google.protobuf.Timestamp created_at = 9;
  google.protobuf.Timestamp updated_at = 10;
}
```

#### Model
```proto
message Model {
  string id = 1;
  string project_id = 2;
  string name = 3;
  string description = 4;
  string key = 5;
  Schema schema = 6;
  string public_api_ep = 7;
  string editor_url = 8;
  google.protobuf.Timestamp created_at = 9;
  google.protobuf.Timestamp updated_at = 10;
}
```

#### Item
```proto
message Item {
  string id = 1;
  map<string, google.protobuf.Any> fields = 2;
  google.protobuf.Timestamp created_at = 5;
  google.protobuf.Timestamp updated_at = 6;
}
```

### Schema Field Types

The system supports comprehensive field types for flexible data modeling:

- **Text Types**: `Text`, `TextArea`, `RichText`, `MarkdownText`
- **Media Types**: `Asset`, `URL`
- **Data Types**: `Date`, `Integer`, `Number`, `Bool`
- **Selection Types**: `Select`, `Tag`, `Checkbox`
- **Relationship Types**: `Reference`, `Group`
- **Geometry Types**: `GeometryObject`, `GeometryEditor`

## API Design

### Service Definition

```proto
service ReEarthCMS {
  // Project Management
  rpc CreateProject(CreateProjectRequest) returns (ProjectResponse) {}
  rpc UpdateProject(UpdateProjectRequest) returns (ProjectResponse) {}
  rpc DeleteProject(DeleteProjectRequest) returns (DeleteProjectResponse) {}
  rpc GetProject(ProjectRequest) returns (ProjectResponse) {}
  rpc ListProjects(ListProjectsRequest) returns (ListProjectsResponse) {}
  
  // Model & Schema Operations
  rpc ListModels(ListModelsRequest) returns (ListModelsResponse) {}
  
  // Content Operations
  rpc ListItems(ListItemsRequest) returns (ListItemsResponse) {}
  
  // Utility Operations
  rpc CheckAliasAvailability(AliasAvailabilityRequest) returns (AliasAvailabilityResponse) {}
  rpc GetModelGeoJSONExportURL(ExportRequest) returns (ExportURLResponse) {}
}
```

### Authentication

Authentication is handled through gRPC metadata:

```
Headers:
- authorization: Bearer <M2M_TOKEN>
- user-id: <USER_ID>
```

## Integration Patterns

### 1. Project Lifecycle Management

```go
// Project creation flow
func (u *CreateProject) Execute(ctx context.Context, input *CreateProjectInput, usr *user.User) error {
    // 1. Validate input
    // 2. Create project in CMS
    cmsProject, err := u.CMSGateway.CreateProject(ctx, &cms.CreateProjectRequest{
        WorkspaceId: input.WorkspaceID,
        Name:        input.Name,
        Description: &input.Description,
        Alias:       input.Alias,
        Visibility:  cms.Visibility(input.Scope),
    }, usr)
    
    // 3. Create corresponding Flow project
    // 4. Link CMS project to Flow project
    
    return nil
}
```

### 2. Data Synchronization

```go
// Model synchronization
func (s *CMSService) SyncModels(ctx context.Context, projectID string) error {
    models, err := s.cmsGateway.ListModels(ctx, projectID, user)
    if err != nil {
        return err
    }
    
    for _, model := range models {
        // Process model schema
        // Update local model cache
        // Trigger downstream updates
    }
    
    return nil
}
```

### 3. Content Processing

```go
// Item processing for workflows
func (p *CMSProcessor) ProcessItems(ctx context.Context, modelID string, filter *ItemFilter) ([]*ProcessedItem, error) {
    items, err := p.cmsGateway.ListItems(ctx, &cms.ListItemsRequest{
        ModelId:   modelID,
        ProjectId: filter.ProjectID,
        Page:      filter.Page,
        PageSize:  filter.PageSize,
    }, user)
    
    processedItems := make([]*ProcessedItem, 0, len(items.Items))
    for _, item := range items.Items {
        processed := p.processItem(item)
        processedItems = append(processedItems, processed)
    }
    
    return processedItems, nil
}
```

## Integration with Re:Earth Dashboard

### Current Implementation Reference

The Re:Earth Dashboard serves as the reference implementation for CMS integration patterns. The dashboard demonstrates a comprehensive CMS integration with the following architecture:

#### Core Implementation Components

1. **gRPC Client Implementation**:
   - **Location**: `/Users/xy/work/eukarya/reearth-dashboard/server/internal/infra/cmsgw/client.go`
   - **Protocol Buffer Schema**: `/Users/xy/work/eukarya/reearth-dashboard/server/proto/reearth_cms_v1.proto`
   - **Authentication**: Bearer token-based with gRPC metadata
   - **Connection Management**: Uses `grpc.NewClient` with configurable TLS

2. **Service Interface**:
   - **Location**: `/Users/xy/work/eukarya/reearth-dashboard/server/internal/usecase/gateway/cms.go`
   - **Gateway Pattern**: Abstraction layer for CMS operations
   - **Error Mapping**: Custom error types for different failure scenarios

3. **UI Components**:
   - **Project Data Viewer**: `/Users/xy/work/eukarya/reearth-dashboard/web/src/features/project/Data/index.tsx`
   - **Project Management**: Unified interface for CMS and Visualizer projects
   - **Data Tables**: Schema-based rendering with dynamic field types

#### Authentication Implementation

The dashboard implements multi-level authentication:

```go
// From reearth-dashboard/server/internal/infra/cmsgw/client.go
func (c *Client) addAuth(ctx context.Context, usr *user.User) context.Context {
    md := metadata.New(map[string]string{
        "authorization": fmt.Sprintf("Bearer %s", c.config.Token),
    })
    if usr != nil {
        md.Append("user-id", usr.ID().String())
    }
    return metadata.NewOutgoingContext(ctx, md)
}
```

#### Project Creation Workflow

The dashboard demonstrates a comprehensive project creation flow:

```go
// From reearth-dashboard/server/internal/usecase/projectuc/create_project.go
func (u *CreateProject) createCMSProject(ctx context.Context, workspaceID string, input *CreateProjectInput, usr *user.User) error {
    _, err := u.CMSGateWays.CreateProject(ctx, &cms.CreateProjectRequest{
        WorkspaceId: workspaceID,
        Name:        input.Name,
        Description: lo.ToPtr(input.Description),
        License:     lo.ToPtr(input.License),
        Readme:      lo.ToPtr(input.ReadMe),
        Alias:       input.Alias,
        Visibility:  cms.Visibility(cms.Visibility_value[strings.ToUpper(input.Scope)]),
    }, usr)
    return err
}
```

#### UI Integration Patterns

The dashboard UI demonstrates effective CMS integration patterns:

```typescript
// From reearth-dashboard/web/src/features/project/Data/index.tsx
const { data: project } = useGetProject({
  projectAlias, projectType, workspaceAlias,
});

const selectedModel = useMemo(
  () => project?.cms?.models?.find((model) => model.name === activeTab),
  [project?.cms?.models, activeTab]
);
```

#### Error Handling Implementation

The dashboard implements structured error handling:

```go
// From reearth-dashboard/server/internal/infra/cmsgw/client.go
var (
    ErrCMSNoGeometry         = status.Error(codes.Unknown, "no geometry field in this model")
    ErrCMSProjectNotFound    = status.Error(codes.Unknown, "not found")
    ErrCMSAliasAlreadyExists = status.Error(codes.Unknown, "project alias is already used")
)
```

#### Configuration Management

The dashboard uses environment-based configuration:

```go
// From reearth-dashboard/server/internal/di/config.go
type Config struct {
    GRPC struct {
        CMS struct {
            Endpoint string `default:"localhost:50051"`
            Token    string `default:""`
            UseTLS   bool   `default:"false"`
        } `config:"cms"`
    } `config:"grpc"`
}
```

### Dashboard Architecture Patterns

The dashboard follows a clean architecture approach:

- **Domain Layer**: CMS entities and business logic
- **Application Layer**: Use cases and service interfaces  
- **Infrastructure Layer**: gRPC clients and external service integration
- **Presentation Layer**: React components and API hooks

### Data Flow Architecture

The dashboard demonstrates the complete data flow:

1. **User Interface**: React components with TanStack Query
2. **REST API**: OpenAPI-generated endpoints
3. **Use Cases**: Business logic layer
4. **Gateway**: gRPC client abstraction
5. **CMS Service**: External CMS via gRPC

### Schema Field Type Support

The dashboard handles all CMS field types:

```typescript
// From reearth-dashboard/web/src/features/project/types.ts
type ProjectCmsSchema = {
  field_id?: string;
  name?: string;
  type?: 
    | "Text" | "TextArea" | "RichText" | "MarkdownText"
    | "Asset" | "URL" 
    | "Date" | "Bool" | "Integer" | "Number"
    | "Select" | "Tag" | "Checkbox"
    | "Reference" | "Group"
    | "GeometryObject" | "GeometryEditor";
  key?: string;
  description?: string;
};
```

### Permission and Authorization

The dashboard implements comprehensive RBAC:

```go
// From reearth-dashboard/server/internal/rbac/definitions.go
"cms.project.edit":      []string{"owner", "admin", "writer"},
"cms.asset.create":      []string{"owner", "admin", "writer"},
"cms.asset.delete":      []string{"owner", "admin", "writer"},
"cms.accessibility.manage": []string{"owner", "admin"},
"cms.integration.manage":   []string{"owner", "admin"},
```

### Testing Strategy

The dashboard includes comprehensive testing:

```typescript
// From reearth-dashboard/e2e/tests/cmsProject.spec.ts
test("create CMS project", async ({ page }) => {
  await page.goto("/");
  await page.getByRole("button", { name: "Create Project" }).click();
  await page.getByRole("button", { name: "CMS" }).click();
  await page.getByLabel("Project Name").fill("Test CMS Project");
  await page.getByLabel("Project Alias").fill("test-cms-project");
  await page.getByRole("button", { name: "Create" }).click();
  
  await expect(page.getByText("Test CMS Project")).toBeVisible();
});
```

## Configuration

### Environment Variables

Based on the dashboard implementation, the Flow system should use similar configuration patterns:

```bash
# CMS gRPC Endpoint - following dashboard pattern
REEARTH_FLOW_GRPC_ENDPOINT_CMS=localhost:50051

# Authentication - M2M token for service communication
REEARTH_FLOW_GRPC_TOKEN_CMS=<M2M_TOKEN>

# TLS Configuration - secure communication
REEARTH_FLOW_GRPC_USE_TLS_CMS=false

# Additional Dashboard-inspired configuration
REEARTH_FLOW_CMS_EDITOR_URL=http://localhost:8080/workspace/%s/project/%s
REEARTH_FLOW_CMS_ENDPOINT_URL=http://localhost:8080/workspace/%s/project/%s/schema/%s
```

### Dashboard Configuration Reference

The dashboard uses a structured configuration approach:

```go
// From reearth-dashboard/server/internal/di/config.go
type Config struct {
    GRPC struct {
        CMS struct {
            Endpoint string `default:"localhost:50051" config:"endpoint"`
            Token    string `default:"" config:"token"`
            UseTLS   bool   `default:"false" config:"use_tls"`
        } `config:"cms"`
    } `config:"grpc"`
    
    // Additional CMS configuration
    CMS struct {
        EditorURL    string `default:"http://localhost:8080/workspace/%s/project/%s" config:"editor_url"`
        EndpointURL  string `default:"http://localhost:8080/workspace/%s/project/%s/schema/%s" config:"endpoint_url"`
    } `config:"cms"`
}

### Dependency Injection

Flow should follow the dashboard's dependency injection pattern using Wire:

```go
// Following dashboard pattern from reearth-dashboard/server/internal/di/wire.go
func NewCMSGateway(config *Config) (gateway.CMSGateway, error) {
    var opts []grpc.DialOption
    
    if config.GRPC.CMS.UseTLS {
        opts = append(opts, grpc.WithTransportCredentials(credentials.NewTLS(&tls.Config{})))
    } else {
        opts = append(opts, grpc.WithTransportCredentials(insecure.NewCredentials()))
    }
    
    conn, err := grpc.NewClient(config.GRPC.CMS.Endpoint, opts...)
    if err != nil {
        return nil, err
    }
    
    client := cms.NewReEarthCMSClient(conn)
    return cmsgw.NewClient(client, &cmsgw.Config{
        Token:  config.GRPC.CMS.Token,
        UseTLS: config.GRPC.CMS.UseTLS,
    }), nil
}

// Wire provider set
var CMSGatewaySet = wire.NewSet(
    NewCMSGateway,
    wire.Bind(new(gateway.CMSGateway), new(*cmsgw.Client)),
)
```

## Error Handling

### Error Types

Based on the dashboard implementation, Flow should use similar error definitions:

```go
// From reearth-dashboard/server/internal/infra/cmsgw/client.go
var (
    ErrCMSNoGeometry         = status.Error(codes.Unknown, "no geometry field in this model")
    ErrCMSProjectNotFound    = status.Error(codes.Unknown, "not found")
    ErrCMSAliasAlreadyExists = status.Error(codes.Unknown, "project alias is already used")
    ErrCMSModelNotFound      = status.Error(codes.Unknown, "model not found")
    ErrCMSUnauthorized       = status.Error(codes.Unauthenticated, "unauthorized")
    ErrCMSInvalidInput       = status.Error(codes.InvalidArgument, "invalid input")
)
```

### Error Handling with Caller Context

The dashboard uses runtime caller information for better error tracing:

```go
// From reearth-dashboard/server/internal/infrastructure/ucutil/caller.go
func WrapWithCaller(msg string, err error) error {
    if err == nil {
        return nil
    }
    
    _, file, line, _ := runtime.Caller(1)
    base := filepath.Base(file)
    return fmt.Errorf("%s at %s:%d: %w", msg, base, line, err)
}
```

### Error Propagation

```go
func (c *cmsGateway) GetProject(ctx context.Context, projectID string, usr *user.User) (*Project, error) {
    resp, err := c.client.GetProject(c.addAuth(ctx, usr), &cms.ProjectRequest{
        ProjectIdOrAlias: projectID,
    })
    
    if err != nil {
        if status.Code(err) == codes.NotFound {
            return nil, ErrProjectNotFound
        }
        return nil, fmt.Errorf("failed to get project: %w", err)
    }
    
    return convertProject(resp.Project), nil
}
```

## Security Considerations

### Authentication & Authorization

1. **M2M Authentication**: Machine-to-machine tokens for service communication
2. **User Context**: User identity propagation through gRPC metadata
3. **Workspace Isolation**: Projects are isolated by workspace
4. **Visibility Control**: Public/private project visibility settings

### Data Protection

1. **TLS Encryption**: gRPC communication over TLS
2. **Input Validation**: Comprehensive input sanitization
3. **Rate Limiting**: API rate limiting to prevent abuse
4. **Audit Logging**: All CMS operations are logged

## Performance Considerations

### Caching Strategy

```go
type CMSCache struct {
    projects map[string]*Project
    models   map[string][]*Model
    ttl      time.Duration
}

func (c *CMSCache) GetProject(projectID string) (*Project, bool) {
    // Cache lookup with TTL check
}
```

### Pagination

```go
// Efficient pagination for large datasets
func (s *CMSService) ListItems(ctx context.Context, req *ListItemsRequest) (*ListItemsResponse, error) {
    const defaultPageSize = 50
    const maxPageSize = 1000
    
    pageSize := req.PageSize
    if pageSize == 0 {
        pageSize = defaultPageSize
    }
    if pageSize > maxPageSize {
        pageSize = maxPageSize
    }
    
    // Implement cursor-based pagination for better performance
}
```

## Testing Strategy

### Unit Testing

```go
func TestCMSGateway_CreateProject(t *testing.T) {
    ctrl := gomock.NewController(t)
    defer ctrl.Finish()
    
    mockClient := mocks.NewMockReEarthCMSClient(ctrl)
    gateway := &cmsGateway{client: mockClient}
    
    // Test implementation
}
```

### Integration Testing

```go
func TestCMSIntegration(t *testing.T) {
    // Test against real CMS instance
    // Verify end-to-end functionality
}
```

## Dashboard UI Patterns and Workflows

### Project Management Interface

The dashboard demonstrates effective UI patterns for CMS project management:

#### Project List Interface
```typescript
// From reearth-dashboard/web/src/features/Dashboard/Projects.tsx
const ProjectList: React.FC = () => {
  const { data: projects, isLoading } = useGetWorkspaceProjects(
    workspaceAlias,
    "cms", // Project type filter
    { scope: "all", search: searchTerm }
  );

  return (
    <div>
      {projects?.map(project => (
        <ProjectCard 
          key={project.id}
          project={project}
          onDelete={handleDelete}
          onEdit={handleEdit}
        />
      ))}
    </div>
  );
};
```

#### Project Creation Form
```typescript
// From reearth-dashboard/web/src/features/NewProject/index.tsx
const NewProjectForm: React.FC = () => {
  const { createProject, isPending } = useCreateProject();
  
  const handleSubmit = (data: ProjectCreateInput) => {
    createProject(workspaceAlias, "cms", {
      name: data.name,
      alias: data.alias,
      description: data.description,
      license: data.license,
      readme: data.readme,
      scope: data.scope, // "public" | "private"
    });
  };

  return (
    <form onSubmit={handleSubmit}>
      <Input label="Project Name" {...register("name")} />
      <Input label="Project Alias" {...register("alias")} />
      <Select label="Scope" options={["public", "private"]} />
      <Button type="submit" disabled={isPending}>
        Create Project
      </Button>
    </form>
  );
};
```

#### CMS Data Viewer
```typescript
// From reearth-dashboard/web/src/features/project/Data/index.tsx
const CMSDataViewer: React.FC = () => {
  const { data: project } = useGetProject({ projectAlias, projectType: "cms" });
  const [activeTab, setActiveTab] = useState<string>();

  const selectedModel = useMemo(
    () => project?.cms?.models?.find(model => model.name === activeTab),
    [project?.cms?.models, activeTab]
  );

  return (
    <div>
      <Tabs value={activeTab} onValueChange={setActiveTab}>
        {project?.cms?.models?.map(model => (
          <TabsContent key={model.name} value={model.name}>
            <DataTable
              data={model.items}
              schema={model.schema}
              endpoint={model.endpoint}
            />
          </TabsContent>
        ))}
      </Tabs>
    </div>
  );
};
```

### API Integration Patterns

#### React Query Integration
```typescript
// From reearth-dashboard/web/src/api/rest/hooks/project.ts
export const useCreateProject = (options?: CommonMutationOptions) => {
  const { mutate, isPending, error } = useMutation({
    mutationFn: ({ workspaceAlias, projectType, body }) =>
      client.POST("/api/workspaces/{workspace_alias}/projects/{project_type}", {
        params: { path: { workspace_alias: workspaceAlias, project_type: projectType } },
        body,
      }),
    onSuccess: (data) => {
      queryClient.invalidateQueries({ queryKey: ["projects"] });
      options?.onSuccess?.(data);
    },
    onError: options?.onError,
  });

  return { createProject: mutate, isPending, error };
};
```

#### Error Handling UI
```typescript
// From reearth-dashboard/web/src/api/rest/hooks/common.ts
export const useAPIErrorNotification = () => {
  const { toast } = useToast();

  return useCallback((error: any) => {
    const message = error?.response?.data?.message || "An error occurred";
    toast({
      title: "Error",
      description: message,
      variant: "destructive",
    });
  }, [toast]);
};
```

### Workflow Demonstrations

#### Complete Project Lifecycle
1. **Project Creation**:
   - User selects "Create Project" → "CMS"
   - Form validation with real-time alias checking
   - Workspace permission verification
   - Project creation with metadata

2. **Project Management**:
   - List view with search and filtering
   - Project cards with action buttons
   - Edit/delete operations with confirmation

3. **Data Visualization**:
   - Tabbed interface for different models
   - Data table with schema-based columns
   - Export functionality (when enabled)

#### Permission-Based UI
```typescript
// From reearth-dashboard/web/src/features/project/hooks/useProjectPermissions.ts
export const useProjectPermissions = (project: Project) => {
  const { user } = useAuth();
  
  const permissions = useMemo(() => ({
    canEdit: hasPermission(user, project, "cms.project.edit"),
    canDelete: hasPermission(user, project, "cms.project.delete"),
    canManageAssets: hasPermission(user, project, "cms.asset.create"),
  }), [user, project]);

  return permissions;
};
```

## Future Enhancements

### Planned Features

1. **Real-time Synchronization**: WebSocket-based real-time updates
2. **Batch Operations**: Bulk create/update/delete operations
3. **Schema Validation**: Enhanced schema validation and migration
4. **Event Sourcing**: Event-driven architecture for audit trails
5. **GraphQL Subscriptions**: Real-time subscriptions for UI updates

### Performance Optimizations

1. **Connection Pooling**: gRPC connection pool management
2. **Streaming**: Streaming APIs for large datasets
3. **Compression**: gRPC compression for reduced bandwidth
4. **Caching**: Distributed caching for frequently accessed data

### Dashboard-Inspired Enhancements

1. **Advanced UI Components**: Rich data tables with sorting and filtering
2. **Real-time Notifications**: Toast-based error and success notifications
3. **Progressive Loading**: Skeleton screens and loading states
4. **Accessibility**: ARIA compliance and keyboard navigation
5. **Responsive Design**: Mobile-first responsive layouts

## Conclusion

The CMS integration provides a robust, scalable foundation for connecting Re:Earth Flow with CMS data sources. The design emphasizes:

- **Type Safety**: Protocol buffers ensure consistent data structures
- **Scalability**: Efficient pagination and caching strategies
- **Security**: Comprehensive authentication and authorization
- **Maintainability**: Clean architecture with clear separation of concerns
- **Extensibility**: Well-defined interfaces for future enhancements

This integration enables Flow to leverage CMS content effectively while maintaining system reliability and performance.