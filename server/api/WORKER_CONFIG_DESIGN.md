# Worker Configuration Management - Design Document

## Overview

This document describes the design and implementation of a feature that allows workspace administrators to configure worker compute resources through a GraphQL API. This enables per-workspace customization of worker parameters such as CPU, memory, disk size, and machine types.

## Current State

Currently, worker configuration is globally defined through environment variables at application startup:

- `WORKER_BOOT_DISK_SIZE_GB`: Boot disk size in GB (default: 50)
- `WORKER_BOOT_DISK_TYPE`: Boot disk type (default: "pd-balanced")
- `WORKER_CHANNEL_BUFFER_SIZE`: Channel buffer size (default: 256)
- `WORKER_COMPUTE_CPU_MILLI`: CPU in millicores (default: 2000)
- `WORKER_COMPUTE_MEMORY_MIB`: Memory in MiB (default: 2000)
- `WORKER_FEATURE_FLUSH_THRESHOLD`: Feature flush threshold (default: 512)
- `WORKER_IMAGE_URL`: Worker container image URL
- `WORKER_MACHINE_TYPE`: GCP machine type (default: "e2-standard-4")
- `WORKER_MAX_CONCURRENCY`: Maximum concurrency (default: 4)

These configurations apply to all users and workspaces uniformly.

## Problem Statement

Users need the ability to:
1. Configure worker resources per workspace to meet specific requirements
2. Adjust configurations in real-time without restarting the API server
3. Control who can modify these settings (admin-only access)
4. Override global defaults with workspace-specific values

## Proposed Solution

### Architecture

```
┌─────────────┐
│   Frontend  │
│   (GraphQL) │
└──────┬──────┘
       │
       ▼
┌─────────────────────┐
│  GraphQL Resolvers  │
│  + RBAC Check       │
└──────┬──────────────┘
       │
       ▼
┌─────────────────────┐
│  WorkerConfig       │
│  Interactor         │
└──────┬──────────────┘
       │
       ▼
┌─────────────────────┐
│  WorkerConfig       │
│  Repository (Mongo) │
└─────────────────────┘
       │
       ▼
┌─────────────────────┐
│   Batch Job         │
│   Submission        │
│   (uses config)     │
└─────────────────────┘
```

### Data Model

#### Domain Model: `WorkerConfig`

Located in `server/api/pkg/workerconfig/`:

```go
type WorkerConfig struct {
    id                      ID
    workspaceID             WorkspaceID
    bootDiskSizeGB          *int
    bootDiskType            *string
    channelBufferSize       *int
    computeCpuMilli         *int
    computeMemoryMib        *int
    featureFlushThreshold   *int
    imageURL                *string
    machineType             *string
    maxConcurrency          *int
    createdAt               time.Time
    updatedAt               time.Time
}
```

**Note**: All configuration fields are pointers to allow:
- Null values (inheriting from global defaults)
- Explicit override of specific values
- Partial updates

#### GraphQL Schema

```graphql
type WorkerConfig implements Node {
  id: ID!
  workspaceId: ID!
  bootDiskSizeGB: Int
  bootDiskType: String
  channelBufferSize: Int
  computeCpuMilli: Int
  computeMemoryMib: Int
  featureFlushThreshold: Int
  imageURL: String
  machineType: String
  maxConcurrency: Int
  createdAt: DateTime!
  updatedAt: DateTime!
}

input UpdateWorkerConfigInput {
  workspaceId: ID!
  bootDiskSizeGB: Int
  bootDiskType: String
  channelBufferSize: Int
  computeCpuMilli: Int
  computeMemoryMib: Int
  featureFlushThreshold: Int
  imageURL: String
  machineType: String
  maxConcurrency: Int
}

input ResetWorkerConfigInput {
  workspaceId: ID!
  fields: [String!]!
}

type UpdateWorkerConfigPayload {
  workerConfig: WorkerConfig!
}

type ResetWorkerConfigPayload {
  workerConfig: WorkerConfig!
}

extend type Query {
  workerConfig(workspaceId: ID!): WorkerConfig
  workerConfigDefaults: WorkerConfig!
}

extend type Mutation {
  updateWorkerConfig(input: UpdateWorkerConfigInput!): UpdateWorkerConfigPayload!
  resetWorkerConfig(input: ResetWorkerConfigInput!): ResetWorkerConfigPayload!
}
```

### Permission Model

