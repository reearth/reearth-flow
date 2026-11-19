//! Mesh Merger: merges the faces of grouped features into meshes.
//!
//! Features are spilled to one file per group as they arrive, as two lines:
//! the geometry, and the rest of the feature. Only each feature's line
//! positions stay in memory. At the end, each group is merged on its own. For
//! the part granularities, the features are read once more, each edge is mapped
//! to the first feature having it, a later feature having the same edge is
//! joined to that one, and each connected set of features is merged and then
//! decomposed.
//!
//! Merging and decomposing read geometry only, and keep for each output part
//! the ids of the features that gave it faces. A part's attributes are built
//! from those features alone, read back one at a time just before the part is
//! sent.

use std::collections::HashMap;
use std::collections::hash_map::Entry as MapEntry;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::PathBuf;

use rayon::prelude::*;
use reearth_flow_geometry::Geometry;
use reearth_flow_geometry::coordinate::CoordinateFrame;
use reearth_flow_geometry::ops::merge::{EdgeKey, edge_keys, is_mergeable, merge};
use reearth_flow_geometry::ops::{
    Aabb, BoundingBox, DecomposeIntoEdgeConnectedParts, DecomposeIntoManifolds, Elevation,
    common_frame,
};
use reearth_flow_runtime::{
    cache::executor_cache_subdir,
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{FEATURES_PORT, Port, Processor, ProcessorFactory, REJECTED_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, Attributes, Feature};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::GeometryProcessorError;
use crate::ACCUMULATOR_BUFFER_BYTE_THRESHOLD;
use crate::accumulation::{AttributeAccumulation, AttributeAccumulator, MEMBERS_ATTRIBUTE};

/// How many features are read and listed in parallel while their edges are
/// mapped.
const EDGE_CHUNK_LEN: usize = 4096;

#[derive(Debug, Clone, Default)]
pub struct MeshMergerFactory;

impl ProcessorFactory for MeshMergerFactory {
    fn name(&self) -> &str {
        "Mesh Merger"
    }

    fn description(&self) -> &str {
        "Merges the polygons and meshes of features that share the same grouping attribute values into meshes, keeping every face as it is, and emits one feature per group or per connected part of it. The result is a triangular mesh when every polygon and mesh merged into it is a triangular mesh, including those inside collections, and a polygon mesh otherwise."
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(MeshMergerParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Geometry"]
    }

    fn tags(&self) -> &[&'static str] {
        &["aggregation"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![FEATURES_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![FEATURES_PORT.clone(), REJECTED_PORT.clone()]
    }

    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let param: MeshMergerParam = match with {
            Some(with) => {
                let value = serde_json::to_value(with).map_err(|e| {
                    GeometryProcessorError::MeshMergerFactory(format!(
                        "Failed to serialize `with` parameter: {e}"
                    ))
                })?;
                serde_json::from_value(value).map_err(|e| {
                    GeometryProcessorError::MeshMergerFactory(format!(
                        "Failed to deserialize `with` parameter: {e}"
                    ))
                })?
            }
            None => MeshMergerParam::default(),
        };
        let group_by = param.group_by.unwrap_or_default();
        if param.attribute_accumulation == AttributeAccumulation::CollectAttributes
            && group_by.contains(&Attribute::new(MEMBERS_ATTRIBUTE))
        {
            return Err(GeometryProcessorError::MeshMergerFactory(format!(
                "`groupBy` cannot include `{MEMBERS_ATTRIBUTE}` with `collectAttributes`, which writes the member attributes there"
            ))
            .into());
        }
        Ok(Box::new(MeshMerger::new(
            group_by,
            param.attribute_accumulation,
            param.granularity,
        )))
    }
}

/// # Mesh Merger Parameters
/// Sets which features are merged together, how the result is divided, and which attributes it keeps.
#[derive(Serialize, Deserialize, Debug, Clone, Default, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct MeshMergerParam {
    /// # Group By Attributes
    /// Attributes whose values decide which features merge together; features matching on all of
    /// them form one group. When omitted, every feature belongs to a single group. Cannot include
    /// `__members` when attributes are collected, since that attribute holds the collected values.
    #[serde(default)]
    group_by: Option<Vec<Attribute>>,
    /// # Attribute Accumulation
    /// Which attributes each merged feature keeps, taken from the features merged into it.
    #[serde(default)]
    attribute_accumulation: AttributeAccumulation,
    /// # Granularity
    /// How finely each group is divided into output features.
    #[serde(default)]
    granularity: Granularity,
}

/// How finely each group is divided into output features.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum Granularity {
    /// # Group
    /// Emits one mesh per group.
    #[default]
    Group,
    /// # Edge-Connected Part
    /// Emits one mesh per set of faces connected through edges with exactly equal coordinates.
    /// Faces meeting at a vertex alone become separate meshes.
    EdgeConnectedPart,
    /// # Manifold Part
    /// Divides each edge-connected part further, so that every edge of a mesh borders one or two
    /// of its faces. Where more faces meet at one edge, two of them stay joined there.
    ManifoldPart,
}

/// What every feature of a group must share to merge: dimension, coordinate
/// frame, and in 2D the elevation.
#[derive(Debug, Clone, PartialEq)]
struct Signature {
    three_dimensional: bool,
    frame: Option<CoordinateFrame>,
    elevation: Option<u64>,
}

/// Where one line sits in a group's file.
#[derive(Debug, Clone, Copy)]
struct Span {
    offset: u64,
    len: u64,
}

/// A stored feature: where its geometry line and its shell line sit in the
/// group's file.
#[derive(Debug, Clone, Copy)]
struct Entry {
    geometry: Span,
    shell: Span,
}

/// A feature without its geometry, as written to the shell line.
#[derive(Serialize)]
struct ShellRef<'a> {
    id: uuid::Uuid,
    attributes: &'a Attributes,
}

