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

## Engine Schema Integration

`src/lib/intermediateData/` reads the intermediate-data JSONL format through the
engine's generated schema, `feature-intermediate.schema.json`. The original lives
at `engine/schema/`; `src/lib/intermediateData/` holds a **committed copy**.

It is copied rather than imported because the production image builds with
`context: ui` (see `build_deploy_ui.yml`), so nothing outside `ui/` resolves at
build time. The engine owns the original and never writes outside `engine/` —
the same arrangement as the API deploy, which pulls `schema/actions*.json` from
there.

- Refresh it by hand when the engine's schema changes, in the same PR:
  `cp ../engine/schema/feature-intermediate.schema.json src/lib/intermediateData/`
- `ci_ui.yml` fails when the two drift, and `engine/schema/**` is in the UI's
  change-detection so that check runs on an engine-only change
- Never hand-edit the copy — regenerate with `cargo make schema-feature` from
  `engine/`, then copy
- Nothing under `src/` may import from `engine/` or `server/`. If the UI needs a
  generated artifact from either, vendor a copy in as above

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

## FlowExpr Editor

`flowExprConstants.ts` is the single source of truth for the editor. Before making any changes, read the engine source directly to understand what the language currently supports — do not rely on docs, which can be stale:

- **Keywords/operators** → `engine/runtime/expr/src/core/lexer.rs` (the `Token` enum)
- **Built-in functions** → `engine/runtime/expr/src/core/eval.rs` (`default_env()`)
- **Math functions** → `engine/runtime/expr/src/core/builtins/` (individual modules)

Then update **all five** in `flowExprConstants.ts` to match:

- `FLOWEXPR_KEYWORDS`
- `FLOWEXPR_BUILTIN_FUNCTIONS`
- `FLOWEXPR_MATH_FUNCTIONS`
- `FLOWEXPR_OPERATORS` (keep longest → shortest within each group)
- `getFlowExprAutocompleteSuggestions` (one entry per item, with `detail` signature and `{{cursor}}` placement)

See [docs/flow-expr-editor.md](docs/flow-expr-editor.md) for architecture details (overlay stack, syntax highlighter quirks, validator scope, autocomplete positioning).

## Documentation

- [UI Architecture](docs/architecture.md) - Technologies, data flow, component patterns, environment configuration
- [FlowExpr Editor Architecture](docs/flow-expr-editor.md) - Overlay stack, tokenizer, validator, autocomplete
