# Notes on the unit test for quality-check plateau6 01-01-common / L06

L06 validates the coordinate counts of `gml:pos` / `gml:posList` elements.

This check is implemented in the shared `common` layer
(`runtime/action-plateau-processor/src/common/domain_of_definition_validator.rs`),
which is reused by both plateau4 (CityGML 2.0) and plateau6 (CityGML 3.0) via
`PlateauProfile` namespace substitution. The `gml:pos` / `gml:posList`
coordinate representation in GML 3.2 (CityGML 3.0) is identical to GML 3.1.1
(CityGML 2.0), so a plateau6 fixture would exercise exactly the same code path
on equivalent input as the existing plateau4 test.

The plateau4 test (`plateau4/01-01-common/L06`) already covers this logic, so we
skip the plateau6 unit test to avoid duplicating fixtures with no added coverage.
