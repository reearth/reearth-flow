# PLATEAU Tiles Test

Testing framework for aligning flow outputs containing tile files, with FME outputs.

## Run

```sh
cargo run -p plateau-tiles-test
```

Run single test with

```
cargo run -p plateau-tiles-test -- <toml_path> [stages]
```

Stages:

- `r` - Run: Pack runtime zip (if not exists) and execute workflow
- `e` - Evaluate: Compare flow output with FME reference

## Directory structure

- `../data/testcases/{workflow-path}/{category}/` - Test-specific data (tracked in git, located in `testing/data/testcases/`)
  - `{workflow-path}` is relative to `runtime/examples/fixture/workflow/` (e.g., `data-convert/plateau4/02-tran-rwy-trk-squr-wwy`)
  - `profile.toml` - Test configuration (`workflow_path` is optional, auto-derived from directory structure)
  - `citymodel/udx/` - Test-specific GML files (filtered from source)
  - `citymodel/{codelists,schemas}` - Symlink to corresponding citymodel data
  - `fme/` - Reference truth output directory (not necessarily FME outputs)
- `../data/results/{workflow-path}/{desc}/` - Runtime outputs (gitignored, located in `testing/data/results/`)
  - `{zip_name}` - Packed citymodel zip (generated from artifacts + testcase)
  - `flow/` - Flow tile outputs
  - `fme_extracted/` - extracted truth
  - `flow_extracted/` - Flow tile outputs extracted for comparison
  - `runtime/` - Flow intermediate data

## Caveats

- `3d-tiles-tools` has several [problems](https://github.com/reearth/reearth-flow/pull/1841) especially when testing tiles containing multiple features.
- FME's MVT writer split features with `aggregate` type of geometry into multiple features. Use `GeometryRefiner` to merge them before export.
- ignore bool vs int difference: FME outputs integer but using native bool is possibly better