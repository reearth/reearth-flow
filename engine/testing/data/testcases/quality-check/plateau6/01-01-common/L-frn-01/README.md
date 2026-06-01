# Notes on the unit test for quality-check plateau6 01-01-common / L-frn-01

L-frn-01 validates the geometric object type carried by `lod{0-3}Geometry`
properties (e.g. an LOD3 geometry must be a `gml:MultiSurface` or `gml:Solid`,
not a `gml:LineString`).

Unlike the other common checks in this directory, this logic does **not** carry
over unchanged from CityGML 2.0 to 3.0, so the plateau4 fixture cannot simply be
ported. The reason is the geometry model change in CityGML 3.0:

- In CityGML 2.0 the LOD geometry is held in a generic `lod{N}Geometry`
  property whose type is `gml:GeometryPropertyType` (any geometry is
  schema-valid). The plateau4 test (`plateau4/01-01-common/L-frn-01`) exploits
  this by placing a `gml:LineString` inside `bldg:lod3Geometry`: the document
  still passes XSD validation, and only the L-frn-01 rule flags it. That is the
  whole point of the check — catch a geometry that is schema-valid but
  semantically wrong for its LOD.
- In CityGML 3.0 the generic `lod{N}Geometry` property is gone. LOD geometry is
  expressed through type-explicit properties inherited from `core:AbstractSpace`
  (`lod0MultiSurface`, `lod0Point`, `lod0MultiCurve`, `lod1Solid`,
  `lod2MultiSurface`, `lod2Solid`, `lod3MultiSurface`, `lod3Solid`, ...). The
  allowed geometry type is now baked into the schema type of the property, so a
  wrong geometry (e.g. a `gml:LineString` inside `core:lod3MultiSurface`) is
  rejected by XSD validation itself and surfaces as an **L02 (schema
  validation) error**, never reaching a separate L-frn-01 rule. A
  plateau4-style L-frn-01 violation therefore cannot be reproduced on these
  properties in CityGML 3.0.

The only place where the L-frn-01 rule remains independently meaningful in
CityGML 3.0 is the small set of properties that keep the generic
`gml:GeometryPropertyType` in i-UR 4.0 (`urc:` namespace), namely
`urc:DmGeometricAttribute/urc:lod0Geometry` and
`urc:DmAnnotation/urc:lod0anchorPoint`. There a schema-valid but
semantically-wrong geometry can still be authored, exactly as in plateau4.

The existing shared `common` implementation
(`runtime/action-plateau-processor/src/common/domain_of_definition_validator.rs`)
still scans for CityGML 2.0 `lod{N}Geometry` element names and the
`gen:GenericCityObject` / `uro:DmGeometricAttribute` / `uro:DmAnnotation`
parents, none of which exist in CityGML 3.0 / i-UR 4.0. Porting L-frn-01 to
plateau6 thus requires a real implementation change (target the `urc:` generic
geometry properties), not just `PlateauProfile` namespace substitution.

Until that implementation decision is made, no plateau6 fixture is added here;
this note records why the plateau4 test cannot be ported as-is.
