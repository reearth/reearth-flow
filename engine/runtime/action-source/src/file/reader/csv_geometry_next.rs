//! The new-geometry CSV geometry parser.
//!
//! Maps parsed input straight to the new geometry model. The old-world parser
//! converts through `geo_types` on its WKT path, and `geo_types` is 2D-only, so
//! it silently discards Z. Nothing here touches `geo_types`.

use std::str::FromStr;

use indexmap::IndexMap;
use reearth_flow_geometry::coordinate::{CoordinateFrame, EpsgCode};
use reearth_flow_geometry::line_string::{LineString2D, LineString3D};
use reearth_flow_geometry::point::{Point2D, Point3D};
use reearth_flow_geometry::polygon::{Polygon2D, Polygon3D};
use reearth_flow_geometry::{Euclidean2DGeometry, Euclidean3DGeometry, Geometry};

use super::{GeometryConfig, GeometryMode};
use crate::errors::GeometryParsingError;

/// The coordinate frame a config declares: a CRS when it names a non-zero EPSG,
/// otherwise bare Euclidean space. Mirrors `geopackage_next.rs`.
pub(crate) fn frame_for(config: &GeometryConfig) -> CoordinateFrame {
    match config.epsg {
        Some(code) if code > 0 => CoordinateFrame::Crs(EpsgCode::new(code)),
        _ => CoordinateFrame::Euclidean,
    }
}

/// Read one named column as an `f64`, naming the column and the offending value
/// on failure. That text reaches the user in the rejected row's error
/// attribute, so it has to be specific.
fn number(row: &IndexMap<String, String>, column: &str) -> Result<f64, GeometryParsingError> {
    let raw = row
        .get(column)
        .ok_or_else(|| GeometryParsingError::ColumnNotFound(column.to_string()))?;
    raw.parse()
        .map_err(|_| GeometryParsingError::InvalidCoordinate {
            column: column.to_string(),
            value: raw.clone(),
        })
}

/// Parse one row's geometry per `config`.
pub fn parse_geometry(
    row: &IndexMap<String, String>,
    config: &GeometryConfig,
) -> Result<Geometry, GeometryParsingError> {
    let frame = frame_for(config);
    match &config.mode {
        GeometryMode::Coordinates {
            x_column,
            y_column,
            z_column,
        } => {
            let x = number(row, x_column)?;
            let y = number(row, y_column)?;
            match z_column {
                None => Ok(Geometry::Euclidean2D(Euclidean2DGeometry::Point(
                    Point2D::new(frame, [x, y]),
                ))),
                Some(z_column) => {
                    let z = number(row, z_column)?;
                    Ok(Geometry::Euclidean3D(Euclidean3DGeometry::Point(
                        Point3D::new(frame, [x, y, z]),
                    )))
                }
            }
        }
        GeometryMode::Wkt { column } => {
            let text = row
                .get(column)
                .ok_or_else(|| GeometryParsingError::ColumnNotFound(column.clone()))?
                .trim();
            if text.is_empty() {
                return Ok(Geometry::None);
            }
            let parsed = parse_wkt_tolerantly(text)?;
            wkt_to_geometry(parsed, frame)
        }
    }
}

/// Parse WKT, accepting the bare 3D form alongside the tagged OGC form.
///
/// The `wkt` crate requires an explicit dimension tag: `POINT Z(1 2 3)` parses
/// and `POINT(1 2 3)` does not. Our own CSV Writer emits the bare form (a
/// reviewed parity decision with the old writer), so reading our own output
/// needs this. On failure we count the ordinates of the first coordinate group
/// and retry once with a `Z` tag inserted.
///
/// Bare three-ordinate WKT is formally ambiguous between XYZ and XYM. We treat
/// it as XYZ, matching both our writer and the older PostGIS convention.
fn parse_wkt_tolerantly(text: &str) -> Result<wkt::Wkt<f64>, GeometryParsingError> {
    match wkt::Wkt::<f64>::from_str(text) {
        Ok(parsed) => Ok(parsed),
        Err(original) => match with_z_tag(text) {
            Some(tagged) => wkt::Wkt::<f64>::from_str(&tagged)
                // Report the ORIGINAL error, not the retry's: the retry is our
                // invention, and its message would confuse a user whose input
                // was simply malformed.
                .map_err(|_| GeometryParsingError::WktParsing(original.to_string())),
            None => Err(GeometryParsingError::WktParsing(original.to_string())),
        },
    }
}

