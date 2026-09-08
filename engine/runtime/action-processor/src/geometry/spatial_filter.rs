use std::collections::HashMap;

use once_cell::sync::Lazy;
#[cfg(feature = "new-geometry")]
use reearth_flow_diagnostics::{DiagnosticDraft, ErrorCode};
#[cfg(not(feature = "new-geometry"))]
use reearth_flow_geometry::algorithm::relate::Relate;
#[cfg(not(feature = "new-geometry"))]
use reearth_flow_geometry::types::geometry::{Geometry2D, Geometry3D};
#[cfg(not(feature = "new-geometry"))]
use reearth_flow_geometry::types::polygon::Polygon2D;
#[cfg(feature = "new-geometry")]
use reearth_flow_geometry::{
    coordinate::CoordinateFrame,
    ops::{Aabb, BoundingBox, FootprintError, FootprintPlane},
    predicates::view::{flatten_2d, Leaf2D},
    predicates::{contains, covers, intersects, relate},
    Geometry,
};
use reearth_flow_runtime::node::REJECTED_PORT;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory},
};
use reearth_flow_types::{Attribute, AttributeValue, Feature};
#[cfg(not(feature = "new-geometry"))]
use reearth_flow_types::{CityGmlGeometry, GeometryValue};
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
        "Spatial Filter"
    }

    fn description(&self) -> &str {
        "Filters candidate features by their spatial relationship to filter geometries, tested in the horizontal plane — a 3D geometry is compared by its footprint and must be in a coordinate frame with linear units. Every candidate passes when no filter geometry is supplied at all."
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(SpatialFilterParams))
    }

    fn categories(&self) -> &[&'static str] {
        &["Filter"]
    }

    fn tags(&self) -> &[&'static str] {
        &["spatial"]
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
            #[cfg(feature = "new-geometry")]
            frame: None,
            #[cfg(feature = "new-geometry")]
            frame_mismatch_reported: false,
            #[cfg(feature = "new-geometry")]
            filters_received: 0,
        }))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema, Default)]
#[serde(rename_all = "camelCase")]
#[schemars(
    title = "Spatial Filter Parameters",
    description = "Configures which spatial relationship is tested between the filter and candidate geometries, and what a passing candidate carries away from the filter."
)]
pub struct SpatialFilterParams {
    /// # Spatial Predicate
    /// The spatial relationship to test, read with the candidate as the subject and the filter geometry as the object.
    #[serde(default)]
    pub predicate: SpatialPredicate,

    /// # Match Mode
    /// Whether a candidate passes by matching any single filter feature, or only by matching every filter feature.
    #[serde(default)]
    pub match_mode: MatchMode,

    /// # Merge Filter Attributes
    /// Copies attributes from every matched filter feature onto passing candidates, overwriting a candidate's own attribute of the same name. When several matched filters share an attribute, the last one wins.
    #[serde(default)]
    pub merge_filter_attributes: bool,

    /// # Merged Attributes Prefix
    /// Prefix applied to merged attribute names so they cannot collide with the candidate's own. A prefix of "filter_" turns a filter attribute "zone" into "filter_zone". Ignored unless attributes are merged.
    #[serde(default)]
    pub merged_attributes_prefix: Option<String>,

    /// # Output Match Count Attribute
    /// Attribute to store how many filter features the candidate matched. Written to passing and failing candidates alike.
    #[serde(default)]
    pub output_match_count_attribute: Option<Attribute>,
}

/// # Match Mode
/// How the tests against the individual filter features combine into pass or fail.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, JsonSchema, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub enum MatchMode {
    /// # Any
    /// Passes a candidate that matches at least one filter feature.
    #[default]
    Any,
    /// # All
    /// Passes a candidate only when every filter feature matches it.
    All,
}

/// # Spatial Predicate
/// The relationship each candidate is tested for against a filter geometry.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub enum SpatialPredicate {
    /// # Contains
    /// Passes a candidate that holds the filter geometry inside it, sharing
    /// interior with it. A filter lying wholly on the candidate's boundary does
    /// not count; use Covers for that.
    Contains,
    /// # Within
    /// Passes a candidate that lies inside the filter geometry, sharing interior
    /// with it. A candidate lying wholly on the filter's boundary does not count;
    /// use Covered By for that.
    Within,
    /// # Intersects
    /// Passes a candidate that shares at least one point with the filter geometry.
    #[default]
    Intersects,
    /// # Disjoint
    /// Passes a candidate that shares no point at all with the filter geometry.
    Disjoint,
    /// # Touches
    /// Passes a candidate that meets the filter geometry only along a boundary,
    /// with no shared interior.
    Touches,
    /// # Crosses
    /// Passes a candidate that cuts through the filter geometry, meeting its
    /// interior in a lower-dimensional overlap such as a line across a polygon.
    Crosses,
    /// # Overlaps
    /// Passes a candidate of the same dimension as the filter geometry that
    /// shares interior with it while each keeps points outside the other.
    Overlaps,
    /// # Covered By
    /// Passes a candidate whose every point lies in the filter geometry,
    /// including one lying wholly on its boundary.
    CoveredBy,
    /// # Covers
    /// Passes a candidate that holds every point of the filter geometry,
    /// including a filter lying wholly on its boundary.
    Covers,
}

#[derive(Debug, Clone)]
struct SpatialFilter {
    params: SpatialFilterParams,
    #[cfg(not(feature = "new-geometry"))]
    filters: Vec<Feature>,
    #[cfg(not(feature = "new-geometry"))]
    candidates: Vec<Feature>,
    #[cfg(feature = "new-geometry")]
    filters: Vec<PreparedFeature>,
    #[cfg(feature = "new-geometry")]
    candidates: Vec<PreparedFeature>,
    /// The coordinate frame every operand must share, fixed by the first
    /// accepted feature. Spatial relationships are only meaningful within one
    /// frame.
    #[cfg(feature = "new-geometry")]
    frame: Option<CoordinateFrame>,
    /// Whether the frame mismatch has already been reported, so the same
    /// upstream problem is logged once rather than once per feature.
    #[cfg(feature = "new-geometry")]
    frame_mismatch_reported: bool,
    /// How many features arrived as filters, whether or not they survived
    /// intake. It separates "no filter was supplied", which imposes no
    /// restriction, from "every supplied filter was unusable", which must not
    /// silently become the same thing.
    #[cfg(feature = "new-geometry")]
    filters_received: usize,
}