/// A feature without its geometry, as read back from the shell line.
#[derive(Deserialize)]
struct Shell {
    id: uuid::Uuid,
    attributes: Attributes,
}

/// What one set of features merges into: each output part with the ids of the
/// features that gave it faces, in id order.
type Parts = Vec<(Geometry, Vec<EntryId>)>;

/// A feature of a group, by its position among the group's entries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct EntryId(usize);

/// One group's spill file and the in-memory index into it.
#[derive(Debug)]
struct Group {
    path: PathBuf,
    signature: Signature,
    entries: Vec<Entry>,
    /// Bytes already in the file.
    written: u64,
    /// Lines not yet written.
    pending: Vec<u8>,
}

impl Group {
    fn flush(&mut self) -> Result<(), BoxedError> {
        if self.pending.is_empty() {
            return Ok(());
        }
        // Write at `written` rather than appending, so a retry after a partial
        // write overwrites the partial bytes.
        let mut file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .open(&self.path)?;
        file.seek(SeekFrom::Start(self.written))?;
        file.write_all(&self.pending)?;
        self.written += self.pending.len() as u64;
        self.pending = Vec::new();
        Ok(())
    }

    /// Append one line to the pending bytes, returning where it will sit.
    fn push_line(&mut self, line: &[u8]) -> Span {
        let span = Span {
            offset: self.written + self.pending.len() as u64,
            len: line.len() as u64,
        };
        self.pending.extend_from_slice(line);
        self.pending.push(b'\n');
        span
    }

    fn entry(&self, id: EntryId) -> &Entry {
        &self.entries[id.0]
    }

    /// Every feature of the group, in id order.
    fn ids(&self) -> impl Iterator<Item = EntryId> + use<> {
        (0..self.entries.len()).map(EntryId)
    }

    fn read_geometry(&self, id: EntryId) -> Result<Geometry, BoxedError> {
        read_span(&mut File::open(&self.path)?, self.entry(id).geometry)
    }

    /// The whole feature, rebuilt from its two lines.
    fn read_feature(&self, id: EntryId) -> Result<Feature, BoxedError> {
        let mut file = File::open(&self.path)?;
        let entry = self.entry(id);
        let shell: Shell = read_span(&mut file, entry.shell)?;
        let geometry: Geometry = read_span(&mut file, entry.geometry)?;
        Ok(Feature {
            id: shell.id,
            attributes: std::sync::Arc::new(shell.attributes),
            geometry: std::sync::Arc::new(geometry),
        })
    }
}

fn read_span<T: serde::de::DeserializeOwned>(file: &mut File, span: Span) -> Result<T, BoxedError> {
    file.seek(SeekFrom::Start(span.offset))?;
    let mut buf = vec![0u8; span.len as usize];
    file.read_exact(&mut buf)?;
    Ok(serde_json::from_slice(&buf)?)
}

#[derive(Debug)]
pub struct MeshMerger {
    group_by: Vec<Attribute>,
    attribute_accumulation: AttributeAccumulation,
    granularity: Granularity,
    /// Pending bytes across all groups that trigger a flush.
    buffer_threshold: usize,
    executor_id: Option<uuid::Uuid>,
    dir: Option<PathBuf>,
    group_index: HashMap<AttributeValue, usize>,
    /// Groups in first-seen order, which is the order they are emitted in.
    groups: Vec<Group>,
    pending_bytes: usize,
}

impl Clone for MeshMerger {
    /// The spilled state belongs to one run, so a clone starts empty.
    fn clone(&self) -> Self {
        let mut merger = Self::new(
            self.group_by.clone(),
            self.attribute_accumulation.clone(),
            self.granularity,
        );
        merger.buffer_threshold = self.buffer_threshold;
        merger
    }
}

impl MeshMerger {
    fn new(
        group_by: Vec<Attribute>,
        attribute_accumulation: AttributeAccumulation,
        granularity: Granularity,
    ) -> Self {
        Self {
            group_by,
            attribute_accumulation,
            granularity,
            buffer_threshold: ACCUMULATOR_BUFFER_BYTE_THRESHOLD,
            executor_id: None,
            dir: None,
            group_index: HashMap::new(),
            groups: Vec::new(),
            pending_bytes: 0,
        }
    }

