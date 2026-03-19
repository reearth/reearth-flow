use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;

use once_cell::sync::Lazy;
use reearth_flow_geometry::algorithm::centroid::Centroid;
use reearth_flow_runtime::cache::executor_cache_subdir;
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
use crate::ACCUMULATOR_BUFFER_BYTE_THRESHOLD;

static BASE_PORT: Lazy<Port> = Lazy::new(|| Port::new("base"));
static CANDIDATE_PORT: Lazy<Port> = Lazy::new(|| Port::new("candidate"));
static MATCHED_PORT: Lazy<Port> = Lazy::new(|| Port::new("matched"));
static UNMATCHED_PORT: Lazy<Port> = Lazy::new(|| Port::new("unmatched"));

/// Earth's radius in meters for haversine calculations
#[allow(dead_code)]
const EARTH_RADIUS_METERS: f64 = 6_371_000.0;

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

        // Validate num_closest >= 1
        if params.num_closest < 1 {
            return Err(GeometryProcessorError::NeighborFinderFactory(
                "numClosest must be >= 1".to_string(),
            )
            .into());
        }

        Ok(Box::new(NeighborFinder {
            params,
            candidates: Vec::new(),
            base_features: Vec::new(),
            buffer_bytes: 0,
            temp_dir: None,
            candidate_chunk_count: 0,
            base_chunk_count: 0,
            executor_id: None,
        }))
    }
}

/// Distance metric for computing spatial proximity.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema, Default)]
#[serde(rename_all = "camelCase")]
pub enum DistanceMetric {
    /// 2D Euclidean distance using X and Y coordinates. Z is ignored.
    #[default]
    Euclidean2d,
    /// 3D Euclidean distance using X, Y, and Z coordinates.
    Euclidean3d,
    /// Great-circle distance treating X as longitude (degrees) and Y as latitude (degrees).
    /// Output is in meters. Intended for WGS-84 inputs.
    Haversine,
}

/// Merge strategy for handling multiple neighbors.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema, Default)]
#[serde(rename_all = "camelCase")]
pub enum MergeStrategy {
    /// Only the single closest neighbor is merged onto the Base feature.
    #[default]
    Closest,
    /// One output feature is emitted per neighbor. Base attributes are duplicated
    /// across all rows, mirroring FME's default multi-neighbor output.
    RepeatBase,
    /// A single Base feature is emitted; each transferred attribute becomes an ordered
    /// array sorted by ascending distance.
    ArrayAttributes,
}

/// # NeighborFinder Parameters
///
/// Configuration for finding spatial neighbors between base and candidate features.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct NeighborFinderParams {
    /// Number of closest neighbors to find per base feature. Must be >= 1.
    #[serde(default = "default_num_closest")]
    pub num_closest: usize,

    /// Maximum distance threshold for matching. If None, no distance limit is applied.
    /// Units depend on the distanceMetric: native units for Euclidean, meters for Haversine.
    pub max_distance: Option<f64>,

    /// Name of the attribute to store the computed distance to the nearest neighbor.
    #[serde(default = "default_distance_attribute")]
    pub distance_attribute: Attribute,

    /// Name of the attribute to store the neighbor index (0-based rank) when
    /// num_closest > 1 and merge_strategy is "repeatBase". Set to empty string to suppress.
    #[serde(default = "default_neighbor_index_attribute")]
    pub neighbor_index_attribute: Attribute,

    /// Prefix applied to transferred candidate attributes to avoid collisions.
    #[serde(default = "default_attribute_prefix")]
    pub attribute_prefix: String,

    /// List of candidate attributes to transfer. Empty list means all attributes are transferred.
    #[serde(default)]
    pub attributes_to_transfer: Vec<Attribute>,

    /// Controls how multiple neighbors are represented on the output.
    #[serde(default)]
    pub merge_strategy: MergeStrategy,

    /// Method used to compute distance between two features.
    #[serde(default)]
    pub distance_metric: DistanceMetric,
}

impl Default for NeighborFinderParams {
    fn default() -> Self {
        Self {
            num_closest: default_num_closest(),
            max_distance: None,
            distance_attribute: default_distance_attribute(),
            neighbor_index_attribute: default_neighbor_index_attribute(),
            attribute_prefix: default_attribute_prefix(),
            attributes_to_transfer: Vec::new(),
            merge_strategy: MergeStrategy::default(),
            distance_metric: DistanceMetric::default(),
        }
    }
}

fn default_num_closest() -> usize {
    1
}

fn default_distance_attribute() -> Attribute {
    Attribute::new("_neighbor_distance")
}

