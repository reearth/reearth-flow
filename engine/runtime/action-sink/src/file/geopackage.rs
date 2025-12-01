use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;
use std::sync::Arc;

use byteorder::{LittleEndian, WriteBytesExt};
use bytes::Bytes;
use indexmap::IndexMap;
use reearth_flow_common::uri::Uri;
use reearth_flow_geometry::types::geometry::{Geometry2D, Geometry3D};
use reearth_flow_geometry::types::line_string::{LineString2D, LineString3D};
use reearth_flow_geometry::types::multi_line_string::{MultiLineString2D, MultiLineString3D};
use reearth_flow_geometry::types::multi_point::{MultiPoint2D, MultiPoint3D};
use reearth_flow_geometry::types::multi_polygon::{MultiPolygon2D, MultiPolygon3D};
use reearth_flow_geometry::types::point::{Point2D, Point3D};
use reearth_flow_geometry::types::polygon::{Polygon2D, Polygon3D};
use reearth_flow_runtime::errors::BoxedError;
use reearth_flow_runtime::event::EventHub;
use reearth_flow_runtime::executor_operation::{ExecutorContext, NodeContext};
use reearth_flow_runtime::node::{Port, Sink, SinkFactory, DEFAULT_PORT};
use reearth_flow_sql::SqlAdapter;
use reearth_flow_types::{Attribute, AttributeValue, Expr, Feature, Geometry, GeometryValue};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::Row;

use crate::errors::SinkError;

#[derive(Debug, Clone, Default)]
pub(crate) struct GeoPackageWriterFactory;

impl SinkFactory for GeoPackageWriterFactory {
    fn name(&self) -> &str {
        "GeoPackageWriter"
    }

    fn description(&self) -> &str {
        "Writes geographic features to GeoPackage (.gpkg) files with proper SQLite structure, spatial indexing, and metadata tables"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(GeoPackageWriterParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["File", "Database"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn prepare(&self) -> Result<(), BoxedError> {
        Ok(())
    }

    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Sink>, BoxedError> {
        let params: GeoPackageWriterParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                SinkError::GeoPackageWriterFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                SinkError::GeoPackageWriterFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(SinkError::GeoPackageWriterFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };

        let default_table = params.table_name.clone();
        let sink = GeoPackageWriter {
            params,
            tables: Default::default(),
            default_table,
        };
        Ok(Box::new(sink))
    }
}

/// Per-table data for multi-layer support
#[derive(Debug, Clone, Default)]
pub(super) struct TableData {
    features: Vec<Feature>,
    schema: IndexMap<String, AttributeType>,
}

#[derive(Debug, Clone)]
pub(super) struct GeoPackageWriter {
    pub(super) params: GeoPackageWriterParam,
    /// Features grouped by table name (for multi-layer support)
    pub(super) tables: HashMap<String, TableData>,
    /// Default table name (used when no grouping)
    pub(super) default_table: String,
}

/// # GeoPackageWriter Parameters
///
/// Configuration for writing features to GeoPackage files.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(super) struct GeoPackageWriterParam {
    /// Output path for the GeoPackage file to create
    pub(super) output: Expr,
    /// Table name to create (default: "features")
    #[serde(default = "default_table_name")]
    pub(super) table_name: String,
    /// Geometry column name (default: "geom")
    #[serde(default = "default_geometry_column")]
    pub(super) geometry_column: String,
    /// Spatial Reference System ID (default: 4326 for WGS84)
    #[serde(default = "default_srs_id")]
    pub(super) srs_id: i32,
    /// Geometry type for table (Point, LineString, Polygon, MultiPoint, MultiLineString, MultiPolygon, or GEOMETRY for mixed)
    #[serde(default = "default_geometry_type")]
    pub(super) geometry_type: String,
    /// Create RTree spatial index (default: true)
    #[serde(default = "default_create_spatial_index")]
    pub(super) create_spatial_index: bool,
    /// Overwrite existing file (default: false)
    #[serde(default)]
    pub(super) overwrite: bool,
    /// Table handling mode (default: CreateIfNeeded)
    /// - CreateIfNeeded: Create table if it doesn't exist, append if it does
    /// - UseExisting: Append to existing table (fail if table doesn't exist)
    /// - DropAndCreate: Drop existing table and recreate it
    #[serde(default)]
    pub(super) table_mode: TableMode,


    /// Attribute name to use for grouping features into multiple tables.
    /// When specified, features will be written to separate tables based on
    /// this attribute's value. Table names will be "{tableName}_{groupValue}".
    #[serde(default)]
    pub(super) group_by: Option<Attribute>,

    /// Z coordinate handling mode (default: Optional)
    /// - Optional: Preserve Z if present, don't add if missing
    /// - Required: Add Z=0 if missing, preserve if present
    /// - NotAllowed: Drop Z coordinates, force 2D output
    #[serde(default)]
    pub(super) z_mode: ZCoordinateMode,

    /// Batch size for transaction commits (default: 1000)
    /// Features are inserted in batches within transactions for better performance.
    /// Set to 0 to disable batching (commit after all features).
    #[serde(default = "default_batch_size")]
    pub(super) batch_size: usize,

    /// Primary key column name (default: "fid")
    /// The column will be created as INTEGER PRIMARY KEY AUTOINCREMENT.
    #[serde(default = "default_primary_key")]
    pub(super) primary_key: String,

    /// Use feature attribute as primary key instead of auto-generated.
    /// If specified, the attribute value must be unique across all features.
    #[serde(default)]
    pub(super) primary_key_attribute: Option<Attribute>,
}

/// Z coordinate handling mode
#[derive(Serialize, Deserialize, Debug, Clone, Default, JsonSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum ZCoordinateMode {
    /// Preserve Z if present, don't add if missing (default)
    #[default]
    Optional,
    /// Add Z=0 if geometry doesn't have Z coordinates
    Required,
    /// Drop Z coordinates, force 2D output
    NotAllowed,
}

impl fmt::Display for ZCoordinateMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ZCoordinateMode::Optional => write!(f, "optional"),
            ZCoordinateMode::Required => write!(f, "required"),
            ZCoordinateMode::NotAllowed => write!(f, "notAllowed"),
        }
    }
}

/// Table handling mode for GeoPackage writer
#[derive(Serialize, Deserialize, Debug, Clone, Default, JsonSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum TableMode {
    /// Create table if it doesn't exist, append if it does (default)
    #[default]
    CreateIfNeeded,
    /// Append to existing table (fail if table doesn't exist)
    UseExisting,
    /// Drop existing table and recreate it
    DropAndCreate,
}

impl fmt::Display for TableMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TableMode::CreateIfNeeded => write!(f, "createIfNeeded"),
            TableMode::UseExisting => write!(f, "useExisting"),
            TableMode::DropAndCreate => write!(f, "dropAndCreate"),
        }
    }
}

fn default_table_name() -> String {
    "features".to_string()
}

fn default_geometry_column() -> String {
    "geom".to_string()
}

fn default_srs_id() -> i32 {
    4326
}

fn default_geometry_type() -> String {
    "GEOMETRY".to_string()
}

fn default_create_spatial_index() -> bool {
    true
}

fn default_batch_size() -> usize {
    1000
}

fn default_primary_key() -> String {
    "fid".to_string()
}

/// Attribute type for GeoPackage columns.
///
/// Type mapping from Flow AttributeValue to SQLite types:
/// - Boolean → INTEGER (0/1)
/// - Number (integer) → INTEGER
/// - Number (float) → REAL
/// - String → TEXT
/// - Array/Map → TEXT (JSON serialized)
/// - DateTime → TEXT (ISO 8601 format)
/// - Bytes → BLOB
/// - Null → preserves NULL (column type determined by non-null values)
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum AttributeType {
    /// SQLite INTEGER type (also used for Boolean as 0/1)
    Integer,
    /// SQLite REAL type (floating point)
    Real,
    /// SQLite TEXT type (strings, JSON, datetime)
    Text,
    /// SQLite BLOB type (binary data)
    Blob,
    /// Boolean stored as INTEGER (0/1)
    Boolean,
}

