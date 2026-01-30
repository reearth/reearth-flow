use std::{collections::HashMap, sync::Arc};

use bytes::Bytes;
use reearth_flow_geometry::types::{
    geometry::{Geometry2D, Geometry3D},
    line_string::{LineString2D, LineString3D},
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

use super::reader::runner::get_content;
use crate::{errors::SourceError, file::reader::runner::FileReaderCommonParam};

#[derive(Debug, Clone, Default)]
pub(crate) struct CzmlReaderFactory;

impl SourceFactory for CzmlReaderFactory {
    fn name(&self) -> &str {
        "CzmlReader"
    }

    fn description(&self) -> &str {
        "Reads geographic features from CZML (Cesium Language) files for 3D visualization, with support for time-dynamic properties and timeseries data"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(CzmlReaderParam))
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
                SourceError::CzmlReaderFactory(format!("Failed to serialize `with` parameter: {e}"))
            })?;
            serde_json::from_value(value).map_err(|e| {
                SourceError::CzmlReaderFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(SourceError::CzmlReaderFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        let reader = CzmlReader { params };
        Ok(Box::new(reader))
    }
}

#[derive(Debug, Clone)]
pub(super) struct CzmlReader {
    pub(super) params: CzmlReaderParam,
}

/// # CzmlReader Parameters
///
/// Configuration for reading CZML files as geographic features.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(super) struct CzmlReaderParam {
    #[serde(flatten)]
    pub(super) common_property: FileReaderCommonParam,
    /// # Force 2D
    /// If true, forces all geometries to be 2D (ignoring Z values)
    #[serde(default)]
    pub(super) force_2d: bool,
    /// # Skip Document Packet
    /// If true, skips the document packet (first packet with version/clock info)
    #[serde(default = "default_skip_document")]
    pub(super) skip_document_packet: bool,
    /// # Time Sampling Strategy
    /// How to handle time-dynamic properties in CZML packets.
    /// Defaults to "preserveRaw" for lossless round-trip with CzmlWriter.
    #[serde(default)]
    pub(super) time_sampling: TimeSamplingStrategy,
}

fn default_skip_document() -> bool {
    true
}

/// Strategy for handling time-dynamic CZML properties.
#[derive(Serialize, Deserialize, Debug, Clone, Default, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(super) enum TimeSamplingStrategy {
    /// Extract all time-tagged samples as separate features, each with a
    /// `czml.timestamp` and `czml.timeOffset` attribute. Useful when you
    /// need per-sample processing in downstream actions.
    AllSamples,
    /// Keep the first sample only (static geometry). Use this for workflows
    /// that don't need timeseries data.
    FirstSampleOnly,
    /// Embed the full timeseries in one feature per entity. The feature
    /// geometry uses the first sample, `czml.timeseries` holds all position
    /// samples as a JSON array, and all other CZML packet properties (point,
    /// path, orientation, ellipsoid, etc.) are preserved as `czml.<key>`
    /// attributes for faithful round-trip through CzmlWriter.
    #[default]
    PreserveRaw,
}

#[async_trait::async_trait]
impl Source for CzmlReader {
    async fn initialize(&self, _ctx: NodeContext) {}

    fn name(&self) -> &str {
        "CzmlReader"
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
        read_czml(&content, &self.params, sender)
            .await
            .map_err(Into::<BoxedError>::into)
    }
}

async fn read_czml(
    content: &Bytes,
    params: &CzmlReaderParam,
    sender: Sender<(Port, IngestionMessage)>,
) -> Result<(), crate::errors::SourceError> {
    let text = String::from_utf8(content.to_vec())
        .map_err(|e| crate::errors::SourceError::CzmlReader(format!("Invalid UTF-8: {e}")))?;

    // Parse as JSON array of packets
    let packets: Vec<Value> = serde_json::from_str(&text).map_err(|e| {
        crate::errors::SourceError::CzmlReader(format!("Failed to parse CZML: {e}"))
    })?;

    for packet in packets {
        // Skip document packet if configured
        if params.skip_document_packet {
            if let Some(version) = packet.get("version") {
                if version.is_string() {
                    continue;
                }
            }
        }

        // Convert packet to feature(s) depending on time sampling strategy
        let features = packet_to_features(&packet, params)?;
        for feature in features {
            sender
                .send((
                    DEFAULT_PORT.clone(),
                    IngestionMessage::OperationEvent { feature },
                ))
                .await
                .map_err(|e| {
                    crate::errors::SourceError::CzmlReader(format!("Failed to send feature: {e}"))
                })?;
        }
    }

    Ok(())
}

