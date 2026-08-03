//! GeoJSON <-> `Feature` conversion for the new-geometry world. Builds
//! `reearth_flow_geometry::Geometry` (per-leaf `CoordinateFrame`) rather than the
//! old `GeometryValue` wrapper.
//!
//! Both directions live here so the axis-order convention stays one invariant:
//! GeoJSON positions are always `(easting/longitude, northing/latitude[, height])`,
//! while a CRS frame stores its axes in the order the CRS authority declares, so
//! reading swaps the horizontal pair and writing swaps it back.

use std::sync::Arc;

use reearth_flow_geometry::types::conversion::{is_2d_geojson_value, is_3d_geojson_value};
use reearth_flow_geometry::{
    collection::{Collection2D, Collection3D},
    coordinate::{CoordinateFrame, EpsgCode},
    line_string::{LineString2D, LineString3D},
    ops::Split,
    point::{Point2D, Point3D},
    polygon::{Polygon2D, Polygon3D},
    Euclidean2DGeometry, Euclidean3DGeometry, Geometry, GeometryCollection,
};

use crate::{
    error::{Error, Result},
    Attribute, Attributes, Feature,
};

// WGS84 geographic CRS codes, defined here so this new-geometry module carries no
// dependency on nusamai-projection (which is slated for removal after the migration).
const EPSG_WGS84_GEOGRAPHIC_2D: u16 = 4326;
const EPSG_WGS84_GEOGRAPHIC_3D: u16 = 4979;

impl TryFrom<geojson::Feature> for Feature {
    type Error = Error;

    fn try_from(geom: geojson::Feature) -> Result<Self> {
        let attributes = geom
            .properties
            .as_ref()
            .map(geojson_object_to_attributes)
            .unwrap_or_default();
        let geometry = match geom.geometry {
            Some(g) => geojson_value_to_geometry(g.value)?,
            None => Geometry::None,
        };
        Ok(Feature {
            id: geom.id.map_or_else(uuid::Uuid::new_v4, geojson_id_to_uuid),
            attributes: Arc::new(attributes),
            geometry: Arc::new(geometry),
        })
    }
}

/// A GeoJSON string id that is a valid UUID is preserved; anything else gets a fresh UUID.
fn geojson_id_to_uuid(id: geojson::feature::Id) -> uuid::Uuid {
    match id {
        geojson::feature::Id::String(v) => {
            uuid::Uuid::parse_str(&v).unwrap_or_else(|_| uuid::Uuid::new_v4())
        }
        geojson::feature::Id::Number(_) => uuid::Uuid::new_v4(),
    }
}

fn geojson_object_to_attributes(obj: &geojson::JsonObject) -> Attributes {
    obj.iter()
        .map(|(k, v)| (Attribute::new(k), v.clone().into()))
        .collect()
}

/// WGS84 geographic frame for 2D coordinates, stored (lat, lon) per EPSG:4326.
fn wgs84_2d() -> CoordinateFrame {
    CoordinateFrame::Crs(EpsgCode::new(EPSG_WGS84_GEOGRAPHIC_2D))
}

/// WGS84 geographic frame for 3D coordinates, stored (lat, lon, height) per EPSG:4979.
fn wgs84_3d() -> CoordinateFrame {
    CoordinateFrame::Crs(EpsgCode::new(EPSG_WGS84_GEOGRAPHIC_3D))
}

fn geojson_value_to_geometry(value: geojson::Value) -> Result<Geometry> {
    match value {
        // A heterogeneous collection: each member converts on its own, so members
        // may differ in dimension / coordinate frame.
        geojson::Value::GeometryCollection(geometries) => {
            let members = geometries
                .into_iter()
                .map(|g| geojson_value_to_geometry(g.value))
                .collect::<Result<Vec<_>>>()?;
            Ok(Geometry::GeometryCollection(GeometryCollection::new(
                members,
            )))
        }
        // A single geometry's coordinates must be uniformly 2D or 3D; mixed or
        // degenerate coordinates are rejected rather than indexed into blindly.
        value if is_2d_geojson_value(&value) => Ok(Geometry::Euclidean2D(value_to_2d(value)?)),
        value if is_3d_geojson_value(&value) => Ok(Geometry::Euclidean3D(value_to_3d(value)?)),
        _ => Err(mixed_dimensions()),
    }
}

fn value_to_2d(value: geojson::Value) -> Result<Euclidean2DGeometry> {
    match value {
        geojson::Value::Point(p) => Ok(Euclidean2DGeometry::Point(point_2d(&p))),
        geojson::Value::MultiPoint(ps) => Ok(collection_2d(
            ps.iter().map(|p| Euclidean2DGeometry::Point(point_2d(p))),
        )),
        geojson::Value::LineString(coords) => {
            Ok(Euclidean2DGeometry::LineString(line_string_2d(&coords)))
        }
        geojson::Value::MultiLineString(lines) => {
            Ok(collection_2d(lines.iter().map(|l| {
                Euclidean2DGeometry::LineString(line_string_2d(l))
            })))
        }
        geojson::Value::Polygon(rings) => {
            Ok(Euclidean2DGeometry::Polygon(Box::new(polygon_2d(&rings))))
        }
        geojson::Value::MultiPolygon(polys) => {
            Ok(collection_2d(polys.iter().map(|rings| {
                Euclidean2DGeometry::Polygon(Box::new(polygon_2d(rings)))
            })))
        }
        _ => Err(mixed_dimensions()),
    }
}

fn value_to_3d(value: geojson::Value) -> Result<Euclidean3DGeometry> {
    match value {
        geojson::Value::Point(p) => Ok(Euclidean3DGeometry::Point(point_3d(&p))),
        geojson::Value::MultiPoint(ps) => Ok(collection_3d(
            ps.iter().map(|p| Euclidean3DGeometry::Point(point_3d(p))),
        )),
        geojson::Value::LineString(coords) => {
            Ok(Euclidean3DGeometry::LineString(line_string_3d(&coords)))
        }
        geojson::Value::MultiLineString(lines) => {
            Ok(collection_3d(lines.iter().map(|l| {
                Euclidean3DGeometry::LineString(line_string_3d(l))
            })))
        }
        geojson::Value::Polygon(rings) => {
            Ok(Euclidean3DGeometry::Polygon(Box::new(polygon_3d(&rings))))
        }
        geojson::Value::MultiPolygon(polys) => {
            Ok(collection_3d(polys.iter().map(|rings| {
                Euclidean3DGeometry::Polygon(Box::new(polygon_3d(rings)))
            })))
        }
        _ => Err(mixed_dimensions()),
    }
}

fn mixed_dimensions() -> Error {
    Error::unsupported_feature(
        "GeoJSON geometry has mixed or unsupported coordinate dimensions \
         (every coordinate must be uniformly 2D or 3D)",
    )
}

fn collection_2d(members: impl IntoIterator<Item = Euclidean2DGeometry>) -> Euclidean2DGeometry {
    Euclidean2DGeometry::Collection(Collection2D::new(members))
}

fn collection_3d(members: impl IntoIterator<Item = Euclidean3DGeometry>) -> Euclidean3DGeometry {
    Euclidean3DGeometry::Collection(Collection3D::new(members))
}

// GeoJSON coordinates are (lon, lat[, height]) per RFC 7946, but the WGS84 frames
// they are tagged with declared (lat, lon[, height]) axis order, so the horizontal
// pair is swapped on read. The swap also flips ring winding, which the frame's
// orientation sign accounts for, keeping the canonical orientation intact.