impl Processor for SpatialFilter {
    fn is_accumulating(&self) -> bool {
        true
    }

    #[cfg(not(feature = "new-geometry"))]
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
                        fw.send(
                            ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()),
                        );
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

    #[cfg(not(feature = "new-geometry"))]
    fn finish(
        &mut self,
        ctx: NodeContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        if self.filters.is_empty() {
            // No filters provided, pass all candidates (no restrictions)
            for candidate in &self.candidates {
                fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                    &ctx,
                    candidate.clone(),
                    PASSED_PORT.clone(),
                ));
            }
            return Ok(());
        }

        // Process each candidate against all filters
        for candidate in &self.candidates {
            match &candidate.geometry.value {
                GeometryValue::FlowGeometry2D(candidate_geo) => {
                    let result = test_2d_geometry(candidate_geo, &self.filters, &self.params);
                    forward_result(result, candidate, &self.filters, &self.params, &ctx, fw);
                }
                GeometryValue::FlowGeometry3D(candidate_geo) => {
                    let result = test_3d_geometry(candidate_geo, &self.filters, &self.params);
                    forward_result(result, candidate, &self.filters, &self.params, &ctx, fw);
                }
                GeometryValue::CityGmlGeometry(candidate_geo) => {
                    let result = test_citygml_geometry(candidate_geo, &self.filters, &self.params);
                    forward_result(result, candidate, &self.filters, &self.params, &ctx, fw);
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

    #[cfg(feature = "new-geometry")]
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let is_filter = match &ctx.port {
            port if port == &*FILTER_PORT => true,
            port if port == &*CANDIDATE_PORT => false,
            _ => {
                fw.send(ctx.new_with_feature_and_port(ctx.feature.clone(), REJECTED_PORT.clone()));
                return Ok(());
            }
        };
        if is_filter {
            self.filters_received += 1;
        }
        match self.prepare(&ctx) {
            Ok(prepared) => {
                if is_filter {
                    self.filters.push(prepared);
                } else {
                    self.candidates.push(prepared);
                }
            }
            Err(rejection) => {
                let message = format!("Spatial Filter rejected a feature: {}", rejection.message);
                match rejection.code {
                    Some(code) => ctx.warn(DiagnosticDraft::new(code).with_message(message)),
                    None => ctx.event_hub.debug_log(Some(ctx.error_span()), message),
                }
                fw.send(ctx.new_with_feature_and_port(ctx.feature.clone(), REJECTED_PORT.clone()));
            }
        }
        Ok(())
    }

    #[cfg(feature = "new-geometry")]
    fn finish(
        &mut self,
        ctx: NodeContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        if self.filters.is_empty() {
            // No filter geometry to test against. Which of the two ways that
            // happened decides the routing: no filter supplied is no
            // restriction, but filters supplied and all rejected is a failed
            // condition — passing those candidates would turn an upstream
            // error into a silent pass-everything.
            let unusable = self.filters_received > 0;
            // Warn once: the cause is one upstream problem, not one per candidate.
            if let (true, Some(first)) = (unusable, self.candidates.first()) {
                ExecutorContext::new_with_node_context_feature_and_port(
                    &ctx,
                    first.feature.clone(),
                    FAILED_PORT.clone(),
                )
                .warn(
                    DiagnosticDraft::new(ErrorCode::GeometryNoUsableFilter).with_message(format!(
                        "Spatial Filter failed every candidate: all {} filter features were rejected, leaving nothing to test against.",
                        self.filters_received
                    )),
                );
            }
            for candidate in &self.candidates {
                // Through `emit` so these candidates are stamped like any
                // other: no filter was matched, so the count is zero and there
                // is nothing to merge.
                self.emit(&ctx, fw, candidate, !unusable, &[]);
            }
            return Ok(());
        }

        // Filter bounding boxes, indexed for the per-candidate prefilter.
        let tree = rstar::RTree::bulk_load(
            self.filters
                .iter()
                .enumerate()
                .map(|(index, filter)| FilterEntry {
                    index,
                    env: envelope(&filter.aabb),
                })
                .collect(),
        );

        // The early exits below skip tests whose outcome cannot change the
        // routing, which is only sound when nothing observes the full set of
        // matches.
        let exhaustive = self.params.output_match_count_attribute.is_some()
            || self.params.merge_filter_attributes;
        let match_all = self.params.match_mode == MatchMode::All;

        for candidate in &self.candidates {
            let mut bbox_hit = vec![false; self.filters.len()];
            for entry in tree.locate_in_envelope_intersecting(&envelope(&candidate.aabb)) {
                bbox_hit[entry.index] = true;
            }

            let mut matched: Vec<usize> = Vec::new();
            let mut error = None;
            for (i, filter) in self.filters.iter().enumerate() {
                // Every predicate except `disjoint` needs the bounding boxes to
                // intersect; `disjoint` holds outright when they do not.
                let result = if matches!(self.params.predicate, SpatialPredicate::Disjoint) {
                    if bbox_hit[i] {
                        test_predicate(
                            &candidate.prepared,
                            &filter.prepared,
                            &self.params.predicate,
                        )
                    } else {
                        Ok(true)
                    }
                } else if bbox_hit[i] {
                    test_predicate(
                        &candidate.prepared,
                        &filter.prepared,
                        &self.params.predicate,
                    )
                } else {
                    Ok(false)
                };
                match result {
                    Ok(true) => {
                        matched.push(i);
                        if !match_all && !exhaustive {
                            break;
                        }
                    }
                    Ok(false) => {
                        if match_all && !exhaustive {
                            break;
                        }
                    }
                    Err(e) => {
                        error = Some(e);
                        break;
                    }
                }
            }

            if let Some(e) = error {
                let rejected = ExecutorContext::new_with_node_context_feature_and_port(
                    &ctx,
                    candidate.feature.clone(),
                    REJECTED_PORT.clone(),
                );
                rejected.warn(
                    DiagnosticDraft::new(ErrorCode::GeometrySpatialTestFailed).with_message(
                        format!("Spatial Filter rejected a feature: the spatial test failed: {e}"),
                    ),
                );
                fw.send(rejected);
                continue;
            }

            let passed = if match_all {
                matched.len() == self.filters.len()
            } else {
                !matched.is_empty()
            };
            self.emit(&ctx, fw, candidate, passed, &matched);
        }
        Ok(())
    }

    fn name(&self) -> &str {
        "Spatial Filter"
    }
}

// --- new geometry --------------------------------------------------------------

/// A feature made relate-ready at intake, so unusable geometry is rejected on
/// arrival and `finish` runs on 2D operands only.
#[cfg(feature = "new-geometry")]
#[derive(Debug, Clone)]
struct PreparedFeature {
    feature: Feature,
    /// The 2D geometry the spatial tests run on: the feature's own 2D geometry,
    /// or the horizontal footprint of a geometry with any 3D part.
    prepared: Geometry,
    /// The prepared geometry's bounding box, for the prefilter.
    aabb: Aabb,
}

/// Why a feature was turned away at intake, and whether that is an upstream
/// problem worth a warning rather than a debug note.
#[cfg(feature = "new-geometry")]
struct Rejection {
    message: String,
    /// `Some` when the cause is worth a warning; `None` keeps it on the debug lane.
    code: Option<ErrorCode>,
}

#[cfg(feature = "new-geometry")]
impl Rejection {
    fn debug(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            code: None,
        }
    }

    fn warn(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            code: Some(code),
        }
    }
}

