use crate::errors::GeometryExportError;
use indexmap::IndexMap;
use reearth_flow_geometry::types::geometry::{Geometry2D, Geometry3D};
use reearth_flow_types::{Geometry, GeometryValue};
use schemars::{
    gen::SchemaGenerator,
    schema::{InstanceType, Metadata, ObjectValidation, Schema, SchemaObject, SubschemaValidation},
    JsonSchema,
};
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

#[derive(Serialize, Deserialize, Debug, Clone)]
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

// Custom JsonSchema implementation to hide the redundant geometryMode field from UI
impl JsonSchema for GeometryExportMode {
    fn schema_name() -> String {
        "GeometryExportMode".to_string()
    }

    fn json_schema(_gen: &mut SchemaGenerator) -> Schema {
        // WKT variant schema without geometryMode field
        let wkt_schema = SchemaObject {
            instance_type: Some(InstanceType::Object.into()),
            metadata: Some(Box::new(Metadata {
                title: Some("WKT Column".to_string()),
                description: Some(
                    "Write geometry as Well-Known Text in a single column".to_string(),
                ),
                ..Default::default()
            })),
            object: Some(Box::new(ObjectValidation {
                properties: {
                    let mut props = schemars::Map::new();
                    props.insert(
                        "column".to_string(),
                        SchemaObject {
                            instance_type: Some(InstanceType::String.into()),
                            metadata: Some(Box::new(Metadata {
                                title: Some("WKT Column Name".to_string()),
                                description: Some(
                                    "Name of the column to write WKT geometry".to_string(),
                                ),
                                ..Default::default()
                            })),
                            ..Default::default()
                        }
                        .into(),
                    );
                    props
                },
                required: ["column".to_string()].into_iter().collect(),
                ..Default::default()
            })),
            ..Default::default()
        };

        // Coordinates variant schema without geometryMode field
        let coordinates_schema = SchemaObject {
            instance_type: Some(InstanceType::Object.into()),
            metadata: Some(Box::new(Metadata {
                title: Some("Coordinate Columns".to_string()),
                description: Some("Write geometry as separate X, Y, (optional Z) columns\nNote: Only supports Point geometries. Non-point geometries will be skipped with a warning.".to_string()),
                ..Default::default()
            })),
            object: Some(Box::new(ObjectValidation {
                properties: {
                    let mut props = schemars::Map::new();
                    props.insert("xColumn".to_string(), SchemaObject {
                        instance_type: Some(InstanceType::String.into()),
                        metadata: Some(Box::new(Metadata {
                            title: Some("X Column Name".to_string()),
                            description: Some("Name of the column for X coordinate (longitude)".to_string()),
                            ..Default::default()
                        })),
                        ..Default::default()
                    }.into());
                    props.insert("yColumn".to_string(), SchemaObject {
                        instance_type: Some(InstanceType::String.into()),
                        metadata: Some(Box::new(Metadata {
                            title: Some("Y Column Name".to_string()),
                            description: Some("Name of the column for Y coordinate (latitude)".to_string()),
                            ..Default::default()
                        })),
                        ..Default::default()
                    }.into());
                    props.insert("zColumn".to_string(), SchemaObject {
                        instance_type: Some(vec![InstanceType::String, InstanceType::Null].into()),
                        metadata: Some(Box::new(Metadata {
                            title: Some("Z Column Name".to_string()),
                            description: Some("Optional name of the column for Z coordinate (elevation)".to_string()),
                            ..Default::default()
                        })),
                        ..Default::default()
                    }.into());
                    props
                },
                required: ["xColumn".to_string(), "yColumn".to_string()].into_iter().collect(),
                ..Default::default()
            })),
            ..Default::default()
        };

        // Combine into oneOf
        SchemaObject {
            subschemas: Some(Box::new(SubschemaValidation {
                one_of: Some(vec![wkt_schema.into(), coordinates_schema.into()]),
                ..Default::default()
            })),
            ..Default::default()
        }
        .into()
    }
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
        | Geometry2D::CSG(_)
        | Geometry2D::TriangularMesh(_) => Err(GeometryExportError::UnsupportedGeometryType(
            format!("{geom:?}"),
        )),
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
        | Geometry3D::CSG(_)
        | Geometry3D::TriangularMesh(_) => Err(GeometryExportError::UnsupportedGeometryType(
            format!("{geom:?}"),
        )),
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

#[cfg(test)]
mod tests {
    use super::*;
    use reearth_flow_geometry::types::{
        coordinate::Coordinate,
        line_string::{LineString2D, LineString3D},
        multi_line_string::{MultiLineString2D, MultiLineString3D},
        multi_point::{MultiPoint2D, MultiPoint3D},
        multi_polygon::{MultiPolygon2D, MultiPolygon3D},
        no_value::NoValue,
        point::{Point2D, Point3D},
        polygon::{Polygon2D, Polygon3D},
    };

