//! Boundary-edge extraction for triangulated PLATEAU surfaces.
//!
//! A watertight triangulated surface uses every interior edge exactly twice,
//! once from each of the two triangles that meet along it. An edge used once is
//! either on the outside of the surface or the mark of a hole in the mesh, so
//! collecting them is how the flood-zone and relief quality checks find missing
//! or misaligned triangles. The outer boundary is separated from the genuine
//! defects downstream, by testing each edge against the surface's own outline.
//!
//! Matching is exact, not tolerant: two triangles either name the same vertex or
//! they do not, and a near-miss is itself the defect being looked for. The
//! [`coordinatePrecision`](UnsharedEdgeExtractorParam::coordinate_precision)
//! parameter exists for input that was written with more digits than it
//! measured, where rounding restores the equality the data intended.

use std::collections::HashMap;

use once_cell::sync::Lazy;
use reearth_flow_geometry::coordinate::CoordinateFrame;
use reearth_flow_geometry::line_string::{LineString2D, LineString3D};
use reearth_flow_geometry::polygon::{Polygon2D, Polygon3D};
use reearth_flow_geometry::{
    Euclidean2DGeometry, Euclidean3DGeometry, Geometry, GeometryCollection,
};
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, FEATURES_PORT, REJECTED_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, Feature};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::Plateau6ProcessorError;
use crate::common::PlateauProfile;

static UNSHARED_PORT: Lazy<Port> = Lazy::new(|| Port::new("unshared"));

#[derive(Debug, Clone)]
pub(crate) struct UnsharedEdgeExtractorFactory {
    name: String,
}

impl UnsharedEdgeExtractorFactory {
    pub(crate) fn new(profile: &PlateauProfile) -> Self {
        Self {
            name: profile.action_name("UnsharedEdgeExtractor"),
        }
    }
}

impl ProcessorFactory for UnsharedEdgeExtractorFactory {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        "Collects the edges of the incoming faces that are used exactly once, emitting each as a \
         two-point line carrying the attributes of the face it came from. Endpoints must match \
         exactly, so faces should already share a projected coordinate frame."
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(UnsharedEdgeExtractorParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["PLATEAU"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![FEATURES_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![UNSHARED_PORT.clone(), REJECTED_PORT.clone()]
    }

    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let params: UnsharedEdgeExtractorParam = match with {
            Some(with) => {
                let value = serde_json::to_value(with).map_err(|e| {
                    Plateau6ProcessorError::UnsharedEdgeExtractorFactory(format!(
                        "Failed to serialize `with` parameter: {e}"
                    ))
                })?;
                serde_json::from_value(value).map_err(|e| {
                    Plateau6ProcessorError::UnsharedEdgeExtractorFactory(format!(
                        "Failed to deserialize `with` parameter: {e}"
                    ))
                })?
            }
            None => UnsharedEdgeExtractorParam::default(),
        };
        Ok(Box::new(UnsharedEdgeExtractor {
            group_by: params.group_by,
            precision: params.coordinate_precision,
            groups: HashMap::new(),
            received: 0,
        }))
    }
}

/// # Unshared Edge Extractor Parameters
///
/// Which faces are matched against each other, and how exactly their endpoints
/// have to agree.
#[derive(Serialize, Deserialize, Debug, Clone, Default, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct UnsharedEdgeExtractorParam {
    /// # Group By
    /// Attributes whose values partition the faces: an edge is only ever matched
    /// against edges of faces with the same values. Empty (the default) matches
    /// every face against every other.
    #[serde(default)]
    group_by: Vec<Attribute>,
    /// # Coordinate Precision
    /// Decimal places each coordinate is rounded to before its endpoints are
    /// compared and written out. Omitted (the default) compares the coordinates
    /// as they arrive, which is what a surface whose triangles were written from
    /// one set of vertices needs.
    #[serde(default)]
    coordinate_precision: Option<CoordinatePrecision>,
}

/// Decimal places per axis. An axis left out is not rounded.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CoordinatePrecision {
    /// # X
    /// Decimal places the first horizontal coordinate is rounded to.
    #[serde(default)]
    x: Option<u32>,
    /// # Y
    /// Decimal places the second horizontal coordinate is rounded to.
    #[serde(default)]
    y: Option<u32>,
    /// # Z
    /// Decimal places the vertical coordinate is rounded to.
    #[serde(default)]
    z: Option<u32>,
}