fn point_2d(p: &[f64]) -> Point2D {
    Point2D::new(wgs84_2d(), [p[1], p[0]])
}

fn point_3d(p: &[f64]) -> Point3D {
    Point3D::new(wgs84_3d(), [p[1], p[0], p[2]])
}

fn line_string_2d(coords: &[Vec<f64>]) -> LineString2D {
    LineString2D::from_coords(wgs84_2d(), coords.iter().map(|c| [c[1], c[0]]))
}

fn line_string_3d(coords: &[Vec<f64>]) -> LineString3D {
    LineString3D::from_coords(wgs84_3d(), coords.iter().map(|c| [c[1], c[0], c[2]]))
}

fn polygon_2d(rings: &[Vec<Vec<f64>>]) -> Polygon2D {
    let mut rings = rings
        .iter()
        .map(|r| r.iter().map(|c| [c[1], c[0]]).collect::<Vec<_>>());
    let exterior = rings.next().unwrap_or_default();
    Polygon2D::from_rings(wgs84_2d(), exterior, rings)
}

fn polygon_3d(rings: &[Vec<Vec<f64>>]) -> Polygon3D {
    let mut rings = rings
        .iter()
        .map(|r| r.iter().map(|c| [c[1], c[0], c[2]]).collect::<Vec<_>>());
    let exterior = rings.next().unwrap_or_default();
    Polygon3D::from_rings(wgs84_3d(), exterior, rings)
}

// ---------------------------------------------------------------------------
// Feature -> GeoJSON
// ---------------------------------------------------------------------------

impl TryFrom<Feature> for Vec<geojson::Feature> {
    type Error = Error;

    fn try_from(feature: Feature) -> Result<Self> {
        let properties = attributes_to_geojson_object(&feature.attributes);
        let features = match &*feature.geometry {
            // A feature carrying attributes but no geometry still produces a row.
            Geometry::None => vec![geojson_feature(feature.id, None, properties)],
            // A cross-dimensional, cross-frame collection has no single GeoJSON
            // geometry, so each member becomes a feature of its own — as the old
            // CityGML geometry did. The members share the feature's properties;
            // per-member attributes stay on the member.
            Geometry::GeometryCollection(collection) => collection
                .members()
                .iter()
                .filter_map(geometry_to_value)
                .map(|value| geojson_feature(uuid::Uuid::new_v4(), Some(value), properties.clone()))
                .collect(),
            geometry => {
                let value = geometry_to_value(geometry).ok_or_else(|| {
                    Error::unsupported_feature("geometry has no GeoJSON counterpart")
                })?;
                vec![geojson_feature(feature.id, Some(value), properties)]
            }
        };
        Ok(features)
    }
}

fn geojson_feature(
    id: uuid::Uuid,
    value: Option<geojson::Value>,
    properties: geojson::JsonObject,
) -> geojson::Feature {
    geojson::Feature {
        bbox: None,
        geometry: value.map(geojson::Geometry::new),
        id: Some(geojson::feature::Id::String(id.to_string())),
        properties: Some(properties),
        foreign_members: None,
    }
}

fn attributes_to_geojson_object(attributes: &Attributes) -> geojson::JsonObject {
    attributes
        .iter()
        .map(|(k, v)| (k.to_string(), v.clone().into()))
        .collect()
}

/// The GeoJSON counterpart of `geometry`, or `None` when it has none.
///
/// A leaf GeoJSON cannot express (`Solid`, `Csg`, `PointCloud`) is dropped where
/// it appears rather than failing the geometry around it, so one such member does
/// not discard its siblings.
fn geometry_to_value(geometry: &Geometry) -> Option<geojson::Value> {
    match geometry {
        Geometry::None => None,
        Geometry::Euclidean2D(g) => value_2d(g),
        Geometry::Euclidean3D(g) => value_3d(g),
        // Nested: only the outermost collection expands into separate features.
        Geometry::GeometryCollection(c) => Some(geometry_collection(
            c.members().iter().filter_map(geometry_to_value).collect(),
        )),
    }
}

fn value_2d(geometry: &Euclidean2DGeometry) -> Option<geojson::Value> {
    match geometry {
        Euclidean2DGeometry::Point(p) => Some(geojson::Value::Point(xy(
            swaps_axes(p.frame()),
            p.position(),
        ))),
        Euclidean2DGeometry::LineString(l) => {
            let swap = swaps_axes(l.frame());
            Some(geojson::Value::LineString(
                l.coords().iter().map(|&c| xy(swap, c)).collect(),
            ))
        }
        Euclidean2DGeometry::Polygon(p) => {
            let swap = swaps_axes(p.frame());
            Some(geojson::Value::Polygon(
                std::iter::once(p.exterior())
                    .chain(p.interiors())
                    .map(|ring| closed_ring(ring.iter().map(|&c| xy(swap, c))))
                    .collect(),
            ))
        }
        Euclidean2DGeometry::PolygonMesh(m) => faces_to_multi_polygon((**m).clone()),
        Euclidean2DGeometry::TriangularMesh(m) => faces_to_multi_polygon((**m).clone()),
        Euclidean2DGeometry::Collection(c) => Some(combine(
            c.members().iter().filter_map(value_2d).collect(),
            one_frame(c.members(), visit_frames_2d),
        )),
    }
}

fn value_3d(geometry: &Euclidean3DGeometry) -> Option<geojson::Value> {
    match geometry {
        Euclidean3DGeometry::Point(p) => Some(geojson::Value::Point(xyz(
            swaps_axes(p.frame()),
            p.position(),
        ))),
        Euclidean3DGeometry::LineString(l) => {
            let swap = swaps_axes(l.frame());
            Some(geojson::Value::LineString(
                l.coords().iter().map(|&c| xyz(swap, c)).collect(),
            ))
        }
        Euclidean3DGeometry::Polygon(p) => {
            let swap = swaps_axes(p.frame());
            Some(geojson::Value::Polygon(
                std::iter::once(p.exterior())
                    .chain(p.interiors())
                    .map(|ring| closed_ring(ring.iter().map(|&c| xyz(swap, c))))
                    .collect(),
            ))
        }
        Euclidean3DGeometry::PolygonMesh(m) => faces_to_multi_polygon((**m).clone()),
        Euclidean3DGeometry::TriangularMesh(m) => faces_to_multi_polygon((**m).clone()),
        Euclidean3DGeometry::Collection(c) => Some(combine(
            c.members().iter().filter_map(value_3d).collect(),
            one_frame(c.members(), visit_frames_3d),
        )),
        // A volume has no GeoJSON counterpart, and neither has the boolean tree
        // built from volumes. A point cloud has one, but expanding it would emit
        // a position per sample.
        Euclidean3DGeometry::Solid(_) => unsupported_leaf("Solid"),
        Euclidean3DGeometry::Csg(_) => unsupported_leaf("Csg"),
        Euclidean3DGeometry::PointCloud(_) => unsupported_leaf("PointCloud"),
    }
}

fn unsupported_leaf(kind: &'static str) -> Option<geojson::Value> {
    tracing::warn!(
        geometry = kind,
        "geometry has no GeoJSON counterpart; omitting it"
    );
    None
}

/// One GeoJSON polygon per mesh face, via the public `Split` op. `Split` takes
/// `&mut self`, hence the mesh is passed by value; splitting a mesh reads it
/// rather than emptying it.
fn faces_to_multi_polygon(mut mesh: impl Split) -> Option<geojson::Value> {
    let mut polygons = Vec::new();
    mesh.split(&mut |face, _| {
        if let Some(geojson::Value::Polygon(rings)) = geometry_to_value(&face) {
            polygons.push(rings);
        }
    })
    .ok()?;
    Some(geojson::Value::MultiPolygon(polygons))
}

