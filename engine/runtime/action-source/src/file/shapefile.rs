use std::{
    collections::HashMap,
    io::{Cursor, Read},
    sync::Arc,
};

use bytes::Bytes;
use indexmap::IndexMap;
use reearth_flow_geometry::types::{
    coordinate::Coordinate,
    geometry::{Geometry2D, Geometry3D},
    line_string::{LineString2D, LineString3D},
    multi_line_string::{MultiLineString2D, MultiLineString3D},
    multi_point::{MultiPoint2D, MultiPoint3D},
    multi_polygon::{MultiPolygon2D, MultiPolygon3D},
    no_value::NoValue,
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

#[derive(Default)]
struct ShapefileComponents {
    shp: Option<Vec<u8>>,
    dbf: Option<Vec<u8>>,
    shx: Option<Vec<u8>>,
}

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

    let mut shapefile_groups: HashMap<String, ShapefileComponents> = HashMap::new();

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| {
            crate::errors::SourceError::ShapefileReader(format!("Failed to read ZIP entry: {e}"))
        })?;

        let file_name = file.name().to_string();

        if file_name.contains("__MACOSX")
            || file_name.contains(".DS_Store")
            || file_name
                .split('/')
                .next_back()
                .is_some_and(|name| name.starts_with('.'))
            || file.is_dir()
        {
            continue;
        }

        let filename = file_name.split('/').next_back().unwrap_or(&file_name);
        let filename_lower = filename.to_lowercase();

        if !filename_lower.ends_with(".shp")
            && !filename_lower.ends_with(".dbf")
            && !filename_lower.ends_with(".shx")
            && !filename_lower.ends_with(".prj")
            && !filename_lower.ends_with(".cpg")
        {
            continue;
        }

        let base_name = if let Some(dot_pos) = filename.rfind('.') {
            filename[..dot_pos].to_string()
        } else {
            continue;
        };

        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).map_err(|e| {
            crate::errors::SourceError::ShapefileReader(format!("Failed to read ZIP entry: {e}"))
        })?;

        let components = shapefile_groups.entry(base_name).or_default();

        if filename_lower.ends_with(".shp") {
            components.shp = Some(buffer);
        } else if filename_lower.ends_with(".dbf") {
            components.dbf = Some(buffer);
        } else if filename_lower.ends_with(".shx") {
            components.shx = Some(buffer);
        }
    }

    let (base_name, components) = shapefile_groups
        .into_iter()
        .find(|(_, comp)| comp.shp.is_some() && comp.dbf.is_some())
        .ok_or_else(|| {
            crate::errors::SourceError::ShapefileReader(
                "No complete shapefile found in ZIP archive (needs both .shp and .dbf files)"
                    .to_string(),
            )
        })?;

    tracing::info!("Processing shapefile: {}", base_name);

    let shp_data = components.shp.unwrap();
    let dbf_data = components.dbf.unwrap();

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

type PolygonData2D = Vec<(
    Vec<Coordinate<f64, NoValue>>,
    Vec<Vec<Coordinate<f64, NoValue>>>,
)>;

type PolygonData3D = Vec<(Vec<Coordinate<f64, f64>>, Vec<Vec<Coordinate<f64, f64>>>)>;

fn process_polygon_rings_2d(
    rings: &[shapefile::PolygonRing<shapefile::Point>],
) -> Result<PolygonData2D, crate::errors::SourceError> {
    use shapefile::PolygonRing;

    if rings.is_empty() {
        return Err(crate::errors::SourceError::ShapefileReader(
            "Polygon has no rings".to_string(),
        ));
    }

    let mut polygon_data = Vec::new();
    let mut current_exterior: Option<Vec<Coordinate<f64, NoValue>>> = None;
    let mut current_holes: Vec<Vec<Coordinate<f64, NoValue>>> = Vec::new();

    for ring in rings {
        match ring {
            PolygonRing::Outer(points) => {
                if let Some(exterior) = current_exterior.take() {
                    polygon_data.push((exterior, current_holes.clone()));
                    current_holes.clear();
                }
                let coords = points
                    .iter()
                    .map(|p| Point2D::from([p.x, p.y]).into())
                    .collect();
                current_exterior = Some(coords);
            }
            PolygonRing::Inner(points) => {
                let coords = points
                    .iter()
                    .map(|p| Point2D::from([p.x, p.y]).into())
                    .collect();
                current_holes.push(coords);
            }
        }
    }

    if let Some(exterior) = current_exterior {
        polygon_data.push((exterior, current_holes));
    }

    if polygon_data.is_empty() {
        return Err(crate::errors::SourceError::ShapefileReader(
            "Polygon has no outer rings".to_string(),
        ));
    }

    Ok(polygon_data)
}