/// `text` with a ` Z` tag inserted after the type keyword, when its first
/// coordinate group holds exactly three ordinates. `None` when the shape does
/// not look like bare 3D, in which case there is nothing to retry.
fn with_z_tag(text: &str) -> Option<String> {
    let open = text.find('(')?;
    let keyword = text[..open].trim_end();
    // An already-tagged or EMPTY input is not bare 3D.
    if keyword.is_empty() || keyword.split_whitespace().count() > 1 {
        return None;
    }
    // The first coordinate group is the run after the opening parens, so
    // `POLYGON((0 0 0, ...` and `POINT(0 0 0)` both work.
    let after_parens = text[open..].trim_start_matches(['(', ' ']);
    let first_coord = after_parens.split([',', ')']).next()?.trim();
    if first_coord.split_whitespace().count() != 3 {
        return None;
    }
    Some(format!("{keyword} Z{}", &text[open..]))
}

/// One `wkt` geometry as a new-model `Geometry`.
///
/// `EMPTY` forms parse successfully with an absent coord or empty ring lists, so
/// every arm checks for emptiness rather than unwrapping.
fn wkt_to_geometry(
    parsed: wkt::Wkt<f64>,
    frame: CoordinateFrame,
) -> Result<Geometry, GeometryParsingError> {
    use wkt::Wkt;
    match parsed {
        Wkt::Point(point) => match point.coord() {
            None => Ok(Geometry::None),
            Some(coord) => Ok(point_geometry(coord, frame)),
        },
        Wkt::LineString(line) => {
            let coords = line.coords();
            if coords.is_empty() {
                return Ok(Geometry::None);
            }
            Ok(line_geometry(coords, frame))
        }
        Wkt::Polygon(polygon) => {
            let rings = polygon.rings();
            if rings.is_empty() {
                return Ok(Geometry::None);
            }
            Ok(polygon_geometry(rings, frame))
        }
        other => Err(GeometryParsingError::UnsupportedGeometryType(format!(
            "{other:?}"
        ))),
    }
}

/// Whether any coord in a run carries a Z ordinate. A run is 3D if any of its
/// coords is, so a mixed run is lifted rather than truncated.
fn is_3d(coords: &[wkt::types::Coord<f64>]) -> bool {
    coords.iter().any(|c| c.z.is_some())
}

fn point_geometry(coord: &wkt::types::Coord<f64>, frame: CoordinateFrame) -> Geometry {
    match coord.z {
        None => Geometry::Euclidean2D(Euclidean2DGeometry::Point(Point2D::new(
            frame,
            [coord.x, coord.y],
        ))),
        Some(z) => Geometry::Euclidean3D(Euclidean3DGeometry::Point(Point3D::new(
            frame,
            [coord.x, coord.y, z],
        ))),
    }
}

fn line_geometry(coords: &[wkt::types::Coord<f64>], frame: CoordinateFrame) -> Geometry {
    if is_3d(coords) {
        Geometry::Euclidean3D(Euclidean3DGeometry::LineString(LineString3D::from_coords(
            frame,
            coords.iter().map(|c| [c.x, c.y, c.z.unwrap_or(0.0)]),
        )))
    } else {
        Geometry::Euclidean2D(Euclidean2DGeometry::LineString(LineString2D::from_coords(
            frame,
            coords.iter().map(|c| [c.x, c.y]),
        )))
    }
}

