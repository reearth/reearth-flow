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
- `testcases/{workflow-path}/{category}/` - Test-specific data (tracked in git)
  - `{workflow-path}` is relative to `runtime/examples/fixture/workflow/` (e.g., `data-convert/plateau4/02-tran-rwy-trk-squr-wwy`)
  - `profile.toml` - Test configuration (`workflow_path` is optional, auto-derived from directory structure)
  - `citymodel/udx/` - Test-specific GML files (filtered from source)
  - `fme/` - Reference FME output directory with tile files
- `results/{workflow-path}/{desc}/` - Runtime outputs (gitignored)
  - `{zip_name}` - Packed citymodel zip (generated from artifacts + testcase)
  - `flow/` - Flow tile outputs
  - `fme_extracted/` - FME tile outputs extracted for comparison
  - `flow_extracted/` - Flow tile outputs extracted for comparison
  - `runtime/` - Flow intermediate data

## Caveats

- draco decoding not supported (TODO), disable it in the workflow to test.
- 3D tiles v1.0 `.b3dm` output by FME is not supported. Use [3d-tiles-tool](https://github.com/CesiumGS/3d-tiles-tools) to upgrade it.
- FME's MVT writer split features with `aggregate` type of geometry into multiple features. Use `GeometryRefiner` to merge them before export.

### Ignored differences in attribute test

- ignore empty string vs Null difference: limitation of `3d-tiles-tools` upgrading when constructing `fme.zip`
- ignore string vs int type difference: FME has implicit type conversion from string to integer
- ignore bool vs int difference: FME outputs integer but using native bool is possibly better

## Tests

- `json_attributes` - Compare JSON outputs.
- `mvt_attributes` - Compare MVT tile attributes.
- `mvt_polygons` - Compare MVT polygon geometries using symmetric difference area.
- `mvt_lines` - Compare MVT tiles linestrings and polygon outliers.
- `cesium_attributes` - Compare 3D Tiles feature attributes.
- `cesium_statistics` - Compare 3D Tiles triangle meshes from statistical values.

## Run single test

Run single test with

```
cargo run -p plateau-tiles-test -- <toml_path> [stages]
```

Stages:

- `r` - Run: Pack runtime zip (if not exists) and execute workflow
- `e` - Evaluate: Compare flow output with FME reference

## Notes about designing robust linestrings comparison algorithm

While XOR area is a robust comparison for polygons, for line segments robust comparison is hard.

- Hausdorff + segmentize: has problem if a single outlier is far away from other parts
- Count outliers from the Hausdorff distribution: need to specify a hard threshold
- Rasterization + RMS: has cumulative error with massive small drift
- (Current implementation) Rasterization with threshold (>0.5) difference + tile extent undersampling (1024 vs >= 4096). It is robust as long as the pixelated lines generally match.