#[cfg(feature = "new-geometry")]
impl SpatialFilter {
    /// The feature made relate-ready, or why it cannot take part: no geometry,
    /// no computable footprint, or a coordinate frame other than the one this
    /// node's inputs fixed.
    fn prepare(&mut self, ctx: &ExecutorContext) -> Result<PreparedFeature, Rejection> {
        let geometry = ctx.feature.geometry.as_ref();
        if matches!(geometry, Geometry::None) {
            return Err(Rejection::debug("the feature has no geometry"));
        }

        let prepared = if has_3d(geometry) {
            geometry
                .footprint_on(&FootprintPlane::Horizontal)
                .map_err(|e| match e {
                    FootprintError::Empty | FootprintError::Unsupported(_) => {
                        Rejection::debug(format!("the geometry has no testable footprint: {e}"))
                    }
                    FootprintError::NonLinearFrame(_) => Rejection::warn(
                        ErrorCode::GeometryNonLinearFrame,
                        format!(
                            "{e}. Reproject 3D input to a coordinate system in linear units upstream."
                        ),
                    ),
                    _ => Rejection::warn(
                        ErrorCode::GeometryFootprintUnavailable,
                        format!("the footprint could not be computed: {e}"),
                    ),
                })?
        } else {
            geometry.clone()
        };

        let mut leaves = Vec::new();
        if !collect_2d_leaves(&prepared, &mut leaves) {
            return Err(Rejection::warn(
                ErrorCode::GeometryMixedDimensions,
                "the geometry mixes 2D and 3D parts in a way the footprint could not flatten",
            ));
        }
        let Some(frame) = leaves.first().map(|leaf| leaf.frame().clone()) else {
            return Err(Rejection::debug("the geometry has nothing to test"));
        };
        if leaves.iter().any(|leaf| leaf.frame() != &frame) {
            return Err(Rejection::warn(
                ErrorCode::GeometryMixedCoordinateFrames,
                "the geometry mixes coordinate frames; reproject it to one frame upstream",
            ));
        }

        match &self.frame {
            None => self.frame = Some(frame),
            Some(node_frame) if *node_frame != frame => {
                let message = format!(
                    "the feature is in {frame:?} while this node's inputs are in {node_frame:?}. \
                     Reproject the input to one coordinate frame upstream."
                );
                return Err(if self.frame_mismatch_reported {
                    Rejection::debug(message)
                } else {
                    self.frame_mismatch_reported = true;
                    Rejection::warn(ErrorCode::GeometryCoordinateFrameMismatch, message)
                });
            }
            Some(_) => {}
        }

        let aabb = prepared
            .bounding_box()
            .map_err(|e| Rejection::debug(format!("the geometry has no bounding box: {e}")))?;

        Ok(PreparedFeature {
            feature: ctx.feature.clone(),
            prepared,
            aabb,
        })
    }

    /// Route the candidate to `passed` or `failed`, stamping the match count
    /// and merged filter attributes where configured.
    fn emit(
        &self,
        ctx: &NodeContext,
        fw: &ProcessorChannelForwarder,
        candidate: &PreparedFeature,
        passed: bool,
        matched: &[usize],
    ) {
        let mut feature = candidate.feature.clone();

        if let Some(ref attr_name) = self.params.output_match_count_attribute {
            feature.attributes_mut().insert(
                attr_name.clone(),
                AttributeValue::Number(serde_json::Number::from(matched.len())),
            );
        }

        if self.params.merge_filter_attributes && passed {
            for &filter_index in matched {
                let filter = &self.filters[filter_index].feature;
                for (key, value) in filter.attributes.iter() {
                    let merged_key = match &self.params.merged_attributes_prefix {
                        Some(prefix) => Attribute::new(format!("{prefix}{key}")),
                        None => key.clone(),
                    };
                    feature.attributes_mut().insert(merged_key, value.clone());
                }
            }
        }

        let port = if passed {
            PASSED_PORT.clone()
        } else {
            FAILED_PORT.clone()
        };
        fw.send(ExecutorContext::new_with_node_context_feature_and_port(
            ctx, feature, port,
        ));
    }
}

/// Whether the geometry has any 3D part, and so compares by its footprint.
#[cfg(feature = "new-geometry")]
fn has_3d(geometry: &Geometry) -> bool {
    match geometry {
        Geometry::Euclidean3D(_) => true,
        Geometry::GeometryCollection(c) => c.members().iter().any(has_3d),
        _ => false,
    }
}

/// Collect the geometry's 2D leaves; `false` if a 3D part appears.
#[cfg(feature = "new-geometry")]
fn collect_2d_leaves<'a>(geometry: &'a Geometry, leaves: &mut Vec<Leaf2D<'a>>) -> bool {
    match geometry {
        Geometry::None => true,
        Geometry::Euclidean2D(g) => {
            flatten_2d(g, leaves);
            true
        }
        Geometry::Euclidean3D(_) => false,
        Geometry::GeometryCollection(c) => c
            .members()
            .iter()
            .all(|member| collect_2d_leaves(member, leaves)),
    }
}

