# AGENTS.md

Go-based GraphQL API backend for the geospatial workflow platform. See [../AGENTS.md](../AGENTS.md) for monorepo-level guidance.

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

## Key Directories

- `api/cmd/reearth-flow/` - Application entry point
- `api/internal/adapter/gql/` - GraphQL resolvers and loaders
- `api/internal/usecase/interactor/` - Use case implementations
- `api/internal/infrastructure/mongo/` - MongoDB repositories
- `api/pkg/` - Domain models (project, job, workflow, deployment, schema)
- `api/gql/` - GraphQL schema definitions (`.graphql` files)
- `subscriber/` - Pub/Sub log processing service
- `websocket/` - Real-time collaboration server (Rust, Y-WebSocket)

## Architecture

Follows **Clean Architecture** with **DDD** patterns:

1. **Domain** (`pkg/`) - Pure business entities, no external dependencies
2. **Use Case** (`internal/usecase/`) - Business rules, repository/gateway interfaces
3. **Infrastructure** (`internal/infrastructure/`) - MongoDB, Redis, GCP implementations
4. **Adapter** (`internal/adapter/`) - GraphQL API, DataLoader for N+1 optimization

## Key Constraints

- Server API must be running **before** starting WebSocket server
- Run `make gql` after any `.graphql` schema changes
- Run `make gen-policies` after RBAC policy changes

## Code Quality

**Before completing any task, always run:**

```bash
make lint
make test
make gql              # If schema changed
```

## Documentation

- [Server Architecture](docs/architecture.md) - Architecture patterns, GraphQL design, real-time collaboration, job execution
- [Development Guide](docs/development-guide.md) - Common workflows, environment variables, adding entities
