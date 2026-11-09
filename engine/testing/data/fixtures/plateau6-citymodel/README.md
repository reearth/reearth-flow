# plateau6-citymodel fixtures

Shared citymodel fixtures (codelists / schemas / objectlist) for the PLATEAU6
(CityGML 3.0 / i-UR 4.0) quality-check workflow tests.

The `z` prefix (e.g. `z16211_...`) marks a fixture as test-only data generated locally by
rewriting a real city's PLATEAU5 (CityGML 2.0 / i-UR 3.x) data into CityGML 3.0 / i-UR 4.0 — not
official data. The single letter keeps these sorted after the digit-prefixed real city codes.

## `__L03test` objectlists

Each `*_objectlist_op__L03test.xlsx` is an edited copy of the objectlist next to
it, for the tests that expect no L03 error.

## `__empty` objectlists

`z08220_tsukuba-shi_city_2023_citygml_2_op/08220_city_2023_objectlist_op__empty.xlsx`
keeps only the header rows of each sheet, for tests that do not test the
objectlist checks.
