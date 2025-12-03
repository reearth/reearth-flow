# PLATEAU Tiles Test

Testing framework for aligning flow outputs containing tile files, with FME outputs.

## Run

```sh
cargo run -p plateau-tiles-test
```

## Directory structure

- `artifacts/citymodel/{zip_stem}/` - Shared codelists and schemas extracted from source zips (tracked in git)
  - `codelists/` - Shared codelist files
  - `schemas/` - Shared schema files
- `testcases/{workflow-path}/{desc}/` - Test-specific data (tracked in git)
  - `{workflow-path}` is relative to `runtime/examples/fixture/workflow/` (e.g., `data-convert/plateau4/02-tran-rwy-trk-squr-wwy`)
  - `{desc}` is the test description (e.g., `rwy`, `multipolygon`)
  - `profile.toml` - Test configuration (`workflow_path` is optional, auto-derived from directory structure)
  - `fme.zip` - Reference FME output
  - `citymodel/udx/` - Test-specific GML files (filtered from source)
- `results/{workflow-path}/{desc}/` - Runtime outputs (gitignored)
  - `{zip_name}` - Packed citymodel zip (generated from artifacts + testcase)
  - `fme/` - Extracted FME outputs
  - `flow/` - Flow outputs
  - `runtime/` - Flow intermediate data

## Caveats

- draco decoding not supported (TODO), disable it in workflow to test.
- FME outputs 3D tiles v1.0 `.b3dm` files which is not supported. Manually edit FME workflows by replacing FME 3d tiles writer with `<CsmapReprojector> -> <CoordinateSwapper> -> <JSON FeatureWriter>`
- FME's MVT writer split features with `aggregate` type of geometry into multiple features. Use `GeometryRefiner` to merge them before export.

## Tests

- `mvt_attributes` - Compare MVT tile attributes.
- `mvt_polygons` - Compare MVT polygon geometries using symmetric difference area.
- `3dtiles_attributes` - Compare 3D Tiles feature attributes.
- `json_attributes` - Compare JSON outputs.
- (TODO) `3dtiles_lines` - Compare 3D Tiles meshes using lines.
- (TODO) `MVT_lines` - Compare MVT tiles linestrings and polygon outliers.

## Run single test

Run single test with

```
cargo run -p plateau-tiles-test -- <toml_path> [stages]
```

Stages:

- `r` - Run: Pack runtime zip (if not exists) and execute workflow
- `e` - Evaluate: Compare flow output with FME reference