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
use reearth_flow_geometry::algorithm::line_intersection::LineIntersection;
use reearth_flow_geometry::algorithm::line_string_ops::{
    LineStringOps, LineStringSplitResult, LineStringWithTree2D,
};
use reearth_flow_geometry::algorithm::GeoFloat;
use reearth_flow_geometry::types::coordinate::Coordinate2D;
use reearth_flow_geometry::types::geometry::Geometry2D;
use reearth_flow_geometry::types::line_string::LineString2D;
use reearth_flow_geometry::types::no_value::NoValue;
use reearth_flow_geometry::types::point::{Point, Point2D};
use reearth_flow_runtime::{
    cache::executor_cache_subdir,
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT, REJECTED_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, Attributes, Feature, Geometry, GeometryValue};
use rstar::{RTree, RTreeObject, AABB};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Number, Value};

use super::errors::GeometryProcessorError;
use crate::ACCUMULATOR_BUFFER_BYTE_THRESHOLD;

pub static POINT_PORT: Lazy<Port> = Lazy::new(|| Port::new("point"));
pub static LINE_PORT: Lazy<Port> = Lazy::new(|| Port::new("line"));

#[derive(Debug, Clone, Default)]
pub struct LineOnLineOverlayerFactory;

impl ProcessorFactory for LineOnLineOverlayerFactory {
    fn name(&self) -> &str {
        "LineOnLineOverlayer"
    }

    fn description(&self) -> &str {
        "Intersection points are turned into point features that can contain the merged list of attributes of the original intersected lines."
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(LineOnLineOverlayerParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Geometry"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
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
            overlaid_lists_attr_name: params
                .overlaid_lists_attr_name
                .unwrap_or_else(|| "overlaidLists".to_string()),
            group_map: HashMap::new(),
            group_count: 0,
            temp_dir: None,
            buffer: HashMap::new(),
            buffer_bytes: 0,
            executor_id: None,
        }))
    }
}

/// # LineOnLineOverlayer Parameters
///
/// Configuration for finding intersection points between line features.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct LineOnLineOverlayerParam {
    group_by: Option<Vec<Attribute>>,
    tolerance: f64,
    /// Name of the attribute to store the overlaid lists. Defaults to "overlaidLists".
    overlaid_lists_attr_name: Option<String>,
}

pub struct LineOnLineOverlayer {
    group_by: Option<Vec<Attribute>>,
    tolerance: f64,
    overlaid_lists_attr_name: String,
    // Disk-backed state (mirrors AreaOnAreaOverlayer).
    group_map: HashMap<AttributeValue, usize>,
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
            overlaid_lists_attr_name: self.overlaid_lists_attr_name.clone(),
            group_map: HashMap::new(),
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
        let geometry = &feature.geometry;
        if geometry.is_empty() {
            fw.send(ctx.new_with_feature_and_port(ctx.feature.clone(), REJECTED_PORT.clone()));
            return Ok(());
        }
        match &geometry.value {
            GeometryValue::FlowGeometry2D(geom_2d) => {
                let line_strings = extract_line_strings(geom_2d);
                if line_strings.is_empty() {
                    fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
                    return Ok(());
                }

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

                let aabbs: Vec<[f64; 4]> = line_strings.iter().map(aabb_of_line_string).collect();
                let aabbs_json = serde_json::to_string(&aabbs)?;
                let feature_json = serde_json::to_string(&ctx.feature)?;
                self.append_to_group(group_idx, aabbs_json, feature_json)?;
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
                &self.overlaid_lists_attr_name,
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
        "LineOnLineOverlayer"
    }
}

fn extract_line_strings(geom: &Geometry2D<f64>) -> Vec<LineString2D<f64>> {
    match geom {
        Geometry2D::LineString(line) => vec![line.clone()],
        Geometry2D::MultiLineString(multi) => multi.0.clone(),
        Geometry2D::Polygon(polygon) => vec![polygon.exterior().clone()],
        Geometry2D::MultiPolygon(multi) => multi.0.iter().map(|p| p.exterior().clone()).collect(),
        _ => Vec::new(),
    }
}

fn aabb_of_line_string(ls: &LineString2D<f64>) -> [f64; 4] {
    let env = ls.envelope();
    let lo = env.lower();
    let hi = env.upper();
    [lo.x(), lo.y(), hi.x(), hi.y()]
}

fn aabb_to_rstar(aabb: [f64; 4]) -> AABB<Point2D<f64>> {
    AABB::from_corners(
        Point2D::new_(aabb[0], aabb[1], NoValue),
        Point2D::new_(aabb[2], aabb[3], NoValue),
    )
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
    entry_idx: usize,
    feature_idx: usize,
    ls_local_idx: usize,
    aabb: AABB<Point2D<f64>>,
}

impl RTreeObject for AabbEntry {
    type Envelope = AABB<Point2D<f64>>;