/// Shared attributes extracted from a CZML packet (id, name, description, availability, parent).
fn extract_common_attributes(packet: &Value) -> indexmap::IndexMap<Attribute, AttributeValue> {
    let mut attributes = indexmap::IndexMap::new();

    if let Some(id) = packet.get("id").and_then(|v| v.as_str()) {
        attributes.insert(Attribute::new("id"), AttributeValue::String(id.to_string()));
    }
    if let Some(name) = packet.get("name").and_then(|v| v.as_str()) {
        attributes.insert(
            Attribute::new("name"),
            AttributeValue::String(name.to_string()),
        );
    }
    if let Some(desc) = packet.get("description") {
        let desc_str = if let Some(s) = desc.as_str() {
            s.to_string()
        } else {
            desc.to_string()
        };
        attributes.insert(
            Attribute::new("description"),
            AttributeValue::String(desc_str),
        );
    }
    if let Some(availability) = packet.get("availability") {
        let availability_str = if let Some(s) = availability.as_str() {
            s.to_string()
        } else {
            availability.to_string()
        };
        attributes.insert(
            Attribute::new("availability"),
            AttributeValue::String(availability_str),
        );
    }
    if let Some(parent) = packet.get("parent").and_then(|v| v.as_str()) {
        attributes.insert(
            Attribute::new("parent"),
            AttributeValue::String(parent.to_string()),
        );
    }

    attributes
}

/// Properties that are handled specially and should NOT be stored as raw
/// `czml.*` attributes.
const CZML_COMMON_PROPERTIES: &[&str] = &[
    "id",
    "name",
    "description",
    "availability",
    "parent",
    "position",
    "version",
    "clock",
];

/// Extract all non-common CZML packet properties as raw JSON `czml.<key>`
/// attributes. This preserves properties like `point`, `path`, `orientation`,
/// `ellipsoid`, `billboard`, `label`, `model`, `polyline`, `polygon`, etc. so
/// they can be round-tripped faithfully through the writer.
fn extract_extra_czml_properties(
    packet: &Value,
    attributes: &mut indexmap::IndexMap<Attribute, AttributeValue>,
) {
    let Some(obj) = packet.as_object() else {
        return;
    };
    for (key, value) in obj {
        if CZML_COMMON_PROPERTIES.contains(&key.as_str()) {
            continue;
        }
        // Store as raw JSON string under czml.<key>
        let json_str = serde_json::to_string(value).unwrap_or_default();
        attributes.insert(
            Attribute::new(format!("czml.{key}")),
            AttributeValue::String(json_str),
        );
    }
}

/// Convert a CZML packet into one or more features depending on the time
/// sampling strategy.
fn packet_to_features(
    packet: &Value,
    params: &CzmlReaderParam,
) -> Result<Vec<Feature>, crate::errors::SourceError> {
    let force_2d = params.force_2d;
    let base_attributes = extract_common_attributes(packet);

    // First try to extract time-dynamic position data
    if let Some(position_value) = packet.get("position") {
        if let Some(ts) = parse_time_tagged_position(position_value) {
            return build_timeseries_features(
                &ts,
                &base_attributes,
                force_2d,
                &params.time_sampling,
                packet,
            );
        }
    }

    // Fall back to static geometry extraction (existing logic)
    let geometry = extract_geometry(packet, force_2d)?;
    if let Some(geom) = geometry {
        let mut feature = Feature {
            geometry: geom,
            attributes: base_attributes,
            ..Default::default()
        };
        // Capture extra CZML properties for roundtrip
        extract_extra_czml_properties(packet, &mut feature.attributes);
        Ok(vec![feature])
    } else {
        Ok(vec![])
    }
}

