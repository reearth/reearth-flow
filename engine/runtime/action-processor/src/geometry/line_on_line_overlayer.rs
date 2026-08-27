use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader, BufWriter, Read as _, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
    sync::Arc,
};

use indexmap::IndexMap;
use once_cell::sync::Lazy;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use reearth_flow_runtime::{
    cache::executor_cache_subdir,
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, FEATURES_PORT, REJECTED_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, Attributes, Feature};
use rstar::{RTree, RTreeObject, AABB};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Number, Value};

#[cfg(not(feature = "new-geometry"))]
use reearth_flow_geometry::{
    algorithm::line_intersection::LineIntersection,
    algorithm::line_string_ops::{LineStringOps, LineStringSplitResult, LineStringWithTree2D},
    types::coordinate::Coordinate2D,
    types::geometry::Geometry2D,
    types::line_string::LineString2D,
    types::no_value::NoValue,
    types::point::{Point, Point2D},
};
#[cfg(not(feature = "new-geometry"))]
use reearth_flow_types::{Geometry, GeometryValue};

#[cfg(feature = "new-geometry")]
use reearth_flow_geometry::{
    coordinate::CoordinateFrame,
    line_string::LineString2D,
    point::Point2D,
    predicates::kernel::{segment_intersection, SegmentIntersection},
    predicates::view::{flatten_2d, Leaf2D},
    Euclidean2DGeometry, Geometry,
};

use super::errors::GeometryProcessorError;
use crate::ACCUMULATOR_BUFFER_BYTE_THRESHOLD;

pub static POINT_PORT: Lazy<Port> = Lazy::new(|| Port::new("point"));
pub static LINE_PORT: Lazy<Port> = Lazy::new(|| Port::new("line"));

/// The attribute the overlap count lands in when the parameter is omitted.
const DEFAULT_OVERLAP_COUNT_ATTRIBUTE: &str = "overlayCount";

fn default_overlap_count_attribute() -> String {
    DEFAULT_OVERLAP_COUNT_ATTRIBUTE.to_string()
}

#[derive(Debug, Clone, Default)]
pub struct LineOnLineOverlayerFactory;

impl ProcessorFactory for LineOnLineOverlayerFactory {
    fn name(&self) -> &str {
        "Line On Line Overlayer"
    }

    fn description(&self) -> &str {
        "Splits lines where they cross, recording on each resulting segment how many input \
         lines run along it, and emits every crossing as a point feature. Inputs must be flat 2D \
         geometries sharing one coordinate frame; place a Two Dimension Forcer or a Coordinate \
         Frame Reprojector upstream to flatten or unify them."
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(LineOnLineOverlayerParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Geometry"]
    }

    fn tags(&self) -> &[&'static str] {
        &["spatial", "aggregation"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![FEATURES_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![POINT_PORT.clone(), LINE_PORT.clone(), REJECTED_PORT.clone()]
    }

    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let params: LineOnLineOverlayerParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                GeometryProcessorError::LineOnLineOverlayerFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                GeometryProcessorError::LineOnLineOverlayerFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(GeometryProcessorError::LineOnLineOverlayerFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        Ok(Box::new(LineOnLineOverlayer {
            group_by: params.group_by,
            tolerance: params.tolerance,
            output_attribute: params.output_attribute,
            list_attribute: params.list_attribute,
            group_map: HashMap::new(),
            #[cfg(feature = "new-geometry")]
            group_frame: HashMap::new(),
            group_count: 0,
            temp_dir: None,
            buffer: HashMap::new(),
            buffer_bytes: 0,
            executor_id: None,
        }))
    }
}

/// # Line On Line Overlayer Parameters
/// Sets which lines are crossed against each other, how far apart two vertices
/// may be and still count as one, and what the resulting segments record about
/// the lines they came from.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct LineOnLineOverlayerParam {
    /// # Tolerance
    /// Distance below which two vertices are treated as the same point, in the
    /// unit of the input's coordinate frame. It decides which crossings split a
    /// line, which crossings are the same crossing, and which segments coincide;
    /// segments shorter than it are dropped. Must be greater than zero, or no
    /// line is ever split.
    tolerance: f64,

    /// # Group By Attributes
    /// Attributes whose values decide which lines are crossed against each
    /// other — only lines matching on all of them are compared. When omitted,
    /// every line is crossed against every other.
    group_by: Option<Vec<Attribute>>,

    /// # Overlap Count Attribute
    /// Attribute that receives the number of input lines running along the
    /// resulting segment. Defaults to `overlayCount`.
    #[serde(default = "default_overlap_count_attribute")]
    output_attribute: String,

    /// # List Attribute
    /// Attribute that receives one entry per line running along the resulting
    /// segment, each holding that line's own attributes. When omitted, no list
    /// is written.
    list_attribute: Option<String>,
}

pub struct LineOnLineOverlayer {
    group_by: Option<Vec<Attribute>>,
    tolerance: f64,
    output_attribute: String,
    list_attribute: Option<String>,
    // Disk-backed state
    group_map: HashMap<AttributeValue, usize>,
    /// The coordinate frame every member of a group must share, fixed by the
    /// group's first feature.
    #[cfg(feature = "new-geometry")]
    group_frame: HashMap<usize, CoordinateFrame>,
    group_count: usize,
    temp_dir: Option<PathBuf>,
    /// group_idx -> Vec<(aabbs_json_for_feature, feature_json)>.
    /// `aabbs_json_for_feature` is a JSON array `[[minx, miny, maxx, maxy], ...]` with one
    /// entry per sub-line-string of the feature. Row i of `aabbs.jsonl` matches row i of
    /// `features.jsonl`.
    buffer: HashMap<usize, Vec<(String, String)>>,
    buffer_bytes: usize,
    executor_id: Option<uuid::Uuid>,
}

impl std::fmt::Debug for LineOnLineOverlayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LineOnLineOverlayer")
            .field("group_count", &self.group_count)
            .finish_non_exhaustive()
    }
}

impl Clone for LineOnLineOverlayer {
    fn clone(&self) -> Self {
        Self {
            group_by: self.group_by.clone(),
            tolerance: self.tolerance,
            output_attribute: self.output_attribute.clone(),
            list_attribute: self.list_attribute.clone(),
            group_map: HashMap::new(),
            #[cfg(feature = "new-geometry")]
            group_frame: HashMap::new(),
            group_count: 0,
            temp_dir: None,
            buffer: HashMap::new(),
            buffer_bytes: 0,
            executor_id: self.executor_id,
        }
    }
}

fn engine_cache_dir(executor_id: uuid::Uuid) -> PathBuf {
    executor_cache_subdir(executor_id, "processors")
}

impl LineOnLineOverlayer {
    fn ensure_temp_dir(&mut self) -> Result<&PathBuf, BoxedError> {
        if self.temp_dir.is_none() {
            let executor_id = self.executor_id.unwrap_or_else(uuid::Uuid::nil);
            let dir = engine_cache_dir(executor_id).join(format!("lol-{}", uuid::Uuid::new_v4()));
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
        aabbs_json: String,
        feature_json: String,
    ) -> Result<(), BoxedError> {
        self.buffer_bytes += aabbs_json.len() + feature_json.len();
        self.buffer
            .entry(group_idx)
            .or_default()
            .push((aabbs_json, feature_json));

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

            let aabbs_file = File::options()
                .create(true)
                .append(true)
                .open(group_dir.join("aabbs.jsonl"))?;
            let mut aabb_w = BufWriter::new(aabbs_file);
            let feats_file = File::options()
                .create(true)
                .append(true)
                .open(group_dir.join("features.jsonl"))?;
            let mut feat_w = BufWriter::new(feats_file);

            for (aabbs_json, feature_json) in &entries {
                aabb_w.write_all(aabbs_json.as_bytes())?;
                aabb_w.write_all(b"\n")?;
                feat_w.write_all(feature_json.as_bytes())?;
                feat_w.write_all(b"\n")?;
            }
            aabb_w.flush()?;
            feat_w.flush()?;
        }

        self.buffer_bytes = 0;
        Ok(())
    }

    /// Whether `frame` matches the group's coordinate frame, which the first
    /// feature of the group fixes. Overlay operands must share one frame.
    #[cfg(feature = "new-geometry")]
    fn admit_frame(&mut self, group_idx: usize, frame: &CoordinateFrame) -> bool {
        use std::collections::hash_map::Entry;
        match self.group_frame.entry(group_idx) {
            Entry::Occupied(entry) => entry.get() == frame,
            Entry::Vacant(entry) => {
                entry.insert(frame.clone());
                true
            }
        }
    }
}

