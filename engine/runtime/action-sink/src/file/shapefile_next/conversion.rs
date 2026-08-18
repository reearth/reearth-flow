//! Shapefile shape conversion. Turns `reearth_flow_geometry::Geometry` (per-leaf
//! `CoordinateFrame`) into the positions a shapefile record holds.
//!
//! A `.shp` holds records of one shape type, so a geometry converts to a
//! [`Payload`] plus the [`Bucket`](super::shape::Bucket) naming the file it
//! belongs in; the concrete shape type is settled per file once every feature in
//! it is known.
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

use chrono::Datelike;
use reearth_flow_common::datetime::DateTime;
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
    /// The positions to write.
    payload: Payload,
    /// Whether the positions carry an elevation the geometry stated.
    elevated: bool,
    /// The frames the positions came from.
    frames: Frames,
    /// Why parts of the geometry were left out.
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

/// What `geometry` writes to, or why it cannot be written.
fn write(geometry: &Geometry) -> Result<Written, Unwritable> {
    match geometry {
        Geometry::None => Err(Unwritable::AbsentGeometry),
        Geometry::Euclidean2D(g) => write_2d(g),
        Geometry::Euclidean3D(g) => write_3d(g),
        Geometry::GeometryCollection(c) => merge(c.members().iter().map(write)),
    }
}

/// What a 2D geometry writes to; a leaf stating an elevation writes as elevated.
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

/// What a 3D geometry writes to, always elevated.
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

/// A stored coordinate as a shapefile position, its height left where it is: a
/// shapefile's `(x, y)` is `(easting, northing)` whatever the CRS declares, so a
/// frame declaring the reverse has its horizontal pair swapped.
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
    /// The exterior ring, then the holes.
    rings: Vec<Ring>,
    /// Why holes were left out.
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

/// The longest DBF field name, in bytes.
const FIELD_NAME_BYTES: usize = 11;
/// The longest DBF character field, in bytes.
const CHARACTER_BYTES: usize = 254;
/// The width of a DBF logical field.
const LOGICAL_BYTES: usize = 1;
/// The width of a DBF date field.
const DATE_BYTES: usize = 8;
/// The years a DBF date field can hold.
const DATE_YEARS: std::ops::RangeInclusive<i32> = 0..=9999;
/// The most decimal places a numeric column is declared with.
const MAX_DECIMAL_PLACES: u8 = 15;

/// One column of the DBF table: the attribute it takes its value from, and the
/// type every value is written as.
///
/// A DBF field name is at most 11 bytes, so it can differ from the attribute's;
/// the field is keyed by its own name and remembers the attribute's.
pub(super) struct Field {
    /// The name the table declares.
    name: String,
    /// The attribute the values come from.
    attribute: Attribute,
    /// The type the values are written as.
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
    Logical,
    /// A calendar date with no time of day.
    Date,
}

impl FieldKind {
    /// The kind a column holding values of both `self` and `other` is declared as.
    fn join(self, other: Self) -> Self {
        match (self, other) {
            (a, b) if a == b => a,
            (Self::Numeric { decimals: a }, Self::Numeric { decimals: b }) => {
                Self::Numeric { decimals: a.max(b) }
            }
            _ => Self::Character,
        }
    }
}

impl Field {
    /// The value this column writes where the feature carries none it can store.
    fn default(&self) -> FieldValue {
        match self.kind {
            FieldKind::Character => FieldValue::Character(None),
            FieldKind::Numeric { .. } => FieldValue::Numeric(None),
            FieldKind::Logical => FieldValue::Logical(None),
            FieldKind::Date => FieldValue::Date(None),
        }
    }
}

/// What the values of one attribute call for in a column: the kind every value
/// so far fits, and how wide they are written out.
struct Column {
    /// The attribute the values come from.
    attribute: Attribute,
    /// `None` until a storable value is seen.
    kind: Option<FieldKind>,
    /// The widest value as text.
    text_width: usize,
    /// The widest integer part of a number, sign included.
    integer_digits: usize,
    /// Whether some value had no DBF counterpart and will be left out.
    unstorable: bool,
}

impl Column {
    /// A column for `attribute` with no value taken into account yet.
    fn new(attribute: &Attribute) -> Self {
        Self {
            attribute: attribute.clone(),
            kind: None,
            text_width: 0,
            integer_digits: 0,
            unstorable: false,
        }
    }