impl AttributeType {
    /// Convert to SQLite type name
    fn to_sql_type(&self) -> &'static str {
        match self {
            AttributeType::Integer => "INTEGER",
            AttributeType::Real => "REAL",
            AttributeType::Text => "TEXT",
            AttributeType::Blob => "BLOB",
            AttributeType::Boolean => "INTEGER",
        }
    }

    /// Infer attribute type from a value
    fn from_attribute_value(value: &AttributeValue) -> Option<Self> {
        match value {
            AttributeValue::Bool(_) => Some(AttributeType::Boolean),
            AttributeValue::Number(n) => {
                if n.is_i64() {
                    Some(AttributeType::Integer)
                } else {
                    Some(AttributeType::Real)
                }
            }
            AttributeValue::String(_) => Some(AttributeType::Text),
            AttributeValue::Array(_) | AttributeValue::Map(_) => Some(AttributeType::Text),
            AttributeValue::DateTime(_) => Some(AttributeType::Text),
            AttributeValue::Bytes(_) => Some(AttributeType::Blob),
            // NULL doesn't determine type - return None to indicate unknown
            AttributeValue::Null => None,
        }
    }

    /// Promote type when there's a conflict between two types.
    /// Returns the more general type that can accommodate both.
    ///
    /// Type promotion rules:
    /// - Integer + Real → Real (float can represent integers)
    /// - Integer + Boolean → Integer (boolean is stored as 0/1)
    /// - Any + Text → Text (text can represent anything)
    /// - Any + Blob → Blob stays Blob (binary data)
    /// - Same types → no change
    fn promote_with(&self, other: &AttributeType) -> AttributeType {
        if self == other {
            return self.clone();
        }

        match (self, other) {
            // Integer and Real - promote to Real
            (AttributeType::Integer, AttributeType::Real)
            | (AttributeType::Real, AttributeType::Integer) => AttributeType::Real,

            // Boolean and Integer - promote to Integer (both stored as INTEGER in SQLite)
            (AttributeType::Boolean, AttributeType::Integer)
            | (AttributeType::Integer, AttributeType::Boolean) => AttributeType::Integer,

            // Boolean and Real - promote to Real
            (AttributeType::Boolean, AttributeType::Real)
            | (AttributeType::Real, AttributeType::Boolean) => AttributeType::Real,

            // Blob stays Blob when mixed with anything except Text
            (AttributeType::Blob, _) | (_, AttributeType::Blob) => {
                // If one is Text, promote to Text (can represent anything as string)
                if *self == AttributeType::Text || *other == AttributeType::Text {
                    AttributeType::Text
                } else {
                    AttributeType::Blob
                }
            }

            // Text is the most general type - can represent anything
            (AttributeType::Text, _) | (_, AttributeType::Text) => AttributeType::Text,

            // Default fallback - use Text as most compatible
            _ => AttributeType::Text,
        }
    }

    /// Check if this type can be safely converted to another type
    #[allow(dead_code)]
    fn is_compatible_with(&self, other: &AttributeType) -> bool {
        if self == other {
            return true;
        }
        // Check if promotion would result in the target type
        self.promote_with(other) == *other || self.promote_with(other) == *self
    }
}

impl Sink for GeoPackageWriter {
    fn name(&self) -> &str {
        "GeoPackageWriter"
    }

    fn process(&mut self, ctx: ExecutorContext) -> Result<(), BoxedError> {
        let feature = ctx.feature.clone();

        // Apply Z coordinate transformation first (before borrowing tables)
        let transformed_feature = self.transform_z_coordinates(feature)?;

        // Determine which table this feature belongs to (for multi-layer support)
        let table_name = if let Some(ref group_attr) = self.params.group_by {
            if let Some(value) = transformed_feature.get(group_attr) {
                // Create table name from group value
                let group_value = match value {
                    AttributeValue::String(s) => sanitize_table_name(s),
                    AttributeValue::Number(n) => sanitize_table_name(&n.to_string()),
                    AttributeValue::Bool(b) => sanitize_table_name(&b.to_string()),
                    _ => "default".to_string(),
                };
                format!("{}_{}", self.default_table, group_value)
            } else {
                self.default_table.clone()
            }
        } else {
            self.default_table.clone()
        };

        // Get or create table data
        let table_data = self.tables.entry(table_name).or_default();

        // Infer schema from ALL features to avoid data loss
        // Merge schema from each feature to capture all possible attributes
        // Handle type conflicts by promoting to more general types
        for (key, value) in transformed_feature.attributes.iter() {
            let key_str = key.to_string();
            // Skip geometry column, primary key, and common system columns
            if key_str != self.params.geometry_column
                && key_str != self.params.primary_key
                && key_str != "fid"
                && key_str != "id"
            {
                // Skip group_by attribute if using it for grouping (it's implicit in table name)
                if let Some(ref group_attr) = self.params.group_by {
                    if key_str == group_attr.to_string() {
                        continue;
                    }
                }

                // Only infer type from non-null values
                if let Some(new_type) = AttributeType::from_attribute_value(value) {
                    if let Some(existing_type) = table_data.schema.get(&key_str) {
                        // Type conflict detected - promote to more general type
                        if existing_type != &new_type {
                            let promoted_type = existing_type.promote_with(&new_type);
                            table_data.schema.insert(key_str, promoted_type);
                        }
                    } else {
                        // New attribute - add to schema
                        table_data.schema.insert(key_str, new_type);
                    }
                } else {
                    // NULL value - only add to schema if not already present
                    // Use Text as default type for NULL-only columns
                    if !table_data.schema.contains_key(&key_str) {
                        table_data.schema.insert(key_str, AttributeType::Text);
                    }
                }
            }
        }

        table_data.features.push(transformed_feature);
        Ok(())
    }

    fn finish(&self, ctx: NodeContext) -> Result<(), BoxedError> {
        // Check if any tables have data
        let has_data = self.tables.values().any(|t| !t.features.is_empty());
        if !has_data {
            return Ok(());
        }

        let storage_resolver = Arc::clone(&ctx.storage_resolver);
        let expr_engine = Arc::clone(&ctx.expr_engine);
        let output = self.params.output.clone();
        let scope = expr_engine.new_scope();
        let path = scope
            .eval::<String>(output.as_ref())
            .unwrap_or_else(|_| output.as_ref().to_string());
        let output_uri = Uri::from_str(path.as_str())?;

        // Check if file exists
        let storage = storage_resolver
            .resolve(&output_uri)
            .map_err(SinkError::geopackage_writer)?;

        if !self.params.overwrite {
            if let Ok(true) = storage.exists_sync(output_uri.path().as_path()) {
                return Err(SinkError::GeoPackageWriter(format!(
                    "File already exists: {}. Set overwrite=true to replace it.",
                    path
                ))
                .into());
            }
        }

        // Create GeoPackage file
        let gpkg_data = self.create_geopackage()?;

        // Write to storage
        storage.put_sync(output_uri.path().as_path(), Bytes::from(gpkg_data))?;

        Ok(())
    }
}

/// Sanitize a string to be used as part of a table name
fn sanitize_table_name(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim_matches('_')
        .to_string()
}

impl GeoPackageWriter {
    /// Transform Z coordinates based on z_mode setting
    fn transform_z_coordinates(&self, mut feature: Feature) -> Result<Feature, BoxedError> {
        match self.params.z_mode {
            ZCoordinateMode::Optional => {
                // Keep as-is
                Ok(feature)
            }
            ZCoordinateMode::Required => {
                // Add Z=0 if missing
                feature.geometry = force_3d_geometry(&feature.geometry);
                Ok(feature)
            }
            ZCoordinateMode::NotAllowed => {
                // Drop Z coordinates
                feature.geometry = force_2d_geometry(&feature.geometry);
                Ok(feature)
            }
        }
    }

    fn create_geopackage(&self) -> Result<Vec<u8>, BoxedError> {
        // Create in-memory SQLite database
        let temp_file = tempfile::NamedTempFile::new()
            .map_err(|e| SinkError::GeoPackageWriter(format!("Failed to create temp file: {e}")))?;

        let db_path = temp_file.path().to_str().ok_or_else(|| {
            SinkError::GeoPackageWriter("Failed to get temp file path".to_string())
        })?;

        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| SinkError::GeoPackageWriter(format!("Failed to create runtime: {e}")))?;

