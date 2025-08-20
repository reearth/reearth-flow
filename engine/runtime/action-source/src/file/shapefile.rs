use std::{
    collections::HashMap,
    io::{Cursor, Read},
    sync::Arc,
};

use bytes::Bytes;
use indexmap::IndexMap;
use reearth_flow_geometry::types::{
    geometry::{Geometry2D, Geometry3D},
    line_string::{LineString2D, LineString3D},
    multi_line_string::{MultiLineString2D, MultiLineString3D},
    multi_point::{MultiPoint2D, MultiPoint3D},
    point::{Point2D, Point3D},
    polygon::{Polygon2D, Polygon3D},
};
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::NodeContext,
    node::{IngestionMessage, Port, Source, SourceFactory, DEFAULT_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, Feature, Geometry, GeometryValue};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::mpsc::Sender;

use crate::{
    errors::SourceError,
    file::reader::runner::{get_content, FileReaderCommonParam},
};

#[derive(Debug, Clone, Default)]
pub(crate) struct ShapefileReaderFactory;

impl SourceFactory for ShapefileReaderFactory {
    fn name(&self) -> &str {
        "ShapefileReader"
    }

    fn description(&self) -> &str {
        "Reads geographic features from Shapefile archives (.zip containing .shp, .dbf, .shx files)"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(ShapefileReaderParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["File"]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
        _state: Option<Vec<u8>>,
    ) -> Result<Box<dyn Source>, BoxedError> {
        let params = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                SourceError::FileReaderFactory(format!("Failed to serialize `with` parameter: {e}"))
            })?;
            serde_json::from_value(value).map_err(|e| {
                SourceError::FileReaderFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(SourceError::FileReaderFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        let reader = ShapefileReader { params };
        Ok(Box::new(reader))
    }
}

#[derive(Debug, Clone)]
pub(super) struct ShapefileReader {
    pub(super) params: ShapefileReaderParam,
}

/// # ShapefileReader Parameters
///
/// Configuration for reading Shapefile archives as geographic features.
/// Expects a ZIP archive containing the required Shapefile components (.shp, .dbf, .shx).
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(super) struct ShapefileReaderParam {
    #[serde(flatten)]
    pub(super) common_property: FileReaderCommonParam,
    /// # Character Encoding
    /// Character encoding for attribute data in the DBF file (e.g., "UTF-8", "Shift_JIS")
    pub(super) encoding: Option<String>,
    /// # Force 2D
    /// If true, forces all geometries to be 2D (ignoring Z values)
    #[serde(default)]
    pub(super) force_2d: bool,
}

#[async_trait::async_trait]
impl Source for ShapefileReader {
    async fn initialize(&self, _ctx: NodeContext) {}

    fn name(&self) -> &str {
        "ShapefileReader"
    }

    async fn serialize_state(&self) -> Result<Vec<u8>, BoxedError> {
        Ok(vec![])
    }

    async fn start(
        &mut self,
        ctx: NodeContext,
        sender: Sender<(Port, IngestionMessage)>,
    ) -> Result<(), BoxedError> {
        let storage_resolver = Arc::clone(&ctx.storage_resolver);

        let content = get_content(&ctx, &self.params.common_property, storage_resolver).await?;
        read_shapefile(&content, &self.params, sender)
            .await
            .map_err(Into::<BoxedError>::into)
    }
}

async fn read_shapefile(
    content: &Bytes,
    params: &ShapefileReaderParam,
    sender: Sender<(Port, IngestionMessage)>,
) -> Result<(), crate::errors::SourceError> {
    let shapes_and_records = if is_zip_file(content) {
        read_shapefile_from_zip(content)?
    } else {
        return Err(crate::errors::SourceError::ShapefileReader(
            "Direct shapefile bytes not supported. Please provide a ZIP archive containing the shapefile components (.shp, .dbf, .shx)".to_string()
        ));
    };

    for (shape, record) in shapes_and_records {
        let geometry = convert_shape_to_geometry(shape, params.force_2d)?;
        let attributes = convert_record_to_attributes(record);

        let feature = Feature {
            geometry,
            attributes,
            ..Default::default()
        };

        sender
            .send((
                DEFAULT_PORT.clone(),
                IngestionMessage::OperationEvent { feature },
            ))
            .await
            .map_err(|e| {
                crate::errors::SourceError::ShapefileReader(format!("Failed to send feature: {e}"))
            })?;
    }

    Ok(())
}

fn is_zip_file(content: &Bytes) -> bool {
    content.len() >= 2 && content[0] == 0x50 && content[1] == 0x4B
}

fn read_shapefile_from_zip(
    content: &Bytes,
) -> Result<Vec<(shapefile::Shape, shapefile::dbase::Record)>, crate::errors::SourceError> {
    let cursor = Cursor::new(content.as_ref());
    let mut archive = zip::ZipArchive::new(cursor).map_err(|e| {
        crate::errors::SourceError::ShapefileReader(format!("Failed to read ZIP archive: {e}"))
    })?;

    let mut shp_data: Option<Vec<u8>> = None;
    let mut dbf_data: Option<Vec<u8>> = None;
    let mut _shx_data: Option<Vec<u8>> = None;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| {
            crate::errors::SourceError::ShapefileReader(format!("Failed to read ZIP entry: {e}"))
        })?;

        let file_name = file.name().to_lowercase();

        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).map_err(|e| {
            crate::errors::SourceError::ShapefileReader(format!("Failed to read ZIP entry: {e}"))
        })?;

