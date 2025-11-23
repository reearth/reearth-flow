use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use byteorder::{LittleEndian, WriteBytesExt};
use bytes::Bytes;
use indexmap::IndexMap;
use reearth_flow_common::uri::Uri;
use reearth_flow_geometry::types::{
    geometry::{Geometry2D, Geometry3D},
    line_string::{LineString2D, LineString3D},
    multi_line_string::{MultiLineString2D, MultiLineString3D},
    multi_point::{MultiPoint2D, MultiPoint3D},
    multi_polygon::{MultiPolygon2D, MultiPolygon3D},
    point::{Point2D, Point3D},
    polygon::{Polygon2D, Polygon3D},
};
use reearth_flow_runtime::errors::BoxedError;
use reearth_flow_runtime::event::EventHub;
use reearth_flow_runtime::executor_operation::{ExecutorContext, NodeContext};
use reearth_flow_runtime::node::{Port, Sink, SinkFactory, DEFAULT_PORT};
use reearth_flow_sql::SqlAdapter;
use reearth_flow_types::{AttributeValue, Expr, Feature, Geometry, GeometryValue};
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
        let params = if let Some(with) = with {
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

        let sink = GeoPackageWriter {
            params,
            buffer: Default::default(),
            schema: Default::default(),
        };
        Ok(Box::new(sink))
    }
}

#[derive(Debug, Clone)]
pub(super) struct GeoPackageWriter {
    pub(super) params: GeoPackageWriterParam,
    pub(super) buffer: Vec<Feature>,
    pub(super) schema: IndexMap<String, AttributeType>,
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

#[derive(Debug, Clone, PartialEq)]
enum AttributeType {
    Integer,
    Real,
    Text,
    Blob,
    Boolean,
}

impl AttributeType {
    fn to_sql_type(&self) -> &'static str {
        match self {
            AttributeType::Integer => "INTEGER",
            AttributeType::Real => "REAL",
            AttributeType::Text => "TEXT",
            AttributeType::Blob => "BLOB",
            AttributeType::Boolean => "INTEGER",
        }
    }

    fn from_attribute_value(value: &AttributeValue) -> Self {
        match value {
            AttributeValue::Bool(_) => AttributeType::Boolean,
            AttributeValue::Number(n) => {
                if n.is_i64() {
                    AttributeType::Integer
                } else {
                    AttributeType::Real
                }
            }
            AttributeValue::String(_) => AttributeType::Text,
            AttributeValue::Array(_) | AttributeValue::Map(_) => AttributeType::Text,
            AttributeValue::Null => AttributeType::Text,
        }
    }
}

impl Sink for GeoPackageWriter {
    fn name(&self) -> &str {
        "GeoPackageWriter"
    }

    fn process(&mut self, ctx: ExecutorContext) -> Result<(), BoxedError> {
        let feature = &ctx.feature;

        // Infer schema from first feature
        if self.schema.is_empty() {
            for (key, value) in feature.attributes.iter() {
                if key != &self.params.geometry_column {
                    let attr_type = AttributeType::from_attribute_value(value);
                    self.schema.insert(key.clone(), attr_type);
                }
            }
        }

        self.buffer.push(feature.clone());
        Ok(())
    }

    fn finish(&self, ctx: NodeContext) -> Result<(), BoxedError> {
        if self.buffer.is_empty() {
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

impl GeoPackageWriter {
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
            let adapter = SqlAdapter::connect(&format!("sqlite://{db_path}"))
                .await
                .map_err(|e| {
                    SinkError::GeoPackageWriter(format!("Failed to connect to database: {e}"))
                })?;

            // Initialize GeoPackage structure
            self.init_geopackage_structure(&adapter).await?;

            // Create feature table
            self.create_feature_table(&adapter).await?;

            // Insert features
            self.insert_features(&adapter).await?;

            // Create spatial index if requested
            if self.params.create_spatial_index {
                self.create_spatial_index(&adapter).await?;
            }

            Ok::<(), BoxedError>(())
        })?;

        // Read the file content
        let content = std::fs::read(temp_file.path()).map_err(|e| {
            SinkError::GeoPackageWriter(format!("Failed to read temp file: {e}"))
        })?;

        Ok(content)
    }