#### RBAC Resource Definition

Add new resource type: `ResourceWorkerConfig = "workerconfig"`

**Permissions**:
- `read`: OWNER, MAINTAINER roles can view configurations
- `edit`: OWNER, MAINTAINER roles can modify configurations

#### Permission Checks

```go
// Read permission: any workspace member
checkPermission(ctx, rbac.ResourceWorkerConfig, rbac.ActionRead)

// Write permission: owners and maintainers only
checkPermission(ctx, rbac.ResourceWorkerConfig, rbac.ActionEdit)
```

### Configuration Hierarchy

1. **Workspace-specific config** (highest priority)
2. **Global defaults** (from environment variables)

When submitting a batch job:
```go
func (b *BatchRepo) SubmitJob(...) {
    // 1. Load workspace config from DB
    wsConfig := loadWorkspaceConfig(workspaceID)
    
    // 2. Merge with global defaults
    finalConfig := mergeConfigs(wsConfig, b.config)
    
    // 3. Use finalConfig for job submission
    computeResource := &batchpb.ComputeResource{
        CpuMilli:  finalConfig.ComputeCpuMilli,
        MemoryMib: finalConfig.ComputeMemoryMib,
        ...
    }
}
```

### API Operations

#### 1. Query Worker Config

```graphql
query GetWorkerConfig($workspaceId: ID!) {
  workerConfig(workspaceId: $workspaceId) {
    id
    workspaceId
    computeCpuMilli
    computeMemoryMib
    machineType
    # ... other fields
  }
}
```

**Returns**: Workspace-specific config or null if using global defaults

#### 2. Query Global Defaults

```graphql
query GetWorkerDefaults {
  workerConfigDefaults {
    bootDiskSizeGB
    bootDiskType
    computeCpuMilli
    computeMemoryMib
    machineType
    # ... other fields
  }
}
```

**Returns**: Current global configuration from environment variables

#### 3. Update Worker Config

```graphql
mutation UpdateWorkerConfig($input: UpdateWorkerConfigInput!) {
  updateWorkerConfig(input: $input) {
    workerConfig {
      id
      computeCpuMilli
      computeMemoryMib
      updatedAt
    }
  }
}
```

**Validation**:
- CPU must be positive
- Memory must be positive
- Machine type must be valid GCP machine type
- Disk size must be >= 10 GB

#### 4. Reset to Defaults

```graphql
mutation ResetWorkerConfig($input: ResetWorkerConfigInput!) {
  resetWorkerConfig(input: $input) {
    workerConfig {
      id
      workspaceId
    }
  }
}
```

**Purpose**: Reset specific fields or all fields to global defaults

### Implementation Plan

#### Phase 1: Domain & Data Layer

1. **Create domain model** (`pkg/workerconfig/`)
   - `workerconfig.go`: Core domain model
   - `builder.go`: Builder pattern for construction
   - `id.go`: ID type definitions

2. **Create repository interface** (`internal/usecase/repo/`)
   - `workerconfig.go`: Repository interface definition

3. **Implement MongoDB repository** (`internal/infrastructure/mongo/`)
   - `workerconfig.go`: MongoDB implementation
   - `mongodoc/workerconfig.go`: MongoDB document schema

#### Phase 2: Business Logic

4. **Create interactor** (`internal/usecase/interactor/`)
   - `workerconfig.go`: Business logic and use cases
   - Implement validation rules
   - Handle permission checks

#### Phase 3: API Layer

5. **Add GraphQL schema** (`gql/workerconfig.graphql`)
   - Define types, inputs, queries, mutations

6. **Implement resolvers** (`internal/adapter/gql/`)
   - `resolver_query.go`: Query resolvers
   - `resolver_mutation.go`: Mutation resolvers
   - `gqlmodel/workerconfig.go`: Model converters

#### Phase 4: RBAC Integration

7. **Add RBAC definitions** (`internal/rbac/definitions.go`)
   - Define `ResourceWorkerConfig`
   - Add permission rules

#### Phase 5: Batch Integration

8. **Update batch submission** (`internal/infrastructure/gcpbatch/batch.go`)
   - Load workspace-specific config
   - Merge with global defaults
   - Apply to job submission

9. **Update batch initialization** (`internal/app/repo.go`)
   - Pass repository to batch implementation

#### Phase 6: Migration & Testing

10. **Create migration support**
    - Existing workspaces use global defaults
    - No database migration needed (lazy creation)