    fn key(&self, feature: &Feature) -> AttributeValue {
        AttributeValue::Array(
            self.group_by
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

    fn dir(&mut self) -> Result<PathBuf, BoxedError> {
        if self.dir.is_none() {
            let executor_id = self.executor_id.unwrap_or_else(uuid::Uuid::nil);
            let dir = executor_cache_subdir(executor_id, "processors")
                .join(format!("mesh-merger-{}", uuid::Uuid::new_v4()));
            std::fs::create_dir_all(&dir)?;
            self.dir = Some(dir);
        }
        Ok(self.dir.clone().expect("set above"))
    }

    fn flush_all(&mut self) -> Result<(), BoxedError> {
        for group in &mut self.groups {
            group.flush()?;
        }
        self.pending_bytes = 0;
        Ok(())
    }

    /// Store `feature` in its group. `false` when it cannot merge, on its own
    /// or with the group's features, or has no faces to give.
    fn admit(&mut self, feature: &Feature) -> Result<bool, BoxedError> {
        let geometry = feature.geometry.as_ref();
        if !is_mergeable(geometry) {
            return Ok(false);
        }
        let Some(signature) = signature(geometry) else {
            return Ok(false);
        };
        let key = self.key(feature);
        let index = match self.group_index.get(&key) {
            Some(&index) => {
                if self.groups[index].signature != signature {
                    return Ok(false);
                }
                index
            }
            None => {
                let path = self
                    .dir()?
                    .join(format!("group_{:06}.jsonl", self.groups.len()));
                self.groups.push(Group {
                    path,
                    signature,
                    entries: Vec::new(),
                    written: 0,
                    pending: Vec::new(),
                });
                self.group_index.insert(key, self.groups.len() - 1);
                self.groups.len() - 1
            }
        };
        let geometry_line = serde_json::to_vec(geometry)?;
        let shell_line = serde_json::to_vec(&ShellRef {
            id: feature.id,
            attributes: &feature.attributes,
        })?;
        let group = &mut self.groups[index];
        let geometry = group.push_line(&geometry_line);
        let shell = group.push_line(&shell_line);
        group.entries.push(Entry { geometry, shell });
        self.pending_bytes += geometry_line.len() + shell_line.len() + 2;
        if self.pending_bytes >= self.buffer_threshold {
            self.flush_all()?;
        }
        Ok(true)
    }

    /// The features of `group`, as sets of ids to merge together, each in id
    /// order and the sets ordered by their first id.
    fn components(&self, group: &Group) -> Result<Vec<Vec<EntryId>>, BoxedError> {
        if self.granularity == Granularity::Group {
            return Ok(vec![group.ids().collect()]);
        }
        let mut parent: Vec<EntryId> = group.ids().collect();
        let mut owner: HashMap<EdgeKey, EntryId, ahash::RandomState> = HashMap::default();
        let ids: Vec<EntryId> = group.ids().collect();
        for chunk in ids.chunks(EDGE_CHUNK_LEN) {
            let edges: Vec<Vec<EdgeKey>> = chunk
                .par_iter()
                .map(|&id| {
                    group
                        .read_geometry(id)
                        .map(|geometry| edge_keys(&geometry))
                        .map_err(|e| e.to_string())
                })
                .collect::<Result<_, String>>()
                .map_err(GeometryProcessorError::MeshMerger)?;
            for (&id, edges) in chunk.iter().zip(edges) {
                for edge in edges {
                    match owner.entry(edge) {
                        MapEntry::Occupied(first) => union(&mut parent, id, *first.get()),
                        MapEntry::Vacant(slot) => {
                            slot.insert(id);
                        }
                    }
                }
            }
        }
        Ok(components(&mut parent))
    }

    /// Merge the features `ids` of `group` and decompose the result, reading
    /// geometry only. `Err` carries a read or merge error, for the features to
    /// be rejected.
    fn merge_component(&self, group: &Group, ids: &[EntryId]) -> Result<Parts, BoxedError> {
        let mut read_error = None;
        let geometries = ids.iter().map_while(|&id| match group.read_geometry(id) {
            Ok(geometry) => Some(geometry),
            Err(e) => {
                read_error = Some(e);
                None
            }
        });
        let merged = merge(geometries);
        if let Some(e) = read_error {
            return Err(e);
        }
        // Every admitted feature has a face, so the merge always has one.
        let (mesh, counts) = merged
            .map_err(|e| GeometryProcessorError::MeshMerger(e.to_string()))?
            .ok_or_else(|| GeometryProcessorError::MeshMerger("no faces to merge".into()))?;
        // The end of each input's faces in the merged mesh.
        let ends: Vec<usize> = counts
            .iter()
            .scan(0, |end, &count| {
                *end += count;
                Some(*end)
            })
            .collect();
        let decomposed = match self.granularity {
            Granularity::Group => return Ok(vec![(mesh, ids.to_vec())]),
            Granularity::EdgeConnectedPart => mesh.decompose_into_edge_connected_parts(),
            Granularity::ManifoldPart => mesh.decompose_into_manifolds(),
        }
        .map_err(|e| GeometryProcessorError::MeshMerger(e.to_string()))?;
        Ok(decomposed
            .into_iter()
            .map(|(part, faces)| {
                let mut members: Vec<EntryId> = faces
                    .iter()
                    .map(|face| ids[ends.partition_point(|&end| end <= face.get())])
                    .collect();
                members.sort_unstable();
                members.dedup();
                (part, members)
            })
            .collect())
    }

    /// The attributes of a part made of the features `members` of `group`,
    /// reading their shells one at a time.
    fn accumulate(&self, group: &Group, members: &[EntryId]) -> Result<Attributes, BoxedError> {
        let mut accumulator =
            AttributeAccumulator::new(&self.attribute_accumulation, &self.group_by);
        let mut file = File::open(&group.path)?;
        for &id in members {
            let shell: Shell = read_span(&mut file, group.entry(id).shell)?;
            accumulator.push(&shell.attributes);
        }
        Ok(accumulator.finish())
    }
}

/// Join the sets of `a` and `b`, keeping the smaller root.
fn union(parent: &mut [EntryId], a: EntryId, b: EntryId) {
    let (a, b) = (find(parent, a), find(parent, b));
    if a != b {
        parent[a.max(b).0] = a.min(b);
    }
}

/// Union-find root of `id`, compressing the path.
fn find(parent: &mut [EntryId], mut id: EntryId) -> EntryId {
    while parent[id.0] != id {
        parent[id.0] = parent[parent[id.0].0];
        id = parent[id.0];
    }
    id
}

/// The ids of each set, given each id's root, ordered by first id.
fn components(parent: &mut [EntryId]) -> Vec<Vec<EntryId>> {
    let mut index_of_root: HashMap<EntryId, usize> = HashMap::new();
    let mut sets: Vec<Vec<EntryId>> = Vec::new();
    for id in (0..parent.len()).map(EntryId) {
        let root = find(parent, id);
        let index = *index_of_root.entry(root).or_insert_with(|| {
            sets.push(Vec::new());
            sets.len() - 1
        });
        sets[index].push(id);
    }
    sets
}

/// What a mergeable geometry must share with its group.
fn signature(geometry: &Geometry) -> Option<Signature> {
    let frame = common_frame(geometry).ok()?;
    Some(match geometry.bounding_box().ok()? {
        Aabb::D2 { .. } => Signature {
            three_dimensional: false,
            frame,
            elevation: geometry.elevation().map(f64::to_bits),
        },
        Aabb::D3 { .. } => Signature {
            three_dimensional: true,
            frame,
            elevation: None,
        },
    })
}

impl Processor for MeshMerger {
    fn is_accumulating(&self) -> bool {
        true
    }

    fn num_threads(&self) -> usize {
        1
    }

    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        if self.executor_id.is_none() {
            self.executor_id = Some(fw.executor_id());
        }
        if !self.admit(&ctx.feature)? {
            fw.send(ctx.new_with_feature_and_port(ctx.feature.clone(), REJECTED_PORT.clone()));
        }
        Ok(())
    }

    fn finish(
        &mut self,
        ctx: NodeContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let result = self.finish_groups(&ctx, fw);
        if let Some(dir) = self.dir.take() {
            let _ = std::fs::remove_dir_all(dir);
        }
        self.groups.clear();
        self.group_index.clear();
        result
    }

    fn name(&self) -> &str {
        "Mesh Merger"
    }
}

impl Drop for MeshMerger {
    /// Remove the spill files when the run ends before `finish`.
    fn drop(&mut self) {
        if let Some(dir) = self.dir.take() {
            let _ = std::fs::remove_dir_all(dir);
        }
    }
}

impl MeshMerger {
    fn finish_groups(
        &mut self,
        ctx: &NodeContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        self.flush_all()?;
        let send = |feature: Feature, port: &Port| {
            fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                ctx,
                feature,
                port.clone(),
            ));
        };
        for group in &self.groups {
            let components = self.components(group)?;
            // A chunk of components merges in parallel and is emitted in order,
            // so only one chunk's geometries are held at a time. Attributes are
            // read part by part as each part is sent.
            for chunk in components.chunks(rayon::current_num_threads().max(1)) {
                let results: Vec<Result<Parts, BoxedError>> = chunk
                    .par_iter()
                    .map(|ids| self.merge_component(group, ids))
                    .collect();
                for (ids, result) in chunk.iter().zip(results) {
                    match result {
                        Ok(parts) => {
                            for (geometry, members) in parts {
                                let attributes = self.accumulate(group, &members)?;
                                send(
                                    Feature::new_with_attributes_and_geometry(attributes, geometry),
                                    &FEATURES_PORT,
                                );
                            }
                        }
                        Err(_) => {
                            for &id in ids {
                                send(group.read_feature(id)?, &REJECTED_PORT);
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::utils::create_default_execute_context;
    use pretty_assertions::assert_eq;
    use reearth_flow_geometry::coordinate::EpsgCode;
    use reearth_flow_geometry::point::Point3D;
    use reearth_flow_geometry::polygon::Polygon3D;
    use reearth_flow_geometry::triangular_mesh::TriangularMesh3D;
    use reearth_flow_geometry::{Euclidean3DGeometry, Geometry};
    use reearth_flow_runtime::forwarder::NoopChannelForwarder;
    use reearth_flow_types::Attributes;

    fn triangle(frame: CoordinateFrame, corners: [[f64; 3]; 3], attrs: &[(&str, &str)]) -> Feature {
        let geometry = Geometry::Euclidean3D(Euclidean3DGeometry::Polygon(Box::new(
            Polygon3D::from_rings(frame, corners, Vec::<Vec<[f64; 3]>>::new()),
        )));
        let mut feature = Feature::from(geometry);
        for (k, v) in attrs {
            feature.insert(*k, AttributeValue::String(v.to_string()));
        }
        feature
    }

    fn lower(attrs: &[(&str, &str)]) -> Feature {
        triangle(
            CoordinateFrame::Euclidean,
            [[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]],
            attrs,
        )
    }

    fn upper(attrs: &[(&str, &str)]) -> Feature {
        triangle(
            CoordinateFrame::Euclidean,
            [[1., 0., 0.], [1., 1., 0.], [0., 1., 0.]],
            attrs,
        )
    }

    fn apart(attrs: &[(&str, &str)]) -> Feature {
        triangle(
            CoordinateFrame::Euclidean,
            [[5., 0., 0.], [6., 0., 0.], [5., 1., 0.]],
            attrs,
        )
    }

    fn merger(group_by: &[&str], strategy: AttributeAccumulation) -> MeshMerger {
        MeshMerger::new(
            group_by.iter().map(|a| Attribute::new(*a)).collect(),
            strategy,
            Granularity::Group,
        )
    }

    fn run(mut merger: MeshMerger, features: Vec<Feature>) -> (Vec<Feature>, Vec<Feature>) {
        let fw = ProcessorChannelForwarder::Noop(NoopChannelForwarder::default());
        for feature in &features {
            merger
                .process(create_default_execute_context(feature), &fw)
                .unwrap();
        }
        merger.finish(NodeContext::default(), &fw).unwrap();
        let ProcessorChannelForwarder::Noop(noop) = fw else {
            unreachable!("the forwarder is the one built above");
        };
        let ports = noop.send_ports.lock().unwrap();
        let sent = noop.send_features.lock().unwrap();
        let mut merged = Vec::new();
        let mut rejected = Vec::new();
        for (port, feature) in ports.iter().zip(sent.iter()) {
            if *port == *FEATURES_PORT {
                merged.push(feature.clone());
            } else {
                rejected.push(feature.clone());
            }
        }
        (merged, rejected)
    }

    fn face_count(feature: &Feature) -> usize {
        let Geometry::Euclidean3D(Euclidean3DGeometry::PolygonMesh(mesh)) =
            feature.geometry.as_ref()
        else {
            panic!("expected a polygon mesh, got {:?}", feature.geometry);
        };
        mesh.num_faces()
    }

    fn string(v: &str) -> AttributeValue {
        AttributeValue::String(v.to_string())
    }

    #[test]
    fn each_group_becomes_one_mesh() {
        let (merged, rejected) = run(
            merger(&["rank"], AttributeAccumulation::UseOneFeature),
            vec![
                lower(&[("rank", "1")]),
                lower(&[("rank", "2")]),
                upper(&[("rank", "1")]),
            ],
        );
        assert!(rejected.is_empty());
        assert_eq!(merged.len(), 2);
        assert_eq!(face_count(&merged[0]), 2);
        assert_eq!(
            merged[0].attributes.get(&Attribute::new("rank")),
            Some(&string("1"))
        );
        assert_eq!(face_count(&merged[1]), 1);
    }

    #[test]
    fn group_granularity_keeps_disconnected_faces_together() {
        let (merged, _) = run(
            merger(&[], AttributeAccumulation::UseOneFeature),
            vec![lower(&[]), apart(&[])],
        );
        assert_eq!(merged.iter().map(face_count).collect::<Vec<_>>(), vec![2]);
    }

    #[test]
    fn collect_attributes_keeps_one_map_per_feature() {
        let (merged, _) = run(
            merger(&["rank"], AttributeAccumulation::CollectAttributes),
            vec![
                lower(&[("rank", "1"), ("id", "a")]),
                upper(&[("rank", "1")]),
            ],
        );
        let attributes = &merged[0].attributes;
        assert_eq!(attributes.get(&Attribute::new("rank")), Some(&string("1")));
        let first: Attributes = [(Attribute::new("id"), string("a"))].into_iter().collect();
        assert_eq!(
            attributes.get(&Attribute::new("__members")),
            Some(&AttributeValue::Array(vec![
                AttributeValue::Map(first),
                AttributeValue::Map(Attributes::new()),
            ]))
        );
        assert_eq!(attributes.get(&Attribute::new("id")), None);
    }

    #[test]
    fn group_attributes_only_drops_the_rest() {
        let (merged, _) = run(
            merger(&["rank"], AttributeAccumulation::GroupAttributesOnly),
            vec![lower(&[("rank", "1"), ("id", "a")])],
        );
        assert_eq!(merged[0].attributes.get(&Attribute::new("id")), None);
        assert_eq!(
            merged[0].attributes.get(&Attribute::new("rank")),
            Some(&string("1"))
        );
    }

    #[test]
    fn an_unmergeable_geometry_is_rejected() {
        let point = Feature::from(Geometry::Euclidean3D(Euclidean3DGeometry::Point(
            Point3D::new(CoordinateFrame::Euclidean, [0., 0., 0.]),
        )));
        let (merged, rejected) = run(
            merger(&[], AttributeAccumulation::UseOneFeature),
            vec![point, lower(&[])],
        );
        assert_eq!(merged.len(), 1);
        assert_eq!(rejected.len(), 1);
    }

    #[test]
    fn a_non_finite_coordinate_is_rejected_before_spilling() {
        let bad = triangle(
            CoordinateFrame::Euclidean,
            [[0., 0., 0.], [1., 0., f64::NAN], [0., 1., 0.]],
            &[],
        );
        let mut merger = merger(&[], AttributeAccumulation::UseOneFeature);
        merger.granularity = Granularity::EdgeConnectedPart;
        let (merged, rejected) = run(merger, vec![bad.clone(), lower(&[])]);
        assert_eq!(merged.len(), 1);
        assert_eq!(rejected.len(), 1);
        assert_eq!(rejected[0].id, bad.id);
    }

    #[test]
    fn coordinates_survive_the_spill_exactly() {
        let x = 0.9999999999999999;
        let feature = triangle(
            CoordinateFrame::Euclidean,
            [[x, 0., 0.], [2., 0., 0.], [0., 1., 0.]],
            &[],
        );
        let (merged, _) = run(
            merger(&[], AttributeAccumulation::UseOneFeature),
            vec![feature],
        );
        let Geometry::Euclidean3D(Euclidean3DGeometry::PolygonMesh(mesh)) =
            merged[0].geometry.as_ref()
        else {
            panic!("expected a polygon mesh, got {:?}", merged[0].geometry);
        };
        assert_eq!(mesh.vertices()[0][0].to_bits(), x.to_bits());
    }

    #[test]
    fn features_without_faces_are_rejected() {
        let empty = Feature::from(Geometry::Euclidean3D(Euclidean3DGeometry::TriangularMesh(
            Box::new(
                TriangularMesh3D::from_parts(
                    CoordinateFrame::Euclidean,
                    vec![[0., 0., 0.], [1., 1., 1.]],
                    Vec::new(),
                )
                .unwrap(),
            ),
        )));
        // It is rejected on arrival, before it is spilled.
        assert!(
            !merger(&[], AttributeAccumulation::UseOneFeature)
                .admit(&empty)
                .unwrap()
        );
        let (merged, rejected) = run(
            merger(&[], AttributeAccumulation::UseOneFeature),
            vec![empty.clone()],
        );
        assert!(merged.is_empty());
        assert_eq!(rejected.len(), 1);
        // The rejected geometry keeps the vertices no triangle uses.
        assert_eq!(rejected[0].geometry, empty.geometry);
    }

    #[test]
    fn a_feature_without_faces_does_not_decide_its_group_frame() {
        let empty = Feature::from(Geometry::Euclidean3D(Euclidean3DGeometry::TriangularMesh(
            Box::new(
                TriangularMesh3D::from_parts(
                    CoordinateFrame::Euclidean,
                    vec![[0., 0., 0.]],
                    Vec::new(),
                )
                .unwrap(),
            ),
        )));
        let other_frame = triangle(
            CoordinateFrame::Crs(EpsgCode::new(4326)),
            [[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]],
            &[],
        );
        let (merged, rejected) = run(
            merger(&[], AttributeAccumulation::UseOneFeature),
            vec![empty.clone(), other_frame],
        );
        assert_eq!(merged.len(), 1);
        assert_eq!(rejected.len(), 1);
        assert_eq!(rejected[0].id, empty.id);
    }

    #[test]
    fn grouping_by_the_members_attribute_is_refused_when_collecting() {
        let build = |strategy: &str| {
            let with = serde_json::from_value(serde_json::json!({
                "groupBy": ["__members"],
                "attributeAccumulation": strategy,
            }))
            .unwrap();
            MeshMergerFactory.build(
                NodeContext::default(),
                EventHub::new(1),
                "Mesh Merger".to_string(),
                Some(with),
            )
        };
        let Err(err) = build("collectAttributes") else {
            panic!("must be refused");
        };
        assert!(err.to_string().contains("__members"), "{err}");
        assert!(build("useOneFeature").is_ok());
    }

    #[test]
    fn dropping_before_finish_removes_the_spill_files() {
        let mut merger = merger(&[], AttributeAccumulation::UseOneFeature);
        merger.buffer_threshold = 1;
        let fw = ProcessorChannelForwarder::Noop(NoopChannelForwarder::default());
        let feature = lower(&[]);
        merger
            .process(create_default_execute_context(&feature), &fw)
            .unwrap();
        let dir = merger.dir.clone().expect("a feature was spilled");
        assert!(dir.exists());
        drop(merger);
        assert!(!dir.exists());
    }

    #[test]
    fn a_feature_in_another_frame_than_its_group_is_rejected_alone() {
        let other = triangle(
            CoordinateFrame::Crs(EpsgCode::new(4326)),
            [[1., 0., 0.], [1., 1., 0.], [0., 1., 0.]],
            &[],
        );
        let (merged, rejected) = run(
            merger(&[], AttributeAccumulation::UseOneFeature),
            vec![lower(&[]), other],
        );
        assert_eq!(merged.iter().map(face_count).collect::<Vec<_>>(), vec![1]);
        assert_eq!(rejected.len(), 1);
    }

    #[test]
    fn edge_connected_parts_become_separate_features() {
        let mut merger = merger(&["rank"], AttributeAccumulation::CollectAttributes);
        merger.granularity = Granularity::EdgeConnectedPart;
        let (merged, _) = run(
            merger,
            vec![
                lower(&[("rank", "1"), ("id", "a")]),
                apart(&[("rank", "1"), ("id", "b")]),
                upper(&[("rank", "1"), ("id", "c")]),
            ],
        );
        assert_eq!(
            merged.iter().map(face_count).collect::<Vec<_>>(),
            vec![2, 1]
        );
        // Each part carries the attributes of the features merged into it.
        let ids = |feature: &Feature| -> Vec<AttributeValue> {
            let Some(AttributeValue::Array(members)) =
                feature.attributes.get(&Attribute::new("__members"))
            else {
                panic!("expected members");
            };
            members
                .iter()
                .map(|m| {
                    let AttributeValue::Map(m) = m else {
                        panic!("expected a map")
                    };
                    m.get(&Attribute::new("id")).unwrap().clone()
                })
                .collect()
        };
        assert_eq!(ids(&merged[0]), vec![string("a"), string("c")]);
        assert_eq!(ids(&merged[1]), vec![string("b")]);
    }

    #[test]
    fn a_later_feature_joins_two_that_share_no_edge() {
        // `beside` touches `lower` at a vertex only; `upper`, last to arrive,
        // shares an edge with each.
        let beside = triangle(
            CoordinateFrame::Euclidean,
            [[1., 0., 0.], [2., 0., 0.], [1., 1., 0.]],
            &[],
        );
        for granularity in [Granularity::EdgeConnectedPart, Granularity::ManifoldPart] {
            let mut merger = merger(&[], AttributeAccumulation::UseOneFeature);
            merger.granularity = granularity;
            let (merged, rejected) = run(merger, vec![lower(&[]), beside.clone(), upper(&[])]);
            assert!(rejected.is_empty());
            assert_eq!(merged.iter().map(face_count).collect::<Vec<_>>(), vec![3]);
        }
    }

    #[test]
    fn a_disconnected_feature_is_split_after_merging() {
        // One feature holds two islands; the second island shares an edge with
        // another feature.
        let islands = Feature::from(Geometry::Euclidean3D(Euclidean3DGeometry::Collection(
            reearth_flow_geometry::collection::Collection3D::new([
                Euclidean3DGeometry::Polygon(Box::new(Polygon3D::from_rings(
                    CoordinateFrame::Euclidean,
                    [[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]],
                    Vec::<Vec<[f64; 3]>>::new(),
                ))),
                Euclidean3DGeometry::Polygon(Box::new(Polygon3D::from_rings(
                    CoordinateFrame::Euclidean,
                    [[5., 0., 0.], [6., 0., 0.], [5., 1., 0.]],
                    Vec::<Vec<[f64; 3]>>::new(),
                ))),
            ]),
        )));
        let next_to_second = triangle(
            CoordinateFrame::Euclidean,
            [[6., 0., 0.], [6., 1., 0.], [5., 1., 0.]],
            &[],
        );
        let mut merger = merger(&[], AttributeAccumulation::UseOneFeature);
        merger.granularity = Granularity::EdgeConnectedPart;
        let (merged, _) = run(merger, vec![islands, next_to_second]);
        assert_eq!(
            merged.iter().map(face_count).collect::<Vec<_>>(),
            vec![1, 2]
        );
    }

    #[test]
    fn manifold_parts_split_at_an_edge_of_three_faces() {
        let fin = |c: [f64; 3]| {
            triangle(
                CoordinateFrame::Euclidean,
                [[0., 0., 0.], [1., 0., 0.], c],
                &[],
            )
        };
        let mut merger = merger(&[], AttributeAccumulation::UseOneFeature);
        merger.granularity = Granularity::ManifoldPart;
        let (merged, _) = run(
            merger,
            vec![fin([0.5, 1., 0.]), fin([0.5, -1., 0.]), fin([0.5, 0., 1.])],
        );
        assert_eq!(
            merged.iter().map(face_count).collect::<Vec<_>>(),
            vec![2, 1]
        );
    }

    /// The `id` of every member of a `collectAttributes` feature.
    fn member_ids(feature: &Feature) -> Vec<AttributeValue> {
        let Some(AttributeValue::Array(members)) =
            feature.attributes.get(&Attribute::new("__members"))
        else {
            panic!("expected members");
        };
        members
            .iter()
            .map(|m| {
                let AttributeValue::Map(m) = m else {
                    panic!("expected a map")
                };
                m.get(&Attribute::new("id")).unwrap().clone()
            })
            .collect()
    }

    fn islands_and_neighbour() -> Vec<Feature> {
        let mut islands = Feature::from(Geometry::Euclidean3D(Euclidean3DGeometry::Collection(
            reearth_flow_geometry::collection::Collection3D::new([
                Euclidean3DGeometry::Polygon(Box::new(Polygon3D::from_rings(
                    CoordinateFrame::Euclidean,
                    [[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]],
                    Vec::<Vec<[f64; 3]>>::new(),
                ))),
                Euclidean3DGeometry::Polygon(Box::new(Polygon3D::from_rings(
                    CoordinateFrame::Euclidean,
                    [[5., 0., 0.], [6., 0., 0.], [5., 1., 0.]],
                    Vec::<Vec<[f64; 3]>>::new(),
                ))),
            ]),
        )));
        islands.insert("id", string("a"));
        let neighbour = triangle(
            CoordinateFrame::Euclidean,
            [[6., 0., 0.], [6., 1., 0.], [5., 1., 0.]],
            &[("id", "b")],
        );
        vec![islands, neighbour]
    }

    #[test]
    fn a_part_keeps_only_the_attributes_of_features_with_faces_in_it() {
        let mut merger = merger(&[], AttributeAccumulation::CollectAttributes);
        merger.granularity = Granularity::EdgeConnectedPart;
        let (merged, _) = run(merger, islands_and_neighbour());
        assert_eq!(
            merged.iter().map(member_ids).collect::<Vec<_>>(),
            vec![vec![string("a")], vec![string("a"), string("b")]]
        );
    }

    #[test]
    fn a_manifold_part_keeps_only_its_own_feature_attributes() {
        let fin = |c: [f64; 3], id: &str| {
            triangle(
                CoordinateFrame::Euclidean,
                [[0., 0., 0.], [1., 0., 0.], c],
                &[("id", id)],
            )
        };
        let features = vec![
            fin([0.5, 1., 0.], "a"),
            fin([0.5, -1., 0.], "b"),
            fin([0.5, 0., 1.], "c"),
        ];
        let mut collect = merger(&[], AttributeAccumulation::CollectAttributes);
        collect.granularity = Granularity::ManifoldPart;
        let (merged, _) = run(collect, features.clone());
        assert_eq!(
            merged.iter().map(member_ids).collect::<Vec<_>>(),
            vec![vec![string("a"), string("b")], vec![string("c")]]
        );
        // The first feature with faces in the part supplies its attributes.
        let mut one = merger(&[], AttributeAccumulation::UseOneFeature);
        one.granularity = Granularity::ManifoldPart;
        let (merged, _) = run(one, features);
        assert_eq!(
            merged
                .iter()
                .map(|f| f.attributes.get(&Attribute::new("id")).cloned())
                .collect::<Vec<_>>(),
            vec![Some(string("a")), Some(string("c"))]
        );
    }

    #[test]
    fn a_feature_without_faces_is_rejected_and_left_out_of_the_attributes() {
        let mut empty = Feature::from(Geometry::Euclidean3D(Euclidean3DGeometry::TriangularMesh(
            Box::new(
                TriangularMesh3D::from_parts(
                    CoordinateFrame::Euclidean,
                    vec![[0., 0., 0.], [1., 1., 1.]],
                    Vec::new(),
                )
                .unwrap(),
            ),
        )));
        empty.insert("id", string("empty"));
        let (merged, rejected) = run(
            merger(&[], AttributeAccumulation::CollectAttributes),
            vec![empty.clone(), lower(&[("id", "a")])],
        );
        assert_eq!(
            merged.iter().map(member_ids).collect::<Vec<_>>(),
            vec![vec![string("a")]]
        );
        // The rejected feature keeps its id and attributes.
        assert_eq!(rejected.len(), 1);
        assert_eq!(rejected[0].id, empty.id);
        assert_eq!(rejected[0].attributes, empty.attributes);
        assert_eq!(rejected[0].geometry, empty.geometry);
    }

    #[test]
    fn spilling_and_reading_back_keep_every_feature() {
        let mut merger = merger(&["rank"], AttributeAccumulation::CollectAttributes);
        merger.granularity = Granularity::EdgeConnectedPart;
        // Flush after every feature.
        merger.buffer_threshold = 1;
        let (merged, rejected) = run(
            merger,
            vec![
                lower(&[("rank", "1")]),
                apart(&[("rank", "2")]),
                upper(&[("rank", "1")]),
                apart(&[("rank", "1")]),
            ],
        );
        assert!(rejected.is_empty());
        assert_eq!(
            merged.iter().map(face_count).collect::<Vec<_>>(),
            vec![2, 1, 1]
        );
    }
}
