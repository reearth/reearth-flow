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

## Legacy attribute keys

Truths are compared with Flow's attribute keys as they are. A truth made before the attribute keys became
valid 3D Tiles property IDs is marked `legacy_keys = true` on its `convs.cesium_attributes`, `convs.mvt_attributes`,
`convs.json` or `tests.json_attributes` entry, and Flow's keys are converted to the legacy keys before comparison:

- `__` becomes `:`, also in the nested `attributes` and `dm_attributes` JSON
- risk columns such as `fld_1_1_2_rank` are named by the label in their `{group}_name` column, e.g. `{label}_浸水ランク`
- generic attribute columns `gen_{i}_value` and `gen_{i}_uom` are named by `gen_{i}_name`

A Flow key with a colon fails the conversion. New truths should not set the flag.

## Caveats

- `3d-tiles-tools` has several [problems](https://github.com/reearth/reearth-flow/pull/1841) especially when testing tiles containing multiple features.
- truth's MVT writer split features with `aggregate` type of geometry into multiple features. Use `GeometryRefiner` to merge them before export.
- ignore bool vs int difference: truth outputs integer but using native bool is possibly better
- ignore padded whitespace differences