        if file_name.ends_with(".shp") {
            shp_data = Some(buffer);
        } else if file_name.ends_with(".dbf") {
            dbf_data = Some(buffer);
        } else if file_name.ends_with(".shx") {
            _shx_data = Some(buffer);
        }
    }

    let shp_data = shp_data.ok_or_else(|| {
        crate::errors::SourceError::ShapefileReader("No .shp file found in ZIP archive".to_string())
    })?;
    let dbf_data = dbf_data.ok_or_else(|| {
        crate::errors::SourceError::ShapefileReader("No .dbf file found in ZIP archive".to_string())
    })?;

    let shp_cursor = Cursor::new(shp_data);
    let dbf_cursor = Cursor::new(dbf_data);
    let shape_reader = shapefile::ShapeReader::new(shp_cursor).map_err(|e| {
        crate::errors::SourceError::ShapefileReader(format!("Failed to create shape reader: {e}"))
    })?;
    let dbase_reader = shapefile::dbase::Reader::new(dbf_cursor).map_err(|e| {
        crate::errors::SourceError::ShapefileReader(format!("Failed to create dbase reader: {e}"))
    })?;

    let mut reader = shapefile::Reader::new(shape_reader, dbase_reader);

    let mut shapes_and_records = Vec::new();
    for result in reader.iter_shapes_and_records() {
        let (shape, record) = result.map_err(|e| {
            crate::errors::SourceError::ShapefileReader(format!(
                "Failed to read shape and record: {e}"
            ))
        })?;
        shapes_and_records.push((shape, record));
    }

    Ok(shapes_and_records)
}

