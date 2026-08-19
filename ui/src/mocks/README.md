# Re:Earth Flow Mock Server

This directory contains the GraphQL mock server implementation for Re:Earth Flow UI development and testing.

## Overview

The mock server uses **Mock Service Worker (MSW)** to intercept GraphQL requests and provide realistic mock responses. This allows frontend development without requiring a running backend server.

## Features

- 🚀 **Full GraphQL Schema**: Complete mock implementation of the Re:Earth Flow GraphQL API
- 🔒 **Authentication Simulation**: Mock Auth0 authentication flow
- 📊 **Rich Mock Data**: Comprehensive datasets for users, workspaces, projects, jobs, and deployments
- 🔄 **Real-time Subscriptions**: Simulated WebSocket subscriptions for job status updates
- 🧪 **Development & Testing**: Perfect for development, testing, and demos

## Quick Start

### 1. Enable Mock Server

Set the environment variable in your `.env` file:

```bash
FLOW_MOCK_ENABLED=true
```

### 2. Start Development Server

```bash
FLOW_MOCK_ENABLED=true yarn start
```

The mock server starts and intercepts GraphQL requests.

Two things to know:

- **Authentication is bypassed in mock mode.** `AuthProvider` substitutes
  `useMockAuth` (`src/lib/auth/mockAuth.ts`) so the app renders without an identity
  provider. Guarded by `import.meta.env.DEV`, which Vite replaces with `false` when
  building, so the bypass is eliminated from production bundles rather than merely
  disabled. Before this existed, mock mode rendered a blank page: `AuthProvider`
  returned `null` and, because that is legal React, there was no error to see.
- **The editor canvas still needs a WebSocket server.** `useYjsSetup` only renders
  the canvas once the Yjs provider syncs, so mocking GraphQL alone is not enough to
  open a project. Run `server/websocket-go` (with Redis and fake-gcs) alongside.

If Service Worker registration fails, which happens in some embedded browsers, the
app now logs the failure and continues to mount rather than silently rendering
nothing. GraphQL will then hit the real API instead of the mocks.

## Configuration

### Environment Variables

- `FLOW_MOCK_ENABLED=true` - Enable the mock server
- `FLOW_AUTH_PROVIDER=mock` - Use mock authentication
- `FLOW_DEV_MODE=true` - Enable development mode

### Mock Data

Mock data is organized in the `data/` directory:

```
data/
├── users.ts          # User accounts and profiles
├── workspaces.ts     # Workspaces and member roles
├── projects.ts       # Projects and parameters
├── jobs.ts           # Job executions and logs
└── deployments.ts    # Deployment statuses
```

## Available Mock Operations

### Queries

- `me` - Current user information
- `workspaces` - List of available workspaces
- `projects(workspaceId, pagination)` - Projects in a workspace
- `jobs(workspaceId, pagination)` - Job executions
- `job(id)` - Individual job details

### Mutations

- `createProject(input)` - Create a new project
- `updateProject(input)` - Update project details
- `deleteProject(input)` - Delete a project
- `runProject(input)` - Execute a project workflow
- `createWorkspace(input)` - Create a new workspace
- `cancelJob(input)` - Cancel a running job

### Subscriptions

- `jobStatus(jobId)` - Real-time job status updates
- `jobLogs(jobId)` - Real-time log streaming

## Mock Data Details

### Users

5 mock users with different roles:

- **admin@reearth.io** - Administrator (default current user)
- **john@reearth.io** - Developer
- **jane@reearth.io** - Designer
- **mike@reearth.io** - Analyst
- **guest@reearth.io** - Guest user

### Workspaces

4 mock workspaces with different configurations:

- **Personal Workspace** - Individual workspace
- **Development Team** - Team collaboration
- **Analytics Project** - Data analysis focus
- **Design Studio** - Design-focused workspace

### Projects

6 mock projects covering various use cases:

- Data Processing Pipeline
- Real-time Analytics
- Machine Learning Workflow
- Data Visualization Dashboard
- Legacy Data Migration (archived)
- Design System Components

