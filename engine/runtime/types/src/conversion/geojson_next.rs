//! GeoJSON <-> `Feature` conversion for the new-geometry world. Builds
//! `reearth_flow_geometry::Geometry` (per-leaf `CoordinateFrame`) rather than the
//! old `GeometryValue` wrapper.
//!
//! Both directions live here so the axis-order convention stays one invariant:
//! GeoJSON positions are always `(easting/longitude, northing/latitude[, height])`,
//! while a CRS frame stores its axes in the order the CRS authority declares, so
//! reading swaps the horizontal pair and writing swaps it back.

use std::collections::HashSet;
use std::sync::{Arc, Mutex, OnceLock};

use itertools::Itertools;
use reearth_flow_geometry::types::conversion::{is_2d_geojson_value, is_3d_geojson_value};
use reearth_flow_geometry::{
    collection::{Collection2D, Collection3D},
    coordinate::{CoordinateFrame, EpsgCode},
    line_string::{LineString2D, LineString3D},
    ops::{Split, UnsupportedOperation},
    point::{Point2D, Point3D},
    polygon::{Polygon2D, Polygon3D},
    Euclidean2DGeometry, Euclidean3DGeometry, Geometry, GeometryCollection,
};

use crate::{
    error::{Error, Result},
    Attribute, Attributes, Feature,
};

pub use super::geojson_shared::{CrsCoverage, WrittenFeature};

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
//
// Writing is one recursive map, `&Geometry -> Result<WrittenGeometry, Unwritable>`:
// a geometry either writes to a `WrittenGeometry` or names the reason it cannot be
// written.

impl TryFrom<Feature> for Vec<geojson::Feature> {
    type Error = Error;

    fn try_from(feature: Feature) -> Result<Self> {
        write_feature(&feature).map(|written| written.features)
    }
}

