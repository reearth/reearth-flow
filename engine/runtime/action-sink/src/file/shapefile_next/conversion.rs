//! Shapefile shape conversion. Turns `reearth_flow_geometry::Geometry` (per-leaf
//! `CoordinateFrame`) into the positions a shapefile record holds.
//!
//! A `.shp` holds records of one shape type, so a geometry converts to a
//! [`Payload`] plus the [`Bucket`] naming the file it belongs in; the concrete
//! shape type is settled per file once every feature in it is known.
//!
//! A geometry winds a face's exterior counter-clockwise and its holes clockwise,
//! where a shapefile winds them the other way round, so every ring is reversed on
//! the way out.
//!
//! Measures (the `M` channel) have no geometry counterpart and are never written.

use std::collections::HashSet;

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
    /// A shapefile part holds at least two positions.
    #[error("a curve with fewer than two positions has no shapefile counterpart")]
    DegenerateCurve,
    /// A shapefile ring holds at least three distinct positions.
    #[error("a ring with fewer than three positions has no shapefile counterpart")]
    DegenerateRing,
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

    /// What a face writes to: its rings, and the holes it left out.
    fn face(frame: &CoordinateFrame, elevated: bool, area: Area) -> Self {
        Self {
            payload: Payload::Area(area.rings),
            elevated,
            frames: Frames::of(frame),
            omitted: area.omitted,
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
                Payload::Curve(vec![curve(l.frame(), l.coords(), z)?]),
            ))
        }
        Polygon(p) => {
            let z = p.elevation();
            Ok(Written::face(
                p.frame(),
                z.is_some(),
                rings(p.frame(), p.exterior(), p.interiors(), z)?,
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
            Payload::Curve(vec![curve(l.frame(), l.coords(), None)?]),
        )),
        Polygon(p) => Ok(Written::face(
            p.frame(),
            true,
            rings(p.frame(), p.exterior(), p.interiors(), None)?,
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

/// Stored coordinates as shapefile positions, their horizontal pair swapped when
/// `swap` says so, at the elevation `z` states or, when it states none, the one
/// each coordinate carries (`0.0` for a 2D coordinate).
///
/// The generic `N` covers both embeddings: `z` supplies the elevation for 2D
/// coordinates and is `None` for 3D ones, which carry their own.
fn positions<'a, const N: usize>(
    swap: bool,
    coords: impl Iterator<Item = &'a [f64; N]>,
    z: Option<f64>,
) -> Vec<[f64; 3]> {
    coords
        .map(|&coordinate| {
            let [x, y] = if swap {
                [coordinate[1], coordinate[0]]
            } else {
                [coordinate[0], coordinate[1]]
            };
            let z = z.unwrap_or_else(|| if N > 2 { coordinate[2] } else { 0.0 });
            [x, y, z]
        })
        .collect()
}

/// A line's positions as one shapefile part.
///
/// Errors on a line too short to be a part.
fn curve<const N: usize>(
    frame: &CoordinateFrame,
    coords: &[[f64; N]],
    z: Option<f64>,
) -> Result<Vec<[f64; 3]>, Unwritable> {
    if coords.len() < 2 {
        return Err(Unwritable::DegenerateCurve);
    }
    Ok(positions(swaps_axes(frame), coords.iter(), z))
}

/// A face's rings as shapefile rings, and the holes left out for being too short.
struct Area {
    rings: Vec<Ring>,
    omitted: Vec<Unwritable>,
}

/// A face's rings: its exterior, then its holes, each reversed into the winding a
/// shapefile expects.
///
/// Errors on a face whose exterior is too short to be a ring; a hole too short to
/// be one is left out.
fn rings<'a, const N: usize>(
    frame: &CoordinateFrame,
    exterior: &'a [[f64; N]],
    interiors: impl Iterator<Item = &'a [[f64; N]]>,
    z: Option<f64>,
) -> Result<Area, Unwritable> {
    if !is_ring(exterior) {
        return Err(Unwritable::DegenerateRing);
    }
    let swap = swaps_axes(frame);
    let mut area = Area {
        rings: vec![Ring {
            outer: true,
            coords: positions(swap, exterior.iter().rev(), z),
        }],
        omitted: Vec::new(),
    };
    for hole in interiors {
        if !is_ring(hole) {
            area.omitted.push(Unwritable::DegenerateRing);
            continue;
        }
        area.rings.push(Ring {
            outer: false,
            coords: positions(swap, hole.iter().rev(), z),
        });
    }
    Ok(area)
}

/// Whether `coords` can be a shapefile ring: three positions and the closing one,
/// which is supplied on writing when it is missing.
fn is_ring<const N: usize>(coords: &[[f64; N]]) -> bool {
    match coords.len() {
        0..=2 => false,
        3 => coords[0] != coords[2],
        _ => true,
    }
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

/// The longest DBF field name, in bytes.
const FIELD_NAME_BYTES: usize = 11;
/// The longest DBF character field, in bytes.
const CHARACTER_BYTES: usize = 254;
/// The width a numeric field is declared with when its values need no more.
const INTEGER_WIDTH: usize = 11;
const DECIMAL_WIDTH: usize = 18;
/// The decimal places a numeric field carrying non-integers is declared with.
const DECIMAL_PLACES: u8 = 6;

/// One column of the DBF table: the attribute it takes its value from, and the
/// type every value is written as.
///
/// A DBF field name is at most 11 bytes, so it can differ from the attribute's;
/// the field is keyed by its own name and remembers the attribute's.
pub(super) struct Field {
    name: String,
    attribute: Attribute,
    kind: FieldKind,
}

/// The DBF type a column is declared as.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FieldKind {
    Character,
    /// A number with `decimals` places after the point.
    Numeric {
        decimals: u8,
    },
}

impl Field {
    /// The value this column writes where the feature carries none it can store.
    fn default(&self) -> FieldValue {
        match self.kind {
            FieldKind::Character => FieldValue::Character(None),
            FieldKind::Numeric { .. } => FieldValue::Numeric(None),
        }
    }
}

/// What the values of one attribute call for in a column: whether they are all
/// numbers, whether all integers, and the widest one written out.
struct Column {
    attribute: Attribute,
    numeric: bool,
    integral: bool,
    width: usize,
}

impl Column {
    fn new(attribute: &Attribute) -> Self {
        Self {
            attribute: attribute.clone(),
            numeric: true,
            integral: true,
            width: 0,
        }
    }

    /// Take a value into account. A value the table cannot store says nothing
    /// about the column.
    fn note(&mut self, value: &AttributeValue) {
        let width = match value {
            AttributeValue::String(s) => {
                self.numeric = false;
                s.len()
            }
            AttributeValue::Number(n) if n.is_f64() => {
                self.integral = false;
                format!("{:.*}", DECIMAL_PLACES as usize, n.as_f64().unwrap_or(0.0)).len()
            }
            AttributeValue::Number(n) => n.to_string().len(),
            AttributeValue::Bool(b) => {
                self.numeric = false;
                b.to_string().len()
            }
            AttributeValue::DateTime(d) => {
                self.numeric = false;
                d.to_rfc3339().len()
            }
            AttributeValue::Null
            | AttributeValue::Array(_)
            | AttributeValue::Map(_)
            | AttributeValue::Bytes(_) => return,
        };
        self.width = self.width.max(width);
    }

    /// The type the column is declared as, and its width, once every value is
    /// taken into account. Integers widen to what the widest needs; a decimal
    /// widens to fit its integer part at [`DECIMAL_PLACES`] places.
    fn kind(&self) -> (FieldKind, u8) {
        let (kind, width) = match (self.numeric, self.integral) {
            (true, true) => (
                FieldKind::Numeric { decimals: 0 },
                self.width.max(INTEGER_WIDTH),
            ),
            (true, false) => (
                FieldKind::Numeric {
                    decimals: DECIMAL_PLACES,
                },
                self.width.max(DECIMAL_WIDTH),
            ),
            (false, _) => (FieldKind::Character, self.width.clamp(1, CHARACTER_BYTES)),
        };
        (kind, width.min(u8::MAX as usize) as u8)
    }
}

/// The table to write the attributes of `features` into, and its columns.
///
/// A column is declared for every attribute any feature carries a storable value
/// for, in the order first seen: one holding only numbers is numeric, wide enough
/// for the widest, and one holding anything else is character. Names are cut to
/// the DBF limit and told apart where the cut makes two the same.
pub(super) fn make_table_builder<'a>(
    features: impl Iterator<Item = &'a IndexMap<Attribute, AttributeValue>>,
) -> crate::errors::Result<(TableWriterBuilder, Vec<Field>)> {
    let mut columns: IndexMap<&Attribute, Column> = IndexMap::new();
    for attributes in features {
        for (attribute, value) in attributes {
            columns
                .entry(attribute)
                .or_insert_with(|| Column::new(attribute))
                .note(value);
        }
    }

    let mut builder = TableWriterBuilder::new();
    let mut fields = Vec::new();
    let mut taken = HashSet::new();
    for column in columns.values().filter(|column| column.width > 0) {
        let name = field_name(column.attribute.as_ref(), &taken);
        taken.insert(name.clone());
        let field_name: FieldName = name.as_str().try_into().map_err(|e| {
            crate::errors::SinkError::ShapefileWriter(format!(
                "Failed to convert field name to FieldName: {e}"
            ))
        })?;
        let (kind, width) = column.kind();
        builder = match kind {
            FieldKind::Character => builder.add_character_field(field_name, width),
            FieldKind::Numeric { decimals } => {
                builder.add_numeric_field(field_name, width, decimals)
            }
        };
        fields.push(Field {
            name,
            attribute: column.attribute.clone(),
            kind,
        });
    }
    Ok((builder, fields))
}

