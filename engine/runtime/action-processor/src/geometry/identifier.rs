//! Geometry identity: labelling the features whose geometries occupy the same
//! space.
//!
//! Deciding whether two geometries occupy the same space is
//! [`Equal`](reearth_flow_geometry::predicates::Equal)'s job, and what that
//! means is settled per geometry type there. This action does the rest: it
//! buffers the input, groups it, pairs candidates up by bounding box, and
//! numbers the results.
//!
//! Occupying the same space is not transitive once the tolerance is above zero,
//! so it is not on its own the equivalence the identifiers report: two features
//! share an identifier when a chain of same-space steps runs between them.

use std::collections::HashMap;

use reearth_flow_geometry::ops::{Aabb, BoundingBox};
use reearth_flow_geometry::predicates::{Equal, Tolerance};
use reearth_flow_geometry::Geometry;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{Context, ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, FEATURES_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, Feature};
use rstar::{RTree, RTreeObject, AABB};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::GeometryProcessorError;

/// Greatest angle between two adjacent faces still counted as lying in one flat
/// facet, which is what lets a mesh be weighed independently of how it was cut
/// into triangles. Not a knob: it is here to absorb the rounding in a computed
/// normal, not to merge shallow creases, so it stays far below any angle a real
/// surface turns through.
const COPLANARITY_TOLERANCE: f64 = 1e-9;

#[derive(Debug, Clone, Default)]
pub(super) struct GeometryIdentifierFactory;

impl ProcessorFactory for GeometryIdentifierFactory {
    fn name(&self) -> &str {
        "Geometry Identifier"
    }

    fn description(&self) -> &str {
        "Labels every feature with an identifier shared by the features whose geometry occupies \
         the same space, up to a tolerance. Optionally records on each feature the identifiers \
         the other features sharing its geometry carry."
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(GeometryIdentifierParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Geometry"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![FEATURES_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![FEATURES_PORT.clone()]
    }

    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let Some(with) = with else {
            return Err(GeometryProcessorError::GeometryIdentifierFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        let value: Value = serde_json::to_value(with).map_err(|e| {
            GeometryProcessorError::GeometryIdentifierFactory(format!(
                "Failed to serialize `with` parameter: {e}"
            ))
        })?;
        let params: GeometryIdentifierParam = serde_json::from_value(value).map_err(|e| {
            GeometryProcessorError::GeometryIdentifierFactory(format!(
                "Failed to deserialize `with` parameter: {e}"
            ))
        })?;
        // NaN is not a distance either, so the check is written to reject it.
        if !matches!(
            params.tolerance.partial_cmp(&0.0),
            Some(std::cmp::Ordering::Greater | std::cmp::Ordering::Equal)
        ) {
            return Err(GeometryProcessorError::GeometryIdentifierFactory(format!(
                "Tolerance must be zero or greater, got {}",
                params.tolerance
            ))
            .into());
        }

        Ok(Box::new(GeometryIdentifier {
            params,
            buffer: Vec::new(),
            groups: HashMap::new(),
        }))
    }
}

/// # Geometry Identifier Parameters
/// How close two geometries must stay to count as one shape, which features are
/// compared against which, and where the results are written.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct GeometryIdentifierParam {
    /// # Tolerance
    /// Greatest distance, in the units the coordinates are expressed in, that two geometries may
    /// stray from one another and still count as the same shape. Zero admits only geometries whose
    /// coordinates coincide exactly, which leaves no room for rounding; prefer a small positive
    /// distance.
    pub tolerance: f64,

    /// # Group By
    /// Attributes delimiting the set a geometry is compared against, such as a parent feature or
    /// a source file. Geometries in different groups are never identified with one another. When
    /// omitted, all input features form a single group.
    pub group_by: Option<Vec<Attribute>>,

    /// # Output Attribute
    /// Attribute the identifier is written to, as a zero-based index counted within the group.
    #[serde(default = "default_output_attribute")]
    pub output_attribute: Attribute,

    /// # ID Attribute
    /// Attribute holding the identifier of the feature itself. When set, the values the other
    /// features sharing a geometry carry for it are collected into the matched IDs attribute.
    pub id_attribute: Option<Attribute>,

    /// # Matched IDs Attribute
    /// Attribute the identifiers collected from the other features sharing a geometry are written
    /// to, as an array. Has no effect unless the ID attribute is set.
    #[serde(default = "default_matched_ids_attribute")]
    pub matched_ids_attribute: Attribute,
}

fn default_output_attribute() -> Attribute {
    Attribute::new("_equivalence_id")
}

fn default_matched_ids_attribute() -> Attribute {
    Attribute::new("_matched_ids")
}

/// One buffered input feature and the box its geometry sits in.
#[derive(Debug, Clone)]
struct BufferedFeature {
    /// `None` for a feature whose geometry occupies nowhere — absent, or an
    /// empty container. Such a feature shares its space with nothing and is
    /// left unlabelled.
    envelope: Option<Aabb>,
    feature: Feature,
}

#[derive(Debug, Clone)]
pub(super) struct GeometryIdentifier {
    params: GeometryIdentifierParam,
    /// Buffered features in arrival order; the output preserves that order.
    buffer: Vec<BufferedFeature>,
    /// Group key -> indices into `buffer`, each in arrival order.
    groups: HashMap<String, Vec<usize>>,
}

impl Processor for GeometryIdentifier {
    fn is_accumulating(&self) -> bool {
        true
    }