fn process_polygonz_rings_2d(
    rings: &[shapefile::PolygonRing<shapefile::PointZ>],
) -> Result<PolygonData2D, crate::errors::SourceError> {
    use shapefile::PolygonRing;

    if rings.is_empty() {
        return Err(crate::errors::SourceError::ShapefileReader(
            "Polygon has no rings".to_string(),
        ));
    }

    let mut polygon_data = Vec::new();
    let mut current_exterior: Option<Vec<Coordinate<f64, NoValue>>> = None;
    let mut current_holes: Vec<Vec<Coordinate<f64, NoValue>>> = Vec::new();

    for ring in rings {
        match ring {
            PolygonRing::Outer(points) => {
                if let Some(exterior) = current_exterior.take() {
                    polygon_data.push((exterior, current_holes.clone()));
                    current_holes.clear();
                }
                let coords = points
                    .iter()
                    .map(|p| Point2D::from([p.x, p.y]).into())
                    .collect();
                current_exterior = Some(coords);
            }
            PolygonRing::Inner(points) => {
                let coords = points
                    .iter()
                    .map(|p| Point2D::from([p.x, p.y]).into())
                    .collect();
                current_holes.push(coords);
            }
        }
    }

    if let Some(exterior) = current_exterior {
        polygon_data.push((exterior, current_holes));
    }

    if polygon_data.is_empty() {
        return Err(crate::errors::SourceError::ShapefileReader(
            "Polygon has no outer rings".to_string(),
        ));
    }

    Ok(polygon_data)
}

fn process_polygon_rings_3d(
    rings: &[shapefile::PolygonRing<shapefile::PointZ>],
) -> Result<PolygonData3D, crate::errors::SourceError> {
    use shapefile::PolygonRing;

    if rings.is_empty() {
        return Err(crate::errors::SourceError::ShapefileReader(
            "Polygon has no rings".to_string(),
        ));
    }

    let mut polygon_data = Vec::new();
    let mut current_exterior: Option<Vec<Coordinate<f64, f64>>> = None;
    let mut current_holes: Vec<Vec<Coordinate<f64, f64>>> = Vec::new();

    for ring in rings {
        match ring {
            PolygonRing::Outer(points) => {
                if let Some(exterior) = current_exterior.take() {
                    polygon_data.push((exterior, current_holes.clone()));
                    current_holes.clear();
                }
                let coords = points
                    .iter()
                    .map(|p| Point3D::from([p.x, p.y, p.z]).into())
                    .collect();
                current_exterior = Some(coords);
            }
            PolygonRing::Inner(points) => {
                let coords = points
                    .iter()
                    .map(|p| Point3D::from([p.x, p.y, p.z]).into())
                    .collect();
                current_holes.push(coords);
            }
        }
    }

    if let Some(exterior) = current_exterior {
        polygon_data.push((exterior, current_holes));
    }

    if polygon_data.is_empty() {
        return Err(crate::errors::SourceError::ShapefileReader(
            "Polygon has no outer rings".to_string(),
        ));
    }

    Ok(polygon_data)
}

fn polygons_to_geometry_2d(polygon_data: PolygonData2D) -> GeometryValue {
    let polygons: Vec<Polygon2D<f64>> = polygon_data
        .into_iter()
        .map(|(exterior, holes)| {
            Polygon2D::new(
                LineString2D::new(exterior),
                holes.into_iter().map(LineString2D::new).collect(),
            )
        })
        .collect();

    if polygons.len() == 1 {
        GeometryValue::FlowGeometry2D(Geometry2D::Polygon(polygons.into_iter().next().unwrap()))
    } else {
        GeometryValue::FlowGeometry2D(Geometry2D::MultiPolygon(MultiPolygon2D::new(polygons)))
    }
}

fn polygons_to_geometry_3d(polygon_data: PolygonData3D) -> GeometryValue {
    let polygons: Vec<Polygon3D<f64>> = polygon_data
        .into_iter()
        .map(|(exterior, holes)| {
            Polygon3D::new(
                LineString3D::new(exterior),
                holes.into_iter().map(LineString3D::new).collect(),
            )
        })
        .collect();

    if polygons.len() == 1 {
        GeometryValue::FlowGeometry3D(Geometry3D::Polygon(polygons.into_iter().next().unwrap()))
    } else {
        GeometryValue::FlowGeometry3D(Geometry3D::MultiPolygon(MultiPolygon3D::new(polygons)))
    }
}

fn convert_polygon_to_geometry(
    polygon: shapefile::Polygon,
) -> Result<GeometryValue, crate::errors::SourceError> {
    let polygon_data = process_polygon_rings_2d(polygon.rings())?;
    Ok(polygons_to_geometry_2d(polygon_data))
}

fn convert_polygonz_to_geometry(
    polygon: shapefile::PolygonZ,
    force_2d: bool,
) -> Result<GeometryValue, crate::errors::SourceError> {
    if force_2d {
        let polygon_data = process_polygonz_rings_2d(polygon.rings())?;
        Ok(polygons_to_geometry_2d(polygon_data))
    } else {
        let polygon_data = process_polygon_rings_3d(polygon.rings())?;
        Ok(polygons_to_geometry_3d(polygon_data))
    }
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
        Shape::Polygon(polygon) => convert_polygon_to_geometry(polygon)?,
        Shape::PolygonZ(polygon) => convert_polygonz_to_geometry(polygon, force_2d)?,
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