    /// Take a value into account. A null says nothing about the column; a value
    /// the table cannot store marks it as leaving something out.
    fn note(&mut self, value: &AttributeValue) {
        let (kind, text) = match value {
            AttributeValue::String(s) => (FieldKind::Character, s.clone()),
            AttributeValue::Number(n) => {
                let text = number_text(n);
                let (integer, fraction) = text.split_once('.').unwrap_or((&text, ""));
                self.integer_digits = self.integer_digits.max(integer.len());
                let decimals = fraction.len().min(MAX_DECIMAL_PLACES as usize) as u8;
                (FieldKind::Numeric { decimals }, text)
            }
            AttributeValue::Bool(b) => (FieldKind::Logical, b.to_string()),
            AttributeValue::DateTime(DateTime::NaiveDate(d)) if DATE_YEARS.contains(&d.year()) => {
                (FieldKind::Date, d.to_string())
            }
            AttributeValue::DateTime(d) => (FieldKind::Character, datetime_text(d)),
            AttributeValue::Null => return,
            AttributeValue::Array(_) | AttributeValue::Map(_) | AttributeValue::Bytes(_) => {
                self.unstorable = true;
                return;
            }
        };
        self.kind = Some(self.kind.map_or(kind, |current| current.join(kind)));
        self.text_width = self.text_width.max(text.len());
    }

    /// The type the column is declared as, and its width, once every value is
    /// taken into account. A column no value can be stored in is character, so
    /// the attribute still has a field.
    ///
    /// A numeric column is as wide as its widest integer part and its decimal
    /// places, and holds as many of those as its values need.
    fn kind(&self) -> (FieldKind, u8) {
        let kind = self.kind.unwrap_or(FieldKind::Character);
        let width = match kind {
            FieldKind::Numeric { decimals: 0 } => self.integer_digits,
            FieldKind::Numeric { decimals } => self.integer_digits + 1 + decimals as usize,
            FieldKind::Character => self.text_width.clamp(1, CHARACTER_BYTES),
            FieldKind::Logical => LOGICAL_BYTES,
            FieldKind::Date => DATE_BYTES,
        };
        (kind, width.min(u8::MAX as usize) as u8)
    }
}

/// A date or time as the text a character column writes it in: a date alone as
/// `YYYY-MM-DD`, an instant in RFC 3339.
fn datetime_text(d: &DateTime) -> String {
    match d {
        DateTime::NaiveDate(d) => d.to_string(),
        d => d.to_rfc3339(),
    }
}

/// A number as the shortest decimal text that reads back as the same number; a
/// non-integer keeps at least one decimal place, so its column stays a decimal
/// one.
fn number_text(n: &serde_json::Number) -> String {
    match n.as_f64().filter(|_| n.is_f64()) {
        Some(f) if f.fract() == 0.0 => format!("{f:.1}"),
        Some(f) => format!("{f}"),
        None => n.to_string(),
    }
}