        rt.block_on(async {
            let adapter = SqlAdapter::new(&format!("sqlite://{db_path}"), 1)
                .await
                .map_err(|e| {
                    SinkError::GeoPackageWriter(format!("Failed to connect to database: {e}"))
                })?;

            // Initialize GeoPackage structure
            self.init_geopackage_structure(&adapter).await?;

            // Process each table (multi-layer support)
            for (table_name, table_data) in &self.tables {
                if table_data.features.is_empty() {
                    continue;
                }

                // Handle table based on mode
                self.handle_table_mode_for_table(&adapter, table_name, table_data)
                    .await?;

                // Insert features with batching
                self.insert_features_batched(&adapter, table_name, table_data)
                    .await?;

                // Create spatial index if requested
                if self.params.create_spatial_index {
                    if self.params.table_mode != TableMode::UseExisting {
                        self.create_spatial_index_for_table(&adapter, table_name, table_data)
                            .await?;
                    } else {
                        let _ = self
                            .create_spatial_index_for_table(&adapter, table_name, table_data)
                            .await;
                    }
                }
            }

            Ok::<(), BoxedError>(())
        })?;

        // Read the file content
        let content = std::fs::read(temp_file.path())
            .map_err(|e| SinkError::GeoPackageWriter(format!("Failed to read temp file: {e}")))?;

