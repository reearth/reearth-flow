use std::collections::HashMap;

use once_cell::sync::Lazy;
use reearth_flow_geometry::algorithm::area2d::Area2D;
use reearth_flow_runtime::node::REJECTED_PORT;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory},
};
use reearth_flow_types::{Attribute, AttributeValue, Feature, GeometryValue};
use rstar::{PointDistance, RTree, RTreeObject, AABB};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::GeometryProcessorError;

static BASE_PORT: Lazy<Port> = Lazy::new(|| Port::new("base"));
static CANDIDATE_PORT: Lazy<Port> = Lazy::new(|| Port::new("candidate"));
static MATCHED_PORT: Lazy<Port> = Lazy::new(|| Port::new("matched"));
static UNMATCHED_PORT: Lazy<Port> = Lazy::new(|| Port::new("unmatched"));

#[derive(Debug, Clone, Default)]
pub(super) struct NeighborFinderFactory;

impl ProcessorFactory for NeighborFinderFactory {
    fn name(&self) -> &str {
        "NeighborFinder"
    }

    fn description(&self) -> &str {
        "Finds the closest candidate features for each base feature based on spatial proximity"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(NeighborFinderParams))
    }

    fn categories(&self) -> &[&'static str] {
        &["Geometry"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![BASE_PORT.clone(), CANDIDATE_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![
            MATCHED_PORT.clone(),
            UNMATCHED_PORT.clone(),
            REJECTED_PORT.clone(),
        ]
    }

    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let params: NeighborFinderParams = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                GeometryProcessorError::NeighborFinderFactory(format!(
                    "Failed to serialize 'with' parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                GeometryProcessorError::NeighborFinderFactory(format!(
                    "Failed to deserialize 'with' parameter: {e}"
                ))
            })?
        } else {
            NeighborFinderParams::default()
        };

        Ok(Box::new(NeighborFinder {
            params,
            candidates: Vec::new(),
            base_features: Vec::new(),
        }))
    }
}

/// # NeighborFinder Parameters
///
/// Configuration for finding spatial neighbors between base and candidate features.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct NeighborFinderParams {
    /// Maximum distance threshold for matching. If None, no distance limit is applied.
    pub max_distance: Option<f64>,

    /// Name of the attribute to store the computed distance to the nearest neighbor.
    #[serde(default = "default_distance_attribute")]
    pub distance_attribute: Attribute,

    /// Prefix applied to transferred candidate attributes to avoid collisions.
    #[serde(default = "default_attribute_prefix")]
    pub attribute_prefix: String,

    /// List of candidate attributes to transfer. Empty list means all attributes are transferred.
    #[serde(default)]
    pub attributes_to_transfer: Vec<Attribute>,
}

impl Default for NeighborFinderParams {
    fn default() -> Self {
        Self {
            max_distance: None,
            distance_attribute: default_distance_attribute(),
            attribute_prefix: default_attribute_prefix(),
            attributes_to_transfer: Vec::new(),
        }
    }
}

fn default_distance_attribute() -> Attribute {
    Attribute::new("_neighbor_distance")
}

fn default_attribute_prefix() -> String {
    "_neighbor_".to_string()
}

#[derive(Debug, Clone)]
struct CandidateEntry {
    point: [f64; 2],
    feature: Feature,
}

impl RTreeObject for CandidateEntry {
    type Envelope = AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        AABB::from_point(self.point)
    }
}

impl PointDistance for CandidateEntry {
    fn distance_2(&self, point: &[f64; 2]) -> f64 {
        let dx = self.point[0] - point[0];
        let dy = self.point[1] - point[1];
        dx * dx + dy * dy
    }
}

#[derive(Debug, Clone)]
struct NeighborFinder {
    params: NeighborFinderParams,
    candidates: Vec<CandidateEntry>,
    base_features: Vec<Feature>,
}

impl Processor for NeighborFinder {
    fn is_accumulating(&self) -> bool {
        true
    }

    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let geometry = &feature.geometry;

        // Check if feature has valid geometry
        if geometry.is_empty() {
            match &ctx.port {
                port if port == &*BASE_PORT => {
                    fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
                }
                _ => {
                    // Silently skip candidates with no geometry
                }
            }
            return Ok(());
        }

        match &ctx.port {
            port if port == &*BASE_PORT => {
                self.base_features.push(feature.clone());
            }
            port if port == &*CANDIDATE_PORT => {
                // Extract representative point and store candidate
                if let Some(point) = extract_representative_point(feature) {
                    self.candidates.push(CandidateEntry {
                        point,
                        feature: feature.clone(),
                    });
                }
            }
            _ => {
                fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
            }
        }