#[derive(Debug, Clone)]
struct TimeSample {
    time_offset: f64,
    time_iso: Option<String>,
    lon: f64,
    lat: f64,
    height: f64,
}

#[derive(Debug, Clone)]
struct TimeTaggedPosition {
    epoch: Option<String>,
    interpolation_algorithm: Option<String>,
    interpolation_degree: Option<f64>,
    samples: Vec<TimeSample>,
}

/// Parse a CZML position value as time-tagged data (groups of 4: time, lon, lat, height).
/// Returns `None` for static positions (3 elements).
fn parse_time_tagged_position(value: &Value) -> Option<TimeTaggedPosition> {
    let obj = value.as_object()?;

    let coords_key = if obj.contains_key("cartographicDegrees") {
        "cartographicDegrees"
    } else if obj.contains_key("cartographicRadians") {
        "cartographicRadians"
    } else {
        return None;
    };

    let is_radians = coords_key == "cartographicRadians";

    let arr = obj.get(coords_key)?.as_array()?;

    if arr.len() <= 3 {
        return None;
    }

    let first = arr.first()?;
    let is_time_tagged = first.is_number() || first.is_string();
    if !is_time_tagged {
        return None;
    }

    if arr.len() == 3 && arr.iter().all(|v| v.is_number()) {
        return None;
    }

    if arr.len() % 4 != 0 {
        return None;
    }

    let epoch = obj
        .get("epoch")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let interpolation_algorithm = obj
        .get("interpolationAlgorithm")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let interpolation_degree = obj.get("interpolationDegree").and_then(|v| v.as_f64());

    let mut samples = Vec::with_capacity(arr.len() / 4);
    for chunk in arr.chunks_exact(4) {
        let (time_offset, time_iso) = parse_time_value(&chunk[0])?;
        let mut lon = chunk[1].as_f64()?;
        let mut lat = chunk[2].as_f64()?;
        let height = chunk[3].as_f64()?;

        if is_radians {
            lon = lon.to_degrees();
            lat = lat.to_degrees();
        }

        samples.push(TimeSample {
            time_offset,
            time_iso,
            lon,
            lat,
            height,
        });
    }

    Some(TimeTaggedPosition {
        epoch,
        interpolation_algorithm,
        interpolation_degree,
        samples,
    })
}

fn parse_time_value(value: &Value) -> Option<(f64, Option<String>)> {
    if let Some(n) = value.as_f64() {
        Some((n, None))
    } else {
        value.as_str().map(|s| (0.0, Some(s.to_string())))
    }
}

/// Build features from time-tagged position data according to the sampling strategy.
fn build_timeseries_features(
    ts: &TimeTaggedPosition,
    base_attributes: &indexmap::IndexMap<Attribute, AttributeValue>,
    force_2d: bool,
    strategy: &TimeSamplingStrategy,
    packet: &Value,
) -> Result<Vec<Feature>, crate::errors::SourceError> {
    match strategy {
        TimeSamplingStrategy::FirstSampleOnly => {
            if let Some(sample) = ts.samples.first() {
                let geom = point_from_sample(sample, force_2d);
                let mut feature = Feature {
                    geometry: geom,
                    attributes: base_attributes.clone(),
                    ..Default::default()
                };
                add_interpolation_attributes(&mut feature, ts);
                extract_extra_czml_properties(packet, &mut feature.attributes);
                Ok(vec![feature])
            } else {
                Ok(vec![])
            }
        }
        TimeSamplingStrategy::AllSamples => {
            let mut features = Vec::with_capacity(ts.samples.len());
            for sample in &ts.samples {
                let geom = point_from_sample(sample, force_2d);
                let mut feature = Feature {
                    geometry: geom,
                    attributes: base_attributes.clone(),
                    ..Default::default()
                };

                let timestamp = sample_timestamp(sample, ts.epoch.as_deref());
                feature.attributes.insert(
                    Attribute::new("czml.timestamp"),
                    AttributeValue::String(timestamp),
                );
                feature.attributes.insert(
                    Attribute::new("czml.timeOffset"),
                    AttributeValue::Number(
                        serde_json::Number::from_f64(sample.time_offset)
                            .unwrap_or_else(|| serde_json::Number::from(0)),
                    ),
                );
                add_interpolation_attributes(&mut feature, ts);
                features.push(feature);
            }
            Ok(features)
        }
        TimeSamplingStrategy::PreserveRaw => {
            let geom = if let Some(sample) = ts.samples.first() {
                point_from_sample(sample, force_2d)
            } else {
                return Ok(vec![]);
            };

            let mut feature = Feature {
                geometry: geom,
                attributes: base_attributes.clone(),
                ..Default::default()
            };

            let timeseries: Vec<Value> = ts
                .samples
                .iter()
                .map(|s| {
                    serde_json::json!({
                        "time": sample_timestamp(s, ts.epoch.as_deref()),
                        "timeOffset": s.time_offset,
                        "lon": s.lon,
                        "lat": s.lat,
                        "height": s.height,
                    })
                })
                .collect();

            feature.attributes.insert(
                Attribute::new("czml.timeseries"),
                AttributeValue::String(serde_json::to_string(&timeseries).unwrap_or_default()),
            );
            add_interpolation_attributes(&mut feature, ts);
            extract_extra_czml_properties(packet, &mut feature.attributes);
            Ok(vec![feature])
        }
    }
}

