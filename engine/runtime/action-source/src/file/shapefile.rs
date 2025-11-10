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
    errors::{ShapefileError, SourceError},
    file::reader::runner::{get_content, FileReaderCommonParam},
};

#[derive(Default)]
struct ShapefileComponents {
    shp: Option<Vec<u8>>,
    dbf: Option<Vec<u8>>,
    shx: Option<Vec<u8>>,
    prj: Option<Vec<u8>>,
    cpg: Option<Vec<u8>>,
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
                SourceError::ShapefileReaderFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                SourceError::ShapefileReaderFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(SourceError::ShapefileReaderFactory(
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
    ///
    /// Character encoding for attribute data in the DBF file.
    /// If not specified, encoding is determined from the .cpg file (if present), otherwise defaults to UTF-8.
    ///
    /// Supported encodings:
    /// - "UTF-8" - Unicode UTF-8 (default, handles most international characters)
    ///
    /// Note: The implementation uses UnicodeLossy encoding which gracefully handles
    /// invalid characters by replacing them with the Unicode replacement character (�).
    /// This ensures robust processing of shapefiles with UTF-8 or ASCII encoded field names and data.
    ///
    /// Legacy code pages (CP1252, etc.) may work with the `encoding_rs` feature but are not officially supported.
    /// UTF-16 is not supported as it requires different byte-level handling.
    ///
    /// Priority order: encoding parameter > .cpg file > UTF-8 default
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
        read_shapefile_from_zip(content, &params.encoding)?
    } else {
        return Err(ShapefileError::DirectBytesNotSupported.into());
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
            .map_err(|_| ShapefileError::FeatureSendError)?;
    }

    Ok(())
}

fn is_zip_file(content: &Bytes) -> bool {
    content.len() >= 2 && content[0] == 0x50 && content[1] == 0x4B
}

/// Parse encoding string from .cpg file content
fn parse_cpg_encoding(cpg_data: &[u8]) -> Option<String> {
    let s = String::from_utf8_lossy(cpg_data);
    let trimmed = s.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

/// Resolve the encoding to use, with priority: parameter > .cpg file > default
fn resolve_encoding(encoding_param: &Option<String>, cpg_data: Option<&Vec<u8>>) -> String {
    // Priority 1: Explicit encoding parameter
    if let Some(enc) = encoding_param {
        if !enc.is_empty() {
            tracing::debug!("Using encoding from parameter: {}", enc);
            return enc.clone();
        }
    }

    // Priority 2: .cpg file
    if let Some(cpg) = cpg_data {
        if let Some(enc) = parse_cpg_encoding(cpg) {
            tracing::debug!("Using encoding from .cpg file: {}", enc);
            return enc;
        }
    }

    // Priority 3: Default to UTF-8
    tracing::debug!("Using default encoding: UTF-8");
    "UTF-8".to_string()
}

/// Create a dbase Reader with the appropriate encoding
fn create_dbase_reader<T: std::io::Read + std::io::Seek>(
    source: T,
    encoding_name: &str,
) -> Result<shapefile::dbase::Reader<T>, crate::errors::SourceError> {
    let encoding_upper = encoding_name.to_uppercase();

    // Match common encoding names and use UnicodeLossy for UTF-8/Unicode variants
    // This handles UTF-8 properly, including UTF-8 field names in DBF headers
    match encoding_upper.as_str() {
        "UTF-8" | "UTF8" | "UNICODE" => {
            tracing::debug!("Using UnicodeLossy encoding for: {}", encoding_name);
            shapefile::dbase::Reader::new_with_encoding(
                source,
                shapefile::dbase::encoding::UnicodeLossy,
            )
            .map_err(|_| {
                ShapefileError::DbaseReaderCreationError {
                    encoding: "UnicodeLossy".to_string(),
                }
                .into()
            })
        }
        "UTF-16" | "UTF16" | "UTF-16LE" | "UTF-16BE" => {
            // UTF-16 requires different byte-level handling and is not supported
            Err(ShapefileError::Utf16NotSupported.into())
        }
        _ => {
            // For other encodings, fall back to UnicodeLossy with a warning
            tracing::warn!(
                "Unsupported or unrecognized encoding '{}', falling back to UnicodeLossy (UTF-8). \
                This may result in incorrect character decoding for legacy code pages.",
                encoding_name
            );
            shapefile::dbase::Reader::new_with_encoding(
                source,
                shapefile::dbase::encoding::UnicodeLossy,
            )
            .map_err(|_| {
                ShapefileError::DbaseReaderCreationError {
                    encoding: "fallback UnicodeLossy".to_string(),
                }
                .into()
            })
        }
    }
}

fn read_shapefile_from_zip(
    content: &Bytes,
    encoding_param: &Option<String>,
) -> Result<Vec<(shapefile::Shape, shapefile::dbase::Record)>, crate::errors::SourceError> {
    let cursor = Cursor::new(content.as_ref());
    let mut archive = zip::ZipArchive::new(cursor).map_err(|_| ShapefileError::ZipReadError)?;

    let mut shapefile_groups: HashMap<String, ShapefileComponents> = HashMap::new();

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|_| ShapefileError::ZipEntryReadError)?;

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
        file.read_to_end(&mut buffer)
            .map_err(|_| ShapefileError::ZipEntryReadError)?;

        let components = shapefile_groups.entry(base_name).or_default();

        if filename_lower.ends_with(".shp") {
            components.shp = Some(buffer);
        } else if filename_lower.ends_with(".dbf") {
            components.dbf = Some(buffer);
        } else if filename_lower.ends_with(".shx") {
            components.shx = Some(buffer);
        } else if filename_lower.ends_with(".prj") {
            components.prj = Some(buffer);
        } else if filename_lower.ends_with(".cpg") {
            components.cpg = Some(buffer);
        }
    }