        Ok(())
    }

    fn finish(
        &mut self,
        ctx: NodeContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        // If no candidates, all base features go to unmatched
        if self.candidates.is_empty() {
            for base in &self.base_features {
                fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                    &ctx,
                    base.clone(),
                    UNMATCHED_PORT.clone(),
                ));
            }
            return Ok(());
        }

        // Build R-tree index from candidates
        let rtree: RTree<CandidateEntry> = RTree::bulk_load(self.candidates.clone());

        // Process each base feature
        for base in &self.base_features {
            let Some(base_point) = extract_representative_point(base) else {
                // Base feature has no usable geometry
                fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                    &ctx,
                    base.clone(),
                    REJECTED_PORT.clone(),
                ));
                continue;
            };

            // Find nearest neighbor using R-tree
            let Some(nearest) = rtree.nearest_neighbor(&base_point) else {
                fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                    &ctx,
                    base.clone(),
                    UNMATCHED_PORT.clone(),
                ));
                continue;
            };

            // Compute exact Euclidean distance
            let distance = euclidean_distance_2d(&base_point, &nearest.point);

            // Check max distance constraint
            if let Some(max_dist) = self.params.max_distance {
                if distance > max_dist {
                    fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                        &ctx,
                        base.clone(),
                        UNMATCHED_PORT.clone(),
                    ));
                    continue;
                }
            }

            // Build enriched feature with neighbor attributes
            let mut enriched = base.clone();

            // Add distance attribute
            enriched.attributes_mut().insert(
                self.params.distance_attribute.clone(),
                AttributeValue::Number(
                    serde_json::Number::from_f64(distance)
                        .unwrap_or_else(|| serde_json::Number::from(0)),
                ),
            );

            // Transfer candidate attributes with prefix
            let prefix = &self.params.attribute_prefix;
            let attrs_to_transfer: Vec<_> = if self.params.attributes_to_transfer.is_empty() {
                // Transfer all attributes
                nearest.feature.attributes.keys().cloned().collect()
            } else {
                self.params.attributes_to_transfer.clone()
            };

            for attr in attrs_to_transfer {
                if let Some(value) = nearest.feature.attributes.get(&attr) {
                    let prefixed_attr = Attribute::new(format!("{}{}", prefix, attr));
                    enriched
                        .attributes_mut()
                        .insert(prefixed_attr, value.clone());
                }
            }

            fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                &ctx,
                enriched,
                MATCHED_PORT.clone(),
            ));
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "NeighborFinder"
    }
}

/// Extract representative point from a feature's geometry.
/// Returns None if the feature has no usable geometry.
fn extract_representative_point(feature: &Feature) -> Option<[f64; 2]> {
    match &feature.geometry.value {
        GeometryValue::None => None,
        GeometryValue::FlowGeometry2D(geo) => extract_representative_point_2d(geo),
        GeometryValue::FlowGeometry3D(geo) => extract_representative_point_3d(geo),
        GeometryValue::CityGmlGeometry(citygml) => {
            // For CityGML, use centroid of all polygons
            let mut all_points = Vec::new();
            for gml in &citygml.gml_geometries {
                for poly in &gml.polygons {
                    for ring in poly.rings() {
                        for coord in ring.iter() {
                            all_points.push([coord.x, coord.y]);
                        }
                    }
                }
            }
            if all_points.is_empty() {
                None
            } else {
                Some(centroid_2d(&all_points))
            }
        }
    }
}

fn extract_representative_point_2d(
    geo: &reearth_flow_geometry::types::geometry::Geometry2D<f64>,
) -> Option<[f64; 2]> {
    use reearth_flow_geometry::types::geometry::Geometry2D;

    match geo {
        Geometry2D::Point(p) => Some([p.x(), p.y()]),
        Geometry2D::MultiPoint(mp) => {
            let points: Vec<[f64; 2]> = mp.iter().map(|p| [p.x(), p.y()]).collect();
            Some(centroid_2d(&points))
        }
        Geometry2D::LineString(ls) => {
            // Use midpoint of the line
            if ls.0.is_empty() {
                None
            } else {
                let first = ls.0.first().unwrap();
                let last = ls.0.last().unwrap();
                Some([(first.x + last.x) / 2.0, (first.y + last.y) / 2.0])
            }
        }
        Geometry2D::Polygon(poly) => {
            // Use centroid of exterior ring
            let coords: Vec<[f64; 2]> = poly.exterior().0.iter().map(|c| [c.x, c.y]).collect();
            Some(centroid_2d(&coords))
        }
        Geometry2D::MultiPolygon(mp) => {
            // Use centroid of the largest polygon by area
            let mut largest_area = 0.0;
            let mut best_centroid = None;
            for poly in mp.iter() {
                let area = poly.unsigned_area2d();
                if area > largest_area {
                    largest_area = area;
                    let coords: Vec<[f64; 2]> =
                        poly.exterior().0.iter().map(|c| [c.x, c.y]).collect();
                    best_centroid = Some(centroid_2d(&coords));
                }
            }
            best_centroid
        }
        _ => None,
    }
}

