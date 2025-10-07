use std::collections::HashMap;
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;

use geozero::ToWkb;
use reearth_flow_common::uri::{Protocol, Uri};
use reearth_flow_runtime::errors::BoxedError;
use reearth_flow_runtime::event::EventHub;
use reearth_flow_runtime::executor_operation::{ExecutorContext, NodeContext};
use reearth_flow_runtime::node::{Port, Sink, SinkFactory, DEFAULT_PORT};
use reearth_flow_types::{Attribute, AttributeValue, Expr, Feature};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use sqlx::ConnectOptions;

use crate::errors::SinkError;

const RESERVED_COLUMN_FID: &str = "fid";
const RESERVED_COLUMN_GEOM: &str = "geom";

const GEOPACKAGE_MAGIC_GP: u8 = 0x47;
const GEOPACKAGE_MAGIC_P: u8 = 0x50;
const GEOPACKAGE_VERSION: u8 = 0x00;
const GEOPACKAGE_FLAGS_LITTLE_ENDIAN: u8 = 0x01;

#[derive(Debug, Clone, Default)]
pub(crate) struct GeoPackageWriterFactory;

impl SinkFactory for GeoPackageWriterFactory {
    fn name(&self) -> &str {
        "GeoPackageWriter"
    }

    fn description(&self) -> &str {
        "Writes geographic features to GeoPackage files with optional grouping"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(GeoPackageWriterParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["File"]
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
        };
        Ok(Box::new(sink))
    }
}

#[derive(Debug, Clone)]
pub(super) struct GeoPackageWriter {
    pub(super) params: GeoPackageWriterParam,
    pub(super) buffer: HashMap<String, HashMap<String, Vec<Feature>>>,
}

/// # GeoPackageWriter Parameters
///
/// Configuration for writing features to GeoPackage files.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(super) struct GeoPackageWriterParam {
    /// # Output
    /// Output path or expression for the GeoPackage file to create
    pub(super) output: Expr,
    /// # Layer Name
    /// Name of the layer/table to create in the GeoPackage (default: "features")
    #[serde(default = "default_layer_name")]
    pub(super) layer_name: String,
    /// # SRID
    /// Spatial Reference System ID for geometries (default: 4326 for WGS84)
    #[serde(default = "default_srid")]
    pub(super) srid: i32,
    /// # Group By
    /// Optional attributes to group features by, creating separate files for each group
    pub(super) group_by: Option<Vec<Attribute>>,
    /// # Write Mode
    /// Controls how features are organized in the output
    #[serde(default)]
    pub(super) write_mode: GeoPackageWriteMode,
    /// # Force 2D
    /// Force 2D output by discarding Z coordinates from geometries
    #[serde(default, rename = "force2D")]
    pub(super) force_2d: bool,
    /// # Overwrite
    /// Overwrite existing file if it exists (default: true)
    #[serde(default = "default_true")]
    pub(super) overwrite: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema, Default)]
#[serde(rename_all = "camelCase")]
pub(super) enum GeoPackageWriteMode {
    /// # Single Layer
    /// Write all features to a single layer
    #[default]
    SingleLayer,
    /// # Multiple Layers
    /// Create separate layers based on _geopackage_layer attribute
    MultipleLayers,
    /// # Separate Files
    /// Create separate files for each group (using groupBy parameter)
    SeparateFiles,
}

fn default_layer_name() -> String {
    "features".to_string()
}

fn default_srid() -> i32 {
    4326
}

fn default_true() -> bool {
    true
}

impl Sink for GeoPackageWriter {
    fn name(&self) -> &str {
        "GeoPackageWriter"
    }

    fn process(&mut self, ctx: ExecutorContext) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let expr_engine = Arc::clone(&ctx.expr_engine);
        let scope = expr_engine.new_scope();
        let output = &self.params.output;
        let path = scope
            .eval::<String>(output.as_ref())
            .unwrap_or_else(|_| output.as_ref().to_string());

