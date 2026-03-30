# AGENTS.md

Re:Earth Flow is a monorepo containing a geospatial workflow platform with three components:

- **Engine** (`engine/`) - Rust-based DAG workflow execution engine — See [engine/AGENTS.md](engine/AGENTS.md)
- **Server** (`server/`) - Go-based GraphQL API backend — See [server/CLAUDE.md](server/CLAUDE.md)
- **UI** (`ui/`) - React/TypeScript visual workflow builder — See [ui/CLAUDE.md](ui/CLAUDE.md)

## Key Constraints

- All workflow IDs (workflow, graph, node, edge) must use **valid UUID format**
- Server API must be running **before** starting WebSocket server
- Services must start in order: MongoDB → Server API → WebSocket → UI

## Git Workflow

- Follow conventional commit format: `feat:`, `fix:`, `chore:`, etc.
- Do not include AI agent attribution in commit messages
- `main` is the production branch; use feature branches for development

## Documentation

- [Architecture & Data Flow](docs/architecture.md)
- [Development Guidelines](docs/development-guidelines.md)
