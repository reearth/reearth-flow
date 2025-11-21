//! Unshared Edge Detector for PLATEAU triangular meshes
//!
//! # Coordinate System Requirements
//!
//! **IMPORTANT**: This processor expects input geometries in a **projected coordinate system**
//! with units in **meters** (e.g., Japan Plane Rectangular CS: EPSG:6669-6687).
//!
//! - The `tolerance` parameter is interpreted as meters
//! - Distance calculations assume Cartesian (flat) geometry
//! - If input is in geographic coordinates (latitude/longitude), results will be incorrect
//!
//! **Typical workflow setup**:
//! 1. CityGML Reader (EPSG:6697 or other geographic CRS)
//! 2. **HorizontalReprojector** (convert to projected CRS like EPSG:6670)
//! 3. UnsharedEdgeDetector (this action)

use std::collections::{HashMap, HashSet};

use once_cell::sync::Lazy;
use reearth_flow_geometry::types::coordinate::Coordinate;
use reearth_flow_geometry::types::coordnum::CoordNum;
use reearth_flow_geometry::types::geometry::Geometry2D;
use reearth_flow_geometry::types::line_string::LineString2D;
use reearth_flow_geometry::types::polygon::Polygon2D;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, Feature, GeometryValue};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::PlateauProcessorError;
use crate::plateau4::face_extractor::{ATTR_IS_INCORRECT_NUM_VERTICES, ATTR_IS_NOT_CLOSED};

/// Fixed-point scale factor for coordinate conversion
///
/// Converts floating-point meter coordinates to integer fixed-point representation
/// for exact hash-based edge matching and comparison.
///
/// With scale factor of 1,000,000:
/// - 1 meter = 1,000,000 units (micrometer precision)
/// - Example: -27891.653215 m → -27891653215 (integer)
///
/// **Assumption**: Input coordinates are in meters (projected coordinate system)
const FIXED_POINT_SCALE: f64 = 1_000_000.0;

pub static UNSHARED_PORT: Lazy<Port> = Lazy::new(|| Port::new("unshared"));

#[derive(Debug, Clone, Default)]
pub struct UnsharedEdgeDetectorFactory;

impl ProcessorFactory for UnsharedEdgeDetectorFactory {
    fn name(&self) -> &str {
        "PLATEAU4.UnsharedEdgeDetector"
    }

    fn description(&self) -> &str {
        "Detect unshared edges in triangular meshes - edges that appear only once. \
         REQUIRES: Input geometries must be in a projected coordinate system (meters). \
         Use HorizontalReprojector before this action if input is in geographic coordinates."
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(UnsharedEdgeDetectorParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["PLATEAU"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![UNSHARED_PORT.clone()]
    }

    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let param: UnsharedEdgeDetectorParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                PlateauProcessorError::UnsharedEdgeDetectorFactory(format!(
                    "Failed to serialize 'with' parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                PlateauProcessorError::UnsharedEdgeDetectorFactory(format!(
                    "Failed to deserialize 'with' parameter: {e}"
                ))
            })?
        } else {
            UnsharedEdgeDetectorParam::default()
        };

        Ok(Box::new(UnsharedEdgeDetector {
            tolerance: param.tolerance,
            features: Vec::new(),
        }))
    }
}

/// # UnsharedEdgeDetector Parameters
/// Configure unshared edge detection behavior
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct UnsharedEdgeDetectorParam {
    /// Tolerance for edge matching in meters (default: 0.1)
    /// Edges within this distance are considered the same edge
    #[serde(default = "default_tolerance")]
    pub tolerance: f64,
}

impl Default for UnsharedEdgeDetectorParam {
    fn default() -> Self {
        Self {
            tolerance: default_tolerance(),
        }
    }
}

fn default_tolerance() -> f64 {
    0.1 // 10 cm tolerance
}

#[derive(Debug, Clone)]
pub struct UnsharedEdgeDetector {
    tolerance: f64,
    features: Vec<Feature>,
}

impl Processor for UnsharedEdgeDetector {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        _fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        // Skip features with validation errors
        // These triangles should not participate in unshared edge detection
        if !self.has_validation_error(&ctx.feature) {
            self.features.push(ctx.feature.clone());
        }
        Ok(())
    }

    fn finish(&self, ctx: NodeContext, fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        // Extract all edges from all polygons
        let all_edges = extract_all_edges(&self.features);

        // Group edges by distance-based matching
        // For each edge, find all edges that match within tolerance
        let mut matched_indices: HashSet<usize> = HashSet::new();
        let mut edge_groups: Vec<Vec<usize>> = Vec::new();

        for i in 0..all_edges.len() {
            if matched_indices.contains(&i) {
                continue; // Already in a group
            }

            let mut group = vec![i];
            matched_indices.insert(i);

            // Find all edges that match this edge within tolerance
            for j in (i + 1)..all_edges.len() {
                if matched_indices.contains(&j) {
                    continue;
                }
                if all_edges[i].matches(&all_edges[j], self.tolerance) {
                    group.push(j);
                    matched_indices.insert(j);
                }
            }

            edge_groups.push(group);
        }

        // Find unshared edges: groups with multiple edges that have different exact coordinates
        // A group with multiple edges indicates edges within tolerance
        // If their exact coordinates differ, it's a micro-gap (unshared edge)
        let mut unshared_edges = Vec::new();
        for group in edge_groups {
            if group.len() > 1 {
                // Check if all edges have identical coordinates
                let first_edge = &all_edges[group[0]];
                let all_identical = group.iter().all(|&idx| {
                    let edge = &all_edges[idx];
                    edge.start == first_edge.start && edge.end == first_edge.end
                });

                if !all_identical {
                    // Edges have different exact coordinates = micro-gap
                    for &idx in &group {
                        unshared_edges.push(all_edges[idx].clone());
                    }
                }
            }
        }

        // Output each unshared edge as a LineString feature
        for edge in unshared_edges {
            let mut edge_feature = Feature::new();
            // Convert fixed-point back to floating point
            let start_x = edge.start.x as f64 / FIXED_POINT_SCALE;
            let start_y = edge.start.y as f64 / FIXED_POINT_SCALE;
            let end_x = edge.end.x as f64 / FIXED_POINT_SCALE;
            let end_y = edge.end.y as f64 / FIXED_POINT_SCALE;

            let line_coords = vec![
                Coordinate::new_(start_x, start_y),
                Coordinate::new_(end_x, end_y),
            ];
            let line = LineString2D::new(line_coords);
            edge_feature.geometry.value =
                GeometryValue::FlowGeometry2D(Geometry2D::LineString(line));

            // Copy source feature attributes (like udxDirs, _file_index)
            if let Some(source_feature) = self.features.first() {
                for (key, value) in &source_feature.attributes {
                    edge_feature.attributes.insert(key.clone(), value.clone());
                }
            }

            fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                &ctx,
                edge_feature,
                UNSHARED_PORT.clone(),
            ));
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "UnsharedEdgeDetector"
    }
}

