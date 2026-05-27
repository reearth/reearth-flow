# `attributes[key]` flowExpr Migration Analysis

**Context**: `attributes[key]` now errors (instead of returning null) when key does not exist.
This means `attributes[X] ?? fallback` and `attributes[X] or Y` patterns are also broken —
the error fires before `??`/`or` can catch it. Migration options: use `.get("key")` which returns null on miss.

Scope: **only `type: flowExpr` blocks** in source `workflow.yml` files and `graphs/*.yml` files.
Generated workflows (flat `.yml` files produced by `generate-examples-cms-workflow`) are excluded.

---

## graphs/plateau4/quality-check/01-01-common.yml

| Line | Expression | Notes |
|------|-----------|-------|
| 81 | `attributes["status"] == "NOT_WELL_FORMED"` | status always set |
| 82 | `attributes["xmlError"].len()` | only reached if status matches |
| 91 | `attributes["status"] == "INVALID"` | always set |
| 92 | `attributes["xmlError"].len()` | only reached if status matches |
| 647 | `if type(attributes[key]) == "int"` | dynamic key, loop iteration |
| 648 | `total += attributes[key]` | dynamic key, loop iteration |

---

## data-convert/plateau4/01-bldg/workflow.yml

| Line | Expression | Notes |
|------|-----------|-------|
| 70, 83 | `attributes = attributes["cityGmlAttributes"]` | CityGML reader output |
| 76 | `gml_city_code or attributes["cityCode"] or env("cityCode")` | `or` broken if `cityCode` absent |
| 89 | `gml_city_name or attributes["cityName"]` | `or` broken if `cityName` absent |
| 147 | `Url(attributes["path"]).name.split("_")[0]` | `path` from file reader |
| 152 | `attributes["__citygml_feature_type"]` | set by reader |
| 231, 276, 317, 358, 399, 440, 481 | `(attributes["_xmin"] + attributes["_xmax"]) * 0.5` | bounding box, set by prior step |
| 236, 281, 322, 363, 404, 445, 486 | `(attributes["_ymin"] + attributes["_ymax"]) * 0.5` | bounding box, set by prior step |
| 245, 286, 327, 368, 409, 450, 491 | `attributes["__citygml_lod_mask"].bit_length() - 1` | reader attribute |
| 512, 521, 539, 548, 566, 575, 593, 602, 620, 629, 647, 656, 674, 683 | `attributes["baseCityCode"]`, `attributes["city_code"]` | set by AttributeManager |
| 524, 551, 578, 605, 632, 659, 686 | `attributes["cityNameEn"]` | set by AttributeManager |

---

## data-convert/plateau4/02-tran-rwy-trk-squr-wwy/workflow.yml

| Line | Expression | Notes |
|------|-----------|-------|
| 76 | `Url(attributes["path"]).name.split("_")[0]` | file reader |
| 81 | `attributes["__citygml_gml_id"] or attributes["gmlId"]` | `or` broken — both may be absent |
| 86 | `attributes["featureType"] or attributes["gmlName"]` | `or` broken — both may be absent |
| 204, 208–209, 300, 304–305, 416, 420–421 | `attributes["__package"]` | renamed by AttributeManager |
| 294, 410 | `attributes["feature_type"].split(":")[1]` | set by AttributeManager |
| 300, 305 | `attributes["__lod"]` | renamed by AttributeManager |

---

## data-convert/plateau4/03-frn-veg/workflow.yml

| Line | Expression | Notes |
|------|-----------|-------|
| 84 | `attributes["featureType"].split(":")[1]` | may be absent |
| 137 | `Url(attributes["path"]).name.split("_")[0]` | file reader |
| 142 | `attributes["__citygml_gml_id"] or attributes["gmlId"]` | `or` broken — both may be absent |
| 147 | `attributes["featureType"] or attributes["gmlName"]` | `or` broken — both may be absent |
| 267, 272, 524, 529, 769, 774 | `attributes["lod"]` | not renamed to `__lod` here |
| 396, 671, 908 | `attributes["feature_type"].split(":")[1]` | set by AttributeManager |
| 500 | `attributes["veg:height"] * 1.0` | CityGML optional — no fallback |
| 505 | `attributes["veg:trunkDiameter"] * 1.0` | CityGML optional — no fallback |
| 510 | `attributes["veg:crownDiameter"] * 1.0` | CityGML optional — no fallback |

---

## data-convert/plateau4/05-fld/workflow.yml