fn default_neighbor_index_attribute() -> Attribute {
    Attribute::new("_neighbor_index")
}

fn default_attribute_prefix() -> String {
    "_neighbor_".to_string()
}

/// A neighbor match result containing the candidate and computed distance.
#[derive(Debug, Clone)]
struct NeighborMatch {
    candidate: CandidateEntry,
    distance: f64,
    index: usize, // 0-based rank
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CandidateEntry {
    point: [f64; 2],
    point_z: Option<f64>, // Z coordinate for 3D distance calculations
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
    // Disk spilling fields
    buffer_bytes: usize,
    temp_dir: Option<PathBuf>,
    candidate_chunk_count: usize,
    base_chunk_count: usize,
    executor_id: Option<uuid::Uuid>,
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
        // Capture executor_id on first process call for cache isolation
        if self.executor_id.is_none() {
            self.executor_id = Some(fw.executor_id());
            // Initialize temp_dir now that we have executor_id
            let temp_dir = executor_cache_subdir(fw.executor_id(), "neighbor_finder");
            self.temp_dir = Some(temp_dir);
        }

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
                // Check if we need to spill to disk
                let feature_bytes = serde_json::to_vec(feature)?.len();
                if self.buffer_bytes + feature_bytes >= ACCUMULATOR_BUFFER_BYTE_THRESHOLD {
                    self.flush_base_features()?;
                }
                self.base_features.push(feature.clone());
                self.buffer_bytes += feature_bytes;
            }
            port if port == &*CANDIDATE_PORT => {
                // Extract representative point and store candidate
                if let Some((point, point_z)) = extract_representative_point(feature) {
                    let entry = CandidateEntry {
                        point,
                        point_z,
                        feature: feature.clone(),
                    };
                    let entry_bytes = serde_json::to_vec(&entry)?.len();
                    if self.buffer_bytes + entry_bytes >= ACCUMULATOR_BUFFER_BYTE_THRESHOLD {
                        self.flush_candidates()?;
                    }
                    self.candidates.push(entry);
                    self.buffer_bytes += entry_bytes;
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
        // Load any spilled data
        self.load_from_disk()?;

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

        // Process each base feature (parallel processing if feature count is large)
        let base_count = self.base_features.len();
        let use_parallel = base_count >= 100; // Use parallel processing for large datasets

        if use_parallel {
            self.process_parallel(&ctx, fw, &rtree)?;
        } else {
            self.process_sequential(&ctx, fw, &rtree)?;
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "NeighborFinder"
    }
}

impl NeighborFinder {
    /// Flush base features to disk (zstd-compressed JSONL)
    fn flush_base_features(&mut self) -> Result<(), BoxedError> {
        if self.base_features.is_empty() {
            return Ok(());
        }

        let chunk_path = self
            .temp_dir
            .as_ref()
            .unwrap()
            .join(format!("base_chunk_{}.jsonl.zst", self.base_chunk_count));
        let file = File::create(&chunk_path)?;
        let mut writer = BufWriter::new(zstd::Encoder::new(file, 3)?);

        for feature in &self.base_features {
            let line = serde_json::to_vec(feature)?;
            writer.write_all(&line)?;
            writer.write_all(b"\n")?;
        }
        writer.flush()?;

        self.base_chunk_count += 1;
        self.base_features.clear();
        self.buffer_bytes = 0;
        Ok(())
    }

    /// Flush candidates to disk (zstd-compressed JSONL)
    fn flush_candidates(&mut self) -> Result<(), BoxedError> {
        if self.candidates.is_empty() {
            return Ok(());
        }

        let chunk_path = self.temp_dir.as_ref().unwrap().join(format!(
            "candidate_chunk_{}.jsonl.zst",
            self.candidate_chunk_count
        ));
        let file = File::create(&chunk_path)?;
        let mut writer = BufWriter::new(zstd::Encoder::new(file, 3)?);

        for entry in &self.candidates {
            let line = serde_json::to_vec(entry)?;
            writer.write_all(&line)?;
            writer.write_all(b"\n")?;
        }
        writer.flush()?;

        self.candidate_chunk_count += 1;
        self.candidates.clear();
        self.buffer_bytes = 0;
        Ok(())
    }

    /// Load spilled data from disk back into memory
    fn load_from_disk(&mut self) -> Result<(), BoxedError> {
        // Load base features
        for i in 0..self.base_chunk_count {
            let chunk_path = self
                .temp_dir
                .as_ref()
                .unwrap()
                .join(format!("base_chunk_{}.jsonl.zst", i));
            let file = File::open(&chunk_path)?;
            let reader = BufReader::new(zstd::Decoder::new(file)?);

            for line in reader.lines() {
                let line = line?;
                let feature: Feature = serde_json::from_str(&line)?;
                self.base_features.push(feature);
            }
        }

        // Load candidates
        for i in 0..self.candidate_chunk_count {
            let chunk_path = self
                .temp_dir
                .as_ref()
                .unwrap()
                .join(format!("candidate_chunk_{}.jsonl.zst", i));
            let file = File::open(&chunk_path)?;
            let reader = BufReader::new(zstd::Decoder::new(file)?);

            for line in reader.lines() {
                let line = line?;
                let entry: CandidateEntry = serde_json::from_str(&line)?;
                self.candidates.push(entry);
            }
        }

        Ok(())
    }

    /// Process base features sequentially
    fn process_sequential(
        &self,
        ctx: &NodeContext,
        fw: &ProcessorChannelForwarder,
        rtree: &RTree<CandidateEntry>,
    ) -> Result<(), BoxedError> {
        for base in &self.base_features {
            self.process_single_base(ctx, fw, rtree, base)?;
        }
        Ok(())
    }

    /// Process base features in parallel using rayon
    fn process_parallel(
        &self,
        ctx: &NodeContext,
        fw: &ProcessorChannelForwarder,
        rtree: &RTree<CandidateEntry>,
    ) -> Result<(), BoxedError> {
        use rayon::prelude::*;

        // Create thread-safe references
        let results: Vec<_> = self
            .base_features
            .par_iter()
            .map(|base| {
                let result = self.find_neighbors_for_base(rtree, base);
                (base, result)
            })
            .collect();

        // Send results (must be done sequentially for the forwarder)
        for (base, result) in results {
            match result {
                Ok((Some(matches), false)) => {
                    if matches.is_empty() {
                        fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                            ctx,
                            base.clone(),
                            UNMATCHED_PORT.clone(),
                        ));
                    } else {
                        self.emit_matches(ctx, fw, base, &matches)?;
                    }
                }
                Ok((None, true)) => {
                    fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                        ctx,
                        base.clone(),
                        REJECTED_PORT.clone(),
                    ));
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// Find neighbors for a single base feature (used in parallel mode)
    fn find_neighbors_for_base(
        &self,
        rtree: &RTree<CandidateEntry>,
        base: &Feature,
    ) -> Result<(Option<Vec<NeighborMatch>>, bool), BoxedError> {
        let Some((base_point, base_z)) = extract_representative_point(base) else {
            return Ok((None, true)); // None for matches, true indicates geometry error
        };

        let neighbor_matches = find_k_nearest_neighbors(
            rtree,
            &base_point,
            base_z,
            self.params.num_closest,
            &self.params.distance_metric,
        );

        // Filter by max_distance and compute actual distance based on metric
        let filtered_matches: Vec<NeighborMatch> = neighbor_matches
            .into_iter()
            .filter(|m| {
                if let Some(max_dist) = self.params.max_distance {
                    m.distance <= max_dist
                } else {
                    true
                }
            })
            .collect();

        if filtered_matches.is_empty() {
            return Ok((Some(vec![]), false)); // Empty matches means no matches found
        }

        Ok((Some(filtered_matches), false))
    }

    /// Process a single base feature
    fn process_single_base(
        &self,
        ctx: &NodeContext,
        fw: &ProcessorChannelForwarder,
        rtree: &RTree<CandidateEntry>,
        base: &Feature,
    ) -> Result<(), BoxedError> {
        let Some((base_point, base_z)) = extract_representative_point(base) else {
            fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                ctx,
                base.clone(),
                REJECTED_PORT.clone(),
            ));
            return Ok(());
        };

        let neighbor_matches = find_k_nearest_neighbors(
            rtree,
            &base_point,
            base_z,
            self.params.num_closest,
            &self.params.distance_metric,
        );

        // Filter by max_distance
        let filtered_matches: Vec<NeighborMatch> = neighbor_matches
            .into_iter()
            .filter(|m| {
                if let Some(max_dist) = self.params.max_distance {
                    m.distance <= max_dist
                } else {
                    true
                }
            })
            .collect();

        if filtered_matches.is_empty() {
            fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                ctx,
                base.clone(),
                UNMATCHED_PORT.clone(),
            ));
            return Ok(());
        }

        self.emit_matches(ctx, fw, base, &filtered_matches)
    }

    /// Emit matches based on merge strategy
    fn emit_matches(
        &self,
        ctx: &NodeContext,
        fw: &ProcessorChannelForwarder,
        base: &Feature,
        filtered_matches: &[NeighborMatch],
    ) -> Result<(), BoxedError> {
        match self.params.merge_strategy {
            MergeStrategy::Closest => {
                let enriched = self.create_enriched_feature(base, &filtered_matches[0], false);
                fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                    ctx,
                    enriched,
                    MATCHED_PORT.clone(),
                ));
            }
            MergeStrategy::RepeatBase => {
                for neighbor_match in filtered_matches {
                    let enriched = self.create_enriched_feature(base, neighbor_match, true);
                    fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                        ctx,
                        enriched,
                        MATCHED_PORT.clone(),
                    ));
                }
            }
            MergeStrategy::ArrayAttributes => {
                let enriched = self.create_array_enriched_feature(base, filtered_matches);
                fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                    ctx,
                    enriched,
                    MATCHED_PORT.clone(),
                ));
            }
        }
        Ok(())
    }

    /// Create an enriched feature with distance and transferred attributes.
    fn create_enriched_feature(
        &self,
        base: &Feature,
        neighbor_match: &NeighborMatch,
        include_index: bool,
    ) -> Feature {
        let mut enriched = base.clone();

        // Add distance attribute
        enriched.attributes_mut().insert(
            self.params.distance_attribute.clone(),
            AttributeValue::Number(
                serde_json::Number::from_f64(neighbor_match.distance)
                    .unwrap_or_else(|| serde_json::Number::from(0)),
            ),
        );

        // Add neighbor index attribute (if not suppressed by empty string and requested)
        if include_index {
            let index_attr = &self.params.neighbor_index_attribute;
            if !index_attr.as_ref().is_empty() {
                enriched.attributes_mut().insert(
                    index_attr.clone(),
                    AttributeValue::Number(serde_json::Number::from(neighbor_match.index as u64)),
                );
            }
        }

        // Transfer candidate attributes with prefix
        self.transfer_attributes(&mut enriched, &neighbor_match.candidate);

        enriched
    }

    /// Create an enriched feature with array-valued attributes for all neighbors.
    fn create_array_enriched_feature(
        &self,
        base: &Feature,
        neighbor_matches: &[NeighborMatch],
    ) -> Feature {
        let mut enriched = base.clone();
        let prefix = &self.params.attribute_prefix;

        // Collect distances into an array
        let distances: Vec<AttributeValue> = neighbor_matches
            .iter()
            .map(|m| {
                AttributeValue::Number(
                    serde_json::Number::from_f64(m.distance)
                        .unwrap_or_else(|| serde_json::Number::from(0)),
                )
            })
            .collect();
        enriched.attributes_mut().insert(
            self.params.distance_attribute.clone(),
            AttributeValue::Array(distances),
        );

        // Collect all unique attribute names from all candidates
        let mut all_attrs: Vec<Attribute> = Vec::new();
        for m in neighbor_matches {
            let candidate_attrs: Vec<_> = if self.params.attributes_to_transfer.is_empty() {
                m.candidate.feature.attributes.keys().cloned().collect()
            } else {
                self.params.attributes_to_transfer.clone()
            };
            for attr in candidate_attrs {
                if !all_attrs.contains(&attr) {
                    all_attrs.push(attr);
                }
            }
        }

        // For each attribute, collect values from all neighbors into an array
        for attr in all_attrs {
            let values: Vec<AttributeValue> = neighbor_matches
                .iter()
                .map(|m| {
                    m.candidate
                        .feature
                        .attributes
                        .get(&attr)
                        .cloned()
                        .unwrap_or(AttributeValue::Null)
                })
                .collect();
            let prefixed_attr = Attribute::new(format!("{}{}", prefix, attr));
            enriched
                .attributes_mut()
                .insert(prefixed_attr, AttributeValue::Array(values));
        }

        enriched
    }

    /// Transfer candidate attributes to the enriched feature with prefix.
    fn transfer_attributes(&self, enriched: &mut Feature, candidate: &CandidateEntry) {
        let prefix = &self.params.attribute_prefix;
        let attrs_to_transfer: Vec<_> = if self.params.attributes_to_transfer.is_empty() {
            candidate.feature.attributes.keys().cloned().collect()
        } else {
            self.params.attributes_to_transfer.clone()
        };

        for attr in attrs_to_transfer {
            if let Some(value) = candidate.feature.attributes.get(&attr) {
                let prefixed_attr = Attribute::new(format!("{}{}", prefix, attr));
                enriched
                    .attributes_mut()
                    .insert(prefixed_attr, value.clone());
            }
        }
    }
}

