//! Geometry identity: labelling the features whose geometries occupy the same space.
//!
//! Two geometries are the same shape when neither strays further than the
//! tolerance from the other: every point of one has a point of the other within
//! that distance, and the other way round. That is the Hausdorff distance
//! between the two point sets, so a shape stays itself under a re-wound ring, a
//! different starting vertex, or an extra vertex sitting on an edge.
//!
//! Being the same shape is not transitive once the tolerance is above zero, so
//! it is not on its own the equivalence the identifiers report: two geometries
//! share an identifier when a chain of same-shape steps runs between them.

use std::collections::HashMap;

use reearth_flow_geometry::coordinate::CoordinateFrame;
use reearth_flow_geometry::ops::triangulation::Cache;
use reearth_flow_geometry::ops::{Coerce, CoercionTarget};
use reearth_flow_geometry::{Euclidean2DGeometry, Euclidean3DGeometry, Geometry};
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{Context, ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, FEATURES_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, Feature};
use rstar::{PointDistance, RTree, RTreeObject, AABB};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::GeometryProcessorError;

thread_local! {
    /// Scratch for the coercion that reduces a geometry to its boundary curves.
    /// Kept off the `Processor` (which must be `Send + Sync + Clone`); one per
    /// worker thread.
    static COERCION_CACHE: std::cell::RefCell<Cache> = std::cell::RefCell::new(Cache::new());
}

/// Number of primitives above which a shape gets its own spatial index; below
/// it, scanning the primitives costs less than building and walking a tree.
const INDEX_THRESHOLD: usize = 64;

/// How many sub-segments one segment may be split into while deciding whether it
/// stays within the tolerance. The refinement below halves a sub-segment only
/// where neither exact test settles it, so this is reached only by a segment that
/// hugs the tolerance along its whole length.
const REFINEMENT_BUDGET: usize = 4096;

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

/// One buffered input feature and the shape read from its geometry.
#[derive(Debug, Clone)]
struct BufferedFeature {
    /// `None` for a feature that arrived without geometry.
    shape: Option<Shape>,
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
        let shape = Shape::of(&feature.geometry).map_err(|e| {
            GeometryProcessorError::GeometryIdentifier(format!("Cannot read the geometry: {e}"))
        })?;

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
            shape,
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