        let file_path = match self.params.write_mode {
            GeoPackageWriteMode::SeparateFiles => {
                if let Some(group_by) = &self.params.group_by {
                    if group_by.is_empty() {
                        path.clone()
                    } else {
                        let group_values: Vec<String> = group_by
                            .iter()
                            .map(|k| {
                                feature
                                    .get(k)
                                    .map(|v| v.to_string())
                                    .unwrap_or_else(|| "null".to_string())
                            })
                            .collect();
                        let group_key = group_values.join("_");
                        let base_path = Path::new(&path);
                        let stem = base_path
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("output");
                        let parent = base_path.parent().unwrap_or(Path::new(""));
                        parent
                            .join(format!("{stem}_{group_key}.gpkg"))
                            .to_string_lossy()
                            .to_string()
                    }
                } else {
                    path.clone()
                }
            }
            _ => path.clone(),
        };

        let layer_name = match self.params.write_mode {
            GeoPackageWriteMode::MultipleLayers => feature
                .get(&Attribute::new("_geopackage_layer"))
                .and_then(|v| match v {
                    AttributeValue::String(s) => Some(s.clone()),
                    _ => None,
                })
                .unwrap_or_else(|| self.params.layer_name.clone()),
            _ => self.params.layer_name.clone(),
        };

        self.buffer
            .entry(file_path)
            .or_default()
            .entry(layer_name)
            .or_default()
            .push(feature.clone());
        Ok(())
    }

    fn finish(&self, _ctx: NodeContext) -> Result<(), BoxedError> {
        for (path_str, layers) in self.buffer.iter() {
            let uri = Uri::from_str(path_str.as_str())?;

            if uri.protocol() == Protocol::File {
                let local_path = uri.path();

                if let Some(parent) = local_path.parent() {
                    std::fs::create_dir_all(parent).map_err(|e| {
                        SinkError::GeoPackageWriter(format!(
                            "Failed to create parent directory: {e}"
                        ))
                    })?;
                }

                if self.params.overwrite && local_path.exists() {
                    std::fs::remove_file(&local_path).map_err(|e| {
                        SinkError::GeoPackageWriter(format!("Failed to remove existing file: {e}"))
                    })?;
                }

                let rt = tokio::runtime::Runtime::new().map_err(|e| {
                    SinkError::GeoPackageWriter(format!("Failed to create tokio runtime: {e}"))
                })?;

                rt.block_on(async {
                    write_geopackage_with_layers(
                        &local_path,
                        layers,
                        self.params.srid,
                        self.params.force_2d,
                    )
                    .await
                })?;
            } else {
                return Err(SinkError::GeoPackageWriter(
                    "GeoPackage writer currently only supports local file paths".to_string(),
                )
                .into());
            }
        }
        Ok(())
    }
}

async fn write_geopackage_with_layers(
    path: &Path,
    layers: &HashMap<String, Vec<Feature>>,
    srid: i32,
    force_2d: bool,
) -> Result<(), BoxedError> {
    let connection_string = format!("sqlite://{}", path.display());
    let options = SqliteConnectOptions::from_str(&connection_string)?
        .create_if_missing(true)
        .disable_statement_logging();

    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
        .map_err(|e| SinkError::GeoPackageWriter(format!("Failed to connect to database: {e}")))?;

    initialize_geopackage(&pool).await?;

    for (layer_name, features) in layers.iter() {
        if features.is_empty() {
            continue;
        }

        let schema = analyze_feature_schema(features);
        let bbox = calculate_bbox(features);

        create_feature_table(&pool, layer_name, &schema, srid).await?;
        register_layer_in_contents(&pool, layer_name, srid, bbox).await?;
        insert_features(&pool, layer_name, features, &schema, srid, force_2d).await?;
    }

    pool.close().await;

    Ok(())
}

