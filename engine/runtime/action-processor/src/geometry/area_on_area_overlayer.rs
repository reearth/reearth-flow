use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader, BufWriter, Read as _, Seek, SeekFrom, Write as _},
    path::{Path, PathBuf},
    sync::Arc,
};

use indexmap::IndexMap;
use once_cell::sync::Lazy;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use reearth_flow_geometry::{
    algorithm::{area2d::Area2D, bool_ops::BooleanOps},
    types::{geometry::Geometry2D, multi_polygon::MultiPolygon2D, polygon::Polygon2D},
};
use reearth_flow_runtime::{
    cache::executor_cache_subdir,
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT, REJECTED_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, Feature, Geometry, GeometryValue};
use rstar::{RTree, AABB};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::GeometryProcessorError;
use crate::ACCUMULATOR_BUFFER_BYTE_THRESHOLD;

static AREA_PORT: Lazy<Port> = Lazy::new(|| Port::new("area"));
static REMNANTS_PORT: Lazy<Port> = Lazy::new(|| Port::new("remnants"));

#[derive(Debug, Clone, Default)]
pub(super) struct AreaOnAreaOverlayerFactory;

impl ProcessorFactory for AreaOnAreaOverlayerFactory {
    fn name(&self) -> &str {
        "AreaOnAreaOverlayer"
    }

    fn description(&self) -> &str {
        "Perform Area Overlay Analysis"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(AreaOnAreaOverlayerParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Geometry"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![
            AREA_PORT.clone(),
            REMNANTS_PORT.clone(),
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
        let param: AreaOnAreaOverlayerParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                GeometryProcessorError::AreaOnAreaOverlayerFactory(format!(
                    "Failed to serialize 'with' parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                GeometryProcessorError::AreaOnAreaOverlayerFactory(format!(
                    "Failed to deserialize 'with' parameter: {e}"
                ))
            })?
        } else {
            return Err(GeometryProcessorError::AreaOnAreaOverlayerFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        let process = AreaOnAreaOverlayer {
            group_by: param.group_by,
            output_attribute: param.output_attribute,
            generate_list: param.generate_list,
            accumulation_mode: param.accumulation_mode,
            tolerance: param.tolerance.unwrap_or(0.0),
            group_map: HashMap::new(),
            group_count: 0,
            temp_dir: None,
            buffer: HashMap::new(),
            buffer_bytes: 0,
            executor_id: None,
        };

        Ok(Box::new(process))
    }
}

/// # AreaOnAreaOverlayer Parameters
/// Configure how area overlay analysis is performed
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct AreaOnAreaOverlayerParam {
    /// # Group By Attributes
    /// Optional attributes to group features by during overlay analysis
    group_by: Option<Vec<Attribute>>,

    /// # Accumulation Mode
    /// Controls how attributes from input features are handled in output features
    #[serde(default)]
    accumulation_mode: AccumulationMode,

    /// # Generate List
    /// Name of the list attribute to store source feature attributes
    generate_list: Option<String>,

    /// # Output Attribute
    /// Name of the attribute to store overlap count
    output_attribute: Option<String>,

    /// # Tolerance
    /// Geometric tolerance. Vertices closer than this distance will be considered identical during the overlay operation.
    tolerance: Option<f64>,
}

struct AreaOnAreaOverlayer {
    group_by: Option<Vec<Attribute>>,
    output_attribute: Option<String>,
    generate_list: Option<String>,
    accumulation_mode: AccumulationMode,
    tolerance: f64,
    // Disk-backed state
    group_map: HashMap<AttributeValue, usize>,
    group_count: usize,
    temp_dir: Option<PathBuf>,
    // In-memory buffer: group_idx -> Vec<(aabb_json, feature_json)>
    buffer: HashMap<usize, Vec<(String, String)>>,
    buffer_bytes: usize,
    /// Executor ID for cache isolation, set on first process() call
    executor_id: Option<uuid::Uuid>,
}

impl std::fmt::Debug for AreaOnAreaOverlayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AreaOnAreaOverlayer")
            .field("group_count", &self.group_count)
            .finish_non_exhaustive()
    }
}

