use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader, BufWriter, Read as _, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
    sync::Arc,
};

use indexmap::IndexMap;
use once_cell::sync::Lazy;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use reearth_flow_runtime::{
    cache::executor_cache_subdir,
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, FEATURES_PORT, REJECTED_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, Feature};
use rstar::{RTree, AABB};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[cfg(not(feature = "new-geometry"))]
use nusamai_projection::crs::EpsgCode;
#[cfg(not(feature = "new-geometry"))]
use reearth_flow_geometry::{
    algorithm::{area2d::Area2D, bool_ops::BooleanOps},
    types::{geometry::Geometry2D, multi_polygon::MultiPolygon2D, polygon::Polygon2D},
};
#[cfg(not(feature = "new-geometry"))]
use reearth_flow_types::{Geometry, GeometryValue};

#[cfg(feature = "new-geometry")]
use reearth_flow_geometry::{
    collection::Collection2D,
    coordinate::CoordinateFrame,
    line_string::LineString2D,
    ops::{Aabb, BoundingBox},
    overlay::{overlay_2d, snap_areal_operands_2d, OverlayOp},
    polygon::Polygon2D,
    predicates::view::{flatten_2d, Leaf2D},
    Euclidean2DGeometry, Geometry,
};

use super::errors::GeometryProcessorError;
use crate::ACCUMULATOR_BUFFER_BYTE_THRESHOLD;

static OVERLAPS_PORT: Lazy<Port> = Lazy::new(|| Port::new("overlaps"));
static REMNANTS_PORT: Lazy<Port> = Lazy::new(|| Port::new("remnants"));

/// The attribute the overlap count lands in when the parameter is omitted.
const DEFAULT_OVERLAP_COUNT_ATTRIBUTE: &str = "overlayCount";

#[derive(Debug, Clone, Default)]
pub(super) struct AreaOnAreaOverlayerFactory;

impl ProcessorFactory for AreaOnAreaOverlayerFactory {
    fn name(&self) -> &str {
        "Area On Area Overlayer"
    }

    fn description(&self) -> &str {
        "Subdivides overlapping areas into non-overlapping pieces and records how many input \
         features cover each piece. Inputs must be flat 2D geometries sharing one coordinate \
         frame; place a Two Dimension Forcer or a Coordinate Frame Reprojector upstream to \
         flatten or unify them."
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(AreaOnAreaOverlayerParam))
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
        vec![
            OVERLAPS_PORT.clone(),
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
            output_attribute: param
                .output_attribute
                .unwrap_or_else(|| DEFAULT_OVERLAP_COUNT_ATTRIBUTE.to_string()),
            list_attribute: param.list_attribute,
            attribute_accumulation: param.attribute_accumulation,
            tolerance: param.tolerance.unwrap_or(0.0),
            group_map: HashMap::new(),
            group_crs: HashMap::new(),
            group_count: 0,
            temp_dir: None,
            buffer: HashMap::new(),
            buffer_bytes: 0,
            executor_id: None,
        };

        Ok(Box::new(process))
    }
}

/// # Area On Area Overlayer Parameters
/// Sets which features are overlaid together, how closely their vertices must
/// line up, and what the resulting pieces record about the features they came
/// from.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct AreaOnAreaOverlayerParam {
    /// # Group By Attributes
    /// Attributes whose values decide which features are overlaid against each
    /// other — only features matching on all of them are compared. When
    /// omitted, every feature is overlaid against every other.
    group_by: Option<Vec<Attribute>>,

    /// # Tolerance
    /// Distance below which two vertices are treated as the same point, in the
    /// unit of the input's coordinate frame. Boundaries that were meant to
    /// coincide but miss by less than this are pulled together before the
    /// overlay, and overlaps smaller than its square are discarded as slivers.
    /// Defaults to zero, which snaps nothing.
    tolerance: Option<f64>,

    /// # Attribute Accumulation
    /// Which attributes the resulting pieces keep.
    #[serde(default)]
    attribute_accumulation: AttributeAccumulation,

    /// # Overlap Count Attribute
    /// Attribute that receives the number of input features covering the
    /// piece — two or more on `overlaps`, always one on `remnants`.
    /// Defaults to `overlayCount`.
    output_attribute: Option<String>,

    /// # List Attribute
    /// Attribute that receives one entry per covering feature, each holding
    /// that feature's own attributes. When omitted, no list is written.
    list_attribute: Option<String>,
}

/// Per-group CRS bookkeeping: the EPSG folded across the group's inputs and
/// carried onto its outputs.
#[cfg(not(feature = "new-geometry"))]
type GroupCrs = GroupEpsg;

/// Per-group CRS bookkeeping: the coordinate frame every member of the group
/// must share.
#[cfg(feature = "new-geometry")]
type GroupCrs = CoordinateFrame;

struct AreaOnAreaOverlayer {
    group_by: Option<Vec<Attribute>>,
    output_attribute: String,
    list_attribute: Option<String>,
    attribute_accumulation: AttributeAccumulation,
    tolerance: f64,
    // Disk-backed state
    group_map: HashMap<AttributeValue, usize>,
    /// Per-group CRS state keeping the overlay inputs and outputs in one
    /// coordinate reference.
    group_crs: HashMap<usize, GroupCrs>,
    group_count: usize,
    temp_dir: Option<PathBuf>,
    // In-memory buffer: group_idx -> Vec<(aabb_json, feature_json)>
    buffer: HashMap<usize, Vec<(String, String)>>,
    buffer_bytes: usize,
    /// Executor ID for cache isolation, set on first process() call
    executor_id: Option<uuid::Uuid>,
}