fn point_from_sample(sample: &TimeSample, force_2d: bool) -> Geometry {
    let value = if force_2d {
        GeometryValue::FlowGeometry2D(Geometry2D::Point(Point2D::from([sample.lon, sample.lat])))
    } else {
        GeometryValue::FlowGeometry3D(Geometry3D::Point(Point3D::from([
            sample.lon,
            sample.lat,
            sample.height,
        ])))
    };
    Geometry {
        epsg: Some(4326),
        value,
    }
}

fn sample_timestamp(sample: &TimeSample, epoch: Option<&str>) -> String {
    if let Some(iso) = &sample.time_iso {
        iso.clone()
    } else if let Some(epoch) = epoch {
        format!("{}+{}s", epoch, sample.time_offset)
    } else {
        format!("{}s", sample.time_offset)
    }
}

fn add_interpolation_attributes(feature: &mut Feature, ts: &TimeTaggedPosition) {
    if let Some(epoch) = &ts.epoch {
        feature.attributes.insert(
            Attribute::new("czml.epoch"),
            AttributeValue::String(epoch.clone()),
        );
    }
    if let Some(alg) = &ts.interpolation_algorithm {
        feature.attributes.insert(
            Attribute::new("czml.interpolationAlgorithm"),
            AttributeValue::String(alg.clone()),
        );
    }
    if let Some(deg) = ts.interpolation_degree {
        feature.attributes.insert(
            Attribute::new("czml.interpolationDegree"),
            AttributeValue::Number(
                serde_json::Number::from_f64(deg).unwrap_or_else(|| serde_json::Number::from(0)),
            ),
        );
    }
}