impl Drop for LineOnLineOverlayer {
    fn drop(&mut self) {
        if let Some(ref dir) = self.temp_dir {
            let _ = std::fs::remove_dir_all(dir);
        }
    }
}

impl Processor for LineOnLineOverlayer {
    fn is_accumulating(&self) -> bool {
        true
    }

    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        if self.executor_id.is_none() {
            self.executor_id = Some(fw.executor_id());
        }

        let feature = &ctx.feature;
        let Some(payload) = intake(feature.geometry.as_ref()) else {
            fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
            return Ok(());
        };
        #[cfg(not(feature = "new-geometry"))]
        let aabbs = payload;
        #[cfg(feature = "new-geometry")]
        let (aabbs, frame) = payload;

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

        #[cfg(feature = "new-geometry")]
        if !self.admit_frame(group_idx, &frame) {
            fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
            return Ok(());
        }

        let aabbs_json = serde_json::to_string(&aabbs)?;
        let feature_json = serde_json::to_string(&ctx.feature)?;
        self.append_to_group(group_idx, aabbs_json, feature_json)?;
        Ok(())
    }

    fn finish(
        &mut self,
        ctx: NodeContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        self.flush_buffer()?;

        let temp_dir = match &self.temp_dir {
            Some(d) => d.clone(),
            None => return Ok(()),
        };

        let output_id = uuid::Uuid::new_v4();
        let line_path = temp_dir.join(format!("lol-line-{output_id}.jsonl.zst"));
        let point_path = temp_dir.join(format!("lol-point-{output_id}.jsonl.zst"));
        let mut line_writer = BufWriter::new(zstd::Encoder::new(File::create(&line_path)?, 1)?);
        let mut point_writer = BufWriter::new(zstd::Encoder::new(File::create(&point_path)?, 1)?);
        let mut line_count: usize = 0;
        let mut point_count: usize = 0;

        for group_idx in 0..self.group_count {
            let group_dir = temp_dir.join(format!("group_{group_idx:06}"));
            let (lc, pc) = process_group(
                &group_dir,
                self.tolerance,
                self.group_by.as_deref(),
                &self.output_attribute,
                self.list_attribute.as_deref(),
                &mut line_writer,
                &mut point_writer,
            )?;
            line_count += lc;
            point_count += pc;
        }

        line_writer
            .into_inner()
            .map_err(|e| e.into_error())?
            .finish()?;
        point_writer
            .into_inner()
            .map_err(|e| e.into_error())?
            .finish()?;

        let context = ctx.as_context();

        if line_count > 0 {
            fw.send_file(line_path, LINE_PORT.clone(), context.clone());
        }
        if point_count > 0 {
            fw.send_file(point_path, POINT_PORT.clone(), context);
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "Line On Line Overlayer"
    }
}

// --- world-specific geometry kernel -----------------------------------------

/// The polyline value carried through the overlay.
#[cfg(not(feature = "new-geometry"))]
type SourceLine = LineString2D<f64>;

/// The polyline value carried through the overlay: its coordinates and frame.
#[cfg(feature = "new-geometry")]
type SourceLine = Polyline;

/// An intersection point produced by the overlay.
#[cfg(not(feature = "new-geometry"))]
type SplitPoint = Coordinate2D<f64>;

/// An intersection point produced by the overlay, in the coordinate frame of
/// the polyline it split.
#[cfg(feature = "new-geometry")]
#[derive(Debug, Clone)]
struct SplitPoint {
    frame: CoordinateFrame,
    coord: [f64; 2],
}

/// One polyline participating in the overlay.
#[cfg(feature = "new-geometry")]
#[derive(Debug, Clone)]
struct Polyline {
    /// Coordinate frame the coordinates are expressed in.
    frame: CoordinateFrame,
    /// The vertices, at least two.
    coords: Vec<[f64; 2]>,
}

/// Accept an incoming geometry into the overlay: any 2D geometry with at
/// least one line string. Returns the bounding box of each line string, or
/// `None` when the feature must be rejected.
#[cfg(not(feature = "new-geometry"))]
fn intake(geometry: &Geometry) -> Option<Vec<[f64; 4]>> {
    if geometry.is_empty() {
        return None;
    }
    let GeometryValue::FlowGeometry2D(geom_2d) = &geometry.value else {
        return None;
    };
    let line_strings = extract_line_strings(geom_2d);
    if line_strings.is_empty() {
        return None;
    }
    Some(line_strings.iter().map(aabb_of_line_string).collect())
}

/// Accept an incoming geometry into the overlay: a planar geometry whose line
/// strings and polygon exteriors yield at least one polyline, all in one
/// coordinate frame. Returns the bounding box of each polyline and the frame,
/// or `None` when the feature must be rejected.
///
/// The overlay reasons about the plane alone, so a leaf placed at an elevation
/// is refused rather than crossed with one at a different height.
#[cfg(feature = "new-geometry")]
fn intake(geometry: &Geometry) -> Option<(Vec<[f64; 4]>, CoordinateFrame)> {
    let Geometry::Euclidean2D(geom_2d) = geometry else {
        return None;
    };
    if carries_elevation(geom_2d) {
        return None;
    }
    let polylines = source_lines_2d(geom_2d);
    let frame = polylines.first()?.frame.clone();
    if polylines.iter().any(|pl| pl.frame != frame) {
        return None;
    }
    let aabbs = polylines
        .iter()
        .map(|pl| polyline_bbox(&pl.coords))
        .collect();
    Some((aabbs, frame))
}

#[cfg(not(feature = "new-geometry"))]
fn extract_line_strings(geom: &Geometry2D<f64>) -> Vec<LineString2D<f64>> {
    match geom {
        Geometry2D::LineString(line) => vec![line.clone()],
        Geometry2D::MultiLineString(multi) => multi.0.clone(),
        Geometry2D::Polygon(polygon) => vec![polygon.exterior().clone()],
        Geometry2D::MultiPolygon(multi) => multi.0.iter().map(|p| p.exterior().clone()).collect(),
        _ => Vec::new(),
    }
}

/// The polylines of a 2D geometry: line strings verbatim and polygon exterior
/// rings; other leaves contribute nothing.
#[cfg(feature = "new-geometry")]
fn source_lines_2d(geom: &Euclidean2DGeometry) -> Vec<Polyline> {
    let mut leaves = Vec::new();
    flatten_2d(geom, &mut leaves);
    leaves
        .iter()
        .filter_map(|leaf| match leaf {
            Leaf2D::Line(line) if line.coords().len() >= 2 => Some(Polyline {
                frame: line.frame().clone(),
                coords: line.coords().to_vec(),
            }),
            Leaf2D::Polygon(polygon) if polygon.exterior().len() >= 2 => Some(Polyline {
                frame: polygon.frame().clone(),
                coords: polygon.exterior().to_vec(),
            }),
            _ => None,
        })
        .collect()
}

/// Whether any leaf of the geometry is placed at an elevation.
#[cfg(feature = "new-geometry")]
fn carries_elevation(geom: &Euclidean2DGeometry) -> bool {
    let mut leaves = Vec::new();
    flatten_2d(geom, &mut leaves);
    leaves.iter().any(|leaf| match leaf {
        Leaf2D::Polygon(p) => p.elevation().is_some(),
        Leaf2D::PolygonMesh(m) => m.elevation().is_some(),
        Leaf2D::TriangularMesh(m) => m.elevation().is_some(),
        Leaf2D::Line(l) => l.elevation().is_some(),
        Leaf2D::Point(_) => false,
    })
}

/// The stored feature's polylines, empty when it has no 2D geometry.
#[cfg(not(feature = "new-geometry"))]
fn feature_source_lines(feature: &Feature) -> Vec<SourceLine> {
    match &feature.geometry.value {
        GeometryValue::FlowGeometry2D(g2) => extract_line_strings(g2),
        _ => Vec::new(),
    }
}

/// The stored feature's polylines, empty when it has no 2D geometry.
#[cfg(feature = "new-geometry")]
fn feature_source_lines(feature: &Feature) -> Vec<SourceLine> {
    match feature.geometry.as_ref() {
        Geometry::Euclidean2D(geom_2d) => source_lines_2d(geom_2d),
        _ => Vec::new(),
    }
}

/// A split line as an output feature's geometry.
#[cfg(not(feature = "new-geometry"))]
fn line_output_geometry(line: &SourceLine) -> Geometry {
    Geometry {
        value: GeometryValue::FlowGeometry2D(Geometry2D::LineString(line.clone())),
        ..Default::default()
    }
}

/// A split line as an output feature's geometry, in its source's frame.
#[cfg(feature = "new-geometry")]
fn line_output_geometry(line: &SourceLine) -> Geometry {
    Geometry::Euclidean2D(Euclidean2DGeometry::LineString(LineString2D::from_coords(
        line.frame.clone(),
        line.coords.iter().copied(),
    )))
}

