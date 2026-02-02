use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::{BufRead, BufReader, BufWriter, Read as _, Seek, SeekFrom, Write as _},
    path::PathBuf,
    sync::Arc,
};

use indexmap::IndexMap;
use once_cell::sync::Lazy;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use reearth_flow_common::dir::project_temp_dir;
use reearth_flow_geometry::{
    algorithm::{area2d::Area2D, bool_ops::BooleanOps},
    types::{geometry::Geometry2D, multi_polygon::MultiPolygon2D, polygon::Polygon2D},
};
use reearth_flow_runtime::{
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
            writers: HashMap::new(),
            temp_dir: None,
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

struct GroupWriters {
    aabb_writer: BufWriter<File>,
    feat_writer: BufWriter<File>,
}

impl std::fmt::Debug for GroupWriters {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GroupWriters").finish_non_exhaustive()
    }
}

struct AreaOnAreaOverlayer {
    group_by: Option<Vec<Attribute>>,
    output_attribute: Option<String>,
    generate_list: Option<String>,
    accumulation_mode: AccumulationMode,
    tolerance: f64,
    // Disk-backed state: open file handles per group
    group_map: HashMap<AttributeValue, usize>,
    group_count: usize,
    writers: HashMap<usize, GroupWriters>,
    temp_dir: Option<PathBuf>,
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
            writers: HashMap::new(),
            temp_dir: None,
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

impl AreaOnAreaOverlayer {
    fn ensure_temp_dir(&mut self) -> Result<&PathBuf, BoxedError> {
        if self.temp_dir.is_none() {
            let dir = project_temp_dir(uuid::Uuid::new_v4().to_string().as_str())?;
            self.temp_dir = Some(dir);
        }
        Ok(self.temp_dir.as_ref().unwrap())
    }

    fn ensure_writers(&mut self, group_idx: usize) -> Result<&mut GroupWriters, BoxedError> {
        if !self.writers.contains_key(&group_idx) {
            let dir = self.ensure_temp_dir()?.clone();
            let group_dir = dir.join(format!("group_{group_idx:06}"));
            std::fs::create_dir_all(&group_dir)?;

            let aabb_file = File::create(group_dir.join("aabbs.jsonl"))?;
            let feat_file = File::create(group_dir.join("features.jsonl"))?;

            self.writers.insert(
                group_idx,
                GroupWriters {
                    aabb_writer: BufWriter::new(aabb_file),
                    feat_writer: BufWriter::new(feat_file),
                },
            );
        }
        Ok(self.writers.get_mut(&group_idx).unwrap())
    }

    fn flush_all_writers(&mut self) -> Result<(), BoxedError> {
        for writers in self.writers.values_mut() {
            writers.aabb_writer.flush()?;
            writers.feat_writer.flush()?;
        }
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
                let writers = self.ensure_writers(group_idx)?;
                serde_json::to_writer(&mut writers.aabb_writer, &aabb)?;
                writers.aabb_writer.write_all(b"\n")?;
                writers.feat_writer.write_all(feature_json.as_bytes())?;
                writers.feat_writer.write_all(b"\n")?;
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
        self.flush_all_writers()?;
        // Drop all file handles before reading
        self.writers.clear();

        let temp_dir = match &self.temp_dir {
            Some(d) => d.clone(),
            None => return Ok(()),
        };

        let mut overlaid = OverlaidFeatures::new();

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
            let disk_geoms = DiskBackedFeatures::scan(&features_path)?;
            let num_features = disk_geoms.offsets.len();

            // Build geometries accessor for overlay_2d
            let midpolygons = overlay_2d_disk(&aabbs, &disk_geoms, self.tolerance);

            // Build OverlaidFeatures from midpolygons, reading attributes from disk
            let overlaid_features = OverlaidFeatures::from_midpolygons_disk(
                midpolygons,
                &disk_geoms,
                num_features,
                &self.group_by,
                &self.output_attribute,
                &self.generate_list,
                &self.accumulation_mode,
            );
            overlaid.extend(overlaid_features);
        }

        for feature in overlaid.area {
            fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                &ctx,
                feature,
                AREA_PORT.clone(),
            ));
        }
        for feature in overlaid.remnant {
            fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                &ctx,
                feature,
                REMNANTS_PORT.clone(),
            ));
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
#[derive(Debug, Clone)]
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

/// Overlay graph that stores which polygons overlay which other polygons.
struct OverlayGraph {
    graph: Vec<HashSet<usize>>,
}

impl OverlayGraph {
    fn bulk_load_from_aabbs(aabbs: &[[f64; 4]]) -> Self {
        let entries: Vec<AabbEntry> = aabbs
            .iter()
            .enumerate()
            .map(|(i, aabb)| AabbEntry {
                index: i,
                aabb: AABB::from_corners([aabb[0], aabb[1]], [aabb[2], aabb[3]]),
            })
            .collect();

        let tree = RTree::bulk_load(entries);

        let mut graph = vec![HashSet::new(); aabbs.len()];

        for i in 0..aabbs.len() {
            let aabb_i = AABB::from_corners([aabbs[i][0], aabbs[i][1]], [aabbs[i][2], aabbs[i][3]]);
            for entry in tree.locate_in_envelope_intersecting(&aabb_i) {
                if entry.index != i {
                    graph[i].insert(entry.index);
                }
            }
        }

        Self { graph }
    }

    fn overlaid_iter(&self, i: usize) -> impl Iterator<Item = &usize> {
        self.graph[i].iter()
    }
}

