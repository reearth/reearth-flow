use std::collections::HashMap;

use once_cell::sync::Lazy;
use reearth_flow_geometry::algorithm::relate::Relate;
use reearth_flow_geometry::types::geometry::{Geometry2D, Geometry3D};
use reearth_flow_runtime::node::REJECTED_PORT;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory},
};
use reearth_flow_types::{Attribute, AttributeValue, CityGmlGeometry, Feature, GeometryValue};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

static FILTER_PORT: Lazy<Port> = Lazy::new(|| Port::new("filter"));
static CANDIDATE_PORT: Lazy<Port> = Lazy::new(|| Port::new("candidate"));
static PASSED_PORT: Lazy<Port> = Lazy::new(|| Port::new("passed"));
static FAILED_PORT: Lazy<Port> = Lazy::new(|| Port::new("failed"));

#[derive(Debug, Clone, Default)]
pub(super) struct SpatialFilterFactory;

impl ProcessorFactory for SpatialFilterFactory {
    fn name(&self) -> &str {
        "SpatialFilter"
    }

    fn description(&self) -> &str {
        "Filter Features by Spatial Relationship"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(SpatialFilterParams))
    }

    fn categories(&self) -> &[&'static str] {
        &["Geometry"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![FILTER_PORT.clone(), CANDIDATE_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![
            PASSED_PORT.clone(),
            FAILED_PORT.clone(),
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
        let params: SpatialFilterParams = if let Some(with) = with {
            let value: Value = serde_json::to_value(with)?;
            serde_json::from_value(value)?
        } else {
            SpatialFilterParams::default()
        };

        Ok(Box::new(SpatialFilter {
            params,
            filters: Vec::new(),
            candidates: Vec::new(),
        }))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SpatialFilterParams {
    /// # Spatial Predicate
    /// The spatial relationship to test between filter and candidate geometries
    #[serde(default)]
    pub predicate: SpatialPredicate,

    /// # Pass on Multiple Matches
    /// If true, pass if ANY filter matches (OR logic). If false, pass only if ALL filters match (AND logic).
    #[serde(default = "default_pass_on_multiple")]
    pub pass_on_multiple_matches: bool,

    /// # Output Match Count Attribute
    /// Optional attribute name to store the number of matching filters
    #[serde(default)]
    pub output_match_count_attribute: Option<Attribute>,
}

fn default_pass_on_multiple() -> bool {
    true
}

impl Default for SpatialFilterParams {
    fn default() -> Self {
        Self {
            predicate: SpatialPredicate::Intersects,
            pass_on_multiple_matches: true,
            output_match_count_attribute: None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum SpatialPredicate {
    /// Filter geometry completely contains candidate
    Contains,
    /// Candidate completely within filter geometry
    Within,
    /// Geometries have any intersection
    Intersects,
    /// Geometries have no spatial relationship
    Disjoint,
    /// Geometries touch at boundaries but don't overlap
    Touches,
    /// Geometries cross each other
    Crosses,
    /// Geometries overlap partially
    Overlaps,
    /// Candidate is covered by filter geometry
    CoveredBy,
    /// Filter geometry covers candidate
    Covers,
}

impl Default for SpatialPredicate {
    fn default() -> Self {
        SpatialPredicate::Intersects
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct SpatialFilter {
    params: SpatialFilterParams,
    filters: Vec<Feature>,
    candidates: Vec<Feature>,
}

impl Processor for SpatialFilter {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let geometry = &feature.geometry;

        if geometry.is_empty() {
            fw.send(ctx.new_with_feature_and_port(ctx.feature.clone(), REJECTED_PORT.clone()));
            return Ok(());
        }

        match &geometry.value {
            GeometryValue::None => {
                fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
            }
            GeometryValue::FlowGeometry2D(_) | GeometryValue::FlowGeometry3D(_) => {
                match &ctx.port {
                    port if port == &*FILTER_PORT => self.filters.push(feature.clone()),
                    port if port == &*CANDIDATE_PORT => self.candidates.push(feature.clone()),
                    _ => {
                        fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
                    }
                }
            }
            GeometryValue::CityGmlGeometry(_) => match &ctx.port {
                port if port == &*FILTER_PORT => self.filters.push(feature.clone()),
                port if port == &*CANDIDATE_PORT => self.candidates.push(feature.clone()),
                _ => {
                    fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
                }
            },
        }
        Ok(())
    }

    fn finish(&self, ctx: NodeContext, fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        if self.filters.is_empty() {
            // No filters provided, reject all candidates
            for candidate in &self.candidates {
                fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                    &ctx,
                    candidate.clone(),
                    REJECTED_PORT.clone(),
                ));
            }
            return Ok(());
        }

        // Process each candidate against all filters
        for candidate in &self.candidates {
            match &candidate.geometry.value {
                GeometryValue::FlowGeometry2D(candidate_geo) => {
                    let result = test_2d_geometry(
                        candidate_geo,
                        &self.filters,
                        &self.params,
                    );
                    forward_result(result, candidate, &self.params, &ctx, fw);
                }
                GeometryValue::FlowGeometry3D(candidate_geo) => {
                    let result = test_3d_geometry(
                        candidate_geo,
                        &self.filters,
                        &self.params,
                    );
                    forward_result(result, candidate, &self.params, &ctx, fw);
                }
                GeometryValue::CityGmlGeometry(candidate_geo) => {
                    let result = test_citygml_geometry(
                        candidate_geo,
                        &self.filters,
                        &self.params,
                    );
                    forward_result(result, candidate, &self.params, &ctx, fw);
                }
                _ => {
                    fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                        &ctx,
                        candidate.clone(),
                        REJECTED_PORT.clone(),
                    ));
                }
            }
        }
        Ok(())
    }

    fn name(&self) -> &str {
        "SpatialFilter"
    }
}

struct TestResult {
    passed: bool,
    match_count: usize,
}

fn forward_result(
    result: TestResult,
    feature: &Feature,
    params: &SpatialFilterParams,
    ctx: &NodeContext,
    fw: &ProcessorChannelForwarder,
) {
    let mut feature = feature.clone();

    // Add match count attribute if configured
    if let Some(ref attr_name) = params.output_match_count_attribute {
        feature.attributes.insert(
            attr_name.clone(),
            AttributeValue::Number(serde_json::Number::from(result.match_count)),
        );
    }

    let port = if result.passed {
        PASSED_PORT.clone()
    } else {
        FAILED_PORT.clone()
    };

    fw.send(ExecutorContext::new_with_node_context_feature_and_port(
        ctx,
        feature,
        port,
    ));
}

fn test_2d_geometry(
    candidate: &Geometry2D,
    filters: &[Feature],
    params: &SpatialFilterParams,
) -> TestResult {
    let mut match_count = 0;

    for filter in filters {
        if let GeometryValue::FlowGeometry2D(filter_geo) = &filter.geometry.value {
            if test_predicate_2d(candidate, filter_geo, &params.predicate) {
                match_count += 1;
                if params.pass_on_multiple_matches {
                    // OR logic: return early on first match
                    return TestResult {
                        passed: true,
                        match_count,
                    };
                }
            } else if !params.pass_on_multiple_matches {
                // AND logic: return early on first non-match
                return TestResult {
                    passed: false,
                    match_count,
                };
            }
        }
    }

    // If we get here:
    // - For OR logic (pass_on_multiple): no matches found, so fail
    // - For AND logic (!pass_on_multiple): all matches passed, so pass
    TestResult {
        passed: if params.pass_on_multiple_matches {
            false
        } else {
            match_count > 0
        },
        match_count,
    }
}

fn test_3d_geometry(
    candidate: &Geometry3D,
    filters: &[Feature],
    params: &SpatialFilterParams,
) -> TestResult {
    let mut match_count = 0;

    for filter in filters {
        let matches = match &filter.geometry.value {
            GeometryValue::FlowGeometry3D(filter_geo) => {
                test_predicate_3d(candidate, filter_geo, &params.predicate)
            }
            GeometryValue::CityGmlGeometry(citygml) => {
                // Test against CityGML polygons
                citygml.gml_geometries.iter().any(|gml| {
                    gml.polygons.iter().any(|poly| {
                        test_predicate_3d_poly(candidate, poly, &params.predicate)
                    })
                })
            }
            _ => false,
        };

        if matches {
            match_count += 1;
            if params.pass_on_multiple_matches {
                return TestResult {
                    passed: true,
                    match_count,
                };
            }
        } else if !params.pass_on_multiple_matches {
            return TestResult {
                passed: false,
                match_count,
            };
        }
    }

    TestResult {
        passed: if params.pass_on_multiple_matches {
            false
        } else {
            match_count > 0
        },
        match_count,
    }
}

fn test_citygml_geometry(
    candidate: &CityGmlGeometry,
    filters: &[Feature],
    params: &SpatialFilterParams,
) -> TestResult {
    let mut match_count = 0;

    // Extract all polygons from candidate CityGML
    let candidate_polygons: Vec<_> = candidate
        .gml_geometries
        .iter()
        .flat_map(|gml| &gml.polygons)
        .collect();

    if candidate_polygons.is_empty() {
        return TestResult {
            passed: false,
            match_count: 0,
        };
    }

    for filter in filters {
        let matches = match &filter.geometry.value {
            GeometryValue::FlowGeometry3D(filter_geo) => {
                // Test if any candidate polygon matches the filter
                candidate_polygons.iter().any(|poly| {
                    test_predicate_3d_poly_reverse(filter_geo, poly, &params.predicate)
                })
            }
            GeometryValue::CityGmlGeometry(filter_citygml) => {
                // Test CityGML against CityGML
                let filter_polygons: Vec<_> = filter_citygml
                    .gml_geometries
                    .iter()
                    .flat_map(|gml| &gml.polygons)
                    .collect();

                candidate_polygons.iter().any(|candidate_poly| {
                    filter_polygons.iter().any(|filter_poly| {
                        test_predicate_poly_poly(candidate_poly, filter_poly, &params.predicate)
                    })
                })
            }
            _ => false,
        };

        if matches {
            match_count += 1;
            if params.pass_on_multiple_matches {
                return TestResult {
                    passed: true,
                    match_count,
                };
            }
        } else if !params.pass_on_multiple_matches {
            return TestResult {
                passed: false,
                match_count,
            };
        }
    }

    TestResult {
        passed: if params.pass_on_multiple_matches {
            false
        } else {
            match_count > 0
        },
        match_count,
    }
}

/// Tests spatial predicates between 2D geometries using the DE-9IM (Dimensionally Extended 9-Intersection Model).
///
/// # Important Limitation with Pure 2D Geometries
/// This function uses the `Relate` trait which currently has a bug with pure 2D geometries that lack Z coordinates
/// (Geometry<T, NoValue>). The relate implementation attempts 3D orient operations that panic on NoValue Z coords.
///
/// **Workaround:** If you need to use this action with pure 2D GeoJSON/Shapefile data, first convert to 3D with Z=0
/// using the **ThreeDimensionForcer** action. This is the recommended workflow:
/// ```
/// GeoJSONReader -> ThreeDimensionForcer -> SpatialFilter
/// ```
///
/// **Note:** This is a known issue in the underlying geometry library and does not affect 3D geometries or
/// geometries that already have Z coordinates.
fn test_predicate_2d(
    candidate: &Geometry2D,
    filter: &Geometry2D,
    predicate: &SpatialPredicate,
) -> bool {
    let matrix = candidate.relate(filter);

    match predicate {
        SpatialPredicate::Contains => matrix.is_contains(),
        SpatialPredicate::Within => matrix.is_within(),
        SpatialPredicate::Intersects => matrix.is_intersects(),
        SpatialPredicate::Touches => matrix.is_touches(),
        SpatialPredicate::Crosses => matrix.is_crosses(),
        SpatialPredicate::Overlaps => matrix.is_overlaps(),
        SpatialPredicate::Disjoint => matrix.is_disjoint(),
        SpatialPredicate::CoveredBy => matrix.is_coveredby(),
        SpatialPredicate::Covers => matrix.is_covers(),
    }
}

fn test_predicate_3d(
    candidate: &Geometry3D,
    filter: &Geometry3D,
    predicate: &SpatialPredicate,
) -> bool {
    let matrix = candidate.relate(filter);

    match predicate {
        SpatialPredicate::Contains => matrix.is_contains(),
        SpatialPredicate::Within => matrix.is_within(),
        SpatialPredicate::Intersects => matrix.is_intersects(),
        SpatialPredicate::Touches => matrix.is_touches(),
        SpatialPredicate::Crosses => matrix.is_crosses(),
        SpatialPredicate::Overlaps => matrix.is_overlaps(),
        SpatialPredicate::Disjoint => matrix.is_disjoint(),
        SpatialPredicate::CoveredBy => matrix.is_coveredby(),
        SpatialPredicate::Covers => matrix.is_covers(),
    }
}

fn test_predicate_3d_poly(
    candidate: &Geometry3D,
    filter: &reearth_flow_geometry::types::polygon::Polygon3D<f64>,
    predicate: &SpatialPredicate,
) -> bool {
    let filter_geo = Geometry3D::Polygon(filter.clone());
    test_predicate_3d(candidate, &filter_geo, predicate)
}

fn test_predicate_3d_poly_reverse(
    filter: &Geometry3D,
    candidate: &reearth_flow_geometry::types::polygon::Polygon3D<f64>,
    predicate: &SpatialPredicate,
) -> bool {
    let candidate_geo = Geometry3D::Polygon(candidate.clone());
    test_predicate_3d(&candidate_geo, filter, predicate)
}

fn test_predicate_poly_poly(
    candidate: &reearth_flow_geometry::types::polygon::Polygon3D<f64>,
    filter: &reearth_flow_geometry::types::polygon::Polygon3D<f64>,
    predicate: &SpatialPredicate,
) -> bool {
    let candidate_geo = Geometry3D::Polygon(candidate.clone());
    let filter_geo = Geometry3D::Polygon(filter.clone());
    test_predicate_3d(&candidate_geo, &filter_geo, predicate)
}

#[cfg(test)]
mod tests {
    use super::*;
    use reearth_flow_geometry::types::coordinate::Coordinate2D;
    use reearth_flow_geometry::types::line_string::LineString2D;
    use reearth_flow_geometry::types::polygon::Polygon2D;
    use reearth_flow_runtime::executor_operation::NodeContext;
    use reearth_flow_runtime::forwarder::NoopChannelForwarder;
    use reearth_flow_types::Geometry;

    use crate::tests::utils::create_default_execute_context;

    fn create_test_polygon_2d() -> Polygon2D<f64> {
        let exterior = LineString2D::new(vec![
            Coordinate2D::new_(0.0, 0.0),
            Coordinate2D::new_(10.0, 0.0),
            Coordinate2D::new_(10.0, 10.0),
            Coordinate2D::new_(0.0, 10.0),
            Coordinate2D::new_(0.0, 0.0),
        ]);
        Polygon2D::new(exterior, vec![])
    }

    fn create_filter_polygon_2d() -> Polygon2D<f64> {
        let exterior = LineString2D::new(vec![
            Coordinate2D::new_(5.0, 5.0),
            Coordinate2D::new_(15.0, 5.0),
            Coordinate2D::new_(15.0, 15.0),
            Coordinate2D::new_(5.0, 15.0),
            Coordinate2D::new_(5.0, 5.0),
        ]);
        Polygon2D::new(exterior, vec![])
    }

    fn create_disjoint_polygon_2d() -> Polygon2D<f64> {
        let exterior = LineString2D::new(vec![
            Coordinate2D::new_(20.0, 20.0),
            Coordinate2D::new_(30.0, 20.0),
            Coordinate2D::new_(30.0, 30.0),
            Coordinate2D::new_(20.0, 30.0),
            Coordinate2D::new_(20.0, 20.0),
        ]);
        Polygon2D::new(exterior, vec![])
    }

    #[test]
    fn test_spatial_filter_accepts_features() {
        let mut filter = SpatialFilter {
            params: SpatialFilterParams {
                predicate: SpatialPredicate::Intersects,
                pass_on_multiple_matches: true,
                output_match_count_attribute: None,
            },
            filters: Vec::new(),
            candidates: Vec::new(),
        };

        let filter_feature = Feature {
            geometry: Geometry {
                value: GeometryValue::FlowGeometry2D(Geometry2D::Polygon(create_filter_polygon_2d())),
                ..Default::default()
            },
            ..Default::default()
        };

        let candidate_feature = Feature {
            geometry: Geometry {
                value: GeometryValue::FlowGeometry2D(Geometry2D::Polygon(create_test_polygon_2d())),
                ..Default::default()
            },
            ..Default::default()
        };

        // Process filter
        let noop = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop);
        let mut ctx = create_default_execute_context(&filter_feature);
        ctx.port = FILTER_PORT.clone();
        let result = filter.process(ctx, &fw);
        assert!(result.is_ok());
        assert_eq!(filter.filters.len(), 1);

        // Process candidate
        let noop = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop);
        let mut ctx = create_default_execute_context(&candidate_feature);
        ctx.port = CANDIDATE_PORT.clone();
        let result = filter.process(ctx, &fw);
        assert!(result.is_ok());
        assert_eq!(filter.candidates.len(), 1);
    }

    #[test]
    fn test_spatial_filter_processes_multiple_ports() {
        let mut filter = SpatialFilter {
            params: SpatialFilterParams {
                predicate: SpatialPredicate::Disjoint,
                pass_on_multiple_matches: true,
                output_match_count_attribute: None,
            },
            filters: Vec::new(),
            candidates: Vec::new(),
        };

        let filter_feature = Feature {
            geometry: Geometry {
                value: GeometryValue::FlowGeometry2D(Geometry2D::Polygon(create_filter_polygon_2d())),
                ..Default::default()
            },
            ..Default::default()
        };

        let candidate_feature = Feature {
            geometry: Geometry {
                value: GeometryValue::FlowGeometry2D(Geometry2D::Polygon(create_disjoint_polygon_2d())),
                ..Default::default()
            },
            ..Default::default()
        };

        // Process filter
        let noop = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop);
        let mut ctx = create_default_execute_context(&filter_feature);
        ctx.port = FILTER_PORT.clone();
        let result = filter.process(ctx, &fw);
        assert!(result.is_ok());

        // Process candidate
        let noop = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop);
        let mut ctx = create_default_execute_context(&candidate_feature);
        ctx.port = CANDIDATE_PORT.clone();
        let result = filter.process(ctx, &fw);
        assert!(result.is_ok());

        // Verify both were added
        assert_eq!(filter.filters.len(), 1, "Filter should have 1 filter geometry");
        assert_eq!(filter.candidates.len(), 1, "Filter should have 1 candidate geometry");
    }

    #[test]
    fn test_spatial_filter_no_filters() {
        let filter = SpatialFilter {
            params: SpatialFilterParams::default(),
            filters: Vec::new(),
            candidates: vec![Feature::default()],
        };

        let noop = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop);
        let ctx = NodeContext::default();
        let _ = filter.finish(ctx, &fw);

        if let ProcessorChannelForwarder::Noop(noop) = fw {
            let ports = noop.send_ports.lock().unwrap();
            assert_eq!(ports.len(), 1);
            assert_eq!(ports[0], *REJECTED_PORT, "No filters should reject candidates");
        }
    }
}
