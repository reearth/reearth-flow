use indexmap::IndexMap;
use nusamai_projection::crs::EpsgCode;
use reearth_flow_geometry::types::{
    geometry::{Geometry2D, Geometry3D},
    line_string::LineString2D,
    multi_line_string::MultiLineString2D,
    multi_point::MultiPoint2D,
    multi_polygon::MultiPolygon2D,
    point::{Point2D, Point3D},
    polygon::Polygon2D,
};
use reearth_flow_types::{Geometry, GeometryValue};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// # Geometry Configuration
/// Configure how geometry data is extracted from CSV columns
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct GeometryConfig {
    /// # Geometry Mode
    /// Specify how geometry is represented in the CSV
    #[serde(flatten)]
    pub mode: GeometryMode,
    /// # EPSG Code
    /// Coordinate Reference System code (e.g., 4326 for WGS84)
    pub epsg: Option<u16>,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase", tag = "geometryMode")]
pub enum GeometryMode {
    /// # WKT Column
    /// Geometry stored as Well-Known Text in a single column
    Wkt {
        /// # WKT Column Name
        /// Name of the column containing WKT geometry
        column: String,
    },
    /// # Coordinate Columns
    /// Geometry stored as separate X, Y, (optional Z) columns
    Coordinates {
        /// # X Column Name
        /// Name of the column containing X coordinate (longitude)
        x_column: String,
        /// # Y Column Name
        /// Name of the column containing Y coordinate (latitude)
        y_column: String,
        /// # Z Column Name
        /// Optional name of the column containing Z coordinate (elevation)
        #[serde(skip_serializing_if = "Option::is_none")]
        z_column: Option<String>,
    },
}

/// Get the names of columns that are used for geometry
pub fn get_geometry_column_names(config: &GeometryConfig) -> Vec<String> {
    match &config.mode {
        GeometryMode::Wkt { column } => vec![column.clone()],
        GeometryMode::Coordinates {
            x_column,
            y_column,
            z_column,
        } => {
            let mut cols = vec![x_column.clone(), y_column.clone()];
            if let Some(z) = z_column {
                cols.push(z.clone());
            }
            cols
        }
    }
}

pub fn parse_geometry(
    row: &IndexMap<String, String>,
    config: &GeometryConfig,
) -> Result<Geometry, String> {
    let epsg = config.epsg.map(EpsgCode::from);

    match &config.mode {
        GeometryMode::Wkt { column } => {
            let wkt_str = row
                .get(column)
                .ok_or_else(|| format!("WKT column '{}' not found", column))?;
            parse_wkt_geometry(wkt_str, epsg)
        }
        GeometryMode::Coordinates {
            x_column,
            y_column,
            z_column,
        } => {
            let x_str = row
                .get(x_column)
                .ok_or_else(|| format!("X column '{}' not found", x_column))?;
            let y_str = row
                .get(y_column)
                .ok_or_else(|| format!("Y column '{}' not found", y_column))?;

            let x: f64 = x_str
                .parse()
                .map_err(|_| format!("Invalid X coordinate: {}", x_str))?;
            let y: f64 = y_str
                .parse()
                .map_err(|_| format!("Invalid Y coordinate: {}", y_str))?;

            if let Some(z_col) = z_column {
                let z_str = row
                    .get(z_col)
                    .ok_or_else(|| format!("Z column '{}' not found", z_col))?;
                let z: f64 = z_str
                    .parse()
                    .map_err(|_| format!("Invalid Z coordinate: {}", z_str))?;

                Ok(Geometry {
                    epsg,
                    value: GeometryValue::FlowGeometry3D(Geometry3D::Point(Point3D::from([
                        x, y, z,
                    ]))),
                })
            } else {
                Ok(Geometry {
                    epsg,
                    value: GeometryValue::FlowGeometry2D(Geometry2D::Point(Point2D::from([x, y]))),
                })
            }
        }
    }
}

fn parse_wkt_geometry(wkt_str: &str, epsg: Option<EpsgCode>) -> Result<Geometry, String> {
    // Trim whitespace
    let wkt_str = wkt_str.trim();
    if wkt_str.is_empty() {
        return Ok(Geometry::default());
    }

    // Parse WKT string
    let wkt: wkt::Wkt<f64> =
        wkt::Wkt::from_str(wkt_str).map_err(|e| format!("Failed to parse WKT: {}", e))?;

    // Convert WKT geometry to geo_types
    let geo_geom: geo_types::Geometry<f64> = geo_types::Geometry::try_from(wkt)
        .map_err(|e| format!("Failed to convert WKT to geometry: {:?}", e))?;

    // Convert geo_types to Flow geometry
    convert_geo_to_flow(geo_geom, epsg)
}

fn convert_geo_to_flow(
    geo_geom: geo_types::Geometry<f64>,
    epsg: Option<EpsgCode>,
) -> Result<Geometry, String> {
    use geo_types::Geometry as GeoGeometry;

    match geo_geom {
        GeoGeometry::Point(pt) => {
            let coord = pt.0;
            Ok(Geometry {
                epsg,
                value: GeometryValue::FlowGeometry2D(Geometry2D::Point(Point2D::from([
                    coord.x, coord.y,
                ]))),
            })
        }
        GeoGeometry::LineString(ls) => {
            let coords: Vec<(f64, f64)> = ls.0.iter().map(|c| (c.x, c.y)).collect();
            Ok(Geometry {
                epsg,
                value: GeometryValue::FlowGeometry2D(Geometry2D::LineString(LineString2D::from(
                    coords,
                ))),
            })
        }
        GeoGeometry::Polygon(poly) => {
            let exterior: Vec<(f64, f64)> = poly.exterior().0.iter().map(|c| (c.x, c.y)).collect();
            let holes: Vec<Vec<(f64, f64)>> = poly
                .interiors()
                .iter()
                .map(|hole| hole.0.iter().map(|c| (c.x, c.y)).collect())
                .collect();

            Ok(Geometry {
                epsg,
                value: GeometryValue::FlowGeometry2D(Geometry2D::Polygon(Polygon2D::new(
                    LineString2D::from(exterior),
                    holes.into_iter().map(LineString2D::from).collect(),
                ))),
            })
        }
        GeoGeometry::MultiPoint(mp) => {
            let points: Vec<Point2D<f64>> =
                mp.0.iter().map(|p| Point2D::from([p.0.x, p.0.y])).collect();
            Ok(Geometry {
                epsg,
                value: GeometryValue::FlowGeometry2D(Geometry2D::MultiPoint(MultiPoint2D::new(
                    points,
                ))),
            })
        }
        GeoGeometry::MultiLineString(mls) => {
            let linestrings: Vec<LineString2D<f64>> = mls
                .0
                .iter()
                .map(|ls| {
                    let coords: Vec<(f64, f64)> = ls.0.iter().map(|c| (c.x, c.y)).collect();
                    LineString2D::from(coords)
                })
                .collect();
            Ok(Geometry {
                epsg,
                value: GeometryValue::FlowGeometry2D(Geometry2D::MultiLineString(
                    MultiLineString2D::new(linestrings),
                )),
            })
        }
        GeoGeometry::MultiPolygon(mpoly) => {
            let polygons: Vec<Polygon2D<f64>> = mpoly
                .0
                .iter()
                .map(|poly| {
                    let exterior: Vec<(f64, f64)> =
                        poly.exterior().0.iter().map(|c| (c.x, c.y)).collect();
                    let holes: Vec<Vec<(f64, f64)>> = poly
                        .interiors()
                        .iter()
                        .map(|hole| hole.0.iter().map(|c| (c.x, c.y)).collect())
                        .collect();
                    Polygon2D::new(
                        LineString2D::from(exterior),
                        holes.into_iter().map(LineString2D::from).collect(),
                    )
                })
                .collect();
            Ok(Geometry {
                epsg,
                value: GeometryValue::FlowGeometry2D(Geometry2D::MultiPolygon(
                    MultiPolygon2D::new(polygons),
                )),
            })
        }
        GeoGeometry::GeometryCollection(_) => {
            Err("GeometryCollection is not yet supported in CSV reader".to_string())
        }
        _ => Err(format!("Unsupported geometry type: {:?}", geo_geom)),
    }
}
