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

use once_cell::sync::Lazy;
use reearth_flow_geometry::ops::{Aabb, BoundingBox};
use reearth_flow_geometry::predicates::{is_comparable, Equal};
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

/// Features whose geometry this action cannot weigh, kept out of the answer
/// rather than out of the run.
static REJECTED_PORT: Lazy<Port> = Lazy::new(|| Port::new("rejected"));

#[derive(Debug, Clone, Default)]
pub(super) struct GeometryIdentifierFactory;

impl ProcessorFactory for GeometryIdentifierFactory {
    fn name(&self) -> &str {
        "Geometry Identifier"
    }

    fn description(&self) -> &str {
        "Labels every feature with an identifier shared by the features whose geometry occupies \
         the same space, up to a tolerance."
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
        vec![FEATURES_PORT.clone(), REJECTED_PORT.clone()]
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

        // Writing the matched IDs means reading an ID off every feature, so the
        // two parameters are only meaningful together.
        if params.matched_ids_attribute.is_some() && params.id_attribute.is_none() {
            return Err(GeometryProcessorError::GeometryIdentifierFactory(
                "`matchedIdsAttribute` needs `idAttribute` to say what to list".to_string(),
            )
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
    /// Attribute the identifier is written to. Features whose geometries occupy the same space
    /// carry the same value. Identifiers are counted across the whole input rather than restarted
    /// per group, so the same value always names the same set of features however they are grouped
    /// again downstream.
    #[serde(default = "default_output_attribute")]
    pub output_attribute: Attribute,

    /// # ID Attribute
    /// Attribute holding the identifier of the feature, such as its gml:id. Read only to write
    /// the matched IDs attribute, and required when that is set.
    pub id_attribute: Option<Attribute>,

    /// # Matched IDs Attribute
    /// Attribute the identifiers of the features sharing this feature's shape are written to, as
    /// an array. Every feature carrying one identifier value lists the whole set, itself included,
    /// in arrival order and without repeats. Left unwritten when omitted.
    pub matched_ids_attribute: Option<Attribute>,
}

fn default_output_attribute() -> Attribute {
    Attribute::new("_equivalence_id")
}

/// One buffered input feature and the box its geometry sits in.
#[derive(Debug, Clone)]
struct BufferedFeature {
    /// `None` for a feature whose geometry occupies nowhere — absent, or an
    /// empty container. Such a feature shares its space with nothing and is
    /// left unlabelled.
    envelope: Option<Aabb>,
    /// Whether the geometry is one this action cannot weigh. Such a feature is
    /// never binned, and leaves by the rejected port rather than failing the
    /// run for every other feature in the batch.
    rejected: bool,
    feature: Feature,
}

#[derive(Debug, Clone)]
pub(super) struct GeometryIdentifier {
    params: GeometryIdentifierParam,
    /// Buffered features in arrival order; the output preserves that order.
    buffer: Vec<BufferedFeature>,
    /// Group key -> indices into `buffer`, each in arrival order. The key holds
    /// the attribute values themselves: joining them into a string would let
    /// `["a|b", "c"]` and `["a", "b|c"]` name one group, and would make an
    /// absent attribute indistinguishable from an empty one.
    groups: HashMap<Vec<Option<AttributeValue>>, Vec<usize>>,
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
        // Asked before anything is weighed: a geometry `Equal` cannot answer for
        // is set aside here rather than failing the batch part-way through.
        let rejected = !is_comparable(&feature.geometry);
        // A geometry that bounds nothing occupies nowhere. `Equal` is what
        // decides whether two geometries match; the box only pairs candidates up.
        let envelope = match &*feature.geometry {
            _ if rejected => None,
            Geometry::None => None,
            geometry => geometry.bounding_box().ok(),
        };

        let group_key = self
            .params
            .group_by
            .as_deref()
            .unwrap_or_default()
            .iter()
            .map(|attribute| feature.attributes.get(attribute).cloned())
            .collect::<Vec<_>>();

        let index = self.buffer.len();
        self.buffer.push(BufferedFeature {
            envelope,
            rejected,
            feature: ctx.feature,
        });
        self.groups.entry(group_key).or_default().push(index);

        Ok(())
    }

    fn finish(
        &mut self,
        ctx: NodeContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let identifiers = self.resolve()?;
        let matched = self.matched_ids(&identifiers);
        let ctx: Context = ctx.as_context();

        self.groups.clear();
        for (buffered, identifier) in std::mem::take(&mut self.buffer)
            .into_iter()
            .zip(identifiers)
        {
            let port = if buffered.rejected {
                REJECTED_PORT.clone()
            } else {
                FEATURES_PORT.clone()
            };
            let mut feature = buffered.feature;
            if let Some(identifier) = identifier {
                feature.attributes_mut().insert(
                    self.params.output_attribute.clone(),
                    AttributeValue::Number(identifier.into()),
                );
                if let (Some(attribute), Some(ids)) =
                    (&self.params.matched_ids_attribute, matched.get(&identifier))
                {
                    feature
                        .attributes_mut()
                        .insert(attribute.clone(), AttributeValue::Array(ids.clone()));
                }
            }
            fw.send(ExecutorContext::new_with_context_feature_and_port(
                &ctx, feature, port,
            ));
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "Geometry Identifier"
    }
}

impl GeometryIdentifier {
    /// One identifier per buffered feature, indexed by position in `buffer`.
    /// `None` where the feature's geometry occupies nowhere and so shares its
    /// space with nothing.
    fn resolve(&self) -> Result<Vec<Option<usize>>, BoxedError> {
        // Feature index -> the index representing the space it shares. Roots are
        // buffer positions, so they stay distinct across groups.
        let mut root_of: HashMap<usize, usize> = HashMap::new();

        for indices in self.groups.values() {
            // A 2D and a 3D geometry are not a pair `Equal` will weigh — there is
            // no implicit promotion between the embeddings — so they are binned
            // apart and never put to it.
            let mut bins: HashMap<std::mem::Discriminant<Aabb>, Vec<(usize, &Aabb)>> =
                HashMap::new();
            for &index in indices {
                if let Some(envelope) = &self.buffer[index].envelope {
                    bins.entry(std::mem::discriminant(envelope))
                        .or_default()
                        .push((index, envelope));
                }
            }

            for members in bins.values() {
                let mut union_find = UnionFind::new(members.len());
                let tree = RTree::bulk_load(
                    members
                        .iter()
                        .enumerate()
                        .map(|(slot, &(_, envelope))| BoxEntry {
                            envelope: box_of(envelope, 0.0),
                            slot,
                        })
                        .collect(),
                );
                for (slot, &(index, envelope)) in members.iter().enumerate() {
                    // Only geometries whose boxes come within the tolerance of
                    // one another can occupy the same space, so the rest are
                    // never weighed.
                    let reach = box_of(envelope, self.params.tolerance);
                    for candidate in tree.locate_in_envelope_intersecting(&reach) {
                        // Each unordered pair is enough, and a geometry need not
                        // be weighed against itself.
                        if candidate.slot <= slot {
                            continue;
                        }
                        let (other, _) = members[candidate.slot];
                        let same = self.buffer[index]
                            .feature
                            .geometry
                            .equal(&self.buffer[other].feature.geometry, self.params.tolerance)
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
                for (slot, &(index, _)) in members.iter().enumerate() {
                    let (root, _) = members[union_find.find(slot)];
                    root_of.insert(index, root);
                }
            }
        }

        // Number the shapes by the arrival of their first feature, so the
        // identifiers do not depend on iteration order — and across the whole
        // input rather than restarting per group, so one value always names one
        // set of features however they are grouped again downstream.
        let mut identifier_of_root: HashMap<usize, usize> = HashMap::new();
        let mut identifiers = vec![None; self.buffer.len()];
        for (index, identifier) in identifiers.iter_mut().enumerate() {
            let Some(&root) = root_of.get(&index) else {
                continue;
            };
            let next = identifier_of_root.len();
            *identifier = Some(*identifier_of_root.entry(root).or_insert(next));
        }

        Ok(identifiers)
    }

    /// The identifiers the features of one shape carry, keyed by the shape's
    /// own identifier. Empty unless the matched IDs were asked for.
    fn matched_ids(&self, identifiers: &[Option<usize>]) -> HashMap<usize, Vec<AttributeValue>> {
        let mut listed_by_shape: HashMap<usize, Vec<AttributeValue>> = HashMap::new();
        if self.params.matched_ids_attribute.is_none() {
            return listed_by_shape;
        }
        let Some(id_attribute) = self.params.id_attribute.as_ref() else {
            return listed_by_shape;
        };
        for (buffered, identifier) in self.buffer.iter().zip(identifiers) {
            let (Some(identifier), Some(value)) =
                (identifier, buffered.feature.attributes.get(id_attribute))
            else {
                continue;
            };
            // Arrival order, without repeats: several features of one shape may
            // carry one identifier, and the list names each of them once.
            let listed = listed_by_shape.entry(*identifier).or_default();
            if !listed.contains(value) {
                listed.push(value.clone());
            }
        }
        listed_by_shape
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

/// One box grown by `distance` on every side, as a 3D box so both embeddings
/// can share a tree. A 2D box is read at zero elevation, which is sound because
/// only boxes of one embedding ever meet in a tree.
fn box_of(aabb: &Aabb, distance: f64) -> AABB<[f64; 3]> {
    let (min, max) = aabb.expanded(distance).corners_3d();
    AABB::from_corners(min, max)
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
            id_attribute: None,
            matched_ids_attribute: None,
        }
    }

    /// Run the processor over `features`, returning the identifier each one left
    /// with, in arrival order.
    fn identify(params: GeometryIdentifierParam, features: Vec<Feature>) -> Vec<Option<i64>> {
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
                feature
                    .attributes
                    .get(&default_output_attribute())
                    .and_then(|value| match value {
                        AttributeValue::Number(number) => number.as_i64(),
                        _ => None,
                    })
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

        assert_eq!(out[0], Some(0));
        assert_eq!(out[1], Some(0));
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

        assert_eq!(out[0], Some(0));
        assert_eq!(out[1], Some(0));
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
        assert_eq!(out[0], Some(0));
        assert_eq!(out[1], Some(1));

        // The same pair, with a tolerance wide enough to swallow the move.
        let out = identify(
            params(0.1, None),
            vec![
                feature("a", "g", face(SQUARE.to_vec())),
                feature("b", "g", face(nudged)),
            ],
        );
        assert_eq!(out[0], Some(0));
        assert_eq!(out[1], Some(0));
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

        assert_eq!(out[0], Some(0));
        assert_eq!(out[1], Some(1));
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

        // The same square twice, but in two groups, so never weighed against one
        // another — and identifiers are counted across the whole input, so the
        // two are told apart rather than both being the first of their group.
        assert_eq!(out[0], Some(0));
        assert_eq!(out[1], Some(1));
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

        assert_eq!(out[0], Some(0));
        assert_eq!(out[1], Some(1));
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

        assert_eq!(out[0], None);
        assert_eq!(out[1], Some(0));
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

        assert_eq!(out[0], Some(0));
        assert_eq!(out[1], Some(0));
        assert_eq!(out[2], Some(0));
    }

    /// Run the processor over `features`, returning the port each one left by
    /// paired with the identifier it carried, in arrival order.
    fn route(
        params: GeometryIdentifierParam,
        features: Vec<Feature>,
    ) -> Vec<(String, Option<i64>)> {
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
        let ports = noop.send_ports.lock().unwrap().clone();
        let sent = noop.send_features.lock().unwrap().clone();
        assert_eq!(
            sent.len(),
            features.len(),
            "one feature in, one feature out"
        );
        ports
            .into_iter()
            .zip(sent)
            .map(|(port, feature)| {
                let identifier = feature
                    .attributes
                    .get(&default_output_attribute())
                    .and_then(|value| match value {
                        AttributeValue::Number(number) => number.as_i64(),
                        _ => None,
                    });
                (port.to_string(), identifier)
            })
            .collect()
    }

    #[test]
    fn a_geometry_that_cannot_be_weighed_leaves_by_the_rejected_port() {
        // A collection has no point set of its own, so `Equal` refuses it. The
        // run carries on and the polygons either side of it are still labelled.
        let bag = Geometry::Euclidean3D(Euclidean3DGeometry::Collection(
            reearth_flow_geometry::collection::Collection3D::new(vec![]),
        ));
        let out = route(
            params(0.0, None),
            vec![
                feature("a", "g", face(SQUARE.to_vec())),
                feature("b", "g", bag),
                feature("c", "g", face(SQUARE.to_vec())),
            ],
        );

        assert_eq!(out[0], ("features".to_string(), Some(0)));
        assert_eq!(out[1], ("rejected".to_string(), None));
        assert_eq!(out[2], ("features".to_string(), Some(0)));
    }

    #[test]
    fn a_mesh_is_rejected_rather_than_bringing_the_run_down() {
        // `Equal` is unwritten for the 3D surfaces and panics rather than
        // refusing, so the action has to keep them away from it altogether.
        use reearth_flow_geometry::polygon_mesh::PolygonMesh3D;
        let polygon = Polygon3D::from_rings(
            CoordinateFrame::Euclidean,
            SQUARE.to_vec(),
            Vec::<Vec<[f64; 3]>>::new(),
        );
        let mesh = Geometry::Euclidean3D(Euclidean3DGeometry::PolygonMesh(Box::new(
            PolygonMesh3D::from_polygons(CoordinateFrame::Euclidean, [&polygon]).unwrap(),
        )));
        let out = route(
            params(0.0, None),
            vec![
                feature("a", "g", mesh.clone()),
                feature("b", "g", mesh),
                feature("c", "g", face(SQUARE.to_vec())),
            ],
        );

        assert_eq!(out[0], ("rejected".to_string(), None));
        assert_eq!(out[1], ("rejected".to_string(), None));
        assert_eq!(out[2], ("features".to_string(), Some(0)));
    }

    /// The identifiers each feature left with under `id`/`matchedIds`.
    fn matched(
        params: GeometryIdentifierParam,
        features: Vec<Feature>,
    ) -> Vec<Option<Vec<String>>> {
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
        sent.into_iter()
            .map(|feature| {
                feature
                    .attributes
                    .get(&Attribute::new("_shared_ids"))
                    .and_then(|value| match value {
                        AttributeValue::Array(values) => Some(
                            values
                                .iter()
                                .map(|value| match value {
                                    AttributeValue::String(id) => id.clone(),
                                    other => unreachable!("unexpected id {other:?}"),
                                })
                                .collect(),
                        ),
                        _ => None,
                    })
            })
            .collect()
    }

    fn params_with_ids(tolerance: f64) -> GeometryIdentifierParam {
        GeometryIdentifierParam {
            id_attribute: Some(Attribute::new("id")),
            matched_ids_attribute: Some(Attribute::new("_shared_ids")),
            ..params(tolerance, None)
        }
    }

    #[test]
    fn every_feature_of_one_shape_lists_the_whole_set() {
        // `a` and `b` occupy one space, `c` another; each names its own set, and
        // the list is what the transitive link resolver reads.
        let out = matched(
            params_with_ids(0.0),
            vec![
                feature("a", "g", face(SQUARE.to_vec())),
                feature("b", "g", face(SQUARE.to_vec())),
                feature("c", "g", point([9.0, 9.0, 9.0])),
            ],
        );

        assert_eq!(out[0], Some(vec!["a".to_string(), "b".to_string()]));
        assert_eq!(out[1], Some(vec!["a".to_string(), "b".to_string()]));
        assert_eq!(out[2], Some(vec!["c".to_string()]));
    }

    #[test]
    fn one_id_carried_by_several_features_is_listed_once() {
        // Several surfaces of one building part carry that part's ID, and the
        // list names the part once however many surfaces landed on the shape.
        let out = matched(
            params_with_ids(0.0),
            vec![
                feature("part-1", "g", face(SQUARE.to_vec())),
                feature("part-1", "g", face(SQUARE.to_vec())),
                feature("part-2", "g", face(SQUARE.to_vec())),
            ],
        );

        assert_eq!(
            out[0],
            Some(vec!["part-1".to_string(), "part-2".to_string()])
        );
        assert_eq!(out[2], out[0]);
    }

    #[test]
    fn the_matched_ids_stay_unwritten_when_not_asked_for() {
        let out = matched(
            params(0.0, None),
            vec![feature("a", "g", face(SQUARE.to_vec()))],
        );

        assert_eq!(out[0], None);
    }
}
