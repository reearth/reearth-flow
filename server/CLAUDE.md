# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with the Server component of this repository.

For monorepo architecture, cross-component workflows, and general project guidance, see @../CLAUDE.md

## Architecture Overview

Re:Earth Flow Server is the backend API and coordination layer for the geospatial workflow platform. It provides GraphQL APIs, manages workflow state, coordinates job execution, and handles real-time collaboration.

### Core Technologies

- **Go 1.24** with GraphQL (gqlgen) for API development
- **MongoDB** for primary data storage (workflows, projects, jobs)
- **Redis** for caching and real-time data
- **Google Cloud Platform** for batch job execution and storage
- **Auth0/Cognito** for authentication and authorization
- **Thrift** for internal service communication
- **Protocol Buffers** for CMS schema definitions

## Development Commands

```bash
cd server/api/

# Development
make run-app          # Start API server
make run-db           # Start MongoDB via Docker Compose
make build            # Build the application

# Code Quality
make lint             # Run golangci-lint
make test             # Run unit tests with race detection
make e2e              # Run end-to-end tests

# Code Generation
make gql              # Generate GraphQL resolvers and types
make gen-policies     # Generate RBAC policies
make gen-thrift       # Generate Thrift code
make proto            # Generate gRPC code from protobuf
```

## Project Structure

The server follows **Clean Architecture** with **Domain-Driven Design (DDD)** patterns:

```
server/
├── api/                          # Main GraphQL API server
│   ├── cmd/reearth-flow/         # Application entry point
│   ├── internal/
│   │   ├── adapter/              # External interfaces (GraphQL, gRPC)
│   │   │   └── gql/              # GraphQL resolvers and loaders
│   │   ├── app/                  # Application setup and config
│   │   ├── infrastructure/       # External dependencies
│   │   │   ├── auth/             # Auth0/Cognito integration
│   │   │   ├── mongo/            # MongoDB repositories
│   │   │   ├── redis/            # Redis caching
│   │   │   └── fs/               # File storage
│   │   └── usecase/              # Business logic layer
│   │       ├── interactor/       # Use case implementations
│   │       ├── interfaces/       # Use case contracts
│   │       ├── repo/             # Repository interfaces
│   │       └── gateway/          # External service interfaces
│   ├── pkg/                      # Domain models and business entities
│   │   ├── project/              # Project domain
│   │   ├── job/                  # Job execution domain
│   │   ├── deployment/           # Deployment domain
│   │   ├── workflow/             # Workflow definition domain
│   │   └── schema/               # Action schema domain
│   └── gql/                      # GraphQL schema definitions
├── subscriber/                   # Log processing service
│   └── internal/                 # Pub/Sub event processing
└── websocket/                    # Real-time collaboration (Rust)
    └── src/                      # Y-WebSocket server implementation
```

## Key Architecture Patterns

### Clean Architecture Layers

1. **Domain Layer** (`pkg/`)
   - Pure business entities and domain logic
   - No external dependencies
   - Defines core types: Project, Job, Workflow, Deployment, etc.

2. **Use Case Layer** (`internal/usecase/`)
   - Application business rules
   - Orchestrates domain entities
   - Defines repository and gateway interfaces

3. **Infrastructure Layer** (`internal/infrastructure/`)
   - Implements repository interfaces
   - External service integrations (MongoDB, Redis, GCP)
   - Authentication providers

4. **Adapter Layer** (`internal/adapter/`)
   - GraphQL API implementation
   - Converts between GraphQL models and domain entities
   - DataLoader pattern for N+1 query optimization

### GraphQL API Design

**Schema Organization** (`gql/*.graphql`):
- `_shared.graphql` - Common types and directives
- `project.graphql` - Project management
- `job.graphql` - Job execution and monitoring
- `deployment.graphql` - Deployment management
- `workspace.graphql` - Multi-tenant workspace
- `user.graphql` - User and authentication
- `document.graphql` - Collaborative editing (Yjs)
- `node.graphql` & `log.graphql` - Real-time workflow state

**Resolver Pattern**:
- Mutation resolvers in `resolver_mutation_*.go`
- Query resolvers in `resolver_query_*.go`
- Field resolvers in `resolver_*.go`
- DataLoaders in `loader_*.go` for efficient batch fetching

### Real-time Collaboration

**Components**:
1. **WebSocket Server** (Rust) - Handles Y-WebSocket protocol
2. **Document API** (Go) - Manages collaborative document state
3. **Pub/Sub** - Event streaming for workflow updates

**Flow**:
- UI connects to WebSocket server for Yjs synchronization
- WebSocket server authenticates via API server
- Real-time workflow changes propagate through Yjs
- State persisted to MongoDB via GraphQL mutations

### Job Execution Coordination

**Architecture**:
1. **Job Creation** - API receives workflow execution request
2. **GCP Batch** - Submits job to Google Cloud Batch
3. **Engine Execution** - Rust engine processes workflow
4. **Status Updates** - Pub/Sub streams logs and status
5. **Monitoring** - API provides real-time job status via subscriptions

**Key Services**:
- `pkg/job/monitor/` - Job status monitoring
- `internal/infrastructure/gcp/` - Google Cloud integration
- `subscriber/` - Processes execution logs from Pub/Sub