/// Present a collection's members as one GeoJSON geometry: a `Multi*` when they
/// are all of one kind and share a coordinate frame, a `GeometryCollection`
/// otherwise.
///
/// `Collection2D` / `Collection3D` are the new geometry's `Multi*`, so folding is
/// the inverse of the `MultiPolygon -> Collection` mapping on the read side.
/// Folding members that differ in frame would put coordinates from different
/// reference systems in one geometry, which no single `crs` member can describe.
fn combine(values: Vec<geojson::Value>, uniform_frame: bool) -> geojson::Value {
    if !uniform_frame {
        return geometry_collection(values);
    }
    match fold_into_multi(values) {
        Ok(folded) => folded,
        Err(values) => geometry_collection(values),
    }
}

fn geometry_collection(values: Vec<geojson::Value>) -> geojson::Value {
    geojson::Value::GeometryCollection(values.into_iter().map(geojson::Geometry::new).collect())
}

/// Fold values that are all points, all curves, or all areas into the matching
/// `Multi*`, handing them back untouched when they are of more than one kind.
/// A member that is already a `Multi*` is flattened into the result.
fn fold_into_multi(values: Vec<geojson::Value>) -> Result<geojson::Value, Vec<geojson::Value>> {
    let Some(kind) = values.first().and_then(value_kind) else {
        return Err(values);
    };
    if !values.iter().all(|v| value_kind(v) == Some(kind)) {
        return Err(values);
    }
    Ok(match kind {
        ValueKind::Point => geojson::Value::MultiPoint(
            values
                .into_iter()
                .flat_map(|v| match v {
                    geojson::Value::Point(p) => vec![p],
                    geojson::Value::MultiPoint(ps) => ps,
                    _ => Vec::new(),
                })
                .collect(),
        ),
        ValueKind::Curve => geojson::Value::MultiLineString(
            values
                .into_iter()
                .flat_map(|v| match v {
                    geojson::Value::LineString(l) => vec![l],
                    geojson::Value::MultiLineString(ls) => ls,
                    _ => Vec::new(),
                })
                .collect(),
        ),
        ValueKind::Area => geojson::Value::MultiPolygon(
            values
                .into_iter()
                .flat_map(|v| match v {
                    geojson::Value::Polygon(p) => vec![p],
                    geojson::Value::MultiPolygon(ps) => ps,
                    _ => Vec::new(),
                })
                .collect(),
        ),
    })
}

/// The `Multi*` family a value belongs to; `None` for a `GeometryCollection`,
/// which has no `Multi*` form to fold into.
#[derive(Clone, Copy, PartialEq, Eq)]
enum ValueKind {
    Point,
    Curve,
    Area,
}

fn value_kind(value: &geojson::Value) -> Option<ValueKind> {
    match value {
        geojson::Value::Point(_) | geojson::Value::MultiPoint(_) => Some(ValueKind::Point),
        geojson::Value::LineString(_) | geojson::Value::MultiLineString(_) => {
            Some(ValueKind::Curve)
        }
        geojson::Value::Polygon(_) | geojson::Value::MultiPolygon(_) => Some(ValueKind::Area),
        geojson::Value::GeometryCollection(_) => None,
    }
}

// GeoJSON positions are `(easting/longitude, northing/latitude[, height])`
// whatever the CRS declares: RFC 7946 section 3.1.1 fixes that order even for the
// alternative reference systems its section 4 allows by prior arrangement, and
// GeoJSON 2008 — the dialect the `crs` member this writer emits comes from —
// states that a CRS shall not change coordinate ordering.

/// Whether a frame stores its horizontal axes reflected from canonical
/// `(East, North)` order, so that they must be swapped on the way out. A frame
/// whose order cannot be established is written as stored.
fn swaps_axes(frame: &CoordinateFrame) -> bool {
    frame.orientation_sign().is_ok_and(|sign| sign < 0)
}

fn xy(swap: bool, [x, y]: [f64; 2]) -> Vec<f64> {
    if swap {
        vec![y, x]
    } else {
        vec![x, y]
    }
}

fn xyz(swap: bool, [x, y, z]: [f64; 3]) -> Vec<f64> {
    if swap {
        vec![y, x, z]
    } else {
        vec![x, y, z]
    }
}

/// GeoJSON requires a ring's first and last positions to be equal. The stored
/// rings carry no such guarantee, so an open one is closed on the way out.
fn closed_ring(positions: impl Iterator<Item = Vec<f64>>) -> Vec<Vec<f64>> {
    let mut ring: Vec<Vec<f64>> = positions.collect();
    let open = match (ring.first(), ring.last()) {
        (Some(first), Some(last)) => first != last,
        _ => false,
    };
    if open {
        let first = ring[0].clone();
        ring.push(first);
    }
    ring
}

// ---------------------------------------------------------------------------
// Coordinate frames of what gets written
// ---------------------------------------------------------------------------

/// Which coordinate reference system the coordinates written for a set of
/// features share. Reported by [`written_crs`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WrittenCrs {
    /// Every written coordinate is expressed in this one CRS.
    Single(EpsgCode),
    /// Written coordinates are expressed in more than one CRS.
    Mixed { first: EpsgCode, other: EpsgCode },
    /// No single CRS covers what is written: some frame is not a CRS at all
    /// (`Euclidean`, or a `Tangent` plane whose in-plane coordinates are not its
    /// base CRS's), or nothing carrying coordinates is written.
    Unknown,
}

/// The CRS the coordinates written for `features` are expressed in.
///
/// Only leaves that reach the output are counted, so the answer describes exactly
/// the coordinates in the file — a `Solid` that is dropped contributes no CRS.
pub fn written_crs(features: &[Feature]) -> WrittenCrs {
    let mut single: Option<EpsgCode> = None;
    let mut mixed: Option<(EpsgCode, EpsgCode)> = None;
    let mut non_crs = false;

    for feature in features {
        visit_frames(&feature.geometry, &mut |frame| {
            let CoordinateFrame::Crs(code) = frame else {
                non_crs = true;
                return;
            };
            match single {
                None => single = Some(*code),
                Some(seen) if seen != *code => {
                    mixed.get_or_insert((seen, *code));
                }
                Some(_) => {}
            }
        });
    }

    match (mixed, single) {
        (Some((first, other)), _) => WrittenCrs::Mixed { first, other },
        (None, Some(code)) if !non_crs => WrittenCrs::Single(code),
        _ => WrittenCrs::Unknown,
    }
}

/// Visit the coordinate frame of every leaf that reaches the GeoJSON output, in
/// order. Leaves with no GeoJSON counterpart contribute nothing, matching what
/// [`geometry_to_value`] writes.
fn visit_frames<'a>(geometry: &'a Geometry, visit: &mut dyn FnMut(&'a CoordinateFrame)) {
    match geometry {
        Geometry::None => {}
        Geometry::Euclidean2D(g) => visit_frames_2d(g, visit),
        Geometry::Euclidean3D(g) => visit_frames_3d(g, visit),
        Geometry::GeometryCollection(c) => {
            for member in c.members() {
                visit_frames(member, visit);
            }
        }
    }
}