/// An intersection point as an output feature's geometry.
#[cfg(not(feature = "new-geometry"))]
fn point_output_geometry(point: &SplitPoint) -> Geometry {
    Geometry {
        value: GeometryValue::FlowGeometry2D(Geometry2D::Point(Point(*point))),
        ..Default::default()
    }
}

/// An intersection point as an output feature's geometry, in the frame of the
/// polyline it split.
#[cfg(feature = "new-geometry")]
fn point_output_geometry(point: &SplitPoint) -> Geometry {
    Geometry::Euclidean2D(Euclidean2DGeometry::Point(Point2D::new(
        point.frame.clone(),
        point.coord,
    )))
}

#[cfg(not(feature = "new-geometry"))]
fn aabb_of_line_string(ls: &LineString2D<f64>) -> [f64; 4] {
    let env = ls.envelope();
    let lo = env.lower();
    let hi = env.upper();
    [lo.x(), lo.y(), hi.x(), hi.y()]
}

/// The `[min_x, min_y, max_x, max_y]` bounding box of a non-empty polyline.
#[cfg(feature = "new-geometry")]
fn polyline_bbox(coords: &[[f64; 2]]) -> [f64; 4] {
    let mut bbox = [
        f64::INFINITY,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::NEG_INFINITY,
    ];
    for c in coords {
        bbox[0] = bbox[0].min(c[0]);
        bbox[1] = bbox[1].min(c[1]);
        bbox[2] = bbox[2].max(c[0]);
        bbox[3] = bbox[3].max(c[1]);
    }
    bbox
}

fn aabb_to_rstar(aabb: [f64; 4]) -> AABB<[f64; 2]> {
    AABB::from_corners([aabb[0], aabb[1]], [aabb[2], aabb[3]])
}

struct DiskBackedFeatures {
    path: PathBuf,
    offsets: Vec<u64>,
    lengths: Vec<usize>,
}

impl DiskBackedFeatures {
    fn scan(path: &Path) -> Result<Self, BoxedError> {
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
            path: path.to_path_buf(),
            offsets,
            lengths,
        })
    }

    // Opens a fresh fd per call so `&self` callers can be parallelised in the future.
    // The current sole caller is sequential, so the N file opens are wasted — but
    // they scale linearly and the target is a local filesystem, so the overhead
    // is bounded.
    fn read_feature(&self, i: usize) -> Result<Feature, BoxedError> {
        let mut file = File::open(&self.path)?;
        file.seek(SeekFrom::Start(self.offsets[i]))?;
        let mut buf = vec![0u8; self.lengths[i]];
        file.read_exact(&mut buf)?;
        Ok(serde_json::from_slice(&buf)?)
    }

    fn len(&self) -> usize {
        self.offsets.len()
    }

    fn is_empty(&self) -> bool {
        self.offsets.is_empty()
    }
}

#[derive(Clone, Copy)]
struct AabbEntry {
    feature_idx: usize,
    ls_local_idx: usize,
    aabb: AABB<[f64; 2]>,
}

impl RTreeObject for AabbEntry {
    type Envelope = AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        self.aabb
    }
}

fn process_group<W: Write>(
    group_dir: &Path,
    tolerance: f64,
    group_by: Option<&[Attribute]>,
    output_attribute: &str,
    list_attribute: Option<&str>,
    line_writer: &mut W,
    point_writer: &mut W,
) -> Result<(usize, usize), BoxedError> {
    let aabbs_path = group_dir.join("aabbs.jsonl");
    let features_path = group_dir.join("features.jsonl");
    if !aabbs_path.exists() || !features_path.exists() {
        return Ok((0, 0));
    }

    let aabbs_per_feature: Vec<Vec<[f64; 4]>> = {
        let file = File::open(&aabbs_path)?;
        let reader = BufReader::new(file);
        let mut out = Vec::new();
        for line in reader.lines() {
            let line = line?;
            if line.is_empty() {
                continue;
            }
            let v: Vec<[f64; 4]> = serde_json::from_str(&line)?;
            out.push(v);
        }
        out
    };

    let total_entries: usize = aabbs_per_feature.iter().map(|v| v.len()).sum();
    let mut entries: Vec<AabbEntry> = Vec::with_capacity(total_entries);
    for (feature_idx, lss) in aabbs_per_feature.iter().enumerate() {
        for (ls_local_idx, aabb) in lss.iter().enumerate() {
            entries.push(AabbEntry {
                feature_idx,
                ls_local_idx,
                aabb: aabb_to_rstar(*aabb),
            });
        }
    }
    if entries.is_empty() {
        return Ok((0, 0));
    }

    let disk_feats = DiskBackedFeatures::scan(&features_path)?;
    if disk_feats.len() != aabbs_per_feature.len() {
        return Err(Box::new(
            GeometryProcessorError::LineOnLineOverlayerFactory(format!(
                "aabbs/features row count mismatch in {}: aabbs={}, features={}",
                group_dir.display(),
                aabbs_per_feature.len(),
                disk_feats.len(),
            )),
        ));
    }
    let mut attributes_by_feature: Vec<Arc<Attributes>> = Vec::with_capacity(disk_feats.len());
    let mut lss_per_feature: Vec<Vec<SourceLine>> = Vec::with_capacity(disk_feats.len());
    for i in 0..disk_feats.len() {
        let feat = disk_feats.read_feature(i)?;
        let lss = feature_source_lines(&feat);
        attributes_by_feature.push(feat.attributes);
        lss_per_feature.push(lss);
    }

    let overlay = overlay_entries(entries, &lss_per_feature, tolerance);

    let mut line_count: usize = 0;
    for meta in &overlay.line_strings_with_metadata {
        let source_feature_idxs = &meta.source_feature_idxs;

        let mut attributes = Attributes::new();
        attributes.insert(
            Attribute::new(output_attribute),
            AttributeValue::Number(Number::from(source_feature_idxs.len())),
        );

        if let Some(list_attribute) = list_attribute {
            let mut overlaid_list: Vec<AttributeValue> =
                Vec::with_capacity(source_feature_idxs.len());
            for &fi in source_feature_idxs {
                let attrs = &attributes_by_feature[fi];
                let attrs_map: HashMap<String, AttributeValue> = attrs
                    .as_ref()
                    .iter()
                    .map(|(k, v)| (k.clone().inner(), v.clone()))
                    .collect();
                overlaid_list.push(AttributeValue::Map(attrs_map));
            }
            attributes.insert(
                Attribute::new(list_attribute),
                AttributeValue::Array(overlaid_list),
            );
        }

        // A grouping attribute the source line never carried is an absence, not
        // a failure: it is carried forward as absent rather than failing the
        // run, which is what `process` already assumed when it grouped the
        // feature without it.
        if let Some(group_by) = group_by {
            let first_attrs = &attributes_by_feature[source_feature_idxs[0]];
            for gb in group_by {
                if let Some(value) = first_attrs.get(gb) {
                    attributes.insert(gb.clone(), value.clone());
                }
            }
        }

        let geometry = line_output_geometry(&meta.line_string);
        let out = Feature::new_with_attributes_and_geometry(attributes, geometry);
        serde_json::to_writer(&mut *line_writer, &out)?;
        line_writer.write_all(b"\n")?;
        line_count += 1;
    }

    // Point attributes come from the last feature in the group (by insertion order),
    // filtered by group_by if set — preserves pre-rewrite convention.
    let last_feature_idx = if disk_feats.is_empty() {
        None
    } else {
        Some(disk_feats.len() - 1)
    };

    let mut point_count: usize = 0;
    for coord in &overlay.split_coords {
        let attributes: IndexMap<Attribute, AttributeValue> =
            if let (Some(group_by), Some(lfi)) = (group_by, last_feature_idx) {
                let attrs = &attributes_by_feature[lfi];
                group_by
                    .iter()
                    .filter_map(|gb| attrs.get(gb).cloned().map(|v| (gb.clone(), v)))
                    .collect()
            } else {
                IndexMap::new()
            };
        let geometry = point_output_geometry(coord);
        let out = Feature::new_with_attributes_and_geometry(attributes, geometry);
        serde_json::to_writer(&mut *point_writer, &out)?;
        point_writer.write_all(b"\n")?;
        point_count += 1;
    }

    Ok((line_count, point_count))
}

/// A split line and the source features whose lines coincide with it.
#[derive(Debug, Clone)]
struct LineStringWithMetadata {
    line_string: SourceLine,
    /// feature_idx of each source feature that contributed to this segment.
    /// Deduplicated — each feature appears at most once.
    source_feature_idxs: Vec<usize>,
}

