# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Architecture Overview

Re:Earth Flow is a **monorepo** containing a comprehensive geospatial workflow platform with three main components:

- **Engine** (`engine/`) - Rust-based DAG workflow execution engine for geospatial data processing
- **Server** (`server/`) - Go-based GraphQL API backend with real-time collaboration
- **UI** (`ui/`) - React/TypeScript frontend with visual workflow builder

## Development Commands

### Engine (Rust)
```bash
cd engine/
cargo install cargo-make
cargo make format      # Format code
cargo make check       # Compilation check
cargo make clippy      # Linting
cargo make test        # Run tests
cargo make doc         # Generate docs

# Run CLI
cargo run --package reearth-flow-cli -- run --workflow path/to/workflow.yml
```

### Server (Go)
```bash
cd server/api/
make run-app          # Start API server
make run-db           # Start MongoDB
make test             # Run tests
make e2e              # End-to-end tests
make gql              # Generate GraphQL code
```

### UI (React/TypeScript)
```bash
cd ui/
yarn                  # Install dependencies
yarn start            # Development server
yarn build            # Production build
yarn test             # Unit tests
yarn gql              # Generate GraphQL code
yarn lint             # ESLint
```

## Project Structure

### Engine - Workflow Execution
- **Languages**: Rust with cargo-make for build management
- **Core**: DAG-based workflow execution with multi-threading
- **Data Model**: Feature-centric geospatial data processing
- **Actions**: Source/Processor/Sink pattern for data operations
- **Storage**: OpenDAL multi-backend abstraction (GCS, S3, local)
- **Expressions**: Rhai scripting for dynamic parameters

**For detailed engine architecture, development patterns, and debugging guidance, see `engine/CLAUDE.md`.**

### Server - API Backend
- **Languages**: Go with GraphQL (gqlgen)
- **Architecture**: Clean Architecture with DDD patterns
- **Database**: MongoDB with Redis for caching
- **Authentication**: JWT with Auth0/Cognito support
- **Cloud**: Google Cloud Batch/Storage/Pub-Sub integration
- **Real-time**: Rust WebSocket server for collaboration

Key services:
- `api/` - Main GraphQL API server
- `websocket/` - Real-time collaboration (Rust)
- `subscriber/` - Log processing service

### UI - Frontend Application
- **Framework**: React 19 with TypeScript, Vite build
- **UI**: Tailwind CSS + Radix UI components
- **State**: Jotai atoms + TanStack Query + Yjs collaboration
- **Routing**: TanStack Router with file-based routes
- **Editor**: ReactFlow for visual workflow building
- **Maps**: Cesium 3D + MapLibre 2D geospatial visualization

Key features:
- Visual workflow editor with real-time collaboration
- Multi-tenant workspace management
- Geospatial data visualization
- Job monitoring and deployment management

## Technology Integration

### Collaborative Editing
- **UI**: Yjs for document synchronization
- **Server**: Rust WebSocket server (`server/websocket/`)
- **Protocol**: Y-WebSocket for real-time updates

### Workflow Execution
- **Definition**: YAML/JSON workflows created in UI
- **Storage**: MongoDB via GraphQL API
- **Execution**: Engine processes workflows via Server coordination
- **Monitoring**: Real-time job status through WebSocket + GraphQL subscriptions

### Data Flow
1. UI creates workflows with visual editor
2. Server stores definitions in MongoDB
3. Engine executes workflows on GCP Batch
4. Results stored in cloud storage
5. Real-time updates via WebSocket/GraphQL

## Environment Configuration

### Engine
- `FLOW_RUNTIME_*` variables control execution behavior
- `FLOW_VAR_*` variables inject into workflow context
- See `engine/CLAUDE.md` for detailed configuration

### Server
- Google Cloud credentials for batch processing
- MongoDB and Redis connection strings
- Auth provider configuration (Auth0, Cognito)

### UI
- Auth0 configuration for authentication
- API endpoint configuration
- Feature flags for development

## Multi-Service Development

When working across components:
1. **Database changes** - Update server GraphQL schema and UI types
2. **New actions** - Add to engine, update server action registry
3. **UI features** - Consider real-time collaboration impact
4. **Testing** - Each component has its own test suite

## Common Workflows

### Adding New Workflow Actions
1. Implement in `engine/runtime/action-*/`
2. Register in engine mapping files
3. Update server action schemas
4. Add UI components for action configuration

### Frontend Development
- Use Storybook for component development
- Real-time features require WebSocket testing
- Geospatial features need sample data

### Backend API Changes
- Update GraphQL schema in `server/api/gql/`
- Run `make gql` to regenerate code
- Update UI with `yarn gql` for TypeScript types

## Git Commit Guidelines

When creating git commits, do not include Claude Code attribution or "Generated with Claude Code" messages in commit messages. Keep commit messages clean and focused on the actual changes made.
