# PLATEAU Tiles Test

Testing framework for aligning flow outputs containing tile files, with FME outputs.

## Install and run

1. install uv (python package manager)
2. cd into this directory and run `sh setup.sh`
3. run `uv run python3 -m plateau-tiles-test`

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

## Steps to create a test

1. Prepare the original CityGML zip under `$CITYGML_SRCDIR`
2. Create `testcases/{workflow-path}/{desc}/profile.toml` with filter configuration
3. Run `uv run python3 -m plateau-tiles-test {workflow-path}/{desc} g` to:
   - Extract codelists/schemas to `artifacts/citymodel/{zip_stem}/`
   - Extract filtered GML files to `testcases/{workflow-path}/{desc}/citymodel/udx/`
   - Pack runtime zip to `results/{workflow-path}/{desc}/{zip_name}`
4. Run FME with the packed zip from `results/{workflow-path}/{desc}/{zip_name}`
   - FME uses 3dtiles v1.0 + draco compression. Workaround: modify FME workflows to export JSON files (csmapreprojector + coordinateswapper needed to match cesium processing)
   - Rename `.mvt` -> `.pbf` if necessary
   - Zip FME output to `testcases/{workflow-path}/{desc}/fme.zip`
5. Run `uv run python3 -m plateau-tiles-test {workflow-path}/{desc} re` to test

## Implemented tests

- `mvt_attributes` - Compare MVT tile attributes.
- `mvt_polygons` - Compare MVT polygon geometries using symmetric difference area.
- `mvt_lines` - Compare MVT line geometries using Hausdorff distance of lines and polygon outlines.
  - Also used for testing polygon topology. For example, polygons before and after union cannot be distinguished by polygon tests.
  - To avoid corner cases, the frame of current tile is added to the line segment set.
- `3dtiles_attributes` - Compare 3D Tiles feature attributes.

## Stages

- `g` - Generate: Extract source zip to artifacts + testcase structure, pack runtime zip
- `r` - Run: Pack runtime zip (if not exists) and execute workflow
- `e` - Evaluate: Compare flow output with FME reference

## Todo

- support draco decoding for 3dtiles v1.1