/// Whether the candidate-relative predicate holds between two prepared 2D
/// geometries sharing one frame. The point-set predicates take the exact fast
/// paths, which stay correct on collections with overlapping members; only the
/// predicates needing the full DE-9IM matrix go through `relate`.
#[cfg(feature = "new-geometry")]
fn test_predicate(
    candidate: &Geometry,
    filter: &Geometry,
    predicate: &SpatialPredicate,
) -> reearth_flow_geometry::predicates::Result<bool> {
    match predicate {
        SpatialPredicate::Intersects => intersects(candidate, filter),
        SpatialPredicate::Disjoint => Ok(!intersects(candidate, filter)?),
        SpatialPredicate::Contains => contains(candidate, filter),
        SpatialPredicate::Within => contains(filter, candidate),
        SpatialPredicate::Covers => covers(candidate, filter),
        SpatialPredicate::CoveredBy => covers(filter, candidate),
        SpatialPredicate::Touches => Ok(relate(candidate, filter)?.is_touches()),
        SpatialPredicate::Crosses => Ok(relate(candidate, filter)?.is_crosses()),
        SpatialPredicate::Overlaps => Ok(relate(candidate, filter)?.is_overlaps()),
    }
}

/// A filter's index under its bounding box, for the R-tree prefilter.
#[cfg(feature = "new-geometry")]
struct FilterEntry {
    index: usize,
    env: rstar::AABB<[f64; 2]>,
}

#[cfg(feature = "new-geometry")]
impl rstar::RTreeObject for FilterEntry {
    type Envelope = rstar::AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        self.env
    }
}

#[cfg(feature = "new-geometry")]
fn envelope(aabb: &Aabb) -> rstar::AABB<[f64; 2]> {
    match *aabb {
        Aabb::D2 { min, max } => rstar::AABB::from_corners(min, max),
        Aabb::D3 { min, max } => rstar::AABB::from_corners([min[0], min[1]], [max[0], max[1]]),
    }
}

// --- legacy geometry -------------------------------------------------------------

#[cfg(not(feature = "new-geometry"))]
struct TestResult {
    passed: bool,
    match_count: usize,
    matched_filter_indices: Vec<usize>,
}

#[cfg(not(feature = "new-geometry"))]
fn forward_result(
    result: TestResult,
    feature: &Feature,
    filters: &[Feature],
    params: &SpatialFilterParams,
    ctx: &NodeContext,
    fw: &ProcessorChannelForwarder,
) {
    let mut feature = feature.clone();

    // Add match count attribute if configured
    if let Some(ref attr_name) = params.output_match_count_attribute {
        feature.attributes_mut().insert(
            attr_name.clone(),
            AttributeValue::Number(serde_json::Number::from(result.match_count)),
        );
    }

    // Add merge filter attribute if configured
    if params.merge_filter_attributes {
        for &filter_index in &result.matched_filter_indices {
            let filter = &filters[filter_index];
            for (key, value) in filter.attributes.iter() {
                let merged_key = match &params.merged_attributes_prefix {
                    Some(prefix) => Attribute::new(format!("{}{}", prefix, key)),
                    None => key.clone(),
                };
                feature.attributes_mut().insert(merged_key, value.clone());
            }
        }
    }
    let port = if result.passed {
        PASSED_PORT.clone()
    } else {
        FAILED_PORT.clone()
    };

    fw.send(ExecutorContext::new_with_node_context_feature_and_port(
        ctx, feature, port,
    ));
}

#[cfg(not(feature = "new-geometry"))]
fn test_2d_geometry(
    candidate: &Geometry2D,
    filters: &[Feature],
    params: &SpatialFilterParams,
) -> TestResult {
    let match_any = params.match_mode == MatchMode::Any;
    let mut match_count = 0;
    let mut matched_filter_indices: Vec<usize> = Vec::new();

    for (i, filter) in filters.iter().enumerate() {
        let filter_matches = match &filter.geometry.value {
            GeometryValue::FlowGeometry2D(filter_geo) => {
                test_predicate_2d(candidate, filter_geo, &params.predicate)
            }
            GeometryValue::FlowGeometry3D(filter_geo) => {
                // Project 3D filter to 2D (drop Z) and test
                let filter_2d: Geometry2D<f64> = filter_geo.clone().into();
                test_predicate_2d(candidate, &filter_2d, &params.predicate)
            }
            GeometryValue::CityGmlGeometry(citygml) => {
                // Project CityGML filter polygons to 2D and test
                citygml.gml_geometries.iter().any(|gml| {
                    gml.polygons.iter().any(|poly| {
                        let poly_2d: Polygon2D<f64> = poly.clone().into();
                        let filter_2d = Geometry2D::Polygon(poly_2d);
                        test_predicate_2d(candidate, &filter_2d, &params.predicate)
                    })
                })
            }
            _ => false,
        };

        if filter_matches {
            match_count += 1;
            if match_any {
                // OR logic: return early on first match
                return TestResult {
                    passed: true,
                    match_count,
                    matched_filter_indices: vec![i],
                };
            } else {
                // AND logic: accumulate index for potential attribute merging
                matched_filter_indices.push(i);
            }
        } else if !match_any {
            // AND logic: return early on first non-match
            return TestResult {
                passed: false,
                match_count,
                matched_filter_indices: Vec::new(),
            };
        }
    }

    // If we get here:
    // - For OR logic (match any): no matches found, so fail
    // - For AND logic (match all): all filters matched, so pass
    TestResult {
        passed: if match_any { false } else { match_count > 0 },
        match_count,
        matched_filter_indices,
    }
}

