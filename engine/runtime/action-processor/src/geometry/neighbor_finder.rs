use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
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
            candidate_zip_path: None,
            candidate_batch_count: 0,
            candidates_per_batch: 1000, // Each ZIP entry contains ~1000 candidates
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
/// Contains only point location and ZIP entry location - the feature data stays in ZIP.
/// Uses ZIP format with batched entries for efficient random access:
/// - Each entry in the ZIP contains multiple candidates (batch)
/// - Only the needed batch is decompressed, not the entire file
#[derive(Debug, Clone)]
struct CandidateIndex {
    point: [f64; 2],
    point_z: Option<f64>,
    /// ZIP entry name (e.g., "batch_0000.jsonl", "batch_0001.jsonl")
    entry_name: String,
    /// Line number within the ZIP entry (0-based)
    line_number: usize,
}

impl RTreeObject for CandidateIndex {
    type Envelope = AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        AABB::from_point(self.point)
    }
}

impl PointDistance for CandidateIndex {
    fn distance_2(&self, point: &[f64; 2]) -> f64 {
        let dx = self.point[0] - point[0];
        let dy = self.point[1] - point[1];
        dx * dx + dy * dy
    }
}

/// A neighbor match result containing the candidate index and computed distance.
#[derive(Debug, Clone)]
struct NeighborMatch {
    candidate_index: CandidateIndex,
    distance: f64,
    index: usize, // 0-based rank
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
    /// Indexed by the position in candidate_index when entry_name is empty.
    in_memory_candidates: Vec<CandidateEntry>,
    base_features: Vec<BaseFeatureEntry>,
    // Disk spilling fields - track bytes separately for each buffer
    base_buffer_bytes: usize,
    candidate_buffer_bytes: usize,
    temp_dir: Option<PathBuf>,
    /// Path to the ZIP file containing candidate batches (if disk spilling is used)
    candidate_zip_path: Option<PathBuf>,
    /// Counter for batch entries within the ZIP file
    candidate_batch_count: usize,
    /// Number of candidates per ZIP batch entry
    candidates_per_batch: usize,
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