        Ok(content)
    }

    /// Handle table creation/modification based on table mode for a specific table
    async fn handle_table_mode_for_table(
        &self,
        adapter: &SqlAdapter,
        table_name: &str,
        table_data: &TableData,
    ) -> Result<(), BoxedError> {
        match self.params.table_mode {
            TableMode::CreateIfNeeded => {
                if self.table_exists_by_name(adapter, table_name).await? {
                    self.verify_and_update_schema_for_table(adapter, table_name, table_data)
                        .await?;
                } else {
                    self.create_feature_table_for_table(adapter, table_name, table_data)
                        .await?;
                }
            }
            TableMode::UseExisting => {
                if !self.table_exists_by_name(adapter, table_name).await? {
                    return Err(SinkError::GeoPackageWriter(format!(
                        "Table '{}' does not exist. TableMode is UseExisting, which requires an existing table.",
                        table_name
                    ))
                    .into());
                }
                self.verify_and_update_schema_for_table(adapter, table_name, table_data)
                    .await?;
            }
            TableMode::DropAndCreate => {
                if self.table_exists_by_name(adapter, table_name).await? {
                    self.drop_table_by_name(adapter, table_name).await?;
                }
                self.create_feature_table_for_table(adapter, table_name, table_data)
                    .await?;
            }
        }
        Ok(())
    }

    /// Check if a table exists by name
    async fn table_exists_by_name(
        &self,
        adapter: &SqlAdapter,
        table_name: &str,
    ) -> Result<bool, BoxedError> {
        let query = format!(
            "SELECT name FROM sqlite_master WHERE type='table' AND name='{}'",
            table_name.replace('\'', "''")
        );
        let rows = adapter.fetch_many(&query).await.map_err(|e| {
            SinkError::GeoPackageWriter(format!("Failed to check table existence: {e}"))
        })?;
        Ok(!rows.is_empty())
    }

    /// Drop a table by name and its metadata
    async fn drop_table_by_name(
        &self,
        adapter: &SqlAdapter,
        table_name: &str,
    ) -> Result<(), BoxedError> {
        let rtree_table = format!(
            "rtree_{}_{}",
            table_name.replace(|c: char| !c.is_alphanumeric() && c != '_', "_"),
            self.params
                .geometry_column
                .replace(|c: char| !c.is_alphanumeric() && c != '_', "_")
        );
        let drop_rtree = format!("DROP TABLE IF EXISTS {}", quote_identifier(&rtree_table));
        let _ = adapter.execute(&drop_rtree).await;

        let delete_ext = format!(
            "DELETE FROM gpkg_extensions WHERE table_name = '{}'",
            table_name.replace('\'', "''")
        );
        let _ = adapter.execute(&delete_ext).await;

        let delete_geom = format!(
            "DELETE FROM gpkg_geometry_columns WHERE table_name = '{}'",
            table_name.replace('\'', "''")
        );
        let _ = adapter.execute(&delete_geom).await;

        let delete_contents = format!(
            "DELETE FROM gpkg_contents WHERE table_name = '{}'",
            table_name.replace('\'', "''")
        );
        let _ = adapter.execute(&delete_contents).await;

        let drop_table = format!("DROP TABLE IF EXISTS {}", quote_identifier(table_name));
        adapter.execute(&drop_table).await.map_err(|e| {
            SinkError::GeoPackageWriter(format!("Failed to drop table: {e}"))
        })?;

        Ok(())
    }

    /// Verify schema compatibility with existing table and add missing columns
    async fn verify_and_update_schema_for_table(
        &self,
        adapter: &SqlAdapter,
        table_name: &str,
        table_data: &TableData,
    ) -> Result<(), BoxedError> {
        let query = format!("PRAGMA table_info({})", quote_identifier(table_name));
        let rows = adapter.fetch_many(&query).await.map_err(|e| {
            SinkError::GeoPackageWriter(format!("Failed to get table info: {e}"))
        })?;

        let mut existing_columns: HashMap<String, String> = HashMap::new();
        for row in rows {
            if let (Ok(name), Ok(col_type)) = (
                row.try_get::<String, _>(1),
                row.try_get::<String, _>(2),
            ) {
                existing_columns.insert(name.to_lowercase(), col_type.to_uppercase());
            }
        }

        for (name, attr_type) in &table_data.schema {
            let name_lower = name.to_lowercase();
            if !existing_columns.contains_key(&name_lower) {
                let add_column = format!(
                    "ALTER TABLE {} ADD COLUMN {} {}",
                    quote_identifier(table_name),
                    quote_identifier(name),
                    attr_type.to_sql_type()
                );
                adapter.execute(&add_column).await.map_err(|e| {
                    SinkError::GeoPackageWriter(format!("Failed to add column '{}': {e}", name))
                })?;
            }
        }

        let (min_x, min_y, max_x, max_y) = self.calculate_bbox_for_table(table_data);
        let update_bbox = format!(
            r#"
            UPDATE gpkg_contents SET
                min_x = MIN(COALESCE(min_x, {}), {}),
                min_y = MIN(COALESCE(min_y, {}), {}),
                max_x = MAX(COALESCE(max_x, {}), {}),
                max_y = MAX(COALESCE(max_y, {}), {}),
                last_change = strftime('%Y-%m-%dT%H:%M:%fZ','now')
            WHERE table_name = '{}'
            "#,
            min_x, min_x, min_y, min_y, max_x, max_x, max_y, max_y,
            table_name.replace('\'', "''")
        );
        let _ = adapter.execute(&update_bbox).await;

        Ok(())
    }

    async fn init_geopackage_structure(&self, adapter: &SqlAdapter) -> Result<(), BoxedError> {
        // Set application_id for GeoPackage
        adapter
            .execute("PRAGMA application_id = 0x47503130")
            .await
            .map_err(|e| {
                SinkError::GeoPackageWriter(format!("Failed to set application_id: {e}"))
            })?;

        // Create gpkg_spatial_ref_sys table
        adapter
            .execute(
                r#"
                CREATE TABLE gpkg_spatial_ref_sys (
                    srs_name TEXT NOT NULL,
                    srs_id INTEGER NOT NULL PRIMARY KEY,
                    organization TEXT NOT NULL,
                    organization_coordsys_id INTEGER NOT NULL,
                    definition TEXT NOT NULL,
                    description TEXT
                )
                "#,
            )
            .await
            .map_err(|e| {
                SinkError::GeoPackageWriter(format!("Failed to create gpkg_spatial_ref_sys: {e}"))
            })?;

        // Insert standard SRS definitions
        self.insert_standard_srs(adapter).await?;

        // Create gpkg_contents table
        adapter
            .execute(
                r#"
                CREATE TABLE gpkg_contents (
                    table_name TEXT NOT NULL PRIMARY KEY,
                    data_type TEXT NOT NULL,
                    identifier TEXT UNIQUE,
                    description TEXT DEFAULT '',
                    last_change DATETIME NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
                    min_x DOUBLE,
                    min_y DOUBLE,
                    max_x DOUBLE,
                    max_y DOUBLE,
                    srs_id INTEGER,
                    CONSTRAINT fk_gc_r_srs_id FOREIGN KEY (srs_id) REFERENCES gpkg_spatial_ref_sys(srs_id)
                )
                "#,
            )
            .await
            .map_err(|e| {
                SinkError::GeoPackageWriter(format!("Failed to create gpkg_contents: {e}"))
            })?;

        // Create gpkg_geometry_columns table
        adapter
            .execute(
                r#"
                CREATE TABLE gpkg_geometry_columns (
                    table_name TEXT NOT NULL,
                    column_name TEXT NOT NULL,
                    geometry_type_name TEXT NOT NULL,
                    srs_id INTEGER NOT NULL,
                    z TINYINT NOT NULL,
                    m TINYINT NOT NULL,
                    CONSTRAINT pk_geom_cols PRIMARY KEY (table_name, column_name),
                    CONSTRAINT fk_gc_tn FOREIGN KEY (table_name) REFERENCES gpkg_contents(table_name),
                    CONSTRAINT fk_gc_srs FOREIGN KEY (srs_id) REFERENCES gpkg_spatial_ref_sys(srs_id)
                )
                "#,
            )
            .await
            .map_err(|e| {
                SinkError::GeoPackageWriter(format!(
                    "Failed to create gpkg_geometry_columns: {e}"
                ))
            })?;

        // Create gpkg_extensions table (required for spatial index)
        adapter
            .execute(
                r#"
                CREATE TABLE gpkg_extensions (
                    table_name TEXT,
                    column_name TEXT,
                    extension_name TEXT NOT NULL,
                    definition TEXT NOT NULL,
                    scope TEXT NOT NULL,
                    CONSTRAINT ge_tce UNIQUE (table_name, column_name, extension_name)
                )
                "#,
            )
            .await
            .map_err(|e| {
                SinkError::GeoPackageWriter(format!("Failed to create gpkg_extensions: {e}"))
            })?;

        Ok(())
    }

    async fn insert_standard_srs(&self, adapter: &SqlAdapter) -> Result<(), BoxedError> {
        // Insert EPSG:4326 (WGS84)
        adapter
            .execute(
                r#"
                INSERT INTO gpkg_spatial_ref_sys (srs_name, srs_id, organization, organization_coordsys_id, definition, description)
                VALUES ('WGS 84', 4326, 'EPSG', 4326, 
                'GEOGCS["WGS 84",DATUM["WGS_1984",SPHEROID["WGS 84",6378137,298.257223563,AUTHORITY["EPSG","7030"]],AUTHORITY["EPSG","6326"]],PRIMEM["Greenwich",0,AUTHORITY["EPSG","8901"]],UNIT["degree",0.0174532925199433,AUTHORITY["EPSG","9122"]],AUTHORITY["EPSG","4326"]]',
                'WGS 84 geographic 2D')
                "#,
            )
            .await
            .map_err(|e| {
                SinkError::GeoPackageWriter(format!("Failed to insert EPSG:4326: {e}"))
            })?;

        // Insert undefined geographic SRS (-1)
        adapter
            .execute(
                r#"
                INSERT INTO gpkg_spatial_ref_sys (srs_name, srs_id, organization, organization_coordsys_id, definition, description)
                VALUES ('Undefined geographic SRS', -1, 'NONE', -1, 'undefined', 'undefined geographic coordinate reference system')
                "#,
            )
            .await
            .map_err(|e| {
                SinkError::GeoPackageWriter(format!("Failed to insert undefined SRS: {e}"))
            })?;

        // Insert undefined Cartesian SRS (0)
        adapter
            .execute(
                r#"
                INSERT INTO gpkg_spatial_ref_sys (srs_name, srs_id, organization, organization_coordsys_id, definition, description)
                VALUES ('Undefined Cartesian SRS', 0, 'NONE', 0, 'undefined', 'undefined Cartesian coordinate reference system')
                "#,
            )
            .await
            .map_err(|e| {
                SinkError::GeoPackageWriter(format!("Failed to insert Cartesian SRS: {e}"))
            })?;

        // Insert custom SRS if not 4326
        if self.params.srs_id != 4326 && self.params.srs_id != -1 && self.params.srs_id != 0 {
            let query = format!(
                r#"
                INSERT OR IGNORE INTO gpkg_spatial_ref_sys (srs_name, srs_id, organization, organization_coordsys_id, definition, description)
                VALUES ('Custom SRS', {}, 'EPSG', {}, 'undefined', 'Custom spatial reference system')
                "#,
                self.params.srs_id, self.params.srs_id
            );
            adapter.execute(&query).await.map_err(|e| {
                SinkError::GeoPackageWriter(format!("Failed to insert custom SRS: {e}"))
            })?;
        }

        Ok(())
    }

    async fn create_feature_table_for_table(
        &self,
        adapter: &SqlAdapter,
        table_name: &str,
        table_data: &TableData,
    ) -> Result<(), BoxedError> {
        // Build column definitions with proper SQL identifier quoting
        let pk_col = if self.params.primary_key_attribute.is_some() {
            format!("{} INTEGER PRIMARY KEY", quote_identifier(&self.params.primary_key))
        } else {
            format!("{} INTEGER PRIMARY KEY AUTOINCREMENT", quote_identifier(&self.params.primary_key))
        };

        let mut columns = vec![
            pk_col,
            format!("{} BLOB", quote_identifier(&self.params.geometry_column)),
        ];

        for (name, attr_type) in &table_data.schema {
            columns.push(format!(
                "{} {}",
                quote_identifier(name),
                attr_type.to_sql_type()
            ));
        }

        let create_table = format!(
            "CREATE TABLE {} ({})",
            quote_identifier(table_name),
            columns.join(", ")
        );

        adapter.execute(&create_table).await.map_err(|e| {
            SinkError::GeoPackageWriter(format!("Failed to create feature table: {e}"))
        })?;

        let (min_x, min_y, max_x, max_y) = self.calculate_bbox_for_table(table_data);

        let insert_contents = format!(
            r#"
            INSERT INTO gpkg_contents (table_name, data_type, identifier, description, srs_id, min_x, min_y, max_x, max_y)
            VALUES ('{}', 'features', '{}', '', {}, {}, {}, {}, {})
            "#,
            table_name.replace('\'', "''"),
            table_name.replace('\'', "''"),
            self.params.srs_id,
            min_x, min_y, max_x, max_y
        );

        adapter.execute(&insert_contents).await.map_err(|e| {
            SinkError::GeoPackageWriter(format!("Failed to insert into gpkg_contents: {e}"))
        })?;

        // Detect Z dimension based on z_mode and first geometry
        let has_z = match self.params.z_mode {
            ZCoordinateMode::Required => true,
            ZCoordinateMode::NotAllowed => false,
            ZCoordinateMode::Optional => table_data
                .features
                .first()
                .map(|f| matches!(&f.geometry.value, GeometryValue::FlowGeometry3D(_)))
                .unwrap_or(false),
        };

        // Update geometry type name based on Z dimension
        let geom_type_name = if has_z && !self.params.geometry_type.to_uppercase().ends_with('Z') {
            format!("{}Z", self.params.geometry_type.to_uppercase())
        } else if !has_z && self.params.geometry_type.to_uppercase().ends_with('Z') {
            self.params.geometry_type.to_uppercase().trim_end_matches('Z').to_string()
        } else {
            self.params.geometry_type.to_uppercase()
        };

        let insert_geom_cols = format!(
            r#"
            INSERT INTO gpkg_geometry_columns (table_name, column_name, geometry_type_name, srs_id, z, m)
            VALUES ('{}', '{}', '{}', {}, {}, 0)
            "#,
            table_name.replace('\'', "''"),
            self.params.geometry_column.replace('\'', "''"),
            geom_type_name.replace('\'', "''"),
            self.params.srs_id,
            if has_z { 1 } else { 0 }
        );

        adapter.execute(&insert_geom_cols).await.map_err(|e| {
            SinkError::GeoPackageWriter(format!("Failed to insert into gpkg_geometry_columns: {e}"))
        })?;

        Ok(())
    }

    /// Insert features with batching for better performance
    async fn insert_features_batched(
        &self,
        adapter: &SqlAdapter,
        table_name: &str,
        table_data: &TableData,
    ) -> Result<(), BoxedError> {
        let batch_size = if self.params.batch_size == 0 {
            table_data.features.len()
        } else {
            self.params.batch_size
        };

        // Process features in batches
        for (batch_idx, batch) in table_data.features.chunks(batch_size).enumerate() {
            // Begin transaction
            adapter.execute("BEGIN TRANSACTION").await.map_err(|e| {
                SinkError::GeoPackageWriter(format!("Failed to begin transaction: {e}"))
            })?;

            for (idx, feature) in batch.iter().enumerate() {
                let feature_idx = batch_idx * batch_size + idx;
                let geom_blob = geometry_to_gpkg_wkb(&feature.geometry, self.params.srs_id)?;
                let insert_query = self.build_insert_query_for_table(
                    feature,
                    &geom_blob,
                    table_name,
                    table_data,
                    feature_idx,
                )?;

                adapter.execute(&insert_query).await.map_err(|e| {
                    // Rollback on error
                    SinkError::GeoPackageWriter(format!("Failed to insert feature: {e}"))
                })?;
            }

            // Commit transaction
            adapter.execute("COMMIT").await.map_err(|e| {
                SinkError::GeoPackageWriter(format!("Failed to commit transaction: {e}"))
            })?;
        }

        Ok(())
    }

    fn build_insert_query_for_table(
        &self,
        feature: &Feature,
        geom_blob: &[u8],
        table_name: &str,
        table_data: &TableData,
        _feature_idx: usize,
    ) -> Result<String, BoxedError> {
        let mut column_names = vec![];
        let mut values = vec![];

        // Handle primary key
        if let Some(ref pk_attr) = self.params.primary_key_attribute {
            if let Some(pk_value) = feature.get(pk_attr) {
                column_names.push(quote_identifier(&self.params.primary_key));
                values.push(attribute_value_to_sql_string(pk_value)?);
            }
        }

        // Geometry column
        column_names.push(quote_identifier(&self.params.geometry_column));
        values.push(format!("X'{}'", hex::encode(geom_blob)));

        // Other attributes
        for (name, _) in &table_data.schema {
            let value = feature
                .attributes
                .iter()
                .find(|(k, _)| k.to_string() == *name)
                .map(|(_, v)| v);

            column_names.push(quote_identifier(name));
            if let Some(value) = value {
                values.push(attribute_value_to_sql_string(value)?);
            } else {
                values.push("NULL".to_string());
            }
        }

        Ok(format!(
            "INSERT INTO {} ({}) VALUES ({})",
            quote_identifier(table_name),
            column_names.join(", "),
            values.join(", ")
        ))
    }

    async fn create_spatial_index_for_table(
        &self,
        adapter: &SqlAdapter,
        table_name: &str,
        table_data: &TableData,
    ) -> Result<(), BoxedError> {
        let rtree_table = format!(
            "rtree_{}_{}",
            table_name.replace(|c: char| !c.is_alphanumeric() && c != '_', "_"),
            self.params
                .geometry_column
                .replace(|c: char| !c.is_alphanumeric() && c != '_', "_")
        );

        let create_rtree = format!(
            r#"
            CREATE VIRTUAL TABLE {} USING rtree(
                id,
                minx, maxx,
                miny, maxy
            )
            "#,
            quote_identifier(&rtree_table)
        );

        adapter.execute(&create_rtree).await.map_err(|e| {
            SinkError::GeoPackageWriter(format!("Failed to create spatial index: {e}"))
        })?;

        for (idx, feature) in table_data.features.iter().enumerate() {
            let bbox = calculate_geometry_bbox(&feature.geometry);
            if let Some((min_x, min_y, max_x, max_y)) = bbox {
                let insert_rtree = format!(
                    "INSERT INTO {} VALUES ({}, {}, {}, {}, {})",
                    quote_identifier(&rtree_table),
                    idx + 1,
                    min_x,
                    max_x,
                    min_y,
                    max_y
                );
                adapter.execute(&insert_rtree).await.map_err(|e| {
                    SinkError::GeoPackageWriter(format!("Failed to populate spatial index: {e}"))
                })?;
            }
        }

        let register_ext = format!(
            r#"
            INSERT INTO gpkg_extensions (table_name, column_name, extension_name, definition, scope)
            VALUES ('{}', '{}', 'gpkg_rtree_index', 'http://www.geopackage.org/spec120/#extension_rtree', 'write-only')
            "#,
            table_name.replace('\'', "''"),
            self.params.geometry_column.replace('\'', "''")
        );

        adapter.execute(&register_ext).await.map_err(|e| {
            SinkError::GeoPackageWriter(format!("Failed to register rtree extension: {e}"))
        })?;

        Ok(())
    }

    fn calculate_bbox_for_table(&self, table_data: &TableData) -> (f64, f64, f64, f64) {
        let mut min_x = f64::INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut max_y = f64::NEG_INFINITY;

        for feature in &table_data.features {
            if let Some((fx, fy, fx2, fy2)) = calculate_geometry_bbox(&feature.geometry) {
                min_x = min_x.min(fx);
                min_y = min_y.min(fy);
                max_x = max_x.max(fx2);
                max_y = max_y.max(fy2);
            }
        }

        (min_x, min_y, max_x, max_y)
    }
}

