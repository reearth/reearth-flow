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
        "Reads geographic features from CZML (Cesium Language) files for 3D visualization"
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
}

fn default_skip_document() -> bool {
    true
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

        // Convert packet to feature
        if let Some(feature) = packet_to_feature(&packet, params.force_2d)? {
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

fn packet_to_feature(
    packet: &Value,
    force_2d: bool,
) -> Result<Option<Feature>, crate::errors::SourceError> {
    let mut feature = Feature::default();

    // Extract attributes from packet
    let mut attributes = indexmap::IndexMap::new();

    // Add ID if present
    if let Some(id) = packet.get("id").and_then(|v| v.as_str()) {
        attributes.insert(Attribute::new("id"), AttributeValue::String(id.to_string()));
    }

    // Add name if present
    if let Some(name) = packet.get("name").and_then(|v| v.as_str()) {
        attributes.insert(
            Attribute::new("name"),
            AttributeValue::String(name.to_string()),
        );
    }

    // Add description if present
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

    // Extract geometry from packet
    let geometry = extract_geometry(packet, force_2d)?;

    // Only create feature if we have geometry
    if let Some(geom) = geometry {
        feature.geometry = geom;
        feature.attributes = attributes;

        // Add availability as attribute if present
        if let Some(availability) = packet.get("availability") {
            let availability_str = if let Some(s) = availability.as_str() {
                s.to_string()
            } else {
                availability.to_string()
            };
            feature.attributes.insert(
                Attribute::new("availability"),
                AttributeValue::String(availability_str),
            );
        }

        // Add parent reference if present
        if let Some(parent) = packet.get("parent").and_then(|v| v.as_str()) {
            feature.attributes.insert(
                Attribute::new("parent"),
                AttributeValue::String(parent.to_string()),
            );
        }

        Ok(Some(feature))
    } else {
        Ok(None)
    }
}

fn extract_geometry(
    packet: &Value,
    force_2d: bool,
) -> Result<Option<Geometry>, crate::errors::SourceError> {
    // Try to extract point from position
    if let Some(position) = packet.get("position") {
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
                    epsg: Some(4326), // CZML uses WGS84
                    value: point,
                }));
            }
        }
    }

    // Try to extract polygon
    if let Some(polygon) = packet.get("polygon") {
        if let Some(positions) = polygon.get("positions") {
            if let Some(coords) = extract_cartographic_degrees(positions) {
                let geometry = convert_polygon_coords(&coords, force_2d)?;
                return Ok(Some(Geometry {
                    epsg: Some(4326),
                    value: geometry,
                }));
            }
        }
    }

    // Try to extract polyline
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

    Ok(None)
}

fn extract_cartographic_degrees(value: &Value) -> Option<Vec<f64>> {
    // Handle direct array of coordinates
    if let Some(arr) = value.as_array() {
        let coords: Option<Vec<f64>> = arr.iter().map(|v| v.as_f64()).collect();
        return coords;
    }

    // Handle object with cartographicDegrees property
    if let Some(obj) = value.as_object() {
        if let Some(deg) = obj.get("cartographicDegrees") {
            if let Some(arr) = deg.as_array() {
                let coords: Option<Vec<f64>> = arr.iter().map(|v| v.as_f64()).collect();
                return coords;
            }
        }

        // Handle cartographicRadians and convert to degrees
        if let Some(rad) = obj.get("cartographicRadians") {
            if let Some(arr) = rad.as_array() {
                let coords: Option<Vec<f64>> = arr
                    .iter()
                    .enumerate()
                    .map(|(i, v)| {
                        v.as_f64().map(|val| {
                            if i % 3 < 2 {
                                // lon/lat
                                val.to_degrees()
                            } else {
                                // height
                                val
                            }
                        })
                    })
                    .collect();
                return coords;
            }
        }
    }

    None
}

fn convert_polygon_coords(
    coords: &[f64],
    force_2d: bool,
) -> Result<GeometryValue, crate::errors::SourceError> {
    if coords.len() < 9 {
        // At least 3 points (triangle)
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
        Ok(GeometryValue::FlowGeometry2D(Geometry2D::Polygon(
            Polygon2D::new(exterior, vec![]),
        )))
    } else {
        let points: Vec<_> = coords
            .chunks_exact(3)
            .map(|chunk| Point3D::from([chunk[0], chunk[1], chunk[2]]).0)
            .collect();
        let exterior = LineString3D::new(points);
        Ok(GeometryValue::FlowGeometry3D(Geometry3D::Polygon(
            Polygon3D::new(exterior, vec![]),
        )))
    }
}

fn convert_line_coords(
    coords: &[f64],
    force_2d: bool,
) -> Result<GeometryValue, crate::errors::SourceError> {
    if coords.len() < 6 {
        // At least 2 points
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