#[cfg(not(feature = "new-geometry"))]
fn test_3d_geometry(
    candidate: &Geometry3D,
    filters: &[Feature],
    params: &SpatialFilterParams,
) -> TestResult {
    let match_any = params.match_mode == MatchMode::Any;
    let mut match_count = 0;
    let mut matched_filter_indices: Vec<usize> = Vec::new();

    for (i, filter) in filters.iter().enumerate() {
        let filter_matches = match &filter.geometry.value {
            GeometryValue::FlowGeometry2D(filter_geo) => {
                // Project 3D candidate to 2D (drop Z) and test against 2D filter
                let candidate_2d: Geometry2D<f64> = candidate.clone().into();
                test_predicate_2d(&candidate_2d, filter_geo, &params.predicate)
            }
            GeometryValue::FlowGeometry3D(filter_geo) => {
                test_predicate_3d(candidate, filter_geo, &params.predicate)
            }
            GeometryValue::CityGmlGeometry(citygml) => {
                // Test against CityGML polygons
                citygml.gml_geometries.iter().any(|gml| {
                    gml.polygons
                        .iter()
                        .any(|poly| test_predicate_3d_poly(candidate, poly, &params.predicate))
                })
            }
            _ => false,
        };

        if filter_matches {
            match_count += 1;
            if match_any {
                return TestResult {
                    passed: true,
                    match_count,
                    matched_filter_indices: vec![i],
                };
            } else {
                matched_filter_indices.push(i);
            }
        } else if !match_any {
            return TestResult {
                passed: false,
                match_count,
                matched_filter_indices: Vec::new(),
            };
        }
    }

    TestResult {
        passed: if match_any { false } else { match_count > 0 },
        match_count,
        matched_filter_indices,
    }
}

