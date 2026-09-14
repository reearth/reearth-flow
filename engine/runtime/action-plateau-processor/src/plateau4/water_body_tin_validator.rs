//! WaterBody TIN surface validator for the flood-family quality check
//! (fld / htd / ifld / tnm / rfld).
//!
//! This is a single action that fuses what would otherwise be a chain of
//! separate nodes — `FaceExtractor`, `HorizontalReprojector`,
//! `TwoDimensionForcer`, `GeometryValidator`, and `UnsharedEdgeDetector`.
//! They are merged into one action for performance: the huge number of
//! intermediate per-face features stays inside the action instead of crossing
//! channels between nodes, and each gml file can be processed in parallel with
//! rayon.
//!
//! The action is accumulating: per-face work runs (in parallel) in `finish()`,
//! and per-file summaries are emitted only after cross-file unshared-edge
//! detection (grouped by river) completes.
//!
//! The unshared-edge core (fixed-point edges, disk-backed K-way merge, sliding
//! window micro-gap scan) is adapted from `unshared_edge_detector.rs`; here the
//! sink counts micro-gap edges per source file instead of emitting LineStrings.

use std::collections::{BinaryHeap, HashMap, VecDeque};
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use once_cell::sync::Lazy;
use parking_lot::Mutex;
use proj::Proj;
use rayon::prelude::*;
use std::cell::RefCell;

use reearth_flow_geometry::types::coordinate::Coordinate;
use reearth_flow_geometry::types::coordnum::CoordNum;
use reearth_flow_geometry::types::geometry::Geometry2D;
use reearth_flow_geometry::types::line_string::LineString2D;
use reearth_flow_geometry::types::polygon::Polygon2D;
use reearth_flow_geometry::validation::{ValidationType, Validator};
use reearth_flow_runtime::{
    cache::executor_cache_subdir,
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_storage::resolve::StorageResolver;
use reearth_flow_types::{Attribute, AttributeValue, Expr, Feature};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tempfile::TempDir;

use super::errors::PlateauProcessorError;
use super::face_extractor::{
    collect_water_body_pos_lists, parse_pos_list, read_file, set_json_filename_from_udx_dirs,
    validate_pos_list,
};

static SUMMARY_PORT: Lazy<Port> = Lazy::new(|| Port::new("summary"));

/// Fixed-point scale factor (micrometer precision). Assumes projected meters.
const FIXED_POINT_SCALE: f64 = 1_000_000.0;
/// In-memory buffer threshold before flushing edges to disk (10 MB).
const ACCUMULATOR_BUFFER_BYTE_THRESHOLD: usize = 10_485_760;
/// Maximum number of chunk files to merge in one K-way pass.
const MERGE_FAN_IN: usize = 64;
/// Binary size of one serialized edge: 4 x i64 + 1 x u32 = 36 bytes.
const EDGE_BINARY_SIZE: usize = 36;

/// Default source CRS for FaceExtractor-style geographic geometry (JGD2011).
const DEFAULT_SOURCE_EPSG: i64 = 6697;

// Thread-local PROJ cache (proj::Proj is not Send). Each rayon worker keeps its
// own transformer, mirroring HorizontalReprojector.
thread_local! {
    static PROJ_CACHE: RefCell<HashMap<(String, String), Proj>> = RefCell::new(HashMap::new());
}

fn with_proj<F, R>(from_crs: &str, to_crs: &str, f: F) -> Result<R, BoxedError>
where
    F: FnOnce(&Proj) -> Result<R, BoxedError>,
{
    PROJ_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        let key = (from_crs.to_string(), to_crs.to_string());
        if !cache.contains_key(&key) {
            let proj = Proj::new_known_crs(from_crs, to_crs, None).map_err(|e| {
                PlateauProcessorError::WaterBodyTinValidator(format!(
                    "Failed to create PROJ {from_crs} -> {to_crs}: {e}"
                ))
            })?;
            cache.insert(key.clone(), proj);
        }
        let proj = cache.get(&key).unwrap();
        f(proj)
    })
}

#[derive(Debug, Clone, Default)]
pub struct WaterBodyTinValidatorFactory;

