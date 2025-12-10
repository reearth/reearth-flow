# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Architecture Overview

Re:Earth Flow is a **monorepo** containing a comprehensive geospatial workflow platform with three main components:

- **Engine** (`engine/`) - Rust-based DAG workflow execution engine for geospatial data processing - See [engine/CLAUDE.md](engine/CLAUDE.md)
- **Server** (`server/`) - Go-based GraphQL API backend with real-time collaboration - See [server/CLAUDE.md](server/CLAUDE.md)
- **UI** (`ui/`) - React/TypeScript frontend with visual workflow builder - See [ui/CLAUDE.md](ui/CLAUDE.md)

## Quick Start

### Initial Setup

```bash
# Clone the repository
git clone <repository-url>
cd reearth-flow

# Set up each component
cd engine && cargo build && cd ..
cd server/api && make run-db && cd ../..
cd ui && yarn install && cd ..
```

### Running the Full Stack

```bash
# Terminal 1: Start MongoDB
cd server/api && make run-db

# Terminal 2: Start API Server (must run before WebSocket!)
cd server/api && make run-app

# Terminal 3: Start WebSocket Server
cd server/websocket && cargo run

# Terminal 4: Start UI Development Server
cd ui && yarn start
```

## End-to-End Data Flow

Understanding how data flows through the entire system:

### 1. Workflow Creation & Editing
- **UI** creates workflow definitions using visual editor (ReactFlow)
- **Yjs** (UI) syncs changes in real-time across collaborative clients
- **WebSocket Server** (Server/Rust) manages Y-WebSocket protocol
- **GraphQL mutations** (UI → Server) persist workflow state to MongoDB
- **Server** stores workflow definitions in database

### 2. Workflow Execution
- **UI** triggers workflow execution via GraphQL mutation
- **Server** creates job record and submits to Google Cloud Batch
- **Engine** receives workflow definition and executes DAG
- **Engine** processes geospatial data through action pipeline
- **Engine** writes results to cloud storage (GCS/S3)

### 3. Real-time Monitoring
- **Engine** publishes logs and status to Google Pub/Sub
- **Subscriber Service** (Server) consumes Pub/Sub events
- **Subscriber** writes logs to MongoDB and Redis
- **Server** streams updates via GraphQL subscriptions
- **UI** displays real-time job progress and logs

### 4. Results & Visualization
- **UI** retrieves result URLs from completed jobs
- **UI** visualizes geospatial outputs in Cesium/MapLibre
- **Server** provides access control for results
- **Engine** data accessible via cloud storage links

## Cross-Component Workflows

### Adding New Workflow Actions

When adding a new action type that spans all three components:

1. **Engine** (`engine/runtime/action-*/`)
   - Implement action logic in Rust
   - Add to action registry
   - Write unit tests
   - See [engine/CLAUDE.md](engine/CLAUDE.md) for details

2. **Server** (`server/api/pkg/schema/`)
   - Define action schema and validation
   - Update GraphQL types if needed
   - Register action in schema generator
   - See [server/CLAUDE.md](server/CLAUDE.md) for details

3. **UI** (`ui/src/features/Editor/`)
   - Add action to node palette
   - Create configuration form component
   - Add validation and connection rules
   - Run `yarn gql` to regenerate types
   - See [ui/CLAUDE.md](ui/CLAUDE.md) for details

4. **Integration Testing**
   - Test workflow creation in UI
   - Test execution through server to engine
   - Verify results and monitoring

### Modifying GraphQL Schema

When making API changes that affect both server and UI:

1. **Server** - Update schema in `server/api/gql/*.graphql`
2. **Server** - Run `make gql` to regenerate Go code
3. **Server** - Implement resolver logic
4. **Server** - Update use cases and repositories as needed
5. **UI** - Run `yarn gql` to regenerate TypeScript types
6. **UI** - Update components using modified types
7. **Testing** - Test end-to-end data flow

### Database Schema Changes

When modifying MongoDB collections:

1. **Server** - Add migration in `server/api/internal/infrastructure/mongo/migration/`
2. **Server** - Update domain models in `server/api/pkg/*/`
3. **Server** - Update repository implementations
4. **Subscriber** - Update if log/node processing affected
5. **Testing** - Verify migration and data integrity

### Real-time Collaboration Features