        // Process in-memory base features first
        for entry in &self.base_features {
            self.process_single_base(&ctx, fw, &rtree, &entry.feature)?;
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

    /// Flush candidates to disk as ZIP file with batched entries.
    /// Each batch contains ~1000 candidates, allowing efficient random access:
    /// only the needed batch is decompressed, not the entire file.
    fn flush_candidates(&mut self) -> Result<(), BoxedError> {
        if self.candidates.is_empty() {
            return Ok(());
        }

        let temp_dir = self.temp_dir.as_ref().unwrap();
        std::fs::create_dir_all(temp_dir)?;

        // Initialize ZIP file path if not set
        if self.candidate_zip_path.is_none() {
            self.candidate_zip_path = Some(temp_dir.join("candidates.zip"));
        }
        let zip_path = self.candidate_zip_path.as_ref().unwrap();

        // Open or create ZIP file
        let (file, is_new) = if zip_path.exists() {
            (
                std::fs::OpenOptions::new()
                    .read(true)
                    .write(true)
                    .open(zip_path)?,
                false,
            )
        } else {
            (File::create(zip_path)?, true)
        };

        // Use append mode for existing ZIPs to preserve previous batches
        let mut zip_writer = if is_new {
            zip::ZipWriter::new(file)
        } else {
            zip::ZipWriter::new_append(file)?
        };

        // Group candidates into batches
        let batch_size = self.candidates_per_batch;
        let batches: Vec<_> = self.candidates.chunks(batch_size).collect();

        for batch in batches {
            let entry_name = format!("batch_{:06}.jsonl", self.candidate_batch_count);

            // Prepare batch content as JSON Lines
            let mut batch_content = Vec::new();
            for (line_num, entry) in batch.iter().enumerate() {
                // Create index entry pointing to this candidate's location in ZIP
                self.candidate_index.push(CandidateIndex {
                    point: entry.point,
                    point_z: entry.point_z,
                    entry_name: entry_name.clone(),
                    line_number: line_num,
                });

                // Use pre-serialized bytes to avoid double serialization
                batch_content.extend_from_slice(&entry.serialized);
                batch_content.push(b'\n');
            }

            // Write batch as a ZIP entry with DEFLATE compression
            let options = zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated)
                .compression_level(Some(6));

            zip_writer.start_file(&entry_name, options)?;
            zip_writer.write_all(&batch_content)?;

            self.candidate_batch_count += 1;
        }

        zip_writer.finish()?;
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
                // Flush remaining candidates to ZIP file
                self.flush_candidates()?;
            } else {
                // No temp_dir (e.g., in tests) - keep candidates in memory
                // Store the full entries and create index entries pointing to them
                for (i, entry) in self.candidates.iter().enumerate() {
                    self.in_memory_candidates.push(entry.clone());
                    self.candidate_index.push(CandidateIndex {
                        point: entry.point,
                        point_z: entry.point_z,
                        entry_name: String::new(), // Empty string marks in-memory
                        line_number: i,
                    });
                }
                self.candidates.clear();
                self.candidate_buffer_bytes = 0;
            }
        }

        // Note: Base features are NOT loaded into memory here.
        // They are processed sequentially from disk in finish() to maintain memory efficiency.

        // Candidates are NOT loaded into memory - their index is already built.
        // The candidate ZIP file remains on disk for random access during matching.

        Ok(())
    }

    /// Read candidates from disk using their index entries.
    /// Optimized to open the ZIP file only once per batch of reads.
    /// For in-memory candidates (empty entry_name), retrieves from in_memory_candidates vector.
    fn read_candidates_from_disk(
        &self,
        indices: &[&CandidateIndex],
    ) -> Result<Vec<CandidateEntry>, BoxedError> {
        if indices.is_empty() {
            return Ok(Vec::new());
        }

        // Check if all are in-memory candidates
        if indices.iter().all(|idx| idx.entry_name.is_empty()) {
            return indices
                .iter()
                .map(|idx| {
                    self.in_memory_candidates
                        .get(idx.line_number)
                        .cloned()
                        .ok_or_else(|| {
                            Box::new(std::io::Error::new(
                                std::io::ErrorKind::NotFound,
                                format!(
                                    "In-memory candidate not found at index {}",
                                    idx.line_number
                                ),
                            )) as BoxedError
                        })
                })
                .collect();
        }

        // Open the ZIP file once
        let zip_path = self.candidate_zip_path.as_ref().ok_or_else(|| {
            Box::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Candidate ZIP path not set",
            )) as BoxedError
        })?;

        let file = File::open(zip_path)?;
        let mut zip_archive = zip::ZipArchive::new(BufReader::new(file))?;

        // Group indices by entry name to minimize repeated entry access
        let mut result: Vec<Option<CandidateEntry>> = vec![None; indices.len()];
        let mut entry_groups: HashMap<&str, Vec<(usize, &CandidateIndex)>> = HashMap::new();

        for (i, idx) in indices.iter().enumerate() {
            if !idx.entry_name.is_empty() {
                entry_groups
                    .entry(&idx.entry_name)
                    .or_default()
                    .push((i, idx));
            } else {
                // In-memory candidate
                result[i] = self.in_memory_candidates.get(idx.line_number).cloned();
            }
        }

        // Process each entry group
        for (entry_name, group_indices) in entry_groups {
            let mut entry = zip_archive.by_name(entry_name)?;
            let reader = BufReader::new(&mut entry);

            // Collect all line numbers needed from this entry
            let line_numbers: std::collections::HashSet<usize> = group_indices
                .iter()
                .map(|(_, idx)| idx.line_number)
                .collect();

            // Read lines and collect needed ones
            let mut line_data: HashMap<usize, String> = HashMap::new();
            for (i, line_result) in reader.lines().enumerate() {
                if line_numbers.contains(&i) {
                    line_data.insert(i, line_result?);
                }
                // Early exit if we've found all needed lines
                if line_data.len() == line_numbers.len() {
                    break;
                }
            }

            // Assign results
            for (result_idx, idx) in group_indices {
                if let Some(line) = line_data.get(&idx.line_number) {
                    result[result_idx] = Some(serde_json::from_str(line)?);
                }
            }
        }

        // Convert Option<Vec> to Vec, returning error for any missing entries
        result
            .into_iter()
            .enumerate()
            .map(|(i, opt)| {
                opt.ok_or_else(|| {
                    Box::new(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        format!(
                            "Candidate not found at entry {} line {}",
                            indices[i].entry_name, indices[i].line_number
                        ),
                    )) as BoxedError
                })
            })
            .collect()
    }

    /// Clean up the temporary directory and candidate ZIP file after processing.
    fn cleanup_temp_dir(&mut self) {
        if let Some(ref temp_dir) = self.temp_dir {
            // Delete candidate ZIP file if it exists
            if let Some(ref zip_path) = self.candidate_zip_path {
                let _ = std::fs::remove_file(zip_path);
            }
            // Then remove the directory itself (recursively, in case files remain)
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
}

/// Over-fetch factor for non-2D Euclidean metrics to ensure correctness.
/// The R-tree uses 2D Euclidean ordering, which may differ from Haversine or 3D Euclidean.
const KNN_OVERFETCH_FACTOR: usize = 4;

/// Find the k nearest neighbors for a given base point using the R-tree.
///
/// For Euclidean2D metric, we can use the R-tree ordering directly since the index
/// is built using 2D Euclidean distance.
///
/// For Haversine and Euclidean3D metrics, we over-fetch candidates and re-sort by
/// the actual metric distance to ensure correctness, since the R-tree's 2D ordering
/// may not match the true nearest neighbors in those metrics.
///
/// Uses lightweight CandidateIndex entries (point + disk location only).
fn find_k_nearest_neighbors(
    rtree: &RTree<CandidateIndex>,
    base_point: &[f64; 2],
    base_z: Option<f64>,
    k: usize,
    metric: &DistanceMetric,
) -> Vec<NeighborMatch> {
    match metric {
        DistanceMetric::Euclidean2d => {
            // For 2D Euclidean, R-tree ordering is correct
            rtree
                .nearest_neighbor_iter(base_point)
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
        _ => {
            // For Haversine and Euclidean3D, over-fetch and re-sort by actual metric
            // to ensure we get the true k-nearest neighbors
            let fetch_count = k.saturating_mul(KNN_OVERFETCH_FACTOR).min(rtree.size());
            let mut matches: Vec<NeighborMatch> = rtree
                .nearest_neighbor_iter(base_point)
                .take(fetch_count)
                .cloned()
                .enumerate()
                .map(|(index, candidate_index)| {
                    let distance = compute_distance(
                        metric,
                        base_point,
                        base_z,
                        &candidate_index.point,
                        candidate_index.point_z,
                    );
                    NeighborMatch {
                        candidate_index,
                        distance,
                        index,
                    }
                })
                .collect();

            // Sort by actual distance using the selected metric and take top k
            matches.sort_by(|a, b| {
                a.distance
                    .partial_cmp(&b.distance)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            matches.truncate(k);

            // Re-assign indices based on new ordering
            for (i, m) in matches.iter_mut().enumerate() {
                m.index = i;
            }
            matches
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
            candidate_index: Vec::new(),
            in_memory_candidates: Vec::new(),
            base_features: Vec::new(),
            base_buffer_bytes: 0,
            candidate_buffer_bytes: 0,
            temp_dir: None,
            candidate_zip_path: None,
            candidate_batch_count: 0,
            candidates_per_batch: 1000,
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
            candidate_index: Vec::new(),
            in_memory_candidates: Vec::new(),
            base_features: Vec::new(),
            base_buffer_bytes: 0,
            candidate_buffer_bytes: 0,
            temp_dir: None,
            candidate_zip_path: None,
            candidate_batch_count: 0,
            candidates_per_batch: 1000,
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
            candidate_index: Vec::new(),
            in_memory_candidates: Vec::new(),
            base_features: Vec::new(),
            base_buffer_bytes: 0,
            candidate_buffer_bytes: 0,
            temp_dir: None,
            candidate_zip_path: None,
            candidate_batch_count: 0,
            candidates_per_batch: 1000,
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
            candidate_index: Vec::new(),
            in_memory_candidates: Vec::new(),
            base_features: Vec::new(),
            base_buffer_bytes: 0,
            candidate_buffer_bytes: 0,
            temp_dir: None,
            candidate_zip_path: None,
            candidate_batch_count: 0,
            candidates_per_batch: 1000,
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
            candidate_index: Vec::new(),
            in_memory_candidates: Vec::new(),
            base_features: Vec::new(),
            base_buffer_bytes: 0,
            candidate_buffer_bytes: 0,
            temp_dir: None,
            candidate_zip_path: None,
            candidate_batch_count: 0,
            candidates_per_batch: 1000,
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
            candidate_index: Vec::new(),
            in_memory_candidates: Vec::new(),
            base_features: Vec::new(),
            base_buffer_bytes: 0,
            candidate_buffer_bytes: 0,
            temp_dir: None,
            candidate_zip_path: None,
            candidate_batch_count: 0,
            candidates_per_batch: 1000,
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
            candidate_index: Vec::new(),
            in_memory_candidates: Vec::new(),
            base_features: Vec::new(),
            base_buffer_bytes: 0,
            candidate_buffer_bytes: 0,
            temp_dir: None,
            candidate_zip_path: None,
            candidate_batch_count: 0,
            candidates_per_batch: 1000,
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
            candidate_index: Vec::new(),
            in_memory_candidates: Vec::new(),
            base_features: Vec::new(),
            base_buffer_bytes: 0,
            candidate_buffer_bytes: 0,
            temp_dir: None,
            candidate_zip_path: None,
            candidate_batch_count: 0,
            candidates_per_batch: 1000,
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
            candidate_index: Vec::new(),
            in_memory_candidates: Vec::new(),
            base_features: Vec::new(),
            base_buffer_bytes: 0,
            candidate_buffer_bytes: 0,
            temp_dir: None,
            candidate_zip_path: None,
            candidate_batch_count: 0,
            candidates_per_batch: 1000,
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
            candidate_zip_path: None,
            candidate_batch_count: 0,
            candidates_per_batch: 1000,
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

        // Verify that candidate ZIP file was created
        let zip_path = temp_path.join("candidates.zip");
        assert!(zip_path.exists(), "Candidate ZIP file should exist");

        // Process base feature
        let mut ctx = create_default_execute_context(&base);
        ctx.port = BASE_PORT.clone();
        finder.process(ctx, &fw).unwrap();

        // Finish processing
        let ctx = NodeContext::default();
        finder.finish(ctx, &fw).unwrap();

        // Verify that the candidate ZIP file was cleaned up
        assert!(
            !zip_path.exists(),
            "Candidate ZIP file should be deleted after processing"
        );

        // Verify output
        if let ProcessorChannelForwarder::Noop(noop) = fw {
            let ports = noop.send_ports.lock().unwrap();
            assert_eq!(ports.len(), 1);
            assert_eq!(ports[0], *MATCHED_PORT);
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
            candidate_zip_path: None,
            candidate_batch_count: 0,
            candidates_per_batch: 1000,
            base_chunk_count: 0,
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
        assert_eq!(finder.candidate_index[0].entry_name, "batch_000000.jsonl");
        assert_eq!(finder.candidate_index[0].line_number, 0);
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
}