/// `name` cut to the DBF limit and, where that cut is already `taken`, made
/// distinct with a counter that fits within the limit.
fn field_name(name: &str, taken: &HashSet<String>) -> String {
    let cut = trim_string_bytes(name.to_string(), FIELD_NAME_BYTES);
    if !taken.contains(&cut) {
        return cut;
    }
    (1..)
        .map(|n| {
            let suffix = format!("_{n}");
            let head = trim_string_bytes(name.to_string(), FIELD_NAME_BYTES - suffix.len());
            format!("{head}{suffix}")
        })
        .find(|candidate| !taken.contains(candidate))
        .expect("the counter is unbounded, so some name is free")
}

/// The DBF record for one feature: exactly the table's fields, taking each from
/// `attributes` where it holds a value the column can store and from the column's
/// default otherwise.
///
/// Every field the table declares must appear, so a feature missing one, or
/// carrying a value with no DBF counterpart, still writes the field as its
/// default.
pub(super) fn attributes_to_record(
    attributes: &IndexMap<Attribute, AttributeValue>,
    fields: &[Field],
) -> Record {
    let mut record = Record::default();
    for field in fields {
        let value = attributes
            .get(&field.attribute)
            .and_then(|value| to_field_value(value, field.kind))
            .unwrap_or_else(|| field.default());
        record.insert(field.name.clone(), value);
    }
    record
}