    fn envelope(&self) -> Self::Envelope {
        self.aabb
    }
}

fn process_group<W: Write>(
    group_dir: &Path,
    tolerance: f64,
    group_by: Option<&[Attribute]>,
    overlaid_lists_attr_name: &str,
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

    let mut entries: Vec<AabbEntry> = Vec::new();
    for (feature_idx, lss) in aabbs_per_feature.iter().enumerate() {
        for (ls_local_idx, aabb) in lss.iter().enumerate() {
            entries.push(AabbEntry {
                entry_idx: entries.len(),
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
    let mut attributes_by_feature: Vec<Arc<Attributes>> = Vec::with_capacity(disk_feats.len());
    let mut lss_per_feature: Vec<Vec<LineString2D<f64>>> = Vec::with_capacity(disk_feats.len());
    for i in 0..disk_feats.len() {
        let feat = disk_feats.read_feature(i)?;
        let lss = match &feat.geometry.value {
            GeometryValue::FlowGeometry2D(g2) => extract_line_strings(g2),
            _ => Vec::new(),
        };
        attributes_by_feature.push(feat.attributes);
        lss_per_feature.push(lss);
    }

    let overlay = overlay_entries(&entries, &lss_per_feature, tolerance);

    let mut line_count: usize = 0;
    for meta in &overlay.line_strings_with_metadata {
        let source_feature_idxs: Vec<usize> = meta
            .overlay_ids
            .iter()
            .map(|&entry_idx| entries[entry_idx].feature_idx)
            .collect();

        let mut attributes = Attributes::new();
        attributes.insert(
            Attribute::new("overlayCount"),
            AttributeValue::Number(Number::from(meta.overlay_count)),
        );

        let mut overlaid_list: Vec<AttributeValue> = Vec::with_capacity(source_feature_idxs.len());
        for &fi in &source_feature_idxs {
            let attrs = &attributes_by_feature[fi];
            let attrs_map: HashMap<String, AttributeValue> = attrs
                .as_ref()
                .iter()
                .map(|(k, v)| (k.clone().inner(), v.clone()))
                .collect();
            overlaid_list.push(AttributeValue::Map(attrs_map));
        }
        attributes.insert(
            Attribute::new(overlaid_lists_attr_name),
            AttributeValue::Array(overlaid_list),
        );

        if let Some(group_by) = group_by {
            let first_fi = source_feature_idxs[0];
            let first_attrs = &attributes_by_feature[first_fi];
            for gb in group_by {
                if let Some(value) = first_attrs.get(gb) {
                    attributes.insert(gb.clone(), value.clone());
                } else {
                    return Err(Box::new(
                        GeometryProcessorError::LineOnLineOverlayerFactory(
                            "Group by attribute not found in feature".to_string(),
                        ),
                    ));
                }
            }
        }

        let geometry = Geometry {
            value: GeometryValue::FlowGeometry2D(Geometry2D::LineString(meta.line_string.clone())),
            ..Default::default()
        };
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
        let geometry = Geometry {
            value: GeometryValue::FlowGeometry2D(Geometry2D::Point(Point(*coord))),
            ..Default::default()
        };
        let out = Feature::new_with_attributes_and_geometry(attributes, geometry);
        serde_json::to_writer(&mut *point_writer, &out)?;
        point_writer.write_all(b"\n")?;
        point_count += 1;
    }

    Ok((line_count, point_count))
}

#[derive(Debug, Clone)]
struct LineString2DWithMetadata<T: GeoFloat> {
    line_string: LineString2D<T>,
    /// Number of original entries that contributed to this line string.
    overlay_count: usize,
    /// Indices into the `entries` slice passed to `overlay_entries`.
    overlay_ids: Vec<usize>,
}

#[derive(Debug, Clone)]
struct OverlayResult {
    line_strings_with_metadata: Vec<LineString2DWithMetadata<f64>>,
    split_coords: Vec<Coordinate2D<f64>>,
}

fn overlay_entries(
    entries: &[AabbEntry],
    lss_per_feature: &[Vec<LineString2D<f64>>],
    tolerance: f64,
) -> OverlayResult {
    let rtree: RTree<AabbEntry> = RTree::bulk_load(entries.to_vec());

    type PerEntryResult = (Vec<(usize, LineString2D<f64>)>, Vec<Coordinate2D<f64>>);
    let per_entry_results: Vec<PerEntryResult> = entries
        .par_iter()
        .map(|entry_i| {
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
                .map(|l| (entry_i.entry_idx, l))
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
    let mut line_strings_with_metadata: Vec<LineString2DWithMetadata<f64>> = Vec::new();
    for i in 0..segments.len() {
        if processed[i] {
            continue;
        }
        let (idx1, ls1) = segments[i].clone();
        let feat_i = entries[idx1].feature_idx;
        let mut overlay_count = 1;
        let mut overlay_ids = vec![idx1];
        // A single feature may contribute multiple matching segments (e.g. a closed ring
        // whose split produces several arcs that all coincide with the rep segment). Count
        // each feature at most once; extra matching segments dedupe silently.
        let mut included_feats: std::collections::HashSet<usize> =
            std::collections::HashSet::from([feat_i]);

        for cand in seg_rtree.locate_in_envelope_intersecting(&seg_aabbs[i]) {
            let j = cand.idx;
            if j <= i || processed[j] {
                continue;
            }
            let (idx2, ls2) = (segments[j].0, &segments[j].1);
            if segments_match(&ls1, ls2, tolerance) {
                let cand_feat = entries[idx2].feature_idx;
                if !included_feats.insert(cand_feat) {
                    processed[j] = true;
                    continue;
                }
                overlay_count += 1;
                overlay_ids.push(idx2);
                processed[j] = true;
            }
        }

        line_strings_with_metadata.push(LineString2DWithMetadata {
            line_string: ls1,
            overlay_count,
            overlay_ids,
        });
    }

    // Each physical intersection is discovered by both sides of the crossing plus extras
    // from 3+-way near-coincidences, so dedup by tolerance-expanded envelope. Sources are
    // discarded by design, matching pre-rewrite behavior.
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

fn line_string_length_2d(ls: &LineString2D<f64>) -> f64 {
    ls.0.windows(2).map(|w| (w[1] - w[0]).norm()).sum()
}

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

#[cfg(test)]
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
            entry_idx: i,
            feature_idx: i,
            ls_local_idx: 0,
            aabb: ls.envelope(),
        })
        .collect();
    overlay_entries(&entries, &lss_per_feature, tolerance)
}

#[cfg(test)]
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
            .map(|ls| ls.overlay_count)
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
            .all(|ls| ls.overlay_count == 1));
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
            .map(|ls| ls.overlay_count)
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
            .all(|m| m.overlay_count == 1));
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
                entry_idx: 0,
                feature_idx: 0,
                ls_local_idx: 0,
                aabb: f0_a.envelope(),
            },
            AabbEntry {
                entry_idx: 1,
                feature_idx: 0,
                ls_local_idx: 1,
                aabb: f0_b.envelope(),
            },
            AabbEntry {
                entry_idx: 2,
                feature_idx: 1,
                ls_local_idx: 0,
                aabb: f1.envelope(),
            },
        ];