fn calculate_geometry_bbox(geometry: &Geometry) -> Option<(f64, f64, f64, f64)> {
    match &geometry.value {
        GeometryValue::FlowGeometry2D(geom) => calculate_bbox_2d(geom),
        GeometryValue::FlowGeometry3D(geom) => calculate_bbox_3d(geom),
        _ => None,
    }
}

fn calculate_bbox_2d(geom: &Geometry2D) -> Option<(f64, f64, f64, f64)> {
    match geom {
        Geometry2D::Point(pt) => Some((pt.x(), pt.y(), pt.x(), pt.y())),
        Geometry2D::LineString(ls) => {
            let mut min_x = f64::INFINITY;
            let mut min_y = f64::INFINITY;
            let mut max_x = f64::NEG_INFINITY;
            let mut max_y = f64::NEG_INFINITY;
            for coord in ls.coords() {
                min_x = min_x.min(coord.x);
                min_y = min_y.min(coord.y);
                max_x = max_x.max(coord.x);
                max_y = max_y.max(coord.y);
            }
            Some((min_x, min_y, max_x, max_y))
        }
        Geometry2D::Polygon(poly) => {
            let mut min_x = f64::INFINITY;
            let mut min_y = f64::INFINITY;
            let mut max_x = f64::NEG_INFINITY;
            let mut max_y = f64::NEG_INFINITY;
            for coord in poly.exterior().coords() {
                min_x = min_x.min(coord.x);
                min_y = min_y.min(coord.y);
                max_x = max_x.max(coord.x);
                max_y = max_y.max(coord.y);
            }
            Some((min_x, min_y, max_x, max_y))
        }
        _ => None,
    }
}

fn calculate_bbox_3d(geom: &Geometry3D) -> Option<(f64, f64, f64, f64)> {
    match geom {
        Geometry3D::Point(pt) => Some((pt.x(), pt.y(), pt.x(), pt.y())),
        Geometry3D::LineString(ls) => {
            let mut min_x = f64::INFINITY;
            let mut min_y = f64::INFINITY;
            let mut max_x = f64::NEG_INFINITY;
            let mut max_y = f64::NEG_INFINITY;
            for coord in ls.coords() {
                min_x = min_x.min(coord.x);
                min_y = min_y.min(coord.y);
                max_x = max_x.max(coord.x);
                max_y = max_y.max(coord.y);
            }
            Some((min_x, min_y, max_x, max_y))
        }
        Geometry3D::Polygon(poly) => {
            let mut min_x = f64::INFINITY;
            let mut min_y = f64::INFINITY;
            let mut max_x = f64::NEG_INFINITY;
            let mut max_y = f64::NEG_INFINITY;
            for coord in poly.exterior().coords() {
                min_x = min_x.min(coord.x);
                min_y = min_y.min(coord.y);
                max_x = max_x.max(coord.x);
                max_y = max_y.max(coord.y);
            }
            Some((min_x, min_y, max_x, max_y))
        }
        _ => None,
    }
}

