# PLATEAU Tiles Test

Testing framework for aligning flow outputs containing tile files, with FME outputs.

## Install and run

1. install uv (python package manager)
2. cd into this directory and run `sh setup.sh`
3. run `uv run python3 -m plateau-tiles-test`

## Directory structure

- `testcases/<name>`: store testing profiles.
- `results`: store evaluation results and intermediate data
  - `results/<name>/fme`: extracted fme outputs
  - `results/<name>/flow`: flow outputs
  - `results/<name>/runtime`: flow intermediate data
  - tests should be small so these files are kept persistently.
- `<name>` naming pattern: `<city_code>_<city_name>_<workflow>_<extra_description>.toml`

## Steps to create a test

1. prepare the original CityGML zip under `$CITYGML_SRCDIR`
2. create `testcases/<name>/profile.toml`, create a filter to minimize features to be tested.
3. `uv run python3 -m plateau-tiles-test <name> g` which generates the filtered zip, update `profile.toml` to use that zip.
4. run FME with the filtered zip
  - FME's uses 3dtiles v1.0 + draco compression which cannot be handled currently. The workaround is to modify FME workflows to export JSON files (csmapreprojector + coordinateswapper needed to match cesium processing result).
  - rename `.mvt` -> `.pbf` if necessary.
  - zip FME output to `testcases/<name>/fme.zip`

## Todo

- support draco decoding for 3dtiles v1.1