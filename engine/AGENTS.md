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

# Action schema generation (run in order when adding/modifying actions)
cargo make schema-base        # generates actions.json + syncs i18n skeletons
cargo make schema-translated  # generates actions_{lang}.json + docs from i18n files
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

Use the `add-action` skill for a full step-by-step guide including i18n workflow.

### Sandbox & sink writes

Sinks accept a **relative path** for their `output` parameter. The engine
joins the path against `ctx.sandbox_root` (the per-job artifact directory)
and validates the result via `sandbox::ensure_under`. The chokepoint is
`reearth_flow_action_sink::SinkOutput::new`.

Workflow authors write:

```yaml
output:
  type: string
  value: "out.gpkg"           # or "group/a.geojson", with attribute concat for dynamic names
```

The engine joins this against whatever `sandbox_root` the worker/CLI was
launched with — `file:///var/jobs/abc/`, `gs://bucket/jobs/abc/`,
`ram:///jobs/abc/`, etc. The same workflow YAML is portable across storage
backends.

`SinkOutput::new(sandbox_root, relative_path, resolver)` rejects:
- Empty strings, leading/trailing whitespace, literal `.` / `..`
- Absolute URIs (`scheme://...`) — error message names `workerArtifactPath`
  to direct customers to the migration
- Leading `/` (ambiguous) or leading `~` (home expansion not supported)
- Paths that resolve to the artifact directory itself after normalization
- Paths that escape the sandbox via `..` (caught by `ensure_under`)

**All sink-side writes MUST go through `SinkOutput::new` → `SinkOutput::write`.**
Calling `Storage::put_sync` (or any other raw I/O like `std::fs::write`)
from a sink or sink-adjacent code path skips the check and reintroduces
unbounded writes. There is no public way to construct a `SinkOutput`
without passing through `SinkOutput::new`, which always validates.
Buffering sinks (cesium3dtiles, mvt, FeatureWriter) buffer features keyed
by the relative path **String** and call `SinkOutput::new` once per output
file at flush time. Sinks that build hierarchical paths (e.g. cesium tile
coordinates) **compose path strings directly** (`format!("{base}/{child}")`)
and call `SinkOutput::new` per file — there is no `SinkOutput::join`
method; the unified constructor is the only chokepoint. Reviewers should
flag any direct `put_sync` / `std::fs` calls in sink code as regressions.

Validation happens exclusively inside `SinkOutput::new`. There is no
separate "validate-only" public API — bad paths fail at flush time when
the sink calls `SinkOutput::new`, not at intake. Single chokepoint, single
validation site, no way for callers to manually pre-check (and therefore
no way to drift out of sync with the real check).

Production entrypoints (`Runner::run_with_sandbox_root`,
`AsyncRunner::run_with_sandbox_root`) reject the `file:///` sentinel so a
misconfigured `workerArtifactPath` cannot silently disable the sandbox.
`Runner::run` (legacy / tests) intentionally uses that sentinel and
bypasses the guard.

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
