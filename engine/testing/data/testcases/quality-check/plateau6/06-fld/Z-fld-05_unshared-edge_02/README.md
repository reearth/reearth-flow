# Notes on the unit test for quality-check plateau6 06-fld / Z-fld-05_unshared-edge_02

The geometry is a copy of `plateau4/06-fld/Z-fld-05_unshared-edge_04`: real
PLATEAU data for ajigasawa-machi, mesh 61400178 under
`fld/pref/nakamuragawa_nakamuragawa`, at both flood scales. The plateau6 numbers
run on their own, so this case is `_02` here while its source is `_04` there.

Only the encoding was rewritten, from CityGML 2.0 + i-UR 3.2 to CityGML 3.0 +
i-UR 4.0. Coordinates, `gml:id` values, coded values and the order the triangles
appear in are carried over unchanged; the surfaces move from
`wtr:lod1MultiSurface` on the WaterBody to `core:boundary` / `wtr:WaterSurface`,
the i-UR attribute groups move under `core:adeOfAbstractCityObject`, and
`core:creationDate` takes the `xs:dateTime` form that 3.0 requires.

What it adds over `Z-fld-05_unshared-edge_01`, whose fixture is five hand-written
triangles in one feature in one file:

- two files in one folder, so edges are matched across files
- the same mesh at both flood scales, which have to stay separate groups
- five `wtr:WaterBody` features per file, so edges are matched across features
- a real outline: 97 of the 167 edges are used once, and every one of them has to
  be recognised as outline and dropped for the run to report nothing

The last two points are what makes the case worth keeping. Dropping `fldScale`
from the `groupBy` of both `PLATEAU6.UnsharedEdgeExtractor` and
`DissolverOutline` makes the run report four unshared edges and this test fail.