fn convert_shape_to_geometry(
    shape: shapefile::Shape,
    force_2d: bool,
) -> Result<Geometry, crate::errors::SourceError> {
    use shapefile::Shape;

    let geometry_value = match shape {
        Shape::Point(point) => {
            if force_2d {
                let p = Point2D::from([point.x, point.y]);
                GeometryValue::FlowGeometry2D(Geometry2D::Point(p))
            } else {
                let p = Point3D::from([point.x, point.y, 0.0]);
                GeometryValue::FlowGeometry3D(Geometry3D::Point(p))
            }
        }
        Shape::PointZ(point) => {
            if force_2d {
                let p = Point2D::from([point.x, point.y]);
                GeometryValue::FlowGeometry2D(Geometry2D::Point(p))
            } else {
                let p = Point3D::from([point.x, point.y, point.z]);
                GeometryValue::FlowGeometry3D(Geometry3D::Point(p))
            }
        }
        Shape::Polyline(polyline) => {
            if force_2d {
                let lines: Vec<LineString2D<f64>> = polyline
                    .parts()
                    .iter()
                    .map(|part| {
                        let coords: Vec<_> =
                            part.iter().map(|p| Point2D::from([p.x, p.y]).0).collect();
                        LineString2D::new(coords)
                    })
                    .collect();

                if lines.len() == 1 {
                    GeometryValue::FlowGeometry2D(Geometry2D::LineString(
                        lines.into_iter().next().unwrap(),
                    ))
                } else {
                    GeometryValue::FlowGeometry2D(Geometry2D::MultiLineString(
                        MultiLineString2D::new(lines),
                    ))
                }
            } else {
                let lines: Vec<LineString3D<f64>> = polyline
                    .parts()
                    .iter()
                    .map(|part| {
                        let coords: Vec<_> = part
                            .iter()
                            .map(|p| Point3D::from([p.x, p.y, 0.0]).0)
                            .collect();
                        LineString3D::new(coords)
                    })
                    .collect();

                if lines.len() == 1 {
                    GeometryValue::FlowGeometry3D(Geometry3D::LineString(
                        lines.into_iter().next().unwrap(),
                    ))
                } else {
                    GeometryValue::FlowGeometry3D(Geometry3D::MultiLineString(
                        MultiLineString3D::new(lines),
                    ))
                }
            }
        }
        Shape::PolylineZ(polyline) => {
            if force_2d {
                let lines: Vec<LineString2D<f64>> = polyline
                    .parts()
                    .iter()
                    .map(|part| {
                        let coords: Vec<_> =
                            part.iter().map(|p| Point2D::from([p.x, p.y]).0).collect();
                        LineString2D::new(coords)
                    })
                    .collect();

                if lines.len() == 1 {
                    GeometryValue::FlowGeometry2D(Geometry2D::LineString(
                        lines.into_iter().next().unwrap(),
                    ))
                } else {
                    GeometryValue::FlowGeometry2D(Geometry2D::MultiLineString(
                        MultiLineString2D::new(lines),
                    ))
                }
            } else {
                let lines: Vec<LineString3D<f64>> = polyline
                    .parts()
                    .iter()
                    .map(|part| {
                        let coords: Vec<_> = part
                            .iter()
                            .map(|p| Point3D::from([p.x, p.y, p.z]).0)
                            .collect();
                        LineString3D::new(coords)
                    })
                    .collect();

                if lines.len() == 1 {
                    GeometryValue::FlowGeometry3D(Geometry3D::LineString(
                        lines.into_iter().next().unwrap(),
                    ))
                } else {
                    GeometryValue::FlowGeometry3D(Geometry3D::MultiLineString(
                        MultiLineString3D::new(lines),
                    ))
                }
            }
        }
        Shape::Polygon(polygon) => {
            if force_2d {
                let rings: Vec<LineString2D<f64>> = polygon
                    .rings()
                    .iter()
                    .map(|ring| {
                        let coords: Vec<_> = ring
                            .points()
                            .iter()
                            .map(|p| Point2D::from([p.x, p.y]).0)
                            .collect();
                        LineString2D::new(coords)
                    })
                    .collect();

                if !rings.is_empty() {
                    let exterior = rings[0].clone();
                    let holes = rings[1..].to_vec();
                    GeometryValue::FlowGeometry2D(Geometry2D::Polygon(Polygon2D::new(
                        exterior, holes,
                    )))
                } else {
                    return Err(crate::errors::SourceError::ShapefileReader(
                        "Polygon has no rings".to_string(),
                    ));
                }
            } else {
                let rings: Vec<LineString3D<f64>> = polygon
                    .rings()
                    .iter()
                    .map(|ring| {
                        let coords: Vec<_> = ring
                            .points()
                            .iter()
                            .map(|p| Point3D::from([p.x, p.y, 0.0]).0)
                            .collect();
                        LineString3D::new(coords)
                    })
                    .collect();

                if !rings.is_empty() {
                    let exterior = rings[0].clone();
                    let holes = rings[1..].to_vec();
                    GeometryValue::FlowGeometry3D(Geometry3D::Polygon(Polygon3D::new(
                        exterior, holes,
                    )))
                } else {
                    return Err(crate::errors::SourceError::ShapefileReader(
                        "Polygon has no rings".to_string(),
                    ));
                }
            }
        }
        Shape::PolygonZ(polygon) => {
            if force_2d {
                let rings: Vec<LineString2D<f64>> = polygon
                    .rings()
                    .iter()
                    .map(|ring| {
                        let coords: Vec<_> = ring
                            .points()
                            .iter()
                            .map(|p| Point2D::from([p.x, p.y]).0)
                            .collect();
                        LineString2D::new(coords)
                    })
                    .collect();

                if !rings.is_empty() {
                    let exterior = rings[0].clone();
                    let holes = rings[1..].to_vec();
                    GeometryValue::FlowGeometry2D(Geometry2D::Polygon(Polygon2D::new(
                        exterior, holes,
                    )))
                } else {
                    return Err(crate::errors::SourceError::ShapefileReader(
                        "Polygon has no rings".to_string(),
                    ));
                }
            } else {
                let rings: Vec<LineString3D<f64>> = polygon
                    .rings()
                    .iter()
                    .map(|ring| {
                        let coords: Vec<_> = ring
                            .points()
                            .iter()
                            .map(|p| Point3D::from([p.x, p.y, p.z]).0)
                            .collect();
                        LineString3D::new(coords)
                    })
                    .collect();

                if !rings.is_empty() {
                    let exterior = rings[0].clone();
                    let holes = rings[1..].to_vec();
                    GeometryValue::FlowGeometry3D(Geometry3D::Polygon(Polygon3D::new(
                        exterior, holes,
                    )))
                } else {
                    return Err(crate::errors::SourceError::ShapefileReader(
                        "Polygon has no rings".to_string(),
                    ));
                }
            }
        }
        Shape::Multipoint(multipoint) => {
            if force_2d {
                let points: Vec<Point2D<f64>> = multipoint
                    .points()
                    .iter()
                    .map(|p| Point2D::from([p.x, p.y]))
                    .collect();
                GeometryValue::FlowGeometry2D(Geometry2D::MultiPoint(MultiPoint2D::new(points)))
            } else {
                let points: Vec<Point3D<f64>> = multipoint
                    .points()
                    .iter()
                    .map(|p| Point3D::from([p.x, p.y, 0.0]))
                    .collect();
                GeometryValue::FlowGeometry3D(Geometry3D::MultiPoint(MultiPoint3D::new(points)))
            }
        }
        Shape::MultipointZ(multipoint) => {
            if force_2d {
                let points: Vec<Point2D<f64>> = multipoint
                    .points()
                    .iter()
                    .map(|p| Point2D::from([p.x, p.y]))
                    .collect();
                GeometryValue::FlowGeometry2D(Geometry2D::MultiPoint(MultiPoint2D::new(points)))
            } else {
                let points: Vec<Point3D<f64>> = multipoint
                    .points()
                    .iter()
                    .map(|p| Point3D::from([p.x, p.y, p.z]))
                    .collect();
                GeometryValue::FlowGeometry3D(Geometry3D::MultiPoint(MultiPoint3D::new(points)))
            }
        }
        _ => {
            return Err(crate::errors::SourceError::ShapefileReader(
                "Unsupported shape type".to_string(),
            ))
        }
    };

    Ok(Geometry {
        epsg: None,
        value: geometry_value,
    })
}