/// The value an attribute writes as in a column of `kind`, or `None` for one the
/// column cannot store.
fn to_field_value(value: &AttributeValue, kind: FieldKind) -> Option<FieldValue> {
    match kind {
        FieldKind::Numeric { .. } => match value {
            AttributeValue::Number(num) => Some(FieldValue::Numeric(num.as_f64())),
            _ => None,
        },
        FieldKind::Character => {
            let text = match value {
                AttributeValue::String(s) => s.clone(),
                AttributeValue::Number(num) => num.to_string(),
                AttributeValue::Bool(b) => b.to_string(),
                AttributeValue::DateTime(d) => d.to_rfc3339(),
                AttributeValue::Null
                | AttributeValue::Array(_)
                | AttributeValue::Map(_)
                | AttributeValue::Bytes(_) => return None,
            };
            Some(FieldValue::Character(Some(trim_string_bytes(
                text,
                CHARACTER_BYTES,
            ))))
        }
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

    // A geometry winds its exterior counter-clockwise; the shapefile needs it
    // clockwise, and its holes the other way round.
    #[test]
    fn rings_are_reversed_into_the_shapefile_winding() {
        let polygon = Geometry::Euclidean2D(Euclidean2DGeometry::Polygon(Box::new(
            Polygon2D::from_rings(
                euclidean(),
                [[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 4.0], [0.0, 0.0]],
                [vec![[1.0, 1.0], [1.0, 2.0], [2.0, 2.0], [1.0, 1.0]]],
            ),
        )));
        let written = write_geometry(&polygon);
        let rings = area(&written);
        assert!(signed_area(&rings[0].coords) < 0.0);
        assert!(signed_area(&rings[1].coords) > 0.0);
    }

    /// Twice the shoelace area: positive for a counter-clockwise ring.
    fn signed_area(ring: &[[f64; 3]]) -> f64 {
        ring.windows(2)
            .map(|w| w[0][0] * w[1][1] - w[1][0] * w[0][1])
            .sum()
    }

    // A one-position line is no shapefile part; it writes no shape rather than
    // failing the file.
    #[test]
    fn a_degenerate_line_writes_no_shape() {
        let line = Geometry::Euclidean2D(Euclidean2DGeometry::LineString(
            LineString2D::from_coords(euclidean(), [[0.0, 0.0]]),
        ));
        assert!(write_geometry(&line).payload.is_none());
    }

    // A hole too short to be a ring is left out; the face around it is still written.
    #[test]
    fn a_degenerate_hole_is_left_out() {
        let polygon = Geometry::Euclidean2D(Euclidean2DGeometry::Polygon(Box::new(
            Polygon2D::from_rings(
                euclidean(),
                [[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 4.0], [0.0, 0.0]],
                [vec![[1.0, 1.0], [2.0, 2.0]]],
            ),
        )));
        assert_eq!(area(&write_geometry(&polygon)).len(), 1);
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

    fn attributes(pairs: &[(&str, AttributeValue)]) -> IndexMap<Attribute, AttributeValue> {
        pairs
            .iter()
            .map(|(k, v)| (Attribute::new(*k), v.clone()))
            .collect()
    }

    fn table(features: &[&IndexMap<Attribute, AttributeValue>]) -> Vec<Field> {
        make_table_builder(features.iter().copied())
            .expect("the table is expected to build")
            .1
    }

    // Two attributes cut to the same DBF name must still be two columns.
    #[test]
    fn attributes_cut_to_the_same_name_get_distinct_fields() {
        let feature = attributes(&[
            ("bldg:measuredHeight", AttributeValue::Number(10.into())),
            (
                "bldg:measuredHeight_uom",
                AttributeValue::String("m".into()),
            ),
        ]);
        let fields = table(&[&feature]);
        assert_eq!(fields.len(), 2);
        assert_eq!(fields[0].name, "bldg:measur");
        assert_eq!(fields[1].name, "bldg:meas_1");
        let record = attributes_to_record(&feature, &fields);
        assert_eq!(
            record.get("bldg:measur"),
            Some(&FieldValue::Numeric(Some(10.0)))
        );
        assert_eq!(
            record.get("bldg:meas_1"),
            Some(&FieldValue::Character(Some("m".into())))
        );
    }

    // Taking the table from the first feature alone would leave out a field only
    // later features carry.
    #[test]
    fn the_table_covers_fields_a_later_feature_introduces() {
        let first = attributes(&[("a", AttributeValue::String("x".into()))]);
        let second = attributes(&[("b", AttributeValue::Bool(true))]);
        let fields = table(&[&first, &second]);
        assert_eq!(fields.len(), 2);
    }

    // A column is numeric only when every value in it is a number; a value of
    // another type is written as text alongside the numbers.
    #[test]
    fn a_column_mixing_numbers_and_text_writes_everything_as_text() {
        let first = attributes(&[("code", AttributeValue::Number(5.into()))]);
        let second = attributes(&[("code", AttributeValue::String("A1".into()))]);
        let fields = table(&[&first, &second]);
        assert_eq!(fields[0].kind, FieldKind::Character);
        assert_eq!(
            attributes_to_record(&first, &fields).get("code"),
            Some(&FieldValue::Character(Some("5".into())))
        );
    }

    // A null says nothing about a column's type; a later value settles it.
    #[test]
    fn a_null_first_value_takes_its_type_from_a_later_feature() {
        let first = attributes(&[("a", AttributeValue::Null)]);
        let second = attributes(&[("a", AttributeValue::Number(1.into()))]);
        let fields = table(&[&first, &second]);
        assert_eq!(fields[0].kind, FieldKind::Numeric { decimals: 0 });
    }

    // An integer wider than the default width is not cut short.
    #[test]
    fn a_wide_integer_widens_its_column() {
        let mut column = Column::new(&Attribute::new("t"));
        column.note(&AttributeValue::Number(1723891200000i64.into()));
        assert_eq!(column.kind(), (FieldKind::Numeric { decimals: 0 }, 13));
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
