//! Shapefile shape conversion. Turns `reearth_flow_geometry::Geometry` (per-leaf
//! `CoordinateFrame`) into the positions a shapefile record holds.
//!
//! A `.shp` holds records of one shape type, so a geometry converts to a
//! [`Payload`] plus the [`Bucket`] naming the file it belongs in; the concrete
//! shape type is settled per file once every feature in it is known.
//!
//! Measures (the `M` channel) have no geometry counterpart and are never written.

use std::collections::HashMap;

use indexmap::IndexMap;
use reearth_flow_geometry::{
    coordinate::CoordinateFrame,
    ops::{Split, UnsupportedOperation},
    polygon_mesh::PolygonMesh3D,
    solid::{Shell, Solid},
    triangular_mesh::TriangularMesh3D,
    Euclidean2DGeometry, Euclidean3DGeometry, Geometry,
};

use reearth_flow_types::{Attribute, AttributeValue};
use shapefile::dbase::{FieldName, FieldValue, Record, TableWriterBuilder};

use super::shape::{epsg_code, Frames, Payload, Ring, WrittenShape};

/// What `geometry` writes to. A part with no shapefile counterpart is dropped
/// where it appears and warned about rather than failing the geometry around it;
/// a geometry left with nothing writes no shape at all, so its feature still
/// reaches the attribute table.
pub(super) fn write_geometry(geometry: &Geometry) -> WrittenShape {
    match write(geometry) {
        Ok(written) => {
            warn_omitted(&written.omitted);
            WrittenShape {
                payload: Some(written.payload),
                elevated: written.elevated,
                frames: written.frames,
            }
        }
        Err(reason) => {
            if !matches!(reason, Unwritable::AbsentGeometry) {
                warn_omitted(&[reason]);
            }
            WrittenShape::none()
        }
    }
}