    fn process(
        &mut self,
        ctx: ExecutorContext,
        _fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        // A geometry that bounds nothing occupies nowhere. `Equal` is what
        // decides whether two geometries match; the box only pairs candidates up.
        let envelope = match &*feature.geometry {
            Geometry::None => None,
            geometry => geometry.bounding_box().ok(),
        };

        let group_key = self
            .params
            .group_by
            .as_deref()
            .unwrap_or_default()
            .iter()
            .map(|attribute| {
                feature
                    .attributes
                    .get(attribute)
                    .map(|value| value.to_string())
                    .unwrap_or_default()
            })
            .collect::<Vec<_>>()
            .join("|");

        let index = self.buffer.len();
        self.buffer.push(BufferedFeature {
            envelope,
            feature: feature.clone(),
        });
        self.groups.entry(group_key).or_default().push(index);

        Ok(())
    }

    fn finish(
        &mut self,
        ctx: NodeContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let verdicts = self.resolve()?;
        let ctx: Context = ctx.as_context();

        self.groups.clear();
        for (buffered, verdict) in std::mem::take(&mut self.buffer).into_iter().zip(verdicts) {
            let mut feature = buffered.feature;
            if let Some(verdict) = verdict {
                let attributes = feature.attributes_mut();
                attributes.insert(
                    self.params.output_attribute.clone(),
                    AttributeValue::Number(verdict.identifier.into()),
                );
                if self.params.id_attribute.is_some() {
                    attributes.insert(
                        self.params.matched_ids_attribute.clone(),
                        AttributeValue::Array(
                            verdict
                                .matched_ids
                                .into_iter()
                                .map(AttributeValue::String)
                                .collect(),
                        ),
                    );
                }
            }
            fw.send(ExecutorContext::new_with_context_feature_and_port(
                &ctx,
                feature,
                FEATURES_PORT.clone(),
            ));
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "Geometry Identifier"
    }
}

/// What one feature learned about the shape it shares.
#[derive(Debug, Clone, Default)]
struct Verdict {
    /// Zero-based index of the feature's shape within its group.
    identifier: usize,
    /// The ID attribute's value on the other features sharing the shape, in
    /// arrival order and without repeats. Empty unless an ID attribute is named.
    matched_ids: Vec<String>,
}

impl GeometryIdentifier {
    /// One verdict per buffered feature, indexed by position in `buffer`. `None`
    /// where the feature arrived without geometry and so shares none.
    fn resolve(&self) -> Result<Vec<Option<Verdict>>, BoxedError> {
        let mut verdicts = vec![None; self.buffer.len()];
        let tolerance = Tolerance {
            distance: self.params.tolerance,
            coplanarity: COPLANARITY_TOLERANCE,
        };

        for indices in self.groups.values() {
            // A 2D and a 3D geometry are not a pair `Equal` will weigh — there is
            // no implicit promotion between the embeddings — so they are binned
            // apart and never put to it.
            let mut bins: HashMap<u8, Vec<usize>> = HashMap::new();
            for &index in indices {
                if let Some(envelope) = &self.buffer[index].envelope {
                    bins.entry(embedding(envelope)).or_default().push(index);
                }
            }

            // Feature index -> the index representing the space it shares. Kept
            // across bins so identifiers are numbered once per group.
            let mut root_of: HashMap<usize, usize> = HashMap::new();
            for members in bins.values() {
                let mut union_find = UnionFind::new(members.len());
                let tree = RTree::bulk_load(
                    members
                        .iter()
                        .enumerate()
                        .map(|(slot, &index)| BoxEntry {
                            envelope: box_of(self.envelope(index), 0.0),
                            slot,
                        })
                        .collect(),
                );
                for (slot, &index) in members.iter().enumerate() {
                    // Only geometries whose boxes come within the tolerance of
                    // one another can occupy the same space, so the rest are
                    // never weighed.
                    let reach = box_of(self.envelope(index), self.params.tolerance);
                    for candidate in tree.locate_in_envelope_intersecting(&reach) {
                        // Each unordered pair is enough, and a geometry need not
                        // be weighed against itself.
                        if candidate.slot <= slot {
                            continue;
                        }
                        let other = members[candidate.slot];
                        let same = self.buffer[index]
                            .feature
                            .geometry
                            .equal(&self.buffer[other].feature.geometry, tolerance)
                            .map_err(|e| {
                                GeometryProcessorError::GeometryIdentifier(format!(
                                    "Cannot tell whether two geometries occupy the same space: {e}"
                                ))
                            })?;
                        if same {
                            union_find.union(slot, candidate.slot);
                        }
                    }
                }
                for (slot, &index) in members.iter().enumerate() {
                    root_of.insert(index, members[union_find.find(slot)]);
                }
            }

            // Number the shapes by the arrival of their first feature, so the
            // identifiers do not depend on iteration order.
            let mut identifier_of_root: HashMap<usize, usize> = HashMap::new();
            let mut members_of_root: HashMap<usize, Vec<usize>> = HashMap::new();
            for &index in indices {
                let Some(&root) = root_of.get(&index) else {
                    continue;
                };
                let next = identifier_of_root.len();
                identifier_of_root.entry(root).or_insert(next);
                members_of_root.entry(root).or_default().push(index);
            }

            for &index in indices {
                let Some(&root) = root_of.get(&index) else {
                    continue;
                };
                let matched_ids = match &self.params.id_attribute {
                    None => Vec::new(),
                    Some(attribute) => {
                        let own = self.read_id(index, attribute)?;
                        let mut matched: Vec<String> = Vec::new();
                        for &member in &members_of_root[&root] {
                            let id = self.read_id(member, attribute)?;
                            if id != own && !matched.contains(&id) {
                                matched.push(id);
                            }
                        }
                        matched
                    }
                };
                verdicts[index] = Some(Verdict {
                    identifier: identifier_of_root[&root],
                    matched_ids,
                });
            }
        }

        Ok(verdicts)
    }