fn visit_frames_2d<'a>(
    geometry: &'a Euclidean2DGeometry,
    visit: &mut dyn FnMut(&'a CoordinateFrame),
) {
    match geometry {
        Euclidean2DGeometry::Point(p) => visit(p.frame()),
        Euclidean2DGeometry::LineString(l) => visit(l.frame()),
        Euclidean2DGeometry::Polygon(p) => visit(p.frame()),
        Euclidean2DGeometry::PolygonMesh(m) => visit(m.frame()),
        Euclidean2DGeometry::TriangularMesh(m) => visit(m.frame()),
        Euclidean2DGeometry::Collection(c) => {
            for member in c.members() {
                visit_frames_2d(member, visit);
            }
        }
    }
}

fn visit_frames_3d<'a>(
    geometry: &'a Euclidean3DGeometry,
    visit: &mut dyn FnMut(&'a CoordinateFrame),
) {
    match geometry {
        Euclidean3DGeometry::Point(p) => visit(p.frame()),
        Euclidean3DGeometry::LineString(l) => visit(l.frame()),
        Euclidean3DGeometry::Polygon(p) => visit(p.frame()),
        Euclidean3DGeometry::PolygonMesh(m) => visit(m.frame()),
        Euclidean3DGeometry::TriangularMesh(m) => visit(m.frame()),
        Euclidean3DGeometry::Collection(c) => {
            for member in c.members() {
                visit_frames_3d(member, visit);
            }
        }
        // Nothing of these reaches the output, so they name no frame.
        Euclidean3DGeometry::Solid(_)
        | Euclidean3DGeometry::Csg(_)
        | Euclidean3DGeometry::PointCloud(_) => {}
    }
}

/// Whether every written leaf under `members` shares one coordinate frame.
fn one_frame<'a, T>(
    members: &'a [T],
    visit_frames: fn(&'a T, &mut dyn FnMut(&'a CoordinateFrame)),
) -> bool {
    let mut frames: Vec<&CoordinateFrame> = Vec::new();
    for member in members {
        visit_frames(member, &mut |frame| frames.push(frame));
    }
    frames.windows(2).all(|pair| pair[0] == pair[1])
}

#[cfg(test)]
mod tests {
    use reearth_flow_geometry::{
        collection::{Collection2D, Collection3D},
        coordinate::{CoordinateFrame, EpsgCode},
        line_string::{LineString2D, LineString3D},
        point::{Point2D, Point3D},
        polygon::{Polygon2D, Polygon3D},
        Euclidean2DGeometry, Euclidean3DGeometry, Geometry, GeometryCollection,
    };

    use super::*;
    use crate::{Attribute, AttributeValue};

    fn crs(code: u16) -> CoordinateFrame {
        CoordinateFrame::Crs(EpsgCode::new(code))
    }

    fn geojson_feature(value: geojson::Value) -> geojson::Feature {
        geojson::Feature {
            bbox: None,
            geometry: Some(geojson::Geometry::new(value)),
            id: None,
            properties: None,
            foreign_members: None,
        }
    }

    // A 2D GeoJSON Point (lon, lat) becomes a Euclidean2D Point stored (lat, lon).
    #[test]
    fn point_2d_converts_to_euclidean_2d_wgs84() {
        let gj = geojson_feature(geojson::Value::Point(vec![139.7, 35.6]));

        let feature: Feature = gj.try_into().unwrap();

        assert_eq!(
            *feature.geometry,
            Geometry::Euclidean2D(Euclidean2DGeometry::Point(Point2D::new(
                crs(4326),
                [35.6, 139.7],
            )))
        );
    }

    // A 3D GeoJSON Point (lon, lat, h) becomes a Euclidean3D Point stored (lat, lon, h).
    #[test]
    fn point_3d_converts_to_euclidean_3d_wgs84() {
        let gj = geojson_feature(geojson::Value::Point(vec![139.7, 35.6, 12.5]));

        let feature: Feature = gj.try_into().unwrap();

        assert_eq!(
            *feature.geometry,
            Geometry::Euclidean3D(Euclidean3DGeometry::Point(Point3D::new(
                crs(4979),
                [35.6, 139.7, 12.5],
            )))
        );
    }

    #[test]
    fn line_string_2d_converts() {
        let gj = geojson_feature(geojson::Value::LineString(vec![
            vec![0.0, 0.0],
            vec![1.0, 2.0],
        ]));

        let feature: Feature = gj.try_into().unwrap();

        assert_eq!(
            *feature.geometry,
            Geometry::Euclidean2D(Euclidean2DGeometry::LineString(LineString2D::from_coords(
                crs(4326),
                [[0.0, 0.0], [2.0, 1.0]],
            )))
        );
    }

    #[test]
    fn line_string_3d_converts() {
        let gj = geojson_feature(geojson::Value::LineString(vec![
            vec![0.0, 0.0, 1.0],
            vec![1.0, 2.0, 3.0],
        ]));

        let feature: Feature = gj.try_into().unwrap();

        assert_eq!(
            *feature.geometry,
            Geometry::Euclidean3D(Euclidean3DGeometry::LineString(LineString3D::from_coords(
                crs(4979),
                [[0.0, 0.0, 1.0], [2.0, 1.0, 3.0]],
            )))
        );
    }

    // A 2D Polygon keeps its exterior ring and interior holes.
    #[test]
    fn polygon_2d_with_hole_converts() {
        let exterior = vec![
            vec![0.0, 0.0],
            vec![4.0, 0.0],
            vec![4.0, 4.0],
            vec![0.0, 0.0],
        ];
        let hole = vec![
            vec![1.0, 1.0],
            vec![2.0, 1.0],
            vec![1.0, 2.0],
            vec![1.0, 1.0],
        ];
        let gj = geojson_feature(geojson::Value::Polygon(vec![exterior, hole]));

        let feature: Feature = gj.try_into().unwrap();

        assert_eq!(
            *feature.geometry,
            Geometry::Euclidean2D(Euclidean2DGeometry::Polygon(Box::new(
                Polygon2D::from_rings(
                    crs(4326),
                    [[0.0, 0.0], [0.0, 4.0], [4.0, 4.0], [0.0, 0.0]],
                    [[[1.0, 1.0], [1.0, 2.0], [2.0, 1.0], [1.0, 1.0]]],
                )
            )))
        );
    }

    /// Shoelace signed area of a closed ring in its stored coordinate order.
    fn signed_area(ring: &[[f64; 2]]) -> f64 {
        ring.windows(2)
            .map(|w| w[0][0] * w[1][1] - w[1][0] * w[0][1])
            .sum::<f64>()
            / 2.0
    }

    // A GeoJSON exterior wound CCW in (lon, lat) is stored CW in the (lat, lon) frame:
    // the axis swap flips the raw winding, and the frame's orientation sign (-1 for
    // EPSG:4326) flips it back, so the canonical orientation stays CCW.
    #[test]
    fn ccw_geojson_exterior_is_stored_clockwise() {
        // (0,0) -> (4,0) -> (4,4) -> (0,0): CCW in (lon, lat), positive area.
        let exterior = vec![
            vec![0.0, 0.0],
            vec![4.0, 0.0],
            vec![4.0, 4.0],
            vec![0.0, 0.0],
        ];
        let gj = geojson_feature(geojson::Value::Polygon(vec![exterior]));

        let feature: Feature = gj.try_into().unwrap();

        let Geometry::Euclidean2D(Euclidean2DGeometry::Polygon(polygon)) = &*feature.geometry
        else {
            panic!("expected a 2D polygon");
        };
        assert!(signed_area(polygon.exterior()) < 0.0);
    }