11. **Add tests**
    - Unit tests for domain model
    - Repository tests
    - Interactor tests with permission checks
    - Integration tests for GraphQL API

### Security Considerations

1. **Authentication**: All operations require authenticated user
2. **Authorization**: Only workspace OWNER and MAINTAINER can modify configs
3. **Validation**: 
   - Prevent resource exhaustion (max CPU/memory limits)
   - Validate machine type against GCP options
   - Prevent malicious image URLs
4. **Audit**: Log all configuration changes with user ID and timestamp

### Validation Rules

```go
const (
    MinCPUMilli         = 100      // 0.1 CPU
    MaxCPUMilli         = 96000    // 96 CPUs
    MinMemoryMib        = 128      // 128 MB
    MaxMemoryMib        = 624000   // 624 GB
    MinBootDiskSizeGB   = 10       // 10 GB
    MaxBootDiskSizeGB   = 10000    // 10 TB
    MinChannelBuffer    = 1
    MaxChannelBuffer    = 10000
    MinMaxConcurrency   = 1
    MaxMaxConcurrency   = 100
)

var validDiskTypes = []string{
    "pd-standard",
    "pd-balanced",
    "pd-ssd",
}
```

### Frontend Integration

Frontend will need to:

1. **Display current config** in workspace settings
2. **Show global defaults** for reference
3. **Provide edit form** with validation
4. **Highlight overridden values** (different from defaults)
5. **Allow reset to defaults** per field or all fields
6. **Show permission-based UI** (only admins see edit buttons)

### Migration Strategy

**For existing deployments**:

1. No database migration required
2. Existing workspaces automatically use global defaults
3. WorkerConfig documents created on first update
4. Backward compatible with existing code

**Rollback plan**:

1. Remove GraphQL schema
2. Update batch submission to ignore workspace configs
3. Workers continue using global environment variables

### Future Enhancements

1. **Usage tracking**: Monitor resource usage per workspace
2. **Quotas**: Set resource limits per workspace tier
3. **Cost estimation**: Show estimated costs for configurations
4. **Templates**: Predefined configuration templates (small, medium, large)
5. **Notification**: Alert admins when jobs fail due to resource limits
6. **Audit log**: Detailed change history for compliance

## Database Schema

### MongoDB Collection: `workerconfigs`

```json
{
  "_id": "ObjectId",
  "workspace_id": "workspace_123",
  "boot_disk_size_gb": 100,
  "boot_disk_type": "pd-ssd",
  "channel_buffer_size": 512,
  "compute_cpu_milli": 4000,
  "compute_memory_mib": 8192,
  "feature_flush_threshold": 1024,
  "image_url": "gcr.io/my-project/worker:v2",
  "machine_type": "e2-standard-8",
  "max_concurrency": 8,
  "created_at": "2025-10-06T00:00:00Z",
  "updated_at": "2025-10-06T12:00:00Z"
}
```

**Indexes**:
- `workspace_id`: Unique index for fast lookup
- `updated_at`: For sorting and audit queries

## Error Handling

```go
var (
    ErrInvalidCPU      = errors.New("CPU must be between 100 and 96000 millicores")
    ErrInvalidMemory   = errors.New("Memory must be between 128 and 624000 MiB")
    ErrInvalidDiskSize = errors.New("Disk size must be between 10 and 10000 GB")
    ErrInvalidDiskType = errors.New("Invalid disk type")
    ErrPermissionDenied = errors.New("Only workspace owners and maintainers can modify worker config")
)
```

## Testing Strategy

1. **Unit Tests**:
   - Domain model validation
   - Configuration merging logic
   - Permission checks

2. **Integration Tests**:
   - GraphQL API endpoints
   - Database operations
   - RBAC enforcement

3. **E2E Tests**:
   - Create workspace
   - Update worker config
   - Submit job with custom config
   - Verify job uses correct resources

## Success Metrics

1. Users can configure worker resources per workspace
2. Configurations persist across API restarts
3. Permission system prevents unauthorized changes
4. Jobs respect workspace-specific configurations
5. System maintains backward compatibility

## References

- GCP Batch API: https://cloud.google.com/batch/docs
- GCP Machine Types: https://cloud.google.com/compute/docs/machine-types
- GraphQL Best Practices: https://graphql.org/learn/best-practices/
- RBAC in Go: https://github.com/casbin/casbin