fn extract_geometry(
    packet: &Value,
    force_2d: bool,
) -> Result<Option<Geometry>, crate::errors::SourceError> {
    if let Some(polygon) = packet.get("polygon") {
        if let Some(positions) = polygon.get("positions") {
            if let Some(coords) = extract_cartographic_degrees(positions) {
                let holes = if let Some(holes_value) = polygon.get("holes") {
                    extract_polygon_holes(holes_value)
                } else {
                    vec![]
                };

                let geometry = convert_polygon_coords(&coords, holes, force_2d)?;
                return Ok(Some(Geometry {
                    epsg: Some(4326),
                    value: geometry,
                }));
            }
        }
    }

    if let Some(polyline) = packet.get("polyline") {
        if let Some(positions) = polyline.get("positions") {
            if let Some(coords) = extract_cartographic_degrees(positions) {
                let geometry = convert_line_coords(&coords, force_2d)?;
                return Ok(Some(Geometry {
                    epsg: Some(4326),
                    value: geometry,
                }));
            }
        }
    }

    if let Some(rectangle) = packet.get("rectangle") {
        if let Some(coordinates) = rectangle.get("coordinates") {
            if let Some(wsen) = extract_rectangle_bounds(coordinates) {
                let geometry = convert_rectangle_to_polygon(wsen, force_2d)?;
                return Ok(Some(Geometry {
                    epsg: Some(4326),
                    value: geometry,
                }));
            }
        }
    }

    if let Some(corridor) = packet.get("corridor") {
        if let Some(positions) = corridor.get("positions") {
            if let Some(coords) = extract_cartographic_degrees(positions) {
                let geometry = convert_line_coords(&coords, force_2d)?;
                return Ok(Some(Geometry {
                    epsg: Some(4326),
                    value: geometry,
                }));
            }
        }
    }

    if let Some(ellipse) = packet.get("ellipse") {
        if let Some(position) = packet.get("position") {
            if let Some(center_coords) = extract_cartographic_degrees(position) {
                if center_coords.len() >= 2 {
                    let semi_major = ellipse
                        .get("semiMajorAxis")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(100.0);
                    let semi_minor = ellipse
                        .get("semiMinorAxis")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(100.0);
                    let geometry = create_ellipse_polygon(
                        center_coords[0],
                        center_coords[1],
                        center_coords.get(2).copied().unwrap_or(0.0),
                        semi_major,
                        semi_minor,
                        force_2d,
                    )?;
                    return Ok(Some(Geometry {
                        epsg: Some(4326),
                        value: geometry,
                    }));
                }
            }
        }
    }

    if let Some(wall) = packet.get("wall") {
        if let Some(positions) = wall.get("positions") {
            if let Some(coords) = extract_cartographic_degrees(positions) {
                let geometry = convert_line_coords(&coords, force_2d)?;
                return Ok(Some(Geometry {
                    epsg: Some(4326),
                    value: geometry,
                }));
            }
        }
    }

    if let Some(position) = packet.get("position") {
        let has_other_geom = packet.get("polygon").is_some()
            || packet.get("polyline").is_some()
            || packet.get("rectangle").is_some()
            || packet.get("corridor").is_some()
            || packet.get("ellipse").is_some()
            || packet.get("wall").is_some();

        if !has_other_geom {
            if let Some(coords) = extract_cartographic_degrees(position) {
                if coords.len() >= 3 {
                    let point = if force_2d {
                        GeometryValue::FlowGeometry2D(Geometry2D::Point(Point2D::from([
                            coords[0], coords[1],
                        ])))
                    } else {
                        GeometryValue::FlowGeometry3D(Geometry3D::Point(Point3D::from([
                            coords[0], coords[1], coords[2],
                        ])))
                    };
                    return Ok(Some(Geometry {
                        epsg: Some(4326),
                        value: point,
                    }));
                }
            }
        }
    }

    Ok(None)
}

fn extract_cartographic_degrees(value: &Value) -> Option<Vec<f64>> {
    if let Some(arr) = value.as_array() {
        let coords: Option<Vec<f64>> = arr.iter().map(|v| v.as_f64()).collect();
        return coords;
    }

    if let Some(obj) = value.as_object() {
        if let Some(deg) = obj.get("cartographicDegrees") {
            if let Some(arr) = deg.as_array() {
                if arr.len() == 3 && arr.iter().all(|v| v.is_number()) {
                    let coords: Option<Vec<f64>> = arr.iter().map(|v| v.as_f64()).collect();
                    return coords;
                }
                // Time-tagged arrays are handled by parse_time_tagged_position
                if arr.len() > 3 && arr.len() % 4 == 0 {
                    return None;
                }
                let coords: Option<Vec<f64>> = arr.iter().map(|v| v.as_f64()).collect();
                return coords;
            }
        }

        if let Some(rad) = obj.get("cartographicRadians") {
            if let Some(arr) = rad.as_array() {
                if arr.len() == 3 && arr.iter().all(|v| v.is_number()) {
                    let coords: Option<Vec<f64>> = arr
                        .iter()
                        .enumerate()
                        .map(|(i, v)| {
                            v.as_f64()
                                .map(|val| if i % 3 < 2 { val.to_degrees() } else { val })
                        })
                        .collect();
                    return coords;
                }
                if arr.len() > 3 && arr.len() % 4 == 0 {
                    return None;
                }
                let coords: Option<Vec<f64>> = arr
                    .iter()
                    .enumerate()
                    .map(|(i, v)| {
                        v.as_f64()
                            .map(|val| if i % 3 < 2 { val.to_degrees() } else { val })
                    })
                    .collect();
                return coords;
            }
        }
    }

    None
}

