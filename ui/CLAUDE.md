# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Architecture Overview

Re:Earth Flow UI is a React/TypeScript frontend for building visual geospatial data workflows. It provides a node-based workflow editor with real-time collaboration capabilities.

### Core Technologies

- **React 19** with TypeScript 5 and Vite 6 for modern development
- **@xyflow/react** for the visual workflow editor (node-based canvas)
- **Yjs + Y-WebSocket** for real-time collaborative editing
- **TanStack Router** for file-based routing with type safety
- **TanStack Query + Jotai** for state management
- **Tailwind CSS + Radix UI** for styling and components
- **GraphQL + graphql-request** for API communication
- **Cesium + MapLibre** for 3D/2D geospatial visualization

## Development Commands

```bash
# Development
yarn start          # Start dev server on port 3000
yarn test           # Run unit tests with Vitest
yarn coverage       # Run tests with coverage
yarn storybook      # Start Storybook on port 6006

# Code Quality
yarn lint           # ESLint checking
yarn fix            # Auto-fix ESLint issues
yarn type           # TypeScript type checking
yarn format:check   # Prettier format checking
yarn format:write   # Apply Prettier formatting

# Build & Deploy
yarn build          # Production build (requires tsc + vite build)
yarn serve          # Preview production build

# GraphQL & Internationalization
yarn gql            # Generate GraphQL types from ../server/api/gql/*.graphql
yarn gql:watch      # Watch mode for GraphQL codegen
yarn i18n           # Extract i18n strings
```

## Project Structure & Key Patterns

### State Management Architecture

- **Jotai atoms** (`src/stores/`) for client state
- **TanStack Query** for server state and caching
- **Yjs** for collaborative document state (workflows, nodes, edges)
- **IndexedDB** for local persistence of drafts and offline data

### Real-time Collaboration

- **Yjs integration** (`src/lib/yjs/`) handles collaborative workflow editing
- **Y-WebSocket** connects to backend WebSocket server
- **YWorkflowClass** manages workflow document synchronization
- **useYWorkflow, useYNode, useYEdge** hooks for reactive Yjs state

### Visual Workflow Editor

- **ReactFlow** (`src/lib/reactFlow/`) powers the node-based canvas
- **Node Types**: GeneralNode, BatchNode, NoteNode with custom rendering
- **Edge Types**: DefaultEdge with custom styling
- **Custom Handles** for node connection points
- **Auto-layout** using Dagre for workflow arrangement

### Component System

- **Radix UI primitives** with custom styling in `src/components/`
- **SchemaForm** for dynamic form generation from JSON schemas
- **DataTable** with sorting, filtering, and pagination
- **Visualizations** for Cesium 3D and MapLibre 2D maps

### Feature-based Organization

```
src/features/
├── Editor/           # Main workflow editor with canvas
├── WorkspaceProjects/# Project management
├── WorkspaceJobs/    # Job execution monitoring
├── Canvas/           # Shared canvas components
├── SharedCanvas/     # Read-only shared workflows
└── common/           # Shared feature components
```

### GraphQL Integration

- **Generated types** from server schema in `src/lib/gql/__gen__/`
- **Feature-specific APIs** (project, job, workspace, etc.) with custom hooks
- **Real-time subscriptions** for job status updates
- **Optimistic updates** for better UX during mutations

### Routing & Navigation

- **File-based routing** with TanStack Router
- **Type-safe navigation** with generated route tree
- **Workspace/Project hierarchy**: `/workspaces/$workspaceId/projects/$projectId`
- **Lazy loading** for better performance

## Key Development Patterns

### Workflow Data Flow

1. **UI creates workflows** via visual editor (ReactFlow)
2. **Yjs syncs changes** in real-time across clients
3. **GraphQL mutations** persist to backend database
4. **Engine processes** workflows via server coordination
5. **WebSocket subscriptions** provide real-time job status

### Testing Strategy

- **Vitest** for unit tests with jsdom environment
- **Testing Library** for component testing
- **Storybook** for component development and visual testing
- **Coverage reporting** with exclusions for generated code

### Import Aliases

- `@flow/*` maps to `src/*` for cleaner imports
- Configured in both `tsconfig.json` and `vite.config.ts`

## Environment Configuration

Uses `FLOW_` prefix for environment variables:

- `FLOW_API_ENDPOINT` - GraphQL API URL
- `FLOW_WS_ENDPOINT` - WebSocket endpoint for real-time features
- `FLOW_AUTH0_*` - Auth0 configuration
- `REEARTH_CONFIG_URL` - Remote configuration URL

## Multi-tenant Architecture

- **Workspaces** contain projects, jobs, deployments, and members
- **Projects** define individual workflows
- **Real-time collaboration** within workspace contexts
- **Permission-based access** via Auth0 integration
