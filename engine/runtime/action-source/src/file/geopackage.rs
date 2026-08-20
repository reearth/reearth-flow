use std::{collections::HashMap, sync::Arc};

use base64::Engine as _;
use bytes::Bytes;
use indexmap::IndexMap;
use reearth_flow_geometry::types::{
    geometry::{Geometry2D, Geometry3D},
    line_string::{LineString2D, LineString3D},
    multi_line_string::{MultiLineString2D, MultiLineString3D},
    multi_point::{MultiPoint2D, MultiPoint3D},
    multi_polygon::{MultiPolygon2D, MultiPolygon3D},
    point::{Point2D, Point3D},
    polygon::{Polygon2D, Polygon3D},
};
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::NodeContext,
    node::{IngestionMessage, Port, Source, SourceFactory, FEATURES_PORT},
};
use reearth_flow_sql::SqlAdapter;
use reearth_flow_types::{Attribute, AttributeValue, Feature, Geometry, GeometryValue};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{sqlite::SqliteRow, Column, Row, TypeInfo, ValueRef};
use tokio::sync::mpsc::Sender;

use crate::{
    errors::SourceError,
    file::reader::runner::{get_content, FileReaderCommonParam, FileReaderCompiledParam},
};

// New-geometry WKB parsing lives in a sibling file, declared here as a child module
// so it can reuse this module's private SQL/decoding helpers via `super::`.
#[cfg(feature = "new-geometry")]
#[path = "geopackage_next.rs"]
mod geopackage_next;

#[derive(Debug, Clone, Default)]
pub(crate) struct GeoPackageReaderFactory;

impl SourceFactory for GeoPackageReaderFactory {
    fn name(&self) -> &str {
        "GeoPackage Reader"
    }