fn extract_polygon_holes(value: &Value) -> Vec<Vec<f64>> {
    let mut holes = Vec::new();

    if let Some(obj) = value.as_object() {
        if let Some(deg) = obj.get("cartographicDegrees") {
            if let Some(arr) = deg.as_array() {
                for hole_value in arr {
                    if let Some(hole_coords) = hole_value.as_array() {
                        let coords: Option<Vec<f64>> =
                            hole_coords.iter().map(|v| v.as_f64()).collect();
                        if let Some(coords) = coords {
                            holes.push(coords);
                        }
                    }
                }
            }
        }
    }

    holes
}

fn convert_polygon_coords(
    coords: &[f64],
    holes: Vec<Vec<f64>>,
    force_2d: bool,
) -> Result<GeometryValue, crate::errors::SourceError> {
    if coords.len() < 9 {
        return Err(crate::errors::SourceError::CzmlReader(
            "Polygon must have at least 3 points".to_string(),
        ));
    }

    if force_2d {
        let points: Vec<_> = coords
            .chunks_exact(3)
            .map(|chunk| Point2D::from([chunk[0], chunk[1]]).0)
            .collect();
        let exterior = LineString2D::new(points);

        let interior_rings: Vec<LineString2D<f64>> = holes
            .into_iter()
            .filter_map(|hole_coords| {
                if hole_coords.len() >= 9 {
                    let hole_points: Vec<_> = hole_coords
                        .chunks_exact(3)
                        .map(|chunk| Point2D::from([chunk[0], chunk[1]]).0)
                        .collect();
                    Some(LineString2D::new(hole_points))
                } else {
                    None
                }
            })
            .collect();

        Ok(GeometryValue::FlowGeometry2D(Geometry2D::Polygon(
            Polygon2D::new(exterior, interior_rings),
        )))
    } else {
        let points: Vec<_> = coords
            .chunks_exact(3)
            .map(|chunk| Point3D::from([chunk[0], chunk[1], chunk[2]]).0)
            .collect();
        let exterior = LineString3D::new(points);

        let interior_rings: Vec<LineString3D<f64>> = holes
            .into_iter()
            .filter_map(|hole_coords| {
                if hole_coords.len() >= 9 {
                    let hole_points: Vec<_> = hole_coords
                        .chunks_exact(3)
                        .map(|chunk| Point3D::from([chunk[0], chunk[1], chunk[2]]).0)
                        .collect();
                    Some(LineString3D::new(hole_points))
                } else {
                    None
                }
            })
            .collect();

        Ok(GeometryValue::FlowGeometry3D(Geometry3D::Polygon(
            Polygon3D::new(exterior, interior_rings),
        )))
    }
}

fn convert_line_coords(
    coords: &[f64],
    force_2d: bool,
) -> Result<GeometryValue, crate::errors::SourceError> {
    if coords.len() < 6 {
        return Err(crate::errors::SourceError::CzmlReader(
            "Polyline must have at least 2 points".to_string(),
        ));
    }

    if force_2d {
        let points: Vec<_> = coords
            .chunks_exact(3)
            .map(|chunk| Point2D::from([chunk[0], chunk[1]]).0)
            .collect();
        Ok(GeometryValue::FlowGeometry2D(Geometry2D::LineString(
            LineString2D::new(points),
        )))
    } else {
        let points: Vec<_> = coords
            .chunks_exact(3)
            .map(|chunk| Point3D::from([chunk[0], chunk[1], chunk[2]]).0)
            .collect();
        Ok(GeometryValue::FlowGeometry3D(Geometry3D::LineString(
            LineString3D::new(points),
        )))
    }
}