#[derive(Debug, Clone)]
struct OverlayResult {
    line_strings_with_metadata: Vec<LineStringWithMetadata>,
    split_coords: Vec<SplitPoint>,
}

#[cfg(not(feature = "new-geometry"))]
fn overlay_entries(
    entries: Vec<AabbEntry>,
    lss_per_feature: &[Vec<LineString2D<f64>>],
    tolerance: f64,
) -> OverlayResult {
    let rtree: RTree<AabbEntry> = RTree::bulk_load(entries);
    let rtree_items: Vec<&AabbEntry> = rtree.iter().collect();

    type PerEntryResult = (Vec<(usize, LineString2D<f64>)>, Vec<Coordinate2D<f64>>);
    let per_entry_results: Vec<PerEntryResult> = rtree_items
        .par_iter()
        .map(|&entry_i| {
            let self_ls = &lss_per_feature[entry_i.feature_idx][entry_i.ls_local_idx];
            let packed = LineStringWithTree2D::new(self_ls.clone());

            // Lazy iteration over R-tree candidates; never materialises the pair list.
            let mut intersection_coords: Vec<Coordinate2D<f64>> = Vec::new();
            for entry_j in rtree.locate_in_envelope_intersecting(&entry_i.aabb) {
                if entry_j.feature_idx == entry_i.feature_idx {
                    continue;
                }
                let other_ls = &lss_per_feature[entry_j.feature_idx][entry_j.ls_local_idx];
                for intersection in packed.intersection(other_ls) {
                    match intersection {
                        LineIntersection::SinglePoint { intersection, .. } => {
                            intersection_coords.push(intersection);
                        }
                        LineIntersection::Collinear { intersection } => {
                            intersection_coords.push(intersection.start);
                            intersection_coords.push(intersection.end);
                        }
                    }
                }
            }

            let LineStringSplitResult {
                split_line_strings,
                split_coords,
            } = packed.split(&intersection_coords, tolerance);

            let segs: Vec<(usize, LineString2D<f64>)> = split_line_strings
                .into_iter()
                .map(|l| (entry_i.feature_idx, l))
                .collect();

            (segs, split_coords)
        })
        .collect();

    let mut segments: Vec<(usize, LineString2D<f64>)> = Vec::new();
    let mut split_coords_flat: Vec<Coordinate2D<f64>> = Vec::new();
    for (segs, coords) in per_entry_results {
        segments.extend(segs);
        split_coords_flat.extend(coords);
    }

    // Drop sub-tolerance segments — zero-length stubs and near-coincident endpoint slivers
    // aren't meaningful overlays and previously dominated the line-port output.
    segments.retain(|(_, ls)| line_string_length_2d(ls) >= tolerance);

    // Two source entries that overlapped geometrically produce identical split segments from
    // different per-entry tasks; we cluster them here.
    let seg_aabbs: Vec<AABB<Point2D<f64>>> = segments.iter().map(|(_, ls)| ls.envelope()).collect();

    #[derive(Clone, Copy)]
    struct SegEntry {
        idx: usize,
        aabb: AABB<Point2D<f64>>,
    }
    impl RTreeObject for SegEntry {
        type Envelope = AABB<Point2D<f64>>;
        fn envelope(&self) -> Self::Envelope {
            self.aabb
        }
    }
    let seg_rtree: RTree<SegEntry> = RTree::bulk_load(
        seg_aabbs
            .iter()
            .enumerate()
            .map(|(idx, aabb)| SegEntry { idx, aabb: *aabb })
            .collect(),
    );

    let mut processed = vec![false; segments.len()];
    let mut line_strings_with_metadata: Vec<LineStringWithMetadata> = Vec::new();
    for i in 0..segments.len() {
        if processed[i] {
            continue;
        }
        let (feat_i, ls1) = segments[i].clone();
        // A single feature may contribute multiple matching segments (e.g. a closed ring
        // whose split produces several arcs that all coincide with the rep segment). Count
        // each feature at most once; extra matching segments dedupe silently.
        let mut source_feature_idxs = vec![feat_i];
        let mut included_feats: std::collections::HashSet<usize> =
            std::collections::HashSet::from([feat_i]);

        for cand in seg_rtree.locate_in_envelope_intersecting(&seg_aabbs[i]) {
            let j = cand.idx;
            if j <= i || processed[j] {
                continue;
            }
            let (feat_j, ls2) = (segments[j].0, &segments[j].1);
            if segments_match(&ls1, ls2, tolerance) {
                if !included_feats.insert(feat_j) {
                    processed[j] = true;
                    continue;
                }
                source_feature_idxs.push(feat_j);
                processed[j] = true;
            }
        }

        line_strings_with_metadata.push(LineStringWithMetadata {
            line_string: ls1,
            source_feature_idxs,
        });
    }

    // Each physical intersection is discovered by both sides of the crossing plus extras
    // from 3+-way near-coincidences, so dedup by tolerance-expanded envelope.
    #[derive(Clone, Copy)]
    struct PointEntry {
        idx: usize,
        point: Point2D<f64>,
    }
    impl RTreeObject for PointEntry {
        type Envelope = AABB<Point2D<f64>>;
        fn envelope(&self) -> Self::Envelope {
            AABB::from_point(self.point)
        }
    }

    let point_rtree: RTree<PointEntry> = RTree::bulk_load(
        split_coords_flat
            .iter()
            .enumerate()
            .map(|(idx, c)| PointEntry {
                idx,
                point: Point2D::new_(c.x, c.y, NoValue),
            })
            .collect(),
    );

    let mut processed_pts = vec![false; split_coords_flat.len()];
    let mut unique_coords: Vec<Coordinate2D<f64>> = Vec::new();
    for i in 0..split_coords_flat.len() {
        if processed_pts[i] {
            continue;
        }
        processed_pts[i] = true;
        let c_i = split_coords_flat[i];
        unique_coords.push(c_i);

        let search_env = AABB::from_corners(
            Point2D::new_(c_i.x - tolerance, c_i.y - tolerance, NoValue),
            Point2D::new_(c_i.x + tolerance, c_i.y + tolerance, NoValue),
        );
        for cand in point_rtree.locate_in_envelope_intersecting(&search_env) {
            let j = cand.idx;
            if j <= i || processed_pts[j] {
                continue;
            }
            let c_j = split_coords_flat[j];
            if (c_i - c_j).norm() < tolerance {
                processed_pts[j] = true;
            }
        }
    }

    OverlayResult {
        line_strings_with_metadata,
        split_coords: unique_coords,
    }
}

