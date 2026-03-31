# Server Development Guide

## Core Technologies

- **Go 1.24** with GraphQL (gqlgen) for API development
- **MongoDB** for primary data storage (workflows, projects, jobs)
- **Redis** for caching and real-time data
- **Google Cloud Platform** for batch job execution and storage
- **Auth0/Cognito** for authentication and authorization
- **Thrift** for internal service communication
- **Protocol Buffers** for CMS schema definitions

## Project Structure

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

## Adding New GraphQL Entities

1. **Define GraphQL schema** in `gql/{entity}.graphql`
2. **Create domain model** in `pkg/{entity}/`
3. **Define repository interface** in `internal/usecase/repo/`
4. **Implement repository** in `internal/infrastructure/mongo/`
5. **Create use case** in `internal/usecase/interactor/`
6. **Implement resolvers** in `internal/adapter/gql/`
7. **Run `make gql`** to generate code

## Repository Pattern

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

## Error Handling

- Use domain-specific errors in `pkg/*/errors.go`
- Convert to GraphQL errors in adapter layer
- Include operation context for debugging
- Log errors with structured logging

## Testing Strategy

- **Unit tests**: Domain logic and use cases
- **Integration tests**: Repository implementations
- **E2E tests**: GraphQL API endpoints (`e2e/`)
- Use `internal/testutil/` for test helpers

## Modifying GraphQL Schema

1. Edit schema files in `gql/*.graphql`
2. Run `make gql` to regenerate resolvers
3. Implement resolver logic in `internal/adapter/gql/`
4. Update UI types with `yarn gql` in `ui/`

## Adding New Actions

When adding workflow actions:

1. Update action schemas in `pkg/schema/`
2. Coordinate with engine team for implementation
3. Update action registry and validation
4. Test with UI workflow editor

## Database Migrations

- Migration files in `internal/infrastructure/mongo/migration/`
- Run automatically on server startup
- Add new migrations to `migrations.go`

## RBAC Policies

- Policy definitions in YAML format
- Generate with `make gen-policies`
- Implemented in `internal/rbac/`

## Environment Configuration

### Database

- `REEARTH_DB` - MongoDB connection string
- `REEARTH_DB_NAME` - Database name
- `REDIS_URL` - Redis connection string

### Authentication

- `REEARTH_AUTH_ISS` - JWT issuer
- `REEARTH_AUTH_AUD` - JWT audience
- `REEARTH_AUTH_ALG` - JWT algorithm
- Auth0/Cognito provider-specific variables

### Google Cloud Platform

- `GOOGLE_APPLICATION_CREDENTIALS` - GCP service account key
- `REEARTH_GCP_PROJECT` - GCP project ID
- `REEARTH_GCP_REGION` - Batch job region
- `REEARTH_GCS_BUCKET` - Cloud Storage bucket

### Application

- `REEARTH_PORT` - HTTP server port
- `REEARTH_HOST` - Server hostname
- `REEARTH_DEV` - Development mode flag
