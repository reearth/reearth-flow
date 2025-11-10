use crate::errors::GeometryExportError;
use indexmap::IndexMap;
use reearth_flow_geometry::types::geometry::{Geometry2D, Geometry3D};
use reearth_flow_types::{Geometry, GeometryValue};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt::Write as _;

/// # Geometry Export Configuration
/// Configure how geometry data is written to CSV columns
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct GeometryExportConfig {
    /// # Geometry Mode
    /// Specify how geometry should be written to the CSV
    #[serde(flatten)]
    pub mode: GeometryExportMode,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase", tag = "geometryMode")]
pub enum GeometryExportMode {
    /// # WKT Column
    /// Write geometry as Well-Known Text in a single column
    Wkt {
        /// # WKT Column Name
        /// Name of the column to write WKT geometry
        column: String,
    },
    /// # Coordinate Columns
    /// Write geometry as separate X, Y, (optional Z) columns
    /// Note: Only supports Point geometries. Non-point geometries will be skipped with a warning.
    Coordinates {
        /// # X Column Name
        /// Name of the column for X coordinate (longitude)
        #[serde(rename = "xColumn")]
        x_column: String,
        /// # Y Column Name
        /// Name of the column for Y coordinate (latitude)
        #[serde(rename = "yColumn")]
        y_column: String,
        /// # Z Column Name
        /// Optional name of the column for Z coordinate (elevation)
        #[serde(rename = "zColumn")]
        #[serde(skip_serializing_if = "Option::is_none")]
        z_column: Option<String>,
    },
}

/// Get the names of columns that will be used for geometry output
pub fn get_geometry_column_names(config: &GeometryExportConfig) -> Vec<String> {
    match &config.mode {
        GeometryExportMode::Wkt { column } => vec![column.clone()],
        GeometryExportMode::Coordinates {
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

/// Export geometry to column values based on configuration
pub fn export_geometry(
    geometry: &Geometry,
    config: &GeometryExportConfig,
) -> Result<IndexMap<String, String>, GeometryExportError> {
    let mut columns = IndexMap::new();

    match &config.mode {
        GeometryExportMode::Wkt { column } => {
            let wkt_string = geometry_to_wkt(geometry)?;
            columns.insert(column.clone(), wkt_string);
        }
        GeometryExportMode::Coordinates {
            x_column,
            y_column,
            z_column,
        } => {
            let coords = extract_coordinates(geometry)?;
            columns.insert(x_column.clone(), coords.0.to_string());
            columns.insert(y_column.clone(), coords.1.to_string());
            if let Some(z) = coords.2 {
                if let Some(z_col) = z_column {
                    columns.insert(z_col.clone(), z.to_string());
                }
            }
        }
    }

    Ok(columns)
}

/// Convert Flow geometry to WKT string
pub fn geometry_to_wkt(geometry: &Geometry) -> Result<String, GeometryExportError> {
    match &geometry.value {
        GeometryValue::None => Ok(String::new()),
        GeometryValue::FlowGeometry2D(geom) => geometry_2d_to_wkt(geom),
        GeometryValue::FlowGeometry3D(geom) => geometry_3d_to_wkt(geom),
        GeometryValue::CityGmlGeometry(_) => Err(GeometryExportError::UnsupportedGeometryType(
            "CityGML geometry".to_string(),
        )),
    }
}

fn geometry_2d_to_wkt(geom: &Geometry2D) -> Result<String, GeometryExportError> {
    match geom {
        Geometry2D::Point(pt) => Ok(format!("POINT({} {})", pt.x(), pt.y())),
        Geometry2D::LineString(ls) => {
            let mut wkt = String::from("LINESTRING(");
            for (i, coord) in ls.coords().enumerate() {
                if i > 0 {
                    wkt.push_str(", ");
                }
                write!(wkt, "{} {}", coord.x, coord.y).unwrap();
            }
            wkt.push(')');
            Ok(wkt)
        }
        Geometry2D::Polygon(poly) => {
            let mut wkt = String::from("POLYGON(");
            // Exterior ring
            wkt.push('(');
            for (i, coord) in poly.exterior().coords().enumerate() {
                if i > 0 {
                    wkt.push_str(", ");
                }
                write!(wkt, "{} {}", coord.x, coord.y).unwrap();
            }
            wkt.push(')');
            // Interior rings (holes)
            for hole in poly.interiors() {
                wkt.push_str(", (");
                for (i, coord) in hole.coords().enumerate() {
                    if i > 0 {
                        wkt.push_str(", ");
                    }
                    write!(wkt, "{} {}", coord.x, coord.y).unwrap();
                }
                wkt.push(')');
            }
            wkt.push(')');
            Ok(wkt)
        }
        Geometry2D::MultiPoint(mp) => {
            let mut wkt = String::from("MULTIPOINT(");
            for (i, pt) in mp.iter().enumerate() {
                if i > 0 {
                    wkt.push_str(", ");
                }
                write!(wkt, "{} {}", pt.x(), pt.y()).unwrap();
            }
            wkt.push(')');
            Ok(wkt)
        }
        Geometry2D::MultiLineString(mls) => {
            let mut wkt = String::from("MULTILINESTRING(");
            for (i, ls) in mls.iter().enumerate() {
                if i > 0 {
                    wkt.push_str(", ");
                }
                wkt.push('(');
                for (j, coord) in ls.coords().enumerate() {
                    if j > 0 {
                        wkt.push_str(", ");
                    }
                    write!(wkt, "{} {}", coord.x, coord.y).unwrap();
                }
                wkt.push(')');
            }
            wkt.push(')');
            Ok(wkt)
        }
        Geometry2D::MultiPolygon(mpoly) => {
            let mut wkt = String::from("MULTIPOLYGON(");
            for (i, poly) in mpoly.iter().enumerate() {
                if i > 0 {
                    wkt.push_str(", ");
                }
                wkt.push_str("((");
                // Exterior ring
                for (j, coord) in poly.exterior().coords().enumerate() {
                    if j > 0 {
                        wkt.push_str(", ");
                    }
                    write!(wkt, "{} {}", coord.x, coord.y).unwrap();
                }
                wkt.push(')');
                // Interior rings
                for hole in poly.interiors() {
                    wkt.push_str(", (");
                    for (j, coord) in hole.coords().enumerate() {
                        if j > 0 {
                            wkt.push_str(", ");
                        }
                        write!(wkt, "{} {}", coord.x, coord.y).unwrap();
                    }
                    wkt.push(')');
                }
                wkt.push(')');
            }
            wkt.push(')');
            Ok(wkt)
        }
        Geometry2D::GeometryCollection(_) => {
            Err(GeometryExportError::UnsupportedGeometryCollection)
        }
        Geometry2D::Line(_)
        | Geometry2D::Rect(_)
        | Geometry2D::Triangle(_)
        | Geometry2D::Solid(_)
        | Geometry2D::CSG(_) => Err(GeometryExportError::UnsupportedGeometryType(format!(
            "{geom:?}"
        ))),
    }
}

fn geometry_3d_to_wkt(geom: &Geometry3D) -> Result<String, GeometryExportError> {
    match geom {
        Geometry3D::Point(pt) => Ok(format!("POINT({} {} {})", pt.x(), pt.y(), pt.z())),
        Geometry3D::LineString(ls) => {
            let mut wkt = String::from("LINESTRING(");
            for (i, coord) in ls.coords().enumerate() {
                if i > 0 {
                    wkt.push_str(", ");
                }
                write!(wkt, "{} {} {}", coord.x, coord.y, coord.z).unwrap();
            }
            wkt.push(')');
            Ok(wkt)
        }
        Geometry3D::Polygon(poly) => {
            let mut wkt = String::from("POLYGON(");
            // Exterior ring
            wkt.push('(');
            for (i, coord) in poly.exterior().coords().enumerate() {
                if i > 0 {
                    wkt.push_str(", ");
                }
                write!(wkt, "{} {} {}", coord.x, coord.y, coord.z).unwrap();
            }
            wkt.push(')');
            // Interior rings (holes)
            for hole in poly.interiors() {
                wkt.push_str(", (");
                for (i, coord) in hole.coords().enumerate() {
                    if i > 0 {
                        wkt.push_str(", ");
                    }
                    write!(wkt, "{} {} {}", coord.x, coord.y, coord.z).unwrap();
                }
                wkt.push(')');
            }
            wkt.push(')');
            Ok(wkt)
        }
        Geometry3D::MultiPoint(mp) => {
            let mut wkt = String::from("MULTIPOINT(");
            for (i, pt) in mp.iter().enumerate() {
                if i > 0 {
                    wkt.push_str(", ");
                }
                write!(wkt, "{} {} {}", pt.x(), pt.y(), pt.z()).unwrap();
            }
            wkt.push(')');
            Ok(wkt)
        }
        Geometry3D::MultiLineString(mls) => {
            let mut wkt = String::from("MULTILINESTRING(");
            for (i, ls) in mls.iter().enumerate() {
                if i > 0 {
                    wkt.push_str(", ");
                }
                wkt.push('(');
                for (j, coord) in ls.coords().enumerate() {
                    if j > 0 {
                        wkt.push_str(", ");
                    }
                    write!(wkt, "{} {} {}", coord.x, coord.y, coord.z).unwrap();
                }
                wkt.push(')');
            }
            wkt.push(')');
            Ok(wkt)
        }
        Geometry3D::MultiPolygon(mpoly) => {
            let mut wkt = String::from("MULTIPOLYGON(");
            for (i, poly) in mpoly.iter().enumerate() {
                if i > 0 {
                    wkt.push_str(", ");
                }
                wkt.push_str("((");
                // Exterior ring
                for (j, coord) in poly.exterior().coords().enumerate() {
                    if j > 0 {
                        wkt.push_str(", ");
                    }
                    write!(wkt, "{} {} {}", coord.x, coord.y, coord.z).unwrap();
                }
                wkt.push(')');
                // Interior rings
                for hole in poly.interiors() {
                    wkt.push_str(", (");
                    for (j, coord) in hole.coords().enumerate() {
                        if j > 0 {
                            wkt.push_str(", ");
                        }
                        write!(wkt, "{} {} {}", coord.x, coord.y, coord.z).unwrap();
                    }
                    wkt.push(')');
                }
                wkt.push(')');
            }
            wkt.push(')');
            Ok(wkt)
        }
        Geometry3D::GeometryCollection(_) => {
            Err(GeometryExportError::UnsupportedGeometryCollection)
        }
        Geometry3D::Line(_)
        | Geometry3D::Rect(_)
        | Geometry3D::Triangle(_)
        | Geometry3D::Solid(_)
        | Geometry3D::CSG(_) => Err(GeometryExportError::UnsupportedGeometryType(format!(
            "{geom:?}"
        ))),
    }
}

/// Extract X, Y, Z coordinates from Point geometries
/// Returns (x, y, optional z)
pub fn extract_coordinates(
    geometry: &Geometry,
) -> Result<(f64, f64, Option<f64>), GeometryExportError> {
    match &geometry.value {
        GeometryValue::None => Err(GeometryExportError::EmptyGeometry),
        GeometryValue::FlowGeometry2D(Geometry2D::Point(pt)) => Ok((pt.x(), pt.y(), None)),
        GeometryValue::FlowGeometry3D(Geometry3D::Point(pt)) => Ok((pt.x(), pt.y(), Some(pt.z()))),
        _ => Err(GeometryExportError::NonPointGeometry),
    }
}