    async fn init_geopackage_structure(&self, adapter: &SqlAdapter) -> Result<(), BoxedError> {
        // Set application_id for GeoPackage
        adapter
            .execute_raw("PRAGMA application_id = 0x47503130")
            .await
            .map_err(|e| {
                SinkError::GeoPackageWriter(format!("Failed to set application_id: {e}"))
            })?;

        // Create gpkg_spatial_ref_sys table
        adapter
            .execute_raw(
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
            .execute_raw(
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
            .execute_raw(
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
            .execute_raw(
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
            .execute_raw(
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
            .execute_raw(
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
            .execute_raw(
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
            adapter.execute_raw(&query).await.map_err(|e| {
                SinkError::GeoPackageWriter(format!("Failed to insert custom SRS: {e}"))
            })?;
        }

        Ok(())
    }

    async fn create_feature_table(&self, adapter: &SqlAdapter) -> Result<(), BoxedError> {
        // Build column definitions
        let mut columns = vec![
            "fid INTEGER PRIMARY KEY AUTOINCREMENT".to_string(),
            format!("{} BLOB", self.params.geometry_column),
        ];

        for (name, attr_type) in &self.schema {
            columns.push(format!("{} {}", name, attr_type.to_sql_type()));
        }

        let create_table = format!(
            "CREATE TABLE {} ({})",
            self.params.table_name,
            columns.join(", ")
        );

        adapter.execute_raw(&create_table).await.map_err(|e| {
            SinkError::GeoPackageWriter(format!("Failed to create feature table: {e}"))
        })?;

        // Calculate bounding box
        let (min_x, min_y, max_x, max_y) = self.calculate_bbox();

        // Insert into gpkg_contents
        let insert_contents = format!(
            r#"
            INSERT INTO gpkg_contents (table_name, data_type, identifier, description, srs_id, min_x, min_y, max_x, max_y)
            VALUES ('{}', 'features', '{}', '', {}, {}, {}, {}, {})
            "#,
            self.params.table_name,
            self.params.table_name,
            self.params.srs_id,
            min_x,
            min_y,
            max_x,
            max_y
        );

        adapter.execute_raw(&insert_contents).await.map_err(|e| {
            SinkError::GeoPackageWriter(format!("Failed to insert into gpkg_contents: {e}"))
        })?;

        // Detect Z dimension from first geometry
        let has_z = self
            .buffer
            .first()
            .and_then(|f| match &f.geometry.value {
                GeometryValue::FlowGeometry3D(_) => Some(true),
                _ => Some(false),
            })
            .unwrap_or(false);

        // Insert into gpkg_geometry_columns
        let insert_geom_cols = format!(
            r#"
            INSERT INTO gpkg_geometry_columns (table_name, column_name, geometry_type_name, srs_id, z, m)
            VALUES ('{}', '{}', '{}', {}, {}, 0)
            "#,
            self.params.table_name,
            self.params.geometry_column,
            self.params.geometry_type.to_uppercase(),
            self.params.srs_id,
            if has_z { 1 } else { 0 }
        );

        adapter.execute_raw(&insert_geom_cols).await.map_err(|e| {
            SinkError::GeoPackageWriter(format!("Failed to insert into gpkg_geometry_columns: {e}"))
        })?;

        Ok(())
    }

    async fn insert_features(&self, adapter: &SqlAdapter) -> Result<(), BoxedError> {
        for feature in &self.buffer {
            // Convert geometry to GeoPackage Binary
            let geom_blob = geometry_to_gpkg_wkb(&feature.geometry, self.params.srs_id)?;

            // Build column names and values
            let mut column_names = vec![self.params.geometry_column.clone()];
            let mut placeholders = vec!["?".to_string()];
            let mut values: Vec<Box<dyn sqlx::Encode<'_, sqlx::Any> + Send>> = vec![];

            // Add geometry
            values.push(Box::new(geom_blob));

            // Add attributes
            for (name, _) in &self.schema {
                if let Some(value) = feature.attributes.get(name) {
                    column_names.push(name.clone());
                    placeholders.push("?".to_string());
                    values.push(attribute_value_to_sql(value));
                }
            }

            let insert_query = format!(
                "INSERT INTO {} ({}) VALUES ({})",
                self.params.table_name,
                column_names.join(", "),
                placeholders.join(", ")
            );

            // For simplicity, we'll use raw SQL with string interpolation for values
            // In production, you'd want to use parameterized queries
            let insert_query_with_values = self.build_insert_query(feature, &geom_blob)?;
            adapter
                .execute_raw(&insert_query_with_values)
                .await
                .map_err(|e| {
                    SinkError::GeoPackageWriter(format!("Failed to insert feature: {e}"))
                })?;
        }

        Ok(())
    }

    fn build_insert_query(&self, feature: &Feature, geom_blob: &[u8]) -> Result<String, BoxedError> {
        let mut column_names = vec![self.params.geometry_column.clone()];
        let mut values = vec![format!("X'{}'", hex::encode(geom_blob))];

        for (name, _) in &self.schema {
            if let Some(value) = feature.attributes.get(name) {
                column_names.push(name.clone());
                values.push(attribute_value_to_sql_string(value)?);
            }
        }

        Ok(format!(
            "INSERT INTO {} ({}) VALUES ({})",
            self.params.table_name,
            column_names.join(", "),
            values.join(", ")
        ))
    }

    async fn create_spatial_index(&self, adapter: &SqlAdapter) -> Result<(), BoxedError> {
        let rtree_table = format!("rtree_{}_{}", self.params.table_name, self.params.geometry_column);

        // Create RTree spatial index
        let create_rtree = format!(
            r#"
            CREATE VIRTUAL TABLE {} USING rtree(
                id,
                minx, maxx,
                miny, maxy
            )
            "#,
            rtree_table
        );

        adapter.execute_raw(&create_rtree).await.map_err(|e| {
            SinkError::GeoPackageWriter(format!("Failed to create spatial index: {e}"))
        })?;

        // Populate RTree index
        for (idx, feature) in self.buffer.iter().enumerate() {
            let bbox = calculate_geometry_bbox(&feature.geometry);
            if let Some((min_x, min_y, max_x, max_y)) = bbox {
                let insert_rtree = format!(
                    "INSERT INTO {} VALUES ({}, {}, {}, {}, {})",
                    rtree_table,
                    idx + 1,
                    min_x,
                    max_x,
                    min_y,
                    max_y
                );
                adapter.execute_raw(&insert_rtree).await.map_err(|e| {
                    SinkError::GeoPackageWriter(format!("Failed to populate spatial index: {e}"))
                })?;
            }
        }

        // Register extension
        let register_ext = format!(
            r#"
            INSERT INTO gpkg_extensions (table_name, column_name, extension_name, definition, scope)
            VALUES ('{}', '{}', 'gpkg_rtree_index', 'http://www.geopackage.org/spec120/#extension_rtree', 'write-only')
            "#,
            self.params.table_name, self.params.geometry_column
        );

        adapter.execute_raw(&register_ext).await.map_err(|e| {
            SinkError::GeoPackageWriter(format!("Failed to register rtree extension: {e}"))
        })?;

        Ok(())
    }

    fn calculate_bbox(&self) -> (f64, f64, f64, f64) {
        let mut min_x = f64::INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut max_y = f64::NEG_INFINITY;

        for feature in &self.buffer {
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
            buffer
                .write_f64::<LittleEndian>(0.0)
                .map_err(|e| SinkError::GeoPackageWriter(format!("Failed to write envelope: {e}")))?;
            buffer
                .write_f64::<LittleEndian>(0.0)
                .map_err(|e| SinkError::GeoPackageWriter(format!("Failed to write envelope: {e}")))?;
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
            buffer
                .write_u32::<LittleEndian>(1)
                .map_err(|e| SinkError::GeoPackageWriter(format!("Failed to write WKB type: {e}")))?;
            buffer
                .write_f64::<LittleEndian>(pt.x())
                .map_err(|e| SinkError::GeoPackageWriter(format!("Failed to write coordinate: {e}")))?;
            buffer
                .write_f64::<LittleEndian>(pt.y())
                .map_err(|e| SinkError::GeoPackageWriter(format!("Failed to write coordinate: {e}")))?;
        }
        Geometry2D::LineString(ls) => {
            // WKB type for LineString (2)
            buffer
                .write_u32::<LittleEndian>(2)
                .map_err(|e| SinkError::GeoPackageWriter(format!("Failed to write WKB type: {e}")))?;
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
        }
        Geometry2D::Polygon(poly) => {
            // WKB type for Polygon (3)
            buffer
                .write_u32::<LittleEndian>(3)
                .map_err(|e| SinkError::GeoPackageWriter(format!("Failed to write WKB type: {e}")))?;
            // Number of rings
            buffer
                .write_u32::<LittleEndian>(1 + poly.interiors().len() as u32)
                .map_err(|e| SinkError::GeoPackageWriter(format!("Failed to write ring count: {e}")))?;
            // Exterior ring
            write_linestring_coords_2d(&mut buffer, poly.exterior())?;
            // Interior rings
            for interior in poly.interiors() {
                write_linestring_coords_2d(&mut buffer, interior)?;
            }
        }
        Geometry2D::MultiPoint(mp) => {
            // WKB type for MultiPoint (4)
            buffer
                .write_u32::<LittleEndian>(4)
                .map_err(|e| SinkError::GeoPackageWriter(format!("Failed to write WKB type: {e}")))?;
            buffer
                .write_u32::<LittleEndian>(mp.0.len() as u32)
                .map_err(|e| SinkError::GeoPackageWriter(format!("Failed to write point count: {e}")))?;
            for pt in mp.iter() {
                buffer.push(0x01); // byte order
                buffer
                    .write_u32::<LittleEndian>(1)
                    .map_err(|e| SinkError::GeoPackageWriter(format!("Failed to write WKB type: {e}")))?; // Point type
                buffer
                    .write_f64::<LittleEndian>(pt.x())
                    .map_err(|e| SinkError::GeoPackageWriter(format!("Failed to write coordinate: {e}")))?;
                buffer
                    .write_f64::<LittleEndian>(pt.y())
                    .map_err(|e| SinkError::GeoPackageWriter(format!("Failed to write coordinate: {e}")))?;
            }
        }
        Geometry2D::MultiLineString(mls) => {
            // WKB type for MultiLineString (5)
            buffer
                .write_u32::<LittleEndian>(5)
                .map_err(|e| SinkError::GeoPackageWriter(format!("Failed to write WKB type: {e}")))?;
            buffer
                .write_u32::<LittleEndian>(mls.0.len() as u32)
                .map_err(|e| SinkError::GeoPackageWriter(format!("Failed to write linestring count: {e}")))?;
            for ls in mls.iter() {
                buffer.push(0x01); // byte order
                buffer
                    .write_u32::<LittleEndian>(2)
                    .map_err(|e| SinkError::GeoPackageWriter(format!("Failed to write WKB type: {e}")))?; // LineString type
                write_linestring_coords_2d(&mut buffer, ls)?;
            }
        }
        Geometry2D::MultiPolygon(mpoly) => {
            // WKB type for MultiPolygon (6)
            buffer
                .write_u32::<LittleEndian>(6)
                .map_err(|e| SinkError::GeoPackageWriter(format!("Failed to write WKB type: {e}")))?;
            buffer
                .write_u32::<LittleEndian>(mpoly.0.len() as u32)
                .map_err(|e| SinkError::GeoPackageWriter(format!("Failed to write polygon count: {e}")))?;
            for poly in mpoly.iter() {
                buffer.push(0x01); // byte order
                buffer
                    .write_u32::<LittleEndian>(3)
                    .map_err(|e| SinkError::GeoPackageWriter(format!("Failed to write WKB type: {e}")))?; // Polygon type
                buffer
                    .write_u32::<LittleEndian>(1 + poly.interiors().len() as u32)
                    .map_err(|e| SinkError::GeoPackageWriter(format!("Failed to write ring count: {e}")))?;
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
            buffer
                .write_u32::<LittleEndian>(0x80000001)
                .map_err(|e| SinkError::GeoPackageWriter(format!("Failed to write WKB type: {e}")))?;
            buffer
                .write_f64::<LittleEndian>(pt.x())
                .map_err(|e| SinkError::GeoPackageWriter(format!("Failed to write coordinate: {e}")))?;
            buffer
                .write_f64::<LittleEndian>(pt.y())
                .map_err(|e| SinkError::GeoPackageWriter(format!("Failed to write coordinate: {e}")))?;
            buffer
                .write_f64::<LittleEndian>(pt.z())
                .map_err(|e| SinkError::GeoPackageWriter(format!("Failed to write coordinate: {e}")))?;
        }
        Geometry3D::LineString(ls) => {
            // WKB type for LineString Z (0x80000002)
            buffer
                .write_u32::<LittleEndian>(0x80000002)
                .map_err(|e| SinkError::GeoPackageWriter(format!("Failed to write WKB type: {e}")))?;
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
        }
        Geometry3D::Polygon(poly) => {
            // WKB type for Polygon Z (0x80000003)
            buffer
                .write_u32::<LittleEndian>(0x80000003)
                .map_err(|e| SinkError::GeoPackageWriter(format!("Failed to write WKB type: {e}")))?;
            buffer
                .write_u32::<LittleEndian>(1 + poly.interiors().len() as u32)
                .map_err(|e| SinkError::GeoPackageWriter(format!("Failed to write ring count: {e}")))?;
            write_linestring_coords_3d(&mut buffer, poly.exterior())?;
            for interior in poly.interiors() {
                write_linestring_coords_3d(&mut buffer, interior)?;
            }
        }
        Geometry3D::MultiPoint(mp) => {
            // WKB type for MultiPoint Z (0x80000004)
            buffer
                .write_u32::<LittleEndian>(0x80000004)
                .map_err(|e| SinkError::GeoPackageWriter(format!("Failed to write WKB type: {e}")))?;
            buffer
                .write_u32::<LittleEndian>(mp.0.len() as u32)
                .map_err(|e| SinkError::GeoPackageWriter(format!("Failed to write point count: {e}")))?;
            for pt in mp.iter() {
                buffer.push(0x01); // byte order
                buffer
                    .write_u32::<LittleEndian>(0x80000001)
                    .map_err(|e| SinkError::GeoPackageWriter(format!("Failed to write WKB type: {e}")))?; // Point Z type
                buffer
                    .write_f64::<LittleEndian>(pt.x())
                    .map_err(|e| SinkError::GeoPackageWriter(format!("Failed to write coordinate: {e}")))?;
                buffer
                    .write_f64::<LittleEndian>(pt.y())
                    .map_err(|e| SinkError::GeoPackageWriter(format!("Failed to write coordinate: {e}")))?;
                buffer
                    .write_f64::<LittleEndian>(pt.z())
                    .map_err(|e| SinkError::GeoPackageWriter(format!("Failed to write coordinate: {e}")))?;
            }
        }
        Geometry3D::MultiLineString(mls) => {
            // WKB type for MultiLineString Z (0x80000005)
            buffer
                .write_u32::<LittleEndian>(0x80000005)
                .map_err(|e| SinkError::GeoPackageWriter(format!("Failed to write WKB type: {e}")))?;
            buffer
                .write_u32::<LittleEndian>(mls.0.len() as u32)
                .map_err(|e| SinkError::GeoPackageWriter(format!("Failed to write linestring count: {e}")))?;
            for ls in mls.iter() {
                buffer.push(0x01); // byte order
                buffer
                    .write_u32::<LittleEndian>(0x80000002)
                    .map_err(|e| SinkError::GeoPackageWriter(format!("Failed to write WKB type: {e}")))?; // LineString Z type
                write_linestring_coords_3d(&mut buffer, ls)?;
            }
        }
        Geometry3D::MultiPolygon(mpoly) => {
            // WKB type for MultiPolygon Z (0x80000006)
            buffer
                .write_u32::<LittleEndian>(0x80000006)
                .map_err(|e| SinkError::GeoPackageWriter(format!("Failed to write WKB type: {e}")))?;
            buffer
                .write_u32::<LittleEndian>(mpoly.0.len() as u32)
                .map_err(|e| SinkError::GeoPackageWriter(format!("Failed to write polygon count: {e}")))?;
            for poly in mpoly.iter() {
                buffer.push(0x01); // byte order
                buffer
                    .write_u32::<LittleEndian>(0x80000003)
                    .map_err(|e| SinkError::GeoPackageWriter(format!("Failed to write WKB type: {e}")))?; // Polygon Z type
                buffer
                    .write_u32::<LittleEndian>(1 + poly.interiors().len() as u32)
                    .map_err(|e| SinkError::GeoPackageWriter(format!("Failed to write ring count: {e}")))?;
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
    ls: &LineString2D,
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
    ls: &LineString3D,
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

fn attribute_value_to_sql(
    _value: &AttributeValue,
) -> Box<dyn sqlx::Encode<'_, sqlx::Any> + Send> {
    // This is a placeholder - we'll use string interpolation instead
    Box::new(())
}

fn attribute_value_to_sql_string(value: &AttributeValue) -> Result<String, BoxedError> {
    match value {
        AttributeValue::Bool(b) => Ok(if *b { "1".to_string() } else { "0".to_string() }),
        AttributeValue::Number(n) => Ok(n.to_string()),
        AttributeValue::String(s) => Ok(format!("'{}'", s.replace('\'', "''"))),
        AttributeValue::Array(arr) => {
            let json = serde_json::to_string(arr)
                .map_err(|e| SinkError::GeoPackageWriter(format!("Failed to serialize array: {e}")))?;
            Ok(format!("'{}'", json.replace('\'', "''")))
        }
        AttributeValue::Map(map) => {
            let json = serde_json::to_string(map)
                .map_err(|e| SinkError::GeoPackageWriter(format!("Failed to serialize map: {e}")))?;
            Ok(format!("'{}'", json.replace('\'', "''")))
        }
        AttributeValue::Null => Ok("NULL".to_string()),
    }
}

