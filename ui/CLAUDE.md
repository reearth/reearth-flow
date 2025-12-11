# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with the UI component of this repository.

For monorepo architecture, cross-component workflows, and general project guidance, see @../CLAUDE.md

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

## Project Structure

### Directory Organization

```
ui/
├── src/
│   ├── features/           # Feature-based modules
│   │   ├── Editor/         # Main workflow editor with canvas
│   │   ├── WorkspaceProjects/  # Project management
│   │   ├── WorkspaceJobs/  # Job execution monitoring
│   │   ├── Canvas/         # Shared canvas components
│   │   ├── SharedCanvas/   # Read-only shared workflows
│   │   └── common/         # Shared feature components
│   ├── components/         # Reusable UI components
│   ├── lib/                # Core libraries and utilities
│   │   ├── yjs/           # Yjs collaboration setup
│   │   ├── reactFlow/     # ReactFlow configuration
│   │   └── gql/           # GraphQL client and hooks
│   ├── stores/            # Jotai state atoms
│   ├── routes/            # TanStack Router routes
│   └── hooks/             # Custom React hooks
├── .storybook/            # Storybook configuration
└── public/                # Static assets
```

### Import Aliases

- `@flow/*` maps to `src/*` for cleaner imports
- Configured in both `tsconfig.json` and `vite.config.ts`

## State Management Architecture

### Multi-layer State Strategy

1. **Client State** - Jotai atoms (`src/stores/`)
   - UI state (modals, panels, selections)
   - User preferences
   - Local workspace state

2. **Server State** - TanStack Query
   - API data fetching and caching
   - Optimistic updates for mutations
   - Real-time subscriptions for job status

3. **Collaborative State** - Yjs
   - Workflow definitions (nodes, edges, configuration)
   - Real-time synchronization across clients
   - Conflict-free replicated data types (CRDTs)

4. **Persistent State** - IndexedDB
   - Draft workflows
   - Offline data
   - User preferences cache

## Key Features & Patterns

### Real-time Collaboration

**Architecture** (`src/lib/yjs/`):
- **YWorkflowClass** - Manages workflow document synchronization
- **Y-WebSocket** - Connects to backend WebSocket server
- **Hooks**: `useYWorkflow`, `useYNode`, `useYEdge` for reactive Yjs state
- **Awareness** - Shows other users' cursors and selections

**Integration Flow**:
1. User opens workflow in editor
2. UI establishes Y-WebSocket connection
3. Yjs syncs local state with server
4. Changes propagate in real-time to all connected clients
5. Conflicts resolved automatically via CRDT algorithms

### Visual Workflow Editor

**ReactFlow Integration** (`src/lib/reactFlow/`):
- **Node Types**:
  - `GeneralNode` - Standard workflow action nodes
  - `BatchNode` - Batch processing nodes
  - `NoteNode` - Annotation and documentation
- **Edge Types**: `DefaultEdge` with custom styling and validation
- **Custom Handles** - Connection points with type checking
- **Auto-layout** - Dagre algorithm for automatic workflow arrangement
- **Minimap & Controls** - Navigation aids for complex workflows

**Node Interaction**:
- Drag-and-drop from action palette
- Click to configure node parameters
- Connect nodes to define data flow
- Real-time validation of connections

### Component System

**Radix UI Integration** (`src/components/`):
- Accessible primitives with custom styling
- Consistent design system
- Dark mode support

**Key Components**:
- **SchemaForm** - Dynamic form generation from JSON schemas (action configuration)
- **DataTable** - Sortable, filterable tables with pagination
- **Visualizations** - Cesium 3D and MapLibre 2D map components
- **Dialog/Modal** - Accessible overlays for workflows
- **Dropdown/Select** - Type-safe selection components

### GraphQL Integration

**Code Generation** (`src/lib/gql/__gen__/`):
- Types generated from server schema
- Type-safe queries and mutations
- React hooks for all operations

**API Organization**:
- Feature-specific API modules (project, job, workspace, etc.)
- Custom hooks wrapping TanStack Query
- Real-time subscriptions for job updates
- Optimistic updates for instant UI feedback

**Example Pattern**:
```typescript
// Generated types and hooks
import { useProjectQuery, useUpdateProjectMutation } from '@flow/lib/gql/project'

// Feature-specific usage
const { data } = useProjectQuery({ id })
const { mutate } = useUpdateProjectMutation()
```

### Routing & Navigation

**TanStack Router** (`src/routes/`):
- File-based routing with type safety
- Generated route tree for autocomplete
- Workspace/Project hierarchy: `/workspaces/$workspaceId/projects/$projectId`
- Lazy loading for code splitting
- Search params validation

## Development Patterns

### Development Workflow & Quality Assurance

**IMPORTANT: Before marking any task as complete, ALWAYS run:**

```bash
yarn lint           # Check for code quality issues
yarn type           # Verify TypeScript compilation
yarn format:write   # Apply Prettier formatting (CRITICAL for CI/CD)
yarn test --run     # Ensure all tests pass
```

### Todo List Management

When using the TodoWrite tool to track progress:

**NEVER mark a todo item as completed without explicit developer confirmation.** Always:

1. **Complete the implementation**
2. **Ask the developer to test** the specific functionality
3. **Wait for confirmation** that it works as expected
4. **Only then mark the todo as completed**

Example flow:
```
✅ Good: "I've implemented the streaming fix. Can you test file switching to confirm it works before I mark this todo complete?"
❌ Bad: Immediately marking todo complete after implementation without testing confirmation
```