    let (base_name, components) = shapefile_groups
        .into_iter()
        .find(|(_, comp)| comp.shp.is_some() && comp.dbf.is_some())
        .ok_or(ShapefileError::MissingComponents)?;

    tracing::info!("Processing shapefile: {}", base_name);

    // Resolve encoding from parameter, .cpg file, or default
    let encoding = resolve_encoding(encoding_param, components.cpg.as_ref());

    let shp_data = components.shp.unwrap();
    let dbf_data = components.dbf.unwrap();

    let shp_cursor = Cursor::new(shp_data);
    let dbf_cursor = Cursor::new(dbf_data);
    let shape_reader =
        shapefile::ShapeReader::new(shp_cursor).map_err(|_| ShapefileError::ShapeReaderCreationError)?;

    // Create dbase reader with resolved encoding
    let dbase_reader = create_dbase_reader(dbf_cursor, &encoding)?;

    let mut reader = shapefile::Reader::new(shape_reader, dbase_reader);

    let mut shapes_and_records = Vec::new();
    for result in reader.iter_shapes_and_records() {
        let (shape, record) =
            result.map_err(|_| ShapefileError::ShapeReaderCreationError)?;
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
        return Err(ShapefileError::PolygonNoRings.into());
    }

    let mut polygon_data = Vec::new();
    let mut current_exterior: Option<Vec<Coordinate<f64, NoValue>>> = None;
    let mut current_holes: Vec<Vec<Coordinate<f64, NoValue>>> = Vec::new();

    for ring in rings {
        match ring {
            PolygonRing::Outer(points) => {
                if let Some(exterior) = current_exterior.take() {
                    polygon_data.push((exterior, std::mem::take(&mut current_holes)));
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
        return Err(ShapefileError::PolygonNoOuterRings.into());
    }

    Ok(polygon_data)
}

fn process_polygonz_rings_2d(
    rings: &[shapefile::PolygonRing<shapefile::PointZ>],
) -> Result<PolygonData2D, crate::errors::SourceError> {
    use shapefile::PolygonRing;

    if rings.is_empty() {
        return Err(ShapefileError::PolygonNoRings.into());
    }

    let mut polygon_data = Vec::new();
    let mut current_exterior: Option<Vec<Coordinate<f64, NoValue>>> = None;
    let mut current_holes: Vec<Vec<Coordinate<f64, NoValue>>> = Vec::new();

    for ring in rings {
        match ring {
            PolygonRing::Outer(points) => {
                if let Some(exterior) = current_exterior.take() {
                    polygon_data.push((exterior, std::mem::take(&mut current_holes)));
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
        return Err(ShapefileError::PolygonNoOuterRings.into());
    }

    Ok(polygon_data)
}

fn process_polygon_rings_3d(
    rings: &[shapefile::PolygonRing<shapefile::PointZ>],
) -> Result<PolygonData3D, crate::errors::SourceError> {
    use shapefile::PolygonRing;

    if rings.is_empty() {
        return Err(ShapefileError::PolygonNoRings.into());
    }

    let mut polygon_data = Vec::new();
    let mut current_exterior: Option<Vec<Coordinate<f64, f64>>> = None;
    let mut current_holes: Vec<Vec<Coordinate<f64, f64>>> = Vec::new();

    for ring in rings {
        match ring {
            PolygonRing::Outer(points) => {
                if let Some(exterior) = current_exterior.take() {
                    polygon_data.push((exterior, std::mem::take(&mut current_holes)));
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
        return Err(ShapefileError::PolygonNoOuterRings.into());
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
            return Err(ShapefileError::UnsupportedShapeType(
                "Unknown or unsupported shape type".to_string(),
            )
            .into())
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
