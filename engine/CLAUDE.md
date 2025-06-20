# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Development Commands

### Building and Testing
```bash
# Install required tools
cargo install cargo-make
cargo install cargo-watch

# Format code
cargo make format

# Check code (basic compilation check)
cargo make check

# Run linting with clippy
cargo make clippy

# Run tests
cargo make test

# Generate documentation
cargo make doc

# Run specific CLI commands
cargo run --package reearth-flow-cli -- run --workflow path/to/workflow.yml
cargo run --package reearth-flow-cli -- schema-action
cargo run --package reearth-flow-cli -- doc-action
```

### Development Dependencies
- **Linux/Debian**: `libxml2-dev`, `pkg-config`
- **macOS**: `brew install libxml2 pkg-config`
- **Windows**: vcpkg with libxml2
- **Optional WASM support**: Python 3.11 + `pip install py2wasm`

## Architecture Overview

This is a **DAG-based geospatial workflow execution engine** with the following key components:

### Core Workflow System
- **Workflows** are defined in YAML/JSON with nodes (actions) connected by edges (data flow)
- **Three node types**: Sources (data input), Processors (transformation), Sinks (output)
- **Multi-threaded execution**: Each node runs in its own thread with channel-based communication
- **Feature-centric**: Primary data unit is a `Feature` with attributes, geometry, and metadata

### Key Directories
- `runtime/runtime/` - Core execution engine, DAG construction, thread orchestration
- `runtime/types/` - Core data structures (Feature, Geometry, Workflow definitions)
- `runtime/action-*` - Action implementations (source, processor, sink, plateau-specific)
- `runtime/geometry/` - Comprehensive 2D/3D geometry operations
- `runtime/eval-expr/` - Rhai-based expression evaluation system
- `cli/` - Command-line interface
- `worker/` - Distributed execution worker component

### Action System
Actions are implemented using factory patterns with three traits:
- `SourceFactory` - Data ingestion (files, databases, synthetic data)
- `ProcessorFactory` - Data transformation (geometry ops, attribute manipulation)  
- `SinkFactory` - Data export (various file formats, 3D tiles, vector tiles)

Each action defines input/output ports, JSON schema for validation, and parameter handling.

### Expression System
- Uses **Rhai scripting language** for dynamic parameter evaluation
- Available in action parameters and workflow variables
- Built-in modules for file ops, JSON processing, string manipulation
- Feature context provides access to attributes, metadata, geometry

### Environment Variables
Runtime behavior controlled by `FLOW_RUNTIME_*` variables:
- `FLOW_RUNTIME_THREAD_POOL_SIZE` - Worker thread count (default: 30)
- `FLOW_RUNTIME_CHANNEL_BUFFER_SIZE` - Inter-node channel buffer (default: 256)
- `FLOW_RUNTIME_FEATURE_FLUSH_THRESHOLD` - Sink flush threshold (default: 512)

Workflow variables use `FLOW_VAR_*` prefix for environment injection.

## Specialized Components

### PLATEAU Processing
`action-plateau-processor/` contains specialized processors for Japanese PLATEAU 3D city model standard:
- CityGML validation and attribute extraction
- LOD (Level of Detail) processing
- Domain validation for Japanese urban data

### Storage Abstraction
Multi-backend storage system built on **OpenDAL** supports:
- Local filesystem (`file://`)
- Google Cloud Storage (`gs://`) with emulator support
- HTTP endpoints (`http://`, `https://`)
- In-memory storage (`ram://`) for testing
- Built-in layers: timeout, retry with jitter, logging, metrics

## Common Development Patterns

### Adding New Actions
1. Create factory struct implementing appropriate trait (`SourceFactory`, `ProcessorFactory`, `SinkFactory`)
2. Define parameter struct with serde + schemars derives
3. Implement `build()` method returning action instance
4. Register in appropriate mapping file
5. Add i18n entries in `schema/i18n/actions/`
6. Run `cargo make doc-action` to update schemas

### Testing
- Integration tests in `runtime/tests/` with YAML workflow fixtures
- Unit tests alongside implementation files
- Use `pretty_assertions` for better diff output

### Error Handling
- Use `thiserror` for custom error types
- Propagate errors through `Result<T, E>` patterns
- Event system captures and broadcasts execution errors

## Intermediate Data & Caching for Debugging

### Working Directory Structure
```
<working_directory>/
├── projects/<project_key>/
│   ├── jobs/<job_id>/
│   │   ├── feature-store/        # Feature data streams by edge ID
│   │   ├── action-log/           # Action execution logs
│   │   └── temp/                 # Job-specific temporary files
│   └── temp/<temp_id>/           # Project-level temporary files
```

### Feature Store - Intermediate Data Capture
- **All intermediate feature data** is automatically captured as features flow between nodes
- **File format**: JSON Lines (`.jsonl`) - one feature per line, organized by edge ID
- **Location**: `<job_id>/feature-store/<edge_id>.jsonl`
- **Critical for debugging**: Examine exact data transformations between workflow nodes

### Environment Variables for Data Handling
- **`FLOW_RUNTIME_WORKING_DIRECTORY`**: Override default cache directory location
  - Default: macOS `$HOME/Library/Caches/<project_path>`, Linux `$HOME/.cache/<project_path>`
- **`FLOW_RUNTIME_FEATURE_WRITER_DISABLE`**: Set to "true" to disable intermediate data capture (impacts debugging)
- **`FLOW_RUNTIME_FEATURE_FLUSH_THRESHOLD`**: Buffer size before writing to disk (default: 512)

### Debugging Workflow Issues
**View intermediate data**:
```bash
# Examine feature data for specific workflow edge
cat <working_dir>/projects/<project>/jobs/<job_id>/feature-store/<edge_id>.jsonl

# Check action execution logs
ls <working_dir>/projects/<project>/jobs/<job_id>/action-log/
```

**Event system provides real-time monitoring**:
- Node status changes (Starting, Processing, Completed, Failed)
- Edge completion events (data flow tracking)
- Structured logging with node context

### State Persistence
- **Key-Value Store**: Runtime state shared across workflow nodes
- **Object Storage**: JSON/JSONL persistence through State API
- **Temporary Files**: Automatic cleanup of job-specific temporary data

**Note**: Keep feature writing enabled during development - intermediate data is essential for debugging complex workflow issues.

## Git Commit Guidelines

When creating git commits, do not include Claude Code attribution or "Generated with Claude Code" messages in commit messages. Keep commit messages clean and focused on the actual changes made.