/// Why a geometry cannot be written to a shapefile.
#[derive(thiserror::Error, Clone, Copy, Debug, PartialEq, Eq)]
enum Unwritable {
    #[error("an absent geometry writes no shape")]
    AbsentGeometry,
    /// A shapefile record holds one kind of shape, so parts of different kinds
    /// cannot be written together.
    #[error("a geometry mixing points, curves and areas has no shapefile counterpart")]
    MixedKinds,
    /// A `MultipointZ` could hold one, but that would write a record per sample.
    #[error("a PointCloud has no shapefile counterpart")]
    PointCloud,
    #[error("a Csg tree has no shapefile counterpart")]
    Csg,
    #[error("an empty collection has no shapefile counterpart")]
    EmptyCollection,
    #[error("a mesh with no face has no shapefile counterpart")]
    EmptyMesh,
    #[error("a solid with no boundary face has no shapefile counterpart")]
    EmptySolid,
    #[error(transparent)]
    Unsplittable(#[from] UnsupportedOperation),
}

/// Report what the writer left out.
fn warn_omitted(omitted: &[Unwritable]) {
    for reason in omitted {
        tracing::warn!(%reason, "omitting a geometry from the shapefile output");
    }
}

/// What a geometry writes to, with the frames its positions came from and the
/// reasons parts of it were left out.
struct Written {
    payload: Payload,
    elevated: bool,
    frames: Frames,
    omitted: Vec<Unwritable>,
}

impl Written {
    /// What a leaf writes to: one payload, in one frame, leaving nothing out.
    fn leaf(frame: &CoordinateFrame, elevated: bool, payload: Payload) -> Self {
        Self {
            payload,
            elevated,
            frames: Frames::of(frame),
            omitted: Vec::new(),
        }
    }
}

fn write(geometry: &Geometry) -> Result<Written, Unwritable> {
    match geometry {
        Geometry::None => Err(Unwritable::AbsentGeometry),
        Geometry::Euclidean2D(g) => write_2d(g),
        Geometry::Euclidean3D(g) => write_3d(g),
        Geometry::GeometryCollection(c) => merge(c.members().iter().map(write)),
    }
}

fn write_2d(geometry: &Euclidean2DGeometry) -> Result<Written, Unwritable> {
    use Euclidean2DGeometry::*;
    match geometry {
        Point(p) => {
            let [x, y] = swapped(p.frame(), p.position());
            Ok(Written::leaf(
                p.frame(),
                false,
                Payload::Points(vec![[x, y, 0.0]]),
            ))
        }
        LineString(l) => {
            let z = l.elevation();
            Ok(Written::leaf(
                l.frame(),
                z.is_some(),
                Payload::Curve(vec![raise(l.frame(), l.coords(), z)]),
            ))
        }
        Polygon(p) => {
            let z = p.elevation();
            Ok(Written::leaf(
                p.frame(),
                z.is_some(),
                Payload::Area(rings(p.frame(), p.exterior(), p.interiors(), z)),
            ))
        }
        PolygonMesh(m) => write_faces((**m).clone()),
        TriangularMesh(m) => write_faces((**m).clone()),
        Collection(c) => merge(c.members().iter().map(write_2d)),
    }
}

fn write_3d(geometry: &Euclidean3DGeometry) -> Result<Written, Unwritable> {
    use Euclidean3DGeometry::*;
    match geometry {
        Point(p) => Ok(Written::leaf(
            p.frame(),
            true,
            Payload::Points(vec![swapped(p.frame(), p.position())]),
        )),
        LineString(l) => Ok(Written::leaf(
            l.frame(),
            true,
            Payload::Curve(vec![swap_all(l.frame(), l.coords())]),
        )),
        Polygon(p) => Ok(Written::leaf(
            p.frame(),
            true,
            Payload::Area(rings(p.frame(), p.exterior(), p.interiors(), None)),
        )),
        PolygonMesh(m) => write_faces((**m).clone()),
        TriangularMesh(m) => write_faces((**m).clone()),
        Solid(s) => write_solid(s),
        Collection(c) => merge(c.members().iter().map(write_3d)),
        Csg(_) => Err(Unwritable::Csg),
        PointCloud(_) => Err(Unwritable::PointCloud),
    }
}

/// A mesh writes as its faces, the way a collection writes as its members.
fn write_faces(mut mesh: impl Split) -> Result<Written, Unwritable> {
    let mut faces = Vec::new();
    mesh.split(&mut |face, _| faces.push(face))?;
    if faces.is_empty() {
        return Err(Unwritable::EmptyMesh);
    }
    merge(faces.iter().map(write))
}

/// A solid writes as the faces of its shells, exterior first. The distinction
/// between bounding a volume and bounding a void is not preserved: a shapefile
/// area has no counterpart to it.
fn write_solid(solid: &Solid) -> Result<Written, Unwritable> {
    let shells = std::iter::once(solid.exterior()).chain(solid.interiors());
    let written: Vec<Result<Written, Unwritable>> = shells
        .map(|shell| write_shell(solid.frame(), shell))
        .collect();
    if written.is_empty() {
        return Err(Unwritable::EmptySolid);
    }
    merge(written.into_iter())
}

/// A shell's faces, taking the frame the enclosing solid states.
fn write_shell(frame: &CoordinateFrame, shell: &Shell) -> Result<Written, Unwritable> {
    match shell {
        Shell::PolygonMesh(data) => write_faces(PolygonMesh3D::new(frame.clone(), data.clone())),
        Shell::TriangularMesh(data) => {
            write_faces(TriangularMesh3D::new(frame.clone(), data.clone()))
        }
    }
}

/// The parts written for a container, as one shape.
///
/// Parts of different kinds cannot share a record, so they are unwritable
/// together. Parts that differ only in whether they carry an elevation are
/// written as one elevated shape, those carrying none lying at `0.0` as they do
/// when a 2D geometry is read in three dimensions.
///
/// `Err` once nothing is left, a container reduced to nothing being unwritable
/// too. That error carries a dropped part's reason, or `EmptyCollection` when
/// there were no parts to drop.
fn merge(parts: impl Iterator<Item = Result<Written, Unwritable>>) -> Result<Written, Unwritable> {
    let mut merged: Option<Written> = None;
    let mut omitted = Vec::new();

    for part in parts {
        let mut part = match part {
            Ok(part) => part,
            Err(reason) => {
                omitted.push(reason);
                continue;
            }
        };
        omitted.append(&mut part.omitted);
        match &mut merged {
            None => merged = Some(part),
            Some(into) => {
                if !into.payload.same_kind(&part.payload) {
                    omitted.push(Unwritable::MixedKinds);
                    continue;
                }
                into.payload.absorb(part.payload);
                into.elevated |= part.elevated;
                into.frames = std::mem::replace(&mut into.frames, Frames::Nothing).and(part.frames);
            }
        }
    }

    match merged {
        Some(mut written) => {
            written.omitted.append(&mut omitted);
            Ok(written)
        }
        None => Err(omitted
            .first()
            .copied()
            .unwrap_or(Unwritable::EmptyCollection)),
    }
}

// A shapefile's (x, y) is (easting, northing) whatever the CRS declares, so a
// frame declaring the reverse has its horizontal pair swapped on the way out.

/// A stored coordinate as a shapefile position, its height left where it is.
fn swapped<const N: usize>(frame: &CoordinateFrame, coordinate: [f64; N]) -> [f64; N] {
    let mut coordinate = coordinate;
    if swaps_axes(frame) {
        coordinate.swap(0, 1);
    }
    coordinate
}

fn swap_all(frame: &CoordinateFrame, coords: &[[f64; 3]]) -> Vec<[f64; 3]> {
    let swap = swaps_axes(frame);
    coords
        .iter()
        .map(|&c| if swap { [c[1], c[0], c[2]] } else { c })
        .collect()
}

/// 2D coordinates as shapefile positions, at the elevation their geometry lies at
/// or `0.0` where it states none.
fn raise(frame: &CoordinateFrame, coords: &[[f64; 2]], z: Option<f64>) -> Vec<[f64; 3]> {
    let swap = swaps_axes(frame);
    let z = z.unwrap_or(0.0);
    coords
        .iter()
        .map(|&[x, y]| if swap { [y, x, z] } else { [x, y, z] })
        .collect()
}

/// A face's rings: its exterior, then its holes.
///
/// The generic `N` covers both embeddings: `z` supplies the elevation for 2D
/// coordinates and is `None` for 3D ones, which carry their own.
fn rings<'a, const N: usize>(
    frame: &CoordinateFrame,
    exterior: &'a [[f64; N]],
    interiors: impl Iterator<Item = &'a [[f64; N]]>,
    z: Option<f64>,
) -> Vec<Ring> {
    std::iter::once((true, exterior))
        .chain(interiors.map(|hole| (false, hole)))
        .map(|(outer, ring)| Ring {
            outer,
            coords: ring
                .iter()
                .map(|coordinate| {
                    let coordinate = swapped(frame, *coordinate);
                    match N {
                        2 => [coordinate[0], coordinate[1], z.unwrap_or(0.0)],
                        _ => [coordinate[0], coordinate[1], coordinate[2]],
                    }
                })
                .collect(),
        })
        .collect()
}