| Line | Expression | Notes |
|------|-----------|-------|
| 74 | `attributes["featureType"] or attributes["gmlName"]` | `or` broken — both may be absent |
| 80 | `attributes["package"] != "fld"` | plain |
| 82, 84 | `attributes["name"]` in string-in check | file reader attr |
| 184, 190 | `attributes["udxDirs"].replace("/", "_")` | fld-specific attr |
| 192 | `if attributes["__scale"] { ... }` | may be absent — no fallback |
| 207 | `attributes["udxDirs"].split("/")[-1]` | fld-specific attr |
| 208 | `attributes["__scale"]` | may be absent |

---

## data-convert/plateau4/06-area-urf/workflow.yml

| Line | Expression | Notes |
|------|-----------|-------|
| 71 | `Url(attributes["path"]).name.split("_")[0]` | file reader |
| 76 | `attributes["__citygml_gml_id"] or attributes["gmlId"]` | `or` broken — both may be absent |
| 81 | `attributes["featureType"] or attributes["gmlName"]` | `or` broken — both may be absent |
| 139, 197, 206, 213 | `attributes["feature_type"].split(":")[1]` | set by AttributeManager |
| 207, 213 | `attributes["__package"]` | renamed by AttributeManager |
| 215 | `attributes["__lod"]` | renamed by AttributeManager |

---

## data-convert/plateau4/07-brid-tun-cons/workflow.yml

| Line | Expression | Notes |
|------|-----------|-------|
| 144 | `attributes["feature_type"]` | set by AttributeManager |
| 168 | `Url(attributes["path"]).name.split("_")[0]` | file reader |
| 173 | `attributes["__citygml_gml_id"] or attributes["gmlId"]` | `or` broken — both may be absent |
| 178 | `attributes["featureType"] or attributes["gmlName"]` | `or` broken — both may be absent |
| 349, 365, 370–371 | `attributes["__package"]` | renamed by AttributeManager |
| 432, 436–437 | `attributes["__package"]` lod1 writer | renamed |
| 451, 455–456 | `attributes["__package"]` lod2 writer | renamed |
| 470, 474–475 | `attributes["__package"]` lod3 writer | renamed |
| **489, 493–494** | `attributes["package"]` (unrenamed!) | **possible bug — lod4 uses `package` not `__package`** |

---

## data-convert/plateau4/08-ubld/workflow.yml

| Line | Expression | Notes |
|------|-----------|-------|
| 72 | `Url(attributes["path"]).name.split("_")[0]` | file reader |
| 77 | `attributes["__citygml_gml_id"] or attributes["gmlId"]` | `or` broken — both may be absent |
| 82 | `attributes["featureType"] or attributes["gmlName"]` | `or` broken — both may be absent |
| 155, 159–160 | `attributes["package"]`, `attributes["lod"]` | not renamed here — different convention |

---

## data-convert/plateau4/09-unf/workflow.yml

| Line | Expression | Notes |
|------|-----------|-------|
| 72 | `attributes["featureType"].split(":")[1]` | may be absent |
| 136 | `Url(attributes["path"]).name.split("_")[0]` | file reader |
| 141 | `attributes["__citygml_gml_id"] or attributes["gmlId"]` | `or` broken — both may be absent |
| 146 | `attributes["featureType"] or attributes["gmlName"]` | `or` broken — both may be absent |
| 234–241 | `attributes["feature_type"]`, `attributes["package"]`, `attributes["lod"]` | set before use |
| 320–332 | `attributes["feature_type"]`, `attributes["__package"]` | set before use |

---

## data-convert/plateau4/10-wtr/workflow.yml

| Line | Expression | Notes |
|------|-----------|-------|
| 72 | `Url(attributes["path"]).name.split("_")[0]` | file reader |
| 77 | `attributes["__citygml_gml_id"] or attributes["gmlId"]` | `or` broken — both may be absent |
| 82 | `attributes["featureType"] or attributes["gmlName"]` | `or` broken — both may be absent |
| 161, 167, 171–172, 189, 193–194 | `attributes["feature_type"]`, `attributes["package"]` | set before use |

---

## data-convert/plateau4/11-gen/workflow.yml

| Line | Expression | Notes |
|------|-----------|-------|
| 72 | `Url(attributes["path"]).name.split("_")[0]` | file reader |
| 77 | `attributes["__citygml_gml_id"] or attributes["gmlId"]` | `or` broken — both may be absent |
| 82 | `attributes["featureType"] or attributes["gmlName"]` | `or` broken — both may be absent |
| 104 | `attributes["gml:name_code"]` | CityGML optional — no fallback |
| 109 | `attributes["gml:name"]` | CityGML optional — no fallback |
| 184, 190, 193 | `attributes["__gml_name_code"]`, `attributes["__lod"]` | set by AttributeManager |

---