impl CoordinatePrecision {
    /// `coord` with each axis rounded to the places configured for it.
    fn apply(&self, coord: [f64; 3]) -> [f64; 3] {
        [
            round_to(coord[0], self.x),
            round_to(coord[1], self.y),
            round_to(coord[2], self.z),
        ]
    }
}

/// `value` rounded to `places` decimal places, or unchanged when no places are
/// configured for its axis.
fn round_to(value: f64, places: Option<u32>) -> f64 {
    match places {
        None => value,
        Some(places) => {
            let scale = 10f64.powi(places as i32);
            (value * scale).round() / scale
        }
    }
}

/// An edge's identity: its two endpoints in a fixed order, so the same edge
/// traversed in opposite directions by the two faces that share it hashes the
/// same. Coordinates are compared as bit patterns rather than as floats, with
/// `-0.0` folded onto `+0.0`, because that is the only way to key a map on them.
/// The embedding is part of the key: a 2D face's edge is not the same edge as a
/// 3D one lying over it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct EdgeKey {
    three_dimensional: bool,
    bits: [u64; 6],
}

/// One endpoint's coordinates, always carried as three components; a 2D
/// endpoint leaves the third at zero and is told apart by `three_dimensional`.
type Endpoint = [f64; 3];

/// An edge seen an odd number of times so far, with everything needed to write
/// it out if it is still unmatched when the input ends.
struct PendingEdge {
    endpoints: [Endpoint; 2],
    three_dimensional: bool,
    frame: CoordinateFrame,
    /// Position in the input, so the output order is the order the edges were
    /// first met rather than the map's iteration order.
    sequence: usize,
    /// The face that contributed the edge, whose attributes the emitted edge
    /// inherits.
    feature: Feature,
}

#[derive(Debug)]
pub(crate) struct UnsharedEdgeExtractor {
    group_by: Vec<Attribute>,
    precision: Option<CoordinatePrecision>,
    /// Per group, the edges still seen an odd number of times. An edge met a
    /// second time is removed, so what remains at the end is exactly the edges
    /// used once.
    groups: HashMap<Vec<AttributeValue>, HashMap<EdgeKey, PendingEdge>>,
    received: usize,
}

impl std::fmt::Debug for PendingEdge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PendingEdge")
            .field("endpoints", &self.endpoints)
            .field("sequence", &self.sequence)
            .finish_non_exhaustive()
    }
}

impl Clone for UnsharedEdgeExtractor {
    /// The accumulated state belongs to one run, so a clone starts empty the way
    /// the other accumulating actions' do.
    fn clone(&self) -> Self {
        Self {
            group_by: self.group_by.clone(),
            precision: self.precision,
            groups: HashMap::new(),
            received: 0,
        }
    }
}

impl Processor for UnsharedEdgeExtractor {
    /// Every edge has to be seen before any of them can be known to be
    /// unmatched, so nothing is emitted until the input ends.
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
        let mut rings = Vec::new();
        if !collect_rings(&ctx.feature.geometry, &mut rings) {
            ctx.event_hub.debug_log(
                Some(ctx.error_span()),
                "unshared edge rejected: the geometry carries no face to take edges from"
                    .to_string(),
            );
            fw.send(ctx.new_with_feature_and_port(ctx.feature.clone(), REJECTED_PORT.clone()));
            return Ok(());
        }

        let key: Vec<AttributeValue> = self
            .group_by
            .iter()
            .map(|attribute| {
                ctx.feature
                    .attributes
                    .get(attribute)
                    .cloned()
                    .unwrap_or(AttributeValue::Null)
            })
            .collect();
        let group = self.groups.entry(key).or_default();