/// What `feature` writes to as GeoJSON.
pub fn write_feature(feature: &Feature) -> Result<WrittenFeature> {
    let properties = attributes_to_geojson_object(&feature.attributes);
    match &*feature.geometry {
        // A feature carrying attributes but no geometry still produces a row,
        // stating nothing about the CRS.
        Geometry::None => Ok(WrittenFeature {
            features: vec![geojson_feature(feature.id, None, properties)],
            crs: CrsCoverage::NoCoordinates,
        }),
        // A cross-dimensional, cross-frame collection has no single GeoJSON
        // geometry, so each member becomes a feature of its own — as the old
        // CityGML geometry did — sharing the feature's properties.
        Geometry::GeometryCollection(collection) => {
            let parts = Parts::of(
                collection.members(),
                write_geometry,
                Unwritable::EmptyCollection,
            )?;
            warn_omitted(&parts.omitted);
            let mut frames = Frames::Nothing;
            let mut features = Vec::with_capacity(parts.written.len());
            for member in parts.written {
                frames = frames.and(member.frames);
                features.push(geojson_feature(
                    uuid::Uuid::new_v4(),
                    Some(member.value),
                    properties.clone(),
                ));
            }
            Ok(WrittenFeature {
                features,
                crs: frames.coverage(),
            })
        }
        geometry => {
            let written = write_geometry(geometry)?;
            warn_omitted(&written.omitted);
            Ok(WrittenFeature {
                features: vec![geojson_feature(feature.id, Some(written.value), properties)],
                crs: written.frames.coverage(),
            })
        }
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

/// Why a geometry cannot be written as GeoJSON — the `Err` half of what
/// [`write_geometry`] returns.
///
/// Kept out of [`Error`] while a geometry is being written: within the writer a
/// reason is a value to compare, collect and warn about, and it turns into a
/// message only where it crosses the conversion's boundary.
#[derive(thiserror::Error, Clone, Copy, Debug, PartialEq, Eq)]
enum Unwritable {
    /// A geometry asked for where the feature has none. The feature still writes
    /// a row, with a null geometry.
    #[error("an absent geometry has no GeoJSON counterpart")]
    AbsentGeometry,
    /// GeoJSON has no volume, nor the boolean tree built from volumes.
    #[error("a Solid has no GeoJSON counterpart")]
    Solid,
    #[error("a Csg tree has no GeoJSON counterpart")]
    Csg,
    /// A `MultiPoint` could hold one, but that would emit a position per sample.
    #[error("a PointCloud has no GeoJSON counterpart")]
    PointCloud,
    /// A collection with nothing writable under it. Writing it would emit an
    /// empty geometry that states nothing about the feature.
    #[error("an empty collection has no GeoJSON counterpart")]
    EmptyCollection,
    /// As above, for a mesh, which writes as its faces.
    #[error("a mesh with no face has no GeoJSON counterpart")]
    EmptyMesh,
    /// A mesh whose faces could not be read.
    #[error(transparent)]
    Unsplittable(#[from] UnsupportedOperation),
}

impl From<Unwritable> for Error {
    fn from(reason: Unwritable) -> Self {
        Error::unsupported_feature(reason)
    }
}

/// Report what the writer left out. An omission does not fail the geometry around
/// it, so this warning is the only trace of it.
fn warn_omitted(omitted: &[Unwritable]) {
    for reason in omitted {
        tracing::warn!(%reason, "omitting a geometry member from the GeoJSON output");
    }
}

/// What a geometry writes to: the GeoJSON geometry, the coordinate frames its
/// positions came from, and the reasons parts of it were left out.
///
/// The frames and reasons come out of the same pass as the value, so nothing has
/// to re-walk the geometry to recover them. Carrying the reasons rather than
/// logging them where a part is dropped also keeps the recursion a pure map, the
/// whole geometry's omissions being reported once it is written.
struct WrittenGeometry {
    value: geojson::Value,
    frames: Frames,
    omitted: Vec<Unwritable>,
}

impl WrittenGeometry {
    /// What a leaf writes to: one value, in one frame, leaving nothing out.
    fn leaf(frame: &CoordinateFrame, value: geojson::Value) -> Self {
        Self {
            value,
            frames: Frames::of(frame),
            omitted: Vec::new(),
        }
    }
}

/// What `geometry` writes to, or the reason it cannot be written.
fn write_geometry(geometry: &Geometry) -> Result<WrittenGeometry, Unwritable> {
    match geometry {
        Geometry::None => Err(Unwritable::AbsentGeometry),
        Geometry::Euclidean2D(g) => write_2d(g),
        Geometry::Euclidean3D(g) => write_3d(g),
        // Only the outermost collection expands into separate features; a nested
        // one stays a GeoJSON `GeometryCollection` whatever its members are, being
        // the cross-dimensional, cross-frame container no `Multi*` describes.
        Geometry::GeometryCollection(c) => {
            Parts::of(c.members(), write_geometry, Unwritable::EmptyCollection)
                .map(Parts::into_geometry_collection)
        }
    }
}

fn write_2d(geometry: &Euclidean2DGeometry) -> Result<WrittenGeometry, Unwritable> {
    use Euclidean2DGeometry::*;
    match geometry {
        Point(p) => Ok(point(p.frame(), p.position())),
        LineString(l) => Ok(curve(l.frame(), l.coords())),
        Polygon(p) => Ok(area(p.frame(), p.exterior(), p.interiors())),
        PolygonMesh(m) => write_faces((**m).clone()),
        TriangularMesh(m) => write_faces((**m).clone()),
        Collection(c) => Parts::of(c.members(), write_2d, Unwritable::EmptyCollection)
            .map(Parts::into_one_geometry),
    }
}

fn write_3d(geometry: &Euclidean3DGeometry) -> Result<WrittenGeometry, Unwritable> {
    use Euclidean3DGeometry::*;
    match geometry {
        Point(p) => Ok(point(p.frame(), p.position())),
        LineString(l) => Ok(curve(l.frame(), l.coords())),
        Polygon(p) => Ok(area(p.frame(), p.exterior(), p.interiors())),
        PolygonMesh(m) => write_faces((**m).clone()),
        TriangularMesh(m) => write_faces((**m).clone()),
        Collection(c) => Parts::of(c.members(), write_3d, Unwritable::EmptyCollection)
            .map(Parts::into_one_geometry),
        Solid(_) => Err(Unwritable::Solid),
        Csg(_) => Err(Unwritable::Csg),
        PointCloud(_) => Err(Unwritable::PointCloud),
    }
}

/// A mesh writes as its faces, folding into a `MultiPolygon` the way a collection
/// writes as its members.
///
/// The `Split` op that yields the faces takes `&mut self`, hence the mesh is passed
/// by value; splitting reads it rather than emptying it, so the geometry it came
/// from is left intact.
fn write_faces(mut mesh: impl Split) -> Result<WrittenGeometry, Unwritable> {
    let mut faces = Vec::new();
    mesh.split(&mut |face, _| faces.push(face))?;
    Parts::of(&faces, write_geometry, Unwritable::EmptyMesh).map(Parts::into_one_geometry)
}

/// The writable parts of a container, and every reason the writer left something
/// out.
struct Parts {
    /// Non-empty: a container with no writable part cannot be written either,
    /// which [`Parts::of`] reports as an error instead.
    written: Vec<WrittenGeometry>,
    omitted: Vec<Unwritable>,
}

impl Parts {
    /// Write every part, keeping the writable ones: a part GeoJSON cannot express
    /// is dropped where it appears rather than failing the geometry around it, so
    /// it does not discard its siblings. The reasons come out together — the
    /// dropped parts' and those the survivors collected themselves — so the whole
    /// geometry's omissions are reported in one place.
    ///
    /// `Err` once nothing is left, a container reduced to nothing being unwritable
    /// too. That error carries out a dropped part's reason, naming what could not
    /// be written once at the top, or `empty` when there were no parts to drop.
    fn of<T>(
        parts: &[T],
        write: impl Fn(&T) -> Result<WrittenGeometry, Unwritable>,
        empty: Unwritable,
    ) -> Result<Self, Unwritable> {
        let (mut written, mut omitted): (Vec<WrittenGeometry>, Vec<Unwritable>) =
            parts.iter().map(write).partition_result();
        if written.is_empty() {
            return Err(omitted.first().copied().unwrap_or(empty));
        }
        for part in &mut written {
            omitted.append(&mut part.omitted);
        }
        Ok(Self { written, omitted })
    }

    /// The parts as one GeoJSON geometry: a `Multi*` when they are all of one
    /// kind and share a coordinate frame, a `GeometryCollection` otherwise.
    ///
    /// `Collection2D` / `Collection3D` are the new geometry's `Multi*`, so folding
    /// is the inverse of the read side's `MultiPolygon -> Collection` mapping.
    /// Parts that differ in frame are not folded: that would put coordinates from
    /// different reference systems in one geometry, which no single `crs` member
    /// describes.
    fn into_one_geometry(self) -> WrittenGeometry {
        self.present(|values, frames| match ValueKind::uniform(&values) {
            Some(kind) if frames.uniform() => kind.fold(values),
            _ => geometry_collection(values),
        })
    }

    /// The parts as one GeoJSON `GeometryCollection`, whatever they are.
    fn into_geometry_collection(self) -> WrittenGeometry {
        self.present(|values, _| geometry_collection(values))
    }

    /// The parts as the one geometry `present` builds from their values, carrying
    /// the frames they were written in and what the writer left out.
    fn present(
        self,
        present: impl FnOnce(Vec<geojson::Value>, &Frames) -> geojson::Value,
    ) -> WrittenGeometry {
        let (values, frames): (Vec<_>, Vec<_>) = self
            .written
            .into_iter()
            .map(|part| (part.value, part.frames))
            .unzip();
        let frames = frames.into_iter().fold(Frames::Nothing, Frames::and);
        WrittenGeometry {
            value: present(values, &frames),
            frames,
            omitted: self.omitted,
        }
    }
}

fn geometry_collection(values: Vec<geojson::Value>) -> geojson::Value {
    geojson::Value::GeometryCollection(values.into_iter().map(geojson::Geometry::new).collect())
}

/// The `Multi*` family a GeoJSON geometry belongs to; a `GeometryCollection`
/// belongs to none, having no `Multi*` form to fold into.
#[derive(Clone, Copy, PartialEq, Eq)]
enum ValueKind {
    Point,
    Curve,
    Area,
}

impl ValueKind {
    fn of(value: &geojson::Value) -> Option<Self> {
        match value {
            geojson::Value::Point(_) | geojson::Value::MultiPoint(_) => Some(Self::Point),
            geojson::Value::LineString(_) | geojson::Value::MultiLineString(_) => Some(Self::Curve),
            geojson::Value::Polygon(_) | geojson::Value::MultiPolygon(_) => Some(Self::Area),
            geojson::Value::GeometryCollection(_) => None,
        }
    }

    /// The one family every value belongs to, or `None` when they belong to more
    /// than one, one of them belongs to none, or there are none at all.
    fn uniform(values: &[geojson::Value]) -> Option<Self> {
        let kind = Self::of(values.first()?)?;
        values
            .iter()
            .all(|value| Self::of(value) == Some(kind))
            .then_some(kind)
    }

    /// Values that all belong to this family, folded into the matching `Multi*`.
    /// A value that is already a `Multi*` is flattened into the result; one of
    /// another family cannot occur, [`ValueKind::uniform`] having established a
    /// single family.
    fn fold(self, values: Vec<geojson::Value>) -> geojson::Value {
        match self {
            Self::Point => geojson::Value::MultiPoint(
                values
                    .into_iter()
                    .flat_map(|value| match value {
                        geojson::Value::Point(p) => vec![p],
                        geojson::Value::MultiPoint(ps) => ps,
                        _ => Vec::new(),
                    })
                    .collect(),
            ),
            Self::Curve => geojson::Value::MultiLineString(
                values
                    .into_iter()
                    .flat_map(|value| match value {
                        geojson::Value::LineString(l) => vec![l],
                        geojson::Value::MultiLineString(ls) => ls,
                        _ => Vec::new(),
                    })
                    .collect(),
            ),
            Self::Area => geojson::Value::MultiPolygon(
                values
                    .into_iter()
                    .flat_map(|value| match value {
                        geojson::Value::Polygon(p) => vec![p],
                        geojson::Value::MultiPolygon(ps) => ps,
                        _ => Vec::new(),
                    })
                    .collect(),
            ),
        }
    }
}

// ---------------------------------------------------------------------------
// Leaves
// ---------------------------------------------------------------------------
//
// The 2D and 3D leaves differ only in how long a position is, so what turns one
// into GeoJSON is written once, over `N`-element positions.

fn point<const N: usize>(frame: &CoordinateFrame, position: [f64; N]) -> WrittenGeometry {
    WrittenGeometry::leaf(
        frame,
        geojson::Value::Point(coordinates(swaps_axes(frame), position)),
    )
}

fn curve<const N: usize>(frame: &CoordinateFrame, coords: &[[f64; N]]) -> WrittenGeometry {
    let swap = swaps_axes(frame);
    WrittenGeometry::leaf(
        frame,
        geojson::Value::LineString(coords.iter().map(|&c| coordinates(swap, c)).collect()),
    )
}

/// What an area writes to: its exterior ring, then its holes.
fn area<'a, const N: usize>(
    frame: &CoordinateFrame,
    exterior: &'a [[f64; N]],
    interiors: impl Iterator<Item = &'a [[f64; N]]>,
) -> WrittenGeometry {
    let swap = swaps_axes(frame);
    WrittenGeometry::leaf(
        frame,
        geojson::Value::Polygon(
            std::iter::once(exterior)
                .chain(interiors)
                .map(|ring| closed_ring(swap, ring))
                .collect(),
        ),
    )
}

// GeoJSON positions are `(easting/longitude, northing/latitude[, height])`
// whatever the CRS declares: RFC 7946 section 3.1.1 fixes that order even for the
// alternative reference systems its section 4 allows, and GeoJSON 2008 — where the
// `crs` member comes from — states that a CRS shall not change coordinate ordering.

/// A stored coordinate as a GeoJSON position. Only the horizontal pair is
/// reordered; a height stays where it is.
fn coordinates<const N: usize>(swap: bool, mut coordinate: [f64; N]) -> Vec<f64> {
    if swap {
        coordinate.swap(0, 1);
    }
    coordinate.to_vec()
}

/// GeoJSON requires a ring's first and last positions to be equal. The stored
/// rings carry no such guarantee, so an open one is closed on the way out.
fn closed_ring<const N: usize>(swap: bool, ring: &[[f64; N]]) -> Vec<Vec<f64>> {
    let mut positions: Vec<Vec<f64>> = ring.iter().map(|&c| coordinates(swap, c)).collect();
    if positions.first() != positions.last() {
        let first = positions[0].clone();
        positions.push(first);
    }
    positions
}

/// Whether a frame stores its horizontal axes reflected from canonical
/// `(East, North)` order, so that they must be swapped on the way out.
///
/// Only a CRS declares an axis order to swap back to. `Euclidean` coordinates are
/// east-first by construction, and a `Tangent` frame's are offsets along its own
/// in-plane axes rather than its base CRS's, so neither is reordered — as
/// [`epsg_code`] names no CRS for them either.
///
/// A CRS whose order cannot be established is written as stored, which reverses
/// its coordinates if it turns out to declare `(North, East)`, hence the warning.
fn swaps_axes(frame: &CoordinateFrame) -> bool {
    let Some(code) = epsg_code(frame) else {
        return false;
    };
    match frame.orientation_sign() {
        Ok(sign) => sign < 0,
        Err(error) => {
            warn_unresolved_axis_order(code, error);
            false
        }
    }
}

/// Warn about a CRS whose axis order could not be established, once per code: an
/// axis order is a fixed property of a CRS, so warning per coordinate would repeat
/// the same line for every feature in the file.
fn warn_unresolved_axis_order(code: EpsgCode, error: impl std::fmt::Display) {
    static WARNED: OnceLock<Mutex<HashSet<EpsgCode>>> = OnceLock::new();

    let mut warned = WARNED
        .get_or_init(Mutex::default)
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    if warned.insert(code) {
        tracing::warn!(
            %error,
            "cannot establish the axis order of EPSG:{code}; writing its \
             coordinates in the order they are stored, which reverses them if \
             the CRS declares (northing, easting)"
        );
    }
}

// ---------------------------------------------------------------------------
// Coordinate frames of what gets written
// ---------------------------------------------------------------------------

/// The coordinate frames the positions written for a geometry came from: the
/// first one, and the first one after it that differs.
///
/// Decides whether written parts may fold into a `Multi*` — which needs frame
/// identity, two `Euclidean` parts folding even though neither names a CRS — and
/// answers what the write was expressed in as a [`CrsCoverage`].
#[derive(Clone, Debug, PartialEq)]
enum Frames {
    /// Nothing carrying coordinates was written: the identity of [`Frames::and`].
    Nothing,
    One(CoordinateFrame),
    Mixed {
        first: CoordinateFrame,
        other: CoordinateFrame,
    },
}

impl Frames {
    /// The frames of a leaf: one.
    fn of(frame: &CoordinateFrame) -> Self {
        Self::One(frame.clone())
    }

    /// The frames of two written parts, together. Keeps the first frame written
    /// and the first one after it that differs; further frames add nothing, two
    /// already ruling out both folding and a single CRS.
    fn and(self, other: Self) -> Self {
        match (self, other) {
            (Self::Nothing, frames) | (frames, Self::Nothing) => frames,
            (mixed @ Self::Mixed { .. }, _) => mixed,
            (Self::One(first), Self::Mixed { first: a, other: b }) => {
                let other = if first == a { b } else { a };
                Self::Mixed { first, other }
            }
            (Self::One(first), Self::One(other)) if first != other => Self::Mixed { first, other },
            (one @ Self::One(_), Self::One(_)) => one,
        }
    }

    /// Whether one frame covers every written position.
    fn uniform(&self) -> bool {
        matches!(self, Self::One(_))
    }

    /// How far one CRS covers the written positions.
    fn coverage(&self) -> CrsCoverage {
        match self {
            Self::Nothing => CrsCoverage::NoCoordinates,
            Self::One(frame) => match epsg_code(frame) {
                Some(code) => CrsCoverage::Single(code),
                None => CrsCoverage::OutsideAnyCrs,
            },
            Self::Mixed { first, other } => match (epsg_code(first), epsg_code(other)) {
                (Some(first), Some(other)) => CrsCoverage::Mixed { first, other },
                // A frame naming no CRS is not covered by the code the other
                // carries, so no code covers the pair.
                _ => CrsCoverage::OutsideAnyCrs,
            },
        }
    }
}

/// The EPSG code a frame names, if it names one. `Euclidean` names none, and a
/// `Tangent` plane's in-plane coordinates are not its base CRS's.
fn epsg_code(frame: &CoordinateFrame) -> Option<EpsgCode> {
    match frame {
        CoordinateFrame::Crs(code) => Some(*code),
        CoordinateFrame::Euclidean | CoordinateFrame::Tangent(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use reearth_flow_geometry::{
        collection::{Collection2D, Collection3D},
        coordinate::{BaseFrame, CoordinateFrame, EpsgCode, TangentPlane},
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

    // A Tangent frame's coordinates are offsets along its own in-plane axes, not
    // its base CRS's, so they are written as stored even though that base CRS
    // (EPSG:6675) declares (northing, easting).
    #[test]
    fn tangent_frame_is_written_as_stored() {
        let frame = CoordinateFrame::Tangent(Box::new(TangentPlane {
            base: BaseFrame::Crs(EpsgCode::new(6675)),
            origin: [0.0, 0.0, 0.0],
            u: [1.0, 0.0, 0.0],
            v: [0.0, 1.0, 0.0],
        }));

        let value = written_value(point_2d_in(frame, [1.0, 2.0]));

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

    // A 2.5D leaf's single elevation is not written; its positions stay
    // two-element, as the old world's 2D geometry was.
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

    // Members become features of their own, sharing the feature's properties.
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

    // A mesh writes as its faces, having no GeoJSON geometry of its own.
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

    fn solid_in(frame: CoordinateFrame) -> Euclidean3DGeometry {
        Euclidean3DGeometry::Solid(Box::new(Solid::from_exterior(frame, quad_mesh_data())))
    }

    /// The reason `geometry` cannot be written.
    fn rejection(geometry: Geometry) -> Unwritable {
        write_geometry(&geometry)
            .err()
            .expect("expected an unwritable geometry")
    }

    #[test]
    fn a_solid_is_rejected_rather_than_panicking() {
        assert_eq!(
            rejection(Geometry::Euclidean3D(solid_in(CoordinateFrame::Euclidean))),
            Unwritable::Solid
        );
    }

    // The reason names the leaf, so the writer's warning says what it omitted.
    #[test]
    fn a_csg_tree_and_a_point_cloud_are_rejected_rather_than_panicking() {
        let solid = || Solid::from_exterior(CoordinateFrame::Euclidean, quad_mesh_data());
        for (geometry, reason) in [
            (
                Geometry::Euclidean3D(Euclidean3DGeometry::Csg(Csg::union(solid(), solid()))),
                Unwritable::Csg,
            ),
            (
                Geometry::Euclidean3D(Euclidean3DGeometry::PointCloud(Box::new(
                    PointCloud::from_positions(CoordinateFrame::Euclidean, [[0.0, 0.0, 0.0]]),
                ))),
                Unwritable::PointCloud,
            ),
        ] {
            assert_eq!(rejection(geometry), reason);
        }
    }

    // A reason becomes a message where it leaves the conversion.
    #[test]
    fn a_reason_reaches_the_caller_as_an_error() {
        let feature = feature_with(Geometry::Euclidean3D(solid_in(CoordinateFrame::Euclidean)));

        let result: Result<Vec<geojson::Feature>> = feature.try_into();

        assert_eq!(
            result.unwrap_err().to_string(),
            "Unsupported feature: a Solid has no GeoJSON counterpart"
        );
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

    // Dropping every member leaves nothing to write, reported as the leaf case is —
    // carrying the member's reason out — rather than passing with no features.
    #[test]
    fn a_collection_of_only_unwritable_members_is_rejected() {
        for geometry in [
            Geometry::Euclidean3D(Euclidean3DGeometry::Collection(Collection3D::new([
                solid_in(CoordinateFrame::Euclidean),
            ]))),
            Geometry::GeometryCollection(GeometryCollection::new([Geometry::Euclidean3D(
                solid_in(CoordinateFrame::Euclidean),
            )])),
        ] {
            assert_eq!(rejection(geometry), Unwritable::Solid);
        }
    }

    // An unwritable member writes no empty geometry of its own, and does not take
    // its siblings with it.
    #[test]
    fn an_unwritable_member_of_a_geometry_collection_is_dropped() {
        let geometry = Geometry::GeometryCollection(GeometryCollection::new([
            Geometry::Euclidean3D(Euclidean3DGeometry::Collection(Collection3D::new([
                solid_in(CoordinateFrame::Euclidean),
            ]))),
            point_2d_in(CoordinateFrame::Euclidean, [0.0, 0.0]),
        ]));

        let features = written(geometry);

        assert_eq!(features.len(), 1);
        assert_eq!(
            features[0].geometry.as_ref().map(|g| &g.value),
            Some(&geojson::Value::Point(vec![0.0, 0.0]))
        );
    }

    // --- features without geometry ---

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

    /// How far one CRS covers the coordinates written for `features`, accumulated
    /// as a caller writing them all does it. A feature that writes nothing
    /// contributes no coordinates.
    fn coverage(features: &[Feature]) -> CrsCoverage {
        features
            .iter()
            .map(|feature| {
                write_feature(feature)
                    .map(|written| written.crs)
                    .unwrap_or_default()
            })
            .fold(CrsCoverage::default(), CrsCoverage::and)
    }

    #[test]
    fn coverage_reports_the_shared_epsg_code() {
        let features = [
            feature_with(point_2d_in(crs(6675), [0.0; 2])),
            feature_with(point_2d_in(crs(6675), [1.0; 2])),
        ];

        assert_eq!(
            coverage(&features),
            CrsCoverage::Single(EpsgCode::new(6675))
        );
    }

    #[test]
    fn coverage_reports_differing_epsg_codes() {
        let features = [
            feature_with(point_2d_in(crs(6675), [0.0; 2])),
            feature_with(point_2d_in(crs(6669), [1.0; 2])),
        ];

        assert_eq!(
            coverage(&features),
            CrsCoverage::Mixed {
                first: EpsgCode::new(6675),
                other: EpsgCode::new(6669),
            }
        );
    }

    // Neither code covers what the `Euclidean` frame carries, so naming both would
    // describe the file no better than naming none.
    #[test]
    fn coverage_breaks_when_a_frame_names_no_crs_among_two_that_do() {
        let features = [
            feature_with(point_2d_in(crs(6675), [0.0; 2])),
            feature_with(point_2d_in(CoordinateFrame::Euclidean, [1.0; 2])),
            feature_with(point_2d_in(crs(6669), [2.0; 2])),
        ];

        assert_eq!(coverage(&features), CrsCoverage::OutsideAnyCrs);
    }

    #[test]
    fn coverage_breaks_without_a_crs_frame() {
        let features = [
            feature_with(point_2d_in(CoordinateFrame::Euclidean, [0.0; 2])),
            feature_with(Geometry::None),
        ];

        assert_eq!(coverage(&features), CrsCoverage::OutsideAnyCrs);
    }

    // A geometry that is dropped contributes no coordinates, the answer describing
    // exactly what is in the file — here nothing, rather than a CRS-less write.
    #[test]
    fn coverage_ignores_geometry_that_is_not_written() {
        let features = [feature_with(Geometry::Euclidean3D(
            Euclidean3DGeometry::Solid(Box::new(Solid::from_exterior(
                CoordinateFrame::Crs(EpsgCode::new(6675)),
                quad_mesh_data(),
            ))),
        ))];

        assert_eq!(coverage(&features), CrsCoverage::NoCoordinates);
    }

    // A feature with attributes but no geometry writes a row, which says nothing
    // about the CRS rather than breaking the coverage of its neighbours.
    #[test]
    fn coverage_survives_a_feature_without_geometry() {
        let features = [
            feature_with(point_2d_in(crs(6675), [0.0; 2])),
            feature_with(Geometry::None),
        ];

        assert_eq!(
            coverage(&features),
            CrsCoverage::Single(EpsgCode::new(6675))
        );
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
            coverage(std::slice::from_ref(&feature)),
            CrsCoverage::Single(EpsgCode::new(6675))
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