fn geometry_to_gpkg_wkb(geometry: &Geometry, srs_id: i32) -> Result<Vec<u8>, BoxedError> {
    let mut buffer = Vec::new();

    // GeoPackage Binary header
    // Magic number "GP" (0x47 0x50)
    buffer.push(0x47);
    buffer.push(0x50);

    // Version (0 for GeoPackage 1.0-1.2)
    buffer.push(0x00);

    // Flags byte
    // bit 0: endianness (0 = big, 1 = little) - we use little endian
    // bits 1-3: envelope type (0 = no envelope, 1 = XY, 2 = XYZ, 3 = XYM, 4 = XYZM)
    // bit 4: empty (0 = not empty)
    // bits 5-7: binary type (0 = standard WKB)
    let is_3d = matches!(geometry.value, GeometryValue::FlowGeometry3D(_));
    let envelope_type = if is_3d { 2 } else { 1 }; // XY or XYZ
    let flags = 0x01 | (envelope_type << 1); // little endian + envelope type
    buffer.push(flags);

    // SRS ID (4 bytes, little endian)
    buffer
        .write_i32::<LittleEndian>(srs_id)
        .map_err(|e| SinkError::GeoPackageWriter(format!("Failed to write SRS ID: {e}")))?;

    // Envelope (optional - we'll include it)
    let bbox = calculate_geometry_bbox(geometry);
    if let Some((min_x, min_y, max_x, max_y)) = bbox {
        buffer
            .write_f64::<LittleEndian>(min_x)
            .map_err(|e| SinkError::GeoPackageWriter(format!("Failed to write envelope: {e}")))?;
        buffer
            .write_f64::<LittleEndian>(max_x)
            .map_err(|e| SinkError::GeoPackageWriter(format!("Failed to write envelope: {e}")))?;
        buffer
            .write_f64::<LittleEndian>(min_y)
            .map_err(|e| SinkError::GeoPackageWriter(format!("Failed to write envelope: {e}")))?;
        buffer
            .write_f64::<LittleEndian>(max_y)
            .map_err(|e| SinkError::GeoPackageWriter(format!("Failed to write envelope: {e}")))?;

        if is_3d {
            // For 3D, add min_z and max_z (we'll use 0 for now as we don't track Z in bbox)
            buffer.write_f64::<LittleEndian>(0.0).map_err(|e| {
                SinkError::GeoPackageWriter(format!("Failed to write envelope: {e}"))
            })?;
            buffer.write_f64::<LittleEndian>(0.0).map_err(|e| {
                SinkError::GeoPackageWriter(format!("Failed to write envelope: {e}"))
            })?;
        }
    }

    // WKB geometry
    let wkb = geometry_to_wkb(geometry)?;
    buffer.extend_from_slice(&wkb);

    Ok(buffer)
}

fn geometry_to_wkb(geometry: &Geometry) -> Result<Vec<u8>, BoxedError> {
    match &geometry.value {
        GeometryValue::FlowGeometry2D(geom) => geometry_2d_to_wkb(geom),
        GeometryValue::FlowGeometry3D(geom) => geometry_3d_to_wkb(geom),
        _ => Err(SinkError::GeoPackageWriter("Unsupported geometry type".to_string()).into()),
    }
}

fn geometry_2d_to_wkb(geom: &Geometry2D) -> Result<Vec<u8>, BoxedError> {
    let mut buffer = Vec::new();

    // Byte order (1 = little endian)
    buffer.push(0x01);

    match geom {
        Geometry2D::Point(pt) => {
            // WKB type for Point (1)
            buffer.write_u32::<LittleEndian>(1).map_err(|e| {
                SinkError::GeoPackageWriter(format!("Failed to write WKB type: {e}"))
            })?;
            buffer.write_f64::<LittleEndian>(pt.x()).map_err(|e| {
                SinkError::GeoPackageWriter(format!("Failed to write coordinate: {e}"))
            })?;
            buffer.write_f64::<LittleEndian>(pt.y()).map_err(|e| {
                SinkError::GeoPackageWriter(format!("Failed to write coordinate: {e}"))
            })?;
        }
        Geometry2D::LineString(ls) => {
            // WKB type for LineString (2)
            buffer.write_u32::<LittleEndian>(2).map_err(|e| {
                SinkError::GeoPackageWriter(format!("Failed to write WKB type: {e}"))
            })?;
            let coords: Vec<_> = ls.coords().collect();
            buffer
                .write_u32::<LittleEndian>(coords.len() as u32)
                .map_err(|e| {
                    SinkError::GeoPackageWriter(format!("Failed to write point count: {e}"))
                })?;
            for coord in coords {
                buffer.write_f64::<LittleEndian>(coord.x).map_err(|e| {
                    SinkError::GeoPackageWriter(format!("Failed to write coordinate: {e}"))
                })?;
                buffer.write_f64::<LittleEndian>(coord.y).map_err(|e| {
                    SinkError::GeoPackageWriter(format!("Failed to write coordinate: {e}"))
                })?;
            }
        }
        Geometry2D::Polygon(poly) => {
            // WKB type for Polygon (3)
            buffer.write_u32::<LittleEndian>(3).map_err(|e| {
                SinkError::GeoPackageWriter(format!("Failed to write WKB type: {e}"))
            })?;
            // Number of rings
            buffer
                .write_u32::<LittleEndian>(1 + poly.interiors().len() as u32)
                .map_err(|e| {
                    SinkError::GeoPackageWriter(format!("Failed to write ring count: {e}"))
                })?;
            // Exterior ring
            write_linestring_coords_2d(&mut buffer, poly.exterior())?;
            // Interior rings
            for interior in poly.interiors() {
                write_linestring_coords_2d(&mut buffer, interior)?;
            }
        }
        Geometry2D::MultiPoint(mp) => {
            // WKB type for MultiPoint (4)
            buffer.write_u32::<LittleEndian>(4).map_err(|e| {
                SinkError::GeoPackageWriter(format!("Failed to write WKB type: {e}"))
            })?;
            buffer
                .write_u32::<LittleEndian>(mp.0.len() as u32)
                .map_err(|e| {
                    SinkError::GeoPackageWriter(format!("Failed to write point count: {e}"))
                })?;
            for pt in mp.iter() {
                buffer.push(0x01); // byte order
                buffer.write_u32::<LittleEndian>(1).map_err(|e| {
                    SinkError::GeoPackageWriter(format!("Failed to write WKB type: {e}"))
                })?; // Point type
                buffer.write_f64::<LittleEndian>(pt.x()).map_err(|e| {
                    SinkError::GeoPackageWriter(format!("Failed to write coordinate: {e}"))
                })?;
                buffer.write_f64::<LittleEndian>(pt.y()).map_err(|e| {
                    SinkError::GeoPackageWriter(format!("Failed to write coordinate: {e}"))
                })?;
            }
        }
        Geometry2D::MultiLineString(mls) => {
            // WKB type for MultiLineString (5)
            buffer.write_u32::<LittleEndian>(5).map_err(|e| {
                SinkError::GeoPackageWriter(format!("Failed to write WKB type: {e}"))
            })?;
            buffer
                .write_u32::<LittleEndian>(mls.0.len() as u32)
                .map_err(|e| {
                    SinkError::GeoPackageWriter(format!("Failed to write linestring count: {e}"))
                })?;
            for ls in mls.iter() {
                buffer.push(0x01); // byte order
                buffer.write_u32::<LittleEndian>(2).map_err(|e| {
                    SinkError::GeoPackageWriter(format!("Failed to write WKB type: {e}"))
                })?; // LineString type
                write_linestring_coords_2d(&mut buffer, ls)?;
            }
        }
        Geometry2D::MultiPolygon(mpoly) => {
            // WKB type for MultiPolygon (6)
            buffer.write_u32::<LittleEndian>(6).map_err(|e| {
                SinkError::GeoPackageWriter(format!("Failed to write WKB type: {e}"))
            })?;
            buffer
                .write_u32::<LittleEndian>(mpoly.0.len() as u32)
                .map_err(|e| {
                    SinkError::GeoPackageWriter(format!("Failed to write polygon count: {e}"))
                })?;
            for poly in mpoly.iter() {
                buffer.push(0x01); // byte order
                buffer.write_u32::<LittleEndian>(3).map_err(|e| {
                    SinkError::GeoPackageWriter(format!("Failed to write WKB type: {e}"))
                })?; // Polygon type
                buffer
                    .write_u32::<LittleEndian>(1 + poly.interiors().len() as u32)
                    .map_err(|e| {
                        SinkError::GeoPackageWriter(format!("Failed to write ring count: {e}"))
                    })?;
                write_linestring_coords_2d(&mut buffer, poly.exterior())?;
                for interior in poly.interiors() {
                    write_linestring_coords_2d(&mut buffer, interior)?;
                }
            }
        }
        _ => {
            return Err(
                SinkError::GeoPackageWriter("Unsupported 2D geometry type".to_string()).into(),
            )
        }
    }

    Ok(buffer)
}