async fn initialize_geopackage(pool: &SqlitePool) -> Result<(), BoxedError> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS gpkg_contents (
            table_name TEXT NOT NULL PRIMARY KEY,
            data_type TEXT NOT NULL,
            identifier TEXT UNIQUE,
            description TEXT DEFAULT '',
            last_change DATETIME NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
            min_x DOUBLE,
            min_y DOUBLE,
            max_x DOUBLE,
            max_y DOUBLE,
            srs_id INTEGER
        )",
    )
    .execute(pool)
    .await
    .map_err(|e| SinkError::GeoPackageWriter(format!("Failed to create gpkg_contents: {e}")))?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS gpkg_geometry_columns (
            table_name TEXT NOT NULL,
            column_name TEXT NOT NULL,
            geometry_type_name TEXT NOT NULL,
            srs_id INTEGER NOT NULL,
            z TINYINT NOT NULL,
            m TINYINT NOT NULL,
            CONSTRAINT pk_geom_cols PRIMARY KEY (table_name, column_name)
        )",
    )
    .execute(pool)
    .await
    .map_err(|e| {
        SinkError::GeoPackageWriter(format!("Failed to create gpkg_geometry_columns: {e}"))
    })?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS gpkg_spatial_ref_sys (
            srs_name TEXT NOT NULL,
            srs_id INTEGER NOT NULL PRIMARY KEY,
            organization TEXT NOT NULL,
            organization_coordsys_id INTEGER NOT NULL,
            definition TEXT NOT NULL,
            description TEXT
        )",
    )
    .execute(pool)
    .await
    .map_err(|e| {
        SinkError::GeoPackageWriter(format!("Failed to create gpkg_spatial_ref_sys: {e}"))
    })?;

    sqlx::query(
        "INSERT OR IGNORE INTO gpkg_spatial_ref_sys
         (srs_name, srs_id, organization, organization_coordsys_id, definition, description)
         VALUES
         ('WGS 84', 4326, 'EPSG', 4326,
          'GEOGCS[\"WGS 84\",DATUM[\"WGS_1984\",SPHEROID[\"WGS 84\",6378137,298.257223563,AUTHORITY[\"EPSG\",\"7030\"]],AUTHORITY[\"EPSG\",\"6326\"]],PRIMEM[\"Greenwich\",0,AUTHORITY[\"EPSG\",\"8901\"]],UNIT[\"degree\",0.0174532925199433,AUTHORITY[\"EPSG\",\"9122\"]],AUTHORITY[\"EPSG\",\"4326\"]]',
          'WGS 84 geographic 2D')",
    )
    .execute(pool)
    .await
    .map_err(|e| SinkError::GeoPackageWriter(format!("Failed to insert WGS84 SRS: {e}")))?;

    Ok(())
}

#[derive(Debug)]
struct ColumnDef {
    name: String,
    sql_type: String,
}

fn analyze_feature_schema(features: &[Feature]) -> Vec<ColumnDef> {
    if features.is_empty() {
        return vec![];
    }

    let mut columns = HashMap::new();

    for feature in features {
        for (key, value) in feature.attributes.iter() {
            let sql_type = match value {
                AttributeValue::Number(_) => "REAL",
                AttributeValue::Bool(_) => "INTEGER",
                AttributeValue::String(_) => "TEXT",
                _ => "TEXT",
            };

            columns
                .entry(key.to_string())
                .or_insert_with(|| sql_type.to_string());
        }
    }

    let mut column_defs: Vec<ColumnDef> = columns
        .into_iter()
        .map(|(name, sql_type)| ColumnDef { name, sql_type })
        .collect();

    column_defs.sort_by(|a, b| a.name.cmp(&b.name));
    column_defs
}

