use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Read, Seek, Write};
use std::path::PathBuf;

use once_cell::sync::Lazy;
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

/// Minimum number of base features to trigger parallel processing
const PARALLEL_THRESHOLD: usize = 100;

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
            candidate_index: Vec::new(),
            in_memory_candidates: Vec::new(),
            base_features: Vec::new(),
            base_buffer_bytes: 0,
            candidate_buffer_bytes: 0,
            temp_dir: None,
            candidates_file_path: None,
            candidates_file_offset: 0,
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
    ///
    /// Note: Distance is computed between representative points (centroids). For accurate
    /// results, input features should be relatively small (e.g., buildings, local roads).
    /// Large geometries (e.g., countries, large water bodies) may produce inaccurate
    /// distances because their centroids may not represent their spatial extent well.
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
    #[schemars(range(min = 1))]
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

/// A lightweight spatial index entry for candidates.
/// Contains only point location and byte offset - the feature data stays in a single JSONL file.
/// Uses byte offsets for O(1) random access into the consolidated file.
#[derive(Debug, Clone)]
struct CandidateIndex {
    point: [f64; 2],
    point_z: Option<f64>,
    /// Byte offset in the JSONL file where this candidate's JSON starts.
    /// For in-memory candidates, this is `u64::MAX` and `in_memory_index` is used instead.
    offset: u64,
    /// Length of the JSON line in bytes (0 for in-memory candidates).
    length: u32,
    /// Index into `in_memory_candidates` when `offset == u64::MAX`.
    /// For disk-based candidates, this is `u32::MAX`.
    in_memory_index: u32,
    /// 3D Cartesian coordinates for spatial indexing.
    ///
    /// For Euclidean2D: [x, y, 0] - 2D point embedded in 3D space
    /// For Euclidean3D: [x, y, z] - actual 3D coordinates
    /// For Haversine: ECEF (Earth-Centered Earth-Fixed) coordinates where Euclidean distance
    ///   correlates monotonically with geodesic distance, ensuring correct nearest-neighbor
    ///   ordering everywhere on Earth (including near poles where lat/lon breaks down).
    projected_3d: [f64; 3],
}

impl RTreeObject for CandidateIndex {
    type Envelope = AABB<[f64; 3]>;

    fn envelope(&self) -> Self::Envelope {
        AABB::from_point(self.projected_3d)
    }
}

impl PointDistance for CandidateIndex {
    fn distance_2(&self, point: &[f64; 3]) -> f64 {
        let dx = self.projected_3d[0] - point[0];
        let dy = self.projected_3d[1] - point[1];
        let dz = self.projected_3d[2] - point[2];
        dx * dx + dy * dy + dz * dz
    }
}

/// A neighbor match result containing the candidate index and computed distance.
#[derive(Debug, Clone)]
struct NeighborMatch {
    candidate_index: CandidateIndex,
    distance: f64,
    index: usize, // 0-based rank
}

/// Result of processing a single base feature for parallel execution.
/// Contains the output feature(s) and the destination port.
#[derive(Debug)]
enum BaseProcessResult {
    Rejected(Feature),
    Unmatched(Feature),
    Matched(Vec<Feature>), // One or more enriched features
}

/// Full candidate entry stored on disk and in memory.
/// Includes pre-serialized bytes to avoid double serialization.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CandidateEntry {
    point: [f64; 2],
    point_z: Option<f64>, // Z coordinate for 3D distance calculations
    feature: Feature,
    #[serde(skip)]
    serialized: Vec<u8>, // Pre-serialized bytes to avoid double serialization
}

/// Base feature buffer entry with pre-serialized bytes to avoid double serialization.
#[derive(Debug, Clone)]
struct BaseFeatureEntry {
    feature: Feature,
    serialized: Vec<u8>, // Pre-serialized bytes
}

#[derive(Debug, Clone)]
struct NeighborFinder {
    params: NeighborFinderParams,
    /// Buffer for candidates before spilling.
    candidates: Vec<CandidateEntry>,
    /// Spatial index built from candidates (point + disk location only).
    /// Populated during finish() after all candidates are flushed to disk.
    candidate_index: Vec<CandidateIndex>,
    /// In-memory storage for candidates when disk spilling is not available (e.g., tests).
    /// Indexed by the position in candidate_index when file_index is usize::MAX.
    in_memory_candidates: Vec<CandidateEntry>,
    base_features: Vec<BaseFeatureEntry>,
    // Disk spilling fields - track bytes separately for each buffer
    base_buffer_bytes: usize,
    candidate_buffer_bytes: usize,
    temp_dir: Option<PathBuf>,
    /// Path to the consolidated candidates JSONL file
    candidates_file_path: Option<PathBuf>,
    /// Current byte offset in the candidates JSONL file for next write
    candidates_file_offset: u64,
    base_chunk_count: usize,
    executor_id: Option<uuid::Uuid>,
}