/// Tracks the CRS of the features accumulated into a single overlay group.
#[cfg(not(feature = "new-geometry"))]
#[derive(Clone, Copy)]
enum GroupEpsg {
    /// The single EPSG known so far (`None` while only EPSG-less features
    /// have been seen).
    Uniform(Option<EpsgCode>),
    /// Two differing known EPSGs were seen; no CRS is carried over.
    Mixed,
}

#[cfg(not(feature = "new-geometry"))]
impl GroupEpsg {
    /// Fold another feature's EPSG into the group's running state.
    fn observe(&mut self, epsg: Option<EpsgCode>) {
        if let GroupEpsg::Uniform(existing) = self {
            match (*existing, epsg) {
                (Some(a), Some(b)) if a != b => *self = GroupEpsg::Mixed,
                (None, Some(_)) => *existing = epsg,
                _ => {}
            }
        }
    }

    /// The EPSG to stamp on this group's outputs, or `None` when mixed.
    fn resolve(self) -> Option<EpsgCode> {
        match self {
            GroupEpsg::Uniform(epsg) => epsg,
            GroupEpsg::Mixed => None,
        }
    }
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
            list_attribute: self.list_attribute.clone(),
            attribute_accumulation: self.attribute_accumulation.clone(),
            tolerance: self.tolerance,
            group_map: HashMap::new(),
            group_crs: HashMap::new(),
            group_count: 0,
            temp_dir: None,
            buffer: HashMap::new(),
            buffer_bytes: 0,
            executor_id: self.executor_id,
        }
    }
}

/// # Attribute Accumulation
/// Which attributes a resulting piece keeps.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub enum AttributeAccumulation {
    /// # Use Attributes From One Feature
    /// Keeps the attributes of a single covering feature and discards the rest.
    #[default]
    UseOneFeature,
    /// # Drop Incoming Attributes
    /// Keeps no incoming attribute, so a piece carries only its overlap count
    /// and list attribute. The grouping attributes are dropped too.
    DropAttributes,
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

    /// Fold `epsg` into the group's CRS state; every accepted feature joins
    /// its group.
    #[cfg(not(feature = "new-geometry"))]
    fn admit_crs(&mut self, group_idx: usize, epsg: Option<EpsgCode>) -> bool {
        self.group_crs
            .entry(group_idx)
            .or_insert(GroupEpsg::Uniform(epsg))
            .observe(epsg);
        true
    }

    /// Whether `frame` matches the group's coordinate frame, which the first
    /// feature of the group fixes. Overlay operands must share one frame.
    #[cfg(feature = "new-geometry")]
    fn admit_crs(&mut self, group_idx: usize, frame: &CoordinateFrame) -> bool {
        use std::collections::hash_map::Entry;
        match self.group_crs.entry(group_idx) {
            Entry::Occupied(entry) => entry.get() == frame,
            Entry::Vacant(entry) => {
                entry.insert(frame.clone());
                true
            }
        }
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
        let Some((aabb, crs)) = intake(feature.geometry.as_ref()) else {
            fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
            return Ok(());
        };

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

        if !self.admit_crs(group_idx, crs) {
            fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
            return Ok(());
        }

        let feature_json = serde_json::to_string(&ctx.feature)?;
        self.append_to_group(group_idx, &aabb, &feature_json)?;
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
        let overlaps_path = temp_dir.join(format!("aoa-overlaps-{output_id}.jsonl.zst"));
        let remnants_path = temp_dir.join(format!("aoa-remnants-{output_id}.jsonl.zst"));
        let mut overlaps_writer =
            BufWriter::new(zstd::Encoder::new(File::create(&overlaps_path)?, 1)?);
        let mut remnants_writer =
            BufWriter::new(zstd::Encoder::new(File::create(&remnants_path)?, 1)?);
        let mut overlaps_count: usize = 0;
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
            #[cfg(not(feature = "new-geometry"))]
            let shaper = OutputShaper {
                epsg: self
                    .group_crs
                    .get(&group_idx)
                    .copied()
                    .and_then(GroupEpsg::resolve),
            };
            #[cfg(feature = "new-geometry")]
            let shaper = OutputShaper;
            let (oc, rc) = from_midpolygons_disk(
                &midpolygons_path,
                &disk_feats,
                &self.output_attribute,
                &self.list_attribute,
                &self.attribute_accumulation,
                shaper,
                &mut overlaps_writer,
                &mut remnants_writer,
            )?;
            overlaps_count += oc;
            remnants_count += rc;
        }

        overlaps_writer
            .into_inner()
            .map_err(|e| e.into_error())?
            .finish()?;
        remnants_writer
            .into_inner()
            .map_err(|e| e.into_error())?
            .finish()?;

        let context = ctx.as_context();

        if overlaps_count > 0 {
            fw.send_file(overlaps_path, OVERLAPS_PORT.clone(), context.clone());
        }
        if remnants_count > 0 {
            fw.send_file(remnants_path, REMNANTS_PORT.clone(), context);
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "Area On Area Overlayer"
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
}

/// Polygon that is created in the middle of the overlay process.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct MiddlePolygon {
    polygon: WorkingArea,
    parents: Vec<usize>,
}