/// The table to write the attributes of `features` into, and its columns.
///
/// A column is declared for every attribute any feature carries, in the order
/// first seen: one holding only numbers is numeric, wide enough for the widest,
/// one holding only booleans is logical, one holding only dates is a date, and
/// one holding anything else, or nothing storable, is character. Names are cut
/// to the DBF limit and told apart where the cut makes two the same. Values with
/// no DBF counterpart are left out and warned about.
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
    for column in columns.values() {
        if column.unstorable {
            tracing::warn!(
                "leaving out the values of '{}' that are arrays, maps or bytes, which a \
                 shapefile attribute table cannot hold",
                column.attribute
            );
        }
        let (kind, width) = column.kind();
        let name = field_name(column.attribute.as_ref(), &taken);
        taken.insert(name.clone());
        let field_name: FieldName = name.as_str().try_into().map_err(|e| {
            crate::errors::SinkError::ShapefileWriter(format!(
                "Failed to convert field name to FieldName: {e}"
            ))
        })?;
        builder = match kind {
            FieldKind::Character => builder.add_character_field(field_name, width),
            FieldKind::Numeric { decimals } => {
                builder.add_numeric_field(field_name, width, decimals)
            }
            FieldKind::Logical => builder.add_logical_field(field_name),
            FieldKind::Date => builder.add_date_field(field_name),
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
            AttributeValue::Number(n) => Some(FieldValue::Numeric(n.as_f64())),
            _ => None,
        },
        FieldKind::Logical => match value {
            AttributeValue::Bool(b) => Some(FieldValue::Logical(Some(*b))),
            _ => None,
        },
        FieldKind::Date => match value {
            AttributeValue::DateTime(DateTime::NaiveDate(d)) if DATE_YEARS.contains(&d.year()) => {
                Some(FieldValue::Date(Some(shapefile::dbase::Date::new(
                    d.day(),
                    d.month(),
                    d.year() as u32,
                ))))
            }
            _ => None,
        },
        FieldKind::Character => {
            let text = match value {
                AttributeValue::String(s) => s.clone(),
                AttributeValue::Number(num) => num.to_string(),
                AttributeValue::Bool(b) => b.to_string(),
                AttributeValue::DateTime(d) => datetime_text(d),
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

    #[test]
    fn a_degenerate_line_writes_no_shape() {
        let line = Geometry::Euclidean2D(Euclidean2DGeometry::LineString(
            LineString2D::from_coords(euclidean(), [[0.0, 0.0]]),
        ));
        assert!(write_geometry(&line).payload.is_none());
    }

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

    #[test]
    fn the_table_covers_fields_a_later_feature_introduces() {
        let first = attributes(&[("a", AttributeValue::String("x".into()))]);
        let second = attributes(&[("b", AttributeValue::Bool(true))]);
        let fields = table(&[&first, &second]);
        assert_eq!(fields.len(), 2);
    }

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

    #[test]
    fn a_null_first_value_takes_its_type_from_a_later_feature() {
        let first = attributes(&[("a", AttributeValue::Null)]);
        let second = attributes(&[("a", AttributeValue::Number(1.into()))]);
        let fields = table(&[&first, &second]);
        assert_eq!(fields[0].kind, FieldKind::Numeric { decimals: 0 });
    }

    #[test]
    fn a_numeric_column_is_as_wide_and_as_precise_as_its_values() {
        let mut column = Column::new(&Attribute::new("t"));
        column.note(&AttributeValue::Number(1723891200000i64.into()));
        assert_eq!(column.kind(), (FieldKind::Numeric { decimals: 0 }, 13));
        column.note(&AttributeValue::Number(
            serde_json::Number::from_f64(-1.23456789).unwrap(),
        ));
        assert_eq!(column.kind(), (FieldKind::Numeric { decimals: 8 }, 22));
    }

    #[test]
    fn an_integral_float_keeps_a_decimal_place() {
        let mut column = Column::new(&Attribute::new("t"));
        column.note(&AttributeValue::Number(
            serde_json::Number::from_f64(889953.0).unwrap(),
        ));
        assert_eq!(column.kind(), (FieldKind::Numeric { decimals: 1 }, 8));
    }

    #[test]
    fn a_column_of_booleans_is_logical_and_one_of_dates_is_a_date() {
        let feature = attributes(&[
            ("flag", AttributeValue::Bool(true)),
            (
                "day",
                AttributeValue::DateTime(DateTime::NaiveDate(
                    chrono::NaiveDate::from_ymd_opt(2025, 7, 17).unwrap(),
                )),
            ),
        ]);
        let fields = table(&[&feature]);
        assert_eq!(fields[0].kind, FieldKind::Logical);
        assert_eq!(fields[1].kind, FieldKind::Date);
        let record = attributes_to_record(&feature, &fields);
        assert_eq!(record.get("flag"), Some(&FieldValue::Logical(Some(true))));
        assert_eq!(
            record.get("day"),
            Some(&FieldValue::Date(Some(shapefile::dbase::Date::new(
                17, 7, 2025
            ))))
        );
    }

    #[test]
    fn a_column_mixing_booleans_and_dates_writes_everything_as_text() {
        let first = attributes(&[("v", AttributeValue::Bool(true))]);
        let second = attributes(&[(
            "v",
            AttributeValue::DateTime(DateTime::NaiveDate(
                chrono::NaiveDate::from_ymd_opt(2025, 7, 17).unwrap(),
            )),
        )]);
        let fields = table(&[&first, &second]);
        assert_eq!(fields[0].kind, FieldKind::Character);
        assert_eq!(
            attributes_to_record(&first, &fields).get("v"),
            Some(&FieldValue::Character(Some("true".into())))
        );
        assert_eq!(
            attributes_to_record(&second, &fields).get("v"),
            Some(&FieldValue::Character(Some("2025-07-17".into())))
        );
    }

    #[test]
    fn a_column_with_nothing_storable_is_still_a_field() {
        let feature = attributes(&[
            ("list", AttributeValue::Array(vec![])),
            ("nothing", AttributeValue::Null),
        ]);
        let fields = table(&[&feature]);
        assert_eq!(fields.len(), 2);
        assert!(fields.iter().all(|f| f.kind == FieldKind::Character));
        let record = attributes_to_record(&feature, &fields);
        assert_eq!(record.get("list"), Some(&FieldValue::Character(None)));
        assert_eq!(record.get("nothing"), Some(&FieldValue::Character(None)));
    }

    // The table's declared types must let every value be written and read back
    // as it was, whatever mix of values the attributes hold.
    #[test]
    fn a_table_writes_and_reads_back_every_kind_of_value() {
        use shapefile::dbase::{FieldType, Reader};

        let date = |y, m, d| {
            AttributeValue::DateTime(DateTime::NaiveDate(
                chrono::NaiveDate::from_ymd_opt(y, m, d).unwrap(),
            ))
        };
        let float = |f: f64| AttributeValue::Number(serde_json::Number::from_f64(f).unwrap());
        let features = [
            attributes(&[
                ("int", AttributeValue::Number((-7i64).into())),
                ("wide", AttributeValue::Number(u64::MAX.into())),
                ("dec", float(-0.000123456789)),
                ("intfloat", float(889953.0)),
                ("mixed", AttributeValue::Number(5.into())),
                ("flag", AttributeValue::Bool(false)),
                ("day", date(1899, 12, 31)),
                ("bc", date(-44, 3, 15)),
                ("text", AttributeValue::String("é".repeat(300))),
                ("nothing", AttributeValue::Null),
            ]),
            attributes(&[
                ("int", AttributeValue::Number(1234567890123i64.into())),
                ("dec", float(1e-3)),
                ("mixed", float(2.5)),
                ("flag", AttributeValue::Bool(true)),
                ("day", date(2025, 7, 17)),
                ("bc", date(2025, 7, 17)),
                ("text", AttributeValue::String("short".into())),
            ]),
        ];

        let (builder, fields) =
            make_table_builder(features.iter()).expect("the table is expected to build");
        let mut dbf = Vec::new();
        {
            let mut writer = builder.build_with_dest(std::io::Cursor::new(&mut dbf));
            for feature in &features {
                writer
                    .write_record(&attributes_to_record(feature, &fields))
                    .expect("the record is expected to write");
            }
            writer
                .finalize()
                .expect("the table is expected to finalize");
        }

        let mut reader =
            Reader::new(std::io::Cursor::new(dbf)).expect("the table is expected to read");
        let declared: Vec<(String, FieldType, u8)> = reader
            .fields()
            .iter()
            .map(|f| (f.name().to_string(), f.field_type(), f.length()))
            .collect();
        assert_eq!(
            declared,
            vec![
                ("int".into(), FieldType::Numeric, 13),
                ("wide".into(), FieldType::Numeric, 20),
                ("dec".into(), FieldType::Numeric, 15),
                ("intfloat".into(), FieldType::Numeric, 8),
                ("mixed".into(), FieldType::Numeric, 3),
                ("flag".into(), FieldType::Logical, 1),
                ("day".into(), FieldType::Date, 8),
                ("bc".into(), FieldType::Character, 11),
                ("text".into(), FieldType::Character, 254),
                ("nothing".into(), FieldType::Character, 1),
            ]
        );

        let records: Vec<Record> = reader
            .read()
            .expect("the records are expected to read")
            .into_iter()
            .collect();
        let first = &records[0];
        assert_eq!(first.get("int"), Some(&FieldValue::Numeric(Some(-7.0))));
        assert_eq!(
            first.get("wide"),
            Some(&FieldValue::Numeric(Some(u64::MAX as f64)))
        );
        assert_eq!(
            first.get("dec"),
            Some(&FieldValue::Numeric(Some(-0.000123456789)))
        );
        assert_eq!(
            first.get("intfloat"),
            Some(&FieldValue::Numeric(Some(889953.0)))
        );
        assert_eq!(first.get("mixed"), Some(&FieldValue::Numeric(Some(5.0))));
        assert_eq!(first.get("flag"), Some(&FieldValue::Logical(Some(false))));
        assert_eq!(
            first.get("day"),
            Some(&FieldValue::Date(Some(shapefile::dbase::Date::new(
                31, 12, 1899
            ))))
        );
        assert_eq!(
            first.get("bc"),
            Some(&FieldValue::Character(Some("-0044-03-15".into())))
        );
        assert_eq!(
            first.get("text"),
            Some(&FieldValue::Character(Some("é".repeat(127))))
        );
        assert_eq!(first.get("nothing"), Some(&FieldValue::Character(None)));
        let second = &records[1];
        assert_eq!(
            second.get("int"),
            Some(&FieldValue::Numeric(Some(1234567890123.0)))
        );
        assert_eq!(second.get("dec"), Some(&FieldValue::Numeric(Some(0.001))));
        assert_eq!(second.get("mixed"), Some(&FieldValue::Numeric(Some(2.5))));
        assert_eq!(second.get("wide"), Some(&FieldValue::Numeric(None)));
        assert_eq!(second.get("nothing"), Some(&FieldValue::Character(None)));
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
