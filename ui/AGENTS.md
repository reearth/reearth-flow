# AGENTS.md

React/TypeScript visual workflow builder frontend. See [../AGENTS.md](../AGENTS.md) for monorepo-level guidance.

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

## Key Directories

- `src/features/` - Feature-based modules (Editor, WorkspaceProjects, WorkspaceJobs, Canvas, etc.)
- `src/components/` - Reusable UI components (Radix UI-based)
- `src/lib/` - Core libraries (yjs/, reactFlow/, gql/)
- `src/stores/` - Jotai state atoms
- `src/routes/` - TanStack Router file-based routes
- `src/hooks/` - Custom React hooks

## Import Aliases

- `@flow/*` maps to `src/*` — configured in both `tsconfig.json` and `vite.config.ts`

## State Management

Four-layer state strategy:

1. **Client State** - Jotai atoms (`src/stores/`) for UI state
2. **Server State** - TanStack Query for API data fetching/caching
3. **Collaborative State** - Yjs for real-time workflow synchronization (CRDTs)
4. **Persistent State** - IndexedDB for drafts and offline data

## GraphQL Integration

- Types generated from server schema into `src/lib/gql/__gen__/`
- Run `yarn gql` after any server schema change
- Feature-specific API modules with TanStack Query hooks

## Testing

- **Vitest** with jsdom environment for unit tests
- **Testing Library** for component testing — test user interactions, not implementation
- **Storybook** for component development and visual testing
- **MSW** (Mock Service Worker) for GraphQL response mocking

## Code Quality

**Before completing any task, always run:**

```bash
yarn lint           # Check for code quality issues
yarn type           # Verify TypeScript compilation
yarn format:write   # Apply Prettier formatting (critical for CI/CD)
yarn test --run     # Ensure all tests pass
```

## Common Tasks

### Adding New Workflow Actions

1. Update server schema for new action type
2. Run `yarn gql` to regenerate types
3. Create action component in appropriate feature directory
4. Add to action palette in Editor
5. Implement configuration form using SchemaForm
6. Add validation logic for connections

### Modifying GraphQL Schema

1. Server updates schema in `server/api/gql/*.graphql`
2. Run `yarn gql` to regenerate UI types
3. Update affected components with new types
4. Fix TypeScript errors from type changes

## Documentation

- [UI Architecture](docs/architecture.md) - Technologies, data flow, component patterns, environment configuration
