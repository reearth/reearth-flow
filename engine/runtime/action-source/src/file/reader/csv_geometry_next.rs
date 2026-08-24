//! The new-geometry CSV geometry parser.
//!
//! Maps parsed input straight to the new geometry model. The old-world parser
//! converts through `geo_types` on its WKT path, and `geo_types` is 2D-only, so
//! it silently discards Z. Nothing here touches `geo_types`.

use indexmap::IndexMap;
use reearth_flow_geometry::coordinate::{CoordinateFrame, EpsgCode};
use reearth_flow_geometry::point::{Point2D, Point3D};
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
        // Task 3 replaces this arm with a real WKT parser.
        GeometryMode::Wkt { .. } => Err(GeometryParsingError::WktParsing(
            "WKT parsing is not yet implemented for the new geometry model".to_string(),
        )),
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
}
