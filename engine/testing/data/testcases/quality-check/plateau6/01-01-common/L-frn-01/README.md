# quality-check plateau6 01-01-common / L-frn-01

L-frn-01 checks the geometry type held by an LOD property (e.g. an LOD3
geometry must be `gml:MultiSurface` or `gml:Solid`, not `gml:LineString`).

**No plateau6 fixture is added yet** — the plateau4 test cannot be ported as-is
because of the CityGML 3.0 geometry model change:

- **CityGML 2.0**: LOD geometry lives in a generic `lod{N}Geometry` property of
  type `gml:GeometryPropertyType`, so any geometry is schema-valid. The plateau4
  test puts a `gml:LineString` in `bldg:lod3Geometry`: XSD passes, only L-frn-01
  flags it. That is the point of the check.
- **CityGML 3.0**: the generic property is gone; LOD geometry uses type-explicit
  properties (`lod2MultiSurface`, `lod3Solid`, ...). A wrong geometry is rejected
  by XSD itself and surfaces as an **L02 (schema validation) error**, so a
  plateau4-style L-frn-01 violation cannot be reproduced on these properties.

L-frn-01 stays meaningful only on the few i-UR 4.0 (`urc:`) properties that keep
`gml:GeometryPropertyType`: `urc:DmGeometricAttribute/urc:lod0Geometry` and
`urc:DmAnnotation/urc:lod0anchorPoint`.

Porting therefore needs a real implementation change, not just a `PlateauProfile`
namespace swap: the shared validator
(`runtime/action-plateau-processor/src/common/domain_of_definition_validator.rs`)
still scans CityGML 2.0 `lod{N}Geometry` names and `gen:GenericCityObject` /
`uro:Dm*` parents, none of which exist in CityGML 3.0 / i-UR 4.0.