fn geometry_3d_to_wkb(geom: &Geometry3D) -> Result<Vec<u8>, BoxedError> {
    let mut buffer = Vec::new();

    // Byte order (1 = little endian)
    buffer.push(0x01);

    match geom {
        Geometry3D::Point(pt) => {
            // WKB type for Point Z (0x80000001)
            buffer.write_u32::<LittleEndian>(0x80000001).map_err(|e| {
                SinkError::GeoPackageWriter(format!("Failed to write WKB type: {e}"))
            })?;
            buffer.write_f64::<LittleEndian>(pt.x()).map_err(|e| {
                SinkError::GeoPackageWriter(format!("Failed to write coordinate: {e}"))
            })?;
            buffer.write_f64::<LittleEndian>(pt.y()).map_err(|e| {
                SinkError::GeoPackageWriter(format!("Failed to write coordinate: {e}"))
            })?;
            buffer.write_f64::<LittleEndian>(pt.z()).map_err(|e| {
                SinkError::GeoPackageWriter(format!("Failed to write coordinate: {e}"))
            })?;
        }
        Geometry3D::LineString(ls) => {
            // WKB type for LineString Z (0x80000002)
            buffer.write_u32::<LittleEndian>(0x80000002).map_err(|e| {
                SinkError::GeoPackageWriter(format!("Failed to write WKB type: {e}"))
            })?;
            let coords: Vec<_> = ls.coords().collect();
            buffer
                .write_u32::<LittleEndian>(coords.len() as u32)
                .map_err(|e| {
                    SinkError::GeoPackageWriter(format!("Failed to write point count: {e}"))
                })?;
            for coord in coords {
                buffer.write_f64::<LittleEndian>(coord.x).map_err(|e| {
                    SinkError::GeoPackageWriter(format!("Failed to write coordinate: {e}"))
                })?;
                buffer.write_f64::<LittleEndian>(coord.y).map_err(|e| {
                    SinkError::GeoPackageWriter(format!("Failed to write coordinate: {e}"))
                })?;
                buffer.write_f64::<LittleEndian>(coord.z).map_err(|e| {
                    SinkError::GeoPackageWriter(format!("Failed to write coordinate: {e}"))
                })?;
            }
        }
        Geometry3D::Polygon(poly) => {
            // WKB type for Polygon Z (0x80000003)
            buffer.write_u32::<LittleEndian>(0x80000003).map_err(|e| {
                SinkError::GeoPackageWriter(format!("Failed to write WKB type: {e}"))
            })?;
            buffer
                .write_u32::<LittleEndian>(1 + poly.interiors().len() as u32)
                .map_err(|e| {
                    SinkError::GeoPackageWriter(format!("Failed to write ring count: {e}"))
                })?;
            write_linestring_coords_3d(&mut buffer, poly.exterior())?;
            for interior in poly.interiors() {
                write_linestring_coords_3d(&mut buffer, interior)?;
            }
        }
        Geometry3D::MultiPoint(mp) => {
            // WKB type for MultiPoint Z (0x80000004)
            buffer.write_u32::<LittleEndian>(0x80000004).map_err(|e| {
                SinkError::GeoPackageWriter(format!("Failed to write WKB type: {e}"))
            })?;
            buffer
                .write_u32::<LittleEndian>(mp.0.len() as u32)
                .map_err(|e| {
                    SinkError::GeoPackageWriter(format!("Failed to write point count: {e}"))
                })?;
            for pt in mp.iter() {
                buffer.push(0x01); // byte order
                buffer.write_u32::<LittleEndian>(0x80000001).map_err(|e| {
                    SinkError::GeoPackageWriter(format!("Failed to write WKB type: {e}"))
                })?; // Point Z type
                buffer.write_f64::<LittleEndian>(pt.x()).map_err(|e| {
                    SinkError::GeoPackageWriter(format!("Failed to write coordinate: {e}"))
                })?;
                buffer.write_f64::<LittleEndian>(pt.y()).map_err(|e| {
                    SinkError::GeoPackageWriter(format!("Failed to write coordinate: {e}"))
                })?;
                buffer.write_f64::<LittleEndian>(pt.z()).map_err(|e| {
                    SinkError::GeoPackageWriter(format!("Failed to write coordinate: {e}"))
                })?;
            }
        }
        Geometry3D::MultiLineString(mls) => {
            // WKB type for MultiLineString Z (0x80000005)
            buffer.write_u32::<LittleEndian>(0x80000005).map_err(|e| {
                SinkError::GeoPackageWriter(format!("Failed to write WKB type: {e}"))
            })?;
            buffer
                .write_u32::<LittleEndian>(mls.0.len() as u32)
                .map_err(|e| {
                    SinkError::GeoPackageWriter(format!("Failed to write linestring count: {e}"))
                })?;
            for ls in mls.iter() {
                buffer.push(0x01); // byte order
                buffer.write_u32::<LittleEndian>(0x80000002).map_err(|e| {
                    SinkError::GeoPackageWriter(format!("Failed to write WKB type: {e}"))
                })?; // LineString Z type
                write_linestring_coords_3d(&mut buffer, ls)?;
            }
        }
        Geometry3D::MultiPolygon(mpoly) => {
            // WKB type for MultiPolygon Z (0x80000006)
            buffer.write_u32::<LittleEndian>(0x80000006).map_err(|e| {
                SinkError::GeoPackageWriter(format!("Failed to write WKB type: {e}"))
            })?;
            buffer
                .write_u32::<LittleEndian>(mpoly.0.len() as u32)
                .map_err(|e| {
                    SinkError::GeoPackageWriter(format!("Failed to write polygon count: {e}"))
                })?;
            for poly in mpoly.iter() {
                buffer.push(0x01); // byte order
                buffer.write_u32::<LittleEndian>(0x80000003).map_err(|e| {
                    SinkError::GeoPackageWriter(format!("Failed to write WKB type: {e}"))
                })?; // Polygon Z type
                buffer
                    .write_u32::<LittleEndian>(1 + poly.interiors().len() as u32)
                    .map_err(|e| {
                        SinkError::GeoPackageWriter(format!("Failed to write ring count: {e}"))
                    })?;
                write_linestring_coords_3d(&mut buffer, poly.exterior())?;
                for interior in poly.interiors() {
                    write_linestring_coords_3d(&mut buffer, interior)?;
                }
            }
        }
        _ => {
            return Err(
                SinkError::GeoPackageWriter("Unsupported 3D geometry type".to_string()).into(),
            )
        }
    }

    Ok(buffer)
}

fn write_linestring_coords_2d(
    buffer: &mut Vec<u8>,
    ls: &reearth_flow_geometry::types::line_string::LineString2D<f64>,
) -> Result<(), BoxedError> {
    let coords: Vec<_> = ls.coords().collect();
    buffer
        .write_u32::<LittleEndian>(coords.len() as u32)
        .map_err(|e| SinkError::GeoPackageWriter(format!("Failed to write point count: {e}")))?;
    for coord in coords {
        buffer
            .write_f64::<LittleEndian>(coord.x)
            .map_err(|e| SinkError::GeoPackageWriter(format!("Failed to write coordinate: {e}")))?;
        buffer
            .write_f64::<LittleEndian>(coord.y)
            .map_err(|e| SinkError::GeoPackageWriter(format!("Failed to write coordinate: {e}")))?;
    }
    Ok(())
}

fn write_linestring_coords_3d(
    buffer: &mut Vec<u8>,
    ls: &reearth_flow_geometry::types::line_string::LineString3D<f64>,
) -> Result<(), BoxedError> {
    let coords: Vec<_> = ls.coords().collect();
    buffer
        .write_u32::<LittleEndian>(coords.len() as u32)
        .map_err(|e| SinkError::GeoPackageWriter(format!("Failed to write point count: {e}")))?;
    for coord in coords {
        buffer
            .write_f64::<LittleEndian>(coord.x)
            .map_err(|e| SinkError::GeoPackageWriter(format!("Failed to write coordinate: {e}")))?;
        buffer
            .write_f64::<LittleEndian>(coord.y)
            .map_err(|e| SinkError::GeoPackageWriter(format!("Failed to write coordinate: {e}")))?;
        buffer
            .write_f64::<LittleEndian>(coord.z)
            .map_err(|e| SinkError::GeoPackageWriter(format!("Failed to write coordinate: {e}")))?;
    }
    Ok(())
}