fn extract_rectangle_bounds(value: &Value) -> Option<Vec<f64>> {
    if let Some(obj) = value.as_object() {
        if let Some(wsen) = obj.get("wsenDegrees") {
            if let Some(arr) = wsen.as_array() {
                if arr.len() >= 4 {
                    let bounds: Option<Vec<f64>> = arr.iter().map(|v| v.as_f64()).collect();
                    return bounds;
                }
            }
        }
        if let Some(wsen) = obj.get("wsenRadians") {
            if let Some(arr) = wsen.as_array() {
                if arr.len() >= 4 {
                    let bounds: Option<Vec<f64>> = arr
                        .iter()
                        .map(|v| v.as_f64().map(|val| val.to_degrees()))
                        .collect();
                    return bounds;
                }
            }
        }
    }
    None
}

fn convert_rectangle_to_polygon(
    wsen: Vec<f64>,
    force_2d: bool,
) -> Result<GeometryValue, crate::errors::SourceError> {
    if wsen.len() < 4 {
        return Err(crate::errors::SourceError::CzmlReader(
            "Rectangle must have west, south, east, north bounds".to_string(),
        ));
    }

    let west = wsen[0];
    let south = wsen[1];
    let east = wsen[2];
    let north = wsen[3];
    let height = wsen.get(4).copied().unwrap_or(0.0);

    if force_2d {
        let points = vec![
            Point2D::from([west, south]).0,
            Point2D::from([east, south]).0,
            Point2D::from([east, north]).0,
            Point2D::from([west, north]).0,
            Point2D::from([west, south]).0,
        ];
        let exterior = LineString2D::new(points);
        Ok(GeometryValue::FlowGeometry2D(Geometry2D::Polygon(
            Polygon2D::new(exterior, vec![]),
        )))
    } else {
        let points = vec![
            Point3D::from([west, south, height]).0,
            Point3D::from([east, south, height]).0,
            Point3D::from([east, north, height]).0,
            Point3D::from([west, north, height]).0,
            Point3D::from([west, south, height]).0,
        ];
        let exterior = LineString3D::new(points);
        Ok(GeometryValue::FlowGeometry3D(Geometry3D::Polygon(
            Polygon3D::new(exterior, vec![]),
        )))
    }
}