#[cfg(not(feature = "new-geometry"))]
fn test_citygml_geometry(
    candidate: &CityGmlGeometry,
    filters: &[Feature],
    params: &SpatialFilterParams,
) -> TestResult {
    let match_any = params.match_mode == MatchMode::Any;
    let mut match_count = 0;
    let mut matched_filter_indices: Vec<usize> = Vec::new();

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
            matched_filter_indices: Vec::new(),
        };
    }

    for (i, filter) in filters.iter().enumerate() {
        let filter_matches = match &filter.geometry.value {
            GeometryValue::FlowGeometry2D(filter_geo) => {
                // Project CityGML 3D polygons to 2D (drop Z) and test against 2D filter
                candidate_polygons.iter().any(|poly| {
                    let poly_2d: Polygon2D<f64> = (*poly).clone().into();
                    let candidate_geo = Geometry2D::Polygon(poly_2d);
                    test_predicate_2d(&candidate_geo, filter_geo, &params.predicate)
                })
            }
            GeometryValue::FlowGeometry3D(filter_geo) => {
                // Test if any candidate polygon matches the filter
                candidate_polygons
                    .iter()
                    .any(|poly| test_predicate_3d_poly_reverse(filter_geo, poly, &params.predicate))
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

        if filter_matches {
            match_count += 1;
            if match_any {
                return TestResult {
                    passed: true,
                    match_count,
                    matched_filter_indices: vec![i],
                };
            } else {
                matched_filter_indices.push(i);
            }
        } else if !match_any {
            return TestResult {
                passed: false,
                match_count,
                matched_filter_indices: Vec::new(),
            };
        }
    }

    TestResult {
        passed: if match_any { false } else { match_count > 0 },
        match_count,
        matched_filter_indices,
    }
}

/// Tests spatial predicates between 2D geometries using the DE-9IM (Dimensionally Extended 9-Intersection Model).
///
/// This function works with both pure 2D geometries (no Z coordinates) and 2D geometries with Z coordinates.
/// Pure 2D geometries are projected to the Z=0 plane for internal orientation calculations.
#[cfg(not(feature = "new-geometry"))]
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

#[cfg(not(feature = "new-geometry"))]
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

#[cfg(not(feature = "new-geometry"))]
fn test_predicate_3d_poly(
    candidate: &Geometry3D,
    filter: &reearth_flow_geometry::types::polygon::Polygon3D<f64>,
    predicate: &SpatialPredicate,
) -> bool {
    let filter_geo = Geometry3D::Polygon(filter.clone());
    test_predicate_3d(candidate, &filter_geo, predicate)
}

#[cfg(not(feature = "new-geometry"))]
fn test_predicate_3d_poly_reverse(
    filter: &Geometry3D,
    candidate: &reearth_flow_geometry::types::polygon::Polygon3D<f64>,
    predicate: &SpatialPredicate,
) -> bool {
    let candidate_geo = Geometry3D::Polygon(candidate.clone());
    test_predicate_3d(&candidate_geo, filter, predicate)
}

#[cfg(not(feature = "new-geometry"))]
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
    use pretty_assertions::assert_eq;
    use reearth_flow_runtime::forwarder::NoopChannelForwarder;
    use reearth_flow_types::feature::Attributes;

    use super::*;
    use crate::tests::utils::create_default_execute_context;

    // --- shared fixtures -----------------------------------------------------

    /// A closed CCW square ring with the corner at `min` and the given side.
    fn square_ring(min: [f64; 2], side: f64) -> Vec<[f64; 2]> {
        vec![
            [min[0], min[1]],
            [min[0] + side, min[1]],
            [min[0] + side, min[1] + side],
            [min[0], min[1] + side],
            [min[0], min[1]],
        ]
    }

    fn attributes(pairs: &[(&str, &str)]) -> Attributes {
        pairs
            .iter()
            .map(|(k, v)| {
                (
                    Attribute::new(k.to_string()),
                    AttributeValue::String(v.to_string()),
                )
            })
            .collect()
    }

    fn attribute(feature: &Feature, key: &str) -> Option<AttributeValue> {
        feature
            .attributes
            .get(&Attribute::new(key.to_string()))
            .cloned()
    }

    fn params(predicate: SpatialPredicate, match_mode: MatchMode) -> SpatialFilterParams {
        SpatialFilterParams {
            predicate,
            match_mode,
            ..Default::default()
        }
    }

    /// Feed filters and candidates through the processor and collect what the
    /// `passed`, `failed`, and `rejected` ports received.
    fn run(
        params: SpatialFilterParams,
        filters: Vec<Feature>,
        candidates: Vec<Feature>,
    ) -> (Vec<Feature>, Vec<Feature>, Vec<Feature>) {
        let mut processor = SpatialFilter {
            params,
            filters: Vec::new(),
            candidates: Vec::new(),
            #[cfg(feature = "new-geometry")]
            frame: None,
            #[cfg(feature = "new-geometry")]
            frame_mismatch_reported: false,
            #[cfg(feature = "new-geometry")]
            filters_received: 0,
        };
        let fw = ProcessorChannelForwarder::Noop(NoopChannelForwarder::default());
        for feature in &filters {
            let mut ctx = create_default_execute_context(feature);
            ctx.port = FILTER_PORT.clone();
            processor.process(ctx, &fw).unwrap();
        }
        for feature in &candidates {
            let mut ctx = create_default_execute_context(feature);
            ctx.port = CANDIDATE_PORT.clone();
            processor.process(ctx, &fw).unwrap();
        }
        processor.finish(NodeContext::default(), &fw).unwrap();

        let ProcessorChannelForwarder::Noop(noop) = fw else {
            unreachable!("the forwarder is the one built above");
        };
        let ports = noop.send_ports.lock().unwrap().clone();
        let sent = noop.send_features.lock().unwrap().clone();
        let (mut passed, mut failed, mut rejected) = (Vec::new(), Vec::new(), Vec::new());
        for (port, feature) in ports.iter().zip(sent) {
            if *port == *PASSED_PORT {
                passed.push(feature);
            } else if *port == *FAILED_PORT {
                failed.push(feature);
            } else if *port == *REJECTED_PORT {
                rejected.push(feature);
            } else {
                panic!("unexpected port {port:?}");
            }
        }
        (passed, failed, rejected)
    }

    // --- per-world fixtures ----------------------------------------------------

    /// A feature carrying a square with the corner at `min`.
    #[cfg(not(feature = "new-geometry"))]
    fn square(min: [f64; 2], side: f64) -> Feature {
        use reearth_flow_geometry::types::{coordinate::Coordinate2D, line_string::LineString2D};
        use reearth_flow_types::Geometry;

        let ring: Vec<Coordinate2D<f64>> = square_ring(min, side)
            .into_iter()
            .map(|[x, y]| Coordinate2D::new_(x, y))
            .collect();
        let polygon = Polygon2D::new(LineString2D::from(ring), vec![]);
        Feature::new_with_attributes_and_geometry(
            Attributes::new(),
            Geometry::with_value(GeometryValue::FlowGeometry2D(Geometry2D::Polygon(polygon))),
        )
    }

    #[cfg(feature = "new-geometry")]
    fn square(min: [f64; 2], side: f64) -> Feature {
        square_in(CoordinateFrame::Euclidean, min, side)
    }

    #[cfg(feature = "new-geometry")]
    fn square_in(frame: CoordinateFrame, min: [f64; 2], side: f64) -> Feature {
        use reearth_flow_geometry::{polygon::Polygon2D, Euclidean2DGeometry};

        Feature::new_with_attributes_and_geometry(
            Attributes::new(),
            Geometry::Euclidean2D(Euclidean2DGeometry::Polygon(Box::new(
                Polygon2D::from_rings(frame, square_ring(min, side), Vec::<Vec<[f64; 2]>>::new()),
            ))),
        )
    }

    fn with_attrs(mut feature: Feature, pairs: &[(&str, &str)]) -> Feature {
        feature.attributes = std::sync::Arc::new(attributes(pairs));
        feature
    }

    // --- shared behavior ---------------------------------------------------------

    #[test]
    fn intersecting_candidates_split_between_passed_and_failed() {
        let (passed, failed, rejected) = run(
            params(SpatialPredicate::Intersects, MatchMode::Any),
            vec![square([5.0, 5.0], 10.0)],
            vec![square([0.0, 0.0], 10.0), square([20.0, 20.0], 5.0)],
        );
        assert_eq!(passed.len(), 1, "the overlapping candidate passes");
        assert_eq!(failed.len(), 1, "the distant candidate fails");
        assert_eq!(rejected.len(), 0);
    }

    #[test]
    fn no_filters_pass_all_candidates() {
        let (passed, failed, rejected) = run(
            params(SpatialPredicate::Intersects, MatchMode::Any),
            Vec::new(),
            vec![square([0.0, 0.0], 10.0), square([20.0, 20.0], 5.0)],
        );
        assert_eq!(passed.len(), 2, "no filters means no restrictions");
        assert_eq!(failed.len(), 0);
        assert_eq!(rejected.len(), 0);
    }

    #[test]
    fn features_without_geometry_are_rejected() {
        let (passed, failed, rejected) = run(
            params(SpatialPredicate::Intersects, MatchMode::Any),
            vec![square([0.0, 0.0], 10.0)],
            vec![Feature::new_with_attributes(Attributes::new())],
        );
        assert_eq!(passed.len(), 0);
        assert_eq!(failed.len(), 0);
        assert_eq!(rejected.len(), 1, "a feature with no geometry is rejected");
    }

    #[test]
    fn merged_attributes_take_the_configured_prefix() {
        let (passed, _, _) = run(
            SpatialFilterParams {
                merge_filter_attributes: true,
                merged_attributes_prefix: Some("filter_".to_string()),
                ..Default::default()
            },
            vec![with_attrs(square([5.0, 5.0], 10.0), &[("name", "zone_a")])],
            vec![square([0.0, 0.0], 10.0)],
        );
        assert_eq!(passed.len(), 1);
        assert_eq!(
            attribute(&passed[0], "filter_name"),
            Some(AttributeValue::String("zone_a".to_string())),
            "the attribute appears under the prefixed key"
        );
        assert_eq!(
            attribute(&passed[0], "name"),
            None,
            "the attribute does not appear under the unprefixed key"
        );
    }

    #[test]
    fn failed_candidates_get_no_merged_attributes() {
        let (passed, failed, _) = run(
            SpatialFilterParams {
                merge_filter_attributes: true,
                ..Default::default()
            },
            vec![with_attrs(
                square([5.0, 5.0], 10.0),
                &[("zone", "commercial")],
            )],
            vec![square([20.0, 20.0], 5.0)],
        );
        assert_eq!(passed.len(), 0);
        assert_eq!(failed.len(), 1);
        assert_eq!(
            attribute(&failed[0], "zone"),
            None,
            "filter attributes are not merged onto failed candidates"
        );
    }

    #[test]
    fn all_mode_merges_every_filter() {
        let (passed, _, _) = run(
            SpatialFilterParams {
                match_mode: MatchMode::All,
                merge_filter_attributes: true,
                ..Default::default()
            },
            vec![
                with_attrs(square([5.0, 5.0], 10.0), &[("zone", "commercial")]),
                with_attrs(square([-5.0, -5.0], 10.0), &[("category", "retail")]),
            ],
            vec![square([0.0, 0.0], 10.0)],
        );
        assert_eq!(passed.len(), 1);
        assert_eq!(
            attribute(&passed[0], "zone"),
            Some(AttributeValue::String("commercial".to_string()))
        );
        assert_eq!(
            attribute(&passed[0], "category"),
            Some(AttributeValue::String("retail".to_string()))
        );
    }

    // --- new geometry ------------------------------------------------------------

    #[cfg(feature = "new-geometry")]
    fn match_count(feature: &Feature) -> Option<u64> {
        match attribute(feature, "matches") {
            Some(AttributeValue::Number(n)) => n.as_u64(),
            _ => None,
        }
    }

    #[cfg(feature = "new-geometry")]
    #[test]
    fn any_mode_match_count_reports_every_match() {
        let (passed, _, _) = run(
            SpatialFilterParams {
                output_match_count_attribute: Some(Attribute::new("matches")),
                ..Default::default()
            },
            vec![square([5.0, 5.0], 10.0), square([-5.0, -5.0], 10.0)],
            vec![square([0.0, 0.0], 10.0)],
        );
        assert_eq!(passed.len(), 1);
        assert_eq!(
            match_count(&passed[0]),
            Some(2),
            "the count reports every matching filter, not the first"
        );
    }

    #[cfg(feature = "new-geometry")]
    #[test]
    fn all_mode_failure_still_reports_the_true_count() {
        let (passed, failed, _) = run(
            SpatialFilterParams {
                match_mode: MatchMode::All,
                output_match_count_attribute: Some(Attribute::new("matches")),
                ..Default::default()
            },
            vec![square([5.0, 5.0], 10.0), square([100.0, 100.0], 5.0)],
            vec![square([0.0, 0.0], 10.0)],
        );
        assert_eq!(passed.len(), 0);
        assert_eq!(failed.len(), 1);
        assert_eq!(
            match_count(&failed[0]),
            Some(1),
            "the failed candidate reports how many filters it did match"
        );
    }

    #[cfg(feature = "new-geometry")]
    #[test]
    fn any_mode_merges_every_matching_filter_with_last_value_winning() {
        let (passed, _, _) = run(
            SpatialFilterParams {
                merge_filter_attributes: true,
                ..Default::default()
            },
            vec![
                with_attrs(square([5.0, 5.0], 10.0), &[("zone", "first"), ("a", "1")]),
                with_attrs(
                    square([-5.0, -5.0], 10.0),
                    &[("zone", "second"), ("b", "2")],
                ),
                with_attrs(square([100.0, 100.0], 5.0), &[("zone", "unmatched")]),
            ],
            vec![square([0.0, 0.0], 10.0)],
        );
        assert_eq!(passed.len(), 1);
        assert_eq!(
            attribute(&passed[0], "zone"),
            Some(AttributeValue::String("second".to_string())),
            "the last matching filter wins a shared attribute"
        );
        assert_eq!(
            attribute(&passed[0], "a"),
            Some(AttributeValue::String("1".to_string())),
            "attributes from every matching filter are merged"
        );
        assert_eq!(
            attribute(&passed[0], "b"),
            Some(AttributeValue::String("2".to_string()))
        );
    }

    #[cfg(feature = "new-geometry")]
    #[test]
    fn disjoint_counts_filters_outside_the_bounding_box() {
        let (passed, failed, _) = run(
            SpatialFilterParams {
                predicate: SpatialPredicate::Disjoint,
                match_mode: MatchMode::All,
                output_match_count_attribute: Some(Attribute::new("matches")),
                ..Default::default()
            },
            vec![square([100.0, 100.0], 5.0), square([200.0, 200.0], 5.0)],
            vec![square([0.0, 0.0], 10.0), square([98.0, 98.0], 10.0)],
        );
        assert_eq!(
            passed.len(),
            1,
            "the candidate away from both filters passes"
        );
        assert_eq!(match_count(&passed[0]), Some(2));
        assert_eq!(failed.len(), 1, "the candidate overlapping a filter fails");
        assert_eq!(match_count(&failed[0]), Some(1));
    }

    #[cfg(feature = "new-geometry")]
    #[test]
    fn candidate_relative_containment_predicates() {
        // The candidate square strictly contains the small filter square.
        let (passed, failed, _) = run(
            params(SpatialPredicate::Contains, MatchMode::Any),
            vec![square([4.0, 4.0], 2.0)],
            vec![square([0.0, 0.0], 10.0), square([4.5, 4.5], 1.0)],
        );
        assert_eq!(
            passed.len(),
            1,
            "only the enclosing candidate contains the filter"
        );
        assert_eq!(failed.len(), 1);

        // And `within` is the converse: the small candidate lies inside the filter.
        let (passed, failed, _) = run(
            params(SpatialPredicate::Within, MatchMode::Any),
            vec![square([0.0, 0.0], 10.0)],
            vec![square([4.0, 4.0], 2.0), square([20.0, 20.0], 5.0)],
        );
        assert_eq!(
            passed.len(),
            1,
            "only the enclosed candidate is within the filter"
        );
        assert_eq!(failed.len(), 1);
    }

    #[cfg(feature = "new-geometry")]
    #[test]
    fn three_dimensional_candidates_compare_by_their_footprint() {
        use reearth_flow_geometry::{polygon::Polygon3D, Euclidean3DGeometry};

        // A roof panel at z = 25 whose footprint overlaps the 2D filter zone.
        let roof = Feature::new_with_attributes_and_geometry(
            Attributes::new(),
            Geometry::Euclidean3D(Euclidean3DGeometry::Polygon(Box::new(
                Polygon3D::from_rings(
                    CoordinateFrame::Euclidean,
                    square_ring([0.0, 0.0], 10.0)
                        .into_iter()
                        .map(|[x, y]| [x, y, 25.0]),
                    Vec::<Vec<[f64; 3]>>::new(),
                ),
            ))),
        );
        let (passed, failed, rejected) = run(
            params(SpatialPredicate::Intersects, MatchMode::Any),
            vec![square([5.0, 5.0], 10.0)],
            vec![roof],
        );
        assert_eq!(
            passed.len(),
            1,
            "the elevated candidate matches by footprint"
        );
        assert_eq!(failed.len(), 0);
        assert_eq!(rejected.len(), 0);
    }

    #[cfg(feature = "new-geometry")]
    #[test]
    fn closed_shells_compare_by_their_dissolved_footprint() {
        use reearth_flow_geometry::{triangular_mesh::TriangularMesh3D, Euclidean3DGeometry};

        // A closed box: its faces overlap in (x, y), which the footprint
        // dissolves into one square before the 2D test.
        let corners = [
            [0.0, 0.0, 0.0],
            [4.0, 0.0, 0.0],
            [4.0, 4.0, 0.0],
            [0.0, 4.0, 0.0],
            [0.0, 0.0, 4.0],
            [4.0, 0.0, 4.0],
            [4.0, 4.0, 4.0],
            [0.0, 4.0, 4.0],
        ];
        let triangles: [u32; 36] = [
            0, 2, 1, 0, 3, 2, // bottom
            4, 5, 6, 4, 6, 7, // top
            0, 1, 5, 0, 5, 4, // front
            1, 2, 6, 1, 6, 5, // right
            2, 3, 7, 2, 7, 6, // back
            3, 0, 4, 3, 4, 7, // left
        ];
        let mesh =
            TriangularMesh3D::from_parts(CoordinateFrame::Euclidean, corners.to_vec(), triangles)
                .unwrap();
        let building = Feature::new_with_attributes_and_geometry(
            Attributes::new(),
            Geometry::Euclidean3D(Euclidean3DGeometry::TriangularMesh(Box::new(mesh))),
        );
        let (passed, failed, rejected) = run(
            params(SpatialPredicate::Within, MatchMode::Any),
            vec![square([-1.0, -1.0], 10.0)],
            vec![building],
        );
        assert_eq!(
            passed.len(),
            1,
            "the shell's footprint lies within the filter"
        );
        assert_eq!(failed.len(), 0);
        assert_eq!(rejected.len(), 0);
    }

    #[cfg(feature = "new-geometry")]
    #[test]
    fn three_dimensional_input_in_angular_frames_is_rejected() {
        use reearth_flow_geometry::{
            coordinate::EpsgCode, polygon::Polygon3D, Euclidean3DGeometry,
        };

        // EPSG:4326 is geographic (degrees): no horizontal footprint.
        let candidate = Feature::new_with_attributes_and_geometry(
            Attributes::new(),
            Geometry::Euclidean3D(Euclidean3DGeometry::Polygon(Box::new(
                Polygon3D::from_rings(
                    CoordinateFrame::Crs(EpsgCode::new(4326)),
                    square_ring([0.0, 0.0], 1.0)
                        .into_iter()
                        .map(|[x, y]| [x, y, 25.0]),
                    Vec::<Vec<[f64; 3]>>::new(),
                ),
            ))),
        );
        let (passed, failed, rejected) = run(
            params(SpatialPredicate::Intersects, MatchMode::Any),
            vec![square_in(
                CoordinateFrame::Crs(EpsgCode::new(4326)),
                [0.0, 0.0],
                1.0,
            )],
            vec![candidate],
        );
        assert_eq!(passed.len(), 0);
        assert_eq!(failed.len(), 0);
        assert_eq!(rejected.len(), 1, "an angular-frame 3D feature is rejected");
    }

    #[cfg(feature = "new-geometry")]
    #[test]
    fn frames_other_than_the_first_accepted_one_are_rejected() {
        use reearth_flow_geometry::coordinate::EpsgCode;

        let (passed, failed, rejected) = run(
            params(SpatialPredicate::Intersects, MatchMode::Any),
            vec![square([5.0, 5.0], 10.0)],
            vec![
                square_in(CoordinateFrame::Crs(EpsgCode::new(6677)), [0.0, 0.0], 10.0),
                square([0.0, 0.0], 10.0),
            ],
        );
        assert_eq!(rejected.len(), 1, "the off-frame candidate is rejected");
        assert_eq!(passed.len(), 1, "the same-frame candidate still passes");
        assert_eq!(failed.len(), 0);
    }

    #[cfg(feature = "new-geometry")]
    #[test]
    fn candidates_fail_when_every_supplied_filter_was_rejected() {
        // A filter set emptied by rejection is not the same as no filter set:
        // treating it as one would turn an upstream error into a pass-everything.
        let (passed, failed, rejected) = run(
            params(SpatialPredicate::Intersects, MatchMode::Any),
            vec![Feature::new_with_attributes(Attributes::new())],
            vec![square([0.0, 0.0], 10.0)],
        );
        assert_eq!(rejected.len(), 1, "the unusable filter is rejected");
        assert_eq!(passed.len(), 0, "the candidate is not passed unrestricted");
        assert_eq!(failed.len(), 1, "it fails the condition instead");
    }

    #[cfg(feature = "new-geometry")]
    #[test]
    fn the_match_count_is_stamped_even_when_there_is_no_filter_to_test() {
        // The count is documented as written to passing and failing candidates
        // alike, so it cannot be skipped on the routes that never run a test.
        let (passed, _, _) = run(
            SpatialFilterParams {
                output_match_count_attribute: Some(Attribute::new("matches")),
                ..Default::default()
            },
            Vec::new(),
            vec![square([0.0, 0.0], 10.0)],
        );
        assert_eq!(passed.len(), 1);
        assert_eq!(
            match_count(&passed[0]),
            Some(0),
            "a candidate passed for want of any filter matched none of them"
        );

        let (_, failed, _) = run(
            SpatialFilterParams {
                output_match_count_attribute: Some(Attribute::new("matches")),
                ..Default::default()
            },
            vec![Feature::new_with_attributes(Attributes::new())],
            vec![square([0.0, 0.0], 10.0)],
        );
        assert_eq!(failed.len(), 1);
        assert_eq!(
            match_count(&failed[0]),
            Some(0),
            "so did a candidate failed because every filter was unusable"
        );
    }
}