/// Whether a frame stores its horizontal axes reflected from `(East, North)`.
///
/// Only a CRS declares an axis order. A CRS whose order cannot be established is
/// written as stored, which reverses its coordinates if it declares
/// `(northing, easting)`.
fn swaps_axes(frame: &CoordinateFrame) -> bool {
    if epsg_code(frame).is_none() {
        return false;
    }
    frame
        .orientation_sign()
        .map(|sign| sign < 0)
        .unwrap_or(false)
}

// ---------------------------------------------------------------------------
// Attribute table
// ---------------------------------------------------------------------------

/// One column of the DBF table: the attribute it takes its value from, and the
/// value to write where that attribute has none.
///
/// A DBF field name is at most 11 bytes, so it can differ from the attribute's;
/// the field is keyed by its own name and remembers the attribute's.
pub(super) struct Field {
    attribute: Attribute,
    default: FieldValue,
}

/// The table to write `attributes` into, and its columns keyed by DBF field name.
///
/// `attributes` is the union of what the features carry, so a field only some of
/// them have still gets a column.
pub(super) fn make_table_builder(
    attributes: &IndexMap<Attribute, AttributeValue>,
) -> crate::errors::Result<(TableWriterBuilder, HashMap<String, Field>)> {
    let mut builder = TableWriterBuilder::new();
    let mut fields = HashMap::new();

    for (attribute, value) in attributes {
        let key = trim_string_bytes(attribute.to_string(), 11);
        let name: FieldName = key.as_str().try_into().map_err(|e| {
            crate::errors::SinkError::ShapefileWriter(format!(
                "Failed to convert field name to FieldName: {e}"
            ))
        })?;

        let default = match value {
            AttributeValue::String(_) => {
                builder = builder.add_character_field(name, 255);
                FieldValue::Character(None)
            }
            AttributeValue::Number(num) => {
                builder = if num.is_i64() {
                    builder.add_numeric_field(name, 11, 0)
                } else {
                    builder.add_numeric_field(name, 18, 6)
                };
                FieldValue::Numeric(None)
            }
            AttributeValue::Bool(_) => {
                builder = builder.add_character_field(name, 6);
                FieldValue::Character(None)
            }
            AttributeValue::DateTime(_) => {
                builder = builder.add_character_field(name, 255);
                FieldValue::Character(None)
            }
            AttributeValue::Null
            | AttributeValue::Array(_)
            | AttributeValue::Map(_)
            | AttributeValue::Bytes(_) => continue,
        };
        fields.insert(
            key,
            Field {
                attribute: attribute.clone(),
                default,
            },
        );
    }
    Ok((builder, fields))
}

/// The DBF record for one feature: exactly the table's fields, taking each from
/// `attributes` where it holds a value the table can store and from the field's
/// default otherwise.
///
/// Every field the table declares must appear, so a feature missing one, or
/// carrying a value with no DBF counterpart, still writes the field as its
/// default.
pub(super) fn attributes_to_record(
    attributes: &IndexMap<Attribute, AttributeValue>,
    fields: &HashMap<String, Field>,
) -> Record {
    let mut record = Record::default();
    for (name, field) in fields {
        let value = attributes
            .get(&field.attribute)
            .and_then(to_field_value)
            .unwrap_or_else(|| field.default.clone());
        record.insert(name.to_string(), value);
    }
    record
}

/// The DBF value an attribute writes as, or `None` for one the table cannot store.
fn to_field_value(value: &AttributeValue) -> Option<FieldValue> {
    match value {
        // Shapefile cannot store a string longer than 254 bytes.
        AttributeValue::String(s) => Some(FieldValue::Character(Some(trim_string_bytes(
            s.clone(),
            254,
        )))),
        AttributeValue::Number(num) => Some(FieldValue::Numeric(num.as_f64())),
        AttributeValue::Bool(b) => Some(FieldValue::Character(Some(b.to_string()))),
        AttributeValue::DateTime(d) => Some(FieldValue::Character(Some(d.to_rfc3339()))),
        AttributeValue::Null
        | AttributeValue::Array(_)
        | AttributeValue::Map(_)
        | AttributeValue::Bytes(_) => None,
    }
}

