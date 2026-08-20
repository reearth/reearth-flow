# PLATEAU Tiles Test

Testing framework for aligning flow outputs containing tile files, with truth outputs.

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
- `e` - Evaluate: Compare flow output with truth reference

Truth files live under a testcase's `truth/` dir. Some are hand-maintained directly (`generate_truth: false`); others (`generate_truth: true`) are derived from a raw source also checked into `truth/` (e.g. a raw 3D Tiles zip), and regenerated with:

```sh
cargo run -p plateau-tiles-test --bin generate-truth -- <toml_path>
```

## Test types

- `json_attributes(_v2)` - Attribute comparison for JSON/GeoJSON output
- `mvt_polygons` / `mvt_lines` / `mvt_points` - Geometry comparison for MVT tile output
- `raster` - Pixel comparison of rasterized MVT geometry (`convs.mvt_png` renders MVT to PNG; antialiased coverage in `[0, 1]`, subpixel diffs ignored via a `0.5` dead-zone)
- `cesium` - Attribute comparison for 3D Tiles/glTF output
- `cesium_statistics` - Per-feature geometric fingerprint (bbox, centroid, average winding, texture presence) for 3D Tiles/glTF output — catches gross regressions but not subtle shape distortions
- `raster3d` - Depth-buffer comparison of rendered 3D Tiles/glTF output (`convs.raster3d` renders named cameras, configured in `profile.toml` with ECEF xyz `position`/`look_at`, into lossless-f32 depth PNGs; see `render3d/` for the renderer and `Canvas::compare_depth` for the comparison)

## Directory structure

- `../data/testcases/{workflow-path}/{category}/` - Test-specific data (tracked in git, located in `testing/data/testcases/`)
  - `{workflow-path}` is relative to `runtime/examples/fixture/workflow/` (e.g., `data-convert/plateau4/02-tran-rwy-trk-squr-wwy`)
  - `profile.toml` - Test configuration (`workflow_path` is optional, auto-derived from directory structure)
  - `citymodel/udx/` - Test-specific GML files (filtered from source)
  - `citymodel/{codelists,schemas}` - Symlink to corresponding citymodel data
  - `truth/` - Reference truth output directory
- `../data/results/{workflow-path}/{desc}/` - Runtime outputs (gitignored, located in `testing/data/results/`)
  - `{zip_name}` - Packed citymodel zip (generated from artifacts + testcase)
  - `flow/` - Flow tile outputs
  - `truth_extracted/` - extracted truth
  - `flow_extracted/` - Flow tile outputs extracted for comparison
  - `runtime/` - Flow intermediate data

## Caveats

- `3d-tiles-tools` has several [problems](https://github.com/reearth/reearth-flow/pull/1841) especially when testing tiles containing multiple features.
- truth's MVT writer split features with `aggregate` type of geometry into multiple features. Use `GeometryRefiner` to merge them before export.
- ignore bool vs int difference: truth outputs integer but using native bool is possibly better
- ignore padded whitespace differences