impl Clone for AreaOnAreaOverlayer {
    fn clone(&self) -> Self {
        Self {
            group_by: self.group_by.clone(),
            output_attribute: self.output_attribute.clone(),
            generate_list: self.generate_list.clone(),
            accumulation_mode: self.accumulation_mode.clone(),
            tolerance: self.tolerance,
            group_map: HashMap::new(),
            group_count: 0,
            temp_dir: None,
            buffer: HashMap::new(),
            buffer_bytes: 0,
            executor_id: self.executor_id,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub enum AccumulationMode {
    #[default]
    UseAttributesFromOneFeature,
    DropIncomingAttributes,
}

/// Executor-specific engine cache folder for accumulating processors
fn engine_cache_dir(executor_id: uuid::Uuid) -> PathBuf {
    executor_cache_subdir(executor_id, "processors")
}

impl AreaOnAreaOverlayer {
    fn ensure_temp_dir(&mut self) -> Result<&PathBuf, BoxedError> {
        if self.temp_dir.is_none() {
            let executor_id = self.executor_id.unwrap_or_else(uuid::Uuid::nil);
            let dir = engine_cache_dir(executor_id).join(format!("aoa-{}", uuid::Uuid::new_v4()));
            std::fs::create_dir_all(&dir)?;
            self.temp_dir = Some(dir);
        }
        Ok(self.temp_dir.as_ref().unwrap())
    }

    fn ensure_group_dir(&mut self, group_idx: usize) -> Result<PathBuf, BoxedError> {
        let dir = self.ensure_temp_dir()?.clone();
        let group_dir = dir.join(format!("group_{group_idx:06}"));
        std::fs::create_dir_all(&group_dir)?;
        Ok(group_dir)
    }

    fn append_to_group(
        &mut self,
        group_idx: usize,
        aabb: &[f64; 4],
        feature_json: &str,
    ) -> Result<(), BoxedError> {
        let aabb_json = serde_json::to_string(aabb)?;
        self.buffer_bytes += aabb_json.len() + feature_json.len();
        self.buffer
            .entry(group_idx)
            .or_default()
            .push((aabb_json, feature_json.to_string()));

        if self.buffer_bytes >= ACCUMULATOR_BUFFER_BYTE_THRESHOLD {
            self.flush_buffer()?;
        }
        Ok(())
    }

    fn flush_buffer(&mut self) -> Result<(), BoxedError> {
        if self.buffer.is_empty() {
            return Ok(());
        }

        for (group_idx, entries) in std::mem::take(&mut self.buffer) {
            let group_dir = self.ensure_group_dir(group_idx)?;

            // Write aabbs
            {
                let aabbs_file = File::options()
                    .create(true)
                    .append(true)
                    .open(group_dir.join("aabbs.jsonl"))?;
                let mut aabb_w = BufWriter::new(aabbs_file);
                for (aabb_json, _) in &entries {
                    aabb_w.write_all(aabb_json.as_bytes())?;
                    aabb_w.write_all(b"\n")?;
                }
                aabb_w.flush()?;
            }

            // Write features
            {
                let feats_file = File::options()
                    .create(true)
                    .append(true)
                    .open(group_dir.join("features.jsonl"))?;
                let mut feat_w = BufWriter::new(feats_file);
                for (_, feature_json) in &entries {
                    feat_w.write_all(feature_json.as_bytes())?;
                    feat_w.write_all(b"\n")?;
                }
                feat_w.flush()?;
            }
        }

        self.buffer_bytes = 0;
        Ok(())
    }
}

impl Drop for AreaOnAreaOverlayer {
    fn drop(&mut self) {
        if let Some(ref dir) = self.temp_dir {
            let _ = std::fs::remove_dir_all(dir);
        }
    }
}

impl Processor for AreaOnAreaOverlayer {
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
        }

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
            GeometryValue::FlowGeometry2D(geom_2d) => {
                let key = if let Some(group_by) = &self.group_by {
                    AttributeValue::Array(
                        group_by
                            .iter()
                            .filter_map(|attr| feature.attributes.get(attr).cloned())
                            .collect(),
                    )
                } else {
                    AttributeValue::Null
                };

                let group_idx = if let Some(&idx) = self.group_map.get(&key) {
                    idx
                } else {
                    let idx = self.group_count;
                    self.group_map.insert(key, idx);
                    self.group_count += 1;
                    idx
                };

                // Compute AABB from geometry (convert closed LineStrings to Polygon first)
                let mp = geom_to_multipolygon(geom_2d);
                let aabb = mp.bounding_box();
                let aabb = match aabb {
                    Some(rect) => [rect.min().x, rect.min().y, rect.max().x, rect.max().y],
                    None => [0.0, 0.0, 0.0, 0.0],
                };

                let feature_json = serde_json::to_string(&ctx.feature)?;
                self.append_to_group(group_idx, &aabb, &feature_json)?;
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
        // Flush any remaining buffered data to disk
        self.flush_buffer()?;

        let temp_dir = match &self.temp_dir {
            Some(d) => d.clone(),
            None => return Ok(()),
        };

        // Output files are placed in temp_dir. send_file() will move them to the
        // channel buffer directory before this processor's Drop cleans up temp_dir.
        let output_id = uuid::Uuid::new_v4();
        let area_path = temp_dir.join(format!("aoa-area-{output_id}.jsonl"));
        let remnants_path = temp_dir.join(format!("aoa-remnants-{output_id}.jsonl"));
        let mut area_writer = BufWriter::new(File::create(&area_path)?);
        let mut remnants_writer = BufWriter::new(File::create(&remnants_path)?);
        let mut area_count: usize = 0;
        let mut remnants_count: usize = 0;

        for group_idx in 0..self.group_count {
            let group_dir = temp_dir.join(format!("group_{group_idx:06}"));
            let aabbs_path = group_dir.join("aabbs.jsonl");
            let features_path = group_dir.join("features.jsonl");

            // Load AABBs into memory (small: ~32 bytes each)
            let aabbs: Vec<[f64; 4]> = {
                let file = File::open(&aabbs_path)?;
                let reader = BufReader::new(file);
                let mut result = Vec::new();
                for line in reader.lines() {
                    let line = line?;
                    if !line.is_empty() {
                        let aabb: [f64; 4] = serde_json::from_str(&line)?;
                        result.push(aabb);
                    }
                }
                result
            };

            // Pre-scan features.jsonl to record byte offsets
            let disk_feats = DiskBackedFeatures::scan(&features_path)?;

            // Compute midpolygons and write to disk
            let midpolygons_path = group_dir.join("midpolygons.jsonl");
            overlay_2d_disk(&aabbs, &disk_feats, self.tolerance, &midpolygons_path)?;

            // Stream midpolygons from disk, build features, write directly to output files
            let (ac, rc) = from_midpolygons_disk(
                &midpolygons_path,
                &disk_feats,
                &self.output_attribute,
                &self.generate_list,
                &self.accumulation_mode,
                &mut area_writer,
                &mut remnants_writer,
            )?;
            area_count += ac;
            remnants_count += rc;
        }

        area_writer.flush()?;
        remnants_writer.flush()?;
        drop(area_writer);
        drop(remnants_writer);

        let context = ctx.as_context();

        if area_count > 0 {
            fw.send_file(area_path, AREA_PORT.clone(), context.clone());
        }
        if remnants_count > 0 {
            fw.send_file(remnants_path, REMNANTS_PORT.clone(), context);
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "AreaOnAreaOverlayer"
    }
}

/// Provides random access to features stored on disk in a JSONL file.
struct DiskBackedFeatures {
    path: PathBuf,
    offsets: Vec<u64>,
    lengths: Vec<usize>,
}

impl DiskBackedFeatures {
    /// Scan a JSONL file to record byte offsets and lengths for each line.
    fn scan(path: &PathBuf) -> Result<Self, BoxedError> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut offsets = Vec::new();
        let mut lengths = Vec::new();
        let mut offset: u64 = 0;
        let mut line = String::new();
        loop {
            line.clear();
            let bytes_read = reader.read_line(&mut line)?;
            if bytes_read == 0 {
                break;
            }
            let trimmed_len = line.trim_end_matches('\n').len();
            if trimmed_len > 0 {
                offsets.push(offset);
                lengths.push(trimmed_len);
            }
            offset += bytes_read as u64;
        }
        Ok(Self {
            path: path.clone(),
            offsets,
            lengths,
        })
    }

    /// Read and deserialize a feature at the given index.
    /// Each call opens its own file handle, making it safe for parallel use.
    fn read_feature(&self, i: usize) -> Feature {
        let mut file = File::open(&self.path).expect("failed to open features file");
        file.seek(SeekFrom::Start(self.offsets[i]))
            .expect("failed to seek in features file");
        let mut buf = vec![0u8; self.lengths[i]];
        file.read_exact(&mut buf)
            .expect("failed to read feature from disk");
        serde_json::from_slice(&buf).expect("failed to deserialize feature")
    }

    /// Read and extract only the geometry from a feature at the given index.
    fn read_geometry(&self, i: usize) -> Arc<Geometry> {
        let feature = self.read_feature(i);
        feature.geometry
    }
}

/// Polygon that is created in the middle of the overlay process.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct MiddlePolygon {
    polygon: MultiPolygon2D<f64>,
    parents: Vec<usize>,
}

/// Type of the subpolygon and its parents.
enum MiddlePolygonType {
    None,
    Area(Vec<usize>),
    Remnant(usize),
}

impl MiddlePolygon {
    fn get_type(&self) -> MiddlePolygonType {
        match self.parents.len() {
            0 => MiddlePolygonType::None,
            1 => MiddlePolygonType::Remnant(self.parents[0]),
            _ => MiddlePolygonType::Area(self.parents.clone()),
        }
    }
}

/// Extract Geometry2D reference from Arc<Geometry>, or None if not FlowGeometry2D
fn as_geometry_2d(geom: &Arc<Geometry>) -> Option<&Geometry2D<f64>> {
    match &geom.value {
        GeometryValue::FlowGeometry2D(flow_geom) => Some(flow_geom),
        _ => None,
    }
}

/// Convert Geometry2D to MultiPolygon2D.
/// Handles Polygon, MultiPolygon, and closed LineStrings (converted to Polygon).
fn geom_to_multipolygon(geom: &Geometry2D<f64>) -> MultiPolygon2D<f64> {
    match geom {
        Geometry2D::Polygon(poly) => MultiPolygon2D::new(vec![poly.clone()]),
        Geometry2D::MultiPolygon(mp) => mp.clone(),
        Geometry2D::LineString(ls) => {
            let coords: Vec<_> = ls.coords().collect();
            if coords.len() >= 4 && coords.first() == coords.last() {
                let polygon = Polygon2D::new(ls.clone(), vec![]);
                MultiPolygon2D::new(vec![polygon])
            } else {
                MultiPolygon2D::new(vec![])
            }
        }
        _ => MultiPolygon2D::new(vec![]),
    }
}

/// Perform intersection between MultiPolygon2D and Geometry2D
fn bool_op_intersection(mp: &MultiPolygon2D<f64>, geom: &Geometry2D<f64>) -> MultiPolygon2D<f64> {
    let other = geom_to_multipolygon(geom);
    if other.0.is_empty() {
        return MultiPolygon2D::new(vec![]);
    }
    mp.intersection(&other)
}

/// Perform difference between MultiPolygon2D and Geometry2D
fn bool_op_difference(mp: &MultiPolygon2D<f64>, geom: &Geometry2D<f64>) -> MultiPolygon2D<f64> {
    let other = geom_to_multipolygon(geom);
    if other.0.is_empty() {
        return mp.clone();
    }
    mp.difference(&other)
}

/// An AABB entry for the RTree built from pre-computed bounding boxes stored on disk.
#[derive(Clone)]
struct AabbEntry {
    index: usize,
    aabb: AABB<[f64; 2]>,
}

impl rstar::RTreeObject for AabbEntry {
    type Envelope = AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        self.aabb
    }
}