    /// The box one binned feature's geometry sits in.
    fn envelope(&self, index: usize) -> &Aabb {
        self.buffer[index].envelope.as_ref().expect("binned")
    }

    /// The ID attribute's value on one buffered feature.
    fn read_id(&self, index: usize, attribute: &Attribute) -> Result<String, BoxedError> {
        self.buffer[index]
            .feature
            .attributes
            .get(attribute)
            .and_then(|value| value.as_string())
            .ok_or_else(|| {
                GeometryProcessorError::GeometryIdentifier(format!(
                    "Feature has no string ID attribute `{attribute}`"
                ))
                .into()
            })
    }
}

/// One geometry's bounding box in the tree that pairs candidates up.
struct BoxEntry {
    envelope: AABB<[f64; 3]>,
    /// Position of the geometry within its bin.
    slot: usize,
}

impl RTreeObject for BoxEntry {
    type Envelope = AABB<[f64; 3]>;

    fn envelope(&self) -> Self::Envelope {
        self.envelope
    }
}

/// Which embedding a box came from: geometries of different embeddings are
/// never weighed against one another.
fn embedding(aabb: &Aabb) -> u8 {
    match aabb {
        Aabb::D2 { .. } => 2,
        Aabb::D3 { .. } => 3,
    }
}

/// One box grown by `distance` on every side, as a 3D box so both embeddings
/// can share a tree. A 2D box is read at zero elevation, which is sound because
/// only boxes of one embedding ever meet in a tree.
fn box_of(aabb: &Aabb, distance: f64) -> AABB<[f64; 3]> {
    let (min, max) = match aabb {
        Aabb::D2 { min, max } => ([min[0], min[1], 0.0], [max[0], max[1], 0.0]),
        Aabb::D3 { min, max } => (*min, *max),
    };
    AABB::from_corners(
        [min[0] - distance, min[1] - distance, min[2] - distance],
        [max[0] + distance, max[1] + distance, max[2] + distance],
    )
}

/// Union-find over the shapes of one bin, indexed by position within it.
#[derive(Debug)]
struct UnionFind {
    parent: Vec<usize>,
    rank: Vec<u32>,
}

impl UnionFind {
    fn new(nodes: usize) -> Self {
        Self {
            parent: (0..nodes).collect(),
            rank: vec![0; nodes],
        }
    }