## Development Patterns

### Adding New GraphQL Entities

1. **Define GraphQL schema** in `gql/{entity}.graphql`
2. **Create domain model** in `pkg/{entity}/`
3. **Define repository interface** in `internal/usecase/repo/`
4. **Implement repository** in `internal/infrastructure/mongo/`
5. **Create use case** in `internal/usecase/interactor/`
6. **Implement resolvers** in `internal/adapter/gql/`
7. **Run `make gql`** to generate code

### Repository Pattern

All data access goes through repository interfaces:

```go
// Define interface
type ProjectRepo interface {
    FindByID(context.Context, ProjectID) (*Project, error)
    Save(context.Context, *Project) error
}

// Implement in infrastructure layer
type projectRepo struct {
    client *mongo.Collection
}
```

### Error Handling

- Use domain-specific errors in `pkg/*/errors.go`
- Convert to GraphQL errors in adapter layer
- Include operation context for debugging
- Log errors with structured logging

### Testing Strategy

- **Unit tests**: Domain logic and use cases
- **Integration tests**: Repository implementations
- **E2E tests**: GraphQL API endpoints (`e2e/`)
- Use `internal/testutil/` for test helpers

## Environment Configuration

Server-specific environment variables:

**Database**:
- `REEARTH_DB` - MongoDB connection string
- `REEARTH_DB_NAME` - Database name
- `REDIS_URL` - Redis connection string

**Authentication**:
- `REEARTH_AUTH_ISS` - JWT issuer
- `REEARTH_AUTH_AUD` - JWT audience
- `REEARTH_AUTH_ALG` - JWT algorithm
- Auth0/Cognito provider-specific variables

**Google Cloud Platform**:
- `GOOGLE_APPLICATION_CREDENTIALS` - GCP service account key
- `REEARTH_GCP_PROJECT` - GCP project ID
- `REEARTH_GCP_REGION` - Batch job region
- `REEARTH_GCS_BUCKET` - Cloud Storage bucket

**Application**:
- `REEARTH_PORT` - HTTP server port
- `REEARTH_HOST` - Server hostname
- `REEARTH_DEV` - Development mode flag

See @../CLAUDE.md for complete environment configuration across all components.

## Multi-Service Coordination

### Service Dependencies

1. **API Server** (`api/`)
   - Primary GraphQL API
   - Authentication service
   - Job coordination

2. **WebSocket Server** (`websocket/`)
   - Real-time collaboration
   - Depends on API for authentication
   - **IMPORTANT**: API server must be running first

3. **Subscriber** (`subscriber/`)
   - Processes Pub/Sub events
   - Updates job logs and status
   - Writes to MongoDB and Redis

### Starting Services

```bash
# 1. Start MongoDB
cd server/api/
make run-db

# 2. Build WebSocket server (one-time)
cd server/websocket/
cargo build --release

# 3. Start API server (must start before WebSocket!)
cd server/api/
make run-app

# 4. Start WebSocket server
cd server/websocket/
cargo run
```

## Common Development Tasks

### Modifying GraphQL Schema

1. Edit schema files in `gql/*.graphql`
2. Run `make gql` to regenerate resolvers
3. Implement resolver logic in `internal/adapter/gql/`
4. Update UI types with `yarn gql` in `ui/`

### Adding New Actions

When adding workflow actions:
1. Update action schemas in `pkg/schema/`
2. Coordinate with engine team for implementation
3. Update action registry and validation
4. Test with UI workflow editor

### Database Migrations

- Migration files in `internal/infrastructure/mongo/migration/`
- Run automatically on server startup
- Add new migrations to `migrations.go`

### RBAC Policies

- Policy definitions in YAML format
- Generate with `make gen-policies`
- Implemented in `internal/rbac/`

## Integration Points

### With Engine
- **Workflow Definition**: JSON/YAML format defined by server
- **Job Execution**: Server submits to GCP Batch, engine processes
- **Results**: Engine writes to cloud storage, server tracks status

### With UI
- **GraphQL API**: All data access and mutations
- **WebSocket**: Real-time collaboration via Yjs
- **Authentication**: JWT tokens from Auth0

### With Cloud Services
- **GCP Batch**: Job execution orchestration
- **Cloud Storage**: Workflow results and assets
- **Pub/Sub**: Real-time event streaming

## Code Quality Requirements

**Before marking any task as complete, ALWAYS run:**

```bash
make lint             # Check code quality
make test             # Ensure all tests pass
make gql              # Regenerate GraphQL code if schema changed
```

## Performance Considerations

- **DataLoader pattern**: Batch and cache related entity fetches
- **Redis caching**: Cache frequently accessed data
- **MongoDB indexes**: Ensure proper indexing for queries
- **GraphQL complexity**: Monitor query complexity and depth
- **Connection pooling**: Reuse database connections

## Security Best Practices

- **Authentication**: Validate JWT tokens on all protected endpoints
- **Authorization**: Check workspace/project permissions via RBAC
- **Input validation**: Validate all GraphQL inputs
- **SQL injection**: Use parameterized queries (MongoDB)
- **Rate limiting**: Implement at API gateway level
- **Secrets**: Never commit credentials or API keys