fn convert_record_to_attributes(
    record: shapefile::dbase::Record,
) -> IndexMap<Attribute, AttributeValue> {
    let mut attributes = IndexMap::new();

    for (name, value) in record.into_iter() {
        let attr_value = match value {
            shapefile::dbase::FieldValue::Character(Some(s)) => AttributeValue::String(s),
            shapefile::dbase::FieldValue::Numeric(Some(n)) => {
                if n.is_finite() {
                    AttributeValue::Number(
                        serde_json::Number::from_f64(n)
                            .unwrap_or_else(|| serde_json::Number::from(0)),
                    )
                } else {
                    AttributeValue::Null
                }
            }
            shapefile::dbase::FieldValue::Logical(Some(b)) => AttributeValue::Bool(b),
            shapefile::dbase::FieldValue::Date(Some(d)) => {
                AttributeValue::String(format!("{:04}-{:02}-{:02}", d.year(), d.month(), d.day()))
            }
            shapefile::dbase::FieldValue::Float(Some(f)) => {
                let f = f as f64;
                if f.is_finite() {
                    AttributeValue::Number(
                        serde_json::Number::from_f64(f)
                            .unwrap_or_else(|| serde_json::Number::from(0)),
                    )
                } else {
                    AttributeValue::Null
                }
            }
            shapefile::dbase::FieldValue::Integer(i) => {
                AttributeValue::Number(serde_json::Number::from(i))
            }
            _ => AttributeValue::Null,
        };

        let attr_key = Attribute::new(name);
        attributes.insert(attr_key, attr_value);
    }

    attributes
}
