use std::{
    collections::HashMap,
    io::{Cursor, Read},
    sync::Arc,
};

use bytes::Bytes;
use indexmap::IndexMap;
use nusamai_projection::crs::EpsgCode;
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

/// Character encoding for shapefile DBF files
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ShapefileEncoding {
    /// UTF-8 and Unicode variants
    Utf8,
    /// encoding_rs supported encoding (100+ encodings)
    EncodingRs(&'static encoding_rs::Encoding),
}

impl ShapefileEncoding {
    /// Parse encoding name string into typed enum
    ///
    /// Returns error for unsupported encodings (e.g., UTF-16)
    fn from_name(name: &str) -> Result<Self, ShapefileError> {
        let name_upper = name.to_uppercase();

        // Handle UTF-8/Unicode variants
        if matches!(name_upper.as_str(), "UTF-8" | "UTF8" | "UNICODE" | "UTF_8") {
            return Ok(Self::Utf8);
        }

        // Reject UTF-16 early
        if matches!(
            name_upper.as_str(),
            "UTF-16" | "UTF16" | "UTF-16LE" | "UTF-16BE" | "UTF_16"
        ) {
            return Err(ShapefileError::Utf16NotSupported);
        }

        // Try encoding_rs for all other encodings
        if let Some(encoding) = encoding_rs::Encoding::for_label(name.as_bytes()) {
            return Ok(Self::EncodingRs(encoding));
        }

        // Unrecognized encoding - fail fast
        Err(ShapefileError::UnsupportedEncoding(name.to_string()))
    }

    /// Get the encoding name for logging
    fn name(&self) -> &str {
        match self {
            Self::Utf8 => "UTF-8",
            Self::EncodingRs(enc) => enc.name(),
        }
    }
}

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
    /// Supported encodings include:
    /// - **UTF-8** - Unicode UTF-8 (default, recommended for all new shapefiles)
    /// - **Windows Code Pages** - Windows-1250 through Windows-1258, Windows-874
    /// - **ISO-8859 family** - ISO-8859-1 (Latin-1) through ISO-8859-16
    /// - **Asian encodings** - Shift-JIS, EUC-JP, EUC-KR, Big5, GBK, GB18030
    /// - **Other legacy encodings** - KOI8-R, KOI8-U, IBM866, Macintosh
    ///
    /// All encoding labels are case-insensitive and support common variations
    /// (e.g., "UTF-8", "UTF8", "utf8" all work).
    ///
    /// UTF-16 is not supported due to byte-level handling requirements.
    /// If a UTF-16 shapefile is encountered, an error with conversion instructions is returned.
    ///
    /// Examples:
    /// - `"UTF-8"` - Modern standard
    /// - `"Windows-1252"` - Common for Western European legacy data
    /// - `"ISO-8859-1"` - Latin-1, common in older shapefiles
    /// - `"Shift-JIS"` - Japanese data
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
    let (shapes_and_records, epsg_code) = if is_zip_file(content) {
        read_shapefile_from_zip(content, &params.encoding)?
    } else {
        return Err(ShapefileError::DirectBytesNotSupported.into());
    };

    for (shape, record) in shapes_and_records {
        let mut geometry = convert_shape_to_geometry(shape, params.force_2d)?;
        // Set EPSG code from .prj file if available
        geometry.epsg = epsg_code;
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
            .map_err(|e| SourceError::shapefile_reader(format!("Failed to send feature: {e}")))?;
    }

    Ok(())
}

fn is_zip_file(content: &Bytes) -> bool {
    content.len() >= 2 && content[0] == 0x50 && content[1] == 0x4B
}