        for indices in self.groups.values() {
            // A face and the curve bounding it cover different point sets, and a
            // face and a one-member collection of that face are two ways of
            // saying the same thing; neither is identified with the other, so
            // the kinds are binned apart and never compared.
            let mut bins: HashMap<&str, Vec<usize>> = HashMap::new();
            for &index in indices {
                if let Some(shape) = &self.buffer[index].shape {
                    bins.entry(shape.kind.as_str()).or_default().push(index);
                }
            }

            // Feature index -> the index representing the shape it shares. Kept
            // across bins so identifiers are numbered once per group.
            let mut root_of: HashMap<usize, usize> = HashMap::new();
            for members in bins.values() {
                let mut union_find = UnionFind::new(members.len());
                let tree = RTree::bulk_load(
                    members
                        .iter()
                        .enumerate()
                        .map(|(slot, &index)| BoxEntry {
                            envelope: self.buffer[index]
                                .shape
                                .as_ref()
                                .expect("binned")
                                .envelope(),
                            slot,
                        })
                        .collect(),
                );
                for (slot, &index) in members.iter().enumerate() {
                    let shape = self.buffer[index].shape.as_ref().expect("binned");
                    let reach = shape.envelope_grown_by(self.params.tolerance);
                    for candidate in tree.locate_in_envelope_intersecting(&reach) {
                        // Each unordered pair is enough, and a shape need not be
                        // compared with itself.
                        if candidate.slot <= slot {
                            continue;
                        }
                        let other = self.buffer[members[candidate.slot]]
                            .shape
                            .as_ref()
                            .expect("binned");
                        if shape.same_as(other, self.params.tolerance) {
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

/// A geometry reduced to the primitives its point set is the union of: the
/// segments of every boundary ring and curve, plus each isolated position as a
/// segment of zero length.
#[derive(Debug, Clone)]
struct Shape {
    primitives: Vec<Primitive>,
    min: [f64; 3],
    max: [f64; 3],
    /// Geometries labelled differently are never identified with one another:
    /// the leaf type they are built from, and the coordinate frames they are
    /// expressed in.
    kind: String,
    /// Built only for a shape large enough for the scan to cost more than the
    /// tree; see [`INDEX_THRESHOLD`].
    index: Option<RTree<Primitive>>,
}

/// One straight piece of a shape's point set. A position is the degenerate case
/// where both ends coincide.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Primitive {
    a: [f64; 3],
    b: [f64; 3],
}

impl RTreeObject for Primitive {
    type Envelope = AABB<[f64; 3]>;

    fn envelope(&self) -> Self::Envelope {
        AABB::from_corners(self.a, self.b)
    }
}

impl PointDistance for Primitive {
    fn distance_2(&self, point: &[f64; 3]) -> f64 {
        point_primitive_distance_2(*point, self)
    }
}

/// One shape's bounding box in the tree that pairs shapes up.
struct BoxEntry {
    envelope: AABB<[f64; 3]>,
    /// Position of the shape within its bin.
    slot: usize,
}

impl RTreeObject for BoxEntry {
    type Envelope = AABB<[f64; 3]>;

    fn envelope(&self) -> Self::Envelope {
        self.envelope
    }
}

impl Shape {
    /// The shape of one geometry, or `None` when the feature carries none.
    fn of(geometry: &Geometry) -> Result<Option<Self>, String> {
        if matches!(geometry, Geometry::None) {
            return Ok(None);
        }
        let mut shape = Shape {
            primitives: Vec::new(),
            min: [f64::INFINITY; 3],
            max: [f64::NEG_INFINITY; 3],
            kind: leaf_kind(geometry).to_string(),
            index: None,
        };
        let mut frames: Vec<CoordinateFrame> = Vec::new();
        collect(&reduce_to_curves(geometry), &mut shape, &mut frames)?;
        if shape.primitives.is_empty() {
            return Ok(None);
        }
        // A frame is part of what a geometry is: the same numbers in two frames
        // are two different places.
        shape.kind.push_str(&format!("|{frames:?}"));
        if shape.primitives.len() > INDEX_THRESHOLD {
            shape.index = Some(RTree::bulk_load(shape.primitives.clone()));
        }
        Ok(Some(shape))
    }

    fn push_position(&mut self, position: [f64; 3]) {
        self.push_primitive(Primitive {
            a: position,
            b: position,
        });
    }

    /// Push the segments between consecutive coordinates of a chain. A chain of
    /// one coordinate contributes that position.
    fn push_chain(&mut self, coords: impl IntoIterator<Item = [f64; 3]>) {
        let mut previous: Option<[f64; 3]> = None;
        for coord in coords {
            match previous {
                None => self.push_position(coord),
                Some(previous) => self.push_primitive(Primitive {
                    a: previous,
                    b: coord,
                }),
            }
            previous = Some(coord);
        }
    }

    fn push_primitive(&mut self, primitive: Primitive) {
        // A chain's first coordinate enters as a position and is covered again by
        // the segment that follows it, so drop the position once it is.
        if let Some(last) = self.primitives.last() {
            if last.a == last.b && last.a == primitive.a {
                self.primitives.pop();
            }
        }
        for axis in 0..3 {
            self.min[axis] = self.min[axis].min(primitive.a[axis]).min(primitive.b[axis]);
            self.max[axis] = self.max[axis].max(primitive.a[axis]).max(primitive.b[axis]);
        }
        self.primitives.push(primitive);
    }

    fn envelope(&self) -> AABB<[f64; 3]> {
        AABB::from_corners(self.min, self.max)
    }

    fn envelope_grown_by(&self, distance: f64) -> AABB<[f64; 3]> {
        AABB::from_corners(
            [
                self.min[0] - distance,
                self.min[1] - distance,
                self.min[2] - distance,
            ],
            [
                self.max[0] + distance,
                self.max[1] + distance,
                self.max[2] + distance,
            ],
        )
    }

    /// Whether the two shapes occupy the same space: neither strays further than
    /// `tolerance` from the other.
    fn same_as(&self, other: &Shape, tolerance: f64) -> bool {
        self.kind == other.kind && self.covers(other, tolerance) && other.covers(self, tolerance)
    }

    /// Whether every point of `other` lies within `tolerance` of this shape.
    fn covers(&self, other: &Shape, tolerance: f64) -> bool {
        other
            .primitives
            .iter()
            .all(|primitive| self.covers_segment(primitive.a, primitive.b, tolerance))
    }

    /// Whether every point of the straight segment from `a` to `b` lies within
    /// `tolerance` of this shape.
    ///
    /// Two exact tests settle a segment without looking inside it, and a segment
    /// neither settles is halved and retried. The distance to a shape is
    /// 1-Lipschitz, which bounds how far it can climb between the two ends; and a
    /// primitive is convex, so one primitive holding both ends within the
    /// tolerance holds everything between them too.
    fn covers_segment(&self, a: [f64; 3], b: [f64; 3], tolerance: f64) -> bool {
        let mut pending = vec![(a, b)];
        let mut budget = REFINEMENT_BUDGET;
        while let Some((p, q)) = pending.pop() {
            let dp = self.distance(p);
            let dq = self.distance(q);
            if dp > tolerance || dq > tolerance {
                return false;
            }
            if (dp + dq + distance(p, q)) / 2.0 <= tolerance {
                continue;
            }
            if self.holds_both(p, q, tolerance) {
                continue;
            }
            // Out of refinement: the ends are within the tolerance and the rest
            // of this sub-segment goes undecided rather than failing the shape.
            if budget == 0 {
                continue;
            }
            budget -= 1;
            let mid = [
                (p[0] + q[0]) / 2.0,
                (p[1] + q[1]) / 2.0,
                (p[2] + q[2]) / 2.0,
            ];
            pending.push((p, mid));
            pending.push((mid, q));
        }
        true
    }

    /// Distance from `point` to the nearest primitive.
    fn distance(&self, point: [f64; 3]) -> f64 {
        let squared = match &self.index {
            Some(index) => index
                .nearest_neighbor(&point)
                .map(|primitive| point_primitive_distance_2(point, primitive))
                .unwrap_or(f64::INFINITY),
            None => self
                .primitives
                .iter()
                .map(|primitive| point_primitive_distance_2(point, primitive))
                .fold(f64::INFINITY, f64::min),
        };
        squared.sqrt()
    }

    /// Whether one primitive alone holds both `p` and `q` within `tolerance`.
    fn holds_both(&self, p: [f64; 3], q: [f64; 3], tolerance: f64) -> bool {
        let limit = tolerance * tolerance;
        let holds = |primitive: &Primitive| point_primitive_distance_2(q, primitive) <= limit;
        match &self.index {
            Some(index) => index.locate_within_distance(p, limit).any(holds),
            None => self
                .primitives
                .iter()
                .filter(|primitive| point_primitive_distance_2(p, primitive) <= limit)
                .any(holds),
        }
    }
}

/// Squared distance from a point to the nearest point of a primitive.
fn point_primitive_distance_2(point: [f64; 3], primitive: &Primitive) -> f64 {
    let (a, b) = (primitive.a, primitive.b);
    let along = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let length_2 = along[0] * along[0] + along[1] * along[1] + along[2] * along[2];
    let to_point = [point[0] - a[0], point[1] - a[1], point[2] - a[2]];
    let projection = if length_2 <= 0.0 {
        0.0
    } else {
        ((to_point[0] * along[0] + to_point[1] * along[1] + to_point[2] * along[2]) / length_2)
            .clamp(0.0, 1.0)
    };
    let offset = [
        to_point[0] - projection * along[0],
        to_point[1] - projection * along[1],
        to_point[2] - projection * along[2],
    ];
    offset[0] * offset[0] + offset[1] * offset[1] + offset[2] * offset[2]
}

fn distance(a: [f64; 3], b: [f64; 3]) -> f64 {
    let d = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
}

/// Re-represent a geometry as the curves bounding it, leaving what already is a
/// curve or a position as it stands.
fn reduce_to_curves(geometry: &Geometry) -> Geometry {
    let mut geometry = geometry.clone();
    let coerced = COERCION_CACHE
        .with(|cache| geometry.coerce(CoercionTarget::LineString, &mut cache.borrow_mut()));
    // Coercion reports `Err` when nothing changed, which leaves the geometry as
    // it arrived — a curve or a position, both of which are already reduced.
    coerced.unwrap_or(geometry)
}

/// The leaf type a geometry is built from, before it is reduced to curves.
fn leaf_kind(geometry: &Geometry) -> &'static str {
    match geometry {
        Geometry::None => "none",
        Geometry::GeometryCollection(_) => "geometryCollection",
        Geometry::Euclidean2D(geometry) => match geometry {
            Euclidean2DGeometry::Point(_) => "2d/point",
            Euclidean2DGeometry::LineString(_) => "2d/lineString",
            Euclidean2DGeometry::Polygon(_) => "2d/polygon",
            Euclidean2DGeometry::PolygonMesh(_) => "2d/polygonMesh",
            Euclidean2DGeometry::TriangularMesh(_) => "2d/triangularMesh",
            Euclidean2DGeometry::Collection(_) => "2d/collection",
        },
        Geometry::Euclidean3D(geometry) => match geometry {
            Euclidean3DGeometry::Point(_) => "3d/point",
            Euclidean3DGeometry::PointCloud(_) => "3d/pointCloud",
            Euclidean3DGeometry::LineString(_) => "3d/lineString",
            Euclidean3DGeometry::Polygon(_) => "3d/polygon",
            Euclidean3DGeometry::PolygonMesh(_) => "3d/polygonMesh",
            Euclidean3DGeometry::TriangularMesh(_) => "3d/triangularMesh",
            Euclidean3DGeometry::Solid(_) => "3d/solid",
            Euclidean3DGeometry::Csg(_) => "3d/csg",
            Euclidean3DGeometry::Collection(_) => "3d/collection",
        },
    }
}

/// Read the primitives out of a geometry already reduced to curves and positions.
fn collect(
    geometry: &Geometry,
    shape: &mut Shape,
    frames: &mut Vec<CoordinateFrame>,
) -> Result<(), String> {
    match geometry {
        Geometry::None => Ok(()),
        Geometry::Euclidean2D(geometry) => collect_2d(geometry, shape, frames),
        Geometry::Euclidean3D(geometry) => collect_3d(geometry, shape, frames),
        Geometry::GeometryCollection(collection) => collection
            .members()
            .iter()
            .try_for_each(|member| collect(member, shape, frames)),
    }
}

fn collect_2d(
    geometry: &Euclidean2DGeometry,
    shape: &mut Shape,
    frames: &mut Vec<CoordinateFrame>,
) -> Result<(), String> {
    match geometry {
        Euclidean2DGeometry::Point(point) => {
            push_frame(frames, point.frame());
            let [x, y] = point.position();
            shape.push_position([x, y, 0.0]);
            Ok(())
        }
        Euclidean2DGeometry::LineString(line) => {
            push_frame(frames, line.frame());
            // A 2.5D chain lies at its own elevation; one without lies at zero.
            let elevation = line.elevation().unwrap_or(0.0);
            shape.push_chain(line.coords().iter().map(|&[x, y]| [x, y, elevation]));
            Ok(())
        }
        Euclidean2DGeometry::Collection(collection) => collection
            .members()
            .iter()
            .try_for_each(|member| collect_2d(member, shape, frames)),
        other => Err(format!(
            "no point set can be read from `{}`",
            leaf_kind(&Geometry::Euclidean2D(other.clone()))
        )),
    }
}

fn collect_3d(
    geometry: &Euclidean3DGeometry,
    shape: &mut Shape,
    frames: &mut Vec<CoordinateFrame>,
) -> Result<(), String> {
    match geometry {
        Euclidean3DGeometry::Point(point) => {
            push_frame(frames, point.frame());
            shape.push_position(point.position());
            Ok(())
        }
        Euclidean3DGeometry::LineString(line) => {
            push_frame(frames, line.frame());
            shape.push_chain(line.coords().iter().copied());
            Ok(())
        }
        Euclidean3DGeometry::Collection(collection) => collection
            .members()
            .iter()
            .try_for_each(|member| collect_3d(member, shape, frames)),
        other => Err(format!(
            "no point set can be read from `{}`",
            leaf_kind(&Geometry::Euclidean3D(other.clone()))
        )),
    }
}

/// Record a frame the shape's coordinates are expressed in, once per distinct
/// frame and in the order they were met.
fn push_frame(frames: &mut Vec<CoordinateFrame>, frame: &CoordinateFrame) {
    if !frames.contains(frame) {
        frames.push(frame.clone());
    }
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
}