    // A 3D Polygon exterior ring keeps z.
    #[test]
    fn polygon_3d_converts() {
        let exterior = vec![
            vec![0.0, 0.0, 1.0],
            vec![4.0, 0.0, 1.0],
            vec![4.0, 4.0, 1.0],
            vec![0.0, 0.0, 1.0],
        ];
        let gj = geojson_feature(geojson::Value::Polygon(vec![exterior]));

        let feature: Feature = gj.try_into().unwrap();

        assert_eq!(
            *feature.geometry,
            Geometry::Euclidean3D(Euclidean3DGeometry::Polygon(Box::new(
                Polygon3D::from_rings(
                    crs(4979),
                    [
                        [0.0, 0.0, 1.0],
                        [0.0, 4.0, 1.0],
                        [4.0, 4.0, 1.0],
                        [0.0, 0.0, 1.0]
                    ],
                    std::iter::empty::<Vec<[f64; 3]>>(),
                )
            )))
        );
    }

    // MultiPoint (2D) becomes a Collection of Points (the new geometry has no Multi* leaf).
    #[test]
    fn multi_point_2d_converts_to_collection() {
        let gj = geojson_feature(geojson::Value::MultiPoint(vec![
            vec![0.0, 0.0],
            vec![1.0, 1.0],
        ]));

        let feature: Feature = gj.try_into().unwrap();

        assert_eq!(
            *feature.geometry,
            Geometry::Euclidean2D(Euclidean2DGeometry::Collection(Collection2D::new([
                Euclidean2DGeometry::Point(Point2D::new(crs(4326), [0.0, 0.0])),
                Euclidean2DGeometry::Point(Point2D::new(crs(4326), [1.0, 1.0])),
            ])))
        );
    }

    // MultiPoint (3D) becomes a 3D Collection of Points.
    #[test]
    fn multi_point_3d_converts_to_collection() {
        let gj = geojson_feature(geojson::Value::MultiPoint(vec![
            vec![0.0, 0.0, 5.0],
            vec![1.0, 1.0, 6.0],
        ]));

        let feature: Feature = gj.try_into().unwrap();

        assert_eq!(
            *feature.geometry,
            Geometry::Euclidean3D(Euclidean3DGeometry::Collection(Collection3D::new([
                Euclidean3DGeometry::Point(Point3D::new(crs(4979), [0.0, 0.0, 5.0])),
                Euclidean3DGeometry::Point(Point3D::new(crs(4979), [1.0, 1.0, 6.0])),
            ])))
        );
    }

    // MultiLineString (2D) becomes a Collection of LineStrings.
    #[test]
    fn multi_line_string_2d_converts_to_collection() {
        let gj = geojson_feature(geojson::Value::MultiLineString(vec![
            vec![vec![0.0, 0.0], vec![1.0, 1.0]],
            vec![vec![2.0, 2.0], vec![3.0, 3.0]],
        ]));

        let feature: Feature = gj.try_into().unwrap();

        assert_eq!(
            *feature.geometry,
            Geometry::Euclidean2D(Euclidean2DGeometry::Collection(Collection2D::new([
                Euclidean2DGeometry::LineString(LineString2D::from_coords(
                    crs(4326),
                    [[0.0, 0.0], [1.0, 1.0]],
                )),
                Euclidean2DGeometry::LineString(LineString2D::from_coords(
                    crs(4326),
                    [[2.0, 2.0], [3.0, 3.0]],
                )),
            ])))
        );
    }

    // MultiPolygon (2D) becomes a Collection of Polygons.
    #[test]
    fn multi_polygon_2d_converts_to_collection() {
        let poly = vec![vec![
            vec![0.0, 0.0],
            vec![1.0, 0.0],
            vec![0.0, 1.0],
            vec![0.0, 0.0],
        ]];
        let gj = geojson_feature(geojson::Value::MultiPolygon(vec![poly]));

        let feature: Feature = gj.try_into().unwrap();

        assert_eq!(
            *feature.geometry,
            Geometry::Euclidean2D(Euclidean2DGeometry::Collection(Collection2D::new([
                Euclidean2DGeometry::Polygon(Box::new(Polygon2D::from_rings(
                    crs(4326),
                    [[0.0, 0.0], [0.0, 1.0], [1.0, 0.0], [0.0, 0.0]],
                    std::iter::empty::<Vec<[f64; 2]>>(),
                ))),
            ])))
        );
    }

    // A GeometryCollection converts to the new GeometryCollection; its members may
    // differ in dimension (each carries its own coordinate frame).
    #[test]
    fn geometry_collection_converts_with_mixed_dimension_members() {
        let gj = geojson_feature(geojson::Value::GeometryCollection(vec![
            geojson::Geometry::new(geojson::Value::Point(vec![0.0, 0.0])),
            geojson::Geometry::new(geojson::Value::Point(vec![1.0, 1.0, 2.0])),
        ]));

        let feature: Feature = gj.try_into().unwrap();

        assert_eq!(
            *feature.geometry,
            Geometry::GeometryCollection(GeometryCollection::new([
                Geometry::Euclidean2D(Euclidean2DGeometry::Point(Point2D::new(
                    crs(4326),
                    [0.0, 0.0],
                ))),
                Geometry::Euclidean3D(Euclidean3DGeometry::Point(Point3D::new(
                    crs(4979),
                    [1.0, 1.0, 2.0],
                ))),
            ]))
        );
    }

    // A nested GeometryCollection recurses.
    #[test]
    fn nested_geometry_collection_converts() {
        let inner = geojson::Value::GeometryCollection(vec![geojson::Geometry::new(
            geojson::Value::Point(vec![3.0, 4.0]),
        )]); // (lon, lat) -> stored (lat, lon) = [4.0, 3.0]
        let gj = geojson_feature(geojson::Value::GeometryCollection(vec![
            geojson::Geometry::new(inner),
        ]));

        let feature: Feature = gj.try_into().unwrap();

        assert_eq!(
            *feature.geometry,
            Geometry::GeometryCollection(GeometryCollection::new([Geometry::GeometryCollection(
                GeometryCollection::new([Geometry::Euclidean2D(Euclidean2DGeometry::Point(
                    Point2D::new(crs(4326), [4.0, 3.0])
                ),)])
            ),]))
        );
    }

    // Mixed-dimension coordinates are rejected, not panicked on.
    #[test]
    fn mixed_dimension_multi_point_is_unsupported() {
        let gj = geojson_feature(geojson::Value::MultiPoint(vec![
            vec![0.0, 0.0],      // 2D
            vec![1.0, 1.0, 1.0], // 3D
        ]));

        let result: Result<Feature> = gj.try_into();

        assert!(result.is_err());
    }

    // A degenerate coordinate (fewer than 2 elements) is rejected, not panicked on.
    #[test]
    fn degenerate_coordinate_is_unsupported() {
        let gj = geojson_feature(geojson::Value::Point(vec![0.0]));

        let result: Result<Feature> = gj.try_into();

        assert!(result.is_err());
    }

    // Feature properties are carried over as attributes.
    #[test]
    fn properties_become_attributes() {
        let mut props = geojson::JsonObject::new();
        props.insert(
            "name".to_string(),
            serde_json::Value::String("bldg-1".to_string()),
        );
        let mut gj = geojson_feature(geojson::Value::Point(vec![0.0, 0.0]));
        gj.properties = Some(props);

        let feature: Feature = gj.try_into().unwrap();

        assert_eq!(
            feature.attributes.get(&Attribute::new("name")),
            Some(&AttributeValue::String("bldg-1".to_string()))
        );
    }