/// Find the k nearest neighbors for a given base point using the R-tree.
fn find_k_nearest_neighbors(
    rtree: &RTree<CandidateEntry>,
    base_point: &[f64; 2],
    base_z: Option<f64>,
    k: usize,
    metric: &DistanceMetric,
) -> Vec<NeighborMatch> {
    rtree
        .nearest_neighbor_iter(base_point)
        .take(k)
        .cloned()
        .enumerate()
        .map(|(index, candidate)| {
            // Compute distance based on the specified metric
            let distance = compute_distance(
                metric,
                base_point,
                base_z,
                &candidate.point,
                candidate.point_z,
            );
            NeighborMatch {
                candidate,
                distance,
                index,
            }
        })
        .collect()
}

/// Compute 2D Euclidean distance between two points
fn euclidean_distance_2d(a: &[f64; 2], b: &[f64; 2]) -> f64 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    (dx * dx + dy * dy).sqrt()
}

/// Compute 3D Euclidean distance between two points
fn euclidean_distance_3d(a: &[f64; 2], a_z: f64, b: &[f64; 2], b_z: f64) -> f64 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a_z - b_z;
    (dx * dx + dy * dy + dz * dz).sqrt()
}

/// Compute great-circle distance using the Haversine formula.
/// Input: longitude and latitude in degrees. Output: meters.
fn haversine_distance(a: &[f64; 2], b: &[f64; 2]) -> f64 {
    let lon1 = a[0].to_radians();
    let lat1 = a[1].to_radians();
    let lon2 = b[0].to_radians();
    let lat2 = b[1].to_radians();

    let dlon = lon2 - lon1;
    let dlat = lat2 - lat1;

    let a = (dlat / 2.0).sin().powi(2) + lat1.cos() * lat2.cos() * (dlon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

    EARTH_RADIUS_METERS * c
}

/// Compute distance between two points based on the specified metric
fn compute_distance(
    metric: &DistanceMetric,
    a: &[f64; 2],
    a_z: Option<f64>,
    b: &[f64; 2],
    b_z: Option<f64>,
) -> f64 {
    match metric {
        DistanceMetric::Euclidean2d => euclidean_distance_2d(a, b),
        DistanceMetric::Euclidean3d => {
            let az = a_z.unwrap_or(0.0);
            let bz = b_z.unwrap_or(0.0);
            euclidean_distance_3d(a, az, b, bz)
        }
        DistanceMetric::Haversine => haversine_distance(a, b),
    }
}

/// Extract representative point from a feature's geometry.
/// Returns (xy, z) where z is None for 2D geometries.
fn extract_representative_point(feature: &Feature) -> Option<([f64; 2], Option<f64>)> {
    match &feature.geometry.value {
        GeometryValue::None => None,
        GeometryValue::FlowGeometry2D(geo) => {
            extract_representative_point_2d(geo).map(|p| (p, None))
        }
        GeometryValue::FlowGeometry3D(geo) => extract_representative_point_3d(geo),
        GeometryValue::CityGmlGeometry(citygml) => {
            // For CityGML, use centroid of highest LOD polygons
            let mut all_points = Vec::new();
            for gml in &citygml.gml_geometries {
                for poly in &gml.polygons {
                    for ring in poly.rings() {
                        for coord in ring.iter() {
                            all_points.push([coord.x, coord.y, coord.z]);
                        }
                    }
                }
            }
            if all_points.is_empty() {
                None
            } else {
                let centroid = centroid_weighted(&all_points);
                Some(([centroid[0], centroid[1]], Some(centroid[2])))
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
        Geometry2D::MultiPoint(mp) => mp.centroid().map(|c| [c.x(), c.y()]),
        Geometry2D::LineString(ls) => ls.centroid().map(|c| [c.x(), c.y()]),
        Geometry2D::Polygon(poly) => poly.centroid().map(|c| [c.x(), c.y()]),
        Geometry2D::MultiPolygon(mp) => mp.centroid().map(|c| [c.x(), c.y()]),
        _ => None,
    }
}

fn extract_representative_point_3d(
    geo: &reearth_flow_geometry::types::geometry::Geometry3D<f64>,
) -> Option<([f64; 2], Option<f64>)> {
    use reearth_flow_geometry::types::geometry::Geometry3D;

    match geo {
        Geometry3D::Point(p) => Some(([p.x(), p.y()], Some(p.z()))),
        Geometry3D::MultiPoint(mp) => mp.centroid().map(|c| ([c.x(), c.y()], Some(c.z()))),
        Geometry3D::LineString(ls) => ls.centroid().map(|c| ([c.x(), c.y()], Some(c.z()))),
        Geometry3D::Polygon(poly) => poly.centroid().map(|c| ([c.x(), c.y()], Some(c.z()))),
        Geometry3D::MultiPolygon(mp) => mp.centroid().map(|c| ([c.x(), c.y()], Some(c.z()))),
        _ => None,
    }
}

/// Compute area-weighted centroid of a set of 3D points (simplified version for CityGML)
fn centroid_weighted(points: &[[f64; 3]]) -> [f64; 3] {
    if points.is_empty() {
        return [0.0, 0.0, 0.0];
    }
    let sum_x: f64 = points.iter().map(|p| p[0]).sum();
    let sum_y: f64 = points.iter().map(|p| p[1]).sum();
    let sum_z: f64 = points.iter().map(|p| p[2]).sum();
    let n = points.len() as f64;
    [sum_x / n, sum_y / n, sum_z / n]
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
            buffer_bytes: 0,
            temp_dir: None,
            candidate_chunk_count: 0,
            base_chunk_count: 0,
            executor_id: None,
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
            buffer_bytes: 0,
            temp_dir: None,
            candidate_chunk_count: 0,
            base_chunk_count: 0,
            executor_id: None,
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
            buffer_bytes: 0,
            temp_dir: None,
            candidate_chunk_count: 0,
            base_chunk_count: 0,
            executor_id: None,
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

        let (point, _) = extract_representative_point(&feature).unwrap();
        let [x, y] = point;
        // With true area-weighted centroid, we should get (5, 5) for a square
        assert!((x - 5.0).abs() < 0.01, "x was {}", x);
        assert!((y - 5.0).abs() < 0.01, "y was {}", y);
    }

    #[test]
    fn test_euclidean_distance() {
        let a = [0.0, 0.0];
        let b = [3.0, 4.0];
        assert!((euclidean_distance_2d(&a, &b) - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_haversine_distance() {
        // Distance between two points in London (approximate)
        // Point 1: Buckingham Palace (~-0.14, 51.50)
        // Point 2: Tower of London (~-0.08, 51.51)
        // Expected: roughly 4 km
        let buckingham = [-0.1419, 51.5014];
        let tower = [-0.0761, 51.5081];
        let distance = haversine_distance(&buckingham, &tower);
        // Should be approximately 4000-5000 meters
        assert!(
            distance > 3000.0 && distance < 6000.0,
            "Distance was {}m",
            distance
        );
    }

    // v2 Tests

    #[test]
    fn test_num_closest_3() {
        let mut finder = NeighborFinder {
            params: NeighborFinderParams {
                num_closest: 3,
                merge_strategy: MergeStrategy::Closest,
                ..Default::default()
            },
            candidates: Vec::new(),
            base_features: Vec::new(),
            buffer_bytes: 0,
            temp_dir: None,
            candidate_chunk_count: 0,
            base_chunk_count: 0,
            executor_id: None,
        };

        // Create 5 candidate features at different distances
        let candidates: Vec<_> = (0..5)
            .map(|i| {
                create_point_feature_with_attr(
                    i as f64 * 10.0, // 0, 10, 20, 30, 40
                    0.0,
                    "id",
                    AttributeValue::Number(serde_json::Number::from(i)),
                )
            })
            .collect();

        // Base feature at position 15 - closest are at 10, 20, 0 (distances: 5, 5, 15)
        let base = create_point_feature(15.0, 0.0);

        let noop = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop);

        // Process all candidates
        for candidate in &candidates {
            let mut ctx = create_default_execute_context(candidate);
            ctx.port = CANDIDATE_PORT.clone();
            finder.process(ctx, &fw).unwrap();
        }

        // Process base
        let mut ctx = create_default_execute_context(&base);
        ctx.port = BASE_PORT.clone();
        finder.process(ctx, &fw).unwrap();

        // Verify buffering
        assert_eq!(finder.candidates.len(), 5);
        assert_eq!(finder.base_features.len(), 1);

        // Finish and check results - should emit 1 feature (closest strategy)
        let ctx = NodeContext::default();
        finder.finish(ctx, &fw).unwrap();

        if let ProcessorChannelForwarder::Noop(noop) = fw {
            let ports = noop.send_ports.lock().unwrap();
            // With closest strategy, we get 1 output feature
            assert_eq!(ports.len(), 1);
            assert_eq!(ports[0], *MATCHED_PORT);
        }
    }

    #[test]
    fn test_repeat_base_strategy() {
        let mut finder = NeighborFinder {
            params: NeighborFinderParams {
                num_closest: 3,
                merge_strategy: MergeStrategy::RepeatBase,
                ..Default::default()
            },
            candidates: Vec::new(),
            base_features: Vec::new(),
            buffer_bytes: 0,
            temp_dir: None,
            candidate_chunk_count: 0,
            base_chunk_count: 0,
            executor_id: None,
        };

        // Create 5 candidate features
        let candidates: Vec<_> = (0..5)
            .map(|i| {
                create_point_feature_with_attr(
                    i as f64 * 10.0,
                    0.0,
                    "name",
                    AttributeValue::String(format!("Candidate{}", i)),
                )
            })
            .collect();

        let base = create_point_feature(15.0, 0.0);

        let noop = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop);

        // Process all candidates
        for candidate in &candidates {
            let mut ctx = create_default_execute_context(candidate);
            ctx.port = CANDIDATE_PORT.clone();
            finder.process(ctx, &fw).unwrap();
        }

        // Process base
        let mut ctx = create_default_execute_context(&base);
        ctx.port = BASE_PORT.clone();
        finder.process(ctx, &fw).unwrap();

        // Finish
        let ctx = NodeContext::default();
        finder.finish(ctx, &fw).unwrap();

        if let ProcessorChannelForwarder::Noop(noop) = fw {
            let ports = noop.send_ports.lock().unwrap();
            // With repeat_base strategy and num_closest=3, we should get 3 output features
            assert_eq!(
                ports.len(),
                3,
                "Expected 3 output features with repeat_base strategy"
            );
            for port in ports.iter() {
                assert_eq!(*port, *MATCHED_PORT);
            }
        }
    }

    #[test]
    fn test_array_attributes_strategy() {
        let mut finder = NeighborFinder {
            params: NeighborFinderParams {
                num_closest: 3,
                merge_strategy: MergeStrategy::ArrayAttributes,
                ..Default::default()
            },
            candidates: Vec::new(),
            base_features: Vec::new(),
            buffer_bytes: 0,
            temp_dir: None,
            candidate_chunk_count: 0,
            base_chunk_count: 0,
            executor_id: None,
        };

        // Create 5 candidate features
        let candidates: Vec<_> = (0..5)
            .map(|i| {
                create_point_feature_with_attr(
                    i as f64 * 10.0,
                    0.0,
                    "name",
                    AttributeValue::String(format!("Candidate{}", i)),
                )
            })
            .collect();

        let base = create_point_feature(15.0, 0.0);

        let noop = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop);

        // Process all candidates
        for candidate in &candidates {
            let mut ctx = create_default_execute_context(candidate);
            ctx.port = CANDIDATE_PORT.clone();
            finder.process(ctx, &fw).unwrap();
        }

        // Process base
        let mut ctx = create_default_execute_context(&base);
        ctx.port = BASE_PORT.clone();
        finder.process(ctx, &fw).unwrap();

        // Finish
        let ctx = NodeContext::default();
        finder.finish(ctx, &fw).unwrap();

        if let ProcessorChannelForwarder::Noop(noop) = fw {
            let ports = noop.send_ports.lock().unwrap();
            // With array_attributes strategy, we should get 1 output feature with arrays
            assert_eq!(
                ports.len(),
                1,
                "Expected 1 output feature with array_attributes strategy"
            );
            assert_eq!(ports[0], *MATCHED_PORT);
        }
    }

    #[test]
    fn test_num_closest_less_than_available() {
        let mut finder = NeighborFinder {
            params: NeighborFinderParams {
                num_closest: 5, // Request 5
                merge_strategy: MergeStrategy::RepeatBase,
                ..Default::default()
            },
            candidates: Vec::new(),
            base_features: Vec::new(),
            buffer_bytes: 0,
            temp_dir: None,
            candidate_chunk_count: 0,
            base_chunk_count: 0,
            executor_id: None,
        };

        // Only create 3 candidates
        let candidates: Vec<_> = (0..3)
            .map(|i| {
                create_point_feature_with_attr(
                    i as f64 * 10.0,
                    0.0,
                    "id",
                    AttributeValue::Number(serde_json::Number::from(i)),
                )
            })
            .collect();

        let base = create_point_feature(15.0, 0.0);

        let noop = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop);

        for candidate in &candidates {
            let mut ctx = create_default_execute_context(candidate);
            ctx.port = CANDIDATE_PORT.clone();
            finder.process(ctx, &fw).unwrap();
        }

        let mut ctx = create_default_execute_context(&base);
        ctx.port = BASE_PORT.clone();
        finder.process(ctx, &fw).unwrap();

        let ctx = NodeContext::default();
        finder.finish(ctx, &fw).unwrap();

        if let ProcessorChannelForwarder::Noop(noop) = fw {
            let ports = noop.send_ports.lock().unwrap();
            // Should only get 3 features since that's all we have
            assert_eq!(
                ports.len(),
                3,
                "Should emit only available candidates when num_closest > available"
            );
        }
    }

    #[test]
    fn test_partial_matches_with_max_distance() {
        let mut finder = NeighborFinder {
            params: NeighborFinderParams {
                num_closest: 3,
                max_distance: Some(15.0), // Only candidates within 15 units
                merge_strategy: MergeStrategy::RepeatBase,
                ..Default::default()
            },
            candidates: Vec::new(),
            base_features: Vec::new(),
            buffer_bytes: 0,
            temp_dir: None,
            candidate_chunk_count: 0,
            base_chunk_count: 0,
            executor_id: None,
        };

        // Candidates at 0, 10, 20, 30 from base at 15
        // Distances: 15, 5, 5, 15
        // With max_distance=15, only 3 are within range (0, 10, 20)
        let candidates: Vec<_> = (0..4)
            .map(|i| {
                create_point_feature_with_attr(
                    i as f64 * 10.0,
                    0.0,
                    "id",
                    AttributeValue::Number(serde_json::Number::from(i)),
                )
            })
            .collect();

        let base = create_point_feature(15.0, 0.0);

        let noop = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop);

        for candidate in &candidates {
            let mut ctx = create_default_execute_context(candidate);
            ctx.port = CANDIDATE_PORT.clone();
            finder.process(ctx, &fw).unwrap();
        }

        let mut ctx = create_default_execute_context(&base);
        ctx.port = BASE_PORT.clone();
        finder.process(ctx, &fw).unwrap();

        let ctx = NodeContext::default();
        finder.finish(ctx, &fw).unwrap();

        if let ProcessorChannelForwarder::Noop(noop) = fw {
            let ports = noop.send_ports.lock().unwrap();
            assert_eq!(ports.len(), 3);
        }
    }

    #[test]
    fn test_num_closest_zero_is_error() {
        let factory = NeighborFinderFactory;
        let params = serde_json::json!({
            "numClosest": 0
        });
        let with: HashMap<String, Value> = serde_json::from_value(params).unwrap();

        let result = factory.build(
            NodeContext::default(),
            EventHub::new(1),
            "NeighborFinder".to_string(),
            Some(with),
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_neighbor_index_attribute_suppressed() {
        let mut finder = NeighborFinder {
            params: NeighborFinderParams {
                num_closest: 2,
                merge_strategy: MergeStrategy::RepeatBase,
                neighbor_index_attribute: Attribute::new(""), // Empty string suppresses index
                ..Default::default()
            },
            candidates: Vec::new(),
            base_features: Vec::new(),
            buffer_bytes: 0,
            temp_dir: None,
            candidate_chunk_count: 0,
            base_chunk_count: 0,
            executor_id: None,
        };

        let candidates: Vec<_> = (0..3)
            .map(|i| {
                create_point_feature_with_attr(
                    i as f64 * 10.0,
                    0.0,
                    "id",
                    AttributeValue::Number(serde_json::Number::from(i)),
                )
            })
            .collect();

        let base = create_point_feature(15.0, 0.0);

        let noop = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop);

        for candidate in &candidates {
            let mut ctx = create_default_execute_context(candidate);
            ctx.port = CANDIDATE_PORT.clone();
            finder.process(ctx, &fw).unwrap();
        }

        let mut ctx = create_default_execute_context(&base);
        ctx.port = BASE_PORT.clone();
        finder.process(ctx, &fw).unwrap();

        let ctx = NodeContext::default();
        finder.finish(ctx, &fw).unwrap();

        // Should complete without error even with empty index attribute
        if let ProcessorChannelForwarder::Noop(noop) = fw {
            let ports = noop.send_ports.lock().unwrap();
            assert_eq!(ports.len(), 2);
        }
    }

    #[test]
    fn test_euclidean_3d_distance() {
        let a = [0.0, 0.0];
        let b = [3.0, 4.0];
        let dist_2d = euclidean_distance_2d(&a, &b);
        let dist_3d = euclidean_distance_3d(&a, 0.0, &b, 0.0);
        // With z=0 for both, 2D and 3D should be equal
        assert!((dist_2d - dist_3d).abs() < 0.001);

        // Test with different z values
        let dist_3d_with_z = euclidean_distance_3d(&a, 0.0, &b, 12.0);
        // sqrt(3^2 + 4^2 + 12^2) = sqrt(9 + 16 + 144) = sqrt(169) = 13
        assert!((dist_3d_with_z - 13.0).abs() < 0.001);
    }
}
