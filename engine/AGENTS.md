# AGENTS.md

DAG-based geospatial workflow execution engine. See [../AGENTS.md](../AGENTS.md) for monorepo-level guidance.

## Development Commands

```bash
# Install required tools
cargo install cargo-make
cargo install cargo-watch

# Build, lint, test
cargo make format
cargo make check
cargo make clippy
cargo make test

# Generate documentation
cargo make doc

# CLI commands
cargo run --package reearth-flow-cli -- run --workflow path/to/workflow.yml
cargo run --package reearth-flow-cli -- schema-action
cargo run --package reearth-flow-cli -- doc-action
```

## Development Dependencies

- **Linux/Debian**: `apt-get install libxml2-dev pkg-config libproj-dev`
- **macOS**: `brew install libxml2 pkg-config proj`
- **Windows**: vcpkg with libxml2 and proj

PROJ library is required for coordinate system transformations (HorizontalReprojector action).

## Key Directories

- `runtime/runtime/` - Core execution engine, DAG construction, thread orchestration
- `runtime/types/` - Core data structures (Feature, Geometry, Workflow definitions)
- `runtime/action-*` - Action implementations (source, processor, sink, plateau-specific)
- `runtime/geometry/` - Comprehensive 2D/3D geometry operations
- `runtime/eval-expr/` - Rhai-based expression evaluation system
- `cli/` - Command-line interface
- `worker/` - Distributed execution worker component
- `testing/` - Workflow integration tests and tile output validation tests

## Action System

Actions are implemented using factory patterns with three traits:

- `SourceFactory` - Data ingestion (files, databases, synthetic data)
- `ProcessorFactory` - Data transformation (geometry ops, attribute manipulation)
- `SinkFactory` - Data export (various file formats, 3D tiles, vector tiles)

Each action defines input/output ports, JSON schema for validation, and parameter handling.

### Adding New Actions

1. Create factory struct implementing appropriate trait
2. Define parameter struct with `serde` + `schemars` derives
3. Implement `build()` method returning action instance
4. Register in appropriate mapping file
5. Add i18n entries in `schema/i18n/actions/`
6. Run `cargo make doc-action` to update schemas

## Key Constraints

- Workflow variables use `FLOW_VAR_*` prefix for environment injection

## Testing

Three test suites run via `cargo make test`:

```bash
cargo make test-rs    # Unit tests (workspace-wide, excludes workflow-tests)
cargo make test-qc    # Workflow integration tests (quality-check)
cargo make test-conv  # Tile output validation tests (3D Tiles, MVT, raster comparison)
```

- **Unit tests** — alongside implementation files, use `pretty_assertions`
- **`runtime/tests/`** — small integration tests with YAML workflow fixtures
- **`testing/workflow-tests/`** — end-to-end workflow tests. Test cases defined in `testing/data/testcases/` as `workflow_test.json`. `build.rs` auto-generates test functions from these files
- **`testing/plateau-tiles-test/`** — tile output comparison tests. Configured via `profile.toml`, compares flow output against truth data
- **`testing/data/fixtures/`** — shared PLATEAU CityGML test data used across test suites

To run a single workflow test:

```bash
cargo test -p workflow-tests test_quality_check_plateau4_02_bldg_t_bldg_02
```

## Code Quality

**Before completing any task, always run:**

```bash
cargo make format
cargo make clippy
cargo make test
```

## Documentation

- [Engine Architecture](dev-docs/architecture.md) - Runtime design, expression system, environment variables, debugging
