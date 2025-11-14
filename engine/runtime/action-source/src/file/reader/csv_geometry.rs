use crate::errors::GeometryParsingError;
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
use schemars::{
    gen::SchemaGenerator,
    schema::{InstanceType, Metadata, ObjectValidation, Schema, SchemaObject, SubschemaValidation},
    JsonSchema,
};
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

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase", untagged)]
pub enum GeometryMode {
    /// # Coordinate Columns
    /// Geometry stored as separate X, Y, (optional Z) columns
    Coordinates {
        /// # X Column Name
        /// Name of the column containing X coordinate (longitude)
        #[serde(rename = "xColumn")]
        x_column: String,
        /// # Y Column Name
        /// Name of the column containing Y coordinate (latitude)
        #[serde(rename = "yColumn")]
        y_column: String,
        /// # Z Column Name
        /// Optional name of the column containing Z coordinate (elevation)
        #[serde(rename = "zColumn")]
        #[serde(skip_serializing_if = "Option::is_none")]
        z_column: Option<String>,
    },
    /// # WKT Column
    /// Geometry stored as Well-Known Text in a single column
    Wkt {
        /// # WKT Column Name
        /// Name of the column containing WKT geometry
        column: String,
    },
}

// Custom JsonSchema implementation to hide the redundant geometryMode field from UI
impl JsonSchema for GeometryMode {
    fn schema_name() -> String {
        "GeometryMode".to_string()
    }

    fn json_schema(_gen: &mut SchemaGenerator) -> Schema {
        // WKT variant schema without geometryMode field
        let wkt_schema = SchemaObject {
            instance_type: Some(InstanceType::Object.into()),
            metadata: Some(Box::new(Metadata {
                title: Some("WKT Column".to_string()),
                description: Some(
                    "Geometry stored as Well-Known Text in a single column".to_string(),
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
                                    "Name of the column containing WKT geometry".to_string(),
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
                description: Some(
                    "Geometry stored as separate X, Y, (optional Z) columns".to_string(),
                ),
                ..Default::default()
            })),
            object: Some(Box::new(ObjectValidation {
                properties: {
                    let mut props = schemars::Map::new();
                    props.insert(
                        "xColumn".to_string(),
                        SchemaObject {
                            instance_type: Some(InstanceType::String.into()),
                            metadata: Some(Box::new(Metadata {
                                title: Some("X Column Name".to_string()),
                                description: Some(
                                    "Name of the column containing X coordinate (longitude)"
                                        .to_string(),
                                ),
                                ..Default::default()
                            })),
                            ..Default::default()
                        }
                        .into(),
                    );
                    props.insert(
                        "yColumn".to_string(),
                        SchemaObject {
                            instance_type: Some(InstanceType::String.into()),
                            metadata: Some(Box::new(Metadata {
                                title: Some("Y Column Name".to_string()),
                                description: Some(
                                    "Name of the column containing Y coordinate (latitude)"
                                        .to_string(),
                                ),
                                ..Default::default()
                            })),
                            ..Default::default()
                        }
                        .into(),
                    );
                    props.insert("zColumn".to_string(), SchemaObject {
                        instance_type: Some(vec![InstanceType::String, InstanceType::Null].into()),
                        metadata: Some(Box::new(Metadata {
                            title: Some("Z Column Name".to_string()),
                            description: Some("Optional name of the column containing Z coordinate (elevation)".to_string()),
                            ..Default::default()
                        })),
                        ..Default::default()
                    }.into());
                    props
                },
                required: ["xColumn".to_string(), "yColumn".to_string()]
                    .into_iter()
                    .collect(),
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
) -> Result<Geometry, GeometryParsingError> {
    let epsg = config.epsg;

    match &config.mode {
        GeometryMode::Wkt { column } => {
            let wkt_str = row
                .get(column)
                .ok_or(GeometryParsingError::ColumnNotFound(column.clone()))?;
            parse_wkt_geometry(wkt_str, epsg)
        }
        GeometryMode::Coordinates {
            x_column,
            y_column,
            z_column,
        } => {
            let x_str = row
                .get(x_column)
                .ok_or(GeometryParsingError::ColumnNotFound(x_column.clone()))?;
            let y_str = row
                .get(y_column)
                .ok_or(GeometryParsingError::ColumnNotFound(y_column.clone()))?;

            let x: f64 = x_str
                .parse()
                .map_err(|_| GeometryParsingError::InvalidCoordinate {
                    column: x_column.clone(),
                    value: x_str.clone(),
                })?;
            let y: f64 = y_str
                .parse()
                .map_err(|_| GeometryParsingError::InvalidCoordinate {
                    column: y_column.clone(),
                    value: y_str.clone(),
                })?;

            if let Some(z_col) = z_column {
                let z_str = row
                    .get(z_col)
                    .ok_or(GeometryParsingError::ColumnNotFound(z_col.clone()))?;
                let z: f64 =
                    z_str
                        .parse()
                        .map_err(|_| GeometryParsingError::InvalidCoordinate {
                            column: z_col.clone(),
                            value: z_str.clone(),
                        })?;

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

fn parse_wkt_geometry(
    wkt_str: &str,
    epsg: Option<EpsgCode>,
) -> Result<Geometry, GeometryParsingError> {
    // Trim whitespace
    let wkt_str = wkt_str.trim();
    if wkt_str.is_empty() {
        return Ok(Geometry::default());
    }

    // Parse WKT string
    let wkt: wkt::Wkt<f64> =
        wkt::Wkt::from_str(wkt_str).map_err(|e| GeometryParsingError::WktParsing(e.to_string()))?;

    // Convert WKT geometry to geo_types
    let geo_geom: geo_types::Geometry<f64> = geo_types::Geometry::try_from(wkt)
        .map_err(|e| GeometryParsingError::WktConversion(format!("{e:?}")))?;

    // Convert geo_types to Flow geometry
    convert_geo_to_flow(geo_geom, epsg)
}

fn convert_geo_to_flow(
    geo_geom: geo_types::Geometry<f64>,
    epsg: Option<EpsgCode>,
) -> Result<Geometry, GeometryParsingError> {
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
            Err(GeometryParsingError::UnsupportedGeometryCollection)
        }
        _ => Err(GeometryParsingError::UnsupportedGeometryType(format!(
            "{geo_geom:?}"
        ))),
    }
}