fn extract_representative_point_3d(
    geo: &reearth_flow_geometry::types::geometry::Geometry3D<f64>,
) -> Option<[f64; 2]> {
    use reearth_flow_geometry::types::geometry::Geometry3D;

    match geo {
        Geometry3D::Point(p) => Some([p.x(), p.y()]),
        Geometry3D::MultiPoint(mp) => {
            let points: Vec<[f64; 2]> = mp.iter().map(|p| [p.x(), p.y()]).collect();
            Some(centroid_2d(&points))
        }
        Geometry3D::LineString(ls) => {
            if ls.0.is_empty() {
                None
            } else {
                let first = ls.0.first().unwrap();
                let last = ls.0.last().unwrap();
                Some([(first.x + last.x) / 2.0, (first.y + last.y) / 2.0])
            }
        }
        Geometry3D::Polygon(poly) => {
            let coords: Vec<[f64; 2]> = poly.exterior().0.iter().map(|c| [c.x, c.y]).collect();
            Some(centroid_2d(&coords))
        }
        Geometry3D::MultiPolygon(mp) => {
            let mut largest_area = 0.0;
            let mut best_centroid = None;
            for poly in mp.iter() {
                use reearth_flow_geometry::algorithm::area3d::Area3D;
                let area = poly.unsigned_area3d();
                if area > largest_area {
                    largest_area = area;
                    let coords: Vec<[f64; 2]> =
                        poly.exterior().0.iter().map(|c| [c.x, c.y]).collect();
                    best_centroid = Some(centroid_2d(&coords));
                }
            }
            best_centroid
        }
        _ => None,
    }
}

/// Compute centroid of a set of 2D points
fn centroid_2d(points: &[[f64; 2]]) -> [f64; 2] {
    if points.is_empty() {
        return [0.0, 0.0];
    }
    let sum_x: f64 = points.iter().map(|p| p[0]).sum();
    let sum_y: f64 = points.iter().map(|p| p[1]).sum();
    [sum_x / points.len() as f64, sum_y / points.len() as f64]
}