    fn find(&mut self, node: usize) -> usize {
        let mut root = node;
        while self.parent[root] != root {
            root = self.parent[root];
        }
        let mut current = node;
        while self.parent[current] != root {
            let next = self.parent[current];
            self.parent[current] = root;
            current = next;
        }
        root
    }

    fn union(&mut self, a: usize, b: usize) {
        let (root_a, root_b) = (self.find(a), self.find(b));
        if root_a == root_b {
            return;
        }
        match self.rank[root_a].cmp(&self.rank[root_b]) {
            std::cmp::Ordering::Less => self.parent[root_a] = root_b,
            std::cmp::Ordering::Greater => self.parent[root_b] = root_a,
            std::cmp::Ordering::Equal => {
                self.parent[root_b] = root_a;
                self.rank[root_a] += 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::utils::create_default_execute_context;
    use reearth_flow_geometry::coordinate::CoordinateFrame;
    use reearth_flow_geometry::point::Point3D;
    use reearth_flow_geometry::polygon::Polygon3D;
    use reearth_flow_geometry::Euclidean3DGeometry;
    use reearth_flow_runtime::forwarder::NoopChannelForwarder;

    /// A closed square ring in the `z = 0` plane, one metre on a side.
    const SQUARE: [[f64; 3]; 5] = [
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0],
    ];

    fn face(ring: Vec<[f64; 3]>) -> Geometry {
        Geometry::Euclidean3D(Euclidean3DGeometry::Polygon(Box::new(
            Polygon3D::from_rings(
                CoordinateFrame::Euclidean,
                ring,
                Vec::<Vec<[f64; 3]>>::new(),
            ),
        )))
    }

    fn point(position: [f64; 3]) -> Geometry {
        Geometry::Euclidean3D(Euclidean3DGeometry::Point(Point3D::new(
            CoordinateFrame::Euclidean,
            position,
        )))
    }

    /// A feature carrying `geometry`, its own `id`, and a `group` to bin it by.
    fn feature(id: &str, group: &str, geometry: Geometry) -> Feature {
        let mut feature = Feature::from(geometry);
        feature.insert("id", AttributeValue::String(id.to_string()));
        feature.insert("group", AttributeValue::String(group.to_string()));
        feature
    }

    fn params(tolerance: f64, group_by: Option<Vec<&str>>) -> GeometryIdentifierParam {
        GeometryIdentifierParam {
            tolerance,
            group_by: group_by.map(|keys| keys.into_iter().map(Attribute::new).collect::<Vec<_>>()),
            output_attribute: default_output_attribute(),
            id_attribute: Some(Attribute::new("id")),
            matched_ids_attribute: default_matched_ids_attribute(),
        }
    }

    /// Run the processor over `features`, returning the identifier and the
    /// matched IDs each one left with, in arrival order.
    fn identify(
        params: GeometryIdentifierParam,
        features: Vec<Feature>,
    ) -> Vec<(Option<i64>, Vec<String>)> {
        let fw = ProcessorChannelForwarder::Noop(NoopChannelForwarder::default());
        let mut processor = GeometryIdentifier {
            params,
            buffer: Vec::new(),
            groups: HashMap::new(),
        };
        for feature in &features {
            processor
                .process(create_default_execute_context(feature), &fw)
                .unwrap();
        }
        processor.finish(NodeContext::default(), &fw).unwrap();

        let ProcessorChannelForwarder::Noop(noop) = fw else {
            unreachable!("built as a noop forwarder");
        };
        let sent = noop.send_features.lock().unwrap().clone();
        assert_eq!(
            sent.len(),
            features.len(),
            "one feature in, one feature out"
        );
        sent.into_iter()
            .map(|feature| {
                let identifier = feature
                    .attributes
                    .get(&default_output_attribute())
                    .and_then(|value| match value {
                        AttributeValue::Number(number) => number.as_i64(),
                        _ => None,
                    });
                let matched = match feature.attributes.get(&default_matched_ids_attribute()) {
                    Some(AttributeValue::Array(values)) => values
                        .iter()
                        .map(|value| value.as_string().unwrap())
                        .collect(),
                    _ => Vec::new(),
                };
                (identifier, matched)
            })
            .collect()
    }

    #[test]
    fn a_ring_is_the_same_shape_re_wound_and_re_started() {
        // The same square, wound the other way and started at another vertex.
        let reversed = vec![
            [1.0, 1.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
        ];
        let out = identify(
            params(0.0, None),
            vec![
                feature("a", "g", face(SQUARE.to_vec())),
                feature("b", "g", face(reversed)),
            ],
        );

        assert_eq!(out[0].0, Some(0));
        assert_eq!(out[1].0, Some(0));
        assert_eq!(out[0].1, vec!["b".to_string()]);
        assert_eq!(out[1].1, vec!["a".to_string()]);
    }

    #[test]
    fn a_vertex_added_on_an_edge_leaves_the_shape_alone() {
        // The same square with a vertex part-way along one edge — a point of the
        // point set that was always there, now named.
        let split = vec![
            [0.0, 0.0, 0.0],
            [0.3, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0],
        ];
        let out = identify(
            params(1e-9, None),
            vec![
                feature("a", "g", face(SQUARE.to_vec())),
                feature("b", "g", face(split)),
            ],
        );

        assert_eq!(out[0].0, Some(0));
        assert_eq!(out[1].0, Some(0));
    }

    #[test]
    fn a_corner_moved_further_than_the_tolerance_is_another_shape() {
        let nudged = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.05, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0],
        ];
        let out = identify(
            params(0.01, None),
            vec![
                feature("a", "g", face(SQUARE.to_vec())),
                feature("b", "g", face(nudged.clone())),
            ],
        );
        assert_eq!(out[0].0, Some(0));
        assert_eq!(out[1].0, Some(1));
        assert!(out[0].1.is_empty());

        // The same pair, with a tolerance wide enough to swallow the move.
        let out = identify(
            params(0.1, None),
            vec![
                feature("a", "g", face(SQUARE.to_vec())),
                feature("b", "g", face(nudged)),
            ],
        );
        assert_eq!(out[0].0, Some(0));
        assert_eq!(out[1].0, Some(0));
    }

    #[test]
    fn edges_that_part_between_shared_vertices_are_another_shape() {
        // Both rings run through the square's four corners, but the second
        // crosses the middle instead of following the sides: every vertex of one
        // sits on the other, and only the points between them tell them apart.
        let crossed = vec![
            [0.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0],
        ];
        let out = identify(
            params(0.01, None),
            vec![
                feature("a", "g", face(SQUARE.to_vec())),
                feature("b", "g", face(crossed)),
            ],
        );

        assert_eq!(out[0].0, Some(0));
        assert_eq!(out[1].0, Some(1));
    }

    #[test]
    fn a_group_is_never_compared_with_another() {
        let out = identify(
            params(0.0, Some(vec!["group"])),
            vec![
                feature("a", "left", face(SQUARE.to_vec())),
                feature("b", "right", face(SQUARE.to_vec())),
            ],
        );

        // Identifiers are counted within a group, so both are the first of theirs.
        assert_eq!(out[0].0, Some(0));
        assert_eq!(out[1].0, Some(0));
        assert!(out[0].1.is_empty());
        assert!(out[1].1.is_empty());
    }

    #[test]
    fn a_position_and_a_face_are_never_the_same_shape() {
        let out = identify(
            params(10.0, None),
            vec![
                feature("a", "g", face(SQUARE.to_vec())),
                feature("b", "g", point([0.0, 0.0, 0.0])),
            ],
        );

        assert_eq!(out[0].0, Some(0));
        assert_eq!(out[1].0, Some(1));
    }

    #[test]
    fn a_feature_without_geometry_is_left_alone() {
        let out = identify(
            params(0.0, None),
            vec![
                feature("a", "g", Geometry::None),
                feature("b", "g", face(SQUARE.to_vec())),
            ],
        );

        assert_eq!(out[0].0, None);
        assert!(out[0].1.is_empty());
        assert_eq!(out[1].0, Some(0));
    }

    #[test]
    fn every_shape_of_a_chain_of_near_matches_shares_one_identifier() {
        // Being the same shape is not transitive: `a` reaches `b` and `b` reaches
        // `c`, but `a` and `c` are further apart than the tolerance. Resolving
        // into shapes takes the relation's transitive closure, so the chain
        // lands in one.
        let shifted = |dx: f64| {
            SQUARE
                .iter()
                .map(|[x, y, z]| [x + dx, *y, *z])
                .collect::<Vec<_>>()
        };
        let out = identify(
            params(0.015, None),
            vec![
                feature("a", "g", face(shifted(0.0))),
                feature("b", "g", face(shifted(0.01))),
                feature("c", "g", face(shifted(0.02))),
            ],
        );

        assert_eq!(out[0].0, Some(0));
        assert_eq!(out[1].0, Some(0));
        assert_eq!(out[2].0, Some(0));
        assert_eq!(out[0].1, vec!["b".to_string(), "c".to_string()]);
    }

    #[test]
    fn without_an_id_attribute_only_the_identifier_is_written() {
        // No ID to collect, so the matched-IDs attribute is left off entirely
        // rather than written as an empty array — and no feature is required to
        // carry an ID at all.
        let mut params = params(0.0, None);
        params.id_attribute = None;

        let fw = ProcessorChannelForwarder::Noop(NoopChannelForwarder::default());
        let mut processor = GeometryIdentifier {
            params,
            buffer: Vec::new(),
            groups: HashMap::new(),
        };
        let mut bare = Feature::from(face(SQUARE.to_vec()));
        bare.insert("group", AttributeValue::String("g".to_string()));
        for feature in [feature("a", "g", face(SQUARE.to_vec())), bare] {
            processor
                .process(create_default_execute_context(&feature), &fw)
                .unwrap();
        }
        processor.finish(NodeContext::default(), &fw).unwrap();

        let ProcessorChannelForwarder::Noop(noop) = fw else {
            unreachable!("built as a noop forwarder");
        };
        let sent = noop.send_features.lock().unwrap().clone();
        for feature in &sent {
            assert_eq!(
                feature.attributes.get(&default_output_attribute()),
                Some(&AttributeValue::Number(0.into()))
            );
            assert!(feature
                .attributes
                .get(&default_matched_ids_attribute())
                .is_none());
        }
    }
}