/// `s` cut to at most `n` bytes, never through the middle of a character.
fn trim_string_bytes(s: String, n: usize) -> String {
    let bytes = s.as_bytes();
    if bytes.len() <= n {
        return s;
    }
    match std::str::from_utf8(&bytes[..n]) {
        Ok(valid) => valid.to_string(),
        Err(e) => std::str::from_utf8(&bytes[..e.valid_up_to()])
            .expect("the prefix up to the first invalid byte is valid UTF-8")
            .to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::file::shapefile_next::shape::Bucket;
    use pretty_assertions::assert_eq;
    use reearth_flow_geometry::{
        collection::{Collection2D, Collection3D},
        coordinate::EpsgCode,
        line_string::{LineString2D, LineString3D},
        point::{Point2D, Point3D},
        polygon::{Polygon2D, Polygon3D},
    };

    fn euclidean() -> CoordinateFrame {
        CoordinateFrame::Euclidean
    }

    fn crs(code: u16) -> CoordinateFrame {
        CoordinateFrame::Crs(EpsgCode::new(code))
    }

    fn positions(written: &WrittenShape) -> &Vec<[f64; 3]> {
        match written.payload.as_ref().expect("expected a shape") {
            Payload::Points(points) => points,
            _ => panic!("expected points"),
        }
    }

    fn parts(written: &WrittenShape) -> &Vec<Vec<[f64; 3]>> {
        match written.payload.as_ref().expect("expected a shape") {
            Payload::Curve(parts) => parts,
            _ => panic!("expected a curve"),
        }
    }

    fn area(written: &WrittenShape) -> &Vec<Ring> {
        match written.payload.as_ref().expect("expected a shape") {
            Payload::Area(rings) => rings,
            _ => panic!("expected an area"),
        }
    }

    #[test]
    fn an_absent_geometry_writes_no_shape() {
        let written = write_geometry(&Geometry::None);
        assert_eq!(written.bucket(), Bucket::Null);
        assert!(written.payload.is_none());
    }

    // A 2D geometry stating the height it lies at is written as an elevated shape
    // rather than losing that height.
    #[test]
    fn a_2d_line_at_an_elevation_writes_an_elevated_curve() {
        let line = Geometry::Euclidean2D(Euclidean2DGeometry::LineString(
            LineString2D::from_coords_at_elevation(euclidean(), [[0.0, 0.0], [1.0, 1.0]], 9.0),
        ));
        let written = write_geometry(&line);
        assert_eq!(written.bucket(), Bucket::CurveZ);
        assert_eq!(
            parts(&written),
            &vec![vec![[0.0, 0.0, 9.0], [1.0, 1.0, 9.0]]]
        );
    }

    // The old writer joined a multi-part curve's parts into one chain; the parts
    // must stay apart.
    #[test]
    fn a_collection_of_lines_keeps_its_parts_apart() {
        let collection =
            Geometry::Euclidean3D(Euclidean3DGeometry::Collection(Collection3D::new([
                Euclidean3DGeometry::LineString(LineString3D::from_coords(
                    euclidean(),
                    [[0.0, 0.0, 0.0], [1.0, 1.0, 0.0]],
                )),
                Euclidean3DGeometry::LineString(LineString3D::from_coords(
                    euclidean(),
                    [[5.0, 5.0, 0.0], [6.0, 6.0, 0.0]],
                )),
            ])));
        let written = write_geometry(&collection);
        assert_eq!(written.bucket(), Bucket::CurveZ);
        assert_eq!(parts(&written).len(), 2);
    }

    #[test]
    fn a_polygon_writes_its_exterior_before_its_holes() {
        let polygon = Geometry::Euclidean3D(Euclidean3DGeometry::Polygon(Box::new(
            Polygon3D::from_rings(
                euclidean(),
                [
                    [0.0, 0.0, 0.0],
                    [0.0, 4.0, 0.0],
                    [4.0, 4.0, 0.0],
                    [0.0, 0.0, 0.0],
                ],
                [vec![
                    [1.0, 1.0, 0.0],
                    [2.0, 1.0, 0.0],
                    [2.0, 2.0, 0.0],
                    [1.0, 1.0, 0.0],
                ]],
            ),
        )));
        let written = write_geometry(&polygon);
        assert_eq!(written.bucket(), Bucket::AreaZ);
        let rings = area(&written);
        assert_eq!(rings.len(), 2);
        assert!(rings[0].outer);
        assert!(!rings[1].outer);
    }

    // A collection whose members are of different kinds cannot be one record, so
    // the kind that got there first is written and the rest are dropped.
    #[test]
    fn a_collection_mixing_kinds_writes_only_one_of_them() {
        let collection =
            Geometry::Euclidean2D(Euclidean2DGeometry::Collection(Collection2D::new([
                Euclidean2DGeometry::Point(Point2D::new(euclidean(), [1.0, 2.0])),
                Euclidean2DGeometry::LineString(LineString2D::from_coords(
                    euclidean(),
                    [[0.0, 0.0], [1.0, 1.0]],
                )),
            ])));
        let written = write_geometry(&collection);
        assert_eq!(written.bucket(), Bucket::Point);
        assert_eq!(positions(&written).len(), 1);
    }

    // Members lying at a height and members stating none write as one elevated
    // shape, the latter at 0.0.
    #[test]
    fn a_collection_mixing_elevations_writes_one_elevated_shape() {
        let collection =
            Geometry::Euclidean2D(Euclidean2DGeometry::Collection(Collection2D::new([
                Euclidean2DGeometry::LineString(LineString2D::from_coords(
                    euclidean(),
                    [[0.0, 0.0], [1.0, 1.0]],
                )),
                Euclidean2DGeometry::LineString(LineString2D::from_coords_at_elevation(
                    euclidean(),
                    [[5.0, 5.0], [6.0, 6.0]],
                    9.0,
                )),
            ])));
        let written = write_geometry(&collection);
        assert_eq!(written.bucket(), Bucket::CurveZ);
        assert_eq!(parts(&written)[0][0], [0.0, 0.0, 0.0]);
        assert_eq!(parts(&written)[1][0], [5.0, 5.0, 9.0]);
    }

    // A CRS declaring (northing, easting) has its pair swapped back on the way out,
    // shapefile positions always being easting-first.
    #[test]
    fn a_northing_first_crs_swaps_the_horizontal_pair_back() {
        let point = Geometry::Euclidean2D(Euclidean2DGeometry::Point(Point2D::new(
            crs(6668),
            [35.0, 139.0],
        )));
        let written = write_geometry(&point);
        assert_eq!(positions(&written), &vec![[139.0, 35.0, 0.0]]);
    }

    #[test]
    fn one_crs_covers_a_file_written_wholly_in_it() {
        let point = Geometry::Euclidean2D(Euclidean2DGeometry::Point(Point2D::new(
            crs(6677),
            [1.0, 2.0],
        )));
        assert_eq!(
            write_geometry(&point).frames.epsg(),
            Some(EpsgCode::new(6677))
        );
    }

    #[test]
    fn no_crs_covers_a_geometry_whose_members_disagree() {
        let collection =
            Geometry::Euclidean2D(Euclidean2DGeometry::Collection(Collection2D::new([
                Euclidean2DGeometry::Point(Point2D::new(crs(6677), [1.0, 2.0])),
                Euclidean2DGeometry::Point(Point2D::new(crs(6668), [1.0, 2.0])),
            ])));
        assert_eq!(write_geometry(&collection).frames.epsg(), None);
    }
}