This prevents issues where implementation appears complete but has functional problems discovered during user testing.

### Documentation for Complex Features

When completing very complex tasks (multi-file implementations, new architectural patterns, performance optimizations, etc.), always ask the developer:

1. **"Is this task complete?"** - Confirm all requirements are met
2. **"Would you like me to create documentation for this feature?"** - Offer to generate dev docs

If the developer agrees, create developer documentation that includes:

- **Architecture overview** and data flow
- **Key implementation details** and configuration
- **Performance considerations** and limitations
- **Troubleshooting guide** with common issues
- **Testing scenarios** and edge cases
- **Future improvement areas**

Place documentation files adjacent to the main implementation (e.g., `src/hooks/feature-name.md` for hook implementations) to ensure discoverability.

### Testing Strategy

- **Vitest** for unit tests with jsdom environment
- **Testing Library** for component testing
- **Storybook** for component development and visual testing
- **Coverage reporting** with exclusions for generated code

**Testing Patterns**:
- Test user interactions, not implementation details
- Mock GraphQL responses with MSW (Mock Service Worker)
- Test accessibility with `@testing-library/jest-dom`
- Visual regression testing via Storybook

### Error Handling

- **GraphQL errors** - Display user-friendly messages
- **Network errors** - Show retry UI with offline detection
- **Validation errors** - Inline form validation
- **Yjs sync errors** - Reconnection logic with backoff

## Common Development Tasks

### Adding New Workflow Actions

1. **Update server schema** for new action type
2. **Run `yarn gql`** to regenerate types
3. **Create action component** in appropriate feature directory
4. **Add to action palette** in Editor
5. **Implement configuration form** using SchemaForm
6. **Add validation logic** for connections
7. **Test in Storybook** and integration tests

### Modifying GraphQL Schema

1. **Server updates schema** in `server/api/gql/*.graphql`
2. **Run `yarn gql`** to regenerate UI types
3. **Update affected components** with new types
4. **Fix TypeScript errors** from type changes
5. **Test data flow** end-to-end

### Adding New Features

1. **Create feature directory** under `src/features/`
2. **Define routes** in `src/routes/`
3. **Create components** with Storybook stories
4. **Add GraphQL queries/mutations** as needed
5. **Wire up state management** (Jotai/TanStack Query)
6. **Add tests** for critical paths
7. **Document** complex interactions

### Styling Guidelines

- **Use Tailwind utilities** for consistent spacing and colors
- **Create reusable components** for repeated patterns
- **Follow Radix UI patterns** for accessible interactions
- **Support dark mode** via CSS variables
- **Responsive design** - mobile-first approach
- **Performance** - Avoid unnecessary re-renders

## UI Data Flow

### Complete Workflow Lifecycle

1. **Create Workflow**
   - User creates workflow in visual editor (ReactFlow)
   - Yjs syncs changes in real-time across collaborative clients
   - GraphQL mutations persist workflow state to backend

2. **Execute Workflow**
   - User triggers workflow execution
   - GraphQL mutation creates job on server
   - Server coordinates with engine for execution

3. **Monitor Execution**
   - WebSocket subscriptions provide real-time job status updates
   - Logs streamed via Pub/Sub to UI
   - Progress updates reflected in job monitoring UI

4. **View Results**
   - Completed workflows show results in visualizations
   - Data accessible via cloud storage links
   - History tracked in job list

For complete end-to-end workflow execution flow across all components, see @../CLAUDE.md

## Environment Configuration

UI-specific environment variables with `FLOW_` prefix:

- `FLOW_API_ENDPOINT` - GraphQL API URL
- `FLOW_WS_ENDPOINT` - WebSocket endpoint for real-time features
- `FLOW_AUTH0_*` - Auth0 configuration
- `REEARTH_CONFIG_URL` - Remote configuration URL

See @../CLAUDE.md for complete environment configuration across all components.

## Multi-tenant Architecture

- **Workspaces** contain projects, jobs, deployments, and members
- **Projects** define individual workflows
- **Real-time collaboration** within workspace contexts
- **Permission-based access** via Auth0 integration

## Performance Considerations

- **Code splitting** - Route-based with TanStack Router lazy loading
- **Memoization** - Use `React.memo`, `useMemo`, `useCallback` appropriately
- **Virtual scrolling** - For large lists and tables
- **Debouncing** - User input and API calls
- **Image optimization** - Lazy loading and responsive images
- **Bundle size** - Monitor with bundle analyzer

## Accessibility Guidelines

- **Semantic HTML** - Use appropriate elements
- **ARIA labels** - Provide context for screen readers
- **Keyboard navigation** - All interactions keyboard-accessible
- **Focus management** - Logical tab order
- **Color contrast** - WCAG AA compliance
- **Screen reader testing** - Test with NVDA/JAWS/VoiceOver

## Integration Points

### With Server
- **GraphQL API** - All data operations
- **WebSocket** - Real-time collaboration and job updates
- **Authentication** - JWT tokens via Auth0

### With Engine
- **Workflow definitions** - JSON format for execution
- **Job monitoring** - Status and logs streaming
- **Results** - Cloud storage URLs for outputs

## Security Best Practices

- **Authentication** - Validate tokens on all protected routes
- **Authorization** - Check workspace/project permissions
- **XSS prevention** - Sanitize user inputs
- **CSRF protection** - Use proper headers
- **Secrets** - Never commit API keys or credentials
- **Content Security Policy** - Restrict script sources