## examples/citygml-roundtrip/workflow.yml

| Line | Expression | Notes |
|------|-----------|-------|
| 59, 91 | `attributes["__citygml_gml_id"]` | reader attribute |

---

## quality-check/plateau4/06-fld/workflow.yml

| Line | Expression | Notes |
|------|-----------|-------|
| 115 | `attributes["name"]` | file reader attr |

---

## quality-check/plateau4/10-cons/workflow.yml

| Line | Expression | Notes |
|------|-----------|-------|
| 192 | `attributes["__citygml_feature_type"] or ""` | `or` broken |
| 202 | `attributes["parentId"] or attributes["gmlId"] or ""` | `or` broken — all may be absent |

---

## quality-check/plateau4/12-unf/workflow.yml

| Line | Expression | Notes |
|------|-----------|-------|
| 183 | `attributes["__citygml_feature_type"] or ""` | `or` broken |

---

## solar-radiation/workflow.yml

| Line | Expression | Notes |
|------|-----------|-------|
| 355, 360 | `attributes["solarTime"].split(...)` | internal pipeline attr |
| 366, 373 | `attributes["solarTime"].split(...)` | internal pipeline attr |
| 547–552 | `attributes["solarDirectionX/Y/Z"]`, `attributes["normalX/Y/Z"]` | internal |
| 569–604 | `attributes["month"]`, `attributes["solarAltitude"]`, `attributes["cosTheta"]`, `attributes["cellArea"]`, `attributes["slope"]`, `attributes["groundReflectivity"]` | internal |
| 666 | `attributes["sunnyRatio"] or 0.0` | `or` broken |
| 667 | `attributes["radiationSunnySum"] or 0.0` | `or` broken |
| 668 | `attributes["radiationCloudySum"] or 0.0` | `or` broken |
| 743 | `attributes["totalSolarRadiation"]` | internal |
| 748 | `attributes["totalSolarPower"]` | internal |
| 796 | `attributes["totalSolarRadiation"] or 0.0` | `or` broken |
| 821, 830, 841 | `attributes["normalizedRadiation"] or 0.0` | `or` broken |
| 936, 941, 1320, 1325, 1374, 1379, 1428, 1433 | `attributes["_xmin/_xmax/_ymin/_ymax"]` | bounding box, set by prior step |
| 996, 1001, 1006 | `attributes["solarDirectionX/Y/Z"]`, `attributes["cosTheta"]`, `attributes["normalX/Y/Z"]` | internal |
| 1587 | `attributes["_relief_index"] or attributes["_split_index"] or 0` | `or` broken |
| 1618, 1631, 1644 | `attributes["azimuth"] or 0.0` | `or` broken |
| 1699, 1743 | `attributes["_grid_col"] or 0`, `attributes["_grid_row"] or 0` | `or` broken |

---

## Migration Priority Summary

### Definitely need `.get()` — key may be absent, fallback patterns broken
- `attributes["__citygml_gml_id"] or attributes["gmlId"]` (all plateau4 data-convert)
- `attributes["featureType"] or attributes["gmlName"]` (all plateau4 data-convert)
- `attributes["featureType"].split(":")[1]` with no fallback (03-frn-veg:84, 09-unf:72)
- `attributes["cityCode"] or env(...)` in 01-bldg
- `attributes["veg:height/trunkDiameter/crownDiameter"]` in 03-frn-veg (no fallback at all)
- `attributes["__scale"]` in 05-fld (no fallback)
- `attributes["__citygml_feature_type"] or ""` (qc 10-cons, 12-unf)
- `attributes["parentId"] or attributes["gmlId"] or ""` (qc 10-cons)
- `attributes["gml:name_code"]`, `attributes["gml:name"]` in 11-gen (no fallback)
- Solar radiation `or 0.0` / `or 0` patterns (lines 666–668, 796, 821, 830, 841, 1587, 1618, 1631, 1644, 1699, 1743)

### Probably safe to keep — key guaranteed by explicit AttributeManager rename upstream
- `attributes["__package"]` (renamed from `package`)
- `attributes["__lod"]` (renamed from `lod`)
- `attributes["feature_type"]` (renamed from `featureType`)
- `attributes["city_code"]` (renamed from `cityCode`)
- `attributes["_xmin/_xmax/_ymin/_ymax"]` (set by bounding box step)
- `attributes["__citygml_lod_mask"]` (set by reader)
- `attributes["__gml_name_code"]` (renamed in 11-gen)

### Possible bug (investigate separately)
- 07-brid-tun-cons line 489: lod4 writer uses `attributes["package"]` while lod1–3 writers use `attributes["__package"]`
