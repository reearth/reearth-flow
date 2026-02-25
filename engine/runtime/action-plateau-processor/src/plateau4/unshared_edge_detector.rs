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

use std::collections::{BinaryHeap, HashMap, VecDeque};
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::PathBuf;

use indexmap::IndexMap;
use once_cell::sync::Lazy;
use reearth_flow_geometry::types::coordinate::Coordinate;
use reearth_flow_geometry::types::coordnum::CoordNum;
use reearth_flow_geometry::types::geometry::Geometry2D;
use reearth_flow_geometry::types::line_string::LineString2D;
use reearth_flow_geometry::types::polygon::Polygon2D;
use reearth_flow_runtime::{
    cache::executor_cache_subdir,
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

/// In-memory buffer threshold before flushing to disk (10 MB)
const ACCUMULATOR_BUFFER_BYTE_THRESHOLD: usize = 10_485_760;

/// Maximum number of chunk files to merge in one pass (K-way merge fan-in)
const MERGE_FAN_IN: usize = 64;

/// Binary size of one serialized edge: 4 × i64 = 32 bytes
const EDGE_BINARY_SIZE: usize = 32;

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

        if param.tolerance <= 0.0 {
            return Err(PlateauProcessorError::UnsharedEdgeDetectorFactory(format!(
                "tolerance must be positive, got {}",
                param.tolerance
            ))
            .into());
        }

        Ok(Box::new(UnsharedEdgeDetector {
            tolerance: param.tolerance,
            executor_id: None,
            temp_dir: None,
            edge_buffer: Vec::new(),
            edge_buffer_bytes: 0,
            chunk_count: 0,
            first_attrs: None,
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
    // Disk-backed state
    executor_id: Option<uuid::Uuid>,
    temp_dir: Option<PathBuf>,
    edge_buffer: Vec<Edge>,
    edge_buffer_bytes: usize,
    chunk_count: usize,
    // Minimal in-memory state (from first valid feature)
    first_attrs: Option<IndexMap<Attribute, AttributeValue>>,
}

impl Drop for UnsharedEdgeDetector {
    fn drop(&mut self) {
        if let Some(ref dir) = self.temp_dir {
            let _ = std::fs::remove_dir_all(dir);
        }
    }
}

impl Processor for UnsharedEdgeDetector {
    fn is_accumulating(&self) -> bool {
        true
    }

    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        // Capture executor_id on first call for cache isolation
        if self.executor_id.is_none() {
            self.executor_id = Some(fw.executor_id());
        }

        // Skip features with validation errors
        if self.has_validation_error(&ctx.feature) {
            return Ok(());
        }

        // Save first valid feature's attributes for populating output features
        if self.first_attrs.is_none() {
            self.first_attrs = Some((*ctx.feature.attributes).clone());
        }

        // Extract edges eagerly (32 bytes each vs. full Feature in memory)
        let new_edges = extract_edges_from_feature(&ctx.feature);
        self.edge_buffer_bytes += new_edges.len() * EDGE_BINARY_SIZE;
        self.edge_buffer.extend(new_edges);

        // Spill to disk when buffer exceeds threshold
        if self.edge_buffer_bytes >= ACCUMULATOR_BUFFER_BYTE_THRESHOLD {
            self.flush_buffer()?;
        }

        Ok(())
    }

    fn finish(
        &mut self,
        ctx: NodeContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        // Flush any remaining in-memory edges
        self.flush_buffer()?;

        // Free buffer memory before the merge phase
        self.edge_buffer = Vec::new();
        self.edge_buffer_bytes = 0;

        if self.chunk_count == 0 {
            return Ok(()); // No valid features processed
        }

        let dir = match &self.temp_dir {
            Some(d) => d.clone(),
            None => return Ok(()),
        };

        // Collect initial chunk paths
        let mut chunk_paths: Vec<PathBuf> = (0..self.chunk_count)
            .map(|i| dir.join(format!("chunk_{:06}.bin", i)))
            .collect();

        // Multi-pass K-way merge: reduce chunks until few enough to merge in one pass
        let mut pass: usize = 0;
        while chunk_paths.len() > MERGE_FAN_IN {
            pass += 1;
            let mut next_paths = Vec::new();

            for (group_idx, group) in chunk_paths.chunks(MERGE_FAN_IN).enumerate() {
                let out_path = dir.join(format!("pass_{pass:03}_chunk_{group_idx:06}.bin"));
                merge_edge_chunks_to_file(group, &out_path)?;
                next_paths.push(out_path);
            }

            // Remove old chunk files
            for p in &chunk_paths {
                let _ = std::fs::remove_file(p);
            }
            chunk_paths = next_paths;
        }

        // Final K-way merge via min-heap, streaming into sliding-window scan
        let first_attrs = self.first_attrs.take().unwrap_or_default();
        let tol_fixed = (self.tolerance * FIXED_POINT_SCALE) as i64;

        let mut readers: Vec<BufReader<File>> = chunk_paths
            .iter()
            .map(|p| File::open(p).map(BufReader::new))
            .collect::<Result<Vec<_>, _>>()?;

        // Seed the min-heap with the first edge from each chunk
        let mut heap: BinaryHeap<HeapEntry> = BinaryHeap::new();
        for (i, reader) in readers.iter_mut().enumerate() {
            if let Some(edge) = read_edge(reader) {
                heap.push(HeapEntry { edge, chunk_idx: i });
            }
        }

        // Sliding-window tolerance scan
        // pending: deque of (ref_edge, members) groups, ordered by ref_edge insertion
        let mut pending: VecDeque<(Edge, Vec<Edge>)> = VecDeque::new();

        while let Some(entry) = heap.pop() {
            let e = entry.edge;

            // Advance the heap with the next edge from the same chunk
            if let Some(next) = read_edge(&mut readers[entry.chunk_idx]) {
                heap.push(HeapEntry {
                    edge: next,
                    chunk_idx: entry.chunk_idx,
                });
            }

            // Close groups whose start.x is too far behind to ever match e
            // (any group with ref.start.x + tol_fixed < e.start.x is complete)
            while pending
                .front()
                .is_some_and(|(ref_edge, _)| ref_edge.start.x + tol_fixed < e.start.x)
            {
                if let Some(group) = pending.pop_front() {
                    emit_microgaps(group, &first_attrs, &ctx, fw);
                }
            }

            // Try to place e into an existing pending group
            let mut placed = false;
            for (ref_edge, members) in &mut pending {
                if e.matches(ref_edge, self.tolerance) {
                    members.push(e.clone());
                    placed = true;
                    break;
                }
            }
            if !placed {
                pending.push_back((e.clone(), vec![e]));
            }
        }

        // Emit all remaining groups
        for group in pending.drain(..) {
            emit_microgaps(group, &first_attrs, &ctx, fw);
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "UnsharedEdgeDetector"
    }
}

impl UnsharedEdgeDetector {
    /// Check if a feature has any validation error flags from FaceExtractor.
    /// Triangles with these errors should not participate in unshared edge detection.
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

        false
    }

    /// Ensure the temporary directory exists, creating it on first call.
    fn ensure_temp_dir(&mut self) -> Result<&PathBuf, BoxedError> {
        if self.temp_dir.is_none() {
            let executor_id = self.executor_id.unwrap_or_else(uuid::Uuid::nil);
            let dir = engine_cache_dir(executor_id)
                .join(format!("unshared-edge-detector-{}", uuid::Uuid::new_v4()));
            std::fs::create_dir_all(&dir)?;
            self.temp_dir = Some(dir);
        }
        Ok(self.temp_dir.as_ref().unwrap())
    }

    /// Sort the in-memory edge buffer and write it as a sorted binary chunk file.
    fn flush_buffer(&mut self) -> Result<(), BoxedError> {
        if self.edge_buffer.is_empty() {
            return Ok(());
        }

        // Sort by (start.x, start.y, end.x, end.y) — derived Ord on Edge/FixedPoint
        self.edge_buffer.sort();

        let dir = self.ensure_temp_dir()?.clone();
        let chunk_path = dir.join(format!("chunk_{:06}.bin", self.chunk_count));
        let file = File::create(&chunk_path)?;
        let mut writer = BufWriter::new(file);

        for edge in &self.edge_buffer {
            write_edge(&mut writer, edge)?;
        }
        writer.flush()?;

        self.chunk_count += 1;
        self.edge_buffer.clear();
        self.edge_buffer_bytes = 0;
        Ok(())
    }
}

/// Returns the executor-specific processors cache directory.
fn engine_cache_dir(executor_id: uuid::Uuid) -> PathBuf {
    executor_cache_subdir(executor_id, "processors")
}

/// Emit a micro-gap group as LineString features.
///
/// A group is a micro-gap when it has more than one member and the members do
/// not all share identical coordinates (i.e., they are within tolerance but not
/// exactly the same — indicating a tiny gap between mesh triangles).
fn emit_microgaps(
    group: (Edge, Vec<Edge>),
    first_attrs: &IndexMap<Attribute, AttributeValue>,
    ctx: &NodeContext,
    fw: &ProcessorChannelForwarder,
) {
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
        let start_x = edge.start.x as f64 / FIXED_POINT_SCALE;
        let start_y = edge.start.y as f64 / FIXED_POINT_SCALE;
        let end_x = edge.end.x as f64 / FIXED_POINT_SCALE;
        let end_y = edge.end.y as f64 / FIXED_POINT_SCALE;

        let line_coords = vec![
            Coordinate::new_(start_x, start_y),
            Coordinate::new_(end_x, end_y),
        ];
        let line = LineString2D::new(line_coords);

        let mut edge_feature = Feature::new_with_attributes(Default::default());
        edge_feature.geometry_mut().value =
            GeometryValue::FlowGeometry2D(Geometry2D::LineString(line));

        // Copy source feature attributes (like udxDirs, _file_index)
        for (key, value) in first_attrs.iter() {
            edge_feature
                .attributes_mut()
                .insert(key.clone(), value.clone());
        }

        fw.send(ExecutorContext::new_with_node_context_feature_and_port(
            ctx,
            edge_feature,
            UNSHARED_PORT.clone(),
        ));
    }
}

/// Write one edge as 4 × i64 little-endian (32 bytes).
fn write_edge<W: Write>(writer: &mut W, edge: &Edge) -> std::io::Result<()> {
    writer.write_all(&edge.start.x.to_le_bytes())?;
    writer.write_all(&edge.start.y.to_le_bytes())?;
    writer.write_all(&edge.end.x.to_le_bytes())?;
    writer.write_all(&edge.end.y.to_le_bytes())?;
    Ok(())
}

/// Read one edge (32 bytes) from a reader; returns `None` on EOF or error.
fn read_edge<R: Read>(reader: &mut R) -> Option<Edge> {
    let mut buf = [0u8; 32];
    reader.read_exact(&mut buf).ok()?;
    let sx = i64::from_le_bytes(buf[0..8].try_into().unwrap());
    let sy = i64::from_le_bytes(buf[8..16].try_into().unwrap());
    let ex = i64::from_le_bytes(buf[16..24].try_into().unwrap());
    let ey = i64::from_le_bytes(buf[24..32].try_into().unwrap());
    Some(Edge {
        start: FixedPoint { x: sx, y: sy },
        end: FixedPoint { x: ex, y: ey },
    })
}

/// Merge multiple sorted binary edge chunk files into one sorted output file.
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

    let file = File::create(out_path)?;
    let mut writer = BufWriter::new(file);
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

/// Extract all edges from a feature's geometry (used eagerly in `process()`).
fn extract_edges_from_feature(feature: &Feature) -> Vec<Edge> {
    let mut edges = Vec::new();
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
    edges
}

/// Extract edges from a single polygon's exterior ring.
fn extract_polygon_edges(polygon: &Polygon2D<f64>) -> Vec<Edge> {
    let mut edges = Vec::new();
    let exterior = polygon.exterior();
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

/// Extract Geometry2D from a GeometryValue.
fn extract_geometry_2d(geometry: &reearth_flow_types::Geometry) -> Option<&Geometry2D<f64>> {
    match &geometry.value {
        GeometryValue::FlowGeometry2D(geom) => Some(geom),
        _ => None,
    }
}

/// Represents a directed edge with fixed-point coordinates.
///
/// Edges are normalized so that `(A, B)` and `(B, A)` produce the same `Edge`
/// (the lexicographically smaller point is always `start`).
///
/// The derived `Ord` sorts by `(start.x, start.y, end.x, end.y)`, which is the
/// key invariant used by the K-way merge and sliding-window scan.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Edge {
    start: FixedPoint,
    end: FixedPoint,
}

/// Fixed-point representation of a 2-D coordinate (micrometer precision).
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

    /// Returns `true` if this point is within `tolerance` meters of `other`.
    fn within_tolerance(&self, other: &Self, tolerance: f64) -> bool {
        let tolerance_fixed = (tolerance * FIXED_POINT_SCALE) as i64;
        let dx = (self.x - other.x).abs();
        let dy = (self.y - other.y).abs();
        dx <= tolerance_fixed && dy <= tolerance_fixed
    }
}

impl Edge {
    fn new(p1: FixedPoint, p2: FixedPoint) -> Self {
        // Normalize direction so (A,B) and (B,A) produce the same edge
        if (p1.x, p1.y) < (p2.x, p2.y) {
            Self { start: p1, end: p2 }
        } else {
            Self { start: p2, end: p1 }
        }
    }

    /// Returns `true` if this edge matches `other` within `tolerance` meters.
    fn matches(&self, other: &Self, tolerance: f64) -> bool {
        self.start.within_tolerance(&other.start, tolerance)
            && self.end.within_tolerance(&other.end, tolerance)
    }
}

/// Entry in the K-way merge min-heap.
///
/// `BinaryHeap` is a max-heap; we reverse the ordering of `Edge` to get
/// min-heap semantics (smallest edge pops first).
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
        // Reverse ordering for min-heap: smallest edge pops first
        other.edge.cmp(&self.edge)
    }
}