        for ring in rings {
            // Consecutive stored vertices only: a ring written without its
            // closing vertex is missing that edge, and inventing it here would
            // paper over the very defect the check reports.
            for pair in ring.coords.windows(2) {
                let a = self.precision.map_or(pair[0], |p| p.apply(pair[0]));
                let b = self.precision.map_or(pair[1], |p| p.apply(pair[1]));
                let endpoints = order_endpoints(a, b);
                let edge = EdgeKey {
                    three_dimensional: ring.three_dimensional,
                    bits: [
                        coord_bits(endpoints[0][0]),
                        coord_bits(endpoints[0][1]),
                        coord_bits(endpoints[0][2]),
                        coord_bits(endpoints[1][0]),
                        coord_bits(endpoints[1][1]),
                        coord_bits(endpoints[1][2]),
                    ],
                };
                if group.remove(&edge).is_none() {
                    group.insert(
                        edge,
                        PendingEdge {
                            endpoints,
                            three_dimensional: ring.three_dimensional,
                            frame: ring.frame.clone(),
                            sequence: self.received,
                            feature: ctx.feature.clone(),
                        },
                    );
                }
                self.received += 1;
            }
        }
        Ok(())
    }

    fn finish(
        &mut self,
        ctx: NodeContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let mut pending: Vec<PendingEdge> = std::mem::take(&mut self.groups)
            .into_values()
            .flat_map(|group| group.into_values())
            .collect();
        pending.sort_by_key(|edge| edge.sequence);

        for edge in pending {
            let geometry =
                if edge.three_dimensional {
                    Geometry::Euclidean3D(Euclidean3DGeometry::LineString(
                        LineString3D::from_coords(edge.frame, edge.endpoints),
                    ))
                } else {
                    Geometry::Euclidean2D(Euclidean2DGeometry::LineString(
                        LineString2D::from_coords(edge.frame, edge.endpoints.map(|c| [c[0], c[1]])),
                    ))
                };
            let mut feature = edge.feature;
            feature.set_geometry(geometry);
            fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                &ctx,
                feature,
                UNSHARED_PORT.clone(),
            ));
        }
        Ok(())
    }

    fn name(&self) -> &str {
        "Unshared Edge Extractor"
    }
}

/// One ring's stored coordinates, lifted to three components so 2D and 3D faces
/// take the same path through the matcher.
struct Ring {
    coords: Vec<Endpoint>,
    three_dimensional: bool,
    frame: CoordinateFrame,
}

/// Append every face ring of `geometry` to `rings`, recursing into collections.
/// Returns whether the geometry held any face at all: a geometry that held none
/// has no edges to contribute and is reported rather than silently consumed.
fn collect_rings(geometry: &Geometry, rings: &mut Vec<Ring>) -> bool {
    match geometry {
        Geometry::None => false,
        Geometry::Euclidean2D(g) => collect_rings_2d(g, rings),
        Geometry::Euclidean3D(g) => collect_rings_3d(g, rings),
        Geometry::GeometryCollection(c) => collect_members(c, rings),
    }
}

fn collect_members(collection: &GeometryCollection, rings: &mut Vec<Ring>) -> bool {
    let mut found = false;
    for member in collection.members() {
        found |= collect_rings(member, rings);
    }
    found
}

fn collect_rings_2d(geometry: &Euclidean2DGeometry, rings: &mut Vec<Ring>) -> bool {
    match geometry {
        Euclidean2DGeometry::Polygon(polygon) => {
            push_polygon_2d(polygon, rings);
            true
        }
        Euclidean2DGeometry::Collection(collection) => {
            let mut found = false;
            for member in collection.members() {
                found |= collect_rings_2d(member, rings);
            }
            found
        }
        _ => false,
    }
}

fn collect_rings_3d(geometry: &Euclidean3DGeometry, rings: &mut Vec<Ring>) -> bool {
    match geometry {
        Euclidean3DGeometry::Polygon(polygon) => {
            push_polygon_3d(polygon, rings);
            true
        }
        Euclidean3DGeometry::Collection(collection) => {
            let mut found = false;
            for member in collection.members() {
                found |= collect_rings_3d(member, rings);
            }
            found
        }
        _ => false,
    }
}

fn push_polygon_2d(polygon: &Polygon2D, rings: &mut Vec<Ring>) {
    let z = polygon.elevation().unwrap_or(0.0);
    for ring in std::iter::once(polygon.exterior()).chain(polygon.interiors()) {
        rings.push(Ring {
            coords: ring.iter().map(|c| [c[0], c[1], z]).collect(),
            three_dimensional: false,
            frame: polygon.frame().clone(),
        });
    }
}

fn push_polygon_3d(polygon: &Polygon3D, rings: &mut Vec<Ring>) {
    for ring in std::iter::once(polygon.exterior()).chain(polygon.interiors()) {
        rings.push(Ring {
            coords: ring.to_vec(),
            three_dimensional: true,
            frame: polygon.frame().clone(),
        });
    }
}

