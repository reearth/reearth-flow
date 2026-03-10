use std::collections::HashMap;
use std::io::Write;
use std::str::FromStr;
use std::sync::Arc;
use std::vec;

use bytes::Bytes;
use indexmap::IndexMap;
use itertools::Itertools;
use nusamai_czml::{
    CzmlBoolean, CzmlPolygon, Packet, PositionList, PositionListOfLists,
    PositionListOfListsProperties, PositionListProperties, StringProperties, StringValueType,
};
use rayon::iter::{ParallelBridge, ParallelIterator};
use reearth_flow_common::str::to_hash;
use reearth_flow_common::uri::Uri;
use reearth_flow_geometry::types::geometry::{Geometry2D, Geometry3D};
use reearth_flow_geometry::types::polygon::Polygon3D;
use reearth_flow_runtime::errors::BoxedError;
use reearth_flow_runtime::event::EventHub;
use reearth_flow_runtime::executor_operation::{Context, ExecutorContext, NodeContext};
use reearth_flow_runtime::node::{Port, Sink, SinkFactory, DEFAULT_PORT};
use reearth_flow_types::{Attribute, AttributeValue, Expr, Feature, GeometryValue};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::errors::SinkError;

#[derive(Debug, Clone, Default)]
pub(crate) struct CzmlWriterFactory;

impl SinkFactory for CzmlWriterFactory {
    fn name(&self) -> &str {
        "CzmlWriter"
    }

    fn description(&self) -> &str {
        "Export features as CZML for Cesium visualization. Supports static entities and time-animated timeseries. Configure timeField, groupTimeseriesBy, and epoch (for numeric times) to enable animation."
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(CzmlWriterParam))
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
        let mut params: CzmlWriterParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                SinkError::CzmlWriterFactory(format!("Failed to serialize `with` parameter: {e}"))
            })?;
            serde_json::from_value(value).map_err(|e| {
                SinkError::CzmlWriterFactory(format!("Failed to deserialize `with` parameter: {e}"))
            })?
        } else {
            return Err(SinkError::CzmlWriterFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        params.sanitize();

        let sink = CzmlWriter {
            params,
            buffer: Default::default(),
        };
        Ok(Box::new(sink))
    }
}

#[derive(Debug, Clone)]
pub(crate) struct CzmlWriter {
    pub(super) params: CzmlWriterParam,
    pub(super) buffer: HashMap<AttributeValue, Vec<Feature>>,
}

/// # CzmlWriter Parameters
///
/// Configuration for writing geographic features to CZML files. Supports both
/// static entities and time-dynamic entities with interpolated position samples.
///
/// ## Timeseries Configuration
///
/// To create time-animated entities in Cesium, configure at least the first two
/// parameters below; configure `epoch` only when using numeric time offsets:
/// 1. `timeField` - Attribute containing time values
/// 2. `groupTimeseriesBy` - Attribute to group features into entities
/// 3. `epoch` (optional) - Base time used when `timeField` contains numeric offsets
///
/// ### Example with ISO 8601 timestamps:
/// ```yaml
/// - action: CzmlWriter
///   with:
///     output: "vehicles.czml"
///     timeField: "timestamp"           # Contains "2024-01-01T00:00:00Z", etc.
///     groupTimeseriesBy: "vehicleId"   # Groups by vehicle ID
///     interpolationAlgorithm: "LAGRANGE"
///     interpolationDegree: 5
/// ```
///
/// ### Example with numeric time offsets:
/// ```yaml
/// - action: CzmlWriter
///   with:
///     output: "sensors.czml"
///     timeField: "timeOffset"          # Contains numeric values: 0, 60, 120, etc.
///     groupTimeseriesBy: "sensorId"
///     epoch: "2024-01-01T00:00:00Z"    # Base time for offsets
///     interpolationAlgorithm: "LINEAR"
/// ```
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CzmlWriterParam {
    /// # Output File Path
    /// Path where the CZML file will be written
    pub(super) output: Expr,
    /// # Group By Attributes
    /// Attributes used to group features into separate CZML files
    pub(super) group_by: Option<Vec<Attribute>>,
    /// # Time Field
    /// Attribute containing the timestamp for each feature. Supports two formats:
    /// - **ISO 8601 strings**: e.g., "2024-01-01T00:00:00Z", "2024-01-01T12:30:45+09:00"
    /// - **Numeric values**: Seconds as offset from epoch (e.g., "0", "60", "120.5")
    ///
    /// When set together with `groupTimeseriesBy`, features sharing the same
    /// group key are combined into a single CZML entity with time-tagged position
    /// samples for animation in Cesium.
    ///
    /// **Example workflow configuration:**
    /// ```yaml
    /// - action: CzmlWriter
    ///   with:
    ///     output: "output.czml"
    ///     timeField: "timestamp"
    ///     groupTimeseriesBy: "vehicleId"
    ///     epoch: "2024-01-01T00:00:00Z"  # Optional for numeric times (auto-defaults to Unix epoch)
    /// ```
    pub(super) time_field: Option<Attribute>,
    /// # Epoch
    /// Reference time (ISO 8601 format) used as the base for numeric time offsets.
    ///
    /// **When to use:**
    /// - Optional but recommended when `timeField` contains numeric values (e.g., "0", "60", "3600")
    /// - Not needed when `timeField` contains ISO 8601 datetime strings
    ///
    /// **Format:** ISO 8601 datetime string with timezone
    /// - Examples: "2024-01-01T00:00:00Z", "2024-06-15T09:00:00+09:00"
    ///
    /// **Auto-detection:** If omitted and all time values are numeric, automatically
    /// defaults to Unix epoch "1970-01-01T00:00:00Z". For custom time ranges,
    /// explicitly set this parameter to your desired base time.
    ///
    /// **Example:**
    /// ```yaml
    /// epoch: "2024-01-01T00:00:00Z"  # Time value "60" means 2024-01-01T00:01:00Z
    /// ```
    pub(super) epoch: Option<String>,
    /// # Interpolation Algorithm
    /// Algorithm used by Cesium to interpolate between time-tagged samples.
    #[serde(default)]
    pub(super) interpolation_algorithm: InterpolationAlgorithm,
    /// # Interpolation Degree
    /// Degree of interpolation (1 for LINEAR, 5 typical for LAGRANGE).
    #[serde(default = "default_interpolation_degree")]
    pub(super) interpolation_degree: u32,
    /// # Group Timeseries By
    /// Attribute used to group features into a single time-dynamic CZML
    /// entity. Features with the same value for this attribute are merged
    /// into one packet with time-tagged positions.
    pub(super) group_timeseries_by: Option<Attribute>,
    /// # Color Attribute
    /// Attribute containing a hex color string (e.g., "#ffd8c0") for polygon fill.
    /// Used when polygon geometry is auto-converted from the feature geometry.
    pub(super) color_attribute: Option<Attribute>,
    /// # Opacity
    /// Alpha value (0–255) for polygon fill color. Default: 180.
    #[serde(default = "default_opacity")]
    pub(super) opacity: u8,
    /// # Height Attribute
    /// Attribute containing a numeric value for polygon extrusion height.
    /// When set, polygons are extruded from ground to this height value.
    pub(super) height_attribute: Option<Attribute>,
}

fn default_interpolation_degree() -> u32 {
    1
}

fn default_opacity() -> u8 {
    180
}

/// Strip common expression wrappers from an attribute name.
///
/// The UI may wrap plain attribute names in expression syntax like
/// `env.get("__value").field_name`. This extracts the bare attribute name.
fn sanitize_attribute(attr: &Attribute) -> Attribute {
    let s = attr.inner();
    // env.get("__value").field_name → field_name
    if let Some(rest) = s.strip_prefix("env.get(\"__value\").") {
        return Attribute::new(rest);
    }
    Attribute::new(s)
}