/// Spatial index for finding overlapping AABBs on-the-fly.
/// Instead of precomputing O(n²) adjacency, we query the RTree as needed.
struct AabbIndex {
    tree: RTree<AabbEntry>,
    aabbs: Vec<[f64; 4]>,
}

impl AabbIndex {
    fn build(aabbs: &[[f64; 4]]) -> Self {
        let entries: Vec<AabbEntry> = aabbs
            .iter()
            .enumerate()
            .map(|(i, aabb)| AabbEntry {
                index: i,
                aabb: AABB::from_corners([aabb[0], aabb[1]], [aabb[2], aabb[3]]),
            })
            .collect();

        Self {
            tree: RTree::bulk_load(entries),
            aabbs: aabbs.to_vec(),
        }
    }

    /// Returns indices of AABBs that intersect with AABB at index `i`, excluding `i` itself.
    fn overlapping_indices(&self, i: usize) -> impl Iterator<Item = usize> + '_ {
        let aabb = &self.aabbs[i];
        let envelope = AABB::from_corners([aabb[0], aabb[1]], [aabb[2], aabb[3]]);
        self.tree
            .locate_in_envelope_intersecting(&envelope)
            .filter_map(move |entry| {
                if entry.index != i {
                    Some(entry.index)
                } else {
                    None
                }
            })
    }
}