/// The two endpoints in ascending coordinate order, so an edge hashes the same
/// whichever of its two faces traversed it.
fn order_endpoints(a: Endpoint, b: Endpoint) -> [Endpoint; 2] {
    let ordered = a
        .iter()
        .zip(b.iter())
        .find_map(|(x, y)| match x.total_cmp(y) {
            std::cmp::Ordering::Equal => None,
            other => Some(other),
        })
        .is_none_or(|order| order == std::cmp::Ordering::Less);
    if ordered {
        [a, b]
    } else {
        [b, a]
    }
}

/// A coordinate's bit pattern, with `-0.0` folded onto `+0.0` so the two hash
/// and compare as the one position they are.
fn coord_bits(value: f64) -> u64 {
    (value + 0.0).to_bits()
}

#[cfg(test)]
mod tests {
    use super::*;
    use reearth_flow_runtime::forwarder::NoopChannelForwarder;
    use reearth_flow_types::Feature;

    /// A triangle, closed, in the Euclidean frame.
    fn triangle(ring: &[[f64; 3]]) -> Geometry {
        let mut coords = ring.to_vec();
        coords.push(ring[0]);
        Geometry::Euclidean3D(Euclidean3DGeometry::Polygon(Box::new(
            Polygon3D::from_rings(
                CoordinateFrame::Euclidean,
                coords,
                Vec::<Vec<[f64; 3]>>::new(),
            ),
        )))
    }

    /// Feed every triangle through one extractor and return the edges it emits,
    /// each as its two endpoints.
    fn unshared(
        mut processor: UnsharedEdgeExtractor,
        features: Vec<Feature>,
    ) -> Vec<[[f64; 3]; 2]> {
        let fw = ProcessorChannelForwarder::Noop(NoopChannelForwarder::default());
        for feature in features {
            let ctx = crate::tests::utils::create_default_execute_context(feature.clone());
            processor.process(ctx, &fw).unwrap();
        }
        processor
            .finish(crate::tests::utils::create_default_node_context(), &fw)
            .unwrap();

        let ProcessorChannelForwarder::Noop(noop) = fw else {
            unreachable!("the forwarder is the one built above");
        };
        let sent = noop.send_features.lock().unwrap();
        sent.iter()
            .map(|feature| {
                let Geometry::Euclidean3D(Euclidean3DGeometry::LineString(line)) =
                    &*feature.geometry
                else {
                    panic!("expected a 3D line, got {:?}", feature.geometry);
                };
                let coords = line.coords();
                [coords[0], coords[1]]
            })
            .collect()
    }

    fn extractor() -> UnsharedEdgeExtractor {
        UnsharedEdgeExtractor {
            group_by: Vec::new(),
            precision: None,
            groups: HashMap::new(),
            received: 0,
        }
    }