    #[test]
    fn test_geometry_2d_point_to_wkt() {
        let point = Point2D::from([1.0, 2.0]);
        let geom = Geometry2D::Point(point);
        let wkt = geometry_2d_to_wkt(&geom).unwrap();
        assert_eq!(wkt, "POINT(1 2)");
    }

    #[test]
    fn test_geometry_3d_point_to_wkt() {
        let point = Point3D::from([1.0, 2.0, 3.0]);
        let geom = Geometry3D::Point(point);
        let wkt = geometry_3d_to_wkt(&geom).unwrap();
        assert_eq!(wkt, "POINT(1 2 3)");
    }

    #[test]
    fn test_geometry_2d_linestring_to_wkt() {
        let coords = vec![
            Coordinate::<f64, NoValue>::from([0.0, 0.0]),
            Coordinate::<f64, NoValue>::from([1.0, 1.0]),
            Coordinate::<f64, NoValue>::from([2.0, 2.0]),
        ];
        let linestring = LineString2D::new(coords);
        let geom = Geometry2D::LineString(linestring);
        let wkt = geometry_2d_to_wkt(&geom).unwrap();
        assert_eq!(wkt, "LINESTRING(0 0, 1 1, 2 2)");
    }

    #[test]
    fn test_geometry_3d_linestring_to_wkt() {
        let coords = vec![
            Coordinate::<f64, f64>::from([0.0, 0.0, 0.0]),
            Coordinate::<f64, f64>::from([1.0, 1.0, 1.0]),
            Coordinate::<f64, f64>::from([2.0, 2.0, 2.0]),
        ];
        let linestring = LineString3D::new(coords);
        let geom = Geometry3D::LineString(linestring);
        let wkt = geometry_3d_to_wkt(&geom).unwrap();
        assert_eq!(wkt, "LINESTRING(0 0 0, 1 1 1, 2 2 2)");
    }

    #[test]
    fn test_geometry_2d_polygon_to_wkt() {
        // Simple polygon without holes
        let exterior = LineString2D::new(vec![
            Coordinate::<f64, NoValue>::from([0.0, 0.0]),
            Coordinate::<f64, NoValue>::from([4.0, 0.0]),
            Coordinate::<f64, NoValue>::from([4.0, 4.0]),
            Coordinate::<f64, NoValue>::from([0.0, 4.0]),
            Coordinate::<f64, NoValue>::from([0.0, 0.0]),
        ]);
        let polygon = Polygon2D::new(exterior, vec![]);
        let geom = Geometry2D::Polygon(polygon);
        let wkt = geometry_2d_to_wkt(&geom).unwrap();
        assert_eq!(wkt, "POLYGON((0 0, 4 0, 4 4, 0 4, 0 0))");
    }

    #[test]
    fn test_geometry_2d_polygon_with_hole_to_wkt() {
        // Polygon with one hole
        let exterior = LineString2D::new(vec![
            Coordinate::<f64, NoValue>::from([0.0, 0.0]),
            Coordinate::<f64, NoValue>::from([10.0, 0.0]),
            Coordinate::<f64, NoValue>::from([10.0, 10.0]),
            Coordinate::<f64, NoValue>::from([0.0, 10.0]),
            Coordinate::<f64, NoValue>::from([0.0, 0.0]),
        ]);
        let hole = LineString2D::new(vec![
            Coordinate::<f64, NoValue>::from([2.0, 2.0]),
            Coordinate::<f64, NoValue>::from([8.0, 2.0]),
            Coordinate::<f64, NoValue>::from([8.0, 8.0]),
            Coordinate::<f64, NoValue>::from([2.0, 8.0]),
            Coordinate::<f64, NoValue>::from([2.0, 2.0]),
        ]);
        let polygon = Polygon2D::new(exterior, vec![hole]);
        let geom = Geometry2D::Polygon(polygon);
        let wkt = geometry_2d_to_wkt(&geom).unwrap();
        assert_eq!(
            wkt,
            "POLYGON((0 0, 10 0, 10 10, 0 10, 0 0), (2 2, 8 2, 8 8, 2 8, 2 2))"
        );
    }

