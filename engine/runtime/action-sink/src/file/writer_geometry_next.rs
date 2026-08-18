//! Geometry export for the CSV Writer: turns a feature's geometry into either a
//! WKT column or x/y/z coordinate columns.
//!
//! Each geometry leaf carries its own `CoordinateFrame`, so a single feature can
//! hold coordinates in more than one reference system. Everything here is written
//! against that: the axis order of a coordinate is decided per leaf, and folding
//! parts into one `MULTI*` requires them to share a frame.

use std::collections::HashSet;
use std::fmt::Write as _;
use std::sync::{Mutex, OnceLock};

use indexmap::IndexMap;
use reearth_flow_geometry::coordinate::{CoordinateFrame, EpsgCode};
use reearth_flow_geometry::ops::Split;
use reearth_flow_geometry::{Euclidean2DGeometry, Euclidean3DGeometry, Geometry};

use super::{GeometryExportConfig, GeometryExportMode};
use crate::errors::GeometryExportError;

/// What a geometry writes to: its WKT text, the `MULTI*` family it can fold into,
/// the coordinate frames its coordinates came from, and the reasons parts of it
/// were left out.
///
/// The frames and reasons come out of the same pass as the text, so nothing has to
/// re-walk the geometry to recover them. Carrying the reasons rather than logging
/// them where a part is dropped keeps the recursion a pure map, the whole
/// geometry's omissions being reported once it is written.
struct WrittenWkt {
    kind: Option<Kind>,
    text: String,
    frames: Frames,
    omitted: Vec<GeometryExportError>,
}

impl WrittenWkt {
    /// What a leaf writes to: one text, in one frame, leaving nothing out.
    fn leaf(frame: &CoordinateFrame, kind: Kind, text: String) -> Self {
        Self {
            kind: Some(kind),
            text,
            frames: Frames::of(frame),
            omitted: Vec::new(),
        }
    }
}

/// The coordinate frames the coordinates written for a geometry came from.
///
/// Decides whether written parts may fold into a `MULTI*`, which needs frame
/// identity: two `Euclidean` parts fold even though neither names a CRS.
#[derive(Clone, Default)]
enum Frames {
    #[default]
    Nothing,
    One(CoordinateFrame),
    Many,
}

impl Frames {
    fn of(frame: &CoordinateFrame) -> Self {
        Self::One(frame.clone())
    }

    fn and(self, other: Self) -> Self {
        match (self, other) {
            (Self::Nothing, frames) | (frames, Self::Nothing) => frames,
            (Self::One(one), Self::One(other)) if one == other => Self::One(one),
            _ => Self::Many,
        }
    }

    fn uniform(&self) -> bool {
        !matches!(self, Self::Many)
    }
}

/// The writable parts of a container, and every reason the writer left something
/// out.
struct Parts {
    /// Non-empty: a container with no writable part cannot be written either,
    /// which `Parts::of` reports as an error instead.
    written: Vec<WrittenWkt>,
    omitted: Vec<GeometryExportError>,
}

impl Parts {
    /// Write every part, keeping the writable ones: a part WKT cannot express is
    /// dropped where it appears rather than failing the geometry around it, so it
    /// does not discard its siblings. `Err` once nothing is left, a container
    /// reduced to nothing being unwritable too.
    fn of<T>(
        parts: &[T],
        write: impl Fn(&T) -> Result<WrittenWkt, GeometryExportError>,
        empty: impl FnOnce() -> GeometryExportError,
    ) -> Result<Self, GeometryExportError> {
        let mut written = Vec::new();
        let mut omitted = Vec::new();
        for part in parts {
            match write(part) {
                Ok(part) => written.push(part),
                Err(reason) => omitted.push(reason),
            }
        }
        if written.is_empty() {
            let mut reasons = omitted.into_iter();
            let first = reasons.next();
            // Reasons past the first would otherwise fall off the Vec unreported.
            warn_omitted(&reasons.collect::<Vec<_>>());
            return Err(first.unwrap_or_else(empty));
        }
        for part in &mut written {
            omitted.append(&mut part.omitted);
        }
        Ok(Self { written, omitted })
    }

    /// The parts as one WKT geometry: a `MULTI*` when they are all of one family
    /// and share a coordinate frame, a `GEOMETRYCOLLECTION` otherwise.
    ///
    /// Parts that differ in frame are not folded: that would put coordinates from
    /// different reference systems in one geometry, and WKT names no CRS to
    /// describe either of them.
    fn into_one_geometry(self) -> WrittenWkt {
        let kinds: Option<Vec<Kind>> = self.written.iter().map(|part| part.kind).collect();
        let uniform = kinds.as_ref().and_then(|kinds| {
            let first = *kinds.first()?;
            kinds.iter().all(|kind| *kind == first).then_some(first)
        });
        match uniform {
            Some(kind) if self.frames().uniform() => self.present(Some(kind), |parts| {
                kind.fold(parts.iter().map(|part| part.text.as_str()))
            }),
            _ => {
                if !self.frames().uniform() {
                    warn_mixed_frames();
                }
                self.into_geometry_collection()
            }
        }
    }

    /// The parts as one WKT `GEOMETRYCOLLECTION`, whatever they are.
    fn into_geometry_collection(self) -> WrittenWkt {
        self.present(None, |parts| {
            format!(
                "GEOMETRYCOLLECTION({})",
                parts
                    .iter()
                    .map(|part| part.text.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        })
    }

    fn frames(&self) -> Frames {
        self.written
            .iter()
            .map(|part| part.frames.clone())
            .fold(Frames::Nothing, Frames::and)
    }

    /// The parts as the one geometry `present` builds from their texts, carrying the
    /// frames they were written in and what the writer left out.
    fn present(
        self,
        kind: Option<Kind>,
        present: impl FnOnce(&[WrittenWkt]) -> String,
    ) -> WrittenWkt {
        WrittenWkt {
            kind,
            text: present(&self.written),
            frames: self.frames(),
            omitted: self.omitted,
        }
    }
}

/// The `MULTI*` family a written geometry belongs to. A `GEOMETRYCOLLECTION`
/// belongs to none, having no `MULTI*` form to fold into.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Point,
    Curve,
    Area,
}

impl Kind {
    /// Texts that all belong to this family, folded into the matching `MULTI*`. A
    /// text that is already a `MULTI*` is flattened into the result: its own
    /// members become members here, rather than nesting.
    fn fold<'a>(self, texts: impl Iterator<Item = &'a str>) -> String {
        let keyword = match self {
            Self::Point => "MULTIPOINT",
            Self::Curve => "MULTILINESTRING",
            Self::Area => "MULTIPOLYGON",
        };
        let members = texts
            .map(|text| Self::members_of(text, keyword))
            .collect::<Vec<_>>()
            .join(", ");
        format!("{keyword}({members})")
    }

    /// A part's contribution to the fold: the inside of an already-folded `MULTI*`,
    /// or the whole text with its own keyword dropped.
    ///
    /// `MULTIPOINT` is the flat form — `MULTIPOINT(0 0, 1 1)`, matching the old
    /// writer — so a `POINT`'s coordinates go in bare; the other families keep
    /// their parenthesised bodies.
    fn members_of(text: &str, multi: &str) -> String {
        let body = |text: &str, keyword: &str| {
            text.strip_prefix(keyword)
                .and_then(|rest| rest.strip_prefix('('))
                .and_then(|rest| rest.strip_suffix(')'))
                .map(str::to_string)
        };
        if let Some(inner) = body(text, multi) {
            return inner;
        }
        for keyword in ["POINT", "LINESTRING", "POLYGON"] {
            if let Some(inner) = body(text, keyword) {
                return match keyword {
                    "POINT" => inner,
                    _ => format!("({inner})"),
                };
            }
        }
        text.to_string()
    }
}

/// Export geometry to column values based on configuration.
pub fn export_geometry(
    geometry: &Geometry,
    config: &GeometryExportConfig,
) -> Result<IndexMap<String, String>, GeometryExportError> {
    let mut columns = IndexMap::new();

    match &config.mode {
        GeometryExportMode::Wkt { column } => {
            let written = geometry_wkt(geometry)?;
            columns.insert(column.clone(), written.text);
            insert_epsg_column(&mut columns, config, single_frame_epsg_code(&written.frames));
        }
        GeometryExportMode::Coordinates {
            x_column,
            y_column,
            z_column,
        } => {
            let (x, y, z) = extract_coordinates(geometry)?;
            columns.insert(x_column.clone(), x.to_string());
            columns.insert(y_column.clone(), y.to_string());
            if let (Some(z), Some(z_column)) = (z, z_column.as_ref()) {
                columns.insert(z_column.clone(), z.to_string());
            }
            insert_epsg_column(&mut columns, config, point_frame(geometry).and_then(epsg_code));
        }
    }

    Ok(columns)
}

/// Write the configured EPSG column, if any, when a code is available.
///
/// Nothing is inserted when either is missing: `csv.rs` pads an unfilled geometry
/// column with an empty string, so leaving the entry out is what produces a blank
/// cell rather than inserting one explicitly.
fn insert_epsg_column(
    columns: &mut IndexMap<String, String>,
    config: &GeometryExportConfig,
    code: Option<EpsgCode>,
) {
    if let (Some(epsg_column), Some(code)) = (&config.epsg_column, code) {
        columns.insert(epsg_column.clone(), code.to_string());
    }
}

/// The single EPSG code a WKT cell's coordinates all came from, or `None` when
/// they came from no CRS frame or from more than one.
fn single_frame_epsg_code(frames: &Frames) -> Option<EpsgCode> {
    match frames {
        Frames::One(frame) => epsg_code(frame),
        Frames::Nothing | Frames::Many => None,
    }
}

/// The coordinate frame of a Point geometry, coordinates mode's only writable
/// shape.
fn point_frame(geometry: &Geometry) -> Option<&CoordinateFrame> {
    match geometry {
        Geometry::Euclidean2D(Euclidean2DGeometry::Point(p)) => Some(p.frame()),
        Geometry::Euclidean3D(Euclidean3DGeometry::Point(p)) => Some(p.frame()),
        _ => None,
    }
}

/// The WKT text a geometry writes to, and the coordinate frame(s) its
/// coordinates came from.
struct GeometryWkt {
    text: String,
    frames: Frames,
}

fn geometry_wkt(geometry: &Geometry) -> Result<GeometryWkt, GeometryExportError> {
    match geometry {
        Geometry::None => Ok(GeometryWkt {
            text: String::new(),
            frames: Frames::Nothing,
        }),
        geometry => {
            let written = write_geometry(geometry)?;
            warn_omitted(&written.omitted);
            Ok(GeometryWkt {
                text: written.text,
                frames: written.frames,
            })
        }
    }
}

/// Convert geometry to a WKT string.
///
/// An absent geometry writes an empty cell rather than failing the row, since the
/// feature's attributes are still worth a row. Coordinates mode deliberately
/// differs and errors with `EmptyGeometry`: an empty WKT cell still reads as "no
/// geometry", whereas blank x/y columns would be indistinguishable from a point at
/// an unknown position.
pub fn geometry_to_wkt(geometry: &Geometry) -> Result<String, GeometryExportError> {
    Ok(geometry_wkt(geometry)?.text)
}

/// Extract X, Y, Z coordinates from Point geometries.
/// Returns (x, y, optional z).
pub fn extract_coordinates(
    geometry: &Geometry,
) -> Result<(f64, f64, Option<f64>), GeometryExportError> {
    match geometry {
        Geometry::None => Err(GeometryExportError::EmptyGeometry),
        Geometry::Euclidean2D(Euclidean2DGeometry::Point(p)) => {
            let [x, y] = horizontal(p.frame(), p.position());
            Ok((x, y, None))
        }
        Geometry::Euclidean3D(Euclidean3DGeometry::Point(p)) => {
            let [x, y, z] = p.position();
            let [x, y] = horizontal(p.frame(), [x, y]);
            Ok((x, y, Some(z)))
        }
        _ => Err(GeometryExportError::NonPointGeometry),
    }
}

/// A stored horizontal pair as `(easting, northing)`, so an x column always holds
/// the easting whatever order the CRS declares.
fn horizontal(frame: &CoordinateFrame, [a, b]: [f64; 2]) -> [f64; 2] {
    if swaps_axes(frame) {
        [b, a]
    } else {
        [a, b]
    }
}

fn write_geometry(geometry: &Geometry) -> Result<WrittenWkt, GeometryExportError> {
    match geometry {
        Geometry::None => Err(GeometryExportError::EmptyGeometry),
        Geometry::Euclidean2D(g) => write_2d(g),
        Geometry::Euclidean3D(g) => write_3d(g),
        // Cross-dimensional and cross-frame, so no `Multi*` describes it.
        Geometry::GeometryCollection(c) => Parts::of(c.members(), write_geometry, || {
            GeometryExportError::UnsupportedGeometryCollection
        })
        .map(Parts::into_geometry_collection),
    }
}

fn write_2d(geometry: &Euclidean2DGeometry) -> Result<WrittenWkt, GeometryExportError> {
    use Euclidean2DGeometry::*;
    match geometry {
        Point(p) => Ok(point(p.frame(), p.position())),
        LineString(l) => Ok(curve(l.frame(), l.coords())),
        Polygon(p) => Ok(area(p.frame(), p.exterior(), p.interiors())),
        PolygonMesh(m) => write_faces((**m).clone(), "PolygonMesh"),
        TriangularMesh(m) => write_faces((**m).clone(), "TriangularMesh"),
        Collection(c) => Parts::of(c.members(), write_2d, || {
            GeometryExportError::UnsupportedGeometryCollection
        })
        .map(Parts::into_one_geometry),
    }
}

fn write_3d(geometry: &Euclidean3DGeometry) -> Result<WrittenWkt, GeometryExportError> {
    use Euclidean3DGeometry::*;
    match geometry {
        Point(p) => Ok(point(p.frame(), p.position())),
        LineString(l) => Ok(curve(l.frame(), l.coords())),
        Polygon(p) => Ok(area(p.frame(), p.exterior(), p.interiors())),
        PolygonMesh(m) => write_faces((**m).clone(), "PolygonMesh"),
        TriangularMesh(m) => write_faces((**m).clone(), "TriangularMesh"),
        Collection(c) => Parts::of(c.members(), write_3d, || {
            GeometryExportError::UnsupportedGeometryCollection
        })
        .map(Parts::into_one_geometry),
        // WKT has no volume, nor the boolean tree built from volumes. A PointCloud
        // could fill a MULTIPOINT, but that would emit a position per sample.
        Solid(_) => Err(unsupported("Solid")),
        Csg(_) => Err(unsupported("Csg")),
        PointCloud(_) => Err(unsupported("PointCloud")),
    }
}

/// A mesh writes as its faces, folding into a `MULTIPOLYGON` the way a collection
/// writes as its members.
///
/// `Split::split` takes `&mut self`, and its contract permits it to empty the
/// receiver of the members it yields; the current mesh impls happen to read
/// rather than empty, but that is not a guarantee this function can rely on. The
/// mesh is therefore passed by value — `write_2d`/`write_3d` clone it first — so
/// the feature's geometry is left intact regardless of what a given `Split` impl
/// actually does. `kind` names the mesh kind in the error when its faces cannot
/// be read.
///
/// A building-sized mesh becomes one cell holding hundreds of rings. That is the
/// same output the GeoJSON writer produces, but CSV is opened in spreadsheet tools
/// with cell-length limits, so the cell may be truncated by the reader rather than
/// by us. Kept as a comment rather than a parameter description: doc comments on the
/// parameter types feed `schema/actions*.json` and the i18n files, which this port
/// must leave byte-identical.
fn write_faces(
    mut mesh: impl Split,
    kind: &'static str,
) -> Result<WrittenWkt, GeometryExportError> {
    let mut faces = Vec::new();
    mesh.split(&mut |face, _| faces.push(face))
        .map_err(|_| unsupported(kind))?;
    Parts::of(&faces, write_geometry, || unsupported(kind)).map(Parts::into_one_geometry)
}

// The 2D and 3D leaves differ only in how long a position is, so what turns one
// into WKT is written once, over `N`-element positions.

fn point<const N: usize>(frame: &CoordinateFrame, position: [f64; N]) -> WrittenWkt {
    WrittenWkt::leaf(
        frame,
        Kind::Point,
        format!("POINT({})", coordinate(swaps_axes(frame), position)),
    )
}

fn curve<const N: usize>(frame: &CoordinateFrame, coords: &[[f64; N]]) -> WrittenWkt {
    WrittenWkt::leaf(
        frame,
        Kind::Curve,
        format!("LINESTRING({})", coordinate_list(swaps_axes(frame), coords)),
    )
}

/// What an area writes to: its exterior ring, then its holes.
fn area<'a, const N: usize>(
    frame: &CoordinateFrame,
    exterior: &'a [[f64; N]],
    interiors: impl Iterator<Item = &'a [[f64; N]]>,
) -> WrittenWkt {
    let swap = swaps_axes(frame);
    let rings = std::iter::once(exterior)
        .chain(interiors)
        .map(|ring| format!("({})", coordinate_list(swap, &closed_ring(ring))))
        .collect::<Vec<_>>()
        .join(", ");
    WrittenWkt::leaf(frame, Kind::Area, format!("POLYGON({rings})"))
}

/// Whether a frame stores its horizontal axes reflected from canonical
/// `(East, North)` order, so that they must be swapped on the way out.
///
/// WKT names no coordinate reference system and consumers read it east-first, so
/// the stored order is undone here. Only a CRS declares an axis order to swap back
/// to: `Euclidean` coordinates are east-first by construction, and a `Tangent`
/// frame's are offsets along its own in-plane axes rather than its base CRS's.
///
/// A CRS whose order cannot be established is written as stored, which reverses its
/// coordinates if it turns out to declare `(North, East)`, hence the warning.
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

/// The EPSG code a frame names, if it names one. `Euclidean` names none, and a
/// `Tangent` plane's in-plane coordinates are not its base CRS's.
fn epsg_code(frame: &CoordinateFrame) -> Option<EpsgCode> {
    match frame {
        CoordinateFrame::Crs(code) => Some(*code),
        CoordinateFrame::Euclidean | CoordinateFrame::Tangent(_) => None,
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

/// One stored coordinate, space-separated, east-first. Only the horizontal pair is
/// reordered; a height stays where it is. Values use `{}` formatting, so a whole
/// number writes as `1` rather than `1.0`.
fn coordinate<const N: usize>(swap: bool, mut coordinate: [f64; N]) -> String {
    if swap {
        coordinate.swap(0, 1);
    }
    let mut text = String::new();
    for (i, value) in coordinate.iter().enumerate() {
        if i > 0 {
            text.push(' ');
        }
        write!(text, "{value}").expect("writing to a String cannot fail");
    }
    text
}

fn coordinate_list<const N: usize>(swap: bool, coords: &[[f64; N]]) -> String {
    coords
        .iter()
        .map(|&c| coordinate(swap, c))
        .collect::<Vec<_>>()
        .join(", ")
}

/// WKT requires a ring's first and last positions to be equal. The stored rings
/// carry no such guarantee, so an open one is closed on the way out.
fn closed_ring<const N: usize>(ring: &[[f64; N]]) -> Vec<[f64; N]> {
    let mut ring = ring.to_vec();
    if let (Some(first), Some(last)) = (ring.first().copied(), ring.last().copied()) {
        if first != last {
            ring.push(first);
        }
    }
    ring
}

/// Build the error for a geometry kind WKT cannot express, naming it by a short,
/// stable label rather than by its `Debug` output.
///
/// A PLATEAU `lod2Solid` parses to a `Euclidean3DGeometry::Solid`, whose `Debug`
/// prints every shell, mesh and vertex; `csv.rs` logs this error once per
/// feature, so a full dump here would turn a CityGML-to-CSV export into
/// gigabytes of log output for a single warning. The label keeps the message a
/// fixed, small size no matter how large the geometry it stands in for is, and
/// (as a side effect) lets `warn_omitted`'s message-based dedup collapse many
/// unwritable parts of one container into a single log line.
fn unsupported(kind: &'static str) -> GeometryExportError {
    GeometryExportError::UnsupportedGeometryType(kind.to_string())
}

/// Warn that a WKT cell holds coordinates from more than one reference system.
///
/// Once per process, like the axis-order warning: WKT names no CRS and a CSV column
/// has nowhere to put one, so this warning is the only trace of the mixture. The
/// members are still written — refusing them would discard data the user asked to
/// export. (The spec says "once per output file"; the module has no notion of the
/// output, so once per process is the honest approximation.)
fn warn_mixed_frames() {
    static WARNED: OnceLock<()> = OnceLock::new();
    WARNED.get_or_init(|| {
        tracing::warn!(
            "a WKT cell holds coordinates in more than one reference system; WKT \
             declares none, so the mixture is not recoverable from the output"
        );
    });
}

/// Report what the writer left out. An omission does not fail the geometry around
/// it, so this warning is the only trace of it. Deduplicated by message: a mesh
/// drops the same reason once per face otherwise.
fn warn_omitted(omitted: &[GeometryExportError]) {
    let mut seen = std::collections::BTreeSet::new();
    for reason in omitted {
        let reason = reason.to_string();
        if seen.insert(reason.clone()) {
            tracing::warn!(%reason, "omitting a geometry member from the CSV output");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reearth_flow_geometry::{
        collection::Collection2D,
        coordinate::{BaseFrame, CoordinateFrame, EpsgCode, TangentPlane},
        line_string::{LineString2D, LineString3D},
        point::{Point2D, Point3D},
        point_cloud::PointCloud,
        polygon::{Polygon2D, Polygon3D},
        solid::Solid,
        triangular_mesh::{TriangularMesh3D, TriangularMesh3DData},
        Euclidean2DGeometry, Euclidean3DGeometry, Geometry, GeometryCollection,
    };

    /// Assert that `result` is `Err(GeometryExportError::UnsupportedGeometryType(label))`
    /// for exactly the given `label`, rather than the wildcard match the type alone
    /// allows: pinning the payload catches a regression back to dumping the whole
    /// geometry's `Debug` output into the message.
    fn assert_unsupported_geometry_type<T>(result: Result<T, GeometryExportError>, label: &str)
    where
        T: std::fmt::Debug,
    {
        match result {
            Err(GeometryExportError::UnsupportedGeometryType(actual)) => {
                assert_eq!(actual, label);
            }
            other => panic!("expected Err(UnsupportedGeometryType({label:?})), got {other:?}"),
        }
    }

    fn wkt_of(geometry: &Geometry) -> String {
        geometry_to_wkt(geometry).expect("geometry expected to write")
    }

    fn euclidean() -> CoordinateFrame {
        CoordinateFrame::Euclidean
    }

    #[test]
    fn point_writes_as_wkt_point() {
        let geometry = Geometry::Euclidean2D(Euclidean2DGeometry::Point(Point2D::new(
            euclidean(),
            [1.0, 2.0],
        )));
        assert_eq!(wkt_of(&geometry), "POINT(1 2)");
    }

    #[test]
    fn line_string_writes_as_wkt_linestring() {
        let geometry = Geometry::Euclidean2D(Euclidean2DGeometry::LineString(
            LineString2D::from_coords(euclidean(), [[0.0, 0.0], [1.0, 1.0], [2.0, 2.0]]),
        ));
        assert_eq!(wkt_of(&geometry), "LINESTRING(0 0, 1 1, 2 2)");
    }

    #[test]
    fn polygon_writes_its_exterior_then_its_holes() {
        let geometry = Geometry::Euclidean2D(Euclidean2DGeometry::Polygon(Box::new(
            Polygon2D::from_rings(
                euclidean(),
                [
                    [0.0, 0.0],
                    [10.0, 0.0],
                    [10.0, 10.0],
                    [0.0, 10.0],
                    [0.0, 0.0],
                ],
                vec![vec![
                    [2.0, 2.0],
                    [8.0, 2.0],
                    [8.0, 8.0],
                    [2.0, 8.0],
                    [2.0, 2.0],
                ]],
            ),
        )));
        assert_eq!(
            wkt_of(&geometry),
            "POLYGON((0 0, 10 0, 10 10, 0 10, 0 0), (2 2, 8 2, 8 8, 2 8, 2 2))"
        );
    }

    // WKT requires a ring's first and last positions to be equal; a stored ring
    // carries no such guarantee.
    #[test]
    fn an_open_ring_is_closed_on_the_way_out() {
        let geometry = Geometry::Euclidean2D(Euclidean2DGeometry::Polygon(Box::new(
            Polygon2D::from_rings(
                euclidean(),
                [[0.0, 0.0], [4.0, 0.0], [4.0, 4.0]],
                Vec::<Vec<[f64; 2]>>::new(),
            ),
        )));
        assert_eq!(wkt_of(&geometry), "POLYGON((0 0, 4 0, 4 4, 0 0))");
    }

    // An absent geometry writes an empty cell: the feature's attributes are still
    // worth a row.
    #[test]
    fn an_absent_geometry_writes_an_empty_cell() {
        assert_eq!(wkt_of(&Geometry::None), "");
    }

    // The 2D tests above exercise `write_2d`; these mirror them for `write_3d` so a
    // bug specific to 3D field extraction (an extra coordinate, a wrong order) does
    // not ship undetected.

    #[test]
    fn point_3d_writes_as_wkt_point() {
        let geometry = Geometry::Euclidean3D(Euclidean3DGeometry::Point(Point3D::new(
            euclidean(),
            [1.0, 2.0, 3.0],
        )));
        assert_eq!(wkt_of(&geometry), "POINT(1 2 3)");
    }

    #[test]
    fn line_string_3d_writes_as_wkt_linestring() {
        let geometry = Geometry::Euclidean3D(Euclidean3DGeometry::LineString(
            LineString3D::from_coords(euclidean(), [[0.0, 0.0, 0.0], [1.0, 1.0, 1.0]]),
        ));
        assert_eq!(wkt_of(&geometry), "LINESTRING(0 0 0, 1 1 1)");
    }

    #[test]
    fn polygon_3d_writes_its_exterior_and_closes_its_ring() {
        let geometry = Geometry::Euclidean3D(Euclidean3DGeometry::Polygon(Box::new(
            Polygon3D::from_rings(
                euclidean(),
                [[0.0, 0.0, 0.0], [4.0, 0.0, 0.0], [4.0, 4.0, 1.0]],
                Vec::<Vec<[f64; 3]>>::new(),
            ),
        )));
        assert_eq!(wkt_of(&geometry), "POLYGON((0 0 0, 4 0 0, 4 4 1, 0 0 0))");
    }

    // EPSG:6675 (JGD2011 plane rectangular CS IX) declares northing first, so a
    // stored coordinate is reversed on the way out: WKT is always east-first.
    #[test]
    fn a_north_first_crs_swaps_its_horizontal_pair() {
        let geometry = Geometry::Euclidean2D(Euclidean2DGeometry::Point(Point2D::new(
            CoordinateFrame::Crs(EpsgCode::new(6675)),
            [1.0, 2.0],
        )));
        assert_eq!(wkt_of(&geometry), "POINT(2 1)");
    }

    // Euclidean coordinates are east-first by construction.
    #[test]
    fn a_euclidean_frame_is_not_swapped() {
        let geometry = Geometry::Euclidean2D(Euclidean2DGeometry::Point(Point2D::new(
            euclidean(),
            [1.0, 2.0],
        )));
        assert_eq!(wkt_of(&geometry), "POINT(1 2)");
    }

    // A tangent plane's coordinates are offsets along its own in-plane axes, not
    // its base CRS's, so the base CRS's axis order does not apply.
    #[test]
    fn a_tangent_frame_is_not_swapped() {
        let frame = CoordinateFrame::Tangent(Box::new(TangentPlane {
            base: BaseFrame::Crs(EpsgCode::new(6675)),
            origin: [0.0, 0.0, 0.0],
            u: [1.0, 0.0, 0.0],
            v: [0.0, 1.0, 0.0],
        }));
        let geometry =
            Geometry::Euclidean2D(Euclidean2DGeometry::Point(Point2D::new(frame, [1.0, 2.0])));
        assert_eq!(wkt_of(&geometry), "POINT(1 2)");
    }

    // A height is not part of the horizontal pair and stays where it is.
    #[test]
    fn a_height_is_not_reordered() {
        let geometry = Geometry::Euclidean3D(Euclidean3DGeometry::Point(Point3D::new(
            CoordinateFrame::Crs(EpsgCode::new(6675)),
            [1.0, 2.0, 3.0],
        )));
        assert_eq!(wkt_of(&geometry), "POINT(2 1 3)");
    }

    // Direct tests of the two public entry points `csv.rs` actually calls
    // (`export_geometry`, `extract_coordinates`), rather than only the internal
    // `geometry_to_wkt` helper the tests above exercise.

    #[test]
    fn export_geometry_in_wkt_mode_writes_the_configured_column() {
        let geometry = Geometry::Euclidean2D(Euclidean2DGeometry::Point(Point2D::new(
            euclidean(),
            [1.0, 2.0],
        )));
        let config = GeometryExportConfig {
            mode: GeometryExportMode::Wkt {
                column: "geometry".to_string(),
            },
            epsg_column: None,
        };
        let columns = export_geometry(&geometry, &config).expect("geometry expected to export");
        assert_eq!(columns.len(), 1);
        assert_eq!(columns.get("geometry"), Some(&"POINT(1 2)".to_string()));
    }

    // Covers the `Some(z)`/`Some(z_column)` guard in `export_geometry`'s coordinates
    // branch. Task 5's coordinate tests may cover similar ground later; that overlap
    // is fine, this one belongs to this task's entry points.
    #[test]
    fn export_geometry_in_coordinates_mode_writes_x_y_z_in_order() {
        let geometry = Geometry::Euclidean3D(Euclidean3DGeometry::Point(Point3D::new(
            euclidean(),
            [1.0, 2.0, 3.0],
        )));
        let config = GeometryExportConfig {
            mode: GeometryExportMode::Coordinates {
                x_column: "x".to_string(),
                y_column: "y".to_string(),
                z_column: Some("z".to_string()),
            },
            epsg_column: None,
        };
        let columns = export_geometry(&geometry, &config).expect("geometry expected to export");
        assert_eq!(
            columns.into_iter().collect::<Vec<_>>(),
            vec![
                ("x".to_string(), "1".to_string()),
                ("y".to_string(), "2".to_string()),
                ("z".to_string(), "3".to_string()),
            ]
        );
    }

    #[test]
    fn extract_coordinates_on_a_2d_point_has_no_z() {
        let geometry = Geometry::Euclidean2D(Euclidean2DGeometry::Point(Point2D::new(
            euclidean(),
            [1.0, 2.0],
        )));
        assert_eq!(extract_coordinates(&geometry).unwrap(), (1.0, 2.0, None));
    }

    #[test]
    fn extract_coordinates_on_a_non_point_geometry_errors() {
        let geometry = Geometry::Euclidean2D(Euclidean2DGeometry::LineString(
            LineString2D::from_coords(euclidean(), [[0.0, 0.0], [1.0, 1.0]]),
        ));
        assert!(matches!(
            extract_coordinates(&geometry),
            Err(GeometryExportError::NonPointGeometry)
        ));
    }

    fn collection_2d(members: Vec<Euclidean2DGeometry>) -> Geometry {
        Geometry::Euclidean2D(Euclidean2DGeometry::Collection(Collection2D::new(members)))
    }

    fn point_2d(frame: CoordinateFrame, position: [f64; 2]) -> Euclidean2DGeometry {
        Euclidean2DGeometry::Point(Point2D::new(frame, position))
    }

    // Coverage for `export_geometry`'s coordinates mode, the counterpart of the
    // `geometry_to_wkt`/WKT-mode tests above. `extract_coordinates` had no test in
    // either geometry world before this task.

    fn coordinates_config(z: Option<&str>) -> GeometryExportConfig {
        GeometryExportConfig {
            mode: GeometryExportMode::Coordinates {
                x_column: "x".to_string(),
                y_column: "y".to_string(),
                z_column: z.map(str::to_string),
            },
            epsg_column: None,
        }
    }

    fn exported(geometry: &Geometry, config: &GeometryExportConfig) -> Vec<(String, String)> {
        export_geometry(geometry, config)
            .expect("geometry expected to export")
            .into_iter()
            .collect()
    }

    #[test]
    fn a_2d_point_exports_x_and_y() {
        let geometry = Geometry::Euclidean2D(point_2d(euclidean(), [1.5, 2.5]));
        assert_eq!(
            exported(&geometry, &coordinates_config(None)),
            vec![
                ("x".to_string(), "1.5".to_string()),
                ("y".to_string(), "2.5".to_string())
            ]
        );
    }

    // A 2D point has no height, so a configured z column is left for `csv.rs` to
    // fill with an empty string.
    #[test]
    fn a_2d_point_leaves_a_configured_z_column_unset() {
        let geometry = Geometry::Euclidean2D(point_2d(euclidean(), [1.0, 2.0]));
        let columns = exported(&geometry, &coordinates_config(Some("z")));
        assert_eq!(columns.len(), 2);
        assert!(!columns.iter().any(|(name, _)| name == "z"));
    }

    // Coordinates mode is Point-only; a collection holding one point included. This
    // is the `export_geometry`-level counterpart of
    // `extract_coordinates_on_a_non_point_geometry_errors` above.
    #[test]
    fn a_non_point_geometry_cannot_export_coordinates() {
        let geometry = collection_2d(vec![point_2d(euclidean(), [0.0, 0.0])]);
        assert!(matches!(
            export_geometry(&geometry, &coordinates_config(None)),
            Err(GeometryExportError::NonPointGeometry)
        ));
    }

    // Unlike WKT mode, which writes an empty cell.
    #[test]
    fn an_absent_geometry_cannot_export_coordinates() {
        assert!(matches!(
            export_geometry(&Geometry::None, &coordinates_config(None)),
            Err(GeometryExportError::EmptyGeometry)
        ));
    }

    // A north-first CRS is swapped here too, so x is always the easting.
    #[test]
    fn coordinates_are_exported_east_first() {
        let geometry = Geometry::Euclidean2D(point_2d(
            CoordinateFrame::Crs(EpsgCode::new(6675)),
            [1.0, 2.0],
        ));
        assert_eq!(
            exported(&geometry, &coordinates_config(None)),
            vec![
                ("x".to_string(), "2".to_string()),
                ("y".to_string(), "1".to_string())
            ]
        );
    }

    // A uniform collection is the new geometry's `Multi*`, so it folds back.
    #[test]
    fn a_uniform_collection_folds_into_a_multi() {
        let geometry = collection_2d(vec![
            point_2d(euclidean(), [0.0, 0.0]),
            point_2d(euclidean(), [1.0, 1.0]),
        ]);
        assert_eq!(wkt_of(&geometry), "MULTIPOINT(0 0, 1 1)");
    }

    #[test]
    fn a_uniform_collection_of_areas_folds_into_a_multipolygon() {
        let square = |offset: f64| {
            Euclidean2DGeometry::Polygon(Box::new(Polygon2D::from_rings(
                euclidean(),
                [
                    [offset, offset],
                    [offset + 1.0, offset],
                    [offset + 1.0, offset + 1.0],
                    [offset, offset],
                ],
                Vec::<Vec<[f64; 2]>>::new(),
            )))
        };
        let geometry = collection_2d(vec![square(0.0), square(3.0)]);
        assert_eq!(
            wkt_of(&geometry),
            "MULTIPOLYGON(((0 0, 1 0, 1 1, 0 0)), ((3 3, 4 3, 4 4, 3 3)))"
        );
    }

    // Members of different families have no `Multi*` form covering them.
    #[test]
    fn a_mixed_collection_writes_a_geometrycollection() {
        let geometry = collection_2d(vec![
            point_2d(euclidean(), [0.0, 0.0]),
            Euclidean2DGeometry::LineString(LineString2D::from_coords(
                euclidean(),
                [[1.0, 1.0], [2.0, 2.0]],
            )),
        ]);
        assert_eq!(
            wkt_of(&geometry),
            "GEOMETRYCOLLECTION(POINT(0 0), LINESTRING(1 1, 2 2))"
        );
    }

    // Folding members from different frames would put coordinates from two
    // reference systems in one geometry that names neither.
    #[test]
    fn members_in_different_frames_do_not_fold() {
        let geometry = collection_2d(vec![
            point_2d(euclidean(), [0.0, 0.0]),
            point_2d(CoordinateFrame::Crs(EpsgCode::new(3857)), [1.0, 1.0]),
        ]);
        assert_eq!(
            wkt_of(&geometry),
            "GEOMETRYCOLLECTION(POINT(0 0), POINT(1 1))"
        );
    }

    // The top-level container is cross-dimensional, so no `Multi*` describes it.
    #[test]
    fn the_top_level_collection_always_writes_a_geometrycollection() {
        let geometry = Geometry::GeometryCollection(GeometryCollection::new(vec![
            Geometry::Euclidean2D(point_2d(euclidean(), [0.0, 0.0])),
            Geometry::Euclidean2D(point_2d(euclidean(), [1.0, 1.0])),
        ]));
        assert_eq!(
            wkt_of(&geometry),
            "GEOMETRYCOLLECTION(POINT(0 0), POINT(1 1))"
        );
    }

    // A nested `Multi*` flattens into the fold rather than nesting inside it.
    #[test]
    fn a_nested_uniform_collection_flattens_when_folded() {
        let inner = Euclidean2DGeometry::Collection(Collection2D::new(vec![
            point_2d(euclidean(), [1.0, 1.0]),
            point_2d(euclidean(), [2.0, 2.0]),
        ]));
        let geometry = collection_2d(vec![point_2d(euclidean(), [0.0, 0.0]), inner]);
        assert_eq!(wkt_of(&geometry), "MULTIPOINT(0 0, 1 1, 2 2)");
    }

    // Nothing writable under it means the geometry itself is unwritable: the spec
    // has an unwritable geometry error so `csv.rs` can warn and count it as a
    // failure, rather than `geometry_to_wkt` swallowing it into a silent empty
    // cell. The cell the CSV row ends up with is still empty either way, that
    // happens in `csv.rs`'s `Err` branch, not here.
    #[test]
    fn an_empty_collection_cannot_be_written() {
        assert!(matches!(
            geometry_to_wkt(&collection_2d(Vec::new())),
            Err(GeometryExportError::UnsupportedGeometryCollection)
        ));
    }

    // A container with one writable and one unwritable member still writes the
    // writable one; the unwritable one is only an omission, not a failure of the
    // whole geometry. The asymmetric coordinate would also catch an axis-order
    // slip in the surviving member.
    #[test]
    fn a_partially_writable_collection_writes_its_writable_members() {
        let geometry = collection_2d(vec![
            point_2d(euclidean(), [1.0, 0.0]),
            Euclidean2DGeometry::Collection(Collection2D::new(Vec::new())),
        ]);
        assert_eq!(wkt_of(&geometry), "MULTIPOINT(1 0)");
    }

    // A mesh writes as its faces, folding into a MULTIPOLYGON the way a collection
    // writes as its members. `Split` closes each face's ring itself.
    #[test]
    fn a_triangular_mesh_writes_its_faces() {
        let mesh = TriangularMesh3D::from_soup(
            euclidean(),
            [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
        );
        let geometry = Geometry::Euclidean3D(Euclidean3DGeometry::TriangularMesh(Box::new(mesh)));
        assert_eq!(
            wkt_of(&geometry),
            "MULTIPOLYGON(((0 0 0, 1 0 0, 0 1 0, 0 0 0)))"
        );
    }

    // A mesh with no face has nothing to write. `geometry_to_wkt` (not `wkt_of`,
    // which unwraps) is used directly: an unwritable geometry is an `Err` at this
    // layer, matching `an_empty_collection_cannot_be_written` above; `csv.rs` is
    // what turns that `Err` into the empty cell the row ends up with.
    // The label pinned here is the mesh kind (`write_faces`'s `Parts::of` empty
    // fallback), not a `Debug` dump of the mesh.
    #[test]
    fn an_empty_mesh_writes_an_empty_cell() {
        let mesh = TriangularMesh3D::from_soup(euclidean(), Vec::<[f64; 3]>::new());
        let geometry = Geometry::Euclidean3D(Euclidean3DGeometry::TriangularMesh(Box::new(mesh)));
        assert_unsupported_geometry_type(geometry_to_wkt(&geometry), "TriangularMesh");
    }

    // WKT has no volume, and a PointCloud would emit a position per sample even
    // though a MULTIPOINT could hold one. Both match the GeoJSON writer's
    // refusals, which are also `Err` at this layer (see the comment above).
    //
    // The label is pinned rather than wildcard-matched: `PointCloud`'s `Debug`
    // prints every sample, so a regression back to `format!("{geometry:?}")`
    // would not be caught by a wildcard `UnsupportedGeometryType(_)`.
    #[test]
    fn a_point_cloud_writes_an_empty_cell() {
        let cloud = PointCloud::from_positions(euclidean(), [[0.0, 0.0, 0.0]]);
        let geometry = Geometry::Euclidean3D(Euclidean3DGeometry::PointCloud(Box::new(cloud)));
        assert_unsupported_geometry_type(geometry_to_wkt(&geometry), "PointCloud");
    }

    // WKT has no volume, nor the boolean tree built from volumes. A PLATEAU
    // `lod1Solid`/`lod2Solid` parses to exactly this variant, and `Solid`'s
    // `Debug` prints every shell, mesh and vertex, so this is the case the label
    // fix matters most for.
    #[test]
    fn a_solid_writes_an_empty_cell() {
        let shell = TriangularMesh3DData::from_parts(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            [0u32, 1, 2],
        )
        .expect("triangle soup expected to build a valid mesh");
        let solid = Solid::from_exterior(euclidean(), shell);
        let geometry = Geometry::Euclidean3D(Euclidean3DGeometry::Solid(Box::new(solid)));
        assert_unsupported_geometry_type(geometry_to_wkt(&geometry), "Solid");
    }

    // The EPSG column: writes the geometry's code when its coordinates resolve to
    // exactly one CRS, and is otherwise left for `csv.rs` to pad with an empty
    // string, whichever export mode is configured.

    fn wkt_config_with_epsg(epsg_column: Option<&str>) -> GeometryExportConfig {
        GeometryExportConfig {
            mode: GeometryExportMode::Wkt {
                column: "geometry".to_string(),
            },
            epsg_column: epsg_column.map(str::to_string),
        }
    }

    // The position is symmetric so the assertion holds regardless of whether
    // EPSG:4326 turns out to swap its horizontal pair; this test is only about
    // the EPSG column, not axis order (covered elsewhere).
    #[test]
    fn wkt_mode_writes_the_epsg_code_of_a_single_crs_geometry() {
        let geometry = Geometry::Euclidean2D(point_2d(
            CoordinateFrame::Crs(EpsgCode::new(4326)),
            [1.0, 1.0],
        ));
        let columns = exported(&geometry, &wkt_config_with_epsg(Some("epsg")));
        assert_eq!(
            columns,
            vec![
                ("geometry".to_string(), "POINT(1 1)".to_string()),
                ("epsg".to_string(), "4326".to_string()),
            ]
        );
    }

    #[test]
    fn wkt_mode_leaves_the_epsg_column_unset_for_a_euclidean_geometry() {
        let geometry = Geometry::Euclidean2D(point_2d(euclidean(), [1.0, 2.0]));
        let columns = exported(&geometry, &wkt_config_with_epsg(Some("epsg")));
        assert_eq!(columns.len(), 1);
        assert!(!columns.iter().any(|(name, _)| name == "epsg"));
    }

    // Members in two different reference systems fold into a GEOMETRYCOLLECTION,
    // not a CRS: the cell has nowhere to put two codes, so it gets none.
    #[test]
    fn wkt_mode_leaves_the_epsg_column_unset_for_a_mixed_crs_collection() {
        let geometry = collection_2d(vec![
            point_2d(CoordinateFrame::Crs(EpsgCode::new(4326)), [0.0, 0.0]),
            point_2d(CoordinateFrame::Crs(EpsgCode::new(3857)), [1.0, 1.0]),
        ]);
        let columns = exported(&geometry, &wkt_config_with_epsg(Some("epsg")));
        assert_eq!(columns.len(), 1);
        assert!(!columns.iter().any(|(name, _)| name == "epsg"));
    }

    #[test]
    fn wkt_mode_has_no_epsg_column_when_none_is_configured() {
        let geometry = Geometry::Euclidean2D(point_2d(
            CoordinateFrame::Crs(EpsgCode::new(4326)),
            [1.0, 2.0],
        ));
        let columns = exported(&geometry, &wkt_config_with_epsg(None));
        assert_eq!(columns.len(), 1);
        assert!(!columns.iter().any(|(name, _)| name == "epsg"));
    }

    fn coordinates_config_with_epsg(epsg_column: Option<&str>) -> GeometryExportConfig {
        GeometryExportConfig {
            mode: GeometryExportMode::Coordinates {
                x_column: "x".to_string(),
                y_column: "y".to_string(),
                z_column: None,
            },
            epsg_column: epsg_column.map(str::to_string),
        }
    }

    // Symmetric position, for the same reason as the WKT-mode test above.
    #[test]
    fn coordinates_mode_writes_the_epsg_code_of_a_single_crs_point() {
        let geometry = Geometry::Euclidean2D(point_2d(
            CoordinateFrame::Crs(EpsgCode::new(4326)),
            [1.0, 1.0],
        ));
        let columns = exported(&geometry, &coordinates_config_with_epsg(Some("epsg")));
        assert_eq!(
            columns,
            vec![
                ("x".to_string(), "1".to_string()),
                ("y".to_string(), "1".to_string()),
                ("epsg".to_string(), "4326".to_string()),
            ]
        );
    }

    #[test]
    fn coordinates_mode_leaves_the_epsg_column_unset_for_a_euclidean_point() {
        let geometry = Geometry::Euclidean2D(point_2d(euclidean(), [1.0, 2.0]));
        let columns = exported(&geometry, &coordinates_config_with_epsg(Some("epsg")));
        assert_eq!(columns.len(), 2);
        assert!(!columns.iter().any(|(name, _)| name == "epsg"));
    }

    #[test]
    fn coordinates_mode_has_no_epsg_column_when_none_is_configured() {
        let geometry = Geometry::Euclidean2D(point_2d(
            CoordinateFrame::Crs(EpsgCode::new(4326)),
            [1.0, 2.0],
        ));
        let columns = exported(&geometry, &coordinates_config_with_epsg(None));
        assert_eq!(columns.len(), 2);
        assert!(!columns.iter().any(|(name, _)| name == "epsg"));
    }
}