### Jobs

5 mock jobs in different states:

- **Completed** - Successful execution with outputs
- **Running** - In-progress execution with live logs
- **Failed** - Failed execution with error logs
- **Pending** - Queued for execution
- **Cancelled** - User-cancelled execution

## Development Workflow

### 1. Adding New Mock Data

Create new entries in the respective data files:

```typescript
// data/projects.ts
export const mockProjects: MockProject[] = [
  // ... existing projects
  {
    id: "project-new",
    name: "New Project",
    description: "Description here",
    workspaceId: "workspace-1",
    // ... other fields
  },
];
```

### 2. Adding New Resolvers

Extend the resolvers in `schema/resolvers.ts`:

```typescript
// schema/resolvers.ts
Query: {
  // ... existing queries
  newQuery: (_, args) => {
    // Implementation here
  },
},
```

### 3. Updating Schema

Modify the GraphQL schema in `schema/typeDefs.ts`:

```graphql
type Query {
  # ... existing queries
  newQuery(input: NewInput!): NewResult!
}
```

## Testing Features

### Authentication Testing

The mock server provides different authentication scenarios:

```typescript
// Mock authenticated user
const authContext = { isAuthenticated: true, token: "mock-token" };

// Mock unauthenticated access
const authContext = { isAuthenticated: false, token: null };
```

### Real-time Features

Test WebSocket subscriptions:

```typescript
// Subscribe to job status updates
const subscription = await sdk.JobStatus({ jobId: "job-1" });
```

### Error Scenarios

Test error handling:

```typescript
// Invalid project ID returns null
const project = await sdk.Project({ id: "invalid-id" });
// project === null
```

## File Structure

```
mocks/
├── README.md                 # This documentation
├── index.ts                  # Mock server entry point
├── browser.ts                # MSW browser setup
├── data/                     # Mock data definitions
│   ├── users.ts
│   ├── workspaces.ts
│   ├── projects.ts
│   ├── jobs.ts
│   └── deployments.ts
├── handlers/                 # Request handlers
│   ├── index.ts
│   └── graphql.ts           # GraphQL operation handlers
└── schema/                  # GraphQL schema and resolvers
    ├── typeDefs.ts          # GraphQL type definitions
    └── resolvers.ts         # GraphQL resolvers
```

## Browser Developer Tools

When the mock server is running, you'll see console logs for GraphQL operations:

```
🚀 GraphQL Operation: Projects
📝 Variables: { workspaceId: "workspace-1", pagination: { page: 1, pageSize: 10 } }
✅ GraphQL Result: { data: { projects: { nodes: [...], pageInfo: {...} } } }
```

## Production Safety

The mock server automatically disables itself in production:

- Only runs when `NODE_ENV === "development"`
- Requires explicit `FLOW_MOCK_ENABLED=true` configuration
- Service worker excluded from production builds

## Troubleshooting

### Mock Server Not Starting

1. Check environment variables:

   ```bash
   FLOW_MOCK_ENABLED=true
   ```

2. Verify MSW service worker is installed:

   ```bash
   ls public/mockServiceWorker.js
   ```

3. Check browser console for startup messages:
   ```
   🚀 Starting Mock Server for Re:Earth Flow
   ```

### GraphQL Operations Not Intercepted

1. Ensure requests are made to `/api/graphql` (relative path)
2. Check MSW is properly registered in browser dev tools
3. Verify Authorization header is present

### Authentication Issues

1. Mock mode should provide default authentication
2. Check GraphQL context includes `isAuthenticated: true`
3. Verify Bearer token is present in request headers

## Contributing

When adding new features to the real API:

1. Update mock schema in `typeDefs.ts`
2. Add mock data in appropriate `data/*.ts` files
3. Implement resolvers in `resolvers.ts`
4. Test with real UI components
5. Update this documentation

## Resources

- [MSW Documentation](https://mswjs.io/)
- [GraphQL Tools](https://the-guild.dev/graphql/tools)
- [Re:Earth Flow API Schema](../../server/api/gql/)
