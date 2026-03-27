# UI Architecture

## Core Technologies

- **React 19** with TypeScript 5 and Vite 6
- **@xyflow/react** for the visual workflow editor (node-based canvas)
- **Yjs + Y-WebSocket** for real-time collaborative editing
- **TanStack Router** for file-based routing with type safety
- **TanStack Query + Jotai** for state management
- **Tailwind CSS + Radix UI** for styling and components
- **GraphQL + graphql-request** for API communication
- **Cesium** for 2D/3D geospatial visualization

## Real-time Collaboration

### Architecture (`src/lib/yjs/`)

- **YWorkflowClass** - Manages workflow document synchronization
- **Y-WebSocket** - Connects to backend WebSocket server
- **Hooks**: `useYWorkflow`, `useYNode`, `useYEdge` for reactive Yjs state
- **Awareness** - Shows other users' cursors and selections

### Integration Flow

1. User opens workflow in editor
2. UI establishes Y-WebSocket connection
3. Yjs syncs local state with server
4. Changes propagate in real-time to all connected clients
5. Conflicts resolved automatically via CRDT algorithms

## Visual Workflow Editor

### ReactFlow Integration (`src/lib/reactFlow/`)

- **Node Types**:
  - `GeneralNode` - Standard workflow action nodes
  - `BatchNode` - Batch processing nodes
  - `NoteNode` - Annotation and documentation
- **Edge Types**: `DefaultEdge` with custom styling and validation
- **Custom Handles** - Connection points with type checking
- **Auto-layout** - Dagre algorithm for automatic workflow arrangement
- **Minimap & Controls** - Navigation aids for complex workflows

### Node Interaction

- Drag-and-drop from action palette
- Click to configure node parameters
- Connect nodes to define data flow
- Real-time validation of connections

## Component System

### Radix UI Integration (`src/components/`)

- Accessible primitives with custom styling
- Consistent design system with dark mode support

### Key Components

- **SchemaForm** - Dynamic form generation from JSON schemas (action configuration)
- **DataTable** - Sortable, filterable tables with pagination
- **Visualizations** - Cesium 2D/3D map components
- **Dialog/Modal** - Accessible overlays for workflows
- **Dropdown/Select** - Type-safe selection components

## Routing & Navigation

### TanStack Router (`src/routes/`)

- File-based routing with type safety
- Generated route tree for autocomplete
- Workspace/Project hierarchy: `/workspaces/$workspaceId/projects/$projectId`
- Lazy loading for code splitting
- Search params validation

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

## Environment Configuration

UI-specific environment variables with `FLOW_` prefix:

- `FLOW_API_ENDPOINT` - GraphQL API URL
- `FLOW_WS_ENDPOINT` - WebSocket endpoint for real-time features
- `FLOW_AUTH0_*` - Auth0 configuration
- `REEARTH_CONFIG_URL` - Remote configuration URL

## Multi-tenant Architecture

- **Workspaces** contain projects, jobs, deployments, and members
- **Projects** define individual workflows
- **Real-time collaboration** within workspace contexts
- **Permission-based access** via Auth0 integration

## Integration Points

### With Server

- **GraphQL API** - All data operations
- **WebSocket** - Real-time collaboration and job updates
- **Authentication** - JWT tokens via Auth0

### With Engine

- **Workflow definitions** - JSON format for execution
- **Job monitoring** - Status and logs streaming
- **Results** - Cloud storage URLs for outputs

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

## Styling Guidelines

- **Use Tailwind utilities** for consistent spacing and colors
- **Create reusable components** for repeated patterns
- **Follow Radix UI patterns** for accessible interactions
- **Support dark mode** via CSS variables
- **Responsive design** - mobile-first approach

## Error Handling

- **GraphQL errors** - Display user-friendly messages
- **Network errors** - Show retry UI with offline detection
- **Validation errors** - Inline form validation
- **Yjs sync errors** - Reconnection logic with backoff

## Security Best Practices

- **Authentication** - Validate tokens on all protected routes
- **Authorization** - Check workspace/project permissions
- **XSS prevention** - Sanitize user inputs
- **CSRF protection** - Use proper headers
- **Secrets** - Never commit API keys or credentials
- **Content Security Policy** - Restrict script sources