/// Parse encoding string from .cpg file content
///
/// .cpg files contain only the encoding name, typically on the first line.
/// We extract only the first line to handle files with additional content.
fn parse_cpg_encoding(cpg_data: &[u8]) -> Option<String> {
    let s = String::from_utf8_lossy(cpg_data);
    // Take only the first line and trim whitespace
    let first_line = s.lines().next()?;
    let trimmed = first_line.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

/// Parse .prj file to extract EPSG code
///
/// .prj files contain WKT (Well-Known Text) coordinate reference system definitions.
/// We use the proj crate to parse the WKT and extract the EPSG code if available.
fn parse_prj_epsg(prj_data: &[u8]) -> Option<EpsgCode> {
    let wkt_string = String::from_utf8_lossy(prj_data);
    let trimmed = wkt_string.trim();

    if trimmed.is_empty() {
        return None;
    }

    // Method 1: Parse WKT directly for AUTHORITY["EPSG","code"]
    // This handles standard OGC WKT format
    if let Some(authority_start) = trimmed.find("AUTHORITY[\"EPSG\",\"") {
        let after_prefix = &trimmed[authority_start + 19..];
        if let Some(quote_end) = after_prefix.find('\"') {
            let code_str = &after_prefix[..quote_end];
            if let Ok(code) = code_str.parse::<u16>() {
                tracing::info!("Extracted EPSG code from WKT AUTHORITY tag: {}", code);
                return Some(code);
            }
        }
    }

    // Method 2: Name-based lookup for common ESRI WKT formats
    // ESRI shapefiles often use PROJCS names without AUTHORITY tags
    // Extract the PROJCS name from the WKT
    if let Some(start) = trimmed.find("PROJCS[\"") {
        let after_start = &trimmed[start + 8..];
        if let Some(end) = after_start.find('\"') {
            let projcs_name = &after_start[..end];

            // Common ESRI projection names mapped to EPSG codes
            let esri_name_mapping: &[(&str, u16)] = &[
                // New Zealand
                ("NZGD_2000_New_Zealand_Transverse_Mercator", 2193),
                ("NZGD_2000", 2193),
                ("NZTM", 2193),
                ("NZGD2000", 2193),
                // WGS84 / Web Mercator
                ("WGS_1984_Web_Mercator_Auxiliary_Sphere", 3857),
                ("WGS_1984_Web_Mercator", 3857),
                ("WGS_84_Pseudo_Mercator", 3857),
                // WGS84 Geographic
                ("GCS_WGS_1984", 4326),
                ("WGS_1984", 4326),
                ("WGS_84", 4326),
                // NAD83 zones (US)
                ("NAD_1983_UTM_Zone_", 0), // Requires zone number parsing
                // Common European systems
                ("ETRS_1989_UTM_Zone_", 0), // Requires zone number parsing
            ];

            for (name_pattern, epsg) in esri_name_mapping {
                if projcs_name.eq_ignore_ascii_case(name_pattern) {
                    tracing::info!(
                        "Identified EPSG code from ESRI WKT name '{}': {}",
                        projcs_name,
                        epsg
                    );
                    return Some(*epsg);
                }
            }

            tracing::debug!("Unrecognized PROJCS name: {}", projcs_name);
        }
    }

    // Method 3: Try PROJ library to parse WKT
    // This may work for some WKT formats where PROJ can identify the CRS
    match proj::Proj::new(trimmed) {
        Ok(proj_obj) => {
            // Try to get EPSG from PROJ definition string
            if let Ok(info) = proj_obj.def() {
                for part in info.split_whitespace() {
                    if let Some(epsg_str) = part.strip_prefix("+init=epsg:") {
                        if let Ok(code) = epsg_str.parse::<u16>() {
                            tracing::info!("Extracted EPSG code from PROJ definition: {}", code);
                            return Some(code);
                        }
                    }
                }
            }

            tracing::debug!("PROJ could not identify EPSG code from WKT");
            None
        }
        Err(e) => {
            tracing::warn!("Failed to parse .prj WKT with PROJ library: {}", e);
            None
        }
    }
}

/// Resolve the encoding to use, with priority: parameter > .cpg file > default
///
/// Converts string encoding name to type-safe enum early, failing fast if unsupported.
fn resolve_encoding(
    encoding_param: &Option<String>,
    cpg_data: Option<&Vec<u8>>,
) -> Result<ShapefileEncoding, ShapefileError> {
    // Priority 1: Explicit encoding parameter
    if let Some(enc) = encoding_param {
        if !enc.is_empty() {
            tracing::debug!("Using encoding from parameter: {}", enc);
            return ShapefileEncoding::from_name(enc);
        }
    }

    // Priority 2: .cpg file
    if let Some(cpg) = cpg_data {
        if let Some(enc) = parse_cpg_encoding(cpg) {
            tracing::debug!("Using encoding from .cpg file: {}", enc);
            return ShapefileEncoding::from_name(&enc);
        }
    }

    // Priority 3: Default to UTF-8
    tracing::debug!("Using default encoding: UTF-8");
    Ok(ShapefileEncoding::Utf8)
}

/// Create a dbase Reader with the appropriate encoding
///
/// Takes a type-safe ShapefileEncoding enum (already validated) and creates
/// the appropriate dbase reader.
fn create_dbase_reader<T: std::io::Read + std::io::Seek>(
    source: T,
    encoding: ShapefileEncoding,
) -> Result<shapefile::dbase::Reader<T>, crate::errors::SourceError> {
    tracing::debug!("Creating dbase reader with {} encoding", encoding.name());

    match encoding {
        ShapefileEncoding::Utf8 => shapefile::dbase::Reader::new_with_encoding(
            source,
            shapefile::dbase::encoding::UnicodeLossy,
        )
        .map_err(|e| {
            SourceError::shapefile_reader(format!(
                "Failed to create dbase reader with UTF-8 encoding: {e}"
            ))
        }),
        ShapefileEncoding::EncodingRs(encoding_rs_encoding) => {
            let dbase_encoding = shapefile::dbase::encoding::EncodingRs::from(encoding_rs_encoding);
            shapefile::dbase::Reader::new_with_encoding(source, dbase_encoding).map_err(|e| {
                SourceError::shapefile_reader(format!(
                    "Failed to create dbase reader with {} encoding: {e}",
                    encoding_rs_encoding.name()
                ))
            })
        }
    }
}

fn read_shapefile_from_zip(
    content: &Bytes,
    encoding_param: &Option<String>,
) -> Result<
    (
        Vec<(shapefile::Shape, shapefile::dbase::Record)>,
        Option<EpsgCode>,
    ),
    crate::errors::SourceError,
> {
    let cursor = Cursor::new(content.as_ref());
    let mut archive = zip::ZipArchive::new(cursor)
        .map_err(|e| SourceError::shapefile_reader(format!("Failed to read ZIP archive: {e}")))?;

    let mut shapefile_groups: HashMap<String, ShapefileComponents> = HashMap::new();

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| {
            SourceError::shapefile_reader(format!("Failed to read ZIP entry at index {i}: {e}"))
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
            SourceError::shapefile_reader(format!("Failed to read ZIP entry '{file_name}': {e}"))
        })?;

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
        .ok_or_else(|| {
            SourceError::shapefile_reader(
                "No complete shapefile found in ZIP archive. Required files: .shp and .dbf"
                    .to_string(),
            )
        })?;

    tracing::info!("Processing shapefile: {}", base_name);

    // Resolve encoding from parameter, .cpg file, or default (fail fast if unsupported)
    let encoding = resolve_encoding(encoding_param, components.cpg.as_ref())?;

    // Parse .prj file to extract EPSG code if available
    let epsg_code = components.prj.as_ref().and_then(|prj| parse_prj_epsg(prj));

    if let Some(code) = epsg_code {
        tracing::info!("Shapefile CRS detected: EPSG:{}", code);
    } else {
        tracing::warn!("No EPSG code found in .prj file, geometries will have no CRS information");
    }

    let shp_data = components.shp.unwrap();
    let dbf_data = components.dbf.unwrap();

    let shp_cursor = Cursor::new(shp_data);
    let dbf_cursor = Cursor::new(dbf_data);
    let shape_reader = shapefile::ShapeReader::new(shp_cursor).map_err(|e| {
        SourceError::shapefile_reader(format!("Failed to create shape reader: {e}"))
    })?;

    // Create dbase reader with type-safe encoding enum
    let dbase_reader = create_dbase_reader(dbf_cursor, encoding)?;

    let mut reader = shapefile::Reader::new(shape_reader, dbase_reader);

    let mut shapes_and_records = Vec::new();
    for result in reader.iter_shapes_and_records() {
        let (shape, record) = result.map_err(|e| {
            SourceError::shapefile_reader(format!("Failed to read shape and record: {e}"))
        })?;
        shapes_and_records.push((shape, record));
    }

    Ok((shapes_and_records, epsg_code))
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