#[cfg(feature = "new-geometry")]
fn overlay_entries(
    entries: Vec<AabbEntry>,
    lss_per_feature: &[Vec<Polyline>],
    tolerance: f64,
) -> OverlayResult {
    let rtree: RTree<AabbEntry> = RTree::bulk_load(entries);
    let rtree_items: Vec<&AabbEntry> = rtree.iter().collect();

    type PerEntryResult = (Vec<(usize, Polyline)>, Vec<SplitPoint>);
    let per_entry_results: Vec<PerEntryResult> = rtree_items
        .par_iter()
        .map(|&entry_i| {
            let self_pl = &lss_per_feature[entry_i.feature_idx][entry_i.ls_local_idx];

            // Lazy iteration over R-tree candidates; never materialises the pair list.
            let mut intersection_coords: Vec<[f64; 2]> = Vec::new();
            for entry_j in rtree.locate_in_envelope_intersecting(&entry_i.aabb) {
                if entry_j.feature_idx == entry_i.feature_idx {
                    continue;
                }
                let other = &lss_per_feature[entry_j.feature_idx][entry_j.ls_local_idx];
                polyline_intersections(&self_pl.coords, &other.coords, &mut intersection_coords);
            }

            let (split_lines, split_coords) =
                split_polyline(&self_pl.coords, &intersection_coords, tolerance);

            let segs: Vec<(usize, Polyline)> = split_lines
                .into_iter()
                .map(|coords| {
                    (
                        entry_i.feature_idx,
                        Polyline {
                            frame: self_pl.frame.clone(),
                            coords,
                        },
                    )
                })
                .collect();
            let points: Vec<SplitPoint> = split_coords
                .into_iter()
                .map(|coord| SplitPoint {
                    frame: self_pl.frame.clone(),
                    coord,
                })
                .collect();

            (segs, points)
        })
        .collect();

    let mut segments: Vec<(usize, Polyline)> = Vec::new();
    let mut split_points_flat: Vec<SplitPoint> = Vec::new();
    for (segs, points) in per_entry_results {
        segments.extend(segs);
        split_points_flat.extend(points);
    }

    // Drop sub-tolerance segments — zero-length stubs and near-coincident endpoint slivers
    // aren't meaningful overlays and previously dominated the line-port output.
    segments.retain(|(_, pl)| polyline_length(&pl.coords) >= tolerance);

    // Two source entries that overlapped geometrically produce identical split segments from
    // different per-entry tasks; we cluster them here.
    let seg_aabbs: Vec<AABB<[f64; 2]>> = segments
        .iter()
        .map(|(_, pl)| aabb_to_rstar(polyline_bbox(&pl.coords)))
        .collect();

    #[derive(Clone, Copy)]
    struct SegEntry {
        idx: usize,
        aabb: AABB<[f64; 2]>,
    }
    impl RTreeObject for SegEntry {
        type Envelope = AABB<[f64; 2]>;
        fn envelope(&self) -> Self::Envelope {
            self.aabb
        }
    }
    let seg_rtree: RTree<SegEntry> = RTree::bulk_load(
        seg_aabbs
            .iter()
            .enumerate()
            .map(|(idx, aabb)| SegEntry { idx, aabb: *aabb })
            .collect(),
    );

    let mut processed = vec![false; segments.len()];
    let mut line_strings_with_metadata: Vec<LineStringWithMetadata> = Vec::new();
    for i in 0..segments.len() {
        if processed[i] {
            continue;
        }
        let (feat_i, rep) = segments[i].clone();
        // A single feature may contribute multiple matching segments (e.g. a closed ring
        // whose split produces several arcs that all coincide with the rep segment). Count
        // each feature at most once; extra matching segments dedupe silently.
        let mut source_feature_idxs = vec![feat_i];
        let mut included_feats: std::collections::HashSet<usize> =
            std::collections::HashSet::from([feat_i]);

        for cand in seg_rtree.locate_in_envelope_intersecting(&seg_aabbs[i]) {
            let j = cand.idx;
            if j <= i || processed[j] {
                continue;
            }
            let (feat_j, pl_j) = (segments[j].0, &segments[j].1);
            if coords_match(&rep.coords, &pl_j.coords, tolerance) {
                if !included_feats.insert(feat_j) {
                    processed[j] = true;
                    continue;
                }
                source_feature_idxs.push(feat_j);
                processed[j] = true;
            }
        }

        line_strings_with_metadata.push(LineStringWithMetadata {
            line_string: rep,
            source_feature_idxs,
        });
    }

    // Each physical intersection is discovered by both sides of the crossing plus extras
    // from 3+-way near-coincidences, so dedup by tolerance-expanded envelope.
    #[derive(Clone, Copy)]
    struct PointEntry {
        idx: usize,
        point: [f64; 2],
    }
    impl RTreeObject for PointEntry {
        type Envelope = AABB<[f64; 2]>;
        fn envelope(&self) -> Self::Envelope {
            AABB::from_point(self.point)
        }
    }

    let point_rtree: RTree<PointEntry> = RTree::bulk_load(
        split_points_flat
            .iter()
            .enumerate()
            .map(|(idx, p)| PointEntry {
                idx,
                point: p.coord,
            })
            .collect(),
    );

    let mut processed_pts = vec![false; split_points_flat.len()];
    let mut unique_points: Vec<SplitPoint> = Vec::new();
    for i in 0..split_points_flat.len() {
        if processed_pts[i] {
            continue;
        }
        processed_pts[i] = true;
        let p_i = split_points_flat[i].clone();

        let search_env = AABB::from_corners(
            [p_i.coord[0] - tolerance, p_i.coord[1] - tolerance],
            [p_i.coord[0] + tolerance, p_i.coord[1] + tolerance],
        );
        for cand in point_rtree.locate_in_envelope_intersecting(&search_env) {
            let j = cand.idx;
            if j <= i || processed_pts[j] {
                continue;
            }
            if dist(p_i.coord, split_points_flat[j].coord) < tolerance {
                processed_pts[j] = true;
            }
        }
        unique_points.push(p_i);
    }

    OverlayResult {
        line_strings_with_metadata,
        split_coords: unique_points,
    }
}

#[cfg(not(feature = "new-geometry"))]
fn line_string_length_2d(ls: &LineString2D<f64>) -> f64 {
    ls.0.windows(2).map(|w| (w[1] - w[0]).norm()).sum()
}

#[cfg(not(feature = "new-geometry"))]
fn segments_match(a: &LineString2D<f64>, b: &LineString2D<f64>, tolerance: f64) -> bool {
    if a.0.len() != b.0.len() {
        return false;
    }
    let forward =
        a.0.iter()
            .zip(b.0.iter())
            .all(|(&c1, &c2)| (c1 - c2).norm() < tolerance);
    if forward {
        return true;
    }
    a.0.iter()
        .rev()
        .zip(b.0.iter())
        .all(|(&c1, &c2)| (c1 - c2).norm() < tolerance)
}

/// The distance between two 2D coordinates.
#[cfg(feature = "new-geometry")]
fn dist(a: [f64; 2], b: [f64; 2]) -> f64 {
    (a[0] - b[0]).hypot(a[1] - b[1])
}

/// The length of a polyline.
#[cfg(feature = "new-geometry")]
fn polyline_length(coords: &[[f64; 2]]) -> f64 {
    coords.windows(2).map(|w| dist(w[0], w[1])).sum()
}

/// Whether two polylines coincide vertex by vertex within `tolerance`, in
/// either direction.
#[cfg(feature = "new-geometry")]
fn coords_match(a: &[[f64; 2]], b: &[[f64; 2]], tolerance: f64) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let forward = a
        .iter()
        .zip(b.iter())
        .all(|(&c1, &c2)| dist(c1, c2) < tolerance);
    if forward {
        return true;
    }
    a.iter()
        .rev()
        .zip(b.iter())
        .all(|(&c1, &c2)| dist(c1, c2) < tolerance)
}

/// Every pairwise segment intersection of two polylines, appended to `out`:
/// crossing points, endpoint touches, and both ends of collinear overlaps.
#[cfg(feature = "new-geometry")]
fn polyline_intersections(a: &[[f64; 2]], b: &[[f64; 2]], out: &mut Vec<[f64; 2]>) {
    for sa in a.windows(2) {
        for sb in b.windows(2) {
            match segment_intersection(sa[0], sa[1], sb[0], sb[1]) {
                Some(SegmentIntersection::SinglePoint { intersection, .. }) => {
                    out.push(intersection);
                }
                Some(SegmentIntersection::Collinear { start, end }) => {
                    out.push(start);
                    out.push(end);
                }
                None => {}
            }
        }
    }
}

/// Split a polyline at the candidate points. A candidate splits a segment when
/// it lies on it within `tolerance` and is not within `tolerance` of either
/// endpoint; a candidate that fails but coincides with an interior vertex
/// splits the polyline at that vertex instead. Returns the sub-polylines and
/// the coordinates where splits occurred.
#[cfg(feature = "new-geometry")]
fn split_polyline(
    coords: &[[f64; 2]],
    candidates: &[[f64; 2]],
    tolerance: f64,
) -> (Vec<Vec<[f64; 2]>>, Vec<[f64; 2]>) {
    if coords.len() < 2 {
        return (Vec::new(), Vec::new());
    }

    let mut split_coords: Vec<[f64; 2]> = Vec::new();
    let mut ignored: Vec<[f64; 2]> = Vec::new();
    let mut lines: Vec<Vec<[f64; 2]>> = Vec::new();
    let mut current: Vec<[f64; 2]> = vec![coords[0]];

    for w in coords.windows(2) {
        let (start, end) = (w[0], w[1]);
        let lo = [
            start[0].min(end[0]) - tolerance,
            start[1].min(end[1]) - tolerance,
        ];
        let hi = [
            start[0].max(end[0]) + tolerance,
            start[1].max(end[1]) + tolerance,
        ];

        // Split this segment iteratively by every nearby candidate; pieces stay
        // ordered along the segment.
        let mut pieces: Vec<([f64; 2], [f64; 2])> = vec![(start, end)];
        for &cand in candidates {
            if cand[0] < lo[0] || cand[0] > hi[0] || cand[1] < lo[1] || cand[1] > hi[1] {
                continue;
            }
            let mut next = Vec::with_capacity(pieces.len() + 1);
            // A candidate that misses any piece is remembered once, not once per
            // piece: the second pass only asks whether it is there at all.
            let mut missed_a_piece = false;
            for piece in pieces {
                let on_line = (dist(piece.0, cand) + dist(cand, piece.1) - dist(piece.0, piece.1))
                    .abs()
                    < tolerance;
                if on_line && dist(cand, piece.0) >= tolerance && dist(cand, piece.1) >= tolerance {
                    next.push((piece.0, cand));
                    next.push((cand, piece.1));
                    split_coords.push(cand);
                } else {
                    next.push(piece);
                    missed_a_piece = true;
                }
            }
            if missed_a_piece {
                ignored.push(cand);
            }
            pieces = next;
        }

        for piece in &pieces[..pieces.len() - 1] {
            current.push(piece.1);
            lines.push(std::mem::replace(&mut current, vec![piece.1]));
        }
        current.push(pieces[pieces.len() - 1].1);
    }
    lines.push(current);

    // Split further at interior vertices that coincide with candidates that
    // failed to split a segment (T-junctions at existing vertices).
    let mut final_lines = Vec::new();
    for line in lines {
        let split_idxs: Vec<usize> = (1..line.len().saturating_sub(1))
            .filter(|&i| ignored.iter().any(|&c| dist(c, line[i]) < tolerance))
            .collect();
        let mut curr = line;
        for &i in split_idxs.iter().rev() {
            split_coords.push(curr[i]);
            let second = curr[i..].to_vec();
            curr.truncate(i + 1);
            final_lines.push(second);
        }
        final_lines.push(curr);
    }

    (final_lines, split_coords)
}