impl Drop for NeighborFinder {
    fn drop(&mut self) {
        // Ensure temporary directory is cleaned up even if finish() panics
        // or isn't called (e.g., due to an error during processing).
        // This prevents disk space leaks in production.
        self.cleanup_temp_dir();
    }
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
            let executor_id = fw.executor_id();
            self.executor_id = Some(executor_id);
            // Only use disk spilling for non-nil executor IDs (actual execution, not tests)
            // Tests use NoopChannelForwarder which returns Uuid::nil(), causing shared temp dirs
            if executor_id != uuid::Uuid::nil() {
                let temp_dir = executor_cache_subdir(executor_id, "neighbor_finder");
                std::fs::create_dir_all(&temp_dir)?;
                self.temp_dir = Some(temp_dir);
            }
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
                // Serialize once and reuse to avoid double serialization
                let serialized = serde_json::to_vec(feature)?;
                let feature_bytes = serialized.len();
                if self.base_buffer_bytes + feature_bytes >= ACCUMULATOR_BUFFER_BYTE_THRESHOLD {
                    self.flush_base_features()?;
                }
                self.base_features.push(BaseFeatureEntry {
                    feature: feature.clone(),
                    serialized,
                });
                self.base_buffer_bytes += feature_bytes;
            }
            port if port == &*CANDIDATE_PORT => {
                // Extract representative point and store candidate
                // Note: Invalid candidates (unsupported or empty geometry) are silently
                // skipped rather than sent to rejected port. Only base features with
                // invalid geometry are routed to rejected. This is intentional because
                // candidates are consumed but never emitted - sending them to rejected
                // would violate this design principle.
                if let Some((point, point_z)) = extract_representative_point(feature) {
                    // Serialize once and reuse to avoid double serialization
                    let serialized = serde_json::to_vec(&CandidateEntry {
                        point,
                        point_z,
                        feature: feature.clone(),
                        serialized: Vec::new(),
                    })?;
                    let entry_bytes = serialized.len();
                    if self.candidate_buffer_bytes + entry_bytes
                        >= ACCUMULATOR_BUFFER_BYTE_THRESHOLD
                    {
                        self.flush_candidates()?;
                    }
                    self.candidates.push(CandidateEntry {
                        point,
                        point_z,
                        feature: feature.clone(),
                        serialized,
                    });
                    self.candidate_buffer_bytes += entry_bytes;
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
        // Load any spilled data and build candidate index
        // If temp_dir is None (e.g., in tests), this will convert in-memory candidates to index
        self.load_from_disk()?;

        // If no base features were processed, nothing to do
        if self.base_features.is_empty() && self.base_chunk_count == 0 {
            return Ok(());
        }

        // If no candidates, all base features go to unmatched
        if self.candidate_index.is_empty() {
            // Process in-memory base features
            for entry in &self.base_features {
                fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                    &ctx,
                    entry.feature.clone(),
                    UNMATCHED_PORT.clone(),
                ));
            }
            // Process disk-based base features sequentially without loading all into memory
            if self.base_chunk_count > 0 {
                self.send_disk_bases_to_unmatched(&ctx, fw)?;
            }
            self.cleanup_temp_dir();
            return Ok(());
        }

        // Build R-tree index from lightweight candidate index (point + disk location only)
        let rtree: RTree<CandidateIndex> = RTree::bulk_load(self.candidate_index.clone());

        // Process in-memory base features (parallelize if above threshold)
        if self.base_features.len() >= PARALLEL_THRESHOLD {
            self.process_base_features_parallel(&ctx, fw, &rtree)?;
        } else {
            // Sequential processing for small batches
            for entry in &self.base_features {
                self.process_single_base(&ctx, fw, &rtree, &entry.feature)?;
            }
        }

        // Process disk-based base features sequentially without loading all into memory
        // This maintains memory efficiency - only one base feature is in memory at a time
        if self.base_chunk_count > 0 {
            self.process_base_features_from_disk(&ctx, fw, &rtree)?;
        }

        // Clean up temporary directory and candidate chunk files after processing
        self.cleanup_temp_dir();

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

        let temp_dir = self.temp_dir.as_ref().unwrap();
        std::fs::create_dir_all(temp_dir)?;

        let chunk_path = temp_dir.join(format!("base_chunk_{}.jsonl.zst", self.base_chunk_count));
        let file = File::create(&chunk_path)?;
        let mut writer = BufWriter::new(zstd::Encoder::new(file, 3)?);

        for entry in &self.base_features {
            writer.write_all(&entry.serialized)?;
            writer.write_all(b"\n")?;
        }
        // Finalize the zstd encoder to ensure the compressed frame is complete.
        let encoder = writer
            .into_inner()
            .map_err(|e| std::io::Error::other(e.to_string()))?;
        encoder.finish()?;

        self.base_chunk_count += 1;
        self.base_features.clear();
        self.base_buffer_bytes = 0;
        Ok(())
    }

    /// Flush candidates to disk as a single consolidated JSONL file.
    /// Uses byte offsets for O(1) random access. Each line is one candidate JSON.
    fn flush_candidates(&mut self) -> Result<(), BoxedError> {
        if self.candidates.is_empty() {
            return Ok(());
        }

        let temp_dir = self.temp_dir.as_ref().unwrap();
        std::fs::create_dir_all(temp_dir)?;

        // Initialize the candidates file path if not set
        if self.candidates_file_path.is_none() {
            self.candidates_file_path = Some(temp_dir.join("candidates.jsonl"));
        }

        // Open file for appending (create if doesn't exist)
        let file_path = self.candidates_file_path.as_ref().unwrap();
        let file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(file_path)?;
        let mut writer = BufWriter::new(file);

        for entry in &self.candidates {
            let offset = self.candidates_file_offset;
            let serialized = &entry.serialized;
            let length = serialized.len() as u32;

            // Write candidate JSON line
            writer.write_all(serialized)?;
            writer.write_all(b"\n")?;

            // Update file offset for next entry
            self.candidates_file_offset += length as u64 + 1; // +1 for newline

            // Create index entry with byte offset
            // Compute 3D projection based on distance metric for spatial indexing
            let projected_3d = self.compute_projected_3d(&entry.point, entry.point_z);
            self.candidate_index.push(CandidateIndex {
                point: entry.point,
                point_z: entry.point_z,
                offset,
                length,
                in_memory_index: u32::MAX, // Not in memory
                projected_3d,
            });
        }

        writer.flush()?;
        self.candidates.clear();
        self.candidate_buffer_bytes = 0;
        Ok(())
    }

    /// Load spilled data from disk back into memory.
    /// For base features: processed sequentially from disk, not loaded into memory.
    /// For candidates: index is already built during flush; just ensure any remaining
    /// in-memory candidates are converted to index entries.
    fn load_from_disk(&mut self) -> Result<(), BoxedError> {
        // Handle in-memory candidates that were never flushed (no temp_dir or empty buffer)
        // Convert them to index entries pointing to in-memory storage
        if !self.candidates.is_empty() {
            if self.temp_dir.is_some() {
                // Flush remaining candidates to individual JSON files
                self.flush_candidates()?;
            } else {
                // No temp_dir (e.g., in tests) - keep candidates in memory
                // Store the full entries and create index entries pointing to them
                for (idx, entry) in self.candidates.iter().enumerate() {
                    self.in_memory_candidates.push(entry.clone());
                    // Compute 3D projection based on distance metric for spatial indexing
                    let projected_3d = self.compute_projected_3d(&entry.point, entry.point_z);
                    self.candidate_index.push(CandidateIndex {
                        point: entry.point,
                        point_z: entry.point_z,
                        offset: u64::MAX, // Marks in-memory
                        length: 0,
                        in_memory_index: idx as u32,
                        projected_3d,
                    });
                }
                self.candidates.clear();
                self.candidate_buffer_bytes = 0;
            }
        }

        // Note: Base features are NOT loaded into memory here.
        // They are processed sequentially from disk in finish() to maintain memory efficiency.

        // Candidates are NOT loaded into memory - their index is already built.
        // Individual candidate JSON files remain on disk for random access during matching.

        Ok(())
    }

    /// Read candidates from disk or memory using their indices.
    /// Disk-based candidates are stored in a single JSONL file with byte offsets for O(1) random access.
    /// For in-memory candidates (offset == u64::MAX), retrieves from in_memory_candidates.
    fn read_candidates_from_disk(
        &self,
        indices: &[&CandidateIndex],
    ) -> Result<Vec<CandidateEntry>, BoxedError> {
        if indices.is_empty() {
            return Ok(Vec::new());
        }

        let mut result = Vec::with_capacity(indices.len());
        const IN_MEMORY_MARKER: u64 = u64::MAX;

        // Check if we need to read from disk
        let needs_disk_read = indices.iter().any(|idx| idx.offset != IN_MEMORY_MARKER);

        // Open file once if needed
        let mut file = if needs_disk_read {
            let file_path = self.candidates_file_path.as_ref().ok_or_else(|| {
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Candidates file path not set",
                )) as BoxedError
            })?;
            Some(File::open(file_path)?)
        } else {
            None
        };

        for idx in indices {
            if idx.offset == IN_MEMORY_MARKER {
                // In-memory candidate - use stored index for correct lookup
                let in_memory_idx = idx.in_memory_index as usize;
                let candidate = self
                    .in_memory_candidates
                    .get(in_memory_idx)
                    .cloned()
                    .ok_or_else(|| {
                        Box::new(std::io::Error::new(
                            std::io::ErrorKind::NotFound,
                            format!("In-memory candidate not found at index {}", in_memory_idx),
                        )) as BoxedError
                    })?;
                result.push(candidate);
            } else {
                // Read from disk using byte offset
                let file = file.as_mut().unwrap();
                file.seek(std::io::SeekFrom::Start(idx.offset))?;

                let mut buffer = vec![0u8; idx.length as usize];
                file.read_exact(&mut buffer)?;

                let candidate: CandidateEntry = serde_json::from_slice(&buffer)?;
                result.push(candidate);
            }
        }

        Ok(result)
    }

    /// Clean up the temporary directory and candidate file after processing.
    fn cleanup_temp_dir(&mut self) {
        if let Some(ref temp_dir) = self.temp_dir {
            // Remove the consolidated candidates JSONL file
            if let Some(ref file_path) = self.candidates_file_path {
                let _ = std::fs::remove_file(file_path);
            }
            // Then remove the directory itself (recursively, in case other files remain)
            let _ = std::fs::remove_dir_all(temp_dir);
        }
    }

    /// Process base features sequentially from disk without loading all into memory.
    /// This maintains memory efficiency - only one base feature is in memory at a time.
    /// Chunk files are deleted immediately after processing to free disk space.
    fn process_base_features_from_disk(
        &self,
        ctx: &NodeContext,
        fw: &ProcessorChannelForwarder,
        rtree: &RTree<CandidateIndex>,
    ) -> Result<(), BoxedError> {
        let Some(ref temp_dir) = self.temp_dir else {
            return Ok(());
        };

        for i in 0..self.base_chunk_count {
            let chunk_path = temp_dir.join(format!("base_chunk_{}.jsonl.zst", i));
            let file = File::open(&chunk_path)?;
            let reader = BufReader::new(zstd::Decoder::new(file)?);

            for line in reader.lines() {
                let line = line?;
                let base: Feature = serde_json::from_str(&line)?;
                self.process_single_base(ctx, fw, rtree, &base)?;
            }

            // Delete the chunk file immediately after processing to free disk space
            std::fs::remove_file(&chunk_path)?;
        }

        Ok(())
    }

    /// Send disk-based base features to unmatched port without loading all into memory.
    /// Used when there are no candidates to match against.
    fn send_disk_bases_to_unmatched(
        &self,
        ctx: &NodeContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let Some(ref temp_dir) = self.temp_dir else {
            return Ok(());
        };

        for i in 0..self.base_chunk_count {
            let chunk_path = temp_dir.join(format!("base_chunk_{}.jsonl.zst", i));
            let file = File::open(&chunk_path)?;
            let reader = BufReader::new(zstd::Decoder::new(file)?);

            for line in reader.lines() {
                let line = line?;
                let base: Feature = serde_json::from_str(&line)?;
                fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                    ctx,
                    base,
                    UNMATCHED_PORT.clone(),
                ));
            }

            // Delete the chunk file immediately after processing
            std::fs::remove_file(&chunk_path)?;
        }

        Ok(())
    }

    /// Process a single base feature
    fn process_single_base(
        &self,
        ctx: &NodeContext,
        fw: &ProcessorChannelForwarder,
        rtree: &RTree<CandidateIndex>,
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
                let enriched = self.create_enriched_feature(base, &filtered_matches[0], false)?;
                fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                    ctx,
                    enriched,
                    MATCHED_PORT.clone(),
                ));
            }
            MergeStrategy::RepeatBase => {
                for neighbor_match in filtered_matches {
                    let enriched = self.create_enriched_feature(base, neighbor_match, true)?;
                    fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                        ctx,
                        enriched,
                        MATCHED_PORT.clone(),
                    ));
                }
            }
            MergeStrategy::ArrayAttributes => {
                let enriched = self.create_array_enriched_feature(base, filtered_matches)?;
                fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                    ctx,
                    enriched,
                    MATCHED_PORT.clone(),
                ));
            }
        }
        Ok(())
    }

    /// Process base features in parallel using rayon.
    /// Performs neighbor matching in parallel, then sends results sequentially.
    fn process_base_features_parallel(
        &self,
        ctx: &NodeContext,
        fw: &ProcessorChannelForwarder,
        rtree: &RTree<CandidateIndex>,
    ) -> Result<(), BoxedError> {
        use rayon::prelude::*;

        // Process all base features in parallel to find matches
        let results: Vec<BaseProcessResult> = self
            .base_features
            .par_iter()
            .map(|entry| self.process_base_feature_to_result(rtree, &entry.feature))
            .collect::<Result<Vec<_>, _>>()?;

        // Send results sequentially to maintain order
        for result in results {
            match result {
                BaseProcessResult::Rejected(feature) => {
                    fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                        ctx,
                        feature,
                        REJECTED_PORT.clone(),
                    ));
                }
                BaseProcessResult::Unmatched(feature) => {
                    fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                        ctx,
                        feature,
                        UNMATCHED_PORT.clone(),
                    ));
                }
                BaseProcessResult::Matched(features) => {
                    for feature in features {
                        fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                            ctx,
                            feature,
                            MATCHED_PORT.clone(),
                        ));
                    }
                }
            }
        }

        Ok(())
    }

    /// Process a single base feature and return the result (for parallel execution).
    /// This is a pure computation function that doesn't interact with the forwarder.
    fn process_base_feature_to_result(
        &self,
        rtree: &RTree<CandidateIndex>,
        base: &Feature,
    ) -> Result<BaseProcessResult, BoxedError> {
        let Some((base_point, base_z)) = extract_representative_point(base) else {
            return Ok(BaseProcessResult::Rejected(base.clone()));
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
            return Ok(BaseProcessResult::Unmatched(base.clone()));
        }

        // Generate enriched feature(s) based on merge strategy
        let enriched_features = match self.params.merge_strategy {
            MergeStrategy::Closest => {
                vec![self.create_enriched_feature(base, &filtered_matches[0], false)?]
            }
            MergeStrategy::RepeatBase => {
                // Batch read all candidates at once for efficiency
                self.create_repeat_base_features(base, &filtered_matches)?
            }
            MergeStrategy::ArrayAttributes => {
                vec![self.create_array_enriched_feature(base, &filtered_matches)?]
            }
        };

        Ok(BaseProcessResult::Matched(enriched_features))
    }

    /// Create an enriched feature with distance and transferred attributes.
    /// Reads the candidate data from disk using the index entry.
    fn create_enriched_feature(
        &self,
        base: &Feature,
        neighbor_match: &NeighborMatch,
        include_index: bool,
    ) -> Result<Feature, BoxedError> {
        let mut enriched = base.clone();

        // Add distance attribute (use Null for non-finite values like NaN/Infinity)
        enriched.attributes_mut().insert(
            self.params.distance_attribute.clone(),
            f64_to_attribute_value(neighbor_match.distance),
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

        // Read candidate from disk and transfer attributes
        let candidates = self.read_candidates_from_disk(&[&neighbor_match.candidate_index])?;
        self.transfer_attributes(&mut enriched, &candidates[0]);

        Ok(enriched)
    }

    /// Create multiple enriched features for RepeatBase strategy with batched candidate reads.
    /// This is more efficient than calling create_enriched_feature for each neighbor separately
    /// because it opens the candidate file only once.
    fn create_repeat_base_features(
        &self,
        base: &Feature,
        neighbor_matches: &[NeighborMatch],
    ) -> Result<Vec<Feature>, BoxedError> {
        if neighbor_matches.is_empty() {
            return Ok(Vec::new());
        }

        // Batch read all candidates at once
        let indices: Vec<_> = neighbor_matches
            .iter()
            .map(|m| &m.candidate_index)
            .collect();
        let candidates = self.read_candidates_from_disk(&indices)?;

        // Create enriched feature for each neighbor
        let mut enriched_features = Vec::with_capacity(neighbor_matches.len());
        for (i, neighbor_match) in neighbor_matches.iter().enumerate() {
            let mut enriched = base.clone();

            // Add distance attribute
            enriched.attributes_mut().insert(
                self.params.distance_attribute.clone(),
                f64_to_attribute_value(neighbor_match.distance),
            );

            // Add neighbor index attribute (if not suppressed)
            let index_attr = &self.params.neighbor_index_attribute;
            if !index_attr.as_ref().is_empty() {
                enriched.attributes_mut().insert(
                    index_attr.clone(),
                    AttributeValue::Number(serde_json::Number::from(neighbor_match.index as u64)),
                );
            }

            // Transfer attributes from the corresponding candidate
            self.transfer_attributes(&mut enriched, &candidates[i]);

            enriched_features.push(enriched);
        }

        Ok(enriched_features)
    }

    /// Create an enriched feature with array-valued attributes for all neighbors.
    /// Reads candidate data from disk for each neighbor using index entries.
    fn create_array_enriched_feature(
        &self,
        base: &Feature,
        neighbor_matches: &[NeighborMatch],
    ) -> Result<Feature, BoxedError> {
        let mut enriched = base.clone();
        let prefix = &self.params.attribute_prefix;

        // Collect distances into an array (use Null for non-finite values)
        let distances: Vec<AttributeValue> = neighbor_matches
            .iter()
            .map(|m| f64_to_attribute_value(m.distance))
            .collect();
        enriched.attributes_mut().insert(
            self.params.distance_attribute.clone(),
            AttributeValue::Array(distances),
        );

        // Read all candidates from disk first (batched for efficiency)
        let indices: Vec<_> = neighbor_matches
            .iter()
            .map(|m| &m.candidate_index)
            .collect();
        let candidates = self.read_candidates_from_disk(&indices)?;

        // Collect all unique attribute names from all candidates
        let mut all_attrs: Vec<Attribute> = Vec::new();
        for candidate in &candidates {
            let candidate_attrs: Vec<_> = if self.params.attributes_to_transfer.is_empty() {
                candidate.feature.attributes.keys().cloned().collect()
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
            let values: Vec<AttributeValue> = candidates
                .iter()
                .map(|c| {
                    c.feature
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

        Ok(enriched)
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

    /// Compute 3D projected coordinates for spatial indexing based on the distance metric.
    ///
    /// - Euclidean2D: [x, y, 0] - embed 2D point in 3D space
    /// - Euclidean3D: [x, y, z] - use actual 3D coordinates
    /// - Haversine: ECEF coordinates - project lat/lon to Cartesian for correct spatial ordering
    fn compute_projected_3d(&self, point: &[f64; 2], point_z: Option<f64>) -> [f64; 3] {
        match self.params.distance_metric {
            DistanceMetric::Euclidean2d => [point[0], point[1], 0.0],
            DistanceMetric::Euclidean3d => [point[0], point[1], point_z.unwrap_or(0.0)],
            DistanceMetric::Haversine => lat_lon_to_ecef(point[1], point[0]),
        }
    }
}

/// Find the k nearest neighbors for a given base point using the R-tree.
///
/// The R-tree uses 3D Cartesian coordinates for spatial indexing:
/// - Euclidean2D: points are embedded in 3D as [x, y, 0]
/// - Euclidean3D: points use actual [x, y, z]
/// - Haversine: points use ECEF (Earth-Centered Earth-Fixed) coordinates
///
/// For Haversine, the ECEF projection ensures Euclidean distance in 3D space
/// correlates monotonically with geodesic distance on the sphere. This allows
/// correct nearest-neighbor ordering everywhere, including near the poles where
/// lat/lon Euclidean distance breaks down.
fn find_k_nearest_neighbors(
    rtree: &RTree<CandidateIndex>,
    base_point: &[f64; 2],
    base_z: Option<f64>,
    k: usize,
    metric: &DistanceMetric,
) -> Vec<NeighborMatch> {
    // Compute the query point in 3D space based on the metric
    let query_3d: [f64; 3] = match metric {
        DistanceMetric::Euclidean2d => [base_point[0], base_point[1], 0.0],
        DistanceMetric::Euclidean3d => [base_point[0], base_point[1], base_z.unwrap_or(0.0)],
        DistanceMetric::Haversine => lat_lon_to_ecef(base_point[1], base_point[0]),
    };

    match metric {
        DistanceMetric::Euclidean2d => {
            // For 2D, R-tree ordering using [x, y, 0] is exact
            rtree
                .nearest_neighbor_iter(&query_3d)
                .take(k)
                .cloned()
                .enumerate()
                .map(|(index, candidate_index)| {
                    let distance = euclidean_distance_2d(base_point, &candidate_index.point);
                    NeighborMatch {
                        candidate_index,
                        distance,
                        index,
                    }
                })
                .collect()
        }
        DistanceMetric::Euclidean3d => {
            // For 3D, we over-fetch and re-sort by actual 3D distance to handle
            // cases where the R-tree's approximate ordering isn't perfect
            let fetch_count = k.saturating_mul(4).min(rtree.size());
            let mut matches: Vec<NeighborMatch> = rtree
                .nearest_neighbor_iter(&query_3d)
                .take(fetch_count)
                .cloned()
                .enumerate()
                .map(|(index, candidate_index)| {
                    let distance = euclidean_distance_3d(
                        base_point,
                        base_z.unwrap_or(0.0),
                        &candidate_index.point,
                        candidate_index.point_z.unwrap_or(0.0),
                    );
                    NeighborMatch {
                        candidate_index,
                        distance,
                        index,
                    }
                })
                .collect();

            matches.sort_by(|a, b| {
                a.distance
                    .partial_cmp(&b.distance)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            matches.truncate(k);

            for (i, m) in matches.iter_mut().enumerate() {
                m.index = i;
            }
            matches
        }
        DistanceMetric::Haversine => {
            // For Haversine, ECEF coordinates ensure correct ordering.
            // The R-tree uses ECEF for indexing, then we compute actual
            // Haversine distance for the output value.
            rtree
                .nearest_neighbor_iter(&query_3d)
                .take(k)
                .cloned()
                .enumerate()
                .map(|(index, candidate_index)| {
                    // Use accurate Haversine distance for the output value
                    let distance = haversine_distance(base_point, &candidate_index.point);
                    NeighborMatch {
                        candidate_index,
                        distance,
                        index,
                    }
                })
                .collect()
        }
    }
}

/// Compute 2D Euclidean distance between two points.
/// Uses reearth-flow-geometry's EuclideanDistance trait for consistency.
fn euclidean_distance_2d(a: &[f64; 2], b: &[f64; 2]) -> f64 {
    use reearth_flow_geometry::algorithm::euclidean_distance::EuclideanDistance;
    use reearth_flow_geometry::types::coordinate::Coordinate2D;

    let coord_a: Coordinate2D<f64> = (*a).into();
    let coord_b: Coordinate2D<f64> = (*b).into();
    coord_a.euclidean_distance(&coord_b)
}

/// Compute 3D Euclidean distance between two points.
/// Note: The geometry crate's EuclideanDistance for Coordinate3D uses Line3D which
/// only computes 2D distance (dx.hypot(dy)), so we manually compute 3D distance here.
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

/// Convert latitude/longitude to ECEF (Earth-Centered Earth-Fixed) Cartesian coordinates.
///
/// This transforms geographic coordinates to a 3D Cartesian coordinate system where
/// Euclidean distance correlates monotonically with geodesic (great-circle) distance.
/// This allows using standard R-tree spatial indexing for Haversine distance queries.
///
/// Input: latitude and longitude in degrees.
/// Output: [x, y, z] coordinates in meters (approximately, scaled by Earth's radius).
fn lat_lon_to_ecef(lat_deg: f64, lon_deg: f64) -> [f64; 3] {
    let lat = lat_deg.to_radians();
    let lon = lon_deg.to_radians();

    // Using unit sphere (radius = 1) is sufficient for ordering purposes.
    // For true ECEF, multiply by Earth's radius, but relative distances are preserved.
    let x = lat.cos() * lon.cos();
    let y = lat.cos() * lon.sin();
    let z = lat.sin();

    // Scale by Earth's radius to get approximate meter-scale coordinates
    // This ensures distances are in a reasonable range for the R-tree
    [
        x * EARTH_RADIUS_METERS,
        y * EARTH_RADIUS_METERS,
        z * EARTH_RADIUS_METERS,
    ]
}

/// Convert an f64 value to AttributeValue.
/// Returns Null for non-finite values (NaN, Infinity) since they cannot be
/// represented as JSON numbers.
fn f64_to_attribute_value(value: f64) -> AttributeValue {
    if let Some(num) = serde_json::Number::from_f64(value) {
        AttributeValue::Number(num)
    } else {
        AttributeValue::Null
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
                let centroid = centroid_simple_mean(&all_points);
                Some(([centroid[0], centroid[1]], Some(centroid[2])))
            }
        }
    }
}

fn extract_representative_point_2d(
    geo: &reearth_flow_geometry::types::geometry::Geometry2D<f64>,
) -> Option<[f64; 2]> {
    use reearth_flow_geometry::algorithm::centroid::Centroid;

    geo.centroid().map(|c| [c.x(), c.y()])
}

fn extract_representative_point_3d(
    geo: &reearth_flow_geometry::types::geometry::Geometry3D<f64>,
) -> Option<([f64; 2], Option<f64>)> {
    use reearth_flow_geometry::algorithm::centroid::Centroid;

    geo.centroid().map(|c| ([c.x(), c.y()], Some(c.z())))
}

/// Compute arithmetic mean of a set of 3D points.
///
/// Note: This is a simple average of vertex coordinates, NOT an area-weighted centroid.
/// For CityGML geometries, we use this simplification as a representative point.
/// For true area-weighted centroids of polygons, use the `Centroid` trait from
/// `reearth_flow_geometry::algorithm::centroid`.
fn centroid_simple_mean(points: &[[f64; 3]]) -> [f64; 3] {
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
        )
    }

    #[test]
    fn test_single_closest_neighbor() {
        let mut finder = NeighborFinder {
            params: NeighborFinderParams::default(),
            candidates: Vec::new(),
            candidate_index: Vec::new(),
            in_memory_candidates: Vec::new(),
            base_features: Vec::new(),
            base_buffer_bytes: 0,
            candidate_buffer_bytes: 0,
            temp_dir: None,
            candidates_file_path: None,
            candidates_file_offset: 0,
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

        // Finish processing to trigger matching
        let ctx = NodeContext::default();
        finder.finish(ctx, &fw).unwrap();

        // Verify the output was sent to matched port
        if let ProcessorChannelForwarder::Noop(noop) = fw {
            let ports = noop.send_ports.lock().unwrap();
            // Should have 1 matched feature
            assert_eq!(ports.len(), 1, "Should have one matched output");
            assert_eq!(ports[0], *MATCHED_PORT, "Should be sent to matched port");

            // Verify the sent features have expected attributes
            let features = noop.send_features.lock().unwrap();
            assert_eq!(features.len(), 1, "Should have one feature");
            let feature = &features[0];

            // Verify distance is correct (distance from (1,0) to (0,0) is 1.0)
            let distance = feature
                .attributes
                .get(&Attribute::new("_neighbor_distance"));
            assert!(distance.is_some(), "Should have distance attribute");
            if let Some(AttributeValue::Number(d)) = distance {
                assert!(
                    (d.as_f64().unwrap() - 1.0).abs() < 0.001,
                    "Distance should be 1.0"
                );
            }

            // Verify transferred attribute from closest candidate (A)
            let neighbor_name = feature.attributes.get(&Attribute::new("_neighbor_name"));
            assert!(neighbor_name.is_some(), "Should have transferred attribute");
            assert_eq!(
                neighbor_name,
                Some(&AttributeValue::String("A".to_string())),
                "Should match closest candidate (A at distance 1, not B at distance 9)"
            );
        }
    }

    #[test]
    fn test_max_distance_filtering() {
        let mut finder = NeighborFinder {
            params: NeighborFinderParams {
                max_distance: Some(5.0),
                ..Default::default()
            },
            candidates: Vec::new(),
            candidate_index: Vec::new(),
            in_memory_candidates: Vec::new(),
            base_features: Vec::new(),
            base_buffer_bytes: 0,
            candidate_buffer_bytes: 0,
            temp_dir: None,
            candidates_file_path: None,
            candidates_file_offset: 0,
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

            // Verify base feature is preserved (emitted to unmatched without distance attribute)
            let features = noop.send_features.lock().unwrap();
            assert_eq!(features.len(), 1, "Should have one unmatched feature");
            let feature = &features[0];
            // No distance attribute since no match within max_distance
            assert!(
                feature
                    .attributes
                    .get(&Attribute::new("_neighbor_distance"))
                    .is_none(),
                "Unmatched feature should not have distance attribute"
            );
        }
    }

    #[test]
    fn test_no_candidates_all_unmatched() {
        let mut finder = NeighborFinder {
            params: NeighborFinderParams::default(),
            candidates: Vec::new(),
            candidate_index: Vec::new(),
            in_memory_candidates: Vec::new(),
            base_features: Vec::new(),
            base_buffer_bytes: 0,
            candidate_buffer_bytes: 0,
            temp_dir: None,
            candidates_file_path: None,
            candidates_file_offset: 0,
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

            // Verify base feature is preserved when no candidates
            let features = noop.send_features.lock().unwrap();
            assert_eq!(features.len(), 1, "Should have one unmatched feature");
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
        // Test that num_closest actually limits the number of neighbors returned.
        // Use RepeatBase strategy so we can observe the effect of num_closest
        // (Closest strategy would always emit only 1 regardless of num_closest).
        let mut finder = NeighborFinder {
            params: NeighborFinderParams {
                num_closest: 3,
                merge_strategy: MergeStrategy::RepeatBase,
                ..Default::default()
            },
            candidates: Vec::new(),
            candidate_index: Vec::new(),
            in_memory_candidates: Vec::new(),
            base_features: Vec::new(),
            base_buffer_bytes: 0,
            candidate_buffer_bytes: 0,
            temp_dir: None,
            candidates_file_path: None,
            candidates_file_offset: 0,
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
            // With RepeatBase strategy and num_closest=3, we get 3 output features
            // This proves num_closest is actually working (not just defaulting to 1)
            assert_eq!(
                ports.len(),
                3,
                "Should have 3 output features with num_closest=3"
            );
            for port in ports.iter() {
                assert_eq!(*port, *MATCHED_PORT);
            }

            // Verify we got exactly 3 features (num_closest limits the output)
            let features = noop.send_features.lock().unwrap();
            assert_eq!(features.len(), 3, "Should have 3 matched features");

            // Extract ids and verify they are the 3 closest candidates
            // Base at 15, candidates at 0(dist 15), 10(dist 5), 20(dist 5), 30(dist 15), 40(dist 25)
            // Closest 3: 10(id=1), 20(id=2), 0(id=0) - distances 5, 5, 15
            let mut ids: Vec<u64> = features
                .iter()
                .filter_map(|f| {
                    if let Some(AttributeValue::Number(id)) =
                        f.attributes.get(&Attribute::new("_neighbor_id"))
                    {
                        Some(id.as_u64().unwrap())
                    } else {
                        None
                    }
                })
                .collect();
            ids.sort();
            assert_eq!(
                ids,
                vec![0, 1, 2],
                "Should match 3 closest candidates (ids 0, 1, 2)"
            );

            // Verify distances are sorted ascending
            let mut distances: Vec<f64> = features
                .iter()
                .filter_map(|f| {
                    if let Some(AttributeValue::Number(d)) =
                        f.attributes.get(&Attribute::new("_neighbor_distance"))
                    {
                        Some(d.as_f64().unwrap())
                    } else {
                        None
                    }
                })
                .collect();
            distances.sort_by(|a, b| a.partial_cmp(b).unwrap());
            // Should be 5.0, 5.0, 15.0 (closest 3)
            assert!(
                (distances[0] - 5.0).abs() < 0.001,
                "First distance should be 5.0"
            );
            assert!(
                (distances[1] - 5.0).abs() < 0.001,
                "Second distance should be 5.0"
            );
            assert!(
                (distances[2] - 15.0).abs() < 0.001,
                "Third distance should be 15.0"
            );
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
            candidate_index: Vec::new(),
            in_memory_candidates: Vec::new(),
            base_features: Vec::new(),
            base_buffer_bytes: 0,
            candidate_buffer_bytes: 0,
            temp_dir: None,
            candidates_file_path: None,
            candidates_file_offset: 0,
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

            // Verify features have correct neighbor indices (0, 1, 2) and names in order
            let features = noop.send_features.lock().unwrap();
            assert_eq!(features.len(), 3, "Should have 3 features");

            // Extract names and indices to verify ordering
            let mut names = Vec::new();
            let mut indices = Vec::new();
            for feature in features.iter() {
                if let Some(AttributeValue::String(name)) =
                    feature.attributes.get(&Attribute::new("_neighbor_name"))
                {
                    names.push(name.clone());
                }
                if let Some(AttributeValue::Number(idx)) =
                    feature.attributes.get(&Attribute::new("_neighbor_index"))
                {
                    indices.push(idx.as_u64().unwrap() as u32);
                }
            }

            // Verify indices are 0, 1, 2 in order
            assert_eq!(indices, vec![0, 1, 2], "Indices should be 0, 1, 2");

            // Verify distances increase with index (sorted by distance)
            let mut distances = Vec::new();
            for feature in features.iter() {
                if let Some(AttributeValue::Number(d)) = feature
                    .attributes
                    .get(&Attribute::new("_neighbor_distance"))
                {
                    distances.push(d.as_f64().unwrap());
                }
            }
            // Verify sorted by ascending distance
            for i in 1..distances.len() {
                assert!(
                    distances[i - 1] <= distances[i],
                    "Distances should be sorted ascending"
                );
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
            candidate_index: Vec::new(),
            in_memory_candidates: Vec::new(),
            base_features: Vec::new(),
            base_buffer_bytes: 0,
            candidate_buffer_bytes: 0,
            temp_dir: None,
            candidates_file_path: None,
            candidates_file_offset: 0,
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

            // Verify the feature has array attributes with 3 values
            let features = noop.send_features.lock().unwrap();
            assert_eq!(features.len(), 1, "Should have 1 feature");
            let feature = &features[0];

            // Verify distances is an array
            let distances = feature
                .attributes
                .get(&Attribute::new("_neighbor_distance"));
            assert!(distances.is_some(), "Should have distance attribute");
            if let Some(AttributeValue::Array(arr)) = distances {
                assert_eq!(arr.len(), 3, "Should have 3 distances in array");
                // Verify sorted ascending
                for i in 1..arr.len() {
                    if let (AttributeValue::Number(d1), AttributeValue::Number(d2)) =
                        (&arr[i - 1], &arr[i])
                    {
                        assert!(
                            d1.as_f64().unwrap() <= d2.as_f64().unwrap(),
                            "Distances should be sorted"
                        );
                    }
                }
            }

            // Verify names is an array with 3 values
            let names = feature.attributes.get(&Attribute::new("_neighbor_name"));
            assert!(names.is_some(), "Should have names array");
            if let Some(AttributeValue::Array(arr)) = names {
                assert_eq!(arr.len(), 3, "Should have 3 names in array");
            }
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
            candidate_index: Vec::new(),
            in_memory_candidates: Vec::new(),
            base_features: Vec::new(),
            base_buffer_bytes: 0,
            candidate_buffer_bytes: 0,
            temp_dir: None,
            candidates_file_path: None,
            candidates_file_offset: 0,
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

            // Verify all 3 features have correct indices
            let features = noop.send_features.lock().unwrap();
            let mut indices: Vec<u32> = features
                .iter()
                .filter_map(|f| {
                    if let Some(AttributeValue::Number(idx)) =
                        f.attributes.get(&Attribute::new("_neighbor_index"))
                    {
                        Some(idx.as_u64().unwrap() as u32)
                    } else {
                        None
                    }
                })
                .collect();
            indices.sort();
            assert_eq!(indices, vec![0, 1, 2], "Should have indices 0, 1, 2");
        }
    }

    #[test]
    fn test_partial_matches_with_max_distance() {
        let mut finder = NeighborFinder {
            params: NeighborFinderParams {
                num_closest: 5,           // Request up to 5 neighbors
                max_distance: Some(15.0), // Only candidates within 15 units
                merge_strategy: MergeStrategy::RepeatBase,
                ..Default::default()
            },
            candidates: Vec::new(),
            candidate_index: Vec::new(),
            in_memory_candidates: Vec::new(),
            base_features: Vec::new(),
            base_buffer_bytes: 0,
            candidate_buffer_bytes: 0,
            temp_dir: None,
            candidates_file_path: None,
            candidates_file_offset: 0,
            base_chunk_count: 0,
            executor_id: None,
        };

        // Candidates at 0, 10, 20, 30, 100 from base at 15
        // Distances: 15, 5, 5, 15, 85
        // With max_distance=15, only 4 are within range (0, 10, 20, 30)
        // Candidate at 100 (distance 85) is beyond max_distance
        let candidates: Vec<_> = (0..4)
            .map(|i| {
                create_point_feature_with_attr(
                    i as f64 * 10.0, // 0, 10, 20, 30
                    0.0,
                    "id",
                    AttributeValue::Number(serde_json::Number::from(i)),
                )
            })
            .collect();
        // Add one more candidate far away
        let far_candidate = create_point_feature_with_attr(
            100.0,
            0.0,
            "id",
            AttributeValue::Number(serde_json::Number::from(4)),
        );

        let base = create_point_feature(15.0, 0.0);

        let noop = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop);

        for candidate in &candidates {
            let mut ctx = create_default_execute_context(candidate);
            ctx.port = CANDIDATE_PORT.clone();
            finder.process(ctx, &fw).unwrap();
        }
        // Process far candidate
        let mut ctx = create_default_execute_context(&far_candidate);
        ctx.port = CANDIDATE_PORT.clone();
        finder.process(ctx, &fw).unwrap();

        let mut ctx = create_default_execute_context(&base);
        ctx.port = BASE_PORT.clone();
        finder.process(ctx, &fw).unwrap();

        let ctx = NodeContext::default();
        finder.finish(ctx, &fw).unwrap();

        if let ProcessorChannelForwarder::Noop(noop) = fw {
            let ports = noop.send_ports.lock().unwrap();
            // Should get only 4 features (within max_distance), not 5 (num_closest)
            // This proves max_distance is actually filtering, not just num_closest limiting
            assert_eq!(
                ports.len(),
                4,
                "Should have 4 features within max_distance, not 5 (num_closest)"
            );

            // Verify we got the correct 4 candidates (ids 0, 1, 2, 3 within max_distance=15)
            // Candidate 4 at distance 85 is excluded
            let features = noop.send_features.lock().unwrap();
            assert_eq!(
                features.len(),
                4,
                "Should have 4 features within max_distance"
            );

            // Extract ids to verify which candidates were matched
            let mut ids: Vec<u64> = features
                .iter()
                .filter_map(|f| {
                    if let Some(AttributeValue::Number(id)) =
                        f.attributes.get(&Attribute::new("_neighbor_id"))
                    {
                        Some(id.as_u64().unwrap())
                    } else {
                        None
                    }
                })
                .collect();
            ids.sort();
            // Should match candidates 0, 1, 2, 3 (at positions 0, 10, 20, 30, distances 15, 5, 5, 15)
            // Candidate 4 at position 100 has distance 85, which exceeds max_distance
            assert_eq!(
                ids,
                vec![0, 1, 2, 3],
                "Should match candidates 0, 1, 2, 3 within max_distance=15, not 4"
            );

            // Verify all distances are <= max_distance
            for feature in features.iter() {
                if let Some(AttributeValue::Number(d)) = feature
                    .attributes
                    .get(&Attribute::new("_neighbor_distance"))
                {
                    let dist = d.as_f64().unwrap();
                    assert!(
                        dist <= 15.0,
                        "Distance {} should be <= max_distance 15",
                        dist
                    );
                }
            }

            // Specifically verify candidate 4 (far) is NOT in results
            assert!(
                !ids.contains(&4),
                "Far candidate (id=4, distance=85) should be filtered by max_distance"
            );
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
            candidate_index: Vec::new(),
            in_memory_candidates: Vec::new(),
            base_features: Vec::new(),
            base_buffer_bytes: 0,
            candidate_buffer_bytes: 0,
            temp_dir: None,
            candidates_file_path: None,
            candidates_file_offset: 0,
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

            // Verify features have distance but no index attribute (since it's suppressed)
            let features = noop.send_features.lock().unwrap();
            assert_eq!(features.len(), 2, "Should have 2 features");

            for feature in features.iter() {
                // Should have distance attribute
                assert!(
                    feature
                        .attributes
                        .get(&Attribute::new("_neighbor_distance"))
                        .is_some(),
                    "Should have distance attribute"
                );
                // Should NOT have index attribute (empty string suppresses it)
                assert!(
                    feature
                        .attributes
                        .get(&Attribute::new("_neighbor_index"))
                        .is_none(),
                    "Index attribute should be suppressed when set to empty string"
                );
            }
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

    /// Test disk spilling behavior by manually setting up a temp directory.
    /// This verifies that the disk-based candidate storage and retrieval works correctly.
    #[test]
    fn test_disk_spilling() {
        use tempfile::tempdir;
        use uuid::Uuid;

        // Create a temporary directory for disk spilling
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_path_buf();

        let mut finder = NeighborFinder {
            params: NeighborFinderParams::default(),
            candidates: Vec::new(),
            candidate_index: Vec::new(),
            in_memory_candidates: Vec::new(),
            base_features: Vec::new(),
            base_buffer_bytes: 0,
            candidate_buffer_bytes: 0,
            temp_dir: Some(temp_path.clone()),
            candidates_file_path: None,
            candidates_file_offset: 0,
            base_chunk_count: 0,
            executor_id: Some(Uuid::new_v4()),
        };

        // Create candidate features
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

        // Use NoopChannelForwarder - note: this returns nil executor_id
        // but we've already set temp_dir manually above
        let noop = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop);

        // Process candidates - they should be flushed to disk when buffer is full
        // or when manually flushed
        for candidate in &candidates {
            let mut ctx = create_default_execute_context(candidate);
            ctx.port = CANDIDATE_PORT.clone();
            finder.process(ctx, &fw).unwrap();
        }

        // Flush candidates to disk
        finder.flush_candidates().unwrap();

        // Verify that candidates JSONL file was created
        let candidates_file_path = temp_path.join("candidates.jsonl");
        assert!(
            candidates_file_path.exists(),
            "Candidates JSONL file should exist"
        );

        // Process base feature
        let mut ctx = create_default_execute_context(&base);
        ctx.port = BASE_PORT.clone();
        finder.process(ctx, &fw).unwrap();

        // Finish processing
        let ctx = NodeContext::default();
        finder.finish(ctx, &fw).unwrap();

        // Verify that the candidates JSONL file was cleaned up
        assert!(
            !candidates_file_path.exists(),
            "Candidates JSONL file should be deleted after processing"
        );

        // Verify output
        if let ProcessorChannelForwarder::Noop(noop) = fw {
            let ports = noop.send_ports.lock().unwrap();
            assert_eq!(ports.len(), 1);
            assert_eq!(ports[0], *MATCHED_PORT);

            // Verify matched feature has correct neighbor from disk
            let features = noop.send_features.lock().unwrap();
            assert_eq!(features.len(), 1, "Should have 1 matched feature");
            let feature = &features[0];

            // Verify distance is correct (base at 15, closest candidate at 10 or 20, distance 5)
            let distance = feature
                .attributes
                .get(&Attribute::new("_neighbor_distance"));
            assert!(distance.is_some(), "Should have distance attribute");
            if let Some(AttributeValue::Number(d)) = distance {
                assert!(
                    (d.as_f64().unwrap() - 5.0).abs() < 0.001,
                    "Distance should be 5.0"
                );
            }

            // Verify transferred name from candidate
            let name = feature.attributes.get(&Attribute::new("_neighbor_name"));
            assert!(name.is_some(), "Should have transferred name attribute");
            // Should be Candidate1 or Candidate2 (both at distance 5)
            if let Some(AttributeValue::String(n)) = name {
                assert!(
                    n == "Candidate1" || n == "Candidate2",
                    "Should match Candidate1 or Candidate2, got {}",
                    n
                );
            }
        }
    }

    /// Test that disk-based candidate reading works correctly.
    /// This tests the `read_candidates_from_disk` function directly.
    #[test]
    fn test_disk_based_candidate_read() {
        use tempfile::tempdir;
        use uuid::Uuid;

        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_path_buf();

        let mut finder = NeighborFinder {
            params: NeighborFinderParams::default(),
            candidates: Vec::new(),
            candidate_index: Vec::new(),
            in_memory_candidates: Vec::new(),
            base_features: Vec::new(),
            base_buffer_bytes: 0,
            candidate_buffer_bytes: 0,
            temp_dir: Some(temp_path.clone()),

            base_chunk_count: 0,
            candidates_file_path: None,
            candidates_file_offset: 0,
            executor_id: Some(Uuid::new_v4()),
        };

        // Create a candidate feature
        let candidate = create_point_feature_with_attr(
            10.0,
            20.0,
            "id",
            AttributeValue::Number(serde_json::Number::from(42)),
        );

        let noop = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop);

        // Process the candidate
        let mut ctx = create_default_execute_context(&candidate);
        ctx.port = CANDIDATE_PORT.clone();
        finder.process(ctx, &fw).unwrap();

        // Flush to disk
        finder.flush_candidates().unwrap();

        // Verify the index was created
        assert_eq!(finder.candidate_index.len(), 1);
        assert_eq!(finder.candidate_index[0].offset, 0); // First candidate at offset 0
        assert_eq!(finder.candidate_index[0].point, [10.0, 20.0]);

        // Read the candidate back from disk using batch API
        let read_candidates = finder
            .read_candidates_from_disk(&[&finder.candidate_index[0]])
            .unwrap();
        assert_eq!(read_candidates.len(), 1);
        let read_candidate = &read_candidates[0];

        // Verify the read candidate matches the original
        assert_eq!(read_candidate.point, [10.0, 20.0]);
        assert_eq!(
            read_candidate.feature.attributes.get(&Attribute::new("id")),
            Some(&AttributeValue::Number(serde_json::Number::from(42)))
        );

        // Cleanup
        finder.cleanup_temp_dir();
    }

    /// Test that Haversine distance with ECEF spatial indexing correctly finds
    /// nearest neighbors near the North Pole, where simple Euclidean distance
    /// on lat/lon coordinates would fail.
    ///
    /// This test verifies the fix for the pole accuracy issue:
    /// - At high latitudes, 1 degree of longitude represents much less distance
    /// - Old approach: Euclidean on (lon, lat) would give wrong nearest neighbor
    /// - New approach: ECEF projection ensures correct spatial ordering
    #[test]
    fn test_haversine_near_pole() {
        // Set up NeighborFinder with Haversine metric
        let mut finder = NeighborFinder {
            params: NeighborFinderParams {
                distance_metric: DistanceMetric::Haversine,
                num_closest: 1,
                merge_strategy: MergeStrategy::Closest,
                ..Default::default()
            },
            candidates: Vec::new(),
            candidate_index: Vec::new(),
            in_memory_candidates: Vec::new(),
            base_features: Vec::new(),
            base_buffer_bytes: 0,
            candidate_buffer_bytes: 0,
            temp_dir: None,
            candidates_file_path: None,
            candidates_file_offset: 0,
            base_chunk_count: 0,
            executor_id: None,
        };

        // Near North Pole (89°N), distances are distorted in lat/lon space
        // Base: (0°E, 89°N) - very close to pole
        let base_lon = 0.0;
        let base_lat = 89.0;
        let base = create_point_feature_with_attr(
            base_lon,
            base_lat,
            "name",
            AttributeValue::String("Base".to_string()),
        );

        // Candidate A: 1° away in longitude (0.01°E, 89°N)
        // Actual distance: ~1.1 km (very close - circling near pole)
        let candidate_a = create_point_feature_with_attr(
            0.01,
            89.0,
            "name",
            AttributeValue::String("CandidateA_Near".to_string()),
        );

        // Candidate B: 1° away in latitude (0°E, 88°N)
        // Actual distance: ~111 km (farther - moving toward equator)
        let candidate_b = create_point_feature_with_attr(
            0.0,
            88.0,
            "name",
            AttributeValue::String("CandidateB_Far".to_string()),
        );

        let noop = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop);

        // Process candidates
        let mut ctx = create_default_execute_context(&candidate_a);
        ctx.port = CANDIDATE_PORT.clone();
        finder.process(ctx, &fw).unwrap();

        let mut ctx = create_default_execute_context(&candidate_b);
        ctx.port = CANDIDATE_PORT.clone();
        finder.process(ctx, &fw).unwrap();

        // Process base
        let mut ctx = create_default_execute_context(&base);
        ctx.port = BASE_PORT.clone();
        finder.process(ctx, &fw).unwrap();

        // Finish and get results
        let ctx = NodeContext::default();
        finder.finish(ctx, &fw).unwrap();

        if let ProcessorChannelForwarder::Noop(noop) = fw {
            let features = noop.send_features.lock().unwrap();
            assert_eq!(features.len(), 1, "Should have one matched feature");

            let feature = &features[0];
            let matched_name = feature
                .attributes
                .get(&Attribute::new("_neighbor_name"))
                .cloned();

            // The key assertion: Should match CandidateA (near) NOT CandidateB (far)
            // In lat/lon Euclidean space, both are "1 degree away", so it would be arbitrary
            // In ECEF/Haversine space, CandidateA is clearly closer (~1km vs ~111km)
            assert_eq!(
                matched_name,
                Some(AttributeValue::String("CandidateA_Near".to_string())),
                "Should match CandidateA (1° lon away, ~1km) not CandidateB (1° lat away, ~111km). \
                 This verifies ECEF projection fixes pole accuracy issue."
            );

            // Verify the distance is approximately correct
            let distance = feature
                .attributes
                .get(&Attribute::new("_neighbor_distance"))
                .and_then(|v| match v {
                    AttributeValue::Number(n) => n.as_f64(),
                    _ => None,
                })
                .unwrap_or(0.0);

            // Distance should be ~1.1 km (not ~111 km)
            assert!(
                distance < 10_000.0, // Less than 10 km
                "Distance should be ~1.1 km, not ~111 km. Got {} m",
                distance
            );
        }
    }

    /// Test ECEF projection directly to verify correct conversion
    /// at various latitudes including poles.
    #[test]
    fn test_ecef_projection() {
        // Test: Points at same latitude, different longitudes
        // The 3D Euclidean distance in ECEF space should reflect the fact that
        // 1 degree of longitude represents different surface distances at different latitudes

        // Helper to compute 3D Euclidean distance between ECEF coordinates
        let ecef_dist = |a: [f64; 3], b: [f64; 3]| {
            ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
        };

        // At equator: points are far apart in 3D space (~111 km surface distance)
        let ecef_eq1 = lat_lon_to_ecef(0.0, 0.0);
        let ecef_eq2 = lat_lon_to_ecef(0.0, 1.0);
        let dist_eq = ecef_dist(ecef_eq1, ecef_eq2);
        // Surface distance ~111 km, chord distance in 3D is slightly less
        assert!(
            dist_eq > 100_000.0 && dist_eq < 120_000.0,
            "Equator 1° distance should be ~111km (surface), got {}m (3D chord)",
            dist_eq
        );

        // At 60° latitude: closer together in 3D space (~55 km surface distance)
        let ecef_60_1 = lat_lon_to_ecef(60.0, 0.0);
        let ecef_60_2 = lat_lon_to_ecef(60.0, 1.0);
        let dist_60 = ecef_dist(ecef_60_1, ecef_60_2);
        assert!(
            dist_60 > 50_000.0 && dist_60 < 60_000.0,
            "60° lat 1° distance should be ~55km (surface), got {}m (3D chord)",
            dist_60
        );

        // Near pole (89°): very close together in 3D space (~2 km surface distance)
        let ecef_89_1 = lat_lon_to_ecef(89.0, 0.0);
        let ecef_89_2 = lat_lon_to_ecef(89.0, 1.0);
        let dist_89 = ecef_dist(ecef_89_1, ecef_89_2);
        assert!(
            dist_89 > 1_000.0 && dist_89 < 3_000.0,
            "89° lat 1° distance should be ~2km (surface), got {}m (3D chord)",
            dist_89
        );

        // Verify ordering: equator > 60° > 89° in terms of 3D distance
        // This is the key property: ECEF preserves the ordering of surface distances
        assert!(
            dist_eq > dist_60 && dist_60 > dist_89,
            "ECEF distances should decrease toward poles: eq={} > 60={} > 89={}",
            dist_eq,
            dist_60,
            dist_89
        );
    }
}