/// Type of the subpolygon and its parents.
enum MiddlePolygonType {
    None,
    Overlap(Vec<usize>),
    Remnant(usize),
}

impl MiddlePolygon {
    fn get_type(&self) -> MiddlePolygonType {
        match self.parents.len() {
            0 => MiddlePolygonType::None,
            1 => MiddlePolygonType::Remnant(self.parents[0]),
            _ => MiddlePolygonType::Overlap(self.parents.clone()),
        }
    }
}

// --- world-specific geometry kernel -----------------------------------------

/// The areal value carried through the overlay subdivision.
#[cfg(not(feature = "new-geometry"))]
type WorkingArea = MultiPolygon2D<f64>;

/// The areal value carried through the overlay subdivision: a feature's own 2D
/// geometry, or the constructed faces cut out of one.
#[cfg(feature = "new-geometry")]
type WorkingArea = Euclidean2DGeometry;

/// Convert Geometry2D to MultiPolygon2D.
/// Handles Polygon, MultiPolygon, and closed LineStrings (converted to Polygon).
#[cfg(not(feature = "new-geometry"))]
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

/// Accept an incoming geometry into the overlay: any 2D geometry. Returns its
/// bounding box and EPSG, or `None` when the feature must be rejected.
#[cfg(not(feature = "new-geometry"))]
fn intake(geometry: &Geometry) -> Option<([f64; 4], Option<EpsgCode>)> {
    if geometry.is_empty() {
        return None;
    }
    let GeometryValue::FlowGeometry2D(geom_2d) = &geometry.value else {
        return None;
    };
    // Compute AABB from geometry (convert closed LineStrings to Polygon first)
    let mp = geom_to_multipolygon(geom_2d);
    let aabb = match mp.bounding_box() {
        Some(rect) => [rect.min().x, rect.min().y, rect.max().x, rect.max().y],
        None => [0.0, 0.0, 0.0, 0.0],
    };
    Some((aabb, geometry.epsg))
}

/// Accept an incoming geometry into the overlay: a planar areal geometry
/// (polygons, meshes, or closed line strings) whose leaves share one
/// coordinate frame. Returns its bounding box and frame, or `None` when the
/// feature must be rejected.
///
/// The overlay reasons about the plane alone, so a leaf placed at an elevation
/// is refused rather than silently overlaid with one at a different height.
#[cfg(feature = "new-geometry")]
fn intake(geometry: &Geometry) -> Option<([f64; 4], &CoordinateFrame)> {
    let Geometry::Euclidean2D(geom_2d) = geometry else {
        return None;
    };
    let mut leaves = Vec::new();
    flatten_2d(geom_2d, &mut leaves);
    let frame = leaves.first()?.frame();
    for leaf in &leaves {
        if leaf.frame() != frame || leaf_elevation(leaf).is_some() {
            return None;
        }
        match leaf {
            Leaf2D::Polygon(_) | Leaf2D::PolygonMesh(_) | Leaf2D::TriangularMesh(_) => {}
            Leaf2D::Line(line) if is_closed_ring(line) => {}
            _ => return None,
        }
    }
    let Ok(Aabb::D2 { min, max }) = geom_2d.bounding_box() else {
        return None;
    };
    Some(([min[0], min[1], max[0], max[1]], frame))
}

/// Whether the line string traces a closed ring that can enclose area.
#[cfg(feature = "new-geometry")]
fn is_closed_ring(line: &LineString2D) -> bool {
    let coords = line.coords();
    coords.len() >= 4 && coords.first() == coords.last()
}

/// The stored feature `i`'s geometry as a working area, or `None` when it is
/// not a 2D geometry.
#[cfg(not(feature = "new-geometry"))]
fn read_working_area(disk_feats: &DiskBackedFeatures, i: usize) -> Option<WorkingArea> {
    let geometry = disk_feats.read_feature(i).geometry;
    match &geometry.value {
        GeometryValue::FlowGeometry2D(geom_2d) => Some(geom_to_multipolygon(geom_2d)),
        _ => None,
    }
}

/// The stored feature `i`'s geometry as a working area, or `None` when it is
/// not a 2D geometry.
#[cfg(feature = "new-geometry")]
fn read_working_area(disk_feats: &DiskBackedFeatures, i: usize) -> Option<WorkingArea> {
    let geometry = disk_feats.read_feature(i).geometry;
    let Geometry::Euclidean2D(geom_2d) = geometry.as_ref() else {
        return None;
    };
    Some(normalize_area(geom_2d))
}

/// The geometry with closed line strings replaced by the polygon faces they
/// trace; every other member is kept verbatim.
#[cfg(feature = "new-geometry")]
fn normalize_area(geom: &Euclidean2DGeometry) -> Euclidean2DGeometry {
    match geom {
        Euclidean2DGeometry::LineString(line) if is_closed_ring(line) => {
            Euclidean2DGeometry::Polygon(Box::new(ring_face(line)))
        }
        Euclidean2DGeometry::Collection(collection) => {
            let members: Vec<_> = collection.members().iter().map(normalize_area).collect();
            let attrs = collection.member_attributes().to_vec();
            Euclidean2DGeometry::Collection(
                Collection2D::with_attributes(members, attrs)
                    .expect("member count is unchanged by normalization"),
            )
        }
        other => other.clone(),
    }
}