fn calculate_bbox(features: &[Feature]) -> Option<(f64, f64, f64, f64)> {
    let mut min_x = f64::MAX;
    let mut min_y = f64::MAX;
    let mut max_x = f64::MIN;
    let mut max_y = f64::MIN;
    let mut has_geometry = false;

    for feature in features {
        let geojson_features: Vec<geojson::Feature> = match feature.clone().try_into() {
            Ok(f) => f,
            Err(_) => continue,
        };

        for geojson_feature in geojson_features {
            if let Some(geom) = &geojson_feature.geometry {
                if let Some((x_min, y_min, x_max, y_max)) =
                    extract_bbox_from_geojson_value(&geom.value)
                {
                    has_geometry = true;
                    min_x = min_x.min(x_min);
                    min_y = min_y.min(y_min);
                    max_x = max_x.max(x_max);
                    max_y = max_y.max(y_max);
                }
            }
        }
    }

    if has_geometry {
        Some((min_x, min_y, max_x, max_y))
    } else {
        None
    }
}

fn has_z_coordinates(value: &geojson::Value) -> bool {
    match value {
        geojson::Value::Point(coords) => coords.len() >= 3,
        geojson::Value::MultiPoint(points) => points.iter().any(|coords| coords.len() >= 3),
        geojson::Value::LineString(coords) => coords.iter().any(|coord| coord.len() >= 3),
        geojson::Value::MultiLineString(lines) => lines
            .iter()
            .any(|line| line.iter().any(|coord| coord.len() >= 3)),
        geojson::Value::Polygon(rings) => rings
            .iter()
            .any(|ring| ring.iter().any(|coord| coord.len() >= 3)),
        geojson::Value::MultiPolygon(polygons) => polygons.iter().any(|polygon| {
            polygon
                .iter()
                .any(|ring| ring.iter().any(|coord| coord.len() >= 3))
        }),
        geojson::Value::GeometryCollection(geometries) => {
            geometries.iter().any(|geom| has_z_coordinates(&geom.value))
        }
    }
}

fn extract_bbox_from_geojson_value(value: &geojson::Value) -> Option<(f64, f64, f64, f64)> {
    match value {
        geojson::Value::Point(coords) => {
            if coords.len() >= 2 {
                Some((coords[0], coords[1], coords[0], coords[1]))
            } else {
                None
            }
        }
        geojson::Value::MultiPoint(points) => {
            let mut min_x = f64::MAX;
            let mut min_y = f64::MAX;
            let mut max_x = f64::MIN;
            let mut max_y = f64::MIN;
            for coords in points {
                if coords.len() >= 2 {
                    min_x = min_x.min(coords[0]);
                    min_y = min_y.min(coords[1]);
                    max_x = max_x.max(coords[0]);
                    max_y = max_y.max(coords[1]);
                }
            }
            if min_x != f64::MAX {
                Some((min_x, min_y, max_x, max_y))
            } else {
                None
            }
        }
        geojson::Value::LineString(coords) => {
            let mut min_x = f64::MAX;
            let mut min_y = f64::MAX;
            let mut max_x = f64::MIN;
            let mut max_y = f64::MIN;
            for coord in coords {
                if coord.len() >= 2 {
                    min_x = min_x.min(coord[0]);
                    min_y = min_y.min(coord[1]);
                    max_x = max_x.max(coord[0]);
                    max_y = max_y.max(coord[1]);
                }
            }
            if min_x != f64::MAX {
                Some((min_x, min_y, max_x, max_y))
            } else {
                None
            }
        }
        geojson::Value::MultiLineString(lines) => {
            let mut min_x = f64::MAX;
            let mut min_y = f64::MAX;
            let mut max_x = f64::MIN;
            let mut max_y = f64::MIN;
            for line in lines {
                for coord in line {
                    if coord.len() >= 2 {
                        min_x = min_x.min(coord[0]);
                        min_y = min_y.min(coord[1]);
                        max_x = max_x.max(coord[0]);
                        max_y = max_y.max(coord[1]);
                    }
                }
            }
            if min_x != f64::MAX {
                Some((min_x, min_y, max_x, max_y))
            } else {
                None
            }
        }
        geojson::Value::Polygon(rings) => {
            let mut min_x = f64::MAX;
            let mut min_y = f64::MAX;
            let mut max_x = f64::MIN;
            let mut max_y = f64::MIN;
            for ring in rings {
                for coord in ring {
                    if coord.len() >= 2 {
                        min_x = min_x.min(coord[0]);
                        min_y = min_y.min(coord[1]);
                        max_x = max_x.max(coord[0]);
                        max_y = max_y.max(coord[1]);
                    }
                }
            }
            if min_x != f64::MAX {
                Some((min_x, min_y, max_x, max_y))
            } else {
                None
            }
        }
        geojson::Value::MultiPolygon(polygons) => {
            let mut min_x = f64::MAX;
            let mut min_y = f64::MAX;
            let mut max_x = f64::MIN;
            let mut max_y = f64::MIN;
            for polygon in polygons {
                for ring in polygon {
                    for coord in ring {
                        if coord.len() >= 2 {
                            min_x = min_x.min(coord[0]);
                            min_y = min_y.min(coord[1]);
                            max_x = max_x.max(coord[0]);
                            max_y = max_y.max(coord[1]);
                        }
                    }
                }
            }
            if min_x != f64::MAX {
                Some((min_x, min_y, max_x, max_y))
            } else {
                None
            }
        }
        geojson::Value::GeometryCollection(geometries) => {
            let mut min_x = f64::MAX;
            let mut min_y = f64::MAX;
            let mut max_x = f64::MIN;
            let mut max_y = f64::MIN;
            for geom in geometries {
                if let Some((x_min, y_min, x_max, y_max)) =
                    extract_bbox_from_geojson_value(&geom.value)
                {
                    min_x = min_x.min(x_min);
                    min_y = min_y.min(y_min);
                    max_x = max_x.max(x_max);
                    max_y = max_y.max(y_max);
                }
            }
            if min_x != f64::MAX {
                Some((min_x, min_y, max_x, max_y))
            } else {
                None
            }
        }
    }
}