    /// Two triangles meeting along the diagonal of a square: the diagonal is
    /// used twice and cancels, leaving the square's four outer edges.
    #[test]
    fn a_shared_edge_cancels_and_the_outline_survives() {
        let left = Feature::from(triangle(&[
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
        ]));
        let right = Feature::from(triangle(&[
            [0.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ]));
        let edges = unshared(extractor(), vec![left, right]);
        assert_eq!(edges.len(), 4);
        assert!(
            !edges
                .iter()
                .any(|edge| edge.contains(&[0.0, 0.0, 0.0]) && edge.contains(&[1.0, 1.0, 0.0])),
            "the shared diagonal should have cancelled, got {edges:?}"
        );
    }

    /// The defect the check exists for: the second triangle's vertex is a
    /// millimetre off, so neither half of what should be one edge finds a
    /// partner and both are reported.
    #[test]
    fn a_near_miss_does_not_cancel() {
        let left = Feature::from(triangle(&[
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
        ]));
        let right = Feature::from(triangle(&[
            [0.0, 0.0, 0.0],
            [1.0, 1.001, 0.0],
            [0.0, 1.0, 0.0],
        ]));
        let edges = unshared(extractor(), vec![left, right]);
        assert_eq!(edges.len(), 6);
    }

    /// Height is part of the match: two triangles whose footprints agree but
    /// whose shared edge sits at different elevations do not cancel.
    #[test]
    fn an_edge_at_a_different_height_does_not_cancel() {
        let low = Feature::from(triangle(&[
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
        ]));
        let high = Feature::from(triangle(&[
            [0.0, 0.0, 0.0],
            [1.0, 1.0, 5.0],
            [0.0, 1.0, 0.0],
        ]));
        let edges = unshared(extractor(), vec![low, high]);
        assert_eq!(edges.len(), 6);
    }

    /// Rounding is what lets input written with more digits than it measured
    /// match: at four decimal places the two vertices become the same point.
    #[test]
    fn rounding_makes_a_sub_tolerance_difference_cancel() {
        let mut processor = extractor();
        processor.precision = Some(CoordinatePrecision {
            x: Some(4),
            y: Some(4),
            z: Some(4),
        });
        let left = Feature::from(triangle(&[
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
        ]));
        let right = Feature::from(triangle(&[
            [0.0, 0.0, 0.0],
            [1.0, 1.000_001, 0.0],
            [0.0, 1.0, 0.0],
        ]));
        let edges = unshared(processor, vec![left, right]);
        assert_eq!(edges.len(), 4);
    }

    /// Faces in different groups never see each other, so the shared diagonal
    /// stays in both.
    #[test]
    fn faces_in_different_groups_do_not_cancel() {
        let mut processor = extractor();
        processor.group_by = vec![Attribute::new("scale")];
        let mut left = Feature::from(triangle(&[
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
        ]));
        left.insert("scale", AttributeValue::String("l1".to_string()));
        let mut right = Feature::from(triangle(&[
            [0.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ]));
        right.insert("scale", AttributeValue::String("l2".to_string()));
        let edges = unshared(processor, vec![left, right]);
        assert_eq!(edges.len(), 6);
    }

    /// An emitted edge carries the attributes of the face that contributed it,
    /// so the per-file counts downstream can tell which GML it came from.
    #[test]
    fn an_emitted_edge_keeps_the_face_attributes() {
        let mut feature = Feature::from(triangle(&[
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
        ]));
        feature.insert("filename", AttributeValue::String("a.gml".to_string()));

        let fw = ProcessorChannelForwarder::Noop(NoopChannelForwarder::default());
        let mut processor = extractor();
        let ctx = crate::tests::utils::create_default_execute_context(feature.clone());
        processor.process(ctx, &fw).unwrap();
        processor
            .finish(crate::tests::utils::create_default_node_context(), &fw)
            .unwrap();

        let ProcessorChannelForwarder::Noop(noop) = fw else {
            unreachable!("the forwarder is the one built above");
        };
        let sent = noop.send_features.lock().unwrap();
        assert_eq!(sent.len(), 3);
        for edge in sent.iter() {
            assert_eq!(
                edge.attributes.get(&Attribute::new("filename")),
                Some(&AttributeValue::String("a.gml".to_string()))
            );
        }
        let ports = noop.send_ports.lock().unwrap();
        assert!(ports.iter().all(|port| *port == *UNSHARED_PORT));
    }

    /// A ring written without its closing vertex is missing that edge: the
    /// action reports the two edges it actually has rather than inventing the
    /// third, which would hide the defect.
    #[test]
    fn an_unclosed_ring_contributes_only_its_stored_edges() {
        let open = Geometry::Euclidean3D(Euclidean3DGeometry::Polygon(Box::new(
            Polygon3D::from_rings(
                CoordinateFrame::Euclidean,
                vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0]],
                Vec::<Vec<[f64; 3]>>::new(),
            ),
        )));
        let edges = unshared(extractor(), vec![Feature::from(open)]);
        assert_eq!(edges.len(), 2);
    }

    #[test]
    fn geometry_with_no_face_is_rejected() {
        let fw = ProcessorChannelForwarder::Noop(NoopChannelForwarder::default());
        let feature = Feature::from(Geometry::None);
        let ctx = crate::tests::utils::create_default_execute_context(feature.clone());
        extractor().process(ctx, &fw).unwrap();

        let ProcessorChannelForwarder::Noop(noop) = fw else {
            unreachable!("the forwarder is the one built above");
        };
        let ports = noop.send_ports.lock().unwrap();
        assert_eq!(ports.len(), 1);
        assert_eq!(ports[0], *REJECTED_PORT);
    }
}