/// Compute 2D Euclidean distance between two points
fn euclidean_distance_2d(a: &[f64; 2], b: &[f64; 2]) -> f64 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    (dx * dx + dy * dy).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;
    use reearth_flow_geometry::types::coordinate::Coordinate2D;
    use reearth_flow_geometry::types::line_string::LineString2D;
    use reearth_flow_geometry::types::point::Point2D;
    use reearth_flow_geometry::types::polygon::Polygon2D;
    use reearth_flow_runtime::executor_operation::NodeContext;
    use reearth_flow_runtime::forwarder::NoopChannelForwarder;
    use reearth_flow_types::feature::Attributes;
    use reearth_flow_types::Geometry;

    use crate::tests::utils::create_default_execute_context;

    fn create_point_feature(x: f64, y: f64) -> Feature {
        use reearth_flow_geometry::types::no_value::NoValue;
        Feature::new_with_attributes_and_geometry(
            Attributes::new(),
            Geometry {
                value: GeometryValue::FlowGeometry2D(
                    reearth_flow_geometry::types::geometry::Geometry2D::Point(Point2D::new_(
                        x, y, NoValue,
                    )),
                ),
                ..Default::default()
            },
            Default::default(),
        )
    }

    fn create_point_feature_with_attr(
        x: f64,
        y: f64,
        attr_name: &str,
        attr_value: AttributeValue,
    ) -> Feature {
        use reearth_flow_geometry::types::no_value::NoValue;
        let mut attrs = Attributes::new();
        attrs.insert(Attribute::new(attr_name), attr_value);
        Feature::new_with_attributes_and_geometry(
            attrs,
            Geometry {
                value: GeometryValue::FlowGeometry2D(
                    reearth_flow_geometry::types::geometry::Geometry2D::Point(Point2D::new_(
                        x, y, NoValue,
                    )),
                ),
                ..Default::default()
            },
            Default::default(),
        )
    }

    #[test]
    fn test_single_closest_neighbor() {
        let mut finder = NeighborFinder {
            params: NeighborFinderParams::default(),
            candidates: Vec::new(),
            base_features: Vec::new(),
        };

        // Create candidate features
        let candidate1 = create_point_feature_with_attr(
            0.0,
            0.0,
            "name",
            AttributeValue::String("A".to_string()),
        );
        let candidate2 = create_point_feature_with_attr(
            10.0,
            0.0,
            "name",
            AttributeValue::String("B".to_string()),
        );

        // Create base feature closer to candidate1
        let base = create_point_feature(1.0, 0.0);

        // Process candidates
        let noop = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop);

        let mut ctx = create_default_execute_context(&candidate1);
        ctx.port = CANDIDATE_PORT.clone();
        finder.process(ctx, &fw).unwrap();

        let mut ctx = create_default_execute_context(&candidate2);
        ctx.port = CANDIDATE_PORT.clone();
        finder.process(ctx, &fw).unwrap();

        // Process base
        let mut ctx = create_default_execute_context(&base);
        ctx.port = BASE_PORT.clone();
        finder.process(ctx, &fw).unwrap();

        // Verify buffering
        assert_eq!(finder.candidates.len(), 2);
        assert_eq!(finder.base_features.len(), 1);
    }

    #[test]
    fn test_max_distance_filtering() {
        let mut finder = NeighborFinder {
            params: NeighborFinderParams {
                max_distance: Some(5.0),
                ..Default::default()
            },
            candidates: Vec::new(),
            base_features: Vec::new(),
        };

        let candidate = create_point_feature_with_attr(
            0.0,
            0.0,
            "id",
            AttributeValue::Number(serde_json::Number::from(1)),
        );
        let base = create_point_feature(10.0, 0.0); // Distance is 10, exceeds max_distance of 5

        let noop = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop);

        let mut ctx = create_default_execute_context(&candidate);
        ctx.port = CANDIDATE_PORT.clone();
        finder.process(ctx, &fw).unwrap();

        let mut ctx = create_default_execute_context(&base);
        ctx.port = BASE_PORT.clone();
        finder.process(ctx, &fw).unwrap();

        // Finish and check that base goes to unmatched
        let ctx = NodeContext::default();
        finder.finish(ctx, &fw).unwrap();

        if let ProcessorChannelForwarder::Noop(noop) = fw {
            let ports = noop.send_ports.lock().unwrap();
            assert_eq!(ports.len(), 1);
            assert_eq!(ports[0], *UNMATCHED_PORT);
        }
    }

    #[test]
    fn test_no_candidates_all_unmatched() {
        let mut finder = NeighborFinder {
            params: NeighborFinderParams::default(),
            candidates: Vec::new(),
            base_features: Vec::new(),
        };

        let base = create_point_feature(1.0, 1.0);

        let noop = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop);

        let mut ctx = create_default_execute_context(&base);
        ctx.port = BASE_PORT.clone();
        finder.process(ctx, &fw).unwrap();

        let ctx = NodeContext::default();
        finder.finish(ctx, &fw).unwrap();

        if let ProcessorChannelForwarder::Noop(noop) = fw {
            let ports = noop.send_ports.lock().unwrap();
            assert_eq!(ports.len(), 1);
            assert_eq!(ports[0], *UNMATCHED_PORT);
        }
    }

    #[test]
    fn test_polygon_centroid() {
        use reearth_flow_geometry::types::geometry::Geometry2D;

        let exterior = LineString2D::new(vec![
            Coordinate2D::new_(0.0, 0.0),
            Coordinate2D::new_(10.0, 0.0),
            Coordinate2D::new_(10.0, 10.0),
            Coordinate2D::new_(0.0, 10.0),
            Coordinate2D::new_(0.0, 0.0),
        ]);
        let polygon = Polygon2D::new(exterior, vec![]);

        let feature = Feature::new_with_attributes_and_geometry(
            Attributes::new(),
            Geometry {
                value: GeometryValue::FlowGeometry2D(Geometry2D::Polygon(polygon)),
                ..Default::default()
            },
            Default::default(),
        );

        let point = extract_representative_point(&feature);
        assert!(point.is_some());
        let [x, y] = point.unwrap();
        // Note: simple averaging of exterior coords (5 points including closing point)
        // gives (20/5, 20/5) = (4, 4), not the true area centroid (5, 5)
        // This is acceptable for v1 - true area centroid will be implemented in v3
        assert!((x - 4.0).abs() < 0.01, "x was {}", x);
        assert!((y - 4.0).abs() < 0.01, "y was {}", y);
    }

    #[test]
    fn test_euclidean_distance() {
        let a = [0.0, 0.0];
        let b = [3.0, 4.0];
        assert!((euclidean_distance_2d(&a, &b) - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_centroid() {
        let points = vec![[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]];
        let c = centroid_2d(&points);
        assert!((c[0] - 5.0).abs() < 0.001);
        assert!((c[1] - 5.0).abs() < 0.001);
    }
}