async fn create_feature_table(
    pool: &SqlitePool,
    table_name: &str,
    schema: &[ColumnDef],
    srid: i32,
) -> Result<(), BoxedError> {
    let mut columns = vec!["fid INTEGER PRIMARY KEY AUTOINCREMENT".to_string()];
    columns.push("geom BLOB".to_string());

    for col in schema {
        let col_name_lower = col.name.to_lowercase();
        if col_name_lower == RESERVED_COLUMN_FID || col_name_lower == RESERVED_COLUMN_GEOM {
            continue;
        }
        columns.push(format!("{} {}", col.name, col.sql_type));
    }

    let create_sql = format!("CREATE TABLE {} ({})", table_name, columns.join(", "));

    sqlx::query(&create_sql)
        .execute(pool)
        .await
        .map_err(|e| SinkError::GeoPackageWriter(format!("Failed to create table: {e}")))?;

    sqlx::query(
        "INSERT INTO gpkg_geometry_columns
         (table_name, column_name, geometry_type_name, srs_id, z, m)
         VALUES (?, 'geom', 'GEOMETRY', ?, 0, 0)",
    )
    .bind(table_name)
    .bind(srid)
    .execute(pool)
    .await
    .map_err(|e| {
        SinkError::GeoPackageWriter(format!("Failed to register in gpkg_geometry_columns: {e}"))
    })?;

    Ok(())
}