/// Disk-backed version of overlay_2d that reads geometries from disk on demand
/// and writes MiddlePolygons to a JSONL file instead of collecting in memory.
fn overlay_2d_disk(
    aabbs: &[[f64; 4]],
    disk_feats: &DiskBackedFeatures,
    tolerance: f64,
    output_path: &Path,
) -> Result<(), BoxedError> {
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let aabb_index = AabbIndex::build(aabbs);
    let num = disk_feats.offsets.len();

    // Load all geometries upfront to avoid disk I/O inside parallel iteration
    let geometries: Vec<Arc<Geometry>> = (0..num).map(|i| disk_feats.read_geometry(i)).collect();

    // Parallel iteration with flat_map to collect all results
    let results: Vec<MiddlePolygon> = (0..num)
        .into_par_iter()
        .flat_map(|i| {
            let geom_i = &geometries[i];
            let geom_i_2d = match as_geometry_2d(geom_i) {
                Some(g) => g,
                None => return Vec::new(),
            };

            let mut polygon_target = geom_to_multipolygon(geom_i_2d);

            // Collect overlapping indices once (the iterator is consumed on use)
            let overlapping: Vec<usize> = aabb_index.overlapping_indices(i).collect();

            // cut off the target polygon by upper polygons
            for &j in &overlapping {
                if i < j {
                    let geom_j = &geometries[j];
                    if let Some(geom_j_2d) = as_geometry_2d(geom_j) {
                        polygon_target = bool_op_difference(&polygon_target, geom_j_2d);
                    }
                }
            }

            let mut queue = vec![MiddlePolygon {
                polygon: polygon_target,
                parents: vec![i],
            }];

            // divide the target polygon by lower polygons
            for &j in &overlapping {
                if i > j {
                    let geom_j = &geometries[j];
                    if let Some(geom_j_2d) = as_geometry_2d(geom_j) {
                        let mut new_queue = Vec::new();
                        for subpolygon in queue {
                            let intersection = bool_op_intersection(&subpolygon.polygon, geom_j_2d);

                            let min_area = tolerance * tolerance;
                            let intersection_area = intersection.unsigned_area2d();
                            let is_significant_intersection = intersection_area > min_area;

                            if !intersection.is_empty() && is_significant_intersection {
                                new_queue.push(MiddlePolygon {
                                    polygon: intersection,
                                    parents: subpolygon
                                        .parents
                                        .clone()
                                        .into_iter()
                                        .chain(vec![j])
                                        .collect(),
                                });
                            }

                            let difference = bool_op_difference(&subpolygon.polygon, geom_j_2d);
                            if !difference.is_empty() {
                                new_queue.push(MiddlePolygon {
                                    polygon: difference,
                                    parents: subpolygon.parents.clone(),
                                });
                            }
                        }
                        queue = new_queue;
                    }
                }
            }

            queue
        })
        .collect();

    // Write all results from a single thread
    let mut writer = BufWriter::new(File::create(output_path)?);
    for mp in results {
        let line = serde_json::to_string(&mp)?;
        writer.write_all(line.as_bytes())?;
        writer.write_all(b"\n")?;
    }
    writer.flush()?;

    Ok(())
}