    // A string feature id that is a valid UUID is preserved.
    #[test]
    fn string_id_parses_as_uuid() {
        let id = "550e8400-e29b-41d4-a716-446655440000";
        let mut gj = geojson_feature(geojson::Value::Point(vec![0.0, 0.0]));
        gj.id = Some(geojson::feature::Id::String(id.to_string()));

        let feature: Feature = gj.try_into().unwrap();

        assert_eq!(feature.id, uuid::Uuid::parse_str(id).unwrap());
    }

    // -----------------------------------------------------------------------
    // Feature -> GeoJSON
    // -----------------------------------------------------------------------

    use reearth_flow_geometry::{
        csg::Csg,
        point_cloud::PointCloud,
        polygon_mesh::{PolygonMesh3D, PolygonMesh3DData},
        solid::Solid,
        triangular_mesh::{TriangularMesh2D, TriangularMesh3D},
    };

    fn feature_with(geometry: Geometry) -> Feature {
        Feature::new_with_attributes_and_geometry(Attributes::new(), geometry)
    }

    fn written(geometry: Geometry) -> Vec<geojson::Feature> {
        feature_with(geometry).try_into().unwrap()
    }

    /// The single GeoJSON geometry `geometry` writes to.
    fn written_value(geometry: Geometry) -> geojson::Value {
        let mut features = written(geometry);
        assert_eq!(features.len(), 1, "expected exactly one GeoJSON feature");
        features
            .remove(0)
            .geometry
            .expect("geometry expected")
            .value
    }

    fn point_2d_in(frame: CoordinateFrame, position: [f64; 2]) -> Geometry {
        Geometry::Euclidean2D(Euclidean2DGeometry::Point(Point2D::new(frame, position)))
    }

    fn polygon_2d_in(frame: CoordinateFrame, ring: Vec<[f64; 2]>) -> Euclidean2DGeometry {
        Euclidean2DGeometry::Polygon(Box::new(Polygon2D::from_rings(
            frame,
            ring,
            std::iter::empty::<Vec<[f64; 2]>>(),
        )))
    }

    fn triangle_mesh_3d(frame: CoordinateFrame) -> TriangularMesh3D {
        TriangularMesh3D::from_parts(
            frame,
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            [0u32, 1, 2],
        )
        .unwrap()
    }

    fn quad_mesh_data() -> PolygonMesh3DData {
        PolygonMesh3DData::from_parts(
            vec![
                [0.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [2.0, 2.0, 0.0],
                [0.0, 2.0, 0.0],
            ],
            vec![vec![0u32, 1, 2, 3]],
        )
        .unwrap()
    }

    // --- axis order ---

    // EPSG:4326 declares (lat, lon), so the stored pair is swapped back to the
    // (lon, lat) GeoJSON always uses.
    #[test]
    fn north_first_geographic_crs_swaps_the_horizontal_pair() {
        let value = written_value(point_2d_in(crs(4326), [35.6, 139.7]));

        assert_eq!(value, geojson::Value::Point(vec![139.7, 35.6]));
    }

    // EPSG:6675 declares (northing, easting): the same swap applies to a
    // projected CRS, matching the quality-check error GeoJSON files.
    #[test]
    fn north_first_projected_crs_swaps_the_horizontal_pair() {
        let value = written_value(point_2d_in(crs(6675), [71805.43, -10191.37]));

        assert_eq!(value, geojson::Value::Point(vec![-10191.37, 71805.43]));
    }

    // EPSG:3857 is already (easting, northing).
    #[test]
    fn east_first_crs_is_written_as_stored() {
        let value = written_value(point_2d_in(crs(3857), [1.0, 2.0]));

        assert_eq!(value, geojson::Value::Point(vec![1.0, 2.0]));
    }

    #[test]
    fn euclidean_frame_is_written_as_stored() {
        let value = written_value(point_2d_in(CoordinateFrame::Euclidean, [1.0, 2.0]));

        assert_eq!(value, geojson::Value::Point(vec![1.0, 2.0]));
    }

    // z is carried through unchanged; only the horizontal pair is reordered.
    #[test]
    fn a_3d_point_keeps_its_height() {
        let value = written_value(Geometry::Euclidean3D(Euclidean3DGeometry::Point(
            Point3D::new(crs(4979), [35.6, 139.7, 12.5]),
        )));

        assert_eq!(value, geojson::Value::Point(vec![139.7, 35.6, 12.5]));
    }

    // A 2.5D leaf's single elevation is not written: the old world's 2D geometry
    // was two-element too.
    #[test]
    fn a_2d_line_string_at_an_elevation_is_written_without_it() {
        let value = written_value(Geometry::Euclidean2D(Euclidean2DGeometry::LineString(
            LineString2D::from_coords_at_elevation(
                CoordinateFrame::Euclidean,
                [[0.0, 0.0], [2.0, 1.0]],
                5.0,
            ),
        )));

        assert_eq!(
            value,
            geojson::Value::LineString(vec![vec![0.0, 0.0], vec![2.0, 1.0]])
        );
    }

    // --- rings ---

    #[test]
    fn a_polygon_writes_its_exterior_then_its_holes() {
        let polygon = Polygon2D::from_rings(
            CoordinateFrame::Euclidean,
            [[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 0.0]],
            [[[1.0, 1.0], [2.0, 1.0], [1.0, 2.0], [1.0, 1.0]]],
        );

        let value = written_value(Geometry::Euclidean2D(Euclidean2DGeometry::Polygon(
            Box::new(polygon),
        )));

        assert_eq!(
            value,
            geojson::Value::Polygon(vec![
                vec![
                    vec![0.0, 0.0],
                    vec![4.0, 0.0],
                    vec![4.0, 4.0],
                    vec![0.0, 0.0]
                ],
                vec![
                    vec![1.0, 1.0],
                    vec![2.0, 1.0],
                    vec![1.0, 2.0],
                    vec![1.0, 1.0]
                ],
            ])
        );
    }

    // Stored rings carry no closure guarantee, but GeoJSON requires one.
    #[test]
    fn an_open_ring_is_closed_on_the_way_out() {
        let polygon = Polygon2D::from_rings(
            CoordinateFrame::Euclidean,
            [[0.0, 0.0], [4.0, 0.0], [4.0, 4.0]],
            std::iter::empty::<Vec<[f64; 2]>>(),
        );

        let value = written_value(Geometry::Euclidean2D(Euclidean2DGeometry::Polygon(
            Box::new(polygon),
        )));

        assert_eq!(
            value,
            geojson::Value::Polygon(vec![vec![
                vec![0.0, 0.0],
                vec![4.0, 0.0],
                vec![4.0, 4.0],
                vec![0.0, 0.0],
            ]])
        );
    }

    // --- collections ---

    #[test]
    fn a_collection_of_polygons_folds_into_a_multi_polygon() {
        let collection = Collection2D::new([
            polygon_2d_in(
                CoordinateFrame::Euclidean,
                vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0], [0.0, 0.0]],
            ),
            polygon_2d_in(
                CoordinateFrame::Euclidean,
                vec![[2.0, 2.0], [3.0, 2.0], [2.0, 3.0], [2.0, 2.0]],
            ),
        ]);

        let value = written_value(Geometry::Euclidean2D(Euclidean2DGeometry::Collection(
            collection,
        )));