/// Quote SQL identifier to prevent SQL injection
/// SQLite uses double quotes for identifiers
fn quote_identifier(name: &str) -> String {
    // Validate identifier: only allow alphanumeric, underscore, and some safe chars
    // This prevents SQL injection attacks
    if name.is_empty() {
        return "\"\"".to_string();
    }

    // Check for dangerous characters
    if name.contains(|c: char| !c.is_alphanumeric() && c != '_' && c != '-' && c != '.') {
        // If contains dangerous chars, escape them by doubling quotes
        format!("\"{}\"", name.replace('"', "\"\""))
    } else {
        format!("\"{}\"", name)
    }
}

fn attribute_value_to_sql_string(value: &AttributeValue) -> Result<String, BoxedError> {
    match value {
        AttributeValue::Bool(b) => Ok(if *b { "1".to_string() } else { "0".to_string() }),
        AttributeValue::Number(n) => Ok(n.to_string()),
        AttributeValue::String(s) => Ok(format!("'{}'", s.replace('\'', "''"))),
        AttributeValue::DateTime(dt) => Ok(format!("'{}'", dt.to_rfc3339().replace('\'', "''"))),
        AttributeValue::Bytes(b) => Ok(format!("X'{}'", hex::encode(b))),
        AttributeValue::Array(arr) => {
            let json = serde_json::to_string(arr).map_err(|e| {
                SinkError::GeoPackageWriter(format!("Failed to serialize array: {e}"))
            })?;
            Ok(format!("'{}'", json.replace('\'', "''")))
        }
        AttributeValue::Map(map) => {
            let json = serde_json::to_string(map).map_err(|e| {
                SinkError::GeoPackageWriter(format!("Failed to serialize map: {e}"))
            })?;
            Ok(format!("'{}'", json.replace('\'', "''")))
        }
        AttributeValue::Null => Ok("NULL".to_string()),
    }
}

// ============================================================================
// Z Coordinate Transformation Functions
// ============================================================================

/// Force geometry to 3D by adding Z=0 where missing
fn force_3d_geometry(geometry: &Geometry) -> Geometry {
    match &geometry.value {
        GeometryValue::FlowGeometry2D(geom2d) => {
            let geom3d = convert_2d_to_3d(geom2d);
            Geometry {
                epsg: geometry.epsg,
                value: GeometryValue::FlowGeometry3D(geom3d),
            }
        }
        GeometryValue::FlowGeometry3D(_) => geometry.clone(),
        _ => geometry.clone(),
    }
}

/// Force geometry to 2D by dropping Z coordinates
fn force_2d_geometry(geometry: &Geometry) -> Geometry {
    match &geometry.value {
        GeometryValue::FlowGeometry3D(geom3d) => {
            let geom2d = convert_3d_to_2d(geom3d);
            Geometry {
                epsg: geometry.epsg,
                value: GeometryValue::FlowGeometry2D(geom2d),
            }
        }
        GeometryValue::FlowGeometry2D(_) => geometry.clone(),
        _ => geometry.clone(),
    }
}

/// Convert 2D geometry to 3D by adding Z=0
fn convert_2d_to_3d(geom: &Geometry2D) -> Geometry3D {
    match geom {
        Geometry2D::Point(pt) => Geometry3D::Point(Point3D::from([pt.x(), pt.y(), 0.0])),
        Geometry2D::LineString(ls) => {
            let coords: Vec<(f64, f64, f64)> = ls.coords().map(|c| (c.x, c.y, 0.0)).collect();
            Geometry3D::LineString(LineString3D::from(coords))
        }
        Geometry2D::Polygon(poly) => {
            let exterior: Vec<(f64, f64, f64)> =
                poly.exterior().coords().map(|c| (c.x, c.y, 0.0)).collect();
            let interiors: Vec<LineString3D<f64>> = poly
                .interiors()
                .iter()
                .map(|ring| {
                    let coords: Vec<(f64, f64, f64)> =
                        ring.coords().map(|c| (c.x, c.y, 0.0)).collect();
                    LineString3D::from(coords)
                })
                .collect();
            Geometry3D::Polygon(Polygon3D::new(LineString3D::from(exterior), interiors))
        }
        Geometry2D::MultiPoint(mp) => {
            let points: Vec<Point3D<f64>> = mp
                .iter()
                .map(|pt| Point3D::from([pt.x(), pt.y(), 0.0]))
                .collect();
            Geometry3D::MultiPoint(MultiPoint3D::new(points))
        }
        Geometry2D::MultiLineString(mls) => {
            let lines: Vec<LineString3D<f64>> = mls
                .iter()
                .map(|ls| {
                    let coords: Vec<(f64, f64, f64)> =
                        ls.coords().map(|c| (c.x, c.y, 0.0)).collect();
                    LineString3D::from(coords)
                })
                .collect();
            Geometry3D::MultiLineString(MultiLineString3D::new(lines))
        }
        Geometry2D::MultiPolygon(mpoly) => {
            let polygons: Vec<Polygon3D<f64>> = mpoly
                .iter()
                .map(|poly| {
                    let exterior: Vec<(f64, f64, f64)> =
                        poly.exterior().coords().map(|c| (c.x, c.y, 0.0)).collect();
                    let interiors: Vec<LineString3D<f64>> = poly
                        .interiors()
                        .iter()
                        .map(|ring| {
                            let coords: Vec<(f64, f64, f64)> =
                                ring.coords().map(|c| (c.x, c.y, 0.0)).collect();
                            LineString3D::from(coords)
                        })
                        .collect();
                    Polygon3D::new(LineString3D::from(exterior), interiors)
                })
                .collect();
            Geometry3D::MultiPolygon(MultiPolygon3D::new(polygons))
        }
        _ => Geometry3D::Point(Point3D::from([0.0, 0.0, 0.0])),
    }
}

/// Convert 3D geometry to 2D by dropping Z coordinates
fn convert_3d_to_2d(geom: &Geometry3D) -> Geometry2D {
    match geom {
        Geometry3D::Point(pt) => Geometry2D::Point(Point2D::from([pt.x(), pt.y()])),
        Geometry3D::LineString(ls) => {
            let coords: Vec<(f64, f64)> = ls.coords().map(|c| (c.x, c.y)).collect();
            Geometry2D::LineString(LineString2D::from(coords))
        }
        Geometry3D::Polygon(poly) => {
            let exterior: Vec<(f64, f64)> = poly.exterior().coords().map(|c| (c.x, c.y)).collect();
            let interiors: Vec<LineString2D<f64>> = poly
                .interiors()
                .iter()
                .map(|ring| {
                    let coords: Vec<(f64, f64)> = ring.coords().map(|c| (c.x, c.y)).collect();
                    LineString2D::from(coords)
                })
                .collect();
            Geometry2D::Polygon(Polygon2D::new(LineString2D::from(exterior), interiors))
        }
        Geometry3D::MultiPoint(mp) => {
            let points: Vec<Point2D<f64>> =
                mp.iter().map(|pt| Point2D::from([pt.x(), pt.y()])).collect();
            Geometry2D::MultiPoint(MultiPoint2D::new(points))
        }
        Geometry3D::MultiLineString(mls) => {
            let lines: Vec<LineString2D<f64>> = mls
                .iter()
                .map(|ls| {
                    let coords: Vec<(f64, f64)> = ls.coords().map(|c| (c.x, c.y)).collect();
                    LineString2D::from(coords)
                })
                .collect();
            Geometry2D::MultiLineString(MultiLineString2D::new(lines))
        }
        Geometry3D::MultiPolygon(mpoly) => {
            let polygons: Vec<Polygon2D<f64>> = mpoly
                .iter()
                .map(|poly| {
                    let exterior: Vec<(f64, f64)> =
                        poly.exterior().coords().map(|c| (c.x, c.y)).collect();
                    let interiors: Vec<LineString2D<f64>> = poly
                        .interiors()
                        .iter()
                        .map(|ring| {
                            let coords: Vec<(f64, f64)> =
                                ring.coords().map(|c| (c.x, c.y)).collect();
                            LineString2D::from(coords)
                        })
                        .collect();
                    Polygon2D::new(LineString2D::from(exterior), interiors)
                })
                .collect();
            Geometry2D::MultiPolygon(MultiPolygon2D::new(polygons))
        }
        _ => Geometry2D::Point(Point2D::from([0.0, 0.0])),
    }
}
