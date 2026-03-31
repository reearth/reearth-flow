# Development Guidelines

## Initial Setup

```bash
git clone <repository-url>
cd reearth-flow

cd engine && cargo build && cd ..
cd server/api && make run-db && cd ../..
cd ui && yarn install && cd ..
```

## Running the Full Stack

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

## Cross-Component Workflows

### Adding New Workflow Actions

When adding a new action type that spans all three components:

1. **Engine** (`engine/runtime/action-*/`)
   - Implement action logic in Rust
   - Add to action registry
   - Write unit tests

2. **Server** (`server/api/pkg/schema/`)
   - Define action schema and validation
   - Update GraphQL types if needed
   - Register action in schema generator

3. **UI** (`ui/src/features/Editor/`)
   - Add action to node palette
   - Create configuration form component
   - Add validation and connection rules
   - Run `yarn gql` to regenerate types

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

## Testing Strategy

- **Unit Tests** - Each component has its own test suite
- **Integration Tests** - Server has e2e tests for API workflows
- **End-to-End** - Manual testing across full stack for critical flows
- **Real-time Features** - Test with multiple clients for collaboration

## Performance Considerations

- **Engine** - Parallel processing of geospatial features
- **Server** - GraphQL DataLoader pattern for N+1 queries, Redis caching
- **UI** - Code splitting, memoization, virtual scrolling
- **Real-time** - WebSocket connection pooling, efficient Yjs updates

## Security Best Practices

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
