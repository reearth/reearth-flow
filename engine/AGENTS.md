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

# Sync i18n skeleton files (adds missing keys, removes stale, preserves existing translations)
cargo make scaffold-i18n
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
2. Define parameter struct with `serde` + `schemars` derives — use `#[schemars(title = "...", description = "...")]` on fields for English display strings
3. Implement `build()` method returning action instance
4. Register in appropriate mapping file
5. Run `cargo make scaffold-i18n` — adds empty i18n skeleton entries for the new action across all language files
6. Fill in translated strings in `schema/i18n/actions/{lang}.json` for each language
7. Run `cargo make doc-action` to regenerate all schema files

### Action i18n

Translated action schemas are generated from source files in `schema/i18n/actions/{lang}.json`. **Never edit the generated `schema/actions_*.json` files directly** — they are always overwritten by `cargo make doc-action`.

Each i18n entry supports:

```json
{
  "name": "MyAction",
  "description": "Translated action description",
  "parameterI18n": {
    "someProperty": { "title": "Translated title", "description": "Translated description" }
  },
  "definitionI18n": {
    "SomeDefinition": {
      "fieldName": { "title": "Translated title" }
    }
  }
}
```

- `parameterI18n` — keys are top-level parameter property names (from `schema["properties"]`)
- `definitionI18n` — keys are definition names (from `schema["definitions"]`), values are maps of property name → i18n
- Both fields are optional; missing or empty values fall back to the English strings from schemars annotations
- Property names come directly from Rust field names after camelCase conversion (predictable without running the generator)
- `cargo make scaffold-i18n` reconciles all lang files: adds missing keys, removes stale keys, preserves existing translations

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
