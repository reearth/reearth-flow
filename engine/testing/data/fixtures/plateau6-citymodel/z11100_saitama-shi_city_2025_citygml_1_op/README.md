# Object list changes (CityGML 3.0 adaptation)

`11100_city_2025_objectlist_op.xlsx` is Saitama City's PLATEAU5 (i-UR 3.x) acquisition item
list. In the `A.3.1_取得項目一覧` sheet, the `tran:Road` block acquired/required i-UR 3.x /
CityGML 2.0 structured attributes that were relocated or restructured in CityGML 3.0 / i-UR 4.0
and no longer exist directly on a valid 3.0 `tran:Road`. Left as-is they cause false positives
in the common checks C05 (missing required) and C06 (missing acquisition target).

Only the `tran:Road` block was adjusted, by clearing the `作成対象` column (column I).
Clearing column I removes the row from the checks; attribute names, xpaths, and the `必須`
indicator (column M) were not rewritten.

## Kept in `作成対象` (exist on a 3.0 `tran:Road` and present in the test GML)

- `gml:name`
- `core:creationDate`
- `tran:class`
- `tran:function`

## Removed from `作成対象` (20 rows; relocated/restructured in 3.0/4.0)

- Surface geometry (moved under `tran:trafficSpace > tran:TrafficSpace > core:boundary > tran:TrafficArea`)
  - `tran:lod1MultiSurface`, `tran:lod2MultiSurface`, `tran:lod3MultiSurface`, `tran:trafficArea`, `tran:auxiliaryTrafficArea`
- Data quality (moved to `core:adeOfAbstractCityObject > urc:DataQualityAttribute`)
  - `uro:tranDataQualityAttribute/uro:DataQualityAttribute` and descendants `uro:geometrySrcDescLod1/2/3`, `uro:thematicSrcDesc`, `uro:lodType`
  - `uro:publicSurveyDataQualityAttribute/uro:PublicSurveyDataQualityAttribute` and descendants `uro:srcScaleLod1/2/3`, `uro:publicSurveySrcDescLod1/2/3`
- Road structure
  - `uro:roadStructureAttribute/uro:RoadStructureAttribute`, `uro:sectionType`

Feature types other than `tran:Road`, packages other than `tran`, and other sheets are untouched.