/// The polygon face a closed line string traces.
#[cfg(feature = "new-geometry")]
fn ring_face(line: &LineString2D) -> Polygon2D {
    Polygon2D::from_rings(
        line.frame().clone(),
        line.coords().iter().copied(),
        Vec::<Vec<[f64; 2]>>::new(),
    )
}

/// Constructed polygons as one working area.
#[cfg(feature = "new-geometry")]
fn wrap_polygons(mut polygons: Vec<Polygon2D>) -> WorkingArea {
    if polygons.len() == 1 {
        Euclidean2DGeometry::Polygon(Box::new(polygons.remove(0)))
    } else {
        Euclidean2DGeometry::Collection(Collection2D::new(
            polygons
                .into_iter()
                .map(|p| Euclidean2DGeometry::Polygon(Box::new(p))),
        ))
    }
}

/// The group's working areas with near-coincident vertices pulled onto shared
/// positions, so a boundary that two features were meant to share is shared
/// before the subdivision cuts along it. The whole group is snapped in one
/// pass: snapping each pair as it is compared would anchor the shared boundary
/// of three neighbours in three places, and the pieces cut from either side
/// would no longer meet.
#[cfg(feature = "new-geometry")]
fn snap_group(
    areas: Vec<Option<WorkingArea>>,
    tolerance: f64,
) -> Result<Vec<Option<WorkingArea>>, BoxedError> {
    if tolerance <= 0.0 {
        return Ok(areas);
    }
    let operands: Vec<&Euclidean2DGeometry> = areas.iter().flatten().collect();
    if operands.is_empty() {
        return Ok(areas);
    }
    let snapped = snap_areal_operands_2d(&operands, tolerance).map_err(|e| {
        GeometryProcessorError::AreaOnAreaOverlayer(format!("vertex snapping failed: {e}"))
    })?;
    let mut snapped = snapped.into_iter();
    Ok(areas
        .into_iter()
        .map(|area| {
            area.map(|area| {
                let snapped = snapped
                    .next()
                    .expect("snapping returns one result per operand");
                // An operand nothing was close enough to move is left exactly as
                // it was read, so a feature the overlay never touches still
                // comes out with the geometry it arrived with rather than the
                // polygons it dissolves to.
                if snapped.moved {
                    wrap_polygons(snapped.polygons)
                } else {
                    area
                }
            })
        })
        .collect())
}

/// Legacy geometry's overlay has no vertex snapping to drive, so the group is
/// subdivided as it was read.
#[cfg(not(feature = "new-geometry"))]
fn snap_group(
    areas: Vec<Option<WorkingArea>>,
    _tolerance: f64,
) -> Result<Vec<Option<WorkingArea>>, BoxedError> {
    Ok(areas)
}

/// The points of `a` not in `b`.
#[cfg(not(feature = "new-geometry"))]
fn area_difference(a: &WorkingArea, b: &WorkingArea) -> Result<WorkingArea, BoxedError> {
    if b.0.is_empty() {
        return Ok(a.clone());
    }
    Ok(a.difference(b))
}

/// The points of `a` not in `b`, as constructed faces.
#[cfg(feature = "new-geometry")]
fn area_difference(a: &WorkingArea, b: &WorkingArea) -> Result<WorkingArea, BoxedError> {
    let polygons = overlay_2d(a, b, OverlayOp::Difference)
        .map_err(|e| GeometryProcessorError::AreaOnAreaOverlayer(format!("overlay failed: {e}")))?;
    Ok(wrap_polygons(polygons))
}

/// The points in both `a` and `b`.
#[cfg(not(feature = "new-geometry"))]
fn area_intersection(a: &WorkingArea, b: &WorkingArea) -> Result<WorkingArea, BoxedError> {
    if b.0.is_empty() {
        return Ok(MultiPolygon2D::new(vec![]));
    }
    Ok(a.intersection(b))
}

/// The points in both `a` and `b`, as constructed faces.
#[cfg(feature = "new-geometry")]
fn area_intersection(a: &WorkingArea, b: &WorkingArea) -> Result<WorkingArea, BoxedError> {
    let polygons = overlay_2d(a, b, OverlayOp::Intersection)
        .map_err(|e| GeometryProcessorError::AreaOnAreaOverlayer(format!("overlay failed: {e}")))?;
    Ok(wrap_polygons(polygons))
}

/// The total planar area of `a`.
#[cfg(not(feature = "new-geometry"))]
fn area_measure(a: &WorkingArea) -> f64 {
    a.unsigned_area2d()
}

/// The total planar area of `a`'s polygon faces; `a` must be a constructed
/// working area (polygons only).
#[cfg(feature = "new-geometry")]
fn area_measure(a: &WorkingArea) -> f64 {
    let mut leaves = Vec::new();
    flatten_2d(a, &mut leaves);
    leaves
        .iter()
        .filter_map(|leaf| match leaf {
            Leaf2D::Polygon(p) => Some(p.area()),
            _ => None,
        })
        .sum()
}

/// Whether `a` covers no area.
#[cfg(not(feature = "new-geometry"))]
fn area_is_empty(a: &WorkingArea) -> bool {
    a.is_empty()
}

/// Whether `a` covers no area.
#[cfg(feature = "new-geometry")]
fn area_is_empty(a: &WorkingArea) -> bool {
    let mut leaves = Vec::new();
    flatten_2d(a, &mut leaves);
    leaves.is_empty()
}

