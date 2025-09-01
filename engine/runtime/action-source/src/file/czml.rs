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
                        epsg: Some(4326), // CZML uses WGS84
                        value: point,
                    }));
                }
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

fn extract_polygon_holes(value: &Value) -> Vec<Vec<f64>> {
    let mut holes = Vec::new();

    // Check if it's an object with cartographicDegrees
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

fn extract_rectangle_bounds(value: &Value) -> Option<Vec<f64>> {
    // Handle wsenDegrees format [west, south, east, north]
    if let Some(obj) = value.as_object() {
        if let Some(wsen) = obj.get("wsenDegrees") {
            if let Some(arr) = wsen.as_array() {
                if arr.len() >= 4 {
                    let bounds: Option<Vec<f64>> = arr.iter().map(|v| v.as_f64()).collect();
                    return bounds;
                }
            }
        }
        // Handle wsenRadians and convert to degrees
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
            Point2D::from([west, south]).0, // Close the polygon
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
            Point3D::from([west, south, height]).0, // Close the polygon
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
    // Approximate ellipse with polygon (32 points for smooth curve)
    const NUM_POINTS: usize = 32;

    // Very rough approximation: 1 degree ≈ 111,000 meters at equator
    // This should use proper geodesic calculations for accuracy
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