#[cfg(all(test, not(feature = "new-geometry")))]
fn line_string_intersection_2d(
    line_strings: &[LineString2D<f64>],
    tolerance: f64,
) -> OverlayResult {
    let lss_per_feature: Vec<Vec<LineString2D<f64>>> =
        line_strings.iter().map(|ls| vec![ls.clone()]).collect();
    let entries: Vec<AabbEntry> = line_strings
        .iter()
        .enumerate()
        .map(|(i, ls)| AabbEntry {
            feature_idx: i,
            ls_local_idx: 0,
            aabb: aabb_to_rstar(aabb_of_line_string(ls)),
        })
        .collect();
    overlay_entries(entries, &lss_per_feature, tolerance)
}

#[cfg(all(test, not(feature = "new-geometry")))]
mod tests {
    use super::*;

    #[test]
    fn test_overlay() {
        let line_string1 = LineString2D::new(vec![
            Coordinate2D::new_(0.0, 0.0),
            Coordinate2D::new_(5.0, 5.0),
        ]);
        let line_string2 = LineString2D::new(vec![
            Coordinate2D::new_(0.0, 5.0),
            Coordinate2D::new_(5.0, 0.0),
        ]);

        let overlay_result = line_string_intersection_2d(&[line_string1, line_string2], 0.1);

        let OverlayResult {
            line_strings_with_metadata,
            split_coords,
        } = overlay_result;
        assert_eq!(line_strings_with_metadata.len(), 4);
        assert_eq!(split_coords.len(), 1);
        let split_coord = &split_coords[0];
        assert!((split_coord.x - 2.5).abs() < 1e-6);
        assert!((split_coord.y - 2.5).abs() < 1e-6);
    }

    #[test]
    fn test_overlay_duplicate_lines() {
        let line_string1 = LineString2D::new(vec![
            Coordinate2D::new_(0.0, 0.0),
            Coordinate2D::new_(4.0, 4.0),
        ]);

        let line_string2 = LineString2D::new(vec![
            Coordinate2D::new_(1.0, 1.0),
            Coordinate2D::new_(4.0, 4.0),
        ]);

        let line_string3 = LineString2D::new(vec![
            Coordinate2D::new_(2.0, 2.0),
            Coordinate2D::new_(3.0, 3.0),
        ]);
        let overlay_result =
            line_string_intersection_2d(&[line_string1, line_string2, line_string3], 0.1);
        let OverlayResult {
            line_strings_with_metadata,
            split_coords,
        } = overlay_result;

        assert_eq!(line_strings_with_metadata.len(), 4);
        let mut overlay_counts = line_strings_with_metadata
            .iter()
            .map(|ls| ls.source_feature_idxs.len())
            .collect::<Vec<_>>();
        overlay_counts.sort();
        assert_eq!(overlay_counts, vec![1, 2, 2, 3]);
        assert_eq!(split_coords.len(), 3);
    }

    #[test]
    fn test_overlay_two_squares() {
        let line_string1 = LineString2D::new(vec![
            Coordinate2D::new_(4.0, 4.0),
            Coordinate2D::new_(4.0, 0.0),
            Coordinate2D::new_(0.0, 0.0),
            Coordinate2D::new_(0.0, 4.0),
            Coordinate2D::new_(4.0, 4.0),
        ]);
        let line_string2 = LineString2D::new(vec![
            Coordinate2D::new_(6.0, 6.0),
            Coordinate2D::new_(2.0, 6.0),
            Coordinate2D::new_(2.0, 2.0),
            Coordinate2D::new_(6.0, 2.0),
            Coordinate2D::new_(6.0, 6.0),
        ]);
        let overlay_result = line_string_intersection_2d(&[line_string1, line_string2], 0.1);
        let OverlayResult {
            line_strings_with_metadata,
            split_coords: _,
        } = overlay_result;

        assert_eq!(line_strings_with_metadata.len(), 6);
    }

    #[test]
    fn test_overlay_k_like_lines() {
        let line_string1 = LineString2D::new(vec![
            Coordinate2D::new_(0.0, 0.0),
            Coordinate2D::new_(0.0, 4.0),
        ]);
        let line_string2 = LineString2D::new(vec![
            Coordinate2D::new_(2.0, 4.0),
            Coordinate2D::new_(0.0, 2.0),
            Coordinate2D::new_(2.0, 0.0),
        ]);

        let overlay_result = line_string_intersection_2d(&[line_string1, line_string2], 0.1);

        let OverlayResult {
            line_strings_with_metadata,
            split_coords,
        } = overlay_result;
        assert_eq!(line_strings_with_metadata.len(), 4);
        assert!(line_strings_with_metadata
            .iter()
            .all(|ls| ls.source_feature_idxs.len() == 1));
        assert_eq!(split_coords.len(), 1);
    }

    #[test]
    fn test_overlay_adjacent_triangles() {
        let line_string1 = LineString2D::new(vec![
            Coordinate2D::new_(1.0, 0.0),
            Coordinate2D::new_(0.0, 0.0),
            Coordinate2D::new_(0.0, 1.0),
            Coordinate2D::new_(1.0, 0.0),
        ]);
        let line_string2 = LineString2D::new(vec![
            Coordinate2D::new_(1.0, 0.0),
            Coordinate2D::new_(1.0, 1.0),
            Coordinate2D::new_(0.0, 1.0),
            Coordinate2D::new_(1.0, 0.0),
        ]);
        let line_string3 = LineString2D::new(vec![
            Coordinate2D::new_(1.0, 0.0),
            Coordinate2D::new_(1.0, 1.0),
            Coordinate2D::new_(2.0, 1.0),
            Coordinate2D::new_(1.0, 0.0),
        ]);

        let overlay_result =
            line_string_intersection_2d(&[line_string1, line_string2, line_string3], 0.1);

        let OverlayResult {
            line_strings_with_metadata,
            split_coords,
        } = overlay_result;
        assert_eq!(line_strings_with_metadata.len(), 5);
        let mut overlap_counts = line_strings_with_metadata
            .iter()
            .map(|ls| ls.source_feature_idxs.len())
            .collect::<Vec<_>>();
        overlap_counts.sort();
        assert_eq!(overlap_counts, vec![1, 1, 1, 2, 2]);
        assert_eq!(split_coords.len(), 2);
    }

    #[test]
    fn test_overlay_sub_tolerance_segments_dropped() {
        // Two collinear lines with a 0.005-long overlap, well below tolerance=0.1.
        // The overlap would have emitted a matched short-segment pair previously.
        let line_string1 = LineString2D::new(vec![
            Coordinate2D::new_(0.0, 0.0),
            Coordinate2D::new_(1.005, 0.0),
        ]);
        let line_string2 = LineString2D::new(vec![
            Coordinate2D::new_(1.0, 0.0),
            Coordinate2D::new_(2.0, 0.0),
        ]);

        let tolerance = 0.1;
        let result = line_string_intersection_2d(&[line_string1, line_string2], tolerance);

        for meta in &result.line_strings_with_metadata {
            let len = line_string_length_2d(&meta.line_string);
            assert!(len >= tolerance, "sub-tolerance segment emitted: len={len}");
        }
        assert!(result
            .line_strings_with_metadata
            .iter()
            .all(|m| m.source_feature_idxs.len() == 1));
    }