/// Shapes each subdivision piece into an output feature's geometry.
#[cfg(not(feature = "new-geometry"))]
struct OutputShaper {
    /// The EPSG stamped onto outputs, when the group's inputs agree on one.
    epsg: Option<EpsgCode>,
}

#[cfg(not(feature = "new-geometry"))]
impl OutputShaper {
    /// Install `area` as `feature`'s geometry.
    fn apply(&mut self, feature: &mut Feature, area: WorkingArea) {
        feature.geometry_mut().value = GeometryValue::FlowGeometry2D(area.into());
        feature.geometry_mut().epsg = self.epsg;
    }
}

/// Shapes each subdivision piece into an output feature's geometry.
#[cfg(feature = "new-geometry")]
struct OutputShaper;

#[cfg(feature = "new-geometry")]
impl OutputShaper {
    /// Install `area` as `feature`'s geometry.
    fn apply(&mut self, feature: &mut Feature, area: WorkingArea) {
        *feature.geometry_mut() = Geometry::Euclidean2D(area);
    }
}

/// The elevation a 2D leaf lies at, or `None` when it is planar.
#[cfg(feature = "new-geometry")]
fn leaf_elevation(leaf: &Leaf2D<'_>) -> Option<f64> {
    match leaf {
        Leaf2D::Polygon(p) => p.elevation(),
        Leaf2D::PolygonMesh(m) => m.elevation(),
        Leaf2D::TriangularMesh(m) => m.elevation(),
        Leaf2D::Line(l) => l.elevation(),
        Leaf2D::Point(_) => None,
    }
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

/// Disk-backed subdivision: reads each stored feature's areal geometry, cuts
/// it against its bounding-box neighbours, and writes the resulting
/// MiddlePolygons to a JSONL file instead of collecting in memory.
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

    // Load all working areas upfront to avoid disk I/O inside parallel iteration
    let areas: Vec<Option<WorkingArea>> =
        (0..num).map(|i| read_working_area(disk_feats, i)).collect();
    let areas = snap_group(areas, tolerance)?;

    let results: Vec<Vec<MiddlePolygon>> = (0..num)
        .into_par_iter()
        .map(|i| -> Result<Vec<MiddlePolygon>, BoxedError> {
            let Some(area_i) = &areas[i] else {
                return Ok(Vec::new());
            };

            // Collect overlapping indices once (the iterator is consumed on use)
            let overlapping: Vec<usize> = aabb_index.overlapping_indices(i).collect();

            // cut off the target area by upper areas
            let mut target = area_i.clone();
            for &j in &overlapping {
                if i < j {
                    if let Some(area_j) = &areas[j] {
                        target = area_difference(&target, area_j)?;
                    }
                }
            }

            let mut queue = vec![MiddlePolygon {
                polygon: target,
                parents: vec![i],
            }];

            // divide the target area by lower areas
            for &j in &overlapping {
                if i > j {
                    let Some(area_j) = &areas[j] else {
                        continue;
                    };
                    let mut new_queue = Vec::new();
                    for subpolygon in queue {
                        let intersection = area_intersection(&subpolygon.polygon, area_j)?;

                        let min_area = tolerance * tolerance;
                        let is_significant_intersection = area_measure(&intersection) > min_area;

                        if !area_is_empty(&intersection) && is_significant_intersection {
                            new_queue.push(MiddlePolygon {
                                polygon: intersection,
                                parents: subpolygon.parents.iter().copied().chain([j]).collect(),
                            });
                        }

                        let difference = area_difference(&subpolygon.polygon, area_j)?;
                        if !area_is_empty(&difference) {
                            new_queue.push(MiddlePolygon {
                                polygon: difference,
                                parents: subpolygon.parents.clone(),
                            });
                        }
                    }
                    queue = new_queue;
                }
            }

            Ok(queue)
        })
        .collect::<Result<Vec<_>, _>>()?;

    // Write all results from a single thread
    let mut writer = BufWriter::new(File::create(output_path)?);
    for mp in results.into_iter().flatten() {
        let line = serde_json::to_string(&mp)?;
        writer.write_all(line.as_bytes())?;
        writer.write_all(b"\n")?;
    }
    writer.flush()?;

    Ok(())
}