fn create_ellipse_polygon(
    center_lon: f64,
    center_lat: f64,
    height: f64,
    semi_major_meters: f64,
    semi_minor_meters: f64,
    force_2d: bool,
) -> Result<GeometryValue, crate::errors::SourceError> {
    const NUM_POINTS: usize = 32;

    let meters_per_degree_lon = 111_000.0 * (center_lat.to_radians().cos());
    let meters_per_degree_lat = 111_000.0;

    let semi_major_deg = semi_major_meters / meters_per_degree_lon;
    let semi_minor_deg = semi_minor_meters / meters_per_degree_lat;

    if force_2d {
        let mut points = Vec::with_capacity(NUM_POINTS + 1);
        for i in 0..=NUM_POINTS {
            let angle = (i as f64 * 2.0 * std::f64::consts::PI) / NUM_POINTS as f64;
            let x = center_lon + semi_major_deg * angle.cos();
            let y = center_lat + semi_minor_deg * angle.sin();
            points.push(Point2D::from([x, y]).0);
        }
        let exterior = LineString2D::new(points);
        Ok(GeometryValue::FlowGeometry2D(Geometry2D::Polygon(
            Polygon2D::new(exterior, vec![]),
        )))
    } else {
        let mut points = Vec::with_capacity(NUM_POINTS + 1);
        for i in 0..=NUM_POINTS {
            let angle = (i as f64 * 2.0 * std::f64::consts::PI) / NUM_POINTS as f64;
            let x = center_lon + semi_major_deg * angle.cos();
            let y = center_lat + semi_minor_deg * angle.sin();
            points.push(Point3D::from([x, y, height]).0);
        }
        let exterior = LineString3D::new(points);
        Ok(GeometryValue::FlowGeometry3D(Geometry3D::Polygon(
            Polygon3D::new(exterior, vec![]),
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_tagged_numeric_epoch() {
        let val = serde_json::json!({
            "epoch": "2024-01-01T00:00:00Z",
            "interpolationAlgorithm": "LAGRANGE",
            "interpolationDegree": 5,
            "cartographicDegrees": [
                0, -75.0, 40.0, 0.0,
                100, -76.0, 41.0, 100.0
            ]
        });
        let ts = parse_time_tagged_position(&val).unwrap();
        assert_eq!(ts.epoch.as_deref(), Some("2024-01-01T00:00:00Z"));
        assert_eq!(ts.interpolation_algorithm.as_deref(), Some("LAGRANGE"));
        assert_eq!(ts.interpolation_degree, Some(5.0));
        assert_eq!(ts.samples.len(), 2);
        assert_eq!(ts.samples[0].time_offset, 0.0);
        assert_eq!(ts.samples[0].lon, -75.0);
        assert_eq!(ts.samples[1].time_offset, 100.0);
        assert_eq!(ts.samples[1].lon, -76.0);
    }

    #[test]
    fn test_preserve_raw_strategy() {
        let ts = TimeTaggedPosition {
            epoch: Some("2024-01-01T00:00:00Z".to_string()),
            interpolation_algorithm: Some("LAGRANGE".to_string()),
            interpolation_degree: Some(5.0),
            samples: vec![
                TimeSample {
                    time_offset: 0.0,
                    time_iso: None,
                    lon: -75.0,
                    lat: 40.0,
                    height: 0.0,
                },
                TimeSample {
                    time_offset: 100.0,
                    time_iso: None,
                    lon: -76.0,
                    lat: 41.0,
                    height: 100.0,
                },
            ],
        };
        let attrs = indexmap::IndexMap::new();
        let packet = serde_json::json!({});
        let features = build_timeseries_features(
            &ts,
            &attrs,
            false,
            &TimeSamplingStrategy::PreserveRaw,
            &packet,
        )
        .unwrap();
        assert_eq!(features.len(), 1);
        let ts_attr = features[0]
            .attributes
            .get(&Attribute::new("czml.timeseries"))
            .unwrap();
        match ts_attr {
            AttributeValue::String(s) => {
                let parsed: Vec<Value> = serde_json::from_str(s).unwrap();
                assert_eq!(parsed.len(), 2);
                assert_eq!(parsed[0]["lon"], -75.0);
                assert_eq!(parsed[1]["lon"], -76.0);
            }
            _ => panic!("Expected string timeseries"),
        }
    }

    #[test]
    fn test_packet_to_features_static() {
        let packet = serde_json::json!({
            "id": "point1",
            "name": "Static Point",
            "position": {
                "cartographicDegrees": [-75.0, 40.0, 100.0]
            }
        });
        let params = CzmlReaderParam {
            common_property: FileReaderCommonParam {
                dataset: None,
                inline: None,
            },
            force_2d: false,
            skip_document_packet: true,
            time_sampling: TimeSamplingStrategy::AllSamples,
        };
        let features = packet_to_features(&packet, &params).unwrap();
        assert_eq!(features.len(), 1);
    }

    #[test]
    fn test_packet_to_features_timeseries() {
        let packet = serde_json::json!({
            "id": "vehicle1",
            "name": "Vehicle",
            "position": {
                "epoch": "2024-01-01T00:00:00Z",
                "cartographicDegrees": [
                    0, -75.0, 40.0, 0.0,
                    100, -76.0, 41.0, 100.0,
                    200, -77.0, 42.0, 200.0
                ]
            }
        });
        let params = CzmlReaderParam {
            common_property: FileReaderCommonParam {
                dataset: None,
                inline: None,
            },
            force_2d: false,
            skip_document_packet: true,
            time_sampling: TimeSamplingStrategy::AllSamples,
        };
        let features = packet_to_features(&packet, &params).unwrap();
        assert_eq!(features.len(), 3);
        for f in &features {
            assert_eq!(
                f.attributes.get(&Attribute::new("id")),
                Some(&AttributeValue::String("vehicle1".to_string()))
            );
        }
    }
}
