# plateau6-citymodel fixtures

Shared citymodel fixtures (codelists / schemas / objectlist) for the PLATEAU6
(CityGML 3.0 / i-UR 4.0) quality-check workflow tests.

The `z` prefix (e.g. `z16211_...`) marks a fixture as test-only data generated locally by
rewriting a real city's PLATEAU5 (CityGML 2.0 / i-UR 3.x) data into CityGML 3.0 / i-UR 4.0 — not
official data. The single letter keeps these sorted after the digit-prefixed real city codes.

## `__L03test` objectlists

Each `*_objectlist_op__L03test.xlsx` is a copy of the objectlist next to it,
edited for the tests that expect no L03 error. The original marks 作成対象 (J)
only on the attributes C05-C08 are tested with, so every feature type the test
GML uses was undefined or not allowed. The copy marks J on the rows the test
GML uses and changes nothing else, except where noted.

- `z16211_imizu-shi_city_2024_citygml_1_op/16211_city_2024_objectlist_op__L03test.xlsx`
  (`01-01-common/Z-common-00_no-error`, `02-bldg/Z-bldg-08_read-errors`),
  sheet `01. 建築物モデル`: rows 3 (`bldg:Building`), 6, 1212, 1218,
  1267-1270 and 1377.
- `z11100_saitama-shi_city_2025_citygml_1_op/11100_city_2025_objectlist_op__L03test.xlsx`
  (`03-tran/common/C05`), sheet `02. 交通（道路）モデル`: rows 3 (`tran:Road`),
  510, 512, 515 and 1258.
- `z08220_tsukuba-shi_city_2023_citygml_2_op/08220_city_2023_objectlist_op__L03test.xlsx`
  (`08-dem/common/C05`, `09-brid-tun-ubld/common/C05`):
  - sheet `10. 橋梁モデル`: rows 3 (`brid:Bridge`), 510, 520, 521, 1210-1212,
    1214, 1218, 1220, 1312, 1314, 1319 (`brid:BridgeConstructiveElement`), 2532,
    2580 (`brid:BridgeInstallation`), 3793 and 3833. Rows 3840 (`con:WallSurface`)
    and 3841 (its `core:lod3MultiSurface`) are added, marked J: the sheet has
    no `con:WallSurface`, which the bridge sheet of the sample package has.
  - sheet `17. 地形モデル`: rows 3 (`dem:ReliefFeature`), 510, 512, 515, 526,
    528, 532, 1210 (`dem:TINRelief`) and 2418.