        assert!(
            matches!(&value, geojson::Value::MultiPolygon(polygons) if polygons.len() == 2),
            "got: {value:?}"
        );
    }

    #[test]
    fn a_collection_of_points_folds_into_a_multi_point() {
        let collection = Collection2D::new([
            Euclidean2DGeometry::Point(Point2D::new(CoordinateFrame::Euclidean, [0.0, 0.0])),
            Euclidean2DGeometry::Point(Point2D::new(CoordinateFrame::Euclidean, [1.0, 1.0])),
        ]);

        let value = written_value(Geometry::Euclidean2D(Euclidean2DGeometry::Collection(
            collection,
        )));

        assert_eq!(
            value,
            geojson::Value::MultiPoint(vec![vec![0.0, 0.0], vec![1.0, 1.0]])
        );
    }

    // Members of different kinds have no common `Multi*`.
    #[test]
    fn a_collection_of_mixed_kinds_becomes_a_geometry_collection() {
        let collection = Collection2D::new([
            Euclidean2DGeometry::Point(Point2D::new(CoordinateFrame::Euclidean, [0.0, 0.0])),
            polygon_2d_in(
                CoordinateFrame::Euclidean,
                vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0], [0.0, 0.0]],
            ),
        ]);

        let value = written_value(Geometry::Euclidean2D(Euclidean2DGeometry::Collection(
            collection,
        )));

        assert!(
            matches!(&value, geojson::Value::GeometryCollection(members) if members.len() == 2),
            "got: {value:?}"
        );
    }

    // Folding members that differ in frame would put coordinates from two
    // reference systems in one geometry, which one `crs` member cannot describe.
    #[test]
    fn a_collection_of_mixed_frames_becomes_a_geometry_collection() {
        let collection = Collection2D::new([
            Euclidean2DGeometry::Point(Point2D::new(crs(3857), [0.0, 0.0])),
            Euclidean2DGeometry::Point(Point2D::new(CoordinateFrame::Euclidean, [1.0, 1.0])),
        ]);

        let value = written_value(Geometry::Euclidean2D(Euclidean2DGeometry::Collection(
            collection,
        )));

        assert!(
            matches!(&value, geojson::Value::GeometryCollection(members) if members.len() == 2),
            "got: {value:?}"
        );
    }

    // A cross-dimensional collection has no single GeoJSON geometry, so its
    // members become features of their own, sharing the feature's properties.
    #[test]
    fn a_geometry_collection_expands_into_one_feature_per_member() {
        let mut attributes = Attributes::new();
        attributes.insert(
            Attribute::new("name"),
            AttributeValue::String("bldg-1".to_string()),
        );
        let geometry = Geometry::GeometryCollection(GeometryCollection::new([
            point_2d_in(CoordinateFrame::Euclidean, [0.0, 0.0]),
            Geometry::Euclidean3D(Euclidean3DGeometry::Point(Point3D::new(
                CoordinateFrame::Euclidean,
                [1.0, 1.0, 1.0],
            ))),
        ]));

        let features: Vec<geojson::Feature> =
            Feature::new_with_attributes_and_geometry(attributes, geometry)
                .try_into()
                .unwrap();

        assert_eq!(features.len(), 2);
        assert_eq!(
            features[0].geometry.as_ref().map(|g| &g.value),
            Some(&geojson::Value::Point(vec![0.0, 0.0]))
        );
        assert_eq!(
            features[1].geometry.as_ref().map(|g| &g.value),
            Some(&geojson::Value::Point(vec![1.0, 1.0, 1.0]))
        );
        for feature in &features {
            assert_eq!(feature.properties.as_ref().unwrap()["name"], "bldg-1");
        }
        assert_ne!(features[0].id, features[1].id);
    }

    // Only the outermost collection expands; a nested one stays a geometry.
    #[test]
    fn a_nested_geometry_collection_stays_one_geojson_geometry() {
        let inner = Geometry::GeometryCollection(GeometryCollection::new([point_2d_in(
            CoordinateFrame::Euclidean,
            [3.0, 4.0],
        )]));
        let geometry = Geometry::GeometryCollection(GeometryCollection::new([inner]));

        let value = written_value(geometry);

        assert_eq!(
            value,
            geojson::Value::GeometryCollection(vec![geojson::Geometry::new(
                geojson::Value::Point(vec![3.0, 4.0])
            )])
        );
    }

    // --- meshes ---

    // A mesh has no GeoJSON counterpart of its own, so its faces are written.
    #[test]
    fn a_polygon_mesh_writes_one_polygon_per_face() {
        let mesh = PolygonMesh3D::new(CoordinateFrame::Euclidean, quad_mesh_data());

        let value = written_value(Geometry::Euclidean3D(Euclidean3DGeometry::PolygonMesh(
            Box::new(mesh),
        )));

        assert_eq!(
            value,
            geojson::Value::MultiPolygon(vec![vec![vec![
                vec![0.0, 0.0, 0.0],
                vec![2.0, 0.0, 0.0],
                vec![2.0, 2.0, 0.0],
                vec![0.0, 2.0, 0.0],
                vec![0.0, 0.0, 0.0],
            ]]])
        );
    }

    #[test]
    fn a_triangular_mesh_writes_one_polygon_per_triangle() {
        let value = written_value(Geometry::Euclidean3D(Euclidean3DGeometry::TriangularMesh(
            Box::new(triangle_mesh_3d(CoordinateFrame::Euclidean)),
        )));

        assert_eq!(
            value,
            geojson::Value::MultiPolygon(vec![vec![vec![
                vec![0.0, 0.0, 0.0],
                vec![1.0, 0.0, 0.0],
                vec![0.0, 1.0, 0.0],
                vec![0.0, 0.0, 0.0],
            ]]])
        );
    }

    // Splitting reads the mesh; the geometry it was taken from is left intact.
    #[test]
    fn writing_a_mesh_leaves_the_geometry_unchanged() {
        let geometry = Geometry::Euclidean2D(Euclidean2DGeometry::TriangularMesh(Box::new(
            TriangularMesh2D::from_parts(
                CoordinateFrame::Euclidean,
                vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]],
                [0u32, 1, 2],
            )
            .unwrap(),
        )));
        let feature = feature_with(geometry.clone());

        let _: Vec<geojson::Feature> = feature.clone().try_into().unwrap();

        assert_eq!(*feature.geometry, geometry);
    }

    // --- geometries with no GeoJSON counterpart ---

    #[test]
    fn a_solid_is_rejected_rather_than_panicking() {
        let solid = Solid::from_exterior(CoordinateFrame::Euclidean, quad_mesh_data());
        let feature = feature_with(Geometry::Euclidean3D(Euclidean3DGeometry::Solid(Box::new(
            solid,
        ))));

        let result: Result<Vec<geojson::Feature>> = feature.try_into();

        assert!(result.is_err());
    }

    #[test]
    fn a_csg_tree_and_a_point_cloud_are_rejected_rather_than_panicking() {
        let solid = || Solid::from_exterior(CoordinateFrame::Euclidean, quad_mesh_data());
        for geometry in [
            Geometry::Euclidean3D(Euclidean3DGeometry::Csg(Csg::union(solid(), solid()))),
            Geometry::Euclidean3D(Euclidean3DGeometry::PointCloud(Box::new(
                PointCloud::from_positions(CoordinateFrame::Euclidean, [[0.0, 0.0, 0.0]]),
            ))),
        ] {
            let result: Result<Vec<geojson::Feature>> = feature_with(geometry).try_into();
            assert!(result.is_err());
        }
    }

    // One member with no counterpart does not discard its siblings.
    #[test]
    fn a_solid_among_polygons_is_dropped_and_the_rest_survive() {
        let collection = Collection3D::new([
            Euclidean3DGeometry::Solid(Box::new(Solid::from_exterior(
                CoordinateFrame::Euclidean,
                quad_mesh_data(),
            ))),
            Euclidean3DGeometry::Polygon(Box::new(Polygon3D::from_rings(
                CoordinateFrame::Euclidean,
                [
                    [0.0, 0.0, 0.0],
                    [1.0, 0.0, 0.0],
                    [0.0, 1.0, 0.0],
                    [0.0, 0.0, 0.0],
                ],
                std::iter::empty::<Vec<[f64; 3]>>(),
            ))),
        ]);

        let value = written_value(Geometry::Euclidean3D(Euclidean3DGeometry::Collection(
            collection,
        )));

        assert!(
            matches!(&value, geojson::Value::MultiPolygon(polygons) if polygons.len() == 1),
            "got: {value:?}"
        );
    }

    // --- features without geometry ---

    // An attribute-only row survives as a feature with a null geometry.
    #[test]
    fn a_feature_without_geometry_writes_a_null_geometry() {
        let features = written(Geometry::None);

        assert_eq!(features.len(), 1);
        assert!(features[0].geometry.is_none());
    }

    #[test]
    fn attributes_and_id_carry_over() {
        let mut attributes = Attributes::new();
        attributes.insert(
            Attribute::new("city"),
            AttributeValue::String("Tokyo".to_string()),
        );
        let feature =
            Feature::new_with_attributes_and_geometry(attributes, point_2d_in(crs(3857), [0.0; 2]));
        let id = feature.id;

        let features: Vec<geojson::Feature> = feature.try_into().unwrap();

        assert_eq!(features[0].properties.as_ref().unwrap()["city"], "Tokyo");
        assert_eq!(
            features[0].id,
            Some(geojson::feature::Id::String(id.to_string()))
        );
    }

    // --- the CRS of what gets written ---

    #[test]
    fn written_crs_reports_the_shared_epsg_code() {
        let features = [
            feature_with(point_2d_in(crs(6675), [0.0; 2])),
            feature_with(point_2d_in(crs(6675), [1.0; 2])),
        ];

        assert_eq!(
            written_crs(&features),
            WrittenCrs::Single(EpsgCode::new(6675))
        );
    }

    #[test]
    fn written_crs_reports_differing_epsg_codes() {
        let features = [
            feature_with(point_2d_in(crs(6675), [0.0; 2])),
            feature_with(point_2d_in(crs(6669), [1.0; 2])),
        ];

        assert_eq!(
            written_crs(&features),
            WrittenCrs::Mixed {
                first: EpsgCode::new(6675),
                other: EpsgCode::new(6669),
            }
        );
    }

    #[test]
    fn written_crs_is_unknown_without_a_crs_frame() {
        let features = [
            feature_with(point_2d_in(CoordinateFrame::Euclidean, [0.0; 2])),
            feature_with(Geometry::None),
        ];

        assert_eq!(written_crs(&features), WrittenCrs::Unknown);
    }

    // A leaf that is dropped names no CRS, so the answer describes exactly the
    // coordinates in the file.
    #[test]
    fn written_crs_ignores_geometry_that_is_not_written() {
        let features = [feature_with(Geometry::Euclidean3D(
            Euclidean3DGeometry::Solid(Box::new(Solid::from_exterior(
                CoordinateFrame::Crs(EpsgCode::new(6675)),
                quad_mesh_data(),
            ))),
        ))];

        assert_eq!(written_crs(&features), WrittenCrs::Unknown);
    }

    // --- round trips ---

    // Reading then writing returns the GeoJSON that was read: the axis swap on
    // the way in is undone on the way out.
    #[test]
    fn reading_then_writing_returns_the_original_geometry() {
        let values = [
            geojson::Value::Point(vec![139.7, 35.6]),
            geojson::Value::Point(vec![139.7, 35.6, 12.5]),
            geojson::Value::LineString(vec![vec![0.0, 0.0], vec![1.0, 2.0]]),
            geojson::Value::Polygon(vec![
                vec![
                    vec![0.0, 0.0],
                    vec![4.0, 0.0],
                    vec![4.0, 4.0],
                    vec![0.0, 0.0],
                ],
                vec![
                    vec![1.0, 1.0],
                    vec![2.0, 1.0],
                    vec![1.0, 2.0],
                    vec![1.0, 1.0],
                ],
            ]),
            geojson::Value::MultiPoint(vec![vec![0.0, 0.0], vec![1.0, 1.0]]),
            geojson::Value::MultiLineString(vec![
                vec![vec![0.0, 0.0], vec![1.0, 1.0]],
                vec![vec![2.0, 2.0], vec![3.0, 3.0]],
            ]),
            geojson::Value::MultiPolygon(vec![vec![vec![
                vec![0.0, 0.0],
                vec![1.0, 0.0],
                vec![0.0, 1.0],
                vec![0.0, 0.0],
            ]]]),
        ];

        for value in values {
            let feature: Feature = geojson_feature(value.clone()).try_into().unwrap();
            assert_eq!(written_value((*feature.geometry).clone()), value);
        }
    }

    // The one asymmetry: a top-level GeometryCollection is read as one feature
    // and written back as one feature per member, so what round-trips is the
    // set of member geometries rather than the collection.
    #[test]
    fn a_geometry_collection_round_trips_as_its_members() {
        let members = vec![
            geojson::Geometry::new(geojson::Value::Point(vec![3.0, 4.0])),
            geojson::Geometry::new(geojson::Value::Point(vec![5.0, 6.0, 7.0])),
        ];
        let feature: Feature = geojson_feature(geojson::Value::GeometryCollection(members.clone()))
            .try_into()
            .unwrap();

        let written: Vec<geojson::Feature> = feature.try_into().unwrap();

        let values: Vec<_> = written
            .into_iter()
            .map(|f| f.geometry.expect("geometry expected").value)
            .collect();
        assert_eq!(
            values,
            members.into_iter().map(|g| g.value).collect::<Vec<_>>()
        );
    }

    // The quality-check error GeoJSON files declare EPSG:6675 and carry easting
    // first. New-geometry workflow tests do not run, so the shape those files
    // depend on is pinned here.
    #[test]
    fn quality_check_error_geometry_keeps_its_stored_coordinates() {
        // As stored in EPSG:6675's declared (northing, easting) order.
        let ring = vec![
            [71805.43986040869, -10191.374874677815],
            [71804.88506036208, -10191.375533456288],
            [71804.88452953615, -10190.928471234001],
            [71805.43932955855, -10190.927812483467],
            [71805.43986040869, -10191.374874677815],
        ];
        let collection = Collection2D::new([polygon_2d_in(crs(6675), ring)]);
        let feature = feature_with(Geometry::Euclidean2D(Euclidean2DGeometry::Collection(
            collection,
        )));

        assert_eq!(
            written_crs(std::slice::from_ref(&feature)),
            WrittenCrs::Single(EpsgCode::new(6675))
        );
        assert_eq!(
            written_value((*feature.geometry).clone()),
            geojson::Value::MultiPolygon(vec![vec![vec![
                vec![-10191.374874677815, 71805.43986040869],
                vec![-10191.375533456288, 71804.88506036208],
                vec![-10190.928471234001, 71804.88452953615],
                vec![-10190.927812483467, 71805.43932955855],
                vec![-10191.374874677815, 71805.43986040869],
            ]]])
        );
    }
}
