# Engine Architecture

## Core Workflow System

- **Workflows** are defined in YAML/JSON with nodes (actions) connected by edges (data flow)
- **Three node types**: Sources (data input), Processors (transformation), Sinks (output)
- **Multi-threaded execution**: Each node runs in its own thread with channel-based communication
- **Feature-centric**: Primary data unit is a `Feature` with attributes, geometry, and metadata

## Expression System

- Uses **Rhai scripting language** for dynamic parameter evaluation
- Available in action parameters and workflow variables
- Built-in modules: `math::`, `file::`, `json::`, `xml::`, `str::`, `datetime::`, `collection::`, `console::`
- **math:: module** provides comprehensive mathematical functions:
  - Trigonometry: `sin()`, `cos()`, `tan()`, `asin()`, `acos()`, `atan()`, `atan2()`
  - Constants: `pi()`, `e()`
  - Utilities: `sqrt()`, `pow()`, `abs()`, `max()`, `min()`, `floor()`, `ceil()`, `round()`
  - Angle conversion: `to_radians()`, `to_degrees()`
- Feature context provides access to attributes, metadata, geometry via `env.get("__value")`

## Environment Variables

Runtime behavior controlled by `FLOW_RUNTIME_*` variables.

### Execution

- `FLOW_RUNTIME_THREAD_POOL_SIZE` - Worker thread count (default: 30)
- `FLOW_RUNTIME_CHANNEL_BUFFER_SIZE` - Inter-node channel buffer (default: 256)
- `FLOW_RUNTIME_ACCUMULATING_FINISH_CONCURRENCY` - Maximum number of accumulating processors that can run `finish()` concurrently (default: 1). Accumulating processors (e.g., aggregators, sorters) buffer data during processing and emit results during `finish()`. Setting this to 1 serializes finish operations to reduce peak memory usage. Increase if you have sufficient memory and want more parallelism.

### Data Handling & Debugging

- `FLOW_RUNTIME_WORKING_DIRECTORY` - Override default cache directory location. Default: macOS `$HOME/Library/Caches/<project_path>`, Linux `$HOME/.cache/<project_path>`
- `FLOW_RUNTIME_FEATURE_FLUSH_THRESHOLD` - Buffer size before writing features to disk (default: 512)
- `FLOW_RUNTIME_FEATURE_WRITER_DISABLE` - Set to `"true"` to disable intermediate data capture (impacts debugging)

### Workflow Variables

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

## Runtime Architecture

### Actor Model Implementation

The engine implements an actor model where each node (source, processor, sink) runs in its own OS thread:

1. **Async Context Separation**:
   - CLI entry: `cargo run --package reearth-flow-cli -- run`
   - Runner: `runtime.block_on(orchestrator.run_all(...))`
   - Orchestrator: `runtime.spawn_blocking(move || run_dag_executor(...))`
   - Each node spawned with `std::thread::Builder::new().spawn()`

2. **Why This Design**:
   - **Action simplicity**: Actions can be written synchronously without async complexity
   - **True parallelism**: Each node runs on its own OS thread
   - **Isolation**: Blocking operations in one node don't affect others
   - **Message passing**: Actors communicate via channels (crossbeam)

3. **Tokio's Role**:
   - **Event system**: Async event handlers for logging, monitoring
   - **Shutdown coordination**: `tokio::sync::watch` for graceful shutdown
   - **I/O efficiency**: Async I/O for sources reading files/network
   - **spawn_blocking management**: Tokio manages the thread pool

## Integration Points

### With Server

- **Workflow definitions** - Receives YAML/JSON workflows from server
- **Job execution** - Coordinated via Google Cloud Batch
- **Results** - Writes output to cloud storage (GCS/S3)
- **Logs** - Publishes execution logs to Google Pub/Sub

### With UI

- **Action schemas** - Provides JSON schemas for action configuration
- **Workflow validation** - Validates workflow structure and connections
- **Documentation** - Generates action documentation for UI display

## Intermediate Data & Debugging

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

Data handling variables are listed in [Environment Variables > Data Handling & Debugging](#data-handling--debugging).

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

**Note**: Keep feature writing enabled during development - intermediate data is essential for debugging complex workflow issues.

## State Persistence

- **Key-Value Store**: Runtime state shared across workflow nodes
- **Object Storage**: JSON/JSONL persistence through State API
- **Temporary Files**: Automatic cleanup of job-specific temporary data