    #[test]
    fn test_geometry_3d_polygon_to_wkt() {
        let exterior = LineString3D::new(vec![
            Coordinate::<f64, f64>::from([0.0, 0.0, 0.0]),
            Coordinate::<f64, f64>::from([4.0, 0.0, 0.0]),
            Coordinate::<f64, f64>::from([4.0, 4.0, 1.0]),
            Coordinate::<f64, f64>::from([0.0, 4.0, 1.0]),
            Coordinate::<f64, f64>::from([0.0, 0.0, 0.0]),
        ]);
        let polygon = Polygon3D::new(exterior, vec![]);
        let geom = Geometry3D::Polygon(polygon);
        let wkt = geometry_3d_to_wkt(&geom).unwrap();
        assert_eq!(wkt, "POLYGON((0 0 0, 4 0 0, 4 4 1, 0 4 1, 0 0 0))");
    }

    #[test]
    fn test_geometry_2d_multipoint_to_wkt() {
        let points = vec![
            Point2D::from([0.0, 0.0]),
            Point2D::from([1.0, 1.0]),
            Point2D::from([2.0, 2.0]),
        ];
        let multipoint = MultiPoint2D::new(points);
        let geom = Geometry2D::MultiPoint(multipoint);
        let wkt = geometry_2d_to_wkt(&geom).unwrap();
        assert_eq!(wkt, "MULTIPOINT(0 0, 1 1, 2 2)");
    }

    #[test]
    fn test_geometry_3d_multipoint_to_wkt() {
        let points = vec![
            Point3D::from([0.0, 0.0, 0.0]),
            Point3D::from([1.0, 1.0, 1.0]),
            Point3D::from([2.0, 2.0, 2.0]),
        ];
        let multipoint = MultiPoint3D::new(points);
        let geom = Geometry3D::MultiPoint(multipoint);
        let wkt = geometry_3d_to_wkt(&geom).unwrap();
        assert_eq!(wkt, "MULTIPOINT(0 0 0, 1 1 1, 2 2 2)");
    }

    #[test]
    fn test_geometry_2d_multilinestring_to_wkt() {
        let line1 = LineString2D::new(vec![
            Coordinate::<f64, NoValue>::from([0.0, 0.0]),
            Coordinate::<f64, NoValue>::from([1.0, 1.0]),
        ]);
        let line2 = LineString2D::new(vec![
            Coordinate::<f64, NoValue>::from([2.0, 2.0]),
            Coordinate::<f64, NoValue>::from([3.0, 3.0]),
        ]);
        let mls = MultiLineString2D::new(vec![line1, line2]);
        let geom = Geometry2D::MultiLineString(mls);
        let wkt = geometry_2d_to_wkt(&geom).unwrap();
        assert_eq!(wkt, "MULTILINESTRING((0 0, 1 1), (2 2, 3 3))");
    }

    #[test]
    fn test_geometry_3d_multilinestring_to_wkt() {
        let line1 = LineString3D::new(vec![
            Coordinate::<f64, f64>::from([0.0, 0.0, 0.0]),
            Coordinate::<f64, f64>::from([1.0, 1.0, 1.0]),
        ]);
        let line2 = LineString3D::new(vec![
            Coordinate::<f64, f64>::from([2.0, 2.0, 2.0]),
            Coordinate::<f64, f64>::from([3.0, 3.0, 3.0]),
        ]);
        let mls = MultiLineString3D::new(vec![line1, line2]);
        let geom = Geometry3D::MultiLineString(mls);
        let wkt = geometry_3d_to_wkt(&geom).unwrap();
        assert_eq!(wkt, "MULTILINESTRING((0 0 0, 1 1 1), (2 2 2, 3 3 3))");
    }

    #[test]
    fn test_geometry_2d_multipolygon_to_wkt() {
        let poly1 = Polygon2D::new(
            LineString2D::new(vec![
                Coordinate::<f64, NoValue>::from([0.0, 0.0]),
                Coordinate::<f64, NoValue>::from([2.0, 0.0]),
                Coordinate::<f64, NoValue>::from([2.0, 2.0]),
                Coordinate::<f64, NoValue>::from([0.0, 2.0]),
                Coordinate::<f64, NoValue>::from([0.0, 0.0]),
            ]),
            vec![],
        );
        let poly2 = Polygon2D::new(
            LineString2D::new(vec![
                Coordinate::<f64, NoValue>::from([3.0, 3.0]),
                Coordinate::<f64, NoValue>::from([5.0, 3.0]),
                Coordinate::<f64, NoValue>::from([5.0, 5.0]),
                Coordinate::<f64, NoValue>::from([3.0, 5.0]),
                Coordinate::<f64, NoValue>::from([3.0, 3.0]),
            ]),
            vec![],
        );
        let mpoly = MultiPolygon2D::new(vec![poly1, poly2]);
        let geom = Geometry2D::MultiPolygon(mpoly);
        let wkt = geometry_2d_to_wkt(&geom).unwrap();
        assert_eq!(
            wkt,
            "MULTIPOLYGON(((0 0, 2 0, 2 2, 0 2, 0 0)), ((3 3, 5 3, 5 5, 3 5, 3 3)))"
        );
    }