/// Strip surrounding literal quote characters from a string.
///
/// The UI may double-quote epoch values, e.g. `"\"2024-01-01T00:00:00Z\""`.
/// After JSON deserialization this becomes `"2024-01-01T00:00:00Z"` (with
/// literal `"` chars). This helper removes them.
fn sanitize_epoch(epoch: &str) -> String {
    let s = epoch.trim();
    if s.starts_with('"') && s.ends_with('"') && s.len() >= 2 {
        s[1..s.len() - 1].to_string()
    } else {
        s.to_string()
    }
}

impl CzmlWriterParam {
    /// Normalize fields that may contain expression syntax from the UI.
    fn sanitize(&mut self) {
        if let Some(ref attr) = self.time_field {
            self.time_field = Some(sanitize_attribute(attr));
        }
        if let Some(ref attr) = self.group_timeseries_by {
            self.group_timeseries_by = Some(sanitize_attribute(attr));
        }
        if let Some(ref attr) = self.color_attribute {
            self.color_attribute = Some(sanitize_attribute(attr));
        }
        if let Some(ref attr) = self.height_attribute {
            self.height_attribute = Some(sanitize_attribute(attr));
        }
        if let Some(ref epoch) = self.epoch {
            self.epoch = Some(sanitize_epoch(epoch));
        }
    }
}

/// Interpolation algorithm for Cesium time-dynamic properties.
#[derive(Serialize, Deserialize, Debug, Clone, Default, JsonSchema)]
#[serde(rename_all = "UPPERCASE")]
pub(super) enum InterpolationAlgorithm {
    /// Linear interpolation between samples (degree 1).
    #[default]
    Linear,
    /// Lagrange polynomial interpolation for smooth curves (typical degree 5).
    Lagrange,
    /// Hermite spline interpolation using tangent data.
    Hermite,
}

impl std::fmt::Display for InterpolationAlgorithm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Linear => write!(f, "LINEAR"),
            Self::Lagrange => write!(f, "LAGRANGE"),
            Self::Hermite => write!(f, "HERMITE"),
        }
    }
}

impl Sink for CzmlWriter {
    fn name(&self) -> &str {
        "CzmlWriter"
    }

