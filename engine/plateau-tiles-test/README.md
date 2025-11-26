# PLATEAU Tiles Test

Testing framework for aligning flow outputs containing tile files, with FME outputs.

## Install and run

- cd to this directory
- install python3, uv
- init venv: `uv venv`
- install dependencies: `uv pip install -r requirements.txt`
- run `uv run python3 -m plateau-tiles-test`
- run individual tests with `uv run python3 -m plateau-tiles-test <test_name> [stages]`

## Directory structure

- `testcases`: store testing profiles.
- `results`: store evaluation results and intermediate data
  - `results/fme`: extracted fme outputs
  - `results/flow`: flow outputs
  - `results/runtime`: flow intermediate data
  - tests should be small so these files are kept persistently.

## Steps to create a test

1. prepare the original CityGML zip under `$CITYGML_SRCDIR`
2. create `testcases/<testname>/profile.toml`, create a filter to minimize features to be tested.
3. `uv run python3 -m plateau-tiles-test <testname> g` which generates the filtered zip, update `profile.toml` to use that zip.
4. run FME with the filtered zip
  - FME's uses 3dtiles v1.0 + draco compression which cannot be handled currently. The workaround is to modify FME workflows to export JSON files (csmapreprojector + coordinateswapper needed to match cesium processing result).
  - rename `.mvt` -> `.pbf` if necessary.
  - save FME output to artifacts path and update `profile.toml` to use that file.

## Todo

- support draco decoding for 3dtiles v1.1