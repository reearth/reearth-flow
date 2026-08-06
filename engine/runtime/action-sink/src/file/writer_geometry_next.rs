//! New-geometry geometry export for the CSV Writer. Reads
//! `reearth_flow_geometry::Geometry`, whose coordinate frame is per-leaf, rather
//! than the old `Geometry { epsg, value }` wrapper. Sibling of the old-world logic
//! in `writer_geometry.rs`; selected under `new-geometry`.

use std::collections::HashSet;
use std::fmt::Write as _;
use std::sync::{Mutex, OnceLock};

use indexmap::IndexMap;
use reearth_flow_geometry::coordinate::{CoordinateFrame, EpsgCode};
use reearth_flow_geometry::{Euclidean2DGeometry, Euclidean3DGeometry, Geometry};

use super::{GeometryExportConfig, GeometryExportMode};
use crate::errors::GeometryExportError;

/// What a geometry writes to: its WKT text and, when it has one, the `MULTI*`
/// family it can fold into. Carried as a value rather than appended to a buffer
/// because folding a collection has to flatten a nested `MULTI*` into its parent,
/// which needs the family after the text is built.
struct WrittenWkt {
    kind: Option<Kind>,
    text: String,
}

/// The `MULTI*` family a written geometry belongs to. A `GEOMETRYCOLLECTION`
/// belongs to none, having no `MULTI*` form to fold into.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Point,
    Curve,
    Area,
}

/// Export geometry to column values based on configuration.
pub fn export_geometry(
    geometry: &Geometry,
    config: &GeometryExportConfig,
) -> Result<IndexMap<String, String>, GeometryExportError> {
    let mut columns = IndexMap::new();

    match &config.mode {
        GeometryExportMode::Wkt { column } => {
            columns.insert(column.clone(), geometry_to_wkt(geometry)?);
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
        }
    }

    Ok(columns)
}

/// Convert geometry to a WKT string.
///
/// An absent geometry writes an empty cell rather than failing the row. Coordinates
/// mode differs, erroring with `EmptyGeometry` — an asymmetry inherited from the
/// old writer and kept for parity.
pub fn geometry_to_wkt(geometry: &Geometry) -> Result<String, GeometryExportError> {
    match geometry {
        Geometry::None => Ok(String::new()),
        geometry => Ok(write_geometry(geometry)?.text),
    }
}

/// Extract X, Y, Z coordinates from Point geometries.
/// Returns (x, y, optional z).
pub fn extract_coordinates(
    geometry: &Geometry,
) -> Result<(f64, f64, Option<f64>), GeometryExportError> {
    match geometry {
        Geometry::None => Err(GeometryExportError::EmptyGeometry),
        Geometry::Euclidean2D(Euclidean2DGeometry::Point(p)) => {
            let [x, y] = p.position();
            Ok((x, y, None))
        }
        Geometry::Euclidean3D(Euclidean3DGeometry::Point(p)) => {
            let [x, y, z] = p.position();
            Ok((x, y, Some(z)))
        }
        _ => Err(GeometryExportError::NonPointGeometry),
    }
}

fn write_geometry(geometry: &Geometry) -> Result<WrittenWkt, GeometryExportError> {
    match geometry {
        Geometry::None => Err(GeometryExportError::EmptyGeometry),
        Geometry::Euclidean2D(g) => write_2d(g),
        Geometry::Euclidean3D(g) => write_3d(g),
        Geometry::GeometryCollection(_) => Err(GeometryExportError::UnsupportedGeometryCollection),
    }
}

fn write_2d(geometry: &Euclidean2DGeometry) -> Result<WrittenWkt, GeometryExportError> {
    use Euclidean2DGeometry::*;
    match geometry {
        Point(p) => Ok(point(p.frame(), p.position())),
        LineString(l) => Ok(curve(l.frame(), l.coords())),
        Polygon(p) => Ok(area(p.frame(), p.exterior(), p.interiors())),
        other => Err(unsupported(other)),
    }
}

fn write_3d(geometry: &Euclidean3DGeometry) -> Result<WrittenWkt, GeometryExportError> {
    use Euclidean3DGeometry::*;
    match geometry {
        Point(p) => Ok(point(p.frame(), p.position())),
        LineString(l) => Ok(curve(l.frame(), l.coords())),
        Polygon(p) => Ok(area(p.frame(), p.exterior(), p.interiors())),
        other => Err(unsupported(other)),
    }
}

// The 2D and 3D leaves differ only in how long a position is, so what turns one
// into WKT is written once, over `N`-element positions.

fn point<const N: usize>(frame: &CoordinateFrame, position: [f64; N]) -> WrittenWkt {
    WrittenWkt {
        kind: Some(Kind::Point),
        text: format!("POINT({})", coordinate(swaps_axes(frame), position)),
    }
}

fn curve<const N: usize>(frame: &CoordinateFrame, coords: &[[f64; N]]) -> WrittenWkt {
    WrittenWkt {
        kind: Some(Kind::Curve),
        text: format!("LINESTRING({})", coordinate_list(swaps_axes(frame), coords)),
    }
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
    WrittenWkt {
        kind: Some(Kind::Area),
        text: format!("POLYGON({rings})"),
    }
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
/// reordered; a height stays where it is. `{}` formatting matches the old writer,
/// so a whole number writes as `1` rather than `1.0`.
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

fn unsupported(geometry: &impl std::fmt::Debug) -> GeometryExportError {
    GeometryExportError::UnsupportedGeometryType(format!("{geometry:?}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use reearth_flow_geometry::{
        coordinate::{BaseFrame, CoordinateFrame, EpsgCode, TangentPlane},
        line_string::{LineString2D, LineString3D},
        point::{Point2D, Point3D},
        polygon::{Polygon2D, Polygon3D},
        Euclidean2DGeometry, Euclidean3DGeometry, Geometry,
    };

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
                [[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0], [0.0, 0.0]],
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
}