impl ProcessorFactory for WaterBodyTinValidatorFactory {
    fn name(&self) -> &str {
        "PLATEAU4.WaterBodyTinValidator"
    }

    fn description(&self) -> &str {
        "Validates WaterBody TIN surfaces end-to-end for the flood-family quality \
         check: extracts faces from CityGML, reprojects, detects degenerate \
         triangles and unshared edges, and emits one summary per file."
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(WaterBodyTinValidatorParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["PLATEAU"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![SUMMARY_PORT.clone()]
    }

    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let params: WaterBodyTinValidatorParam = if let Some(ref with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                PlateauProcessorError::WaterBodyTinValidatorFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                PlateauProcessorError::WaterBodyTinValidatorFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(PlateauProcessorError::WaterBodyTinValidatorFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };

        if params.unshared_edge_tolerance <= 0.0 {
            return Err(PlateauProcessorError::WaterBodyTinValidatorFactory(format!(
                "unsharedEdgeTolerance must be positive, got {}",
                params.unshared_edge_tolerance
            ))
            .into());
        }

        Ok(Box::new(WaterBodyTinValidator {
            params,
            global_params: with,
            buffer: Vec::new(),
        }))
    }
}

/// # WaterBodyTinValidator Parameters
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct WaterBodyTinValidatorParam {
    /// Attribute holding the CityGML file path (default: "path").
    #[serde(default = "default_gml_path_attribute")]
    pub city_gml_path_attribute: Attribute,

    /// Target EPSG code expression for reprojection (e.g. `env.get("prcs")`).
    pub target_epsg_code: Expr,

    /// Source EPSG code (default: 6697, JGD2011 geographic — FaceExtractor output).
    #[serde(default = "default_source_epsg")]
    pub source_epsg_code: i64,

    /// Tolerance for unshared-edge matching in meters (default: 0.1).
    #[serde(default = "default_unshared_edge_tolerance")]
    pub unshared_edge_tolerance: f64,

    /// Group-by attributes for unshared-edge detection (default: [_fld_scale, udxDirs]).
    #[serde(default = "default_unshared_edge_group_by")]
    pub unshared_edge_group_by: Vec<Attribute>,

    /// Tolerance for duplicate-consecutive-points degenerate check (default: 0.009).
    #[serde(default = "default_duplicate_consecutive_points_tolerance")]
    pub duplicate_consecutive_points_tolerance: f64,

    /// Tolerance for corrupt-geometry degenerate check (default: 0.01).
    #[serde(default = "default_corrupt_geometry_tolerance")]
    pub corrupt_geometry_tolerance: f64,
}

fn default_gml_path_attribute() -> Attribute {
    Attribute::new("path")
}
fn default_source_epsg() -> i64 {
    DEFAULT_SOURCE_EPSG
}
fn default_unshared_edge_tolerance() -> f64 {
    0.1
}
fn default_unshared_edge_group_by() -> Vec<Attribute> {
    vec![Attribute::new("_fld_scale"), Attribute::new("udxDirs")]
}
fn default_duplicate_consecutive_points_tolerance() -> f64 {
    0.009
}
fn default_corrupt_geometry_tolerance() -> f64 {
    0.01
}

#[derive(Debug, Clone)]
pub(crate) struct WaterBodyTinValidator {
    params: WaterBodyTinValidatorParam,
    global_params: Option<HashMap<String, Value>>,
    /// One input feature per gml file, buffered until `finish()`.
    buffer: Vec<Feature>,
}

/// Per-file accumulated counts.
#[derive(Debug, Clone, Copy, Default)]
struct FileCounts {
    num_instances: usize,
    num_incorrect_num_vertices: usize,
    num_not_closed: usize,
    num_wrong_orientation: usize,
    num_degenerate_triangles: usize,
}

/// Per-file result carried to summary emission (edges are spilled separately).
struct FileResult {
    file_idx: u32,
    base_feature: Feature,
    counts: FileCounts,
}

impl Processor for WaterBodyTinValidator {
    fn is_accumulating(&self) -> bool {
        true
    }

    fn process(
        &mut self,
        ctx: ExecutorContext,
        _fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        // One feature per gml file; the heavy work runs (in parallel) in finish().
        self.buffer.push(ctx.feature.clone());
        Ok(())
    }

    fn finish(
        &mut self,
        ctx: NodeContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        if self.buffer.is_empty() {
            return Ok(());
        }

        // Resolve target EPSG once (expression is env-based, feature-independent).
        let target_epsg = self.eval_target_epsg(&ctx)?;
        let from_crs = format!("EPSG:{}", self.params.source_epsg_code);
        let to_crs = format!("EPSG:{target_epsg}");

        let validation_types = [
            ValidationType::DuplicateConsecutivePoints(
                self.params.duplicate_consecutive_points_tolerance,
            ),
            ValidationType::CorruptGeometry(Some(self.params.corrupt_geometry_tolerance)),
        ];

        // Disk-backed per-group edge spills, keyed by the group-by attribute values.
        let temp_dir = self.make_temp_dir(fw)?;
        let spills: Mutex<GroupSpills> =
            Mutex::new(GroupSpills::new(temp_dir.path().to_path_buf()));

        let storage_resolver = Arc::clone(&ctx.storage_resolver);
        let group_by = &self.params.unshared_edge_group_by;

        // Parallel per-file: parse -> validate -> reproject -> degenerate check ->
        // edge extraction. Edges are appended to the shared group spills; only the
        // small per-file result is returned.
        let mut results: Vec<FileResult> = self
            .buffer
            .par_iter()
            .enumerate()
            .map(|(idx, feature)| -> Result<FileResult, BoxedError> {
                let file_idx = idx as u32;
                let (base_feature, counts, group_key, edges) = process_one_file(
                    file_idx,
                    feature,
                    &self.params.city_gml_path_attribute,
                    &from_crs,
                    &to_crs,
                    &validation_types,
                    group_by,
                    &storage_resolver,
                )?;

                if !edges.is_empty() {
                    spills.lock().append(group_key, edges)?;
                }

                Ok(FileResult {
                    file_idx,
                    base_feature,
                    counts,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        // Cross-file unshared-edge detection per river group -> per-file counts.
        let mut spills = spills.into_inner();
        spills.flush_all()?;
        let unshared_counts = spills.detect_unshared(self.params.unshared_edge_tolerance)?;

        // Emit one summary per file with all five counts.
        results.sort_by_key(|r| r.file_idx);
        for result in results {
            let num_unshared = unshared_counts.get(&result.file_idx).copied().unwrap_or(0);
            let summary = build_summary(result.base_feature, &result.counts, num_unshared);
            fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                &ctx,
                summary,
                SUMMARY_PORT.clone(),
            ));
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "WaterBodyTinValidator"
    }
}

impl WaterBodyTinValidator {
    fn eval_target_epsg(&self, ctx: &NodeContext) -> Result<i64, BoxedError> {
        let expr_engine = Arc::clone(&ctx.expr_engine);
        let ast = expr_engine
            .compile(self.params.target_epsg_code.as_ref())
            .map_err(|e| {
                PlateauProcessorError::WaterBodyTinValidator(format!(
                    "Failed to compile target EPSG expression: {e:?}"
                ))
            })?;
        let feature = &self.buffer[0];
        let scope = feature.new_scope(expr_engine, &self.global_params);
        let value: i64 = scope.eval_ast(&ast).map_err(|e| {
            PlateauProcessorError::WaterBodyTinValidator(format!(
                "Failed to evaluate target EPSG expression: {e}"
            ))
        })?;
        Ok(value)
    }

    fn make_temp_dir(&self, fw: &ProcessorChannelForwarder) -> Result<TempDir, BoxedError> {
        let parent = executor_cache_subdir(fw.executor_id(), "processors");
        std::fs::create_dir_all(&parent)?;
        let dir = tempfile::Builder::new()
            .prefix("water-body-tin-validator-")
            .tempdir_in(&parent)?;
        Ok(dir)
    }
}

// ---------------------------------------------------------------------------
// Per-file processing
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
fn process_one_file(
    file_idx: u32,
    feature: &Feature,
    path_attribute: &Attribute,
    from_crs: &str,
    to_crs: &str,
    validation_types: &[ValidationType],
    group_by: &[Attribute],
    storage_resolver: &Arc<StorageResolver>,
) -> Result<(Feature, FileCounts, AttributeValue, Vec<Edge>), BoxedError> {
    let group_key = compute_group_key(feature, group_by);

    let file_path_attr = feature.attributes.get(path_attribute).ok_or_else(|| {
        PlateauProcessorError::WaterBodyTinValidator(format!(
            "path attribute `{}` not found",
            path_attribute
        ))
    })?;
    let file_path = file_path_attr.to_string();

    let xml_content = read_file(&file_path, storage_resolver)?;
    let pos_lists = collect_water_body_pos_lists(&xml_content)?;

    let mut counts = FileCounts::default();
    let mut edges = Vec::new();

    for (_, pos_text) in pos_lists {
        let coords = parse_pos_list(&pos_text)?;
        counts.num_instances += 1;

        let validation = validate_pos_list(&coords);
        if validation.is_incorrect_num_vertices {
            counts.num_incorrect_num_vertices += 1;
        }
        if validation.is_not_closed {
            counts.num_not_closed += 1;
        }
        if validation.is_wrong_orientation {
            counts.num_wrong_orientation += 1;
        }

        // Empty geometry: FaceExtractor never builds a polygon, so downstream
        // GeometryValidator rejects it (not counted degenerate) and it yields no
        // edges. Validation counts above already reflect it.
        if coords.is_empty() {
            continue;
        }

        // Reproject (lon,lat) -> plane rectangular meters and force 2D.
        let projected: Vec<(f64, f64)> = with_proj(from_crs, to_crs, |proj| {
            coords
                .iter()
                .map(|(lon, lat, _)| {
                    proj.convert((*lon, *lat)).map_err(|e| {
                        PlateauProcessorError::WaterBodyTinValidator(format!(
                            "Reprojection failed: {e}"
                        ))
                        .into()
                    })
                })
                .collect::<Result<Vec<_>, BoxedError>>()
        })?;

        let exterior: Vec<_> = projected
            .iter()
            .map(|(x, y)| Coordinate::new_(*x, *y))
            .collect();
        let polygon = Polygon2D::new(LineString2D::new(exterior), vec![]);

        // Degenerate (line-like / point-like) detection, matching GeometryValidator
        // on the reprojected 2D geometry.
        let geometry = Geometry2D::Polygon(polygon.clone());
        let is_degenerate = validation_types
            .iter()
            .any(|vt| geometry.validate(vt.clone()).is_some());
        if is_degenerate {
            counts.num_degenerate_triangles += 1;
        }

        // Unshared-edge extraction excludes structurally invalid faces (matches
        // UnsharedEdgeDetector::has_validation_error); wrong orientation is kept.
        if !validation.is_incorrect_num_vertices && !validation.is_not_closed {
            extract_polygon_edges(&polygon, file_idx, &mut edges);
        }
    }

    Ok((feature.clone(), counts, group_key, edges))
}

fn compute_group_key(feature: &Feature, group_by: &[Attribute]) -> AttributeValue {
    AttributeValue::Array(
        group_by
            .iter()
            .map(|attr| {
                feature
                    .attributes
                    .get(attr)
                    .cloned()
                    .unwrap_or(AttributeValue::Null)
            })
            .collect(),
    )
}

fn extract_polygon_edges(polygon: &Polygon2D<f64>, file_idx: u32, edges: &mut Vec<Edge>) {
    let coords: Vec<_> = polygon.exterior().coords().collect();
    for window in coords.windows(2) {
        let p1 = FixedPoint::from_coordinate(window[0]);
        let p2 = FixedPoint::from_coordinate(window[1]);
        if p1 != p2 {
            edges.push(Edge::new(p1, p2, file_idx));
        }
    }
}

/// Build a per-file summary feature carrying all five error counts.
fn build_summary(base_feature: Feature, counts: &FileCounts, num_unshared: u64) -> Feature {
    let mut feature = base_feature;
    let attrs = feature.attributes_mut();

    attrs.insert(Attribute::new("__is_summary"), num(1));
    attrs.insert(
        Attribute::new("_num_instances"),
        num(counts.num_instances as u64),
    );
    attrs.insert(
        Attribute::new("_num_incorrect_num_vertices"),
        num(counts.num_incorrect_num_vertices as u64),
    );
    attrs.insert(
        Attribute::new("_num_not_closed"),
        num(counts.num_not_closed as u64),
    );
    attrs.insert(
        Attribute::new("_num_wrong_orientation"),
        num(counts.num_wrong_orientation as u64),
    );
    attrs.insert(
        Attribute::new("_num_degenerate_triangles"),
        num(counts.num_degenerate_triangles as u64),
    );
    attrs.insert(Attribute::new("_num_unshared_edges"), num(num_unshared));

    // _json_filename derived from udxDirs (flatten '/' to '_'), matching FaceExtractor.
    set_json_filename_from_udx_dirs(&mut feature);

    feature
}

fn num(v: u64) -> AttributeValue {
    AttributeValue::Number(serde_json::Number::from(v))
}

// ---------------------------------------------------------------------------
// Unshared-edge detection (disk-backed, adapted from unshared_edge_detector.rs)
// ---------------------------------------------------------------------------

/// Per-group disk-backed edge spill.
struct GroupSpill {
    dir: PathBuf,
    buffer: Vec<Edge>,
    buffer_bytes: usize,
    chunk_count: usize,
}

impl GroupSpill {
    fn flush(&mut self) -> Result<(), BoxedError> {
        if self.buffer.is_empty() {
            return Ok(());
        }
        self.buffer.sort();
        std::fs::create_dir_all(&self.dir)?;
        let chunk_path = self.dir.join(format!("chunk_{:06}.bin", self.chunk_count));
        let mut writer = BufWriter::new(File::create(&chunk_path)?);
        for edge in &self.buffer {
            write_edge(&mut writer, edge)?;
        }
        writer.flush()?;
        self.chunk_count += 1;
        self.buffer.clear();
        self.buffer_bytes = 0;
        Ok(())
    }
}

/// Collection of per-group spills, keyed by group-by attribute values.
struct GroupSpills {
    base_dir: PathBuf,
    groups: HashMap<AttributeValue, GroupSpill>,
    group_count: usize,
}

impl GroupSpills {
    fn new(base_dir: PathBuf) -> Self {
        Self {
            base_dir,
            groups: HashMap::new(),
            group_count: 0,
        }
    }

    fn append(&mut self, key: AttributeValue, edges: Vec<Edge>) -> Result<(), BoxedError> {
        let group_count = &mut self.group_count;
        let base_dir = &self.base_dir;
        let group = self.groups.entry(key).or_insert_with(|| {
            let idx = *group_count;
            *group_count += 1;
            GroupSpill {
                dir: base_dir.join(format!("group_{idx:06}")),
                buffer: Vec::new(),
                buffer_bytes: 0,
                chunk_count: 0,
            }
        });
        group.buffer_bytes += edges.len() * EDGE_BINARY_SIZE;
        group.buffer.extend(edges);
        if group.buffer_bytes >= ACCUMULATOR_BUFFER_BYTE_THRESHOLD {
            group.flush()?;
        }
        Ok(())
    }

    fn flush_all(&mut self) -> Result<(), BoxedError> {
        for group in self.groups.values_mut() {
            group.flush()?;
        }
        Ok(())
    }

    /// Detect micro-gap (unshared) edges per group and count them per source file.
    fn detect_unshared(&self, tolerance: f64) -> Result<HashMap<u32, u64>, BoxedError> {
        let per_group: Vec<Result<HashMap<u32, u64>, BoxedError>> = self
            .groups
            .values()
            .collect::<Vec<_>>()
            .par_iter()
            .map(|group| detect_group_unshared(&group.dir, group.chunk_count, tolerance))
            .collect();

        let mut totals: HashMap<u32, u64> = HashMap::new();
        for group_result in per_group {
            for (file_idx, count) in group_result? {
                *totals.entry(file_idx).or_insert(0) += count;
            }
        }
        Ok(totals)
    }
}

/// Process one group's chunks: multi-pass K-way merge + sliding-window scan,
/// counting micro-gap edges per source file. Adapted from
/// `unshared_edge_detector::process_group_chunks`, but counts instead of emits.
fn detect_group_unshared(
    dir: &Path,
    chunk_count: usize,
    tolerance: f64,
) -> Result<HashMap<u32, u64>, BoxedError> {
    let mut counts: HashMap<u32, u64> = HashMap::new();
    if chunk_count == 0 {
        return Ok(counts);
    }

    let mut chunk_paths: Vec<PathBuf> = (0..chunk_count)
        .map(|i| dir.join(format!("chunk_{i:06}.bin")))
        .collect();

    // Multi-pass merge until few enough chunks for a single final pass.
    let mut pass: usize = 0;
    while chunk_paths.len() > MERGE_FAN_IN {
        pass += 1;
        let mut next_paths = Vec::new();
        for (group_idx, group) in chunk_paths.chunks(MERGE_FAN_IN).enumerate() {
            let out_path = dir.join(format!("pass_{pass:03}_chunk_{group_idx:06}.bin"));
            merge_edge_chunks_to_file(group, &out_path)?;
            next_paths.push(out_path);
        }
        for p in &chunk_paths {
            let _ = std::fs::remove_file(p);
        }
        chunk_paths = next_paths;
    }

    let tol_fixed = (tolerance * FIXED_POINT_SCALE) as i64;
    let mut readers: Vec<BufReader<File>> = chunk_paths
        .iter()
        .map(|p| File::open(p).map(BufReader::new))
        .collect::<Result<Vec<_>, _>>()?;

    let mut heap: BinaryHeap<HeapEntry> = BinaryHeap::new();
    for (i, reader) in readers.iter_mut().enumerate() {
        if let Some(edge) = read_edge(reader) {
            heap.push(HeapEntry { edge, chunk_idx: i });
        }
    }

    // pending: groups of near-equal edges, ordered by ref edge insertion.
    let mut pending: VecDeque<(Edge, Vec<Edge>)> = VecDeque::new();

    while let Some(entry) = heap.pop() {
        let e = entry.edge;
        if let Some(next) = read_edge(&mut readers[entry.chunk_idx]) {
            heap.push(HeapEntry {
                edge: next,
                chunk_idx: entry.chunk_idx,
            });
        }

        while pending
            .front()
            .is_some_and(|(ref_edge, _)| ref_edge.start.x + tol_fixed < e.start.x)
        {
            if let Some(group) = pending.pop_front() {
                count_microgaps(group, &mut counts);
            }
        }

        let mut placed = false;
        for (ref_edge, members) in &mut pending {
            if e.matches(ref_edge, tolerance) {
                members.push(e.clone());
                placed = true;
                break;
            }
        }
        if !placed {
            pending.push_back((e.clone(), vec![e]));
        }
    }

    for group in pending.drain(..) {
        count_microgaps(group, &mut counts);
    }

    Ok(counts)
}

/// Count each member of a qualifying micro-gap group against its source file.
/// A group is a micro-gap when it has >1 member that are not all identical.
fn count_microgaps(group: (Edge, Vec<Edge>), counts: &mut HashMap<u32, u64>) {
    let (_, members) = group;
    if members.len() <= 1 {
        return;
    }
    let first = &members[0];
    let all_identical = members
        .iter()
        .all(|e| e.start == first.start && e.end == first.end);
    if all_identical {
        return;
    }
    for edge in &members {
        *counts.entry(edge.file_idx).or_insert(0) += 1;
    }
}

fn write_edge<W: Write>(writer: &mut W, edge: &Edge) -> std::io::Result<()> {
    writer.write_all(&edge.start.x.to_le_bytes())?;
    writer.write_all(&edge.start.y.to_le_bytes())?;
    writer.write_all(&edge.end.x.to_le_bytes())?;
    writer.write_all(&edge.end.y.to_le_bytes())?;
    writer.write_all(&edge.file_idx.to_le_bytes())?;
    Ok(())
}

fn read_edge<R: Read>(reader: &mut R) -> Option<Edge> {
    let mut buf = [0u8; EDGE_BINARY_SIZE];
    reader.read_exact(&mut buf).ok()?;
    let sx = i64::from_le_bytes(buf[0..8].try_into().unwrap());
    let sy = i64::from_le_bytes(buf[8..16].try_into().unwrap());
    let ex = i64::from_le_bytes(buf[16..24].try_into().unwrap());
    let ey = i64::from_le_bytes(buf[24..32].try_into().unwrap());
    let file_idx = u32::from_le_bytes(buf[32..36].try_into().unwrap());
    Some(Edge {
        start: FixedPoint { x: sx, y: sy },
        end: FixedPoint { x: ex, y: ey },
        file_idx,
    })
}

fn merge_edge_chunks_to_file(
    chunk_paths: &[PathBuf],
    out_path: &PathBuf,
) -> Result<(), BoxedError> {
    let mut readers: Vec<BufReader<File>> = chunk_paths
        .iter()
        .map(|p| File::open(p).map(BufReader::new))
        .collect::<Result<Vec<_>, _>>()?;
    let mut heap: BinaryHeap<HeapEntry> = BinaryHeap::new();
    for (i, reader) in readers.iter_mut().enumerate() {
        if let Some(edge) = read_edge(reader) {
            heap.push(HeapEntry { edge, chunk_idx: i });
        }
    }
    let mut writer = BufWriter::new(File::create(out_path)?);
    while let Some(entry) = heap.pop() {
        write_edge(&mut writer, &entry.edge)?;
        if let Some(next) = read_edge(&mut readers[entry.chunk_idx]) {
            heap.push(HeapEntry {
                edge: next,
                chunk_idx: entry.chunk_idx,
            });
        }
    }
    writer.flush()?;
    Ok(())
}

/// Directed edge with fixed-point coordinates. `(A,B)` and `(B,A)` normalize to
/// the same edge; `Ord`/`Eq` compare only coordinates (not `file_idx`).
#[derive(Debug, Clone)]
struct Edge {
    start: FixedPoint,
    end: FixedPoint,
    file_idx: u32,
}

impl PartialEq for Edge {
    fn eq(&self, other: &Self) -> bool {
        self.start == other.start && self.end == other.end
    }
}
impl Eq for Edge {}
impl PartialOrd for Edge {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Edge {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.start.cmp(&other.start).then(self.end.cmp(&other.end))
    }
}

impl Edge {
    fn new(p1: FixedPoint, p2: FixedPoint, file_idx: u32) -> Self {
        if (p1.x, p1.y) < (p2.x, p2.y) {
            Self {
                start: p1,
                end: p2,
                file_idx,
            }
        } else {
            Self {
                start: p2,
                end: p1,
                file_idx,
            }
        }
    }

    fn matches(&self, other: &Self, tolerance: f64) -> bool {
        self.start.within_tolerance(&other.start, tolerance)
            && self.end.within_tolerance(&other.end, tolerance)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct FixedPoint {
    x: i64,
    y: i64,
}

impl FixedPoint {
    fn from_coordinate<Z: CoordNum>(coord: &Coordinate<f64, Z>) -> Self {
        Self {
            x: (coord.x * FIXED_POINT_SCALE).round() as i64,
            y: (coord.y * FIXED_POINT_SCALE).round() as i64,
        }
    }

    fn within_tolerance(&self, other: &Self, tolerance: f64) -> bool {
        let tolerance_fixed = (tolerance * FIXED_POINT_SCALE) as i64;
        (self.x - other.x).abs() <= tolerance_fixed && (self.y - other.y).abs() <= tolerance_fixed
    }
}

/// Min-heap entry for K-way merge (`BinaryHeap` is a max-heap; reverse to min).
struct HeapEntry {
    edge: Edge,
    chunk_idx: usize,
}
impl PartialEq for HeapEntry {
    fn eq(&self, other: &Self) -> bool {
        self.edge == other.edge
    }
}
impl Eq for HeapEntry {}
impl PartialOrd for HeapEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for HeapEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.edge.cmp(&self.edge)
    }
}