/// Stream MiddlePolygons from a JSONL file, convert to Features, and write
/// directly to area/remnants output files without collecting in memory.
/// Returns (area_count, remnants_count).
fn from_midpolygons_disk(
    midpolygons_path: &Path,
    disk_feats: &DiskBackedFeatures,
    output_attribute: &Option<String>,
    generate_list: &Option<String>,
    accumulation_mode: &AccumulationMode,
    area_writer: &mut BufWriter<File>,
    remnants_writer: &mut BufWriter<File>,
) -> Result<(usize, usize), BoxedError> {
    let file = File::open(midpolygons_path)?;
    let reader = BufReader::new(file);

    // Cache attributes loaded from disk to avoid re-reading the same feature
    let mut attributes_cache: HashMap<usize, Arc<IndexMap<Attribute, AttributeValue>>> =
        HashMap::new();

    let mut area_count = 0usize;
    let mut remnants_count = 0usize;

    for line in reader.lines() {
        let line = line?;
        if line.is_empty() {
            continue;
        }
        let subpolygon: MiddlePolygon = serde_json::from_str(&line)?;

        match subpolygon.get_type() {
            MiddlePolygonType::None => {}
            MiddlePolygonType::Area(parents) => {
                // Ensure all parent attributes are cached
                for &p in &parents {
                    attributes_cache.entry(p).or_insert_with(|| {
                        let feature = disk_feats.read_feature(p);
                        feature.attributes
                    });
                }

                let attrs = match accumulation_mode {
                    AccumulationMode::DropIncomingAttributes => IndexMap::new(),
                    AccumulationMode::UseAttributesFromOneFeature => {
                        let first_feature = &attributes_cache[&parents[0]];
                        (**first_feature).clone()
                    }
                };
                let mut feature = Feature::new_with_attributes(attrs);

                if let Some(attr_name) = output_attribute {
                    let overlap_count = parents.len();
                    feature.attributes_mut().insert(
                        Attribute::new(attr_name.clone()),
                        AttributeValue::Number(overlap_count.into()),
                    );
                }

                if let Some(list_name) = generate_list {
                    let list_items: Vec<AttributeValue> = parents
                        .iter()
                        .map(|&parent_index| {
                            let mut map = HashMap::new();
                            for (attr, value) in &*attributes_cache[&parent_index] {
                                map.insert(attr.as_ref().to_string(), value.clone());
                            }
                            AttributeValue::Map(map)
                        })
                        .collect();

                    feature.attributes_mut().insert(
                        Attribute::new(list_name.clone()),
                        AttributeValue::Array(list_items),
                    );
                }

                feature.geometry_mut().value =
                    GeometryValue::FlowGeometry2D(subpolygon.polygon.into());
                serde_json::to_writer(&mut *area_writer, &feature)?;
                area_writer.write_all(b"\n")?;
                area_count += 1;
            }
            MiddlePolygonType::Remnant(parent) => {
                attributes_cache.entry(parent).or_insert_with(|| {
                    let feature = disk_feats.read_feature(parent);
                    feature.attributes
                });

                let attrs = match accumulation_mode {
                    AccumulationMode::DropIncomingAttributes => IndexMap::new(),
                    AccumulationMode::UseAttributesFromOneFeature => {
                        (*attributes_cache[&parent]).clone()
                    }
                };
                let mut feature = Feature::new_with_attributes(attrs);

                if let Some(attr_name) = output_attribute {
                    feature.attributes_mut().insert(
                        Attribute::new(attr_name.clone()),
                        AttributeValue::Number(1.into()),
                    );
                }

                if let Some(list_name) = generate_list {
                    let mut map = HashMap::new();
                    for (attr, value) in &*attributes_cache[&parent] {
                        map.insert(attr.as_ref().to_string(), value.clone());
                    }
                    let list_items = vec![AttributeValue::Map(map)];

                    feature.attributes_mut().insert(
                        Attribute::new(list_name.clone()),
                        AttributeValue::Array(list_items),
                    );
                }

                feature.geometry_mut().value =
                    GeometryValue::FlowGeometry2D(subpolygon.polygon.into());
                serde_json::to_writer(&mut *remnants_writer, &feature)?;
                remnants_writer.write_all(b"\n")?;
                remnants_count += 1;
            }
        }
    }

    Ok((area_count, remnants_count))
}

