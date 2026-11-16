# 13997_tokyo_udx-mlit_2026_citygml_1_sample-tokyoCG3_op

Codelists, i-UR 4.0 schemas and the objectlist of a CityGML 3.0 / i-UR 4.0
sample package under development for city code 13997. The files are copied
unchanged, but only the parts the tests use are kept:

- `codelists/`: codelists of the package.
- `schemas/`: the i-UR 4.0 schemas of the package.
- `13997_milt_2026_objectlist_sample-tokyoCG3_op.xlsx`: the package objectlist
  with only the `01. 建築物モデル` and `20. 汎用都市オブジェクトモデル` sheets
  left. The sheets themselves are unchanged. Add a sheet from the package when a
  test needs another model.

Edited copies of the objectlist:

- `13997_milt_2026_objectlist_sample-tokyoCG3_op__empty.xlsx`: no rows, for tests
  that do not test the objectlist checks.
- `13997_milt_2026_objectlist_sample-tokyoCG3_op__L03test.xlsx`: 作成対象 cleared
  where the L03 test GML lacks the attribute, so that it has L03 errors only.