/// Disk-backed version of overlay_2d that reads geometries from disk on demand.
fn overlay_2d_disk(
    aabbs: &[[f64; 4]],
    disk_feats: &DiskBackedFeatures,
    tolerance: f64,
) -> Vec<MiddlePolygon> {
    let overlay_graph = OverlayGraph::bulk_load_from_aabbs(aabbs);
    let num = disk_feats.offsets.len();

    (0..num)
        .into_par_iter()
        .filter_map(|i| {
            let geom_i = disk_feats.read_geometry(i);
            let geom_i_2d = as_geometry_2d(&geom_i)?;

            let mut polygon_target = geom_to_multipolygon(geom_i_2d);

            // cut off the target polygon by upper polygons
            for j in overlay_graph.overlaid_iter(i).copied() {
                if i < j {
                    let geom_j = disk_feats.read_geometry(j);
                    if let Some(geom_j_2d) = as_geometry_2d(&geom_j) {
                        polygon_target = bool_op_difference(&polygon_target, geom_j_2d);
                    }
                }
            }

            let mut queue = vec![MiddlePolygon {
                polygon: polygon_target,
                parents: vec![i],
            }];

            // divide the target polygon by lower polygons
            for j in overlay_graph.overlaid_iter(i).copied() {
                if i > j {
                    let geom_j = disk_feats.read_geometry(j);
                    if let Some(geom_j_2d) = as_geometry_2d(&geom_j) {
                        let mut new_queue = Vec::new();
                        for subpolygon in queue {
                            let intersection =
                                bool_op_intersection(&subpolygon.polygon, geom_j_2d);

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

                            let difference =
                                bool_op_difference(&subpolygon.polygon, geom_j_2d);
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

            Some(queue)
        })
        .flatten()
        .collect::<Vec<_>>()
}

/// Features that are created as the result of the overlay process.
#[derive(Debug)]
struct OverlaidFeatures {
    area: Vec<Feature>,
    remnant: Vec<Feature>,
}

impl OverlaidFeatures {
    fn new() -> Self {
        Self {
            area: Vec::new(),
            remnant: Vec::new(),
        }
    }

    fn from_midpolygons_disk(
        midpolygons: Vec<MiddlePolygon>,
        disk_feats: &DiskBackedFeatures,
        _num_features: usize,
        _group_by: &Option<Vec<Attribute>>,
        output_attribute: &Option<String>,
        generate_list: &Option<String>,
        accumulation_mode: &AccumulationMode,
    ) -> Self {
        // Collect which feature indices we need to load attributes from
        let mut needed_indices: HashSet<usize> = HashSet::new();
        for mp in &midpolygons {
            for &p in &mp.parents {
                needed_indices.insert(p);
            }
        }

        // Load only the needed features' attributes
        let mut attributes_cache: HashMap<usize, Arc<IndexMap<Attribute, AttributeValue>>> =
            HashMap::new();
        for &idx in &needed_indices {
            let feature = disk_feats.read_feature(idx);
            attributes_cache.insert(idx, feature.attributes);
        }

        let mut area = Vec::new();
        let mut remnant = Vec::new();
        for subpolygon in midpolygons {
            match subpolygon.get_type() {
                MiddlePolygonType::None => {}
                MiddlePolygonType::Area(parents) => {
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
                    area.push(feature);
                }
                MiddlePolygonType::Remnant(parent) => {
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
                    remnant.push(feature);
                }
            }
        }
        Self { area, remnant }
    }

    fn extend(&mut self, other: Self) {
        self.area.extend(other.area);
        self.remnant.extend(other.remnant);
    }
}

#[cfg(test)]
mod tests {
    use reearth_flow_geometry::types::{
        coordinate::Coordinate2D, line_string::LineString2D, polygon::Polygon2D,
    };

    use super::*;

    fn make_geom(coords: Vec<(f64, f64)>) -> Arc<Geometry> {
        let ls = LineString2D::new(coords.into_iter().map(|(x, y)| Coordinate2D::new_(x, y)).collect());
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
        let dir = project_temp_dir(uuid::Uuid::new_v4().to_string().as_str()).unwrap();
        let group_dir = dir.join("group_000000");
        std::fs::create_dir_all(&group_dir).unwrap();

        let features = vec![
            make_feature(vec![(0.0, 0.0), (2.0, 0.0), (2.0, 2.0), (0.0, 2.0), (0.0, 0.0)]),
            make_feature(vec![(1.0, 1.0), (3.0, 1.0), (3.0, 3.0), (1.0, 3.0), (1.0, 1.0)]),
        ];

        let aabbs: Vec<[f64; 4]> = vec![
            [0.0, 0.0, 2.0, 2.0],
            [1.0, 1.0, 3.0, 3.0],
        ];

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
        let midpolygons = overlay_2d_disk(&aabbs, &disk_feats, 0.01);
        assert_eq!(midpolygons.len(), 3);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_overlay_triangles_sharing_an_edge_disk() {
        let dir = project_temp_dir(uuid::Uuid::new_v4().to_string().as_str()).unwrap();
        let group_dir = dir.join("group_000000");
        std::fs::create_dir_all(&group_dir).unwrap();

        let features = vec![
            make_feature(vec![(0.0, 0.0), (2.0, 0.0), (1.0, 2.0), (0.0, 0.0)]),
            make_feature(vec![(0.0, 0.0), (2.0, 0.0), (1.0, 1.0), (0.0, 0.0)]),
        ];

        let aabbs: Vec<[f64; 4]> = vec![
            [0.0, 0.0, 2.0, 2.0],
            [0.0, 0.0, 2.0, 1.0],
        ];

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
        let midpolygons = overlay_2d_disk(&aabbs, &disk_feats, 0.01);
        assert_eq!(midpolygons.len(), 2);

        let _ = std::fs::remove_dir_all(&dir);
    }
}