/// Stream MiddlePolygons from a JSONL file, convert to Features, and write
/// directly to the overlaps/remnants output files without collecting in memory.
/// Returns (overlaps_count, remnants_count).
#[allow(clippy::too_many_arguments)]
fn from_midpolygons_disk<W: Write>(
    midpolygons_path: &Path,
    disk_feats: &DiskBackedFeatures,
    output_attribute: &str,
    list_attribute: &Option<String>,
    attribute_accumulation: &AttributeAccumulation,
    mut shaper: OutputShaper,
    overlaps_writer: &mut W,
    remnants_writer: &mut W,
) -> Result<(usize, usize), BoxedError> {
    let file = File::open(midpolygons_path)?;
    let reader = BufReader::new(file);

    // Cache attributes loaded from disk to avoid re-reading the same feature
    let mut attributes_cache: HashMap<usize, Arc<IndexMap<Attribute, AttributeValue>>> =
        HashMap::new();

    let mut overlaps_count = 0usize;
    let mut remnants_count = 0usize;

    for line in reader.lines() {
        let line = line?;
        if line.is_empty() {
            continue;
        }
        let subpolygon: MiddlePolygon = serde_json::from_str(&line)?;

        match subpolygon.get_type() {
            MiddlePolygonType::None => {}
            MiddlePolygonType::Overlap(parents) => {
                // Ensure all parent attributes are cached
                for &p in &parents {
                    attributes_cache.entry(p).or_insert_with(|| {
                        let feature = disk_feats.read_feature(p);
                        feature.attributes
                    });
                }

                let attrs = match attribute_accumulation {
                    AttributeAccumulation::DropAttributes => IndexMap::new(),
                    AttributeAccumulation::UseOneFeature => {
                        let first_feature = &attributes_cache[&parents[0]];
                        (**first_feature).clone()
                    }
                };
                let mut feature = Feature::new_with_attributes(attrs);

                feature.attributes_mut().insert(
                    Attribute::new(output_attribute),
                    AttributeValue::Number(parents.len().into()),
                );

                if let Some(list_name) = list_attribute {
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

                shaper.apply(&mut feature, subpolygon.polygon);
                serde_json::to_writer(&mut *overlaps_writer, &feature)?;
                overlaps_writer.write_all(b"\n")?;
                overlaps_count += 1;
            }
            MiddlePolygonType::Remnant(parent) => {
                attributes_cache.entry(parent).or_insert_with(|| {
                    let feature = disk_feats.read_feature(parent);
                    feature.attributes
                });

                let attrs = match attribute_accumulation {
                    AttributeAccumulation::DropAttributes => IndexMap::new(),
                    AttributeAccumulation::UseOneFeature => (*attributes_cache[&parent]).clone(),
                };
                let mut feature = Feature::new_with_attributes(attrs);

                feature.attributes_mut().insert(
                    Attribute::new(output_attribute),
                    AttributeValue::Number(1.into()),
                );

                if let Some(list_name) = list_attribute {
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

                shaper.apply(&mut feature, subpolygon.polygon);
                serde_json::to_writer(&mut *remnants_writer, &feature)?;
                remnants_writer.write_all(b"\n")?;
                remnants_count += 1;
            }
        }
    }

    Ok((overlaps_count, remnants_count))
}

#[cfg(all(test, not(feature = "new-geometry")))]
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
    fn group_epsg_uniform_carries_the_shared_code() {
        let mut state = GroupEpsg::Uniform(Some(6675));
        state.observe(Some(6675));
        assert_eq!(state.resolve(), Some(6675));
    }

    #[test]
    fn group_epsg_all_none_stays_none() {
        let mut state = GroupEpsg::Uniform(None);
        state.observe(None);
        assert_eq!(state.resolve(), None);
    }

    #[test]
    fn group_epsg_mixed_codes_fall_back_to_none() {
        let mut state = GroupEpsg::Uniform(Some(6675));
        state.observe(Some(6669));
        assert_eq!(state.resolve(), None);
    }

    #[test]
    fn group_epsg_missing_code_does_not_conflict_with_known_code() {
        let mut state = GroupEpsg::Uniform(Some(6675));
        state.observe(None);
        assert_eq!(state.resolve(), Some(6675));
    }

    #[test]
    fn group_epsg_known_code_upgrades_missing_state() {
        let mut state = GroupEpsg::Uniform(None);
        state.observe(Some(6675));
        assert_eq!(state.resolve(), Some(6675));
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

#[cfg(all(test, feature = "new-geometry"))]
mod tests {
    use pretty_assertions::assert_eq;
    use reearth_flow_geometry::triangular_mesh::TriangularMesh2D;

    use super::*;

    fn square_ring(min: [f64; 2], max: [f64; 2]) -> Vec<[f64; 2]> {
        vec![
            [min[0], min[1]],
            [max[0], min[1]],
            [max[0], max[1]],
            [min[0], max[1]],
            [min[0], min[1]],
        ]
    }

    fn square(min: [f64; 2], max: [f64; 2]) -> Geometry {
        Geometry::Euclidean2D(Euclidean2DGeometry::Polygon(Box::new(
            Polygon2D::from_rings(
                CoordinateFrame::Euclidean,
                square_ring(min, max),
                Vec::<Vec<[f64; 2]>>::new(),
            ),
        )))
    }

    fn square_at(min: [f64; 2], max: [f64; 2], z: f64) -> Geometry {
        Geometry::Euclidean2D(Euclidean2DGeometry::Polygon(Box::new(
            Polygon2D::from_rings_at_elevation(
                CoordinateFrame::Euclidean,
                square_ring(min, max),
                Vec::<Vec<[f64; 2]>>::new(),
                z,
            ),
        )))
    }

    fn triangle(coords: [[f64; 2]; 3]) -> Geometry {
        let ring = vec![coords[0], coords[1], coords[2], coords[0]];
        Geometry::Euclidean2D(Euclidean2DGeometry::Polygon(Box::new(
            Polygon2D::from_rings(
                CoordinateFrame::Euclidean,
                ring,
                Vec::<Vec<[f64; 2]>>::new(),
            ),
        )))
    }

    /// Two triangles forming the square [0,2] x [0,2], as one triangular mesh.
    fn square_mesh() -> Geometry {
        let mesh = TriangularMesh2D::from_parts(
            CoordinateFrame::Euclidean,
            vec![[0.0, 0.0], [2.0, 0.0], [2.0, 2.0], [0.0, 2.0]],
            [0u32, 1, 2, 0, 2, 3],
        )
        .unwrap();
        Geometry::Euclidean2D(Euclidean2DGeometry::TriangularMesh(Box::new(mesh)))
    }

    /// Write `features` as one group on disk, returning its directory and the
    /// scanned features file.
    fn setup_group(features: &[Feature]) -> (PathBuf, DiskBackedFeatures) {
        let dir =
            engine_cache_dir(uuid::Uuid::nil()).join(format!("test-aoa-{}", uuid::Uuid::new_v4()));
        let group_dir = dir.join("group_000000");
        std::fs::create_dir_all(&group_dir).unwrap();
        let features_path = group_dir.join("features.jsonl");
        {
            let mut writer = BufWriter::new(File::create(&features_path).unwrap());
            for f in features {
                serde_json::to_writer(&mut writer, f).unwrap();
                writer.write_all(b"\n").unwrap();
            }
            writer.flush().unwrap();
        }
        let disk_feats = DiskBackedFeatures::scan(&features_path).unwrap();
        (dir, disk_feats)
    }

    fn run_overlay(geometries: Vec<Geometry>, tolerance: f64) -> (PathBuf, Vec<MiddlePolygon>) {
        let features: Vec<Feature> = geometries.into_iter().map(Feature::from).collect();
        let aabbs: Vec<[f64; 4]> = features
            .iter()
            .map(|f| intake(f.geometry.as_ref()).unwrap().0)
            .collect();
        let (dir, disk_feats) = setup_group(&features);
        let midpolygons_path = dir.join("group_000000").join("midpolygons.jsonl");
        overlay_2d_disk(&aabbs, &disk_feats, tolerance, &midpolygons_path).unwrap();
        let pieces = BufReader::new(File::open(&midpolygons_path).unwrap())
            .lines()
            .map(|l| l.unwrap())
            .filter(|l| !l.is_empty())
            .map(|l| serde_json::from_str(&l).unwrap())
            .collect();
        (dir, pieces)
    }

    #[test]
    fn two_overlapping_squares_subdivide_into_three_pieces() {
        let (dir, pieces) = run_overlay(
            vec![
                square([0.0, 0.0], [2.0, 2.0]),
                square([1.0, 1.0], [3.0, 3.0]),
            ],
            0.01,
        );
        assert_eq!(pieces.len(), 3);
        let areas = pieces.iter().filter(|p| p.parents.len() == 2).count();
        let remnants = pieces.iter().filter(|p| p.parents.len() == 1).count();
        assert_eq!((areas, remnants), (1, 2));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_triangle_atop_the_shared_base_of_another_yields_two_pieces() {
        let (dir, pieces) = run_overlay(
            vec![
                triangle([[0.0, 0.0], [2.0, 0.0], [1.0, 2.0]]),
                triangle([[0.0, 0.0], [2.0, 0.0], [1.0, 1.0]]),
            ],
            0.01,
        );
        assert_eq!(pieces.len(), 2);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_mesh_overlaps_a_polygon_like_the_polygon_it_dissolves_to() {
        let (dir, pieces) = run_overlay(vec![square_mesh(), square([1.0, 1.0], [3.0, 3.0])], 0.01);
        assert_eq!(pieces.len(), 3);
        let areas = pieces.iter().filter(|p| p.parents.len() == 2).count();
        assert_eq!(areas, 1);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn an_untouched_feature_keeps_its_geometry_verbatim() {
        let (dir, pieces) = run_overlay(vec![square_mesh()], 0.01);
        assert_eq!(pieces.len(), 1);
        let Geometry::Euclidean2D(expected) = square_mesh() else {
            unreachable!();
        };
        assert_eq!(pieces[0].polygon, expected);
        assert_eq!(pieces[0].parents, vec![0]);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_tolerance_pulls_boundaries_that_nearly_coincide_together() {
        // Two squares meant to share the edge at x = 2, overlapping past it by
        // 0.001. The strip they share has area 0.002, which clears the
        // sub-tolerance area filter (0.01^2 = 0.0001), so without snapping it
        // survives as a spurious overlap piece alongside the two remnants.
        let inputs = vec![
            square([0.0, 0.0], [2.0, 2.0]),
            square([1.999, 0.0], [4.0, 2.0]),
        ];

        let (dir, pieces) = run_overlay(inputs.clone(), 0.0);
        assert_eq!(pieces.len(), 3);
        assert_eq!(pieces.iter().filter(|p| p.parents.len() == 2).count(), 1);
        let _ = std::fs::remove_dir_all(&dir);

        // Snapping shares the boundary before the subdivision runs, so there is
        // no overlap left to find and each square is simply its own remnant.
        let (dir, pieces) = run_overlay(inputs, 0.01);
        assert_eq!(pieces.len(), 2);
        assert!(pieces.iter().all(|p| p.parents.len() == 1));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_thin_overlap_below_the_tolerance_is_dropped() {
        let (dir, pieces) = run_overlay(
            vec![
                square([0.0, 0.0], [2.0, 2.0]),
                square([1.95, 0.0], [3.95, 2.0]),
            ],
            0.5,
        );
        assert!(pieces.iter().all(|p| p.parents.len() == 1));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn intake_accepts_a_closed_line_string_as_areal() {
        let line = LineString2D::from_coords(
            CoordinateFrame::Euclidean,
            square_ring([0.0, 0.0], [2.0, 2.0]),
        );
        let geometry = Geometry::Euclidean2D(Euclidean2DGeometry::LineString(line));
        let (aabb, frame) = intake(&geometry).unwrap();
        assert_eq!(aabb, [0.0, 0.0, 2.0, 2.0]);
        assert_eq!(frame, &CoordinateFrame::Euclidean);
    }

    #[test]
    fn intake_rejects_an_open_line_string() {
        let line = LineString2D::from_coords(
            CoordinateFrame::Euclidean,
            [[0.0, 0.0], [2.0, 0.0], [2.0, 2.0]],
        );
        let geometry = Geometry::Euclidean2D(Euclidean2DGeometry::LineString(line));
        assert!(intake(&geometry).is_none());
    }

    #[test]
    fn intake_rejects_a_three_dimensional_geometry() {
        let line = reearth_flow_geometry::line_string::LineString3D::from_coords(
            CoordinateFrame::Euclidean,
            [[0.0, 0.0, 0.0], [1.0, 1.0, 1.0]],
        );
        let geometry =
            Geometry::Euclidean3D(reearth_flow_geometry::Euclidean3DGeometry::LineString(line));
        assert!(intake(&geometry).is_none());
    }

    #[test]
    fn intake_rejects_members_in_different_frames() {
        let in_euclidean = Polygon2D::from_rings(
            CoordinateFrame::Euclidean,
            square_ring([0.0, 0.0], [1.0, 1.0]),
            Vec::<Vec<[f64; 2]>>::new(),
        );
        let in_crs = Polygon2D::from_rings(
            CoordinateFrame::Crs(reearth_flow_geometry::coordinate::EpsgCode::new(6677)),
            square_ring([0.0, 0.0], [1.0, 1.0]),
            Vec::<Vec<[f64; 2]>>::new(),
        );
        let geometry = Geometry::Euclidean2D(Euclidean2DGeometry::Collection(Collection2D::new([
            Euclidean2DGeometry::Polygon(Box::new(in_euclidean)),
            Euclidean2DGeometry::Polygon(Box::new(in_crs)),
        ])));
        assert!(intake(&geometry).is_none());
    }

    #[test]
    fn a_group_admits_only_the_frame_its_first_feature_fixes() {
        let mut overlayer = AreaOnAreaOverlayer {
            group_by: None,
            output_attribute: DEFAULT_OVERLAP_COUNT_ATTRIBUTE.to_string(),
            list_attribute: None,
            attribute_accumulation: AttributeAccumulation::default(),
            tolerance: 0.0,
            group_map: HashMap::new(),
            group_crs: HashMap::new(),
            group_count: 0,
            temp_dir: None,
            buffer: HashMap::new(),
            buffer_bytes: 0,
            executor_id: None,
        };
        let euclidean = CoordinateFrame::Euclidean;
        let crs = CoordinateFrame::Crs(reearth_flow_geometry::coordinate::EpsgCode::new(6677));

        assert!(overlayer.admit_crs(0, &euclidean));
        assert!(overlayer.admit_crs(0, &euclidean));
        assert!(!overlayer.admit_crs(0, &crs));
        // A different group is free to fix a different frame.
        assert!(overlayer.admit_crs(1, &crs));
    }

    #[test]
    fn intake_rejects_an_elevated_polygon() {
        assert!(intake(&square_at([0.0, 0.0], [2.0, 2.0], 5.0)).is_none());
    }

    #[test]
    fn intake_rejects_a_geometry_whose_members_are_partly_elevated() {
        let Geometry::Euclidean2D(planar) = square([0.0, 0.0], [1.0, 1.0]) else {
            unreachable!();
        };
        let Geometry::Euclidean2D(elevated) = square_at([2.0, 2.0], [3.0, 3.0], 5.0) else {
            unreachable!();
        };
        let geometry = Geometry::Euclidean2D(Euclidean2DGeometry::Collection(Collection2D::new([
            planar, elevated,
        ])));
        assert!(intake(&geometry).is_none());
    }

    #[test]
    fn intake_rejects_an_elevated_closed_line_string() {
        let line = LineString2D::from_coords_at_elevation(
            CoordinateFrame::Euclidean,
            square_ring([0.0, 0.0], [2.0, 2.0]),
            5.0,
        );
        let geometry = Geometry::Euclidean2D(Euclidean2DGeometry::LineString(line));
        assert!(intake(&geometry).is_none());
    }

    #[test]
    fn pieces_of_planar_parents_stay_planar() {
        let features = vec![
            Feature::from(square([0.0, 0.0], [2.0, 2.0])),
            Feature::from(square([1.0, 1.0], [3.0, 3.0])),
        ];
        let (dir, disk_feats) = setup_group(&features);
        let mut shaper = OutputShaper;

        let area_0 = read_working_area(&disk_feats, 0).unwrap();
        let area_1 = read_working_area(&disk_feats, 1).unwrap();
        let piece = area_intersection(&area_0, &area_1).unwrap();

        let mut feature = Feature::new_with_attributes(IndexMap::new());
        shaper.apply(&mut feature, piece);
        let Geometry::Euclidean2D(geom) = feature.geometry.as_ref() else {
            panic!("expected a 2D geometry");
        };
        let mut leaves = Vec::new();
        flatten_2d(geom, &mut leaves);
        assert!(!leaves.is_empty());
        assert!(leaves.iter().all(|l| leaf_elevation(l).is_none()));

        let _ = std::fs::remove_dir_all(&dir);
    }
}