    #[test]
    fn test_overlay_same_feature_duplicate_rings_not_double_counted() {
        // Feature 0 has two identical sub-line-strings (like a degenerate MultiPolygon);
        // Feature 1 is a separate collinear line overlapping part of them.
        // Expect: overlap segment has overlay_count=2 (feat 0 + feat 1), not 3.
        let f0_a = LineString2D::new(vec![
            Coordinate2D::new_(0.0, 0.0),
            Coordinate2D::new_(4.0, 0.0),
        ]);
        let f0_b = f0_a.clone();
        let f1 = LineString2D::new(vec![
            Coordinate2D::new_(1.0, 0.0),
            Coordinate2D::new_(3.0, 0.0),
        ]);

        let lss_per_feature = vec![vec![f0_a.clone(), f0_b.clone()], vec![f1.clone()]];
        let entries = vec![
            AabbEntry {
                feature_idx: 0,
                ls_local_idx: 0,
                aabb: aabb_to_rstar(aabb_of_line_string(&f0_a)),
            },
            AabbEntry {
                feature_idx: 0,
                ls_local_idx: 1,
                aabb: aabb_to_rstar(aabb_of_line_string(&f0_b)),
            },
            AabbEntry {
                feature_idx: 1,
                ls_local_idx: 0,
                aabb: aabb_to_rstar(aabb_of_line_string(&f1)),
            },
        ];

        let result = overlay_entries(entries, &lss_per_feature, 0.01);

        for meta in &result.line_strings_with_metadata {
            let feats = &meta.source_feature_idxs;
            let unique: std::collections::HashSet<_> = feats.iter().copied().collect();
            assert_eq!(
                feats.len(),
                unique.len(),
                "source_feature_idxs contains duplicates: {feats:?}"
            );
        }

        let overlapping: Vec<_> = result
            .line_strings_with_metadata
            .iter()
            .filter(|m| m.source_feature_idxs.len() >= 2)
            .collect();
        assert!(!overlapping.is_empty(), "expected an overlap segment");
        for m in &overlapping {
            assert_eq!(m.source_feature_idxs.len(), 2);
        }
    }

    #[test]
    fn test_overlay_multiple_same_feature_candidates_dedupe() {
        // Feature 1 contributes two identical sub-line-strings that both match the rep segment
        // from feature 0 — neither is the rep, but both are the same feature. The old
        // "candidate-same-as-rep" check wouldn't catch this; the feature-set dedup must.
        let f0 = LineString2D::new(vec![
            Coordinate2D::new_(0.0, 0.0),
            Coordinate2D::new_(4.0, 0.0),
        ]);
        let f1_a = LineString2D::new(vec![
            Coordinate2D::new_(1.0, 0.0),
            Coordinate2D::new_(3.0, 0.0),
        ]);
        let f1_b = f1_a.clone();

        let lss_per_feature = vec![vec![f0.clone()], vec![f1_a.clone(), f1_b.clone()]];
        let entries = vec![
            AabbEntry {
                feature_idx: 0,
                ls_local_idx: 0,
                aabb: aabb_to_rstar(aabb_of_line_string(&f0)),
            },
            AabbEntry {
                feature_idx: 1,
                ls_local_idx: 0,
                aabb: aabb_to_rstar(aabb_of_line_string(&f1_a)),
            },
            AabbEntry {
                feature_idx: 1,
                ls_local_idx: 1,
                aabb: aabb_to_rstar(aabb_of_line_string(&f1_b)),
            },
        ];

        let result = overlay_entries(entries, &lss_per_feature, 0.01);

        let overlap: Vec<_> = result
            .line_strings_with_metadata
            .iter()
            .filter(|m| m.source_feature_idxs.len() >= 2)
            .collect();
        assert!(!overlap.is_empty(), "expected an overlap segment");
        for m in &overlap {
            let feats: std::collections::HashSet<usize> =
                m.source_feature_idxs.iter().copied().collect();
            assert_eq!(
                feats.len(),
                m.source_feature_idxs.len(),
                "duplicate feature_idx in source_feature_idxs: {:?}",
                m.source_feature_idxs
            );
            assert_eq!(m.source_feature_idxs.len(), 2);
        }
    }