        let result = overlay_entries(&entries, &lss_per_feature, 0.01);

        for meta in &result.line_strings_with_metadata {
            let feats: Vec<usize> = meta
                .overlay_ids
                .iter()
                .map(|&e| entries[e].feature_idx)
                .collect();
            let unique: std::collections::HashSet<_> = feats.iter().copied().collect();
            assert_eq!(
                feats.len(),
                unique.len(),
                "overlay_ids contains duplicate feature_idx: {feats:?}"
            );
            assert_eq!(meta.overlay_count, meta.overlay_ids.len());
        }

        let overlapping: Vec<_> = result
            .line_strings_with_metadata
            .iter()
            .filter(|m| m.overlay_count >= 2)
            .collect();
        assert!(!overlapping.is_empty(), "expected an overlap segment");
        for m in &overlapping {
            assert_eq!(m.overlay_count, 2);
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
                entry_idx: 0,
                feature_idx: 0,
                ls_local_idx: 0,
                aabb: f0.envelope(),
            },
            AabbEntry {
                entry_idx: 1,
                feature_idx: 1,
                ls_local_idx: 0,
                aabb: f1_a.envelope(),
            },
            AabbEntry {
                entry_idx: 2,
                feature_idx: 1,
                ls_local_idx: 1,
                aabb: f1_b.envelope(),
            },
        ];

        let result = overlay_entries(&entries, &lss_per_feature, 0.01);

        let overlap: Vec<_> = result
            .line_strings_with_metadata
            .iter()
            .filter(|m| m.overlay_count >= 2)
            .collect();
        assert!(!overlap.is_empty(), "expected an overlap segment");
        for m in &overlap {
            let feats: std::collections::HashSet<usize> = m
                .overlay_ids
                .iter()
                .map(|&e| entries[e].feature_idx)
                .collect();
            assert_eq!(
                feats.len(),
                m.overlay_ids.len(),
                "duplicate feature_idx in overlay_ids: {:?}",
                m.overlay_ids
                    .iter()
                    .map(|&e| entries[e].feature_idx)
                    .collect::<Vec<_>>()
            );
            assert_eq!(m.overlay_count, 2);
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
            "overlaidLists",
            &mut line_buf,
            &mut point_buf,
        )
        .unwrap();
        assert_eq!(lc, 4);
        assert_eq!(pc, 1);

        let _ = std::fs::remove_dir_all(&dir);
    }
}