    fn description(&self) -> &str {
        "Reads vector features from GeoPackage (.gpkg) files, or the file's spatial reference and extension metadata."
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(GeoPackageReaderParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Input"]
    }

    fn tags(&self) -> &[&'static str] {
        &["geopackage", "vector"]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![FEATURES_PORT.clone()]
    }

    fn build(
        &self,
        ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
        _state: Option<Vec<u8>>,
    ) -> Result<Box<dyn Source>, BoxedError> {
        let params: GeoPackageReaderParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                SourceError::GeoPackageReader(format!("Failed to serialize `with` parameter: {e}"))
            })?;
            serde_json::from_value(value).map_err(|e| {
                SourceError::GeoPackageReader(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(SourceError::GeoPackageReader(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        let compiled_params = GeoPackageReaderCompiledParam {
            common: params.common_property.compile(&ctx).map_err(|e| {
                SourceError::GeoPackageReader(format!("Failed to compile params: {e:?}"))
            })?,
            read_mode: params.read_mode,
            layer_name: params.layer_name,
            force_2d: params.force_2d,
        };
        Ok(Box::new(GeoPackageReader {
            params: compiled_params,
        }))
    }
}

/// # GeoPackage Reader Parameters
/// Configures which content to read from the GeoPackage file and how geometries are produced.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(super) struct GeoPackageReaderParam {
    #[serde(flatten)]
    pub(super) common_property: FileReaderCommonParam,
    /// # Read Mode
    /// Which content to read from the GeoPackage file. Defaults to reading vector features.
    #[serde(default)]
    read_mode: GeoPackageReadMode,
    /// # Layer Name
    /// Name of the layer to read. When omitted, every feature layer in the file is read.
    layer_name: Option<String>,
    /// # Force 2D
    /// Drops the Z value from every geometry, producing 2D output.
    #[serde(default, rename = "force2D")]
    force_2d: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema, Default)]
#[serde(rename_all = "camelCase")]
enum GeoPackageReadMode {
    /// # Features
    /// Reads vector features, each carrying its geometry and attributes.
    #[default]
    Features,
    /// # Metadata Only
    /// Reads the file's spatial reference systems and registered extensions
    /// instead of its features. Emits one feature per metadata record.
    MetadataOnly,
}

#[derive(Debug, Clone)]
struct GeoPackageReaderCompiledParam {
    common: FileReaderCompiledParam,
    read_mode: GeoPackageReadMode,
    layer_name: Option<String>,
    force_2d: bool,
}

#[derive(Debug, Clone)]
pub(super) struct GeoPackageReader {
    params: GeoPackageReaderCompiledParam,
}

#[async_trait::async_trait]
impl Source for GeoPackageReader {
    async fn initialize(&self, _ctx: NodeContext) {}

    fn name(&self) -> &str {
        "GeoPackage Reader"
    }

    async fn serialize_state(&self) -> Result<Vec<u8>, BoxedError> {
        Ok(vec![])
    }

    #[cfg(not(feature = "new-geometry"))]
    async fn start(
        &mut self,
        ctx: NodeContext,
        sender: Sender<(Port, IngestionMessage)>,
    ) -> Result<(), BoxedError> {
        let storage_resolver = Arc::clone(&ctx.storage_resolver);
        let content = get_content(&self.params.common, storage_resolver).await?;

        let features = process_geopackage(content, &self.params).await?;

        for feature in features {
            sender
                .send((
                    FEATURES_PORT.clone(),
                    IngestionMessage::OperationEvent { feature },
                ))
                .await
                .map_err(|e| {
                    SourceError::GeoPackageReader(format!("Failed to send feature: {e}"))
                })?;
        }

        Ok(())
    }

    #[cfg(feature = "new-geometry")]
    async fn start(
        &mut self,
        ctx: NodeContext,
        sender: Sender<(Port, IngestionMessage)>,
    ) -> Result<(), BoxedError> {
        let storage_resolver = Arc::clone(&ctx.storage_resolver);
        let content = get_content(&self.params.common, storage_resolver).await?;

        let features = geopackage_next::read(content, &self.params).await?;

        for feature in features {
            sender
                .send((
                    FEATURES_PORT.clone(),
                    IngestionMessage::OperationEvent { feature },
                ))
                .await
                .map_err(|e| {
                    SourceError::GeoPackageReader(format!("Failed to send feature: {e}"))
                })?;
        }

        Ok(())
    }
}

#[cfg(not(feature = "new-geometry"))]
async fn process_geopackage(
    content: Bytes,
    params: &GeoPackageReaderCompiledParam,
) -> Result<Vec<Feature>, SourceError> {
    let temp_file = tempfile::NamedTempFile::new()
        .map_err(|e| SourceError::GeoPackageReader(format!("Failed to create temp file: {e}")))?;

    std::fs::write(temp_file.path(), content)
        .map_err(|e| SourceError::GeoPackageReader(format!("Failed to write temp file: {e}")))?;

    let db_url = format!("sqlite://{}", temp_file.path().display());
    let adapter = SqlAdapter::new(db_url, 1)
        .await
        .map_err(|e| SourceError::GeoPackageReader(format!("Failed to open GeoPackage: {e}")))?;

    verify_geopackage(&adapter).await?;

    let mut all_features = Vec::new();

    match params.read_mode {
        GeoPackageReadMode::Features => {
            all_features.extend(read_features(&adapter, params).await?);
        }
        GeoPackageReadMode::MetadataOnly => {
            all_features.extend(read_metadata(&adapter).await?);
        }
    }

    Ok(all_features)
}

fn validate_table_name(name: &str) -> Result<(), SourceError> {
    if !name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    {
        return Err(SourceError::GeoPackageReader(format!(
            "Invalid table name: {name}"
        )));
    }
    Ok(())
}

fn escape_identifier(name: &str) -> String {
    name.replace('"', "\"\"")
}

fn escape_string(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

async fn verify_geopackage(adapter: &SqlAdapter) -> Result<(), SourceError> {
    let query = "SELECT name FROM sqlite_master WHERE type='table' AND name='gpkg_contents'";
    let rows = adapter.fetch_many(query).await.map_err(|e| {
        SourceError::GeoPackageReader(format!("Failed to query gpkg_contents: {e}"))
    })?;

    if rows.is_empty() {
        return Err(SourceError::GeoPackageReader(
            "Invalid GeoPackage: gpkg_contents table not found".to_string(),
        ));
    }

    Ok(())
}

#[cfg(not(feature = "new-geometry"))]
async fn read_features(
    adapter: &SqlAdapter,
    params: &GeoPackageReaderCompiledParam,
) -> Result<Vec<Feature>, SourceError> {
    let layers = if let Some(ref layer_name) = params.layer_name {
        vec![layer_name.clone()]
    } else {
        get_feature_layers(adapter).await?
    };

    let mut all_features = Vec::new();
    for layer in layers {
        let features = read_layer_features(adapter, &layer, params.force_2d).await?;
        all_features.extend(features);
    }

    Ok(all_features)
}

async fn get_feature_layers(adapter: &SqlAdapter) -> Result<Vec<String>, SourceError> {
    let query = "SELECT table_name FROM gpkg_contents WHERE data_type = 'features'";
    let rows = adapter.fetch_many(query).await.map_err(|e| {
        SourceError::GeoPackageReader(format!("Failed to query feature layers: {e}"))
    })?;

    let layers: Vec<String> = rows
        .into_iter()
        .filter_map(|row| row.try_get::<String, _>(0).ok())
        .collect();

    Ok(layers)
}

#[cfg(not(feature = "new-geometry"))]
async fn read_layer_features(
    adapter: &SqlAdapter,
    layer_name: &str,
    force_2d: bool,
) -> Result<Vec<Feature>, SourceError> {
    let geom_col = get_geometry_column(adapter, layer_name).await?;
    let srs_id = get_layer_srs_id(adapter, layer_name).await?;

    validate_table_name(layer_name)?;
    let query = format!("SELECT * FROM \"{}\"", escape_identifier(layer_name));
    // Read via the native SQLite driver: GeoPackage is a SQLite database and its
    // user columns can be types the generic `Any` driver rejects (e.g. BOOLEAN),
    // which would otherwise fail the whole layer read.
    let rows = adapter.fetch_many_sqlite(&query).await.map_err(|e| {
        SourceError::GeoPackageReader(format!("Failed to query layer {layer_name}: {e}"))
    })?;

    let mut features = Vec::new();
    for row in rows {
        let mut feature = row_to_feature(&row, &geom_col, srs_id, force_2d)?;
        feature.insert(
            "_geopackage_source",
            AttributeValue::String("features".to_string()),
        );
        feature.insert(
            "_geopackage_layer",
            AttributeValue::String(layer_name.to_string()),
        );
        features.push(feature);
    }

    Ok(features)
}

async fn get_geometry_column(
    adapter: &SqlAdapter,
    table_name: &str,
) -> Result<String, SourceError> {
    let query = format!(
        "SELECT column_name FROM gpkg_geometry_columns WHERE table_name = {}",
        escape_string(table_name)
    );
    let rows = adapter.fetch_many(&query).await.map_err(|e| {
        SourceError::GeoPackageReader(format!("Failed to query geometry column: {e}"))
    })?;

    if let Some(row) = rows.first() {
        if let Ok(col_name) = row.try_get::<String, _>(0) {
            return Ok(col_name);
        }
    }

    Ok("geom".to_string())
}

async fn get_layer_srs_id(adapter: &SqlAdapter, table_name: &str) -> Result<i32, SourceError> {
    // Escape table name to prevent SQL injection
    let query = format!(
        "SELECT srs_id FROM gpkg_geometry_columns WHERE table_name = {}",
        escape_string(table_name)
    );
    let rows = adapter
        .fetch_many(&query)
        .await
        .map_err(|e| SourceError::GeoPackageReader(format!("Failed to query SRS ID: {e}")))?;

    if let Some(row) = rows.first() {
        if let Ok(srs_id) = row.try_get::<i32, _>(0) {
            return Ok(srs_id);
        }
    }

    Ok(4326)
}

#[cfg(not(feature = "new-geometry"))]
fn row_to_feature(
    row: &SqliteRow,
    geom_col: &str,
    srs_id: i32,
    force_2d: bool,
) -> Result<Feature, SourceError> {
    let mut attributes = IndexMap::new();
    let mut geometry = None;

    for (idx, column) in row.columns().iter().enumerate() {
        let col_name = column.name();

        if col_name == geom_col {
            if let Ok(blob) = row.try_get::<Vec<u8>, _>(idx) {
                geometry = Some(parse_geopackage_geometry(&blob, srs_id, force_2d)?);
            }
        } else {
            let value = get_attribute_value(row, idx)?;
            attributes.insert(Attribute::new(col_name.to_string()), value);
        }
    }

    let mut feature = Feature::new_with_attributes(attributes);
    if let Some(geom) = geometry {
        feature.geometry = Arc::new(geom);
    }

    Ok(feature)
}

fn get_attribute_value(row: &SqliteRow, idx: usize) -> Result<AttributeValue, SourceError> {
    let raw = row.try_get_raw(idx).map_err(|e| {
        SourceError::GeoPackageReader(format!("Failed to get value at index {idx}: {e}"))
    })?;

    if raw.is_null() {
        return Ok(AttributeValue::Null);
    }

    let type_info = raw.type_info();
    let type_name = type_info.name();

    match type_name {
        "BOOLEAN" | "BOOL" => {
            if let Ok(v) = row.try_get::<bool, _>(idx) {
                Ok(AttributeValue::Bool(v))
            } else {
                Ok(AttributeValue::Null)
            }
        }
        "INTEGER" | "INT" | "INT4" | "INT8" | "BIGINT" | "SMALLINT" => {
            if let Ok(v) = row.try_get::<i64, _>(idx) {
                Ok(AttributeValue::Number(serde_json::Number::from(v)))
            } else if let Ok(v) = row.try_get::<i32, _>(idx) {
                Ok(AttributeValue::Number(serde_json::Number::from(v)))
            } else {
                Ok(AttributeValue::Null)
            }
        }
        "REAL" | "DOUBLE" | "FLOAT" | "NUMERIC" => {
            if let Ok(v) = row.try_get::<f64, _>(idx) {
                if let Some(n) = serde_json::Number::from_f64(v) {
                    Ok(AttributeValue::Number(n))
                } else {
                    Ok(AttributeValue::String(v.to_string()))
                }
            } else {
                Ok(AttributeValue::Null)
            }
        }
        "TEXT" | "VARCHAR" => {
            if let Ok(v) = row.try_get::<String, _>(idx) {
                Ok(AttributeValue::String(v))
            } else {
                Ok(AttributeValue::Null)
            }
        }
        "BLOB" => {
            if let Ok(v) = row.try_get::<Vec<u8>, _>(idx) {
                let encoded = base64::engine::general_purpose::STANDARD.encode(&v);
                Ok(AttributeValue::String(encoded))
            } else {
                Ok(AttributeValue::Null)
            }
        }
        _ => Ok(AttributeValue::Null),
    }
}

fn parse_geopackage_geometry(
    blob: &[u8],
    srs_id: i32,
    force_2d: bool,
) -> Result<Geometry, SourceError> {
    if blob.len() < 8 {
        return Err(SourceError::GeoPackageReader(
            "Invalid geometry blob: too short".to_string(),
        ));
    }

    let mut cursor = std::io::Cursor::new(blob);
    use byteorder::{LittleEndian, ReadBytesExt};

    let magic1 = cursor
        .read_u8()
        .map_err(|e| SourceError::GeoPackageReader(format!("Failed to read magic byte 1: {e}")))?;
    if magic1 != 0x47 {
        return Err(SourceError::GeoPackageReader(format!(
            "Invalid GeoPackage geometry magic 1: {magic1:#x}"
        )));
    }

    let magic2 = cursor
        .read_u8()
        .map_err(|e| SourceError::GeoPackageReader(format!("Failed to read magic byte 2: {e}")))?;
    if magic2 != 0x50 {
        return Err(SourceError::GeoPackageReader(format!(
            "Invalid GeoPackage geometry magic 2: {magic2:#x}"
        )));
    }

    let _version = cursor
        .read_u8()
        .map_err(|e| SourceError::GeoPackageReader(format!("Failed to read version: {e}")))?;

    let flags = cursor
        .read_u8()
        .map_err(|e| SourceError::GeoPackageReader(format!("Failed to read flags: {e}")))?;

    let _envelope_type = (flags >> 1) & 0x07;
    let _endianness = flags & 0x01;
    let _empty = (flags >> 4) & 0x01;
    let _binary_type = (flags >> 5) & 0x01;

    let gp_srs_id = cursor
        .read_i32::<LittleEndian>()
        .map_err(|e| SourceError::GeoPackageReader(format!("Failed to read SRS ID: {e}")))?;

    let envelope_length = match _envelope_type {
        0 => 0,
        1 => 32,
        2 | 3 => 48,
        4 => 64,
        _ => {
            return Err(SourceError::GeoPackageReader(format!(
                "Invalid envelope type: {_envelope_type}"
            )))
        }
    };

    cursor.set_position(cursor.position() + envelope_length);

    let wkb_start = cursor.position() as usize;
    let wkb = &blob[wkb_start..];

    // Use SRS ID from GP header if available, otherwise use the one from database
    let final_srs_id = if gp_srs_id != 0 { gp_srs_id } else { srs_id };
    parse_wkb(wkb, final_srs_id, force_2d)
}

fn parse_wkb(wkb: &[u8], srs_id: i32, force_2d: bool) -> Result<Geometry, SourceError> {
    if wkb.len() < 5 {
        return Err(SourceError::GeoPackageReader("WKB too short".to_string()));
    }

    let mut cursor = std::io::Cursor::new(wkb);
    use byteorder::{BigEndian, LittleEndian, ReadBytesExt};

    let byte_order = cursor
        .read_u8()
        .map_err(|e| SourceError::GeoPackageReader(format!("Failed to read byte order: {e}")))?;

    let wkb_type = if byte_order == 0x01 {
        let t = cursor
            .read_u32::<LittleEndian>()
            .map_err(|e| SourceError::GeoPackageReader(format!("Failed to read WKB type: {e}")))?;

        t
    } else {
        let t = cursor
            .read_u32::<BigEndian>()
            .map_err(|e| SourceError::GeoPackageReader(format!("Failed to read WKB type: {e}")))?;

        t
    };

    let has_z = (wkb_type & 0x80000000) != 0;
    let has_m = (wkb_type & 0x40000000) != 0;
    let geom_type = wkb_type & 0x1FFFFFFF;

    let epsg = if srs_id > 0 {
        Some(srs_id as u16)
    } else {
        None
    };

    match geom_type {
        1 => parse_point(&mut cursor, has_z, has_m, epsg, byte_order, force_2d),
        2 => parse_linestring(&mut cursor, has_z, has_m, epsg, byte_order, force_2d),
        3 => parse_polygon(&mut cursor, has_z, has_m, epsg, byte_order, force_2d),
        4 => parse_multipoint(&mut cursor, has_z, has_m, epsg, byte_order, force_2d),
        5 => parse_multilinestring(&mut cursor, has_z, has_m, epsg, byte_order, force_2d),
        6 => parse_multipolygon(&mut cursor, has_z, has_m, epsg, byte_order, force_2d),
        7 => parse_geometrycollection(&mut cursor, has_z, has_m, epsg, byte_order, force_2d),
        _ => {
            // More detailed error message
            Err(SourceError::GeoPackageReader(format!(
                "Unsupported geometry type: {geom_type} (raw WKB type: {wkb_type:#x})"
            )))
        }
    }
}

fn parse_point(
    cursor: &mut std::io::Cursor<&[u8]>,
    has_z: bool,
    _has_m: bool,
    epsg: Option<u16>,
    byte_order: u8,
    force_2d: bool,
) -> Result<Geometry, SourceError> {
    use byteorder::{BigEndian, LittleEndian, ReadBytesExt};

    let (x, y) = if byte_order == 0x01 {
        let x = cursor.read_f64::<LittleEndian>().map_err(|e| {
            SourceError::GeoPackageReader(format!("Failed to read X coordinate: {e}"))
        })?;
        let y = cursor.read_f64::<LittleEndian>().map_err(|e| {
            SourceError::GeoPackageReader(format!("Failed to read Y coordinate: {e}"))
        })?;
        (x, y)
    } else {
        let x = cursor.read_f64::<BigEndian>().map_err(|e| {
            SourceError::GeoPackageReader(format!("Failed to read X coordinate: {e}"))
        })?;
        let y = cursor.read_f64::<BigEndian>().map_err(|e| {
            SourceError::GeoPackageReader(format!("Failed to read Y coordinate: {e}"))
        })?;
        (x, y)
    };

    if has_z && !force_2d {
        let z = if byte_order == 0x01 {
            cursor.read_f64::<LittleEndian>().map_err(|e| {
                SourceError::GeoPackageReader(format!("Failed to read Z coordinate: {e}"))
            })?
        } else {
            cursor.read_f64::<BigEndian>().map_err(|e| {
                SourceError::GeoPackageReader(format!("Failed to read Z coordinate: {e}"))
            })?
        };
        Ok(Geometry {
            epsg,
            value: GeometryValue::FlowGeometry3D(Geometry3D::Point(Point3D::from([x, y, z]))),
        })
    } else {
        // If force_2d is true or it's a 2D point, skip Z coordinate if present
        if has_z && force_2d {
            // Read and discard Z coordinate
            let _ = if byte_order == 0x01 {
                cursor.read_f64::<LittleEndian>()
            } else {
                cursor.read_f64::<BigEndian>()
            };
        }
        Ok(Geometry {
            epsg,
            value: GeometryValue::FlowGeometry2D(Geometry2D::Point(Point2D::from([x, y]))),
        })
    }
}

fn parse_linestring(
    cursor: &mut std::io::Cursor<&[u8]>,
    has_z: bool,
    has_m: bool,
    epsg: Option<u16>,
    byte_order: u8,
    force_2d: bool,
) -> Result<Geometry, SourceError> {
    use byteorder::{BigEndian, LittleEndian, ReadBytesExt};

    let num_points = if byte_order == 0x01 {
        cursor.read_u32::<LittleEndian>().map_err(|e| {
            SourceError::GeoPackageReader(format!("Failed to read point count: {e}"))
        })?
    } else {
        cursor.read_u32::<BigEndian>().map_err(|e| {
            SourceError::GeoPackageReader(format!("Failed to read point count: {e}"))
        })?
    };

    let coords = read_coordinates(cursor, num_points, has_z, has_m, byte_order)?;

    if has_z && !force_2d {
        let coords_3d: Vec<(f64, f64, f64)> = coords
            .into_iter()
            .map(|c| (c.0, c.1, c.2.unwrap_or(0.0)))
            .collect();
        Ok(Geometry {
            epsg,
            value: GeometryValue::FlowGeometry3D(Geometry3D::LineString(LineString3D::from(
                coords_3d,
            ))),
        })
    } else {
        let coords_2d: Vec<(f64, f64)> = coords.into_iter().map(|c| (c.0, c.1)).collect();
        Ok(Geometry {
            epsg,
            value: GeometryValue::FlowGeometry2D(Geometry2D::LineString(LineString2D::from(
                coords_2d,
            ))),
        })
    }
}

type CoordRing = Vec<(f64, f64, Option<f64>)>;

type PolygonData = (CoordRing, Vec<CoordRing>);

fn parse_polygon(
    cursor: &mut std::io::Cursor<&[u8]>,
    has_z: bool,
    has_m: bool,
    epsg: Option<u16>,
    byte_order: u8,
    force_2d: bool,
) -> Result<Geometry, SourceError> {
    use byteorder::{BigEndian, LittleEndian, ReadBytesExt};

    let num_rings = if byte_order == 0x01 {
        cursor
            .read_u32::<LittleEndian>()
            .map_err(|e| SourceError::GeoPackageReader(format!("Failed to read ring count: {e}")))?
    } else {
        cursor
            .read_u32::<BigEndian>()
            .map_err(|e| SourceError::GeoPackageReader(format!("Failed to read ring count: {e}")))?
    };

    if num_rings == 0 {
        return Err(SourceError::GeoPackageReader(
            "Polygon has no rings".to_string(),
        ));
    }

    let mut rings = Vec::new();
    for _ in 0..num_rings {
        let num_points = if byte_order == 0x01 {
            cursor.read_u32::<LittleEndian>().map_err(|e| {
                SourceError::GeoPackageReader(format!("Failed to read point count: {e}"))
            })?
        } else {
            cursor.read_u32::<BigEndian>().map_err(|e| {
                SourceError::GeoPackageReader(format!("Failed to read point count: {e}"))
            })?
        };
        let coords = read_coordinates(cursor, num_points, has_z, has_m, byte_order)?;
        rings.push(coords);
    }

    // WKB standard: first ring is always exterior, subsequent rings are holes
    let mut rings_iter = rings.into_iter();
    let exterior = rings_iter.next().unwrap();
    let holes: Vec<CoordRing> = rings_iter.collect();

    if has_z && !force_2d {
        let exterior_3d: Vec<(f64, f64, f64)> = exterior
            .iter()
            .map(|c| (c.0, c.1, c.2.unwrap_or(0.0)))
            .collect();
        let holes_3d: Vec<Vec<(f64, f64, f64)>> = holes
            .iter()
            .map(|ring| {
                ring.iter()
                    .map(|c| (c.0, c.1, c.2.unwrap_or(0.0)))
                    .collect()
            })
            .collect();

        Ok(Geometry {
            epsg,
            value: GeometryValue::FlowGeometry3D(Geometry3D::Polygon(Polygon3D::new(
                LineString3D::from(exterior_3d),
                holes_3d.into_iter().map(LineString3D::from).collect(),
            ))),
        })
    } else {
        let exterior_2d: Vec<(f64, f64)> = exterior.iter().map(|c| (c.0, c.1)).collect();
        let holes_2d: Vec<Vec<(f64, f64)>> = holes
            .iter()
            .map(|ring| ring.iter().map(|c| (c.0, c.1)).collect())
            .collect();

        Ok(Geometry {
            epsg,
            value: GeometryValue::FlowGeometry2D(Geometry2D::Polygon(Polygon2D::new(
                LineString2D::from(exterior_2d),
                holes_2d.into_iter().map(LineString2D::from).collect(),
            ))),
        })
    }
}

fn parse_multipoint(
    cursor: &mut std::io::Cursor<&[u8]>,
    has_z: bool,
    has_m: bool,
    epsg: Option<u16>,
    byte_order: u8,
    force_2d: bool,
) -> Result<Geometry, SourceError> {
    use byteorder::{BigEndian, LittleEndian, ReadBytesExt};

    let num_points = if byte_order == 0x01 {
        cursor.read_u32::<LittleEndian>().map_err(|e| {
            SourceError::GeoPackageReader(format!("Failed to read point count: {e}"))
        })?
    } else {
        cursor.read_u32::<BigEndian>().map_err(|e| {
            SourceError::GeoPackageReader(format!("Failed to read point count: {e}"))
        })?
    };

    let mut points = Vec::new();
    for _ in 0..num_points {
        let inner_byte_order = cursor.read_u8().map_err(|e| {
            SourceError::GeoPackageReader(format!("Failed to read byte order: {e}"))
        })?;
        let _wkb_type = if inner_byte_order == 0x01 {
            cursor.read_u32::<LittleEndian>().map_err(|e| {
                SourceError::GeoPackageReader(format!("Failed to read WKB type: {e}"))
            })?
        } else {
            cursor.read_u32::<BigEndian>().map_err(|e| {
                SourceError::GeoPackageReader(format!("Failed to read WKB type: {e}"))
            })?
        };

        let coords = read_coordinates(cursor, 1, has_z, has_m, inner_byte_order)?;
        if let Some(coord) = coords.first() {
            points.push(*coord);
        }
    }

    if has_z && !force_2d {
        let points_3d: Vec<Point3D<f64>> = points
            .into_iter()
            .map(|c| Point3D::from([c.0, c.1, c.2.unwrap_or(0.0)]))
            .collect();
        Ok(Geometry {
            epsg,
            value: GeometryValue::FlowGeometry3D(Geometry3D::MultiPoint(MultiPoint3D::new(
                points_3d,
            ))),
        })
    } else {
        let points_2d: Vec<Point2D<f64>> = points
            .into_iter()
            .map(|c| Point2D::from([c.0, c.1]))
            .collect();
        Ok(Geometry {
            epsg,
            value: GeometryValue::FlowGeometry2D(Geometry2D::MultiPoint(MultiPoint2D::new(
                points_2d,
            ))),
        })
    }
}

fn parse_multilinestring(
    cursor: &mut std::io::Cursor<&[u8]>,
    has_z: bool,
    has_m: bool,
    epsg: Option<u16>,
    byte_order: u8,
    force_2d: bool,
) -> Result<Geometry, SourceError> {
    use byteorder::{BigEndian, LittleEndian, ReadBytesExt};

    let num_lines = if byte_order == 0x01 {
        cursor
            .read_u32::<LittleEndian>()
            .map_err(|e| SourceError::GeoPackageReader(format!("Failed to read line count: {e}")))?
    } else {
        cursor
            .read_u32::<BigEndian>()
            .map_err(|e| SourceError::GeoPackageReader(format!("Failed to read line count: {e}")))?
    };

    let mut lines = Vec::new();
    for _ in 0..num_lines {
        let inner_byte_order = cursor.read_u8().map_err(|e| {
            SourceError::GeoPackageReader(format!("Failed to read byte order: {e}"))
        })?;
        let _wkb_type = if inner_byte_order == 0x01 {
            cursor.read_u32::<LittleEndian>().map_err(|e| {
                SourceError::GeoPackageReader(format!("Failed to read WKB type: {e}"))
            })?
        } else {
            cursor.read_u32::<BigEndian>().map_err(|e| {
                SourceError::GeoPackageReader(format!("Failed to read WKB type: {e}"))
            })?
        };

        let num_points = if inner_byte_order == 0x01 {
            cursor.read_u32::<LittleEndian>().map_err(|e| {
                SourceError::GeoPackageReader(format!("Failed to read point count: {e}"))
            })?
        } else {
            cursor.read_u32::<BigEndian>().map_err(|e| {
                SourceError::GeoPackageReader(format!("Failed to read point count: {e}"))
            })?
        };
        let coords = read_coordinates(cursor, num_points, has_z, has_m, inner_byte_order)?;
        lines.push(coords);
    }

    if has_z && !force_2d {
        let lines_3d: Vec<LineString3D<f64>> = lines
            .into_iter()
            .map(|coords| {
                let coords_3d: Vec<(f64, f64, f64)> = coords
                    .into_iter()
                    .map(|c| (c.0, c.1, c.2.unwrap_or(0.0)))
                    .collect();
                LineString3D::from(coords_3d)
            })
            .collect();
        Ok(Geometry {
            epsg,
            value: GeometryValue::FlowGeometry3D(Geometry3D::MultiLineString(
                MultiLineString3D::new(lines_3d),
            )),
        })
    } else {
        let lines_2d: Vec<LineString2D<f64>> = lines
            .into_iter()
            .map(|coords| {
                let coords_2d: Vec<(f64, f64)> = coords.into_iter().map(|c| (c.0, c.1)).collect();
                LineString2D::from(coords_2d)
            })
            .collect();
        Ok(Geometry {
            epsg,
            value: GeometryValue::FlowGeometry2D(Geometry2D::MultiLineString(
                MultiLineString2D::new(lines_2d),
            )),
        })
    }
}

fn parse_multipolygon(
    cursor: &mut std::io::Cursor<&[u8]>,
    has_z: bool,
    has_m: bool,
    epsg: Option<u16>,
    byte_order: u8,
    force_2d: bool,
) -> Result<Geometry, SourceError> {
    use byteorder::{BigEndian, LittleEndian, ReadBytesExt};

    let num_polygons = if byte_order == 0x01 {
        cursor.read_u32::<LittleEndian>().map_err(|e| {
            SourceError::GeoPackageReader(format!("Failed to read polygon count: {e}"))
        })?
    } else {
        cursor.read_u32::<BigEndian>().map_err(|e| {
            SourceError::GeoPackageReader(format!("Failed to read polygon count: {e}"))
        })?
    };

    let mut polygons: Vec<PolygonData> = Vec::new();
    for _ in 0..num_polygons {
        let inner_byte_order = cursor.read_u8().map_err(|e| {
            SourceError::GeoPackageReader(format!("Failed to read byte order: {e}"))
        })?;
        let _wkb_type = if inner_byte_order == 0x01 {
            cursor.read_u32::<LittleEndian>().map_err(|e| {
                SourceError::GeoPackageReader(format!("Failed to read WKB type: {e}"))
            })?
        } else {
            cursor.read_u32::<BigEndian>().map_err(|e| {
                SourceError::GeoPackageReader(format!("Failed to read WKB type: {e}"))
            })?
        };

        let num_rings = if inner_byte_order == 0x01 {
            cursor.read_u32::<LittleEndian>().map_err(|e| {
                SourceError::GeoPackageReader(format!("Failed to read ring count: {e}"))
            })?
        } else {
            cursor.read_u32::<BigEndian>().map_err(|e| {
                SourceError::GeoPackageReader(format!("Failed to read ring count: {e}"))
            })?
        };

        if num_rings == 0 {
            return Err(SourceError::GeoPackageReader(
                "Polygon in MultiPolygon has no rings".to_string(),
            ));
        }

        let mut rings = Vec::new();
        for _ in 0..num_rings {
            let num_points = if inner_byte_order == 0x01 {
                cursor.read_u32::<LittleEndian>().map_err(|e| {
                    SourceError::GeoPackageReader(format!("Failed to read point count: {e}"))
                })?
            } else {
                cursor.read_u32::<BigEndian>().map_err(|e| {
                    SourceError::GeoPackageReader(format!("Failed to read point count: {e}"))
                })?
            };
            let coords = read_coordinates(cursor, num_points, has_z, has_m, inner_byte_order)?;
            rings.push(coords);
        }

        // WKB standard: first ring is always exterior, subsequent rings are holes
        let mut rings_iter = rings.into_iter();
        let exterior = rings_iter.next().unwrap();
        let holes: Vec<CoordRing> = rings_iter.collect();
        polygons.push((exterior, holes));
    }

    if has_z && !force_2d {
        let polygons_3d: Vec<Polygon3D<f64>> = polygons
            .into_iter()
            .map(|(exterior_coords, holes_coords)| {
                let exterior: Vec<(f64, f64, f64)> = exterior_coords
                    .iter()
                    .map(|c| (c.0, c.1, c.2.unwrap_or(0.0)))
                    .collect();
                let holes: Vec<Vec<(f64, f64, f64)>> = holes_coords
                    .iter()
                    .map(|ring| {
                        ring.iter()
                            .map(|c| (c.0, c.1, c.2.unwrap_or(0.0)))
                            .collect()
                    })
                    .collect();

                Polygon3D::new(
                    LineString3D::from(exterior),
                    holes.into_iter().map(LineString3D::from).collect(),
                )
            })
            .collect();
        Ok(Geometry {
            epsg,
            value: GeometryValue::FlowGeometry3D(Geometry3D::MultiPolygon(MultiPolygon3D::new(
                polygons_3d,
            ))),
        })
    } else {
        let polygons_2d: Vec<Polygon2D<f64>> = polygons
            .into_iter()
            .map(|(exterior_coords, holes_coords)| {
                let exterior: Vec<(f64, f64)> =
                    exterior_coords.iter().map(|c| (c.0, c.1)).collect();
                let holes: Vec<Vec<(f64, f64)>> = holes_coords
                    .iter()
                    .map(|ring| ring.iter().map(|c| (c.0, c.1)).collect())
                    .collect();

                Polygon2D::new(
                    LineString2D::from(exterior),
                    holes.into_iter().map(LineString2D::from).collect(),
                )
            })
            .collect();
        Ok(Geometry {
            epsg,
            value: GeometryValue::FlowGeometry2D(Geometry2D::MultiPolygon(MultiPolygon2D::new(
                polygons_2d,
            ))),
        })
    }
}

fn parse_geometrycollection(
    _cursor: &mut std::io::Cursor<&[u8]>,
    _has_z: bool,
    _has_m: bool,
    _epsg: Option<u16>,
    _byte_order: u8,
    _force_2d: bool,
) -> Result<Geometry, SourceError> {
    Err(SourceError::GeoPackageReader(
        "GeometryCollection not yet supported".to_string(),
    ))
}

fn read_coordinates(
    cursor: &mut std::io::Cursor<&[u8]>,
    count: u32,
    has_z: bool,
    has_m: bool,
    byte_order: u8,
) -> Result<Vec<(f64, f64, Option<f64>)>, SourceError> {
    use byteorder::{BigEndian, LittleEndian, ReadBytesExt};

    let mut coords = Vec::new();
    for _ in 0..count {
        let (x, y) = if byte_order == 0x01 {
            let x = cursor.read_f64::<LittleEndian>().map_err(|e| {
                SourceError::GeoPackageReader(format!("Failed to read X coordinate: {e}"))
            })?;
            let y = cursor.read_f64::<LittleEndian>().map_err(|e| {
                SourceError::GeoPackageReader(format!("Failed to read Y coordinate: {e}"))
            })?;
            (x, y)
        } else {
            let x = cursor.read_f64::<BigEndian>().map_err(|e| {
                SourceError::GeoPackageReader(format!("Failed to read X coordinate: {e}"))
            })?;
            let y = cursor.read_f64::<BigEndian>().map_err(|e| {
                SourceError::GeoPackageReader(format!("Failed to read Y coordinate: {e}"))
            })?;
            (x, y)
        };
        let z = if has_z {
            if byte_order == 0x01 {
                Some(cursor.read_f64::<LittleEndian>().map_err(|e| {
                    SourceError::GeoPackageReader(format!("Failed to read Z coordinate: {e}"))
                })?)
            } else {
                Some(cursor.read_f64::<BigEndian>().map_err(|e| {
                    SourceError::GeoPackageReader(format!("Failed to read Z coordinate: {e}"))
                })?)
            }
        } else {
            None
        };
        if has_m {
            if byte_order == 0x01 {
                let _m = cursor.read_f64::<LittleEndian>().map_err(|e| {
                    SourceError::GeoPackageReader(format!("Failed to read M coordinate: {e}"))
                })?;
            } else {
                let _m = cursor.read_f64::<BigEndian>().map_err(|e| {
                    SourceError::GeoPackageReader(format!("Failed to read M coordinate: {e}"))
                })?;
            }
        }
        coords.push((x, y, z));
    }

    Ok(coords)
}

async fn read_metadata(adapter: &SqlAdapter) -> Result<Vec<Feature>, SourceError> {
    let mut all_features = Vec::new();

    all_features.extend(read_srs_metadata(adapter).await?);
    all_features.extend(read_extensions_metadata(adapter).await?);

    Ok(all_features)
}

async fn read_srs_metadata(adapter: &SqlAdapter) -> Result<Vec<Feature>, SourceError> {
    let query = "SELECT srs_name, srs_id, organization, organization_coordsys_id, definition, description FROM gpkg_spatial_ref_sys";
    let rows = adapter
        .fetch_many(query)
        .await
        .map_err(|e| SourceError::GeoPackageReader(format!("Failed to query SRS metadata: {e}")))?;

    let mut features = Vec::new();
    for row in rows {
        let mut attributes = IndexMap::new();

        attributes.insert(
            Attribute::new("_geopackage_source".to_string()),
            AttributeValue::String("metadata".to_string()),
        );
        attributes.insert(
            Attribute::new("_metadata_type".to_string()),
            AttributeValue::String("spatial_ref_sys".to_string()),
        );

        if let Ok(v) = row.try_get::<String, _>(0) {
            attributes.insert(
                Attribute::new("srsName".to_string()),
                AttributeValue::String(v),
            );
        }
        if let Ok(v) = row.try_get::<i32, _>(1) {
            attributes.insert(
                Attribute::new("srsId".to_string()),
                AttributeValue::Number(serde_json::Number::from(v)),
            );
        }
        if let Ok(v) = row.try_get::<String, _>(2) {
            attributes.insert(
                Attribute::new("organization".to_string()),
                AttributeValue::String(v),
            );
        }
        if let Ok(v) = row.try_get::<i32, _>(3) {
            attributes.insert(
                Attribute::new("organizationCoordsysId".to_string()),
                AttributeValue::Number(serde_json::Number::from(v)),
            );
        }
        if let Ok(v) = row.try_get::<String, _>(4) {
            attributes.insert(
                Attribute::new("definition".to_string()),
                AttributeValue::String(v),
            );
        }
        if let Ok(v) = row.try_get::<String, _>(5) {
            attributes.insert(
                Attribute::new("description".to_string()),
                AttributeValue::String(v),
            );
        }

        let feature = Feature::new_with_attributes(attributes);
        features.push(feature);
    }

    Ok(features)
}

async fn read_extensions_metadata(adapter: &SqlAdapter) -> Result<Vec<Feature>, SourceError> {
    let query =
        "SELECT table_name, column_name, extension_name, definition, scope FROM gpkg_extensions";
    let rows = adapter
        .fetch_many(query)
        .await
        .map_err(|_| {
            SourceError::GeoPackageReader("Extensions table not found (this is normal)".to_string())
        })
        .unwrap_or_default();

    let mut features = Vec::new();
    for row in rows {
        let mut attributes = IndexMap::new();

        attributes.insert(
            Attribute::new("_geopackage_source".to_string()),
            AttributeValue::String("metadata".to_string()),
        );
        attributes.insert(
            Attribute::new("_metadata_type".to_string()),
            AttributeValue::String("extension".to_string()),
        );

        if let Ok(v) = row.try_get::<String, _>(0) {
            attributes.insert(
                Attribute::new("tableName".to_string()),
                AttributeValue::String(v),
            );
        }
        if let Ok(v) = row.try_get::<String, _>(1) {
            attributes.insert(
                Attribute::new("columnName".to_string()),
                AttributeValue::String(v),
            );
        }
        if let Ok(v) = row.try_get::<String, _>(2) {
            attributes.insert(
                Attribute::new("extensionName".to_string()),
                AttributeValue::String(v),
            );
        }
        if let Ok(v) = row.try_get::<String, _>(3) {
            attributes.insert(
                Attribute::new("definition".to_string()),
                AttributeValue::String(v),
            );
        }
        if let Ok(v) = row.try_get::<String, _>(4) {
            attributes.insert(
                Attribute::new("scope".to_string()),
                AttributeValue::String(v),
            );
        }

        let feature = Feature::new_with_attributes(attributes);
        features.push(feature);
    }

    Ok(features)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_wkb_point() {
        let wkb = vec![
            0x01, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xF0, 0x3F, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x40,
        ];

        let geom = parse_wkb(&wkb, 4326, false).unwrap();
        match geom.value {
            GeometryValue::FlowGeometry2D(Geometry2D::Point(p)) => {
                assert_eq!(p.x(), 1.0);
                assert_eq!(p.y(), 2.0);
            }
            _ => panic!("Expected Point2D"),
        }
    }

    #[test]
    fn test_camelcase_serialization() {
        use crate::file::reader::runner::FileReaderCommonParam;
        use reearth_flow_types::{Code, CodeType};

        let params = GeoPackageReaderParam {
            common_property: FileReaderCommonParam {
                dataset: Some(Code {
                    ty: CodeType::String,
                    value: "test.gpkg".to_string(),
                }),
                inline: None,
            },
            read_mode: GeoPackageReadMode::Features,
            layer_name: Some("test_layer".to_string()),
            force_2d: false,
        };

        let json = serde_json::to_string(&params).unwrap();

        // Check that snake_case fields are serialized as camelCase
        assert!(json.contains("\"readMode\""));
        assert!(json.contains("\"layerName\""));
        assert!(json.contains("\"force2D\"")); // Verify explicit rename

        // Check that values are serialized correctly
        assert!(json.contains("\"layerName\":\"test_layer\""));
        assert!(json.contains("\"readMode\":\"features\""));
    }
}