#[cfg(test)]
mod tests {
    use reearth_flow_geometry::types::{
        coordinate::Coordinate2D, line_string::LineString2D, polygon::Polygon2D,
    };

    use super::*;

    fn make_geom(coords: Vec<(f64, f64)>) -> Arc<Geometry> {
        let ls = LineString2D::new(
            coords
                .into_iter()
                .map(|(x, y)| Coordinate2D::new_(x, y))
                .collect(),
        );
        Arc::new(Geometry::with_value(GeometryValue::FlowGeometry2D(
            Geometry2D::MultiPolygon(MultiPolygon2D::new(vec![Polygon2D::new(ls, vec![])])),
        )))
    }

    fn make_feature(coords: Vec<(f64, f64)>) -> Feature {
        let geom = make_geom(coords);
        let mut f = Feature::new_with_attributes(IndexMap::new());
        *f.geometry_mut() = (*geom).clone();
        f
    }

    #[test]
    fn test_overlay_two_squares_disk() {
        // Create temp dir and write features to disk
        let dir =
            engine_cache_dir(uuid::Uuid::nil()).join(format!("test-aoa-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let group_dir = dir.join("group_000000");
        std::fs::create_dir_all(&group_dir).unwrap();

        let features = vec![
            make_feature(vec![
                (0.0, 0.0),
                (2.0, 0.0),
                (2.0, 2.0),
                (0.0, 2.0),
                (0.0, 0.0),
            ]),
            make_feature(vec![
                (1.0, 1.0),
                (3.0, 1.0),
                (3.0, 3.0),
                (1.0, 3.0),
                (1.0, 1.0),
            ]),
        ];

        let aabbs: Vec<[f64; 4]> = vec![[0.0, 0.0, 2.0, 2.0], [1.0, 1.0, 3.0, 3.0]];

        // Write features.jsonl
        let features_path = group_dir.join("features.jsonl");
        {
            let mut writer = BufWriter::new(File::create(&features_path).unwrap());
            for f in &features {
                serde_json::to_writer(&mut writer, f).unwrap();
                writer.write_all(b"\n").unwrap();
            }
            writer.flush().unwrap();
        }

        let disk_feats = DiskBackedFeatures::scan(&features_path).unwrap();
        let midpolygons_path = group_dir.join("midpolygons.jsonl");
        overlay_2d_disk(&aabbs, &disk_feats, 0.01, &midpolygons_path).unwrap();
        let count = BufReader::new(File::open(&midpolygons_path).unwrap())
            .lines()
            .filter(|l| l.as_ref().map(|s| !s.is_empty()).unwrap_or(false))
            .count();
        assert_eq!(count, 3);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_overlay_triangles_sharing_an_edge_disk() {
        let dir =
            engine_cache_dir(uuid::Uuid::nil()).join(format!("test-aoa-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let group_dir = dir.join("group_000000");
        std::fs::create_dir_all(&group_dir).unwrap();

        let features = vec![
            make_feature(vec![(0.0, 0.0), (2.0, 0.0), (1.0, 2.0), (0.0, 0.0)]),
            make_feature(vec![(0.0, 0.0), (2.0, 0.0), (1.0, 1.0), (0.0, 0.0)]),
        ];

        let aabbs: Vec<[f64; 4]> = vec![[0.0, 0.0, 2.0, 2.0], [0.0, 0.0, 2.0, 1.0]];

        let features_path = group_dir.join("features.jsonl");
        {
            let mut writer = BufWriter::new(File::create(&features_path).unwrap());
            for f in &features {
                serde_json::to_writer(&mut writer, f).unwrap();
                writer.write_all(b"\n").unwrap();
            }
            writer.flush().unwrap();
        }

        let disk_feats = DiskBackedFeatures::scan(&features_path).unwrap();
        let midpolygons_path = group_dir.join("midpolygons.jsonl");
        overlay_2d_disk(&aabbs, &disk_feats, 0.01, &midpolygons_path).unwrap();
        let count = BufReader::new(File::open(&midpolygons_path).unwrap())
            .lines()
            .filter(|l| l.as_ref().map(|s| !s.is_empty()).unwrap_or(false))
            .count();
        assert_eq!(count, 2);

        let _ = std::fs::remove_dir_all(&dir);
    }
}