impl UnsharedEdgeDetector {
    /// Check if a feature has any validation error flags from FaceExtractor
    /// Triangles with these errors should not participate in unshared edge detection
    ///
    /// Note: Wrong orientation is NOT excluded because such triangles can still
    /// contribute to unshared edge detection. Only structural errors (incorrect
    /// vertex count, not closed) prevent edge extraction.
    fn has_validation_error(&self, feature: &Feature) -> bool {
        // Check for incorrect vertex count
        if let Some(AttributeValue::Number(n)) = feature
            .attributes
            .get(&Attribute::new(ATTR_IS_INCORRECT_NUM_VERTICES))
        {
            if n.as_f64().is_some_and(|v| v != 0.0) {
                return true;
            }
        }

        // Check for not closed
        if let Some(AttributeValue::Number(n)) =
            feature.attributes.get(&Attribute::new(ATTR_IS_NOT_CLOSED))
        {
            if n.as_f64().is_some_and(|v| v != 0.0) {
                return true;
            }
        }

        // Note: Wrong orientation is NOT a structural error - triangles with
        // wrong orientation can still participate in unshared edge detection

        false
    }
}

/// Represents an edge with fixed-point coordinates for exact comparison
#[derive(Debug, Clone)]
struct Edge {
    start: FixedPoint,
    end: FixedPoint,
}

/// Fixed-point representation of a coordinate for exact comparison
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FixedPoint {
    x: i64, // Fixed-point: actual_x * FIXED_POINT_SCALE
    y: i64, // Fixed-point: actual_y * FIXED_POINT_SCALE
}

impl FixedPoint {
    fn from_coordinate<Z: CoordNum>(coord: &Coordinate<f64, Z>) -> Self {
        Self {
            x: (coord.x * FIXED_POINT_SCALE).round() as i64,
            y: (coord.y * FIXED_POINT_SCALE).round() as i64,
        }
    }

    /// Check if this point is within tolerance of another point
    fn within_tolerance(&self, other: &Self, tolerance: f64) -> bool {
        let tolerance_fixed = (tolerance * FIXED_POINT_SCALE) as i64;
        let dx = (self.x - other.x).abs();
        let dy = (self.y - other.y).abs();
        dx <= tolerance_fixed && dy <= tolerance_fixed
    }
}

impl Edge {
    fn new(p1: FixedPoint, p2: FixedPoint) -> Self {
        // Normalize edge direction so (A,B) and (B,A) are the same
        if (p1.x, p1.y) < (p2.x, p2.y) {
            Self { start: p1, end: p2 }
        } else {
            Self { start: p2, end: p1 }
        }
    }

    /// Check if this edge matches another edge within tolerance
    fn matches(&self, other: &Self, tolerance: f64) -> bool {
        self.start.within_tolerance(&other.start, tolerance)
            && self.end.within_tolerance(&other.end, tolerance)
    }
}

/// Extract all edges from all polygon features
fn extract_all_edges(features: &[Feature]) -> Vec<Edge> {
    let mut edges = Vec::new();

    for feature in features {
        if let Some(geom_2d) = extract_geometry_2d(&feature.geometry) {
            match geom_2d {
                Geometry2D::Polygon(poly) => {
                    edges.extend(extract_polygon_edges(poly));
                }
                Geometry2D::MultiPolygon(mpoly) => {
                    for poly in mpoly.iter() {
                        edges.extend(extract_polygon_edges(poly));
                    }
                }
                _ => {}
            }
        }
    }

    edges
}

/// Extract edges from a single polygon
fn extract_polygon_edges(polygon: &Polygon2D<f64>) -> Vec<Edge> {
    let mut edges = Vec::new();
    let exterior = polygon.exterior();

    // Extract edges from exterior ring
    let coords: Vec<_> = exterior.coords().collect();
    for window in coords.windows(2) {
        let p1 = FixedPoint::from_coordinate(window[0]);
        let p2 = FixedPoint::from_coordinate(window[1]);
        if p1 != p2 {
            // Skip degenerate edges
            edges.push(Edge::new(p1, p2));
        }
    }

    edges
}

/// Extract Geometry2D from GeometryValue
fn extract_geometry_2d(geometry: &reearth_flow_types::Geometry) -> Option<&Geometry2D<f64>> {
    match &geometry.value {
        GeometryValue::FlowGeometry2D(geom) => Some(geom),
        _ => None,
    }
}
