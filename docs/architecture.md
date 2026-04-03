# Architecture

Re:Earth Flow is a monorepo containing a comprehensive geospatial workflow platform.

## Components

- **Engine** (`engine/`) - Rust-based DAG workflow execution engine for geospatial data processing
- **Server** (`server/`) - Go-based GraphQL API backend with real-time collaboration
- **UI** (`ui/`) - React/TypeScript frontend with visual workflow builder

## End-to-End Data Flow

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
- **UI** visualizes geospatial outputs in Cesium
- **Server** provides access control for results
- **Engine** data accessible via cloud storage links

## Service Dependencies

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

**Critical Startup Order:**

1. **MongoDB** - Database must be running first
2. **Server API** - Provides authentication service
3. **WebSocket Server** - Depends on API for auth
4. **UI** - Connects to both API and WebSocket

> **Important**: Server API must be running before starting WebSocket server!

## Environment Configuration

Each component uses environment variables with consistent prefixes:

- `FLOW_*` - Application-specific configuration
- `REEARTH_*` - Platform-level configuration
- `GOOGLE_*` - Google Cloud Platform credentials

Component-specific details are documented in each component's documentation (AGENTS.md or CLAUDE.md).
