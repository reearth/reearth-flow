# 13997_tokyo_udx-mlit_2026_citygml_1_sample-tokyoCG3_op

Codelists, i-UR 4.0 schemas and the objectlist of a CityGML 3.0 / i-UR 4.0
sample package under development for city code 13997. The files are copied
unchanged, but only the parts the tests use are kept:

- `codelists/`: the codelists the test GML refers to, for the common checks
  (including the buildings cut out for L03) and the flood risk (fld, htd) data
  conversion.
- `schemas/`: the i-UR 4.0 schemas of the package.
- `13997_milt_2026_objectlist_sample-tokyoCG3_op.xlsx`: the package objectlist
  with only the `01. 建築物モデル` and `20. 汎用都市オブジェクトモデル` sheets
  left. The sheets themselves are unchanged. Add a sheet from the package when a
  test needs another model.

One file is edited for tests rather than copied:

- `13997_milt_2026_objectlist_sample-tokyoCG3_op_empty.xlsx`: the objectlist
  above with every row after the two header rows deleted from each sheet. It
  defines no feature type, so the objectlist checks (the L03 attribute and
  association role check, C05-C08) have nothing to check. Tests that do not
  test those checks use it.