    #[test]
    fn test_geometry_3d_multipolygon_to_wkt() {
        let poly1 = Polygon3D::new(
            LineString3D::new(vec![
                Coordinate::<f64, f64>::from([0.0, 0.0, 0.0]),
                Coordinate::<f64, f64>::from([2.0, 0.0, 0.0]),
                Coordinate::<f64, f64>::from([2.0, 2.0, 1.0]),
                Coordinate::<f64, f64>::from([0.0, 2.0, 1.0]),
                Coordinate::<f64, f64>::from([0.0, 0.0, 0.0]),
            ]),
            vec![],
        );
        let poly2 = Polygon3D::new(
            LineString3D::new(vec![
                Coordinate::<f64, f64>::from([3.0, 3.0, 2.0]),
                Coordinate::<f64, f64>::from([5.0, 3.0, 2.0]),
                Coordinate::<f64, f64>::from([5.0, 5.0, 3.0]),
                Coordinate::<f64, f64>::from([3.0, 5.0, 3.0]),
                Coordinate::<f64, f64>::from([3.0, 3.0, 2.0]),
            ]),
            vec![],
        );
        let mpoly = MultiPolygon3D::new(vec![poly1, poly2]);
        let geom = Geometry3D::MultiPolygon(mpoly);
        let wkt = geometry_3d_to_wkt(&geom).unwrap();
        assert_eq!(
            wkt,
            "MULTIPOLYGON(((0 0 0, 2 0 0, 2 2 1, 0 2 1, 0 0 0)), ((3 3 2, 5 3 2, 5 5 3, 3 5 3, 3 3 2)))"
        );
    }

    #[test]
    fn test_geometry_2d_multipolygon_with_holes_to_wkt() {
        // MultiPolygon with polygons containing holes
        let poly1_exterior = LineString2D::new(vec![
            Coordinate::<f64, NoValue>::from([0.0, 0.0]),
            Coordinate::<f64, NoValue>::from([10.0, 0.0]),
            Coordinate::<f64, NoValue>::from([10.0, 10.0]),
            Coordinate::<f64, NoValue>::from([0.0, 10.0]),
            Coordinate::<f64, NoValue>::from([0.0, 0.0]),
        ]);
        let poly1_hole = LineString2D::new(vec![
            Coordinate::<f64, NoValue>::from([2.0, 2.0]),
            Coordinate::<f64, NoValue>::from([8.0, 2.0]),
            Coordinate::<f64, NoValue>::from([8.0, 8.0]),
            Coordinate::<f64, NoValue>::from([2.0, 8.0]),
            Coordinate::<f64, NoValue>::from([2.0, 2.0]),
        ]);
        let poly1 = Polygon2D::new(poly1_exterior, vec![poly1_hole]);

        let poly2_exterior = LineString2D::new(vec![
            Coordinate::<f64, NoValue>::from([20.0, 20.0]),
            Coordinate::<f64, NoValue>::from([30.0, 20.0]),
            Coordinate::<f64, NoValue>::from([30.0, 30.0]),
            Coordinate::<f64, NoValue>::from([20.0, 30.0]),
            Coordinate::<f64, NoValue>::from([20.0, 20.0]),
        ]);
        let poly2 = Polygon2D::new(poly2_exterior, vec![]);

        let mpoly = MultiPolygon2D::new(vec![poly1, poly2]);
        let geom = Geometry2D::MultiPolygon(mpoly);
        let wkt = geometry_2d_to_wkt(&geom).unwrap();
        assert_eq!(
            wkt,
            "MULTIPOLYGON(((0 0, 10 0, 10 10, 0 10, 0 0), (2 2, 8 2, 8 8, 2 8, 2 2)), ((20 20, 30 20, 30 30, 20 30, 20 20)))"
        );
    }

    #[test]
    fn test_geometry_collection_2d_unsupported() {
        let geom = Geometry2D::GeometryCollection(vec![]);
        let result = geometry_2d_to_wkt(&geom);
        assert!(matches!(
            result,
            Err(GeometryExportError::UnsupportedGeometryCollection)
        ));
    }

    #[test]
    fn test_geometry_collection_3d_unsupported() {
        let geom = Geometry3D::GeometryCollection(vec![]);
        let result = geometry_3d_to_wkt(&geom);
        assert!(matches!(
            result,
            Err(GeometryExportError::UnsupportedGeometryCollection)
        ));
    }
}