async fn register_layer_in_contents(
    pool: &SqlitePool,
    table_name: &str,
    srid: i32,
    bbox: Option<(f64, f64, f64, f64)>,
) -> Result<(), BoxedError> {
    if let Some((min_x, min_y, max_x, max_y)) = bbox {
        sqlx::query(
            "INSERT INTO gpkg_contents
             (table_name, data_type, identifier, srs_id, min_x, min_y, max_x, max_y)
             VALUES (?, 'features', ?, ?, ?, ?, ?, ?)",
        )
        .bind(table_name)
        .bind(table_name)
        .bind(srid)
        .bind(min_x)
        .bind(min_y)
        .bind(max_x)
        .bind(max_y)
        .execute(pool)
        .await
        .map_err(|e| {
            SinkError::GeoPackageWriter(format!("Failed to register in gpkg_contents: {e}"))
        })?;
    } else {
        sqlx::query(
            "INSERT INTO gpkg_contents
             (table_name, data_type, identifier, srs_id)
             VALUES (?, 'features', ?, ?)",
        )
        .bind(table_name)
        .bind(table_name)
        .bind(srid)
        .execute(pool)
        .await
        .map_err(|e| {
            SinkError::GeoPackageWriter(format!("Failed to register in gpkg_contents: {e}"))
        })?;
    }

    Ok(())
}

async fn insert_features(
    pool: &SqlitePool,
    table_name: &str,
    features: &[Feature],
    schema: &[ColumnDef],
    srid: i32,
    force_2d: bool,
) -> Result<(), BoxedError> {
    for feature in features {
        let geojson_features: Vec<geojson::Feature> = match feature.clone().try_into() {
            Ok(f) => f,
            Err(e) => {
                return Err(SinkError::GeoPackageWriter(format!(
                    "Failed to convert feature to geojson: {e}"
                ))
                .into())
            }
        };

        let wkb = if let Some(geojson_feature) = geojson_features.first() {
            if let Some(geom) = &geojson_feature.geometry {
                let geo_geom: geo_types::Geometry<f64> = match geom.clone().value.try_into() {
                    Ok(g) => g,
                    Err(e) => {
                        return Err(SinkError::GeoPackageWriter(format!(
                            "Failed to convert geojson to geo_types: {e}"
                        ))
                        .into())
                    }
                };

                let coord_dims = if force_2d {
                    geozero::CoordDimensions::xy()
                } else if has_z_coordinates(&geom.value) {
                    geozero::CoordDimensions::xyz()
                } else {
                    geozero::CoordDimensions::xy()
                };
                let wkb_bytes = geo_geom.to_wkb(coord_dims).map_err(|e| {
                    SinkError::GeoPackageWriter(format!("Failed to convert to WKB: {e}"))
                })?;

                Some(create_gpkg_wkb_header(srid, &wkb_bytes))
            } else {
                None
            }
        } else {
            None
        };

        let mut columns = vec!["geom".to_string()];
        let mut placeholders = vec!["?".to_string()];

        for col in schema {
            columns.push(col.name.clone());
            placeholders.push("?".to_string());
        }

        let insert_sql = format!(
            "INSERT INTO {} ({}) VALUES ({})",
            table_name,
            columns.join(", "),
            placeholders.join(", ")
        );

        let mut query = sqlx::query(&insert_sql);
        query = query.bind(wkb);

        for col in schema {
            let attr = Attribute::new(&col.name);
            let value = feature
                .attributes
                .get(&attr)
                .cloned()
                .unwrap_or(AttributeValue::Null);

            query = match value {
                AttributeValue::Number(n) => query.bind(n.as_f64()),
                AttributeValue::Bool(b) => query.bind(b as i32),
                AttributeValue::String(s) => query.bind(s),
                AttributeValue::Null => query.bind(None::<String>),
                _ => query.bind(value.to_string()),
            };
        }

        query
            .execute(pool)
            .await
            .map_err(|e| SinkError::GeoPackageWriter(format!("Failed to insert feature: {e}")))?;
    }

    Ok(())
}

fn create_gpkg_wkb_header(srid: i32, wkb: &[u8]) -> Vec<u8> {
    let mut result = vec![
        GEOPACKAGE_MAGIC_GP,
        GEOPACKAGE_MAGIC_P,
        GEOPACKAGE_VERSION,
        GEOPACKAGE_FLAGS_LITTLE_ENDIAN,
    ];

    result.extend_from_slice(&srid.to_le_bytes());
    result.extend_from_slice(wkb);

    result
}