    fn process(&mut self, ctx: ExecutorContext) -> Result<(), BoxedError> {
        let feature = &ctx.feature;

        let key = if let Some(group_by) = &self.params.group_by {
            if group_by.is_empty() {
                AttributeValue::Null
            } else {
                let key = group_by
                    .iter()
                    .map(|k| feature.get(k).cloned().unwrap_or(AttributeValue::Null))
                    .collect::<Vec<_>>();
                AttributeValue::Array(key)
            }
        } else {
            AttributeValue::Null
        };
        self.buffer.entry(key).or_default().push(feature.clone());
        Ok(())
    }
    fn finish(&self, ctx: NodeContext) -> Result<(), BoxedError> {
        let storage_resolver = Arc::clone(&ctx.storage_resolver);
        let expr_engine = Arc::clone(&ctx.expr_engine);
        let output = self.params.output.clone();
        let scope = expr_engine.new_scope();
        let path = scope
            .eval::<String>(output.as_ref())
            .unwrap_or_else(|_| output.as_ref().to_string());
        let output = Uri::from_str(path.as_str())?;

        for (key, features) in self.buffer.iter() {
            let file_path = if *key == AttributeValue::Null {
                output.clone()
            } else {
                output.join(format!("{}.json", to_hash(key.to_string().as_str())))?
            };
            let storage = storage_resolver
                .resolve(&file_path)
                .map_err(crate::errors::SinkError::czml_writer)?;

            let is_grouped_timeseries =
                self.params.group_timeseries_by.is_some() && self.params.time_field.is_some();
            let has_citygml = features
                .iter()
                .any(|f| matches!(&f.geometry.value, GeometryValue::CityGmlGeometry(_)));

            if is_grouped_timeseries {
                let buffer = build_timeseries_czml(features, &self.params, &ctx)?;
                storage
                    .put_sync(file_path.path().as_path(), Bytes::from(buffer))
                    .map_err(crate::errors::SinkError::czml_writer)?;
            } else if has_citygml {
                let (sender, receiver) = std::sync::mpsc::sync_channel(1000);
                let gctx = ctx.as_context();

                let (ra, rb) = rayon::join(
                    || {
                        features
                            .iter()
                            .par_bridge()
                            .try_for_each_with(sender, |sender, feature| {
                                let packets = feature_to_packets(&gctx, feature);
                                for packet in packets {
                                    let bytes = serde_json::to_vec(&packet).unwrap();
                                    if sender.send(bytes).is_err() {
                                        return Err(SinkError::czml_writer(
                                            "Failed to send packet".to_string(),
                                        ));
                                    };
                                }
                                Ok(())
                            })
                    },
                    || {
                        let doc = Packet {
                            id: Some("document".into()),
                            version: Some("1.0".into()),
                            ..Default::default()
                        };
                        let mut buffer =
                            Vec::from(format!(r#"[{},"#, serde_json::to_string(&doc).unwrap()));

                        let mut iter = receiver.into_iter().peekable();
                        while let Some(bytes) = iter.next() {
                            buffer
                                .write(&bytes)
                                .map_err(crate::errors::SinkError::czml_writer)?;
                            if iter.peek().is_some() {
                                buffer
                                    .write(b",")
                                    .map_err(crate::errors::SinkError::czml_writer)?;
                            };
                        }

                        buffer
                            .write(b"]\n")
                            .map_err(crate::errors::SinkError::czml_writer)?;
                        storage
                            .put_sync(file_path.path().as_path(), Bytes::from(buffer))
                            .map_err(crate::errors::SinkError::czml_writer)
                    },
                );
                ra?;
                rb?;
            } else {
                let buffer = build_embedded_czml(features, &self.params)?;
                storage
                    .put_sync(file_path.path().as_path(), Bytes::from(buffer))
                    .map_err(crate::errors::SinkError::czml_writer)?;
            }
        }
        Ok(())
    }
}

/// Build a CZML document from features with embedded `czml.*` attributes
/// (produced by the reader's `PreserveRaw` strategy).
fn build_embedded_czml(
    features: &[Feature],
    params: &CzmlWriterParam,
) -> Result<Vec<u8>, BoxedError> {
    let per_entity_mode = params.time_field.is_some() && params.group_timeseries_by.is_none();

    // Auto-detect epoch for numeric time values in per-entity mode
    let effective_epoch: Option<String> = if per_entity_mode {
        if params.epoch.is_some() {
            params.epoch.clone()
        } else {
            let all_numeric = params.time_field.as_ref().is_some_and(|tf| {
                features.iter().all(|f| {
                    f.get(tf)
                        .map(|v| {
                            let s = attribute_value_to_string(v);
                            s.parse::<f64>().is_ok()
                        })
                        .unwrap_or(true)
                })
            });
            if all_numeric {
                Some("1970-01-01T00:00:00Z".to_string())
            } else {
                None
            }
        }
    } else {
        params.epoch.clone()
    };

    let mut global_start: Option<String> = None;
    let mut global_end: Option<String> = None;

    // Pass 1: Collect time range
    if per_entity_mode {
        if let Some(time_field) = &params.time_field {
            for feature in features {
                if let Some(time_val) = feature.get(time_field) {
                    let time_str = attribute_value_to_string(time_val);
                    let start_iso =
                        strip_epoch_offset_for_availability(&time_str, effective_epoch.as_deref());
                    if global_start.is_none() || start_iso < *global_start.as_ref().unwrap() {
                        global_start = Some(start_iso.clone());
                    }
                    if global_end.is_none() || start_iso > *global_end.as_ref().unwrap() {
                        global_end = Some(start_iso);
                    }
                }
            }
        }
    }
    if !per_entity_mode {
        for feature in features {
            if let Some(AttributeValue::String(avail)) = feature.get(Attribute::new("availability"))
            {
                if let Some((s, e)) = avail.split_once('/') {
                    if !s.is_empty() && !e.is_empty() {
                        if global_start.is_none() || s < global_start.as_deref().unwrap_or("") {
                            global_start = Some(s.to_string());
                        }
                        if global_end.is_none() || e > global_end.as_deref().unwrap_or("") {
                            global_end = Some(e.to_string());
                        }
                    }
                }
            }
        }
    }

    let mut doc = serde_json::json!({
        "id": "document",
        "version": "1.0",
    });
    if let (Some(start), Some(end)) = (&global_start, &global_end) {
        let availability = format!("{start}/{end}");
        doc["clock"] = serde_json::json!({
            "interval": availability,
            "currentTime": start,
            "multiplier": 1,
            "range": "LOOP_STOP",
            "step": "SYSTEM_CLOCK_MULTIPLIER",
        });
    }

    let mut output_buffer = Vec::new();
    output_buffer
        .write(format!("[{}", serde_json::to_string(&doc).unwrap()).as_bytes())
        .map_err(SinkError::czml_writer)?;

    // Pass 2: Build packets
    for (idx, feature) in features.iter().enumerate() {
        let mut packet = build_embedded_packet(
            feature,
            params,
            global_end.as_deref(),
            effective_epoch.as_deref(),
        )?;
        // Ensure every packet has an id (required by Cesium)
        if let Some(obj) = packet.as_object_mut() {
            if !obj.contains_key("id") {
                obj.insert("id".to_string(), serde_json::json!(format!("entity_{idx}")));
            }
        }
        output_buffer.write(b",").map_err(SinkError::czml_writer)?;
        output_buffer
            .write(&serde_json::to_vec(&packet).map_err(SinkError::czml_writer)?)
            .map_err(SinkError::czml_writer)?;
    }

    output_buffer
        .write(b"]\n")
        .map_err(SinkError::czml_writer)?;
    Ok(output_buffer)
}

/// Build a single CZML packet from a feature with embedded `czml.*` attributes.
/// When `params` provides `time_field` and `global_end` is set, per-entity availability
/// is computed. Polygon geometry is auto-converted when no graphic property exists.
/// `effective_epoch` is the epoch to use for numeric time conversion (may be auto-detected).
fn build_embedded_packet(
    feature: &Feature,
    params: &CzmlWriterParam,
    global_end: Option<&str>,
    effective_epoch: Option<&str>,
) -> Result<Value, BoxedError> {
    let mut packet = serde_json::Map::new();

    if let Some(AttributeValue::String(id)) = feature.get(Attribute::new("id")) {
        packet.insert("id".to_string(), serde_json::json!(id));
    } else if let Some(AttributeValue::String(name)) = feature.get(Attribute::new("name")) {
        // Use name as fallback id
        packet.insert("id".to_string(), serde_json::json!(name));
    }
    if let Some(AttributeValue::String(name)) = feature.get(Attribute::new("name")) {
        packet.insert("name".to_string(), serde_json::json!(name));
    }

    // Per-entity availability from time_field + global_end
    if let (Some(time_field), Some(end)) = (&params.time_field, global_end) {
        if let Some(time_val) = feature.get(time_field) {
            let time_str = attribute_value_to_string(time_val);
            let start_iso = strip_epoch_offset_for_availability(&time_str, effective_epoch);
            packet.insert(
                "availability".to_string(),
                serde_json::json!(format!("{start_iso}/{end}")),
            );
        }
    } else if let Some(AttributeValue::String(avail)) = feature.get(Attribute::new("availability"))
    {
        if !avail.is_empty() && avail != "/" {
            packet.insert("availability".to_string(), serde_json::json!(avail));
        }
    }

    if let Some(AttributeValue::String(parent)) = feature.get(Attribute::new("parent")) {
        packet.insert("parent".to_string(), serde_json::json!(parent));
    }

    let epoch = feature
        .get(Attribute::new("czml.epoch"))
        .and_then(|v| match v {
            AttributeValue::String(s) => Some(s.clone()),
            _ => None,
        });
    let interp_alg = feature
        .get(Attribute::new("czml.interpolationAlgorithm"))
        .and_then(|v| match v {
            AttributeValue::String(s) => Some(s.clone()),
            _ => None,
        });
    let interp_deg = feature
        .get(Attribute::new("czml.interpolationDegree"))
        .and_then(|v| match v {
            AttributeValue::Number(n) => n.as_f64(),
            _ => None,
        });

    if let Some(AttributeValue::String(ts_json)) = feature.get(Attribute::new("czml.timeseries")) {
        if let Ok(samples) = serde_json::from_str::<Vec<Value>>(ts_json) {
            let mut cartographic_degrees: Vec<Value> = Vec::new();

            for sample in &samples {
                let time_str = sample["time"].as_str().unwrap_or("");
                let lon = sample["lon"].as_f64().unwrap_or(0.0);
                let lat = sample["lat"].as_f64().unwrap_or(0.0);
                let height = sample["height"].as_f64().unwrap_or(0.0);

                let time_value: Value = if let Some(offset) = parse_epoch_offset_timestamp(time_str)
                {
                    serde_json::json!(offset)
                } else if let Ok(n) = time_str.parse::<f64>() {
                    serde_json::json!(n)
                } else if let Some(offset) = sample["timeOffset"].as_f64() {
                    if offset == 0.0 && !time_str.is_empty() && time_str.contains('T') {
                        serde_json::json!(time_str)
                    } else {
                        serde_json::json!(offset)
                    }
                } else {
                    serde_json::json!(time_str)
                };

                cartographic_degrees.push(time_value);
                cartographic_degrees.push(serde_json::json!(lon));
                cartographic_degrees.push(serde_json::json!(lat));
                cartographic_degrees.push(serde_json::json!(height));
            }

            let mut position = serde_json::json!({
                "cartographicDegrees": cartographic_degrees,
            });
            if let Some(ep) = &epoch {
                position["epoch"] = serde_json::json!(ep);
            }
            if let Some(alg) = &interp_alg {
                position["interpolationAlgorithm"] = serde_json::json!(alg);
            }
            if let Some(deg) = interp_deg {
                position["interpolationDegree"] = serde_json::json!(deg as u32);
            }

            packet.insert("position".to_string(), position);
        }
    } else {
        // Static entity: position from geometry (only for point-like entities)
        if let Some((lon, lat, height)) = extract_point_coords(feature) {
            packet.insert(
                "position".to_string(),
                serde_json::json!({
                    "cartographicDegrees": [lon, lat, height],
                }),
            );
        }
    }

    let skip_czml_keys = [
        "czml.timeseries",
        "czml.epoch",
        "czml.interpolationAlgorithm",
        "czml.interpolationDegree",
    ];
    for (attr, value) in feature.attributes.iter() {
        let key = attr.to_string();
        if let Some(czml_key) = key.strip_prefix("czml.") {
            if skip_czml_keys.contains(&key.as_str()) {
                continue;
            }
            if let AttributeValue::String(json_str) = value {
                if let Ok(parsed) = serde_json::from_str::<Value>(json_str.as_str()) {
                    packet.insert(czml_key.to_string(), parsed);
                }
            }
        }
    }

    // Auto-convert polygon geometry when no graphic property was set via czml.* attributes
    if !packet.contains_key("point")
        && !packet.contains_key("billboard")
        && !packet.contains_key("model")
        && !packet.contains_key("label")
        && !packet.contains_key("polygon")
        && !packet.contains_key("polyline")
    {
        if let Some(polygon_val) = feature_geometry_to_polygon_json(feature, params) {
            packet.insert("polygon".to_string(), polygon_val);
            // Remove position for polygon entities (positions are in polygon.positions)
            packet.remove("position");
        }
        // No fallback point — if the geometry cannot be converted, rely on the
        // position that was already set (if any) or let the entity be data-only.
    }

    if !packet.contains_key("description") {
        if let Some(AttributeValue::String(desc)) = feature.get(Attribute::new("description")) {
            packet.insert("description".to_string(), serde_json::json!(desc));
        } else {
            // Auto-generate description from non-internal attributes
            let filtered = filter_description_attributes(&feature.attributes);
            let desc = map_to_html_table(&filtered);
            if !desc.is_empty() && desc != "<table></table>" {
                packet.insert("description".to_string(), serde_json::json!(desc));
            }
        }
    }

    // Add properties bag
    if !packet.contains_key("properties") {
        if let Some(props) = build_properties_bag(feature) {
            packet.insert("properties".to_string(), props);
        }
    }

    Ok(Value::Object(packet))
}

/// Build a CZML document with time-dynamic entities grouped by attribute.
fn build_timeseries_czml(
    features: &[Feature],
    params: &CzmlWriterParam,
    _ctx: &NodeContext,
) -> Result<Vec<u8>, BoxedError> {
    let time_field = params
        .time_field
        .as_ref()
        .ok_or_else(|| SinkError::czml_writer("time_field is required for timeseries output"))?;
    let group_attr = params.group_timeseries_by.as_ref().ok_or_else(|| {
        SinkError::czml_writer("group_timeseries_by is required for timeseries output")
    })?;

    let mut groups: IndexMap<String, Vec<&Feature>> = IndexMap::new();
    for feature in features {
        let key = feature
            .get(group_attr)
            .map(attribute_value_to_string)
            .unwrap_or_else(|| "unknown".to_string());
        groups.entry(key).or_default().push(feature);
    }

    let mut doc = serde_json::json!({
        "id": "document",
        "version": "1.0",
    });

    let mut all_timestamps: Vec<String> = Vec::new();
    for feature in features {
        if let Some(ts) = feature.get(time_field) {
            all_timestamps.push(attribute_value_to_string(ts));
        }
    }
    if all_timestamps.len() >= 2 {
        // Check if all timestamps are numeric
        let all_numeric = all_timestamps.iter().all(|ts| {
            if let Ok(n) = ts.parse::<f64>() {
                n.is_finite()
            } else {
                false
            }
        });

        // Sort timestamps (numeric or string)
        if all_numeric {
            all_timestamps.sort_by(|a, b| {
                let na = a.parse::<f64>().unwrap();
                let nb = b.parse::<f64>().unwrap();
                na.partial_cmp(&nb).unwrap_or(std::cmp::Ordering::Equal)
            });
        } else {
            all_timestamps.sort();
        }

        let start = &all_timestamps[0];
        let end = &all_timestamps[all_timestamps.len() - 1];

        // Convert numeric times to ISO 8601 for clock if epoch is available
        let (start_iso, end_iso) = if all_numeric && params.epoch.is_some() {
            let start_iso = strip_epoch_offset_for_availability(start, params.epoch.as_deref());
            let end_iso = strip_epoch_offset_for_availability(end, params.epoch.as_deref());
            (start_iso, end_iso)
        } else {
            (start.clone(), end.clone())
        };

        let availability = format!("{start_iso}/{end_iso}");
        doc["clock"] = serde_json::json!({
            "interval": availability,
            "currentTime": start_iso,
            "multiplier": 1,
            "range": "LOOP_STOP",
            "step": "SYSTEM_CLOCK_MULTIPLIER",
        });
    }

    let mut output_buffer = Vec::new();
    output_buffer
        .write(format!("[{}", serde_json::to_string(&doc).unwrap()).as_bytes())
        .map_err(SinkError::czml_writer)?;

    for (entity_id, group_features) in &groups {
        let packet = build_entity_packet(entity_id, group_features, params)?;
        output_buffer.write(b",").map_err(SinkError::czml_writer)?;
        output_buffer
            .write(&serde_json::to_vec(&packet).map_err(SinkError::czml_writer)?)
            .map_err(SinkError::czml_writer)?;
    }

    output_buffer
        .write(b"]\n")
        .map_err(SinkError::czml_writer)?;
    Ok(output_buffer)
}

/// Build a single CZML packet for a time-dynamic entity from grouped features.
fn build_entity_packet(
    entity_id: &str,
    features: &[&Feature],
    params: &CzmlWriterParam,
) -> Result<Value, BoxedError> {
    let time_field = params.time_field.as_ref().unwrap();

    // Check if all time values are numeric (finite numbers) and present
    let all_numeric_times = features.iter().all(|f| {
        if let Some(time_val) = f.get(time_field) {
            let time_str = attribute_value_to_string(time_val);
            if time_str.is_empty() {
                return false; // Empty times disqualify numeric mode
            }
            if let Ok(n) = time_str.parse::<f64>() {
                n.is_finite() // Reject NaN/Inf
            } else {
                false
            }
        } else {
            false // Missing time values disqualify numeric mode
        }
    });

    // Auto-generate epoch for numeric time values if not provided
    let epoch = if params.epoch.is_some() {
        params.epoch.clone()
    } else if all_numeric_times {
        // Use a default epoch for numeric time values
        // Using a fixed epoch makes the CZML time values consistent
        Some("1970-01-01T00:00:00Z".to_string())
    } else {
        None
    };

    let mut sorted: Vec<&Feature> = features.to_vec();
    sorted.sort_by(|a, b| {
        let ta = a
            .get(time_field)
            .map(attribute_value_to_string)
            .unwrap_or_default();
        let tb = b
            .get(time_field)
            .map(attribute_value_to_string)
            .unwrap_or_default();

        // For numeric comparison, parse and compare as numbers
        if all_numeric_times {
            // Safe to unwrap since all_numeric_times guarantees valid finite numbers
            let na = ta.parse::<f64>().unwrap();
            let nb = tb.parse::<f64>().unwrap();
            na.partial_cmp(&nb).unwrap_or(std::cmp::Ordering::Equal)
        } else {
            ta.cmp(&tb)
        }
    });

    let mut cartographic_degrees: Vec<Value> = Vec::new();
    let mut first_time: Option<String> = None;
    let mut last_time: Option<String> = None;

    for feature in &sorted {
        let time_str = feature
            .get(time_field)
            .map(attribute_value_to_string)
            .unwrap_or_default();

        if time_str.is_empty() {
            if let Some((lon, lat, height)) = extract_point_coords(feature) {
                cartographic_degrees.push(serde_json::json!(lon));
                cartographic_degrees.push(serde_json::json!(lat));
                cartographic_degrees.push(serde_json::json!(height));
            }
            continue;
        }

        if first_time.is_none() {
            first_time = Some(time_str.clone());
        }
        last_time = Some(time_str.clone());

        // Convert time values to proper format
        // Use numeric samples only if all times are numeric to avoid mixing types
        let time_value: Value = if all_numeric_times {
            // Emit numeric time samples
            if let Ok(n) = time_str.parse::<f64>() {
                if !n.is_finite() {
                    // Skip features with NaN/Inf in numeric mode
                    continue;
                }
                serde_json::json!(n)
            } else if let Some(offset) = parse_epoch_offset_timestamp(&time_str) {
                serde_json::json!(offset)
            } else {
                // Skip features with invalid numeric time
                continue;
            }
        } else {
            // Emit ISO 8601 string time samples
            serde_json::json!(time_str)
        };

        if let Some((lon, lat, height)) = extract_point_coords(feature) {
            cartographic_degrees.push(time_value);
            cartographic_degrees.push(serde_json::json!(lon));
            cartographic_degrees.push(serde_json::json!(lat));
            cartographic_degrees.push(serde_json::json!(height));
        }
    }

    let is_time_dynamic = first_time.is_some();
    let mut position = if is_time_dynamic {
        serde_json::json!({
            "cartographicDegrees": cartographic_degrees,
            "interpolationAlgorithm": params.interpolation_algorithm.to_string(),
            "interpolationDegree": params.interpolation_degree,
        })
    } else {
        serde_json::json!({
            "cartographicDegrees": cartographic_degrees,
        })
    };
    if is_time_dynamic {
        if let Some(epoch) = &epoch {
            position["epoch"] = serde_json::json!(epoch);
        }
    }

    let availability = match (&first_time, &last_time) {
        (Some(start), Some(end)) if start != end => {
            let start_iso = strip_epoch_offset_for_availability(start, epoch.as_deref());
            let end_iso = strip_epoch_offset_for_availability(end, epoch.as_deref());
            Some(format!("{start_iso}/{end_iso}"))
        }
        _ => None,
    };

    let name = sorted
        .first()
        .and_then(|f| f.get(Attribute::new("name")))
        .map(attribute_value_to_string);

    let description = sorted.first().map(|f| {
        let filtered = filter_description_attributes(&f.attributes);
        map_to_html_table(&filtered)
    });

    let mut packet = serde_json::json!({
        "id": entity_id,
        "position": position,
    });

    if let Some(avail) = availability {
        packet["availability"] = serde_json::json!(avail);
    }
    if let Some(n) = name {
        packet["name"] = serde_json::json!(n);
    }
    if let Some(d) = description {
        packet["description"] = serde_json::json!(d);
    }

    // Add graphic based on geometry type: polygon for polygon features, point for others
    if let Some(first_feature) = sorted.first() {
        if let Some(polygon_val) = feature_geometry_to_polygon_json(first_feature, params) {
            packet["polygon"] = polygon_val;
            // Polygon positions are self-contained; remove redundant position
            packet.as_object_mut().map(|m| m.remove("position"));
        } else {
            packet["point"] = serde_json::json!({
                "pixelSize": 10,
                "heightReference": "NONE",
            });
        }
    }

    Ok(packet)
}

fn extract_point_coords(feature: &Feature) -> Option<(f64, f64, f64)> {
    match &feature.geometry.value {
        GeometryValue::FlowGeometry3D(Geometry3D::Point(p)) => Some((p.x(), p.y(), p.z())),
        GeometryValue::FlowGeometry2D(Geometry2D::Point(p)) => Some((p.x(), p.y(), 0.0)),
        GeometryValue::FlowGeometry3D(Geometry3D::LineString(ls)) => {
            ls.0.first().map(|c| (c.x, c.y, c.z))
        }
        GeometryValue::FlowGeometry2D(Geometry2D::LineString(ls)) => {
            ls.0.first().map(|c| (c.x, c.y, 0.0))
        }
        GeometryValue::FlowGeometry3D(Geometry3D::Polygon(poly)) => {
            poly.exterior().0.first().map(|c| (c.x, c.y, c.z))
        }
        GeometryValue::FlowGeometry2D(Geometry2D::Polygon(poly)) => {
            poly.exterior().0.first().map(|c| (c.x, c.y, 0.0))
        }
        _ => None,
    }
}

/// Parse `"<iso>+<N>s"` and return the numeric offset `N`.
fn parse_epoch_offset_timestamp(s: &str) -> Option<f64> {
    if let Some(idx) = s.rfind('+') {
        let suffix = &s[idx + 1..];
        if let Some(num_str) = suffix.strip_suffix('s') {
            return num_str.parse::<f64>().ok();
        }
    }
    None
}

/// Convert an epoch-relative timestamp to ISO 8601 for CZML `availability`.
fn strip_epoch_offset_for_availability(ts: &str, epoch: Option<&str>) -> String {
    // Handle epoch offset format like "2024-01-01T00:00:00Z+120s"
    if let Some(offset) = parse_epoch_offset_timestamp(ts) {
        if let Some(epoch_str) = epoch {
            if let Ok(epoch_dt) = chrono::DateTime::parse_from_rfc3339(epoch_str) {
                let result = epoch_dt + chrono::Duration::seconds(offset as i64);
                return result.to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
            }
        }
    }

    // Handle pure numeric strings like "155" when epoch is provided
    if let Ok(offset) = ts.parse::<f64>() {
        // Reject NaN/Inf
        if !offset.is_finite() {
            return ts.to_string();
        }
        if let Some(epoch_str) = epoch {
            if let Ok(epoch_dt) = chrono::DateTime::parse_from_rfc3339(epoch_str) {
                // Preserve fractional seconds by converting to milliseconds
                let millis = (offset * 1000.0) as i64;
                let result = epoch_dt + chrono::Duration::milliseconds(millis);
                // Use seconds format for whole seconds, millis for fractional
                let format = if offset.fract() == 0.0 {
                    chrono::SecondsFormat::Secs
                } else {
                    chrono::SecondsFormat::Millis
                };
                return result.to_rfc3339_opts(format, true);
            }
        }
    }

    ts.to_string()
}

fn attribute_value_to_string(value: &AttributeValue) -> String {
    match value {
        AttributeValue::String(s) => s.clone(),
        AttributeValue::Number(n) => n.to_string(),
        AttributeValue::Bool(b) => b.to_string(),
        _ => {
            let v: serde_json::Value = value.clone().into();
            v.to_string()
        }
    }
}

/// Filter attributes for description, removing internal and duplicate keys.
fn filter_description_attributes(
    attrs: &IndexMap<Attribute, AttributeValue>,
) -> IndexMap<Attribute, AttributeValue> {
    let skip_prefixes = ["czml.", "_", "http_"];
    let skip_keys = [
        "id",
        "name",
        "description",
        "availability",
        "parent",
        "match",
    ];

    attrs
        .iter()
        .filter(|(k, _)| {
            let key = k.to_string();
            !skip_prefixes.iter().any(|p| key.starts_with(p)) && !skip_keys.contains(&key.as_str())
        })
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect()
}

fn map_to_html_table(map: &IndexMap<Attribute, AttributeValue>) -> String {
    let mut html = String::new();
    html.push_str("<table>");
    for (key, value) in map {
        let display = attribute_value_to_string(value);
        html.push_str(&format!("<tr><td>{key}</td><td>{display}</td></tr>"));
    }
    html.push_str("</table>");
    html
}

fn polygon_to_czml_polygon(poly: &Polygon3D<f64>) -> CzmlPolygon {
    let mut czml_polygon = CzmlPolygon::default();

    let exteriors = poly
        .exterior()
        .iter()
        .flat_map(|coord| vec![coord.x, coord.y, coord.z])
        .collect_vec();
    czml_polygon.positions = Some(PositionList::Object(PositionListProperties {
        cartographic_degrees: Some(exteriors),
        ..Default::default()
    }));

    let interiors = poly
        .interiors()
        .iter()
        .flat_map(|line| line.iter().map(|coord| vec![coord.x, coord.y, coord.z]))
        .collect_vec();
    czml_polygon.holes = Some(PositionListOfLists::Object(PositionListOfListsProperties {
        cartographic_degrees: Some(interiors),
        ..Default::default()
    }));

    czml_polygon
}

/// Parse a hex color string like "#ffd8c0" into [r, g, b, alpha].
fn hex_to_rgba(hex: &str, alpha: u8) -> Option<[u8; 4]> {
    let hex = hex.strip_prefix('#').unwrap_or(hex);
    if hex.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some([r, g, b, alpha])
}

/// Convert a Feature's polygon geometry to a styled CZML polygon JSON value.
fn feature_geometry_to_polygon_json(feature: &Feature, params: &CzmlWriterParam) -> Option<Value> {
    let czml_polygon = match &feature.geometry.value {
        GeometryValue::FlowGeometry3D(Geometry3D::Polygon(poly)) => polygon_to_czml_polygon(poly),
        GeometryValue::FlowGeometry2D(Geometry2D::Polygon(poly)) => {
            let mut czml_poly = CzmlPolygon::default();
            let exteriors: Vec<f64> = poly
                .exterior()
                .iter()
                .flat_map(|coord| vec![coord.x, coord.y, 0.0])
                .collect();
            czml_poly.positions = Some(PositionList::Object(PositionListProperties {
                cartographic_degrees: Some(exteriors),
                ..Default::default()
            }));
            let interiors: Vec<Vec<f64>> = poly
                .interiors()
                .iter()
                .map(|line| {
                    line.iter()
                        .flat_map(|coord| vec![coord.x, coord.y, 0.0])
                        .collect()
                })
                .collect();
            if !interiors.is_empty() {
                czml_poly.holes =
                    Some(PositionListOfLists::Object(PositionListOfListsProperties {
                        cartographic_degrees: Some(interiors),
                        ..Default::default()
                    }));
            }
            czml_poly
        }
        GeometryValue::FlowGeometry3D(Geometry3D::MultiPolygon(mp)) => {
            mp.0.first().map(polygon_to_czml_polygon)?
        }
        GeometryValue::FlowGeometry2D(Geometry2D::MultiPolygon(mp)) => {
            let poly = mp.0.first()?;
            let mut czml_poly = CzmlPolygon::default();
            let exteriors: Vec<f64> = poly
                .exterior()
                .iter()
                .flat_map(|coord| vec![coord.x, coord.y, 0.0])
                .collect();
            czml_poly.positions = Some(PositionList::Object(PositionListProperties {
                cartographic_degrees: Some(exteriors),
                ..Default::default()
            }));
            czml_poly
        }
        _ => return None,
    };

    // Serialize base polygon to JSON, then apply styling
    let mut polygon_val = serde_json::to_value(&czml_polygon).ok()?;
    let obj = polygon_val.as_object_mut()?;

    // Apply color from attribute
    if let Some(color_attr) = &params.color_attribute {
        if let Some(AttributeValue::String(hex)) = feature.get(color_attr) {
            if let Some(rgba) = hex_to_rgba(hex, params.opacity) {
                obj.insert(
                    "material".to_string(),
                    serde_json::json!({
                        "solidColor": {
                            "color": {
                                "rgba": rgba
                            }
                        }
                    }),
                );
            }
        }
    }

    // Apply extrusion height from attribute
    if let Some(height_attr) = &params.height_attribute {
        if let Some(height_val) = feature.get(height_attr) {
            if let Some(h) = height_val.as_f64() {
                obj.insert("extrudedHeight".to_string(), serde_json::json!(h));
                obj.insert("closeBottom".to_string(), serde_json::json!(true));
                obj.insert("closeTop".to_string(), serde_json::json!(true));
            }
        }
    }

    // Default outline
    obj.insert("fill".to_string(), serde_json::json!(true));
    obj.insert("outline".to_string(), serde_json::json!(true));
    obj.insert(
        "outlineColor".to_string(),
        serde_json::json!({ "rgba": [80, 80, 80, 220] }),
    );
    obj.insert("outlineWidth".to_string(), serde_json::json!(1.0));

    Some(polygon_val)
}

/// Build a properties bag from feature attributes, excluding internal keys.
fn build_properties_bag(feature: &Feature) -> Option<Value> {
    let skip_prefixes = ["czml."];
    let skip_keys = ["id", "name", "description", "availability", "parent"];

    let mut properties = serde_json::Map::new();
    for (attr, value) in feature.attributes.iter() {
        let key = attr.to_string();
        if skip_prefixes.iter().any(|p| key.starts_with(p)) {
            continue;
        }
        if skip_keys.contains(&key.as_str()) {
            continue;
        }
        let json_val: serde_json::Value = value.clone().into();
        // Skip null values — Cesium's CzmlDataSource crashes on null property values
        if json_val.is_null() {
            continue;
        }
        properties.insert(key, json_val);
    }

    if properties.is_empty() {
        None
    } else {
        Some(Value::Object(properties))
    }
}

fn feature_to_packets(ctx: &Context, feature: &Feature) -> Vec<Packet> {
    let Some(parent_id) = feature.metadata.feature_id.clone() else {
        ctx.event_hub
            .warn_log(None, "Feature does not have a feature_id".to_string());
        return vec![];
    };

    let properties = map_to_html_table(&feature.attributes);

    let GeometryValue::CityGmlGeometry(geometry) = &feature.geometry.value else {
        ctx.event_hub.warn_log(
            None,
            format!(
                "Geometry is not a CityGML geometry with: feature_id={}",
                feature.id
            ),
        );
        return vec![];
    };

    let polygons = geometry
        .gml_geometries
        .iter()
        .filter(|geometry| geometry.lod.unwrap_or(0) > 0)
        .flat_map(|geometry| geometry.polygons.clone())
        .collect_vec();

    if polygons.is_empty() {
        ctx.event_hub.warn_log(
            None,
            format!(
                "Geometry does not contain any polygons: feature_id={}",
                feature.id
            ),
        );
        return vec![];
    }

    // Create a Packet that retains attributes and references it from child features
    let properties_packet = Packet {
        id: Some(parent_id.clone()),
        description: Some(StringValueType::String(properties)),
        ..Default::default()
    };
    let mut packets: Vec<Packet> = vec![properties_packet];

    for poly in polygons {
        let mut czml_polygon = polygon_to_czml_polygon(&poly);
        // In Cesium, if perPositionHeight is false, the polygon height is fixed
        czml_polygon.per_position_height = CzmlBoolean::Boolean(true);

        let packet = Packet {
            polygon: Some(czml_polygon),
            description: Some(StringValueType::Object(StringProperties {
                reference: Some(format!("{parent_id}#description")),
                ..Default::default()
            })),
            parent: Some(parent_id.clone()),
            ..Default::default()
        };
        packets.push(packet);
    }

    packets
}

#[cfg(test)]
mod tests {
    use super::*;
    use reearth_flow_geometry::types::coordinate::Coordinate;
    use reearth_flow_geometry::types::line_string::LineString;
    use reearth_flow_geometry::types::no_value::NoValue;
    use reearth_flow_geometry::types::point::Point3D;
    use reearth_flow_geometry::types::polygon::Polygon;
    use reearth_flow_types::Geometry;

    fn make_feature_3d(lon: f64, lat: f64, height: f64) -> Feature {
        Feature::new_with_attributes_and_geometry(
            indexmap::IndexMap::new(),
            Geometry {
                epsg: Some(4326),
                value: GeometryValue::FlowGeometry3D(Geometry3D::Point(Point3D::new(
                    lon, lat, height,
                ))),
            },
            Default::default(),
        )
    }

    fn make_timeseries_params() -> CzmlWriterParam {
        CzmlWriterParam {
            output: Expr::new("/tmp/test.czml".to_string()),
            group_by: None,
            time_field: Some(Attribute::new("timestamp")),
            epoch: Some("2024-01-01T00:00:00Z".into()),
            interpolation_algorithm: InterpolationAlgorithm::Lagrange,
            interpolation_degree: 5,
            group_timeseries_by: Some(Attribute::new("entityId")),
            color_attribute: None,
            opacity: default_opacity(),
            height_attribute: None,
        }
    }

    #[test]
    fn test_build_entity_packet_basic() {
        let params = make_timeseries_params();
        let mut f1 = make_feature_3d(-75.0, 40.0, 0.0);
        f1.insert(
            Attribute::new("timestamp"),
            AttributeValue::String("2024-01-01T00:00:00Z".into()),
        );
        f1.insert(
            Attribute::new("entityId"),
            AttributeValue::String("v1".into()),
        );
        f1.insert(
            Attribute::new("name"),
            AttributeValue::String("Vehicle".into()),
        );

        let mut f2 = make_feature_3d(-76.0, 41.0, 100.0);
        f2.insert(
            Attribute::new("timestamp"),
            AttributeValue::String("2024-01-01T00:01:00Z".into()),
        );
        f2.insert(
            Attribute::new("entityId"),
            AttributeValue::String("v1".into()),
        );

        let features_ref: Vec<&Feature> = vec![&f1, &f2];
        let packet = build_entity_packet("v1", &features_ref, &params).unwrap();

        assert_eq!(packet["id"], "v1");
        assert_eq!(packet["name"], "Vehicle");
        assert_eq!(packet["position"]["interpolationAlgorithm"], "LAGRANGE");
        assert_eq!(packet["position"]["epoch"], "2024-01-01T00:00:00Z");

        let coords = packet["position"]["cartographicDegrees"]
            .as_array()
            .unwrap();
        assert_eq!(coords.len(), 8);
        assert!(packet["availability"].as_str().is_some());
    }

    fn make_embedded_feature_with_timeseries() -> Feature {
        let mut f = make_feature_3d(139.6917, 35.6895, 50.0);
        f.insert(
            Attribute::new("id"),
            AttributeValue::String("vehicle-a".into()),
        );
        f.insert(
            Attribute::new("name"),
            AttributeValue::String("Vehicle Alpha".into()),
        );
        f.insert(
            Attribute::new("availability"),
            AttributeValue::String("2024-01-01T00:00:00Z/2024-01-01T00:10:00Z".into()),
        );
        f.insert(
            Attribute::new("czml.epoch"),
            AttributeValue::String("2024-01-01T00:00:00Z".into()),
        );
        f.insert(
            Attribute::new("czml.interpolationAlgorithm"),
            AttributeValue::String("LAGRANGE".into()),
        );
        f.insert(
            Attribute::new("czml.interpolationDegree"),
            AttributeValue::Number(serde_json::Number::from(5)),
        );
        let ts = serde_json::json!([
            {"time": "2024-01-01T00:00:00Z+0s", "timeOffset": 0.0, "lon": 139.6917, "lat": 35.6895, "height": 50.0},
            {"time": "2024-01-01T00:00:00Z+120s", "timeOffset": 120.0, "lon": 139.7003, "lat": 35.69, "height": 52.0},
        ]);
        f.insert(
            Attribute::new("czml.timeseries"),
            AttributeValue::String(serde_json::to_string(&ts).unwrap()),
        );
        f.insert(
            Attribute::new("czml.point"),
            AttributeValue::String(r#"{"pixelSize":12,"color":{"rgba":[255,0,0,255]}}"#.into()),
        );
        f.insert(
            Attribute::new("czml.path"),
            AttributeValue::String(
                r#"{"material":{"solidColor":{"color":{"rgba":[255,0,0,128]}}},"width":2}"#.into(),
            ),
        );
        f
    }

    fn make_embedded_static_feature() -> Feature {
        let mut f = make_feature_3d(139.7454, 35.6586, 333.0);
        f.insert(
            Attribute::new("id"),
            AttributeValue::String("static-poi".into()),
        );
        f.insert(
            Attribute::new("name"),
            AttributeValue::String("Tokyo Tower".into()),
        );
        f.insert(
            Attribute::new("czml.label"),
            AttributeValue::String(r#"{"text":"Tokyo Tower","font":"14pt sans-serif"}"#.into()),
        );
        f
    }

    fn make_default_params() -> CzmlWriterParam {
        CzmlWriterParam {
            output: Expr::new("/tmp/test.czml".to_string()),
            group_by: None,
            time_field: None,
            epoch: None,
            interpolation_algorithm: InterpolationAlgorithm::default(),
            interpolation_degree: 1,
            group_timeseries_by: None,
            color_attribute: None,
            opacity: default_opacity(),
            height_attribute: None,
        }
    }

    #[test]
    fn test_build_embedded_packet_timeseries() {
        let f = make_embedded_feature_with_timeseries();
        let params = make_default_params();
        let packet = build_embedded_packet(&f, &params, None, None).unwrap();

        assert_eq!(packet["id"], "vehicle-a");
        assert_eq!(packet["name"], "Vehicle Alpha");

        assert_eq!(packet["position"]["epoch"], "2024-01-01T00:00:00Z");
        assert_eq!(packet["position"]["interpolationAlgorithm"], "LAGRANGE");
        assert_eq!(packet["position"]["interpolationDegree"], 5);

        let coords = packet["position"]["cartographicDegrees"]
            .as_array()
            .unwrap();
        assert_eq!(coords.len(), 8);
        assert_eq!(coords[0].as_f64().unwrap(), 0.0);
        assert_eq!(coords[4].as_f64().unwrap(), 120.0);

        assert_eq!(packet["point"]["pixelSize"], 12);
        assert_eq!(packet["path"]["width"], 2);
        assert!(packet["availability"].as_str().unwrap().contains('/'));
    }

    #[test]
    fn test_build_embedded_packet_static() {
        let f = make_embedded_static_feature();
        let params = make_default_params();
        let packet = build_embedded_packet(&f, &params, None, None).unwrap();

        assert_eq!(packet["id"], "static-poi");
        assert_eq!(packet["name"], "Tokyo Tower");

        let coords = packet["position"]["cartographicDegrees"]
            .as_array()
            .unwrap();
        assert_eq!(coords.len(), 3);
        assert!((coords[0].as_f64().unwrap() - 139.7454).abs() < 1e-4);

        assert_eq!(packet["label"]["text"], "Tokyo Tower");
        assert!(packet.get("point").is_none());
        assert!(packet.get("availability").is_none());
    }

    #[test]
    fn test_build_embedded_czml_document() {
        let f1 = make_embedded_feature_with_timeseries();
        let f2 = make_embedded_static_feature();
        let features = vec![f1, f2];

        let params = CzmlWriterParam {
            output: Expr::new("/tmp/test.czml".to_string()),
            group_by: None,
            time_field: None,
            epoch: None,
            interpolation_algorithm: InterpolationAlgorithm::default(),
            interpolation_degree: 1,
            group_timeseries_by: None,
            color_attribute: None,
            opacity: default_opacity(),
            height_attribute: None,
        };

        let buffer = build_embedded_czml(&features, &params).unwrap();
        let czml: Vec<Value> = serde_json::from_slice(&buffer).unwrap();

        assert_eq!(czml.len(), 3);
        assert_eq!(czml[0]["id"], "document");
        assert_eq!(czml[0]["version"], "1.0");

        assert!(czml[0]["clock"]["interval"]
            .as_str()
            .unwrap()
            .contains("2024-01-01T00:00:00Z"));

        assert_eq!(czml[1]["id"], "vehicle-a");
        assert_eq!(czml[2]["id"], "static-poi");
    }

    #[test]
    fn test_build_entity_packet_numeric_times() {
        // Test with numeric time values and no explicit epoch
        let params = CzmlWriterParam {
            output: Expr::new("/tmp/test.czml".to_string()),
            group_by: None,
            time_field: Some(Attribute::new("timestamp")),
            epoch: None, // No explicit epoch - should auto-generate
            interpolation_algorithm: InterpolationAlgorithm::Linear,
            interpolation_degree: 1,
            group_timeseries_by: Some(Attribute::new("entityId")),
            color_attribute: None,
            opacity: default_opacity(),
            height_attribute: None,
        };

        let mut f1 = make_feature_3d(139.7, 35.7, 0.0);
        f1.insert(
            Attribute::new("timestamp"),
            AttributeValue::String("0".into()), // Numeric string
        );
        f1.insert(
            Attribute::new("entityId"),
            AttributeValue::String("test1".into()),
        );

        let mut f2 = make_feature_3d(139.8, 35.8, 100.0);
        f2.insert(
            Attribute::new("timestamp"),
            AttributeValue::String("60".into()), // Numeric string (60 seconds later)
        );
        f2.insert(
            Attribute::new("entityId"),
            AttributeValue::String("test1".into()),
        );

        let features_ref: Vec<&Feature> = vec![&f1, &f2];
        let packet = build_entity_packet("test1", &features_ref, &params).unwrap();

        assert_eq!(packet["id"], "test1");
        // Should have auto-generated epoch
        assert_eq!(packet["position"]["epoch"], "1970-01-01T00:00:00Z");

        let coords = packet["position"]["cartographicDegrees"]
            .as_array()
            .unwrap();
        assert_eq!(coords.len(), 8);

        // Time values should be numeric
        assert_eq!(coords[0].as_f64().unwrap(), 0.0);
        assert_eq!(coords[4].as_f64().unwrap(), 60.0);

        // Availability should be ISO 8601 format
        let avail = packet["availability"].as_str().unwrap();
        assert!(avail.contains("1970-01-01T00:00:00Z"));
        assert!(avail.contains("1970-01-01T00:01:00Z"));
    }

    #[test]
    fn test_hex_to_rgba() {
        assert_eq!(hex_to_rgba("#ffd8c0", 180), Some([255, 216, 192, 180]));
        assert_eq!(hex_to_rgba("ff0000", 255), Some([255, 0, 0, 255]));
        assert_eq!(hex_to_rgba("#000000", 0), Some([0, 0, 0, 0]));
        assert_eq!(hex_to_rgba("zzzzzz", 180), None);
        assert_eq!(hex_to_rgba("#fff", 180), None); // too short
    }

    fn make_polygon_2d_feature() -> Feature {
        let coords: Vec<Coordinate<f64, NoValue>> = vec![
            (139.75, 35.68).into(),
            (139.76, 35.68).into(),
            (139.76, 35.69).into(),
            (139.75, 35.69).into(),
            (139.75, 35.68).into(),
        ];
        let exterior = LineString(coords);
        let polygon: Polygon<f64, NoValue> = Polygon::new(exterior, vec![]);

        let mut f = Feature::new_with_attributes_and_geometry(
            indexmap::IndexMap::new(),
            Geometry {
                epsg: Some(4326),
                value: GeometryValue::FlowGeometry2D(Geometry2D::Polygon(polygon)),
            },
            Default::default(),
        );
        f.insert(Attribute::new("id"), AttributeValue::String("poly1".into()));
        f.insert(
            Attribute::new("name"),
            AttributeValue::String("Test Polygon".into()),
        );
        f
    }

    #[test]
    fn test_feature_geometry_to_polygon_2d() {
        let f = make_polygon_2d_feature();
        let params = make_default_params();
        let polygon_val = feature_geometry_to_polygon_json(&f, &params);
        assert!(polygon_val.is_some());

        let pv = polygon_val.unwrap();
        let positions = &pv["positions"]["cartographicDegrees"];
        assert!(positions.is_array());
        let coords = positions.as_array().unwrap();
        // 5 coords * 3 values (lon, lat, 0.0) = 15
        assert_eq!(coords.len(), 15);
        // First coord: lon
        assert!((coords[0].as_f64().unwrap() - 139.75).abs() < 1e-4);
        // Z should be 0
        assert_eq!(coords[2].as_f64().unwrap(), 0.0);

        // Should have fill and outline
        assert_eq!(pv["fill"], true);
        assert_eq!(pv["outline"], true);
    }

    #[test]
    fn test_polygon_with_styling() {
        let mut f = make_polygon_2d_feature();
        f.insert(
            Attribute::new("fill_color"),
            AttributeValue::String("#ffd8c0".into()),
        );
        f.insert(
            Attribute::new("depth"),
            AttributeValue::Number(serde_json::Number::from_f64(1.5).unwrap()),
        );

        let params = CzmlWriterParam {
            color_attribute: Some(Attribute::new("fill_color")),
            opacity: 180,
            height_attribute: Some(Attribute::new("depth")),
            ..make_default_params()
        };

        let polygon_val = feature_geometry_to_polygon_json(&f, &params).unwrap();

        // Check material color
        let rgba = &polygon_val["material"]["solidColor"]["color"]["rgba"];
        assert_eq!(rgba[0], 255);
        assert_eq!(rgba[1], 216);
        assert_eq!(rgba[2], 192);
        assert_eq!(rgba[3], 180);

        // Check extrusion
        assert_eq!(polygon_val["extrudedHeight"], 1.5);
        assert_eq!(polygon_val["closeBottom"], true);
        assert_eq!(polygon_val["closeTop"], true);
    }

    #[test]
    fn test_per_entity_availability() {
        let mut f1 = make_polygon_2d_feature();
        f1.insert(
            Attribute::new("start_time"),
            AttributeValue::String("3600".into()), // 1 hour offset
        );

        let mut f2 = make_polygon_2d_feature();
        f2.insert(Attribute::new("id"), AttributeValue::String("poly2".into()));
        f2.insert(
            Attribute::new("start_time"),
            AttributeValue::String("7200".into()), // 2 hour offset
        );

        let params = CzmlWriterParam {
            time_field: Some(Attribute::new("start_time")),
            epoch: Some("2024-01-01T00:00:00Z".into()),
            ..make_default_params()
        };

        let features = vec![f1, f2];
        let buffer = build_embedded_czml(&features, &params).unwrap();
        let czml: Vec<Value> = serde_json::from_slice(&buffer).unwrap();

        // Document + 2 entities
        assert_eq!(czml.len(), 3);
        assert_eq!(czml[0]["id"], "document");
        assert!(czml[0]["clock"].is_object());

        // First entity should have availability starting at 1 hour
        let avail1 = czml[1]["availability"].as_str().unwrap();
        assert!(avail1.contains("2024-01-01T01:00:00Z"));

        // Second entity should have availability starting at 2 hours
        let avail2 = czml[2]["availability"].as_str().unwrap();
        assert!(avail2.contains("2024-01-01T02:00:00Z"));

        // Both should end at the same time (global end = max start = 2 hours)
        assert!(avail1.ends_with("2024-01-01T02:00:00Z"));
        assert!(avail2.ends_with("2024-01-01T02:00:00Z"));

        // Should have polygon geometry (not point)
        assert!(czml[1]["polygon"].is_object());
        assert!(czml[1].get("point").is_none());
    }

    #[test]
    fn test_build_properties_bag() {
        let mut f = make_feature_3d(139.7, 35.7, 0.0);
        f.insert(Attribute::new("id"), AttributeValue::String("test".into()));
        f.insert(
            Attribute::new("name"),
            AttributeValue::String("Test".into()),
        );
        f.insert(
            Attribute::new("depth"),
            AttributeValue::Number(serde_json::Number::from_f64(1.5).unwrap()),
        );
        f.insert(
            Attribute::new("czml.point"),
            AttributeValue::String("{}".into()),
        );
        f.insert(
            Attribute::new("custom_field"),
            AttributeValue::String("hello".into()),
        );

        let props = build_properties_bag(&f).unwrap();
        let obj = props.as_object().unwrap();

        // Should include custom fields but not internal ones
        assert!(obj.contains_key("depth"));
        assert!(obj.contains_key("custom_field"));
        assert!(!obj.contains_key("id"));
        assert!(!obj.contains_key("name"));
        assert!(!obj.contains_key("czml.point"));
    }
}