    #[test]
    fn test_process_group_two_crossing_lines() {
        let dir =
            engine_cache_dir(uuid::Uuid::nil()).join(format!("test-lol-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let group_dir = dir.join("group_000000");
        std::fs::create_dir_all(&group_dir).unwrap();

        let f1 = {
            let ls = LineString2D::new(vec![
                Coordinate2D::new_(0.0, 0.0),
                Coordinate2D::new_(5.0, 5.0),
            ]);
            let geom =
                Geometry::with_value(GeometryValue::FlowGeometry2D(Geometry2D::LineString(ls)));
            Feature::new_with_attributes_and_geometry(Attributes::new(), geom)
        };
        let f2 = {
            let ls = LineString2D::new(vec![
                Coordinate2D::new_(0.0, 5.0),
                Coordinate2D::new_(5.0, 0.0),
            ]);
            let geom =
                Geometry::with_value(GeometryValue::FlowGeometry2D(Geometry2D::LineString(ls)));
            Feature::new_with_attributes_and_geometry(Attributes::new(), geom)
        };

        {
            let mut w = BufWriter::new(File::create(group_dir.join("aabbs.jsonl")).unwrap());
            let a1: Vec<[f64; 4]> = vec![[0.0, 0.0, 5.0, 5.0]];
            let a2: Vec<[f64; 4]> = vec![[0.0, 0.0, 5.0, 5.0]];
            writeln!(w, "{}", serde_json::to_string(&a1).unwrap()).unwrap();
            writeln!(w, "{}", serde_json::to_string(&a2).unwrap()).unwrap();
            w.flush().unwrap();
        }
        {
            let mut w = BufWriter::new(File::create(group_dir.join("features.jsonl")).unwrap());
            writeln!(w, "{}", serde_json::to_string(&f1).unwrap()).unwrap();
            writeln!(w, "{}", serde_json::to_string(&f2).unwrap()).unwrap();
            w.flush().unwrap();
        }

        let mut line_buf: Vec<u8> = Vec::new();
        let mut point_buf: Vec<u8> = Vec::new();
        let (lc, pc) = process_group(
            &group_dir,
            0.01,
            None,
            DEFAULT_OVERLAP_COUNT_ATTRIBUTE,
            Some("overlaidLists"),
            &mut line_buf,
            &mut point_buf,
        )
        .unwrap();
        assert_eq!(lc, 4);
        assert_eq!(pc, 1);

        let _ = std::fs::remove_dir_all(&dir);
    }
}

#[cfg(all(test, feature = "new-geometry"))]
mod tests {
    use pretty_assertions::assert_eq;
    use reearth_flow_geometry::polygon::Polygon2D;

    use super::*;

    fn line(coords: Vec<[f64; 2]>) -> Geometry {
        Geometry::Euclidean2D(Euclidean2DGeometry::LineString(LineString2D::from_coords(
            CoordinateFrame::Euclidean,
            coords,
        )))
    }

    #[test]
    fn a_group_admits_only_the_frame_its_first_feature_fixes() {
        let mut overlayer = LineOnLineOverlayer {
            group_by: None,
            tolerance: 0.0,
            output_attribute: DEFAULT_OVERLAP_COUNT_ATTRIBUTE.to_string(),
            list_attribute: None,
            group_map: HashMap::new(),
            group_frame: HashMap::new(),
            group_count: 0,
            temp_dir: None,
            buffer: HashMap::new(),
            buffer_bytes: 0,
            executor_id: None,
        };
        let euclidean = CoordinateFrame::Euclidean;
        let crs = CoordinateFrame::Crs(reearth_flow_geometry::coordinate::EpsgCode::new(6677));

        assert!(overlayer.admit_frame(0, &euclidean));
        assert!(overlayer.admit_frame(0, &euclidean));
        assert!(!overlayer.admit_frame(0, &crs));
        // A different group is free to fix a different frame.
        assert!(overlayer.admit_frame(1, &crs));
    }

    #[test]
    fn intake_rejects_a_feature_whose_members_are_in_different_frames() {
        let in_euclidean =
            LineString2D::from_coords(CoordinateFrame::Euclidean, [[0.0, 0.0], [1.0, 1.0]]);
        let in_crs = LineString2D::from_coords(
            CoordinateFrame::Crs(reearth_flow_geometry::coordinate::EpsgCode::new(6677)),
            [[0.0, 0.0], [1.0, 1.0]],
        );
        let geometry = Geometry::Euclidean2D(Euclidean2DGeometry::Collection(
            reearth_flow_geometry::collection::Collection2D::new([
                Euclidean2DGeometry::LineString(in_euclidean),
                Euclidean2DGeometry::LineString(in_crs),
            ]),
        ));
        assert!(intake(&geometry).is_none());
    }

    fn overlay_lines(lines: Vec<Vec<[f64; 2]>>, tolerance: f64) -> OverlayResult {
        let lss_per_feature: Vec<Vec<Polyline>> = lines
            .into_iter()
            .map(|coords| {
                vec![Polyline {
                    frame: CoordinateFrame::Euclidean,
                    coords,
                }]
            })
            .collect();
        let entries: Vec<AabbEntry> = lss_per_feature
            .iter()
            .enumerate()
            .map(|(i, pls)| AabbEntry {
                feature_idx: i,
                ls_local_idx: 0,
                aabb: aabb_to_rstar(polyline_bbox(&pls[0].coords)),
            })
            .collect();
        overlay_entries(entries, &lss_per_feature, tolerance)
    }

    #[test]
    fn crossing_lines_split_into_four_segments_and_one_point() {
        let result = overlay_lines(
            vec![vec![[0.0, 0.0], [5.0, 5.0]], vec![[0.0, 5.0], [5.0, 0.0]]],
            0.1,
        );
        assert_eq!(result.line_strings_with_metadata.len(), 4);
        assert_eq!(result.split_coords.len(), 1);
        let point = &result.split_coords[0];
        assert!((point.coord[0] - 2.5).abs() < 1e-6);
        assert!((point.coord[1] - 2.5).abs() < 1e-6);
    }

    #[test]
    fn collinear_overlaps_count_each_source_once() {
        let result = overlay_lines(
            vec![
                vec![[0.0, 0.0], [4.0, 4.0]],
                vec![[1.0, 1.0], [4.0, 4.0]],
                vec![[2.0, 2.0], [3.0, 3.0]],
            ],
            0.1,
        );
        assert_eq!(result.line_strings_with_metadata.len(), 4);
        let mut overlay_counts = result
            .line_strings_with_metadata
            .iter()
            .map(|ls| ls.source_feature_idxs.len())
            .collect::<Vec<_>>();
        overlay_counts.sort();
        assert_eq!(overlay_counts, vec![1, 2, 2, 3]);
        assert_eq!(result.split_coords.len(), 3);
    }

    #[test]
    fn intake_rejects_an_elevated_line_string() {
        let elevated = LineString2D::from_coords_at_elevation(
            CoordinateFrame::Euclidean,
            [[0.0, 0.0], [1.0, 1.0]],
            5.0,
        );
        let geometry = Geometry::Euclidean2D(Euclidean2DGeometry::LineString(elevated));
        assert!(intake(&geometry).is_none());
    }

    #[test]
    fn intake_rejects_an_elevated_polygon_exterior() {
        let geometry = Geometry::Euclidean2D(Euclidean2DGeometry::Polygon(Box::new(
            Polygon2D::from_rings_at_elevation(
                CoordinateFrame::Euclidean,
                vec![[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 4.0], [0.0, 0.0]],
                Vec::<Vec<[f64; 2]>>::new(),
                5.0,
            ),
        )));
        assert!(intake(&geometry).is_none());
    }

    #[test]
    fn a_polygon_exterior_participates_in_the_overlay() {
        let polygon = Geometry::Euclidean2D(Euclidean2DGeometry::Polygon(Box::new(
            Polygon2D::from_rings(
                CoordinateFrame::Euclidean,
                vec![[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 4.0], [0.0, 0.0]],
                Vec::<Vec<[f64; 2]>>::new(),
            ),
        )));
        let ring = feature_source_lines(&Feature::from(polygon));
        assert_eq!(ring.len(), 1);

        let crossing = feature_source_lines(&Feature::from(line(vec![[-1.0, 2.0], [5.0, 2.0]])));
        let lss_per_feature = vec![ring, crossing];
        let entries: Vec<AabbEntry> = lss_per_feature
            .iter()
            .enumerate()
            .map(|(i, pls)| AabbEntry {
                feature_idx: i,
                ls_local_idx: 0,
                aabb: aabb_to_rstar(polyline_bbox(&pls[0].coords)),
            })
            .collect();
        let result = overlay_entries(entries, &lss_per_feature, 0.1);

        // The crossing line pierces the ring's left and right edges.
        assert_eq!(result.split_coords.len(), 2);
    }

    #[test]
    fn process_group_emits_split_lines_and_intersection_points() {
        let dir =
            engine_cache_dir(uuid::Uuid::nil()).join(format!("test-lol-{}", uuid::Uuid::new_v4()));
        let group_dir = dir.join("group_000000");
        std::fs::create_dir_all(&group_dir).unwrap();

        let features = [
            Feature::from(line(vec![[0.0, 0.0], [5.0, 5.0]])),
            Feature::from(line(vec![[0.0, 5.0], [5.0, 0.0]])),
        ];

        {
            let mut w = BufWriter::new(File::create(group_dir.join("aabbs.jsonl")).unwrap());
            for f in &features {
                let (aabbs, _) = intake(f.geometry.as_ref()).unwrap();
                writeln!(w, "{}", serde_json::to_string(&aabbs).unwrap()).unwrap();
            }
            w.flush().unwrap();
        }
        {
            let mut w = BufWriter::new(File::create(group_dir.join("features.jsonl")).unwrap());
            for f in &features {
                writeln!(w, "{}", serde_json::to_string(f).unwrap()).unwrap();
            }
            w.flush().unwrap();
        }

        let mut line_buf: Vec<u8> = Vec::new();
        let mut point_buf: Vec<u8> = Vec::new();
        let (lc, pc) = process_group(
            &group_dir,
            0.01,
            None,
            DEFAULT_OVERLAP_COUNT_ATTRIBUTE,
            Some("overlaidLists"),
            &mut line_buf,
            &mut point_buf,
        )
        .unwrap();
        assert_eq!(lc, 4);
        assert_eq!(pc, 1);

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Writes one group to disk and overlays it, returning the group directory
    /// alongside the result of the overlay.
    fn run_group(
        features: &[Feature],
        group_by: Option<&[Attribute]>,
    ) -> (PathBuf, Result<(usize, usize), BoxedError>) {
        let dir =
            engine_cache_dir(uuid::Uuid::nil()).join(format!("test-lol-{}", uuid::Uuid::new_v4()));
        let group_dir = dir.join("group_000000");
        std::fs::create_dir_all(&group_dir).unwrap();
        {
            let mut w = BufWriter::new(File::create(group_dir.join("aabbs.jsonl")).unwrap());
            for f in features {
                let (aabbs, _) = intake(f.geometry.as_ref()).unwrap();
                writeln!(w, "{}", serde_json::to_string(&aabbs).unwrap()).unwrap();
            }
            w.flush().unwrap();
        }
        {
            let mut w = BufWriter::new(File::create(group_dir.join("features.jsonl")).unwrap());
            for f in features {
                writeln!(w, "{}", serde_json::to_string(f).unwrap()).unwrap();
            }
            w.flush().unwrap();
        }
        let mut line_buf: Vec<u8> = Vec::new();
        let mut point_buf: Vec<u8> = Vec::new();
        let result = process_group(
            &group_dir,
            0.01,
            group_by,
            DEFAULT_OVERLAP_COUNT_ATTRIBUTE,
            None,
            &mut line_buf,
            &mut point_buf,
        );
        (dir, result)
    }

    fn crossing_pair() -> [Feature; 2] {
        [
            Feature::from(line(vec![[0.0, 0.0], [5.0, 5.0]])),
            Feature::from(line(vec![[0.0, 5.0], [5.0, 0.0]])),
        ]
    }

    #[test]
    fn a_line_missing_a_grouping_attribute_does_not_fail_the_run() {
        // Neither line carries `surfaceId`. `process` admits such a feature to a
        // group without it, so `finish` must carry the absence forward rather
        // than failing: the two halves used to disagree and this returned Err,
        // taking the whole run down with it.
        let group_by = [Attribute::new("surfaceId")];
        let (dir, result) = run_group(&crossing_pair(), Some(&group_by));
        let (lc, pc) = result.expect("an absent grouping attribute is not a failure");
        assert_eq!((lc, pc), (4, 1));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_grouping_attribute_the_line_carries_is_copied_onto_its_segments() {
        let group_by = [Attribute::new("surfaceId")];
        let features: Vec<Feature> = crossing_pair()
            .into_iter()
            .map(|mut f| {
                f.attributes_mut().insert(
                    Attribute::new("surfaceId"),
                    AttributeValue::String("s1".to_string()),
                );
                f
            })
            .collect();
        let (dir, result) = run_group(&features, Some(&group_by));
        let (lc, _) = result.unwrap();
        assert_eq!(lc, 4);

        let _ = std::fs::remove_dir_all(&dir);
    }
}