When adding features that require real-time sync:

1. **UI** - Update Yjs document structure
2. **WebSocket Server** - Verify Y-WebSocket compatibility
3. **Server** - Update GraphQL schema for persistence
4. **UI** - Create hooks for reactive Yjs state
5. **Testing** - Test with multiple concurrent users

## Environment Configuration

Each component uses environment variables with consistent prefixes:

### Shared Conventions
- `FLOW_*` - Application-specific configuration
- `REEARTH_*` - Platform-level configuration
- `GOOGLE_*` - Google Cloud Platform credentials

### Component-Specific Details
- **Engine** - See [engine/CLAUDE.md](engine/CLAUDE.md) for `FLOW_RUNTIME_*` and `FLOW_VAR_*`
- **Server** - See [server/CLAUDE.md](server/CLAUDE.md) for database and cloud configuration
- **UI** - See [ui/CLAUDE.md](ui/CLAUDE.md) for API endpoints and Auth0 settings

## Multi-Service Coordination

### Service Dependencies

```
MongoDB (required by Server API and Subscriber)
  ↓
Server API (provides authentication for WebSocket)
  ↓
WebSocket Server (handles real-time collaboration)
  ↓
UI (connects to both API and WebSocket)

Engine (standalone, coordinated by Server via GCP Batch)
```

### Critical Startup Order

1. **MongoDB** - Database must be running first
2. **Server API** - Provides authentication service
3. **WebSocket Server** - Depends on API for auth
4. **UI** - Connects to both API and WebSocket

**Important**: Server API must be running before starting WebSocket server!

## Development Best Practices

### Code Quality

Each component has its own quality checks:
- **Engine**: `cargo make format && cargo make clippy && cargo make test`
- **Server**: `make lint && make test`
- **UI**: `yarn lint && yarn type && yarn format:write && yarn test`

Always run the appropriate checks before committing changes.

### Testing Strategy

- **Unit Tests** - Each component has its own test suite
- **Integration Tests** - Server has e2e tests for API workflows
- **End-to-End** - Manual testing across full stack for critical flows
- **Real-time Features** - Test with multiple clients for collaboration

### Git Workflow

**Commit Message Guidelines**:
- Keep messages clean and focused on actual changes
- Do not include Claude Code attribution or "Generated with Claude Code" messages
- Follow conventional commit format when appropriate: `feat:`, `fix:`, `chore:`, etc.

**Branch Strategy**:
- `main` - Production-ready code
- Feature branches for new development
- Component-specific changes can often be isolated to one directory

### Performance Considerations

- **Engine** - Parallel processing of geospatial features
- **Server** - GraphQL DataLoader pattern for N+1 queries, Redis caching
- **UI** - Code splitting, memoization, virtual scrolling
- **Real-time** - WebSocket connection pooling, efficient Yjs updates

### Security Best Practices

- **Authentication** - JWT tokens from Auth0/Cognito validated at all layers
- **Authorization** - Workspace/project permissions checked via RBAC
- **Secrets** - Never commit credentials, use environment variables
- **Input Validation** - Validate at API boundaries (GraphQL resolvers, engine inputs)
- **Cloud Security** - GCP service accounts with minimal required permissions

## Troubleshooting

### WebSocket Connection Issues
- Verify Server API is running (WebSocket depends on it for auth)
- Check `FLOW_WS_ENDPOINT` in UI configuration
- Review WebSocket server logs for authentication errors

### Job Execution Failures
- Check GCP credentials and permissions
- Verify engine can access cloud storage buckets
- Review job logs in Server subscriber service
- Check workflow definition format

### GraphQL Type Mismatches
- Ensure both `server/api: make gql` and `ui: yarn gql` have been run
- Restart dev servers after schema changes
- Check for manual edits to generated files

### Real-time Sync Issues
- Verify WebSocket connection is established
- Check Yjs document state in browser devtools
- Review network tab for WebSocket messages
- Ensure multiple clients are in same workspace context

## Component Documentation

For detailed component-specific guidance:
- **Engine Development** - [engine/CLAUDE.md](engine/CLAUDE.md)
- **Server Development** - [server/CLAUDE.md](server/CLAUDE.md)
- **UI Development** - [ui/CLAUDE.md](ui/CLAUDE.md)
