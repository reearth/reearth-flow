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
        "Export features as CZML for Cesium visualization, with support for time-dynamic entities and timeseries positions"
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
        let params = if let Some(with) = with {
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
    /// Attribute containing the timestamp (ISO 8601 string or numeric seconds
    /// since epoch) for each feature. When set together with
    /// `groupTimeseriesBy`, features sharing the same group key are combined
    /// into a single CZML entity with time-tagged position samples.
    pub(super) time_field: Option<Attribute>,
    /// # Epoch
    /// ISO 8601 datetime used as the base time for numeric time offsets in
    /// the output CZML.  If omitted the first timestamp encountered is used.
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
}

fn default_interpolation_degree() -> u32 {
    1
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
            let key = group_by
                .iter()
                .map(|k| feature.get(k).cloned().unwrap_or(AttributeValue::Null))
                .collect::<Vec<_>>();
            AttributeValue::Array(key)
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

            let has_embedded = features
                .first()
                .map(|f| {
                    f.attributes
                        .contains_key(&Attribute::new("czml.timeseries"))
                })
                .unwrap_or(false);
            let is_grouped_timeseries =
                self.params.group_timeseries_by.is_some() && self.params.time_field.is_some();

            if has_embedded {
                let buffer = build_embedded_czml(features, &self.params)?;
                storage
                    .put_sync(file_path.path().as_path(), Bytes::from(buffer))
                    .map_err(crate::errors::SinkError::czml_writer)?;
            } else if is_grouped_timeseries {
                let buffer = build_timeseries_czml(features, &self.params, &ctx)?;
                storage
                    .put_sync(file_path.path().as_path(), Bytes::from(buffer))
                    .map_err(crate::errors::SinkError::czml_writer)?;
            } else {
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
            }
        }
        Ok(())
    }
}

/// Build a CZML document from features with embedded `czml.*` attributes
/// (produced by the reader's `PreserveRaw` strategy).
fn build_embedded_czml(
    features: &[Feature],
    _params: &CzmlWriterParam,
) -> Result<Vec<u8>, BoxedError> {
    let mut global_start: Option<String> = None;
    let mut global_end: Option<String> = None;

    for feature in features {
        if let Some(AttributeValue::String(avail)) = feature.get(Attribute::new("availability")) {
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

    for feature in features {
        let packet = build_embedded_packet(feature)?;
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
fn build_embedded_packet(feature: &Feature) -> Result<Value, BoxedError> {
    let mut packet = serde_json::Map::new();

    if let Some(AttributeValue::String(id)) = feature.get(Attribute::new("id")) {
        packet.insert("id".to_string(), serde_json::json!(id));
    }
    if let Some(AttributeValue::String(name)) = feature.get(Attribute::new("name")) {
        packet.insert("name".to_string(), serde_json::json!(name));
    }
    if let Some(AttributeValue::String(avail)) = feature.get(Attribute::new("availability")) {
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
        // Static entity: position from geometry
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
    for (attr, value) in &feature.attributes {
        let key = attr.to_string();
        if let Some(czml_key) = key.strip_prefix("czml.") {
            if skip_czml_keys.contains(&key.as_str()) {
                continue;
            }
            if let AttributeValue::String(json_str) = value {
                if let Ok(parsed) = serde_json::from_str::<Value>(json_str) {
                    packet.insert(czml_key.to_string(), parsed);
                }
            }
        }
    }

    if !packet.contains_key("point")
        && !packet.contains_key("billboard")
        && !packet.contains_key("model")
        && !packet.contains_key("label")
        && !packet.contains_key("polygon")
        && !packet.contains_key("polyline")
    {
        packet.insert(
            "point".to_string(),
            serde_json::json!({
                "pixelSize": 10,
                "heightReference": "NONE",
            }),
        );
    }

    if !packet.contains_key("description") {
        if let Some(AttributeValue::String(desc)) = feature.get(Attribute::new("description")) {
            packet.insert("description".to_string(), serde_json::json!(desc));
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
        all_timestamps.sort();
        let start = &all_timestamps[0];
        let end = &all_timestamps[all_timestamps.len() - 1];
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
    let epoch = params.epoch.clone();

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
        ta.cmp(&tb)
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

        let time_value: Value = if epoch.is_some() {
            if let Ok(n) = time_str.parse::<f64>() {
                serde_json::json!(n)
            } else if let Some(offset) = parse_epoch_offset_timestamp(&time_str) {
                serde_json::json!(offset)
            } else {
                serde_json::json!(time_str)
            }
        } else {
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
        let filtered: IndexMap<Attribute, AttributeValue> = f
            .attributes
            .iter()
            .filter(|(k, _)| {
                let name = k.to_string();
                !name.starts_with("czml.") && name != time_field.to_string()
            })
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
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

    packet["point"] = serde_json::json!({
        "pixelSize": 10,
        "heightReference": "NONE",
    });

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
    if let Some(offset) = parse_epoch_offset_timestamp(ts) {
        if let Some(epoch_str) = epoch {
            if let Ok(epoch_dt) = chrono::DateTime::parse_from_rfc3339(epoch_str) {
                let result = epoch_dt + chrono::Duration::seconds(offset as i64);
                return result.to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
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

fn map_to_html_table(map: &IndexMap<Attribute, AttributeValue>) -> String {
    let mut html = String::new();
    html.push_str("<table>");
    for (key, value) in map {
        let value: serde_json::Value = value.clone().into();
        html.push_str(&format!("<tr><td>{key}</td><td>{value}</td></tr>"));
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
    use reearth_flow_geometry::types::point::Point3D;
    use reearth_flow_types::Geometry;

    fn make_feature_3d(lon: f64, lat: f64, height: f64) -> Feature {
        Feature {
            geometry: Geometry {
                epsg: Some(4326),
                value: GeometryValue::FlowGeometry3D(Geometry3D::Point(Point3D::new(
                    lon, lat, height,
                ))),
            },
            ..Default::default()
        }
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

    #[test]
    fn test_build_embedded_packet_timeseries() {
        let f = make_embedded_feature_with_timeseries();
        let packet = build_embedded_packet(&f).unwrap();

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
        let packet = build_embedded_packet(&f).unwrap();

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
}
