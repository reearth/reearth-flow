# Server Architecture

## Clean Architecture Layers

### 1. Domain Layer (`pkg/`)

- Pure business entities and domain logic
- No external dependencies
- Core types: Project, Job, Workflow, Deployment, etc.

### 2. Use Case Layer (`internal/usecase/`)

- Application business rules
- Orchestrates domain entities
- Defines repository and gateway interfaces

### 3. Infrastructure Layer (`internal/infrastructure/`)

- Implements repository interfaces
- External service integrations (MongoDB, Redis, GCP)
- Authentication providers (Auth0/Cognito)

### 4. Adapter Layer (`internal/adapter/`)

- GraphQL API implementation
- Converts between GraphQL models and domain entities
- DataLoader pattern for N+1 query optimization

## GraphQL API Design

### Schema Organization (`gql/*.graphql`)

- `_shared.graphql` - Common types and directives
- `project.graphql` - Project management
- `job.graphql` - Job execution and monitoring
- `deployment.graphql` - Deployment management
- `workspace.graphql` - Multi-tenant workspace
- `user.graphql` - User and authentication
- `document.graphql` - Collaborative editing (Yjs)
- `node.graphql` & `log.graphql` - Real-time workflow state

### Resolver Pattern

- Mutation resolvers in `resolver_mutation_*.go`
- Query resolvers in `resolver_query_*.go`
- Field resolvers in `resolver_*.go`
- DataLoaders in `loader_*.go` for efficient batch fetching

## Real-time Collaboration

### Components

1. **WebSocket Server** (Rust) - Handles Y-WebSocket protocol
2. **Document API** (Go) - Manages collaborative document state
3. **Pub/Sub** - Event streaming for workflow updates

### Flow

- UI connects to WebSocket server for Yjs synchronization
- WebSocket server authenticates via API server
- Real-time workflow changes propagate through Yjs
- State persisted to MongoDB via GraphQL mutations

## Job Execution Coordination

### Architecture

1. **Job Creation** - API receives workflow execution request
2. **GCP Batch** - Submits job to Google Cloud Batch
3. **Engine Execution** - Rust engine processes workflow
4. **Status Updates** - Pub/Sub streams logs and status
5. **Monitoring** - API provides real-time job status via subscriptions

### Key Services

- `pkg/job/monitor/` - Job status monitoring
- `internal/infrastructure/gcp/` - Google Cloud integration
- `subscriber/` - Processes execution logs from Pub/Sub

## Multi-Service Coordination

### Service Dependencies

1. **API Server** (`api/`) - Primary GraphQL API, authentication, job coordination
2. **WebSocket Server** (`websocket/`) - Real-time collaboration, depends on API for authentication
3. **Subscriber** (`subscriber/`) - Processes Pub/Sub events, writes to MongoDB and Redis

### Starting Services

```bash
# 1. Start MongoDB
cd server/api/ && make run-db

# 2. Build WebSocket server (one-time)
cd server/websocket/ && cargo build --release

# 3. Start API server (must start before WebSocket!)
cd server/api/ && make run-app

# 4. Start WebSocket server
cd server/websocket/ && cargo run
```

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
- **Parameterized queries**: Use MongoDB parameterized queries
- **Rate limiting**: Implement at API gateway level
- **Secrets**: Never commit credentials or API keys