fn polygon_geometry(rings: &[wkt::types::LineString<f64>], frame: CoordinateFrame) -> Geometry {
    let any_3d = rings.iter().any(|r| is_3d(r.coords()));
    let exterior = rings[0].coords();
    let interiors = &rings[1..];
    if any_3d {
        Geometry::Euclidean3D(Euclidean3DGeometry::Polygon(Box::new(
            Polygon3D::from_rings(
                frame,
                exterior
                    .iter()
                    .map(|c| [c.x, c.y, c.z.unwrap_or(0.0)])
                    .collect::<Vec<_>>(),
                interiors
                    .iter()
                    .map(|r| {
                        r.coords()
                            .iter()
                            .map(|c| [c.x, c.y, c.z.unwrap_or(0.0)])
                            .collect::<Vec<_>>()
                    })
                    .collect::<Vec<_>>(),
            ),
        )))
    } else {
        Geometry::Euclidean2D(Euclidean2DGeometry::Polygon(Box::new(
            Polygon2D::from_rings(
                frame,
                exterior.iter().map(|c| [c.x, c.y]).collect::<Vec<_>>(),
                interiors
                    .iter()
                    .map(|r| r.coords().iter().map(|c| [c.x, c.y]).collect::<Vec<_>>())
                    .collect::<Vec<_>>(),
            ),
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reearth_flow_geometry::coordinate::{CoordinateFrame, EpsgCode};
    use reearth_flow_geometry::{Euclidean2DGeometry, Euclidean3DGeometry, Geometry};

    fn row(pairs: &[(&str, &str)]) -> IndexMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    fn coords_config(z: Option<&str>, epsg: Option<u16>) -> GeometryConfig {
        GeometryConfig {
            mode: GeometryMode::Coordinates {
                x_column: "lon".to_string(),
                y_column: "lat".to_string(),
                z_column: z.map(|s| s.to_string()),
            },
            epsg,
        }
    }

    #[test]
    fn x_and_y_build_a_2d_point() {
        let g = parse_geometry(
            &row(&[("lon", "1.5"), ("lat", "2.5")]),
            &coords_config(None, None),
        )
        .unwrap();
        match g {
            Geometry::Euclidean2D(Euclidean2DGeometry::Point(p)) => {
                assert_eq!(*p.frame(), CoordinateFrame::Euclidean);
            }
            other => panic!("expected a 2D point, got {other:?}"),
        }
    }

    #[test]
    fn x_y_and_z_build_a_3d_point() {
        let g = parse_geometry(
            &row(&[("lon", "1.0"), ("lat", "2.0"), ("h", "3.0")]),
            &coords_config(Some("h"), None),
        )
        .unwrap();
        match g {
            Geometry::Euclidean3D(Euclidean3DGeometry::Point(_)) => {}
            other => panic!("expected a 3D point, got {other:?}"),
        }
    }

    /// A declared EPSG becomes a CRS frame; its absence stays Euclidean.
    #[test]
    fn an_epsg_becomes_a_crs_frame() {
        assert_eq!(
            frame_for(&coords_config(None, Some(6677))),
            CoordinateFrame::Crs(EpsgCode::new(6677))
        );
        assert_eq!(
            frame_for(&coords_config(None, None)),
            CoordinateFrame::Euclidean
        );
        // Zero is not a CRS.
        assert_eq!(
            frame_for(&coords_config(None, Some(0))),
            CoordinateFrame::Euclidean
        );
    }

    /// The error names the column and the value, because that text is what
    /// reaches the user in the rejected row's error attribute.
    #[test]
    fn a_non_numeric_cell_errors_naming_the_column_and_value() {
        let err = parse_geometry(
            &row(&[("lon", "abc"), ("lat", "2.0")]),
            &coords_config(None, None),
        )
        .unwrap_err();
        let text = err.to_string();
        assert!(text.contains("lon"), "{text}");
        assert!(text.contains("abc"), "{text}");
    }

    #[test]
    fn a_missing_column_errors_naming_the_column() {
        let err = parse_geometry(&row(&[("lat", "2.0")]), &coords_config(None, None)).unwrap_err();
        assert!(err.to_string().contains("lon"), "{err}");
    }

    fn wkt_config(epsg: Option<u16>) -> GeometryConfig {
        GeometryConfig {
            mode: GeometryMode::Wkt {
                column: "geom".to_string(),
            },
            epsg,
        }
    }

    fn parse(text: &str) -> Result<Geometry, GeometryParsingError> {
        parse_geometry(&row(&[("geom", text)]), &wkt_config(None))
    }

    #[test]
    fn a_2d_point_parses() {
        match parse("POINT(1 2)").unwrap() {
            Geometry::Euclidean2D(Euclidean2DGeometry::Point(_)) => {}
            other => panic!("expected a 2D point, got {other:?}"),
        }
    }

    /// The case the old reader silently truncated to 2D by hopping through
    /// `geo_types`. Z must survive.
    #[test]
    fn an_ogc_3d_point_keeps_its_z() {
        match parse("POINT Z(1 2 3)").unwrap() {
            Geometry::Euclidean3D(Euclidean3DGeometry::Point(p)) => {
                assert_eq!(p.position()[2], 3.0);
            }
            other => panic!("expected a 3D point, got {other:?}"),
        }
    }

    #[test]
    fn a_linestring_parses_in_both_dimensions() {
        assert!(matches!(
            parse("LINESTRING(0 0, 1 1)").unwrap(),
            Geometry::Euclidean2D(Euclidean2DGeometry::LineString(_))
        ));
        assert!(matches!(
            parse("LINESTRING Z(0 0 0, 1 1 1)").unwrap(),
            Geometry::Euclidean3D(Euclidean3DGeometry::LineString(_))
        ));
    }

    #[test]
    fn a_polygon_keeps_its_hole() {
        let text = "POLYGON((0 0, 4 0, 4 4, 0 4, 0 0), (1 1, 2 1, 2 2, 1 2, 1 1))";
        match parse(text).unwrap() {
            Geometry::Euclidean2D(Euclidean2DGeometry::Polygon(p)) => {
                assert_eq!(p.interiors().count(), 1, "the hole must survive");
            }
            other => panic!("expected a 2D polygon, got {other:?}"),
        }
    }

    /// EMPTY is a parse *success* in the wkt crate, yielding `coord: None` and
    /// empty ring lists. An `unwrap()` on those panics on valid input.
    #[test]
    fn empty_geometries_become_an_absent_geometry_not_a_panic() {
        for text in ["POINT EMPTY", "POLYGON EMPTY", "LINESTRING EMPTY"] {
            assert!(
                matches!(parse(text).unwrap(), Geometry::None),
                "{text} should map to Geometry::None"
            );
        }
    }

    /// A blank cell is a row without geometry, not a malformed one.
    #[test]
    fn a_blank_cell_is_an_absent_geometry_not_an_error() {
        assert!(matches!(parse("").unwrap(), Geometry::None));
        assert!(matches!(parse("   ").unwrap(), Geometry::None));
    }

    #[test]
    fn malformed_wkt_errors_rather_than_panicking() {
        assert!(parse("NOT WKT AT ALL").is_err());
        assert!(parse("POINT(").is_err());
    }

    /// Our own CSV Writer emits the bare 3D form. The `wkt` crate refuses it, so
    /// reading back what we wrote used to abort the whole file. Verified
    /// against wkt 0.14: `POINT(1 2 3)` fails with "Missing closing parenthesis
    /// for type".
    #[test]
    fn the_bare_3d_form_our_writer_emits_is_accepted() {
        match parse("POINT(1 2 3)").unwrap() {
            Geometry::Euclidean3D(Euclidean3DGeometry::Point(p)) => {
                assert_eq!(p.position()[2], 3.0);
            }
            other => panic!("expected a 3D point, got {other:?}"),
        }
    }

    #[test]
    fn bare_3d_linestrings_and_polygons_are_accepted() {
        assert!(matches!(
            parse("LINESTRING(0 0 0, 1 1 1)").unwrap(),
            Geometry::Euclidean3D(Euclidean3DGeometry::LineString(_))
        ));
        assert!(matches!(
            parse("POLYGON((0 0 0, 4 0 0, 4 4 1, 0 0 0))").unwrap(),
            Geometry::Euclidean3D(Euclidean3DGeometry::Polygon(_))
        ));
    }

    /// Nested parens are where a normalising pre-pass is most likely to go
    /// wrong, so a holed bare-3D polygon is the case that matters.
    #[test]
    fn a_bare_3d_polygon_with_a_hole_keeps_both_rings() {
        let text = "POLYGON((0 0 0, 9 0 0, 9 9 0, 0 9 0, 0 0 0), (1 1 0, 2 1 0, 2 2 0, 1 1 0))";
        match parse(text).unwrap() {
            Geometry::Euclidean3D(Euclidean3DGeometry::Polygon(p)) => {
                assert_eq!(p.interiors().count(), 1);
            }
            other => panic!("expected a 3D polygon, got {other:?}"),
        }
    }

    /// Tolerance must not swallow real errors, and must not mangle 2D input.
    #[test]
    fn tolerance_does_not_change_2d_or_hide_malformed_input() {
        assert!(matches!(
            parse("POINT(1 2)").unwrap(),
            Geometry::Euclidean2D(Euclidean2DGeometry::Point(_))
        ));
        assert!(parse("POINT(1 2 3 4 5)").is_err());
        assert!(parse("POINT EMPTY").is_ok());
    }
}
