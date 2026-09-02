# PLATEAU quality-check error geometry

Reference for the GeoJSON error-detail outputs of the PLATEAU quality-check
workflows: what each layer holds, what its properties mean, what coordinate
reference system it is written in, and the two wiring patterns that get the
right geometry to the writer.

The reference implementation is
[`runtime/examples/fixture/workflow/quality-check/plateau6/02-bldg/building_checks.yml`](/runtime/examples/fixture/workflow/quality-check/plateau6/02-bldg/building_checks.yml).

---

## 1. What the layers are for

Each quality check writes two kinds of output:

- a **CSV** with one row per error, attributes only, aggregated into
  `02_建築物_検査結果一覧.csv` and `summary_bldg.json`
- a **GeoJSON** under `error_geometry/` with the same errors carrying their
  geometry, so the error can be found on a map

This mirrors the original workflows these checks were ported from, which
separate the attribute-only Excel sheets from the coordinate-bearing Shapefiles.
One check category maps to one GeoJSON file, named after the CSV with the
extension changed, so a row and a feature can be matched up by eye.

A check that reports no position at all — a duplicate ID, a malformed attribute
value, a broken xlink — writes no GeoJSON. There is nothing to draw.

## 2. Two granularities per check

A geometry check reports at two granularities, and they go to separate files:

- **the offending geometry itself** — the face, or the solid, that failed. From
  `Geometry Validator`'s `failed` port.
- **the spot the check flagged** — one feature per position, its geometry
  replaced by that position. From `Geometry Validator`'s `issue-locations` port,
  which also sets `validationCheck` to the single check that produced it.

They are separate layers because a position is a point or a line while the
offending geometry is a face or a mesh, and a GIS cannot give one layer holding
both a sensible symbology. The position layer's name is the face layer's name
plus `位置`.

## 3. Common properties

Property names are lowerCamelCase. CSV column names are unchanged — the two
outputs do not share a naming convention.

| Property | Required | Content |
| --- | --- | --- |
| `check` | yes | Check category, matching the check part of the CSV file name (e.g. `LOD0面の交差`) |
| `issue` | yes | The individual error, same value as the CSV's `issue` column |
| `filename` | yes | Input GML file name |
| `featureType` | yes | Type of the feature in error; for a boundary surface, the surface's own type (e.g. `con:WallSurface`) |
| `gmlId` | yes | `gml:id` of the feature in error |
| `rootGmlId` | no | `gml:id` of the root feature, the key for grouping errors per building |
| `parentGmlId` | no | `gml:id` of the immediate parent feature |
| `lod` | no | LOD the error was found at, as a number |
| `relatedGmlId` | no | The other party's `gml:id` for a pairwise check (intersection, connectivity) |
| `pairId` | no | Groups the records belonging to one pair of a pairwise check |

`rootGmlId` and `parentGmlId` come straight from the CityGML 3 reader's
`__citygml_root_gml_id` and `__citygml_parent_gml_id`; an `Attribute Mapper`
copies them across. A check-specific property may be added on top of these.

An `Attribute Mapper` keeps only what it maps, so putting one immediately before
each writer is also what stops internal working attributes from reaching the
file.

## 4. Coordinate reference system

**Coordinates are written in the projected CRS the checks ran in — the workflow's
`prcs` variable — and are not projected back.** `Feature GeoJSON Writer` with
`writeCrs: true` declares it in a GeoJSON 2008 `crs` member, taking the EPSG code
from the geometry itself.

This follows the original workflows, which project every building into a
plane-rectangular CRS before checking and write the error Shapefiles in that same
CRS, leaving the CRS declaration to the sidecar `.prj`.

The projection is not a presentation choice: `Geometry Validator` skips the
planarity and 3D surface self-intersection checks on an angular-unit
(geographic) CRS and says so in a warning, so `Coordinate Frame Reprojector`
has to come before the checks for them to run at all.

RFC 7946 dropped the `crs` member and fixes GeoJSON to WGS84 longitude/latitude,
so this output is a non-standard extension. The reader it is written for is the
desktop GIS user, the same audience as the original Shapefile output. A
back-projected variant is worth adding when a web viewer needs one.

## 5. Getting the geometry to the writer

### Pattern 1: use the validator's ports directly

`Geometry Validator` emits the offending geometry on `failed` and each flagged
position on `issue-locations`, so both layers need only an `Attribute Mapper`
and a `Feature GeoJSON Writer`.

### Pattern 2: put the geometry aside and restore it

A check that has to transform the geometry to reach its answer — flattening a
face to read its 2D winding, replacing a solid with its footprint to find
candidate overlaps — must not report the transformed geometry. Put the geometry
aside with `Geometry Extractor` into an attribute before the transformation and
restore it with `Geometry Replacer` after the verdict.

`Geometry Extractor` writes a compressed string, so the stashed geometry
survives `Area On Area Overlayer`'s `listAttribute` and `List Indexer`. That is
how the participants of an intersection are recovered: stash before the
overlay, then index the list and restore each side from its own suffixed copy.

## 6. GeoJSON cannot hold a solid

The GeoJSON conversion rejects `Solid`, `Csg` and `PointCloud`
([`runtime/types/src/conversion/geojson_next.rs`](/runtime/types/src/conversion/geojson_next.rs)),
and the writer only warns about a feature it cannot convert
([`runtime/action-sink/src/file/geojson.rs`](/runtime/action-sink/src/file/geojson.rs)).
A solid layer wired without a conversion therefore produces an **empty file and
a successful run** — the easiest failure to miss.

Put a `Geometry Coercer` with `targetType: polygon` in front of the writer: it
decomposes a solid into the faces it is built from, which the writer emits as a
MultiPolygon.

## 7. Checklist for a new layer

- Tap the port that carries the geometry you mean to show, not a downstream node
  that has already transformed it.
- `Attribute Mapper` immediately before the writer, mapping the §3 properties.
- `writeCrs: true` on the writer.
- Route the writer's output feature into the workflow's `Output Router` so the
  file lands in the result zip.
- Open the real output. An empty file is a passing run.
- Add the file to `expectedOutput.expectedFile` of a test case that produces it,
  with an expected GeoJSON under the case's `error_geometry/`. The test harness
  strips the per-run feature `id`, rounds coordinates, and sorts features, so
  the expected file needs neither ids nor a fixed feature order.
