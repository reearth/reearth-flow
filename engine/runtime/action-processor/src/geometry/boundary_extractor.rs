use std::collections::HashMap;

use once_cell::sync::Lazy;
#[cfg(not(feature = "new-geometry"))]
use reearth_flow_geometry::types::geometry::Geometry2D;
#[cfg(not(feature = "new-geometry"))]
use reearth_flow_geometry::types::geometry::Geometry3D;
#[cfg(not(feature = "new-geometry"))]
use reearth_flow_geometry::types::line_string::{LineString2D, LineString3D};
#[cfg(not(feature = "new-geometry"))]
use reearth_flow_geometry::types::multi_line_string::{MultiLineString2D, MultiLineString3D};
#[cfg(not(feature = "new-geometry"))]
use reearth_flow_geometry::types::triangular_mesh::TriangularMesh;
#[cfg(feature = "new-geometry")]
use reearth_flow_geometry::{ops::ExtractBoundary, Geometry};
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, FEATURES_PORT, REJECTED_PORT},
};
#[cfg(not(feature = "new-geometry"))]
use reearth_flow_types::GeometryValue;
use serde_json::Value;

/// Geometry that closes on itself, or carries no extent to bound, leaves here
/// with the geometry it arrived with.
pub static NO_BOUNDARY_PORT: Lazy<Port> = Lazy::new(|| Port::new("no-boundary"));

#[derive(Debug, Clone, Default)]
pub(super) struct BoundaryExtractorFactory;

impl ProcessorFactory for BoundaryExtractorFactory {
    fn name(&self) -> &str {
        "Boundary Extractor"
    }

    fn description(&self) -> &str {
        "Replaces a geometry with its boundary: the endpoints of a curve, the boundary rings of a surface, and the bounding shells of a volume."
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        None
    }

    fn categories(&self) -> &[&'static str] {
        &["Geometry"]
    }

    fn tags(&self) -> &[&'static str] {
        &["3d", "spatial"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![FEATURES_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![
            FEATURES_PORT.clone(),
            NO_BOUNDARY_PORT.clone(),
            REJECTED_PORT.clone(),
        ]
    }

    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        _with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        Ok(Box::new(BoundaryExtractor))
    }
}

#[derive(Debug, Clone)]
struct BoundaryExtractor;

impl Processor for BoundaryExtractor {
    /// Replace the geometry with what bounds it, one dimension down: a volume
    /// with its shells, a surface with the rings around it, a curve with its two
    /// ends.
    ///
    /// Geometry bounded by nothing leaves via `no-boundary` with the geometry it
    /// arrived with, so a workflow can tell "closed" from "not a surface". A
    /// feature with no geometry, or one whose type has no boundary to give,
    /// leaves via `rejected`.
    #[cfg(feature = "new-geometry")]
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        match ctx.feature.geometry.extract_boundary() {
            // An answer, not a failure, so the feature goes on with the
            // geometry it came in with.
            Ok(Geometry::None) => {
                fw.send(
                    ctx.new_with_feature_and_port(ctx.feature.clone(), NO_BOUNDARY_PORT.clone()),
                );
            }
            Ok(boundary) => {
                let mut feature = ctx.feature.clone();
                feature.set_geometry(boundary);
                fw.send(ctx.new_with_feature_and_port(feature, FEATURES_PORT.clone()));
            }
            Err(e) => {
                // This port's normal business, so not worth a warning per feature.
                ctx.event_hub.debug_log(
                    Some(ctx.error_span()),
                    format!("boundary extraction rejected: {e}"),
                );
                fw.send(ctx.new_with_feature_and_port(ctx.feature.clone(), REJECTED_PORT.clone()));
            }
        }
        Ok(())
    }

    #[cfg(not(feature = "new-geometry"))]
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let geometry = &feature.geometry;

        if geometry.is_empty() {
            fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
            return Ok(());
        }

        let boundary = match &geometry.value {
            GeometryValue::FlowGeometry2D(geo) => {
                extract_2d_boundary(geo).map(GeometryValue::FlowGeometry2D)
            }
            GeometryValue::FlowGeometry3D(geo) => {
                extract_3d_boundary(geo).map(GeometryValue::FlowGeometry3D)
            }
            // CityGML geometry has to be converted to a plain geometry first.
            GeometryValue::None | GeometryValue::CityGmlGeometry(_) => LegacyBoundary::None,
        };

        match boundary {
            LegacyBoundary::Boundary(value) => {
                let mut new_geometry = (**geometry).clone();
                new_geometry.value = value;
                let mut new_feature = feature.clone();
                new_feature.geometry = std::sync::Arc::new(new_geometry);
                fw.send(ctx.new_with_feature_and_port(new_feature, FEATURES_PORT.clone()));
            }
            LegacyBoundary::Empty => {
                fw.send(ctx.new_with_feature_and_port(feature.clone(), NO_BOUNDARY_PORT.clone()));
            }
            LegacyBoundary::None => {
                fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
            }
        }
        Ok(())
    }

    fn finish(
        &mut self,
        _ctx: NodeContext,
        _fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "Boundary Extractor"
    }

    fn num_threads(&self) -> usize {
        16
    }
}

/// What a geometry turned out to be bounded by, so the three cases the ports
/// distinguish cannot be collapsed into one another.
#[cfg(not(feature = "new-geometry"))]
enum LegacyBoundary<T> {
    /// The geometry it is bounded by.
    Boundary(T),
    /// Bounded by nothing: it closes on itself, or has no extent to bound.
    Empty,
    /// A type with no boundary to give.
    None,
}

#[cfg(not(feature = "new-geometry"))]
impl<T> LegacyBoundary<T> {
    fn map<U>(self, f: impl FnOnce(T) -> U) -> LegacyBoundary<U> {
        match self {
            LegacyBoundary::Boundary(t) => LegacyBoundary::Boundary(f(t)),
            LegacyBoundary::Empty => LegacyBoundary::Empty,
            LegacyBoundary::None => LegacyBoundary::None,
        }
    }
}

/// The rings of every face, exterior then holes, as one geometry.
#[cfg(not(feature = "new-geometry"))]
fn rings_2d(rings: Vec<LineString2D<f64>>) -> LegacyBoundary<Geometry2D<f64>> {
    match rings.len() {
        0 => LegacyBoundary::Empty,
        1 => LegacyBoundary::Boundary(Geometry2D::LineString(rings.into_iter().next().unwrap())),
        _ => LegacyBoundary::Boundary(Geometry2D::MultiLineString(MultiLineString2D::new(rings))),
    }
}

#[cfg(not(feature = "new-geometry"))]
fn rings_3d(rings: Vec<LineString3D<f64>>) -> LegacyBoundary<Geometry3D<f64>> {
    match rings.len() {
        0 => LegacyBoundary::Empty,
        1 => LegacyBoundary::Boundary(Geometry3D::LineString(rings.into_iter().next().unwrap())),
        _ => LegacyBoundary::Boundary(Geometry3D::MultiLineString(MultiLineString3D::new(rings))),
    }
}

#[cfg(not(feature = "new-geometry"))]
fn extract_2d_boundary(geo: &Geometry2D<f64>) -> LegacyBoundary<Geometry2D<f64>> {
    match geo {
        // Positions have no extent, so nothing bounds them.
        Geometry2D::Point(_) | Geometry2D::MultiPoint(_) => LegacyBoundary::Empty,

        Geometry2D::Line(line) => LegacyBoundary::Boundary(Geometry2D::GeometryCollection(vec![
            Geometry2D::Point(line.start_point()),
            Geometry2D::Point(line.end_point()),
        ])),

        // A chain is bounded by its two ends; one that closes has none.
        Geometry2D::LineString(ls) => match chain_ends_2d(ls) {
            Some(points) => LegacyBoundary::Boundary(Geometry2D::GeometryCollection(points)),
            None => LegacyBoundary::Empty,
        },

        Geometry2D::Polygon(polygon) => rings_2d(polygon.rings().to_vec()),

        Geometry2D::MultiLineString(mls) => {
            let ends: Vec<_> = mls.iter().filter_map(chain_ends_2d).flatten().collect();
            if ends.is_empty() {
                LegacyBoundary::Empty
            } else {
                LegacyBoundary::Boundary(Geometry2D::GeometryCollection(ends))
            }
        }

        Geometry2D::MultiPolygon(mp) => {
            rings_2d(mp.iter().flat_map(|p| p.rings().to_vec()).collect())
        }

        Geometry2D::Rect(rect) => {
            LegacyBoundary::Boundary(Geometry2D::LineString(rect.to_polygon().exterior().clone()))
        }

        Geometry2D::Triangle(triangle) => {
            let c = triangle.to_array();
            LegacyBoundary::Boundary(Geometry2D::LineString(LineString2D::from(vec![
                c[0], c[1], c[2], c[0],
            ])))
        }

        _ => LegacyBoundary::None,
    }
}

#[cfg(not(feature = "new-geometry"))]
fn extract_3d_boundary(geo: &Geometry3D<f64>) -> LegacyBoundary<Geometry3D<f64>> {
    match geo {
        Geometry3D::Point(_) | Geometry3D::MultiPoint(_) => LegacyBoundary::Empty,

        Geometry3D::Line(line) => LegacyBoundary::Boundary(Geometry3D::GeometryCollection(vec![
            Geometry3D::Point(line.start_point()),
            Geometry3D::Point(line.end_point()),
        ])),

        Geometry3D::LineString(ls) => match chain_ends_3d(ls) {
            Some(points) => LegacyBoundary::Boundary(Geometry3D::GeometryCollection(points)),
            None => LegacyBoundary::Empty,
        },

        Geometry3D::Polygon(polygon) => rings_3d(polygon.rings().to_vec()),

        Geometry3D::MultiLineString(mls) => {
            let ends: Vec<_> = mls.iter().filter_map(chain_ends_3d).flatten().collect();
            if ends.is_empty() {
                LegacyBoundary::Empty
            } else {
                LegacyBoundary::Boundary(Geometry3D::GeometryCollection(ends))
            }
        }

        Geometry3D::MultiPolygon(mp) => {
            rings_3d(mp.iter().flat_map(|p| p.rings().to_vec()).collect())
        }

        Geometry3D::Rect(rect) => {
            LegacyBoundary::Boundary(Geometry3D::MultiPolygon(rect.to_multi_polygon()))
        }

        Geometry3D::Triangle(triangle) => {
            let c = triangle.to_array();
            LegacyBoundary::Boundary(Geometry3D::LineString(LineString3D::from(vec![
                c[0], c[1], c[2], c[0],
            ])))
        }

        Geometry3D::TriangularMesh(mesh) => {
            extract_mesh_boundary(mesh).map(Geometry3D::MultiLineString)
        }

        // A volume is bounded by its surface.
        Geometry3D::Solid(solid) => match solid.clone().as_triangle_mesh(None) {
            Ok(mesh) => LegacyBoundary::Boundary(Geometry3D::TriangularMesh(mesh)),
            Err(_) => LegacyBoundary::None,
        },

        Geometry3D::GeometryCollection(collection) => {
            let mut bounded = Vec::new();
            let mut any = false;
            for member in collection {
                match extract_3d_boundary(member) {
                    LegacyBoundary::Boundary(b) => {
                        any = true;
                        bounded.push(b);
                    }
                    LegacyBoundary::Empty => any = true,
                    LegacyBoundary::None => {}
                }
            }
            if !any && !collection.is_empty() {
                LegacyBoundary::None
            } else if bounded.is_empty() {
                LegacyBoundary::Empty
            } else if bounded.len() == 1 {
                LegacyBoundary::Boundary(bounded.into_iter().next().unwrap())
            } else {
                LegacyBoundary::Boundary(Geometry3D::GeometryCollection(bounded))
            }
        }

        _ => LegacyBoundary::None,
    }
}

/// The two ends of an open chain, as points. A chain that closes on itself, or
/// that spans nothing, has no ends to give.
#[cfg(not(feature = "new-geometry"))]
fn chain_ends_2d(ls: &LineString2D<f64>) -> Option<Vec<Geometry2D<f64>>> {
    let coords: Vec<_> = ls.coords().cloned().collect();
    if ls.is_closed() || coords.len() < 2 {
        return None;
    }
    Some(vec![
        Geometry2D::Point(coords[0].into()),
        Geometry2D::Point(coords[coords.len() - 1].into()),
    ])
}

#[cfg(not(feature = "new-geometry"))]
fn chain_ends_3d(ls: &LineString3D<f64>) -> Option<Vec<Geometry3D<f64>>> {
    let coords: Vec<_> = ls.coords().cloned().collect();
    if ls.is_closed() || coords.len() < 2 {
        return None;
    }
    Some(vec![
        Geometry3D::Point(coords[0].into()),
        Geometry3D::Point(coords[coords.len() - 1].into()),
    ])
}

/// The edges only one triangle walks, chained into rings.
#[cfg(not(feature = "new-geometry"))]
fn extract_mesh_boundary(
    mesh: &TriangularMesh<f64, f64>,
) -> LegacyBoundary<MultiLineString3D<f64>> {
    let mut walks: HashMap<(usize, usize), usize> = HashMap::new();
    for triangle in mesh.get_triangles() {
        for edge in [
            (triangle[0].min(triangle[1]), triangle[0].max(triangle[1])),
            (triangle[1].min(triangle[2]), triangle[1].max(triangle[2])),
            (triangle[0].min(triangle[2]), triangle[0].max(triangle[2])),
        ] {
            *walks.entry(edge).or_insert(0) += 1;
        }
    }

    let mut edges = walks
        .into_iter()
        .filter_map(|(edge, count)| (count == 1).then_some(edge))
        .collect::<Vec<_>>();
    if edges.is_empty() {
        return LegacyBoundary::Empty;
    }
    edges.sort_unstable();

    let edge_idx: HashMap<_, _> = edges
        .iter()
        .enumerate()
        .map(|(idx, &edge)| (edge, idx))
        .collect();

    let mut adjacency: Vec<Vec<usize>> = vec![Vec::new(); mesh.get_vertices().len()];
    for (v1, v2) in &edges {
        adjacency[*v1].push(*v2);
        adjacency[*v2].push(*v1);
    }

    let mut used_edges = vec![false; edges.len()];
    let mut chains = Vec::new();

    for start_idx in 0..edges.len() {
        if used_edges[start_idx] {
            continue;
        }

        used_edges[start_idx] = true;

        let start_v0 = edges[start_idx].0;
        let start_v1 = edges[start_idx].1;

        let mut chain = vec![mesh.get_vertices()[start_v0], mesh.get_vertices()[start_v1]];

        let mut prev_vertex = start_v0;
        let mut current_vertex = start_v1;

        loop {
            if adjacency[current_vertex].len() != 2 {
                break;
            }
            let next_vertex = *adjacency[current_vertex]
                .iter()
                .find(|&&v| v != prev_vertex)
                .unwrap();

            if next_vertex == start_v0 {
                chain.push(mesh.get_vertices()[next_vertex]);
                let idx = edge_idx
                    .get(&(
                        current_vertex.min(next_vertex),
                        current_vertex.max(next_vertex),
                    ))
                    .unwrap();
                used_edges[*idx] = true;
                break;
            }

            let idx = edge_idx
                .get(&(
                    current_vertex.min(next_vertex),
                    current_vertex.max(next_vertex),
                ))
                .unwrap();
            if used_edges[*idx] {
                break;
            }
            used_edges[*idx] = true;

            chain.push(mesh.get_vertices()[next_vertex]);
            prev_vertex = current_vertex;
            current_vertex = next_vertex;
        }

        if chain.first() != chain.last() {
            prev_vertex = start_v1;
            current_vertex = start_v0;

            loop {
                if adjacency[current_vertex].len() != 2 {
                    break;
                }
                let next_vertex = *adjacency[current_vertex]
                    .iter()
                    .find(|&&v| v != prev_vertex)
                    .unwrap();

                let idx = edge_idx
                    .get(&(
                        current_vertex.min(next_vertex),
                        current_vertex.max(next_vertex),
                    ))
                    .unwrap();
                if used_edges[*idx] {
                    break;
                }
                used_edges[*idx] = true;

                chain.insert(0, mesh.get_vertices()[next_vertex]);
                prev_vertex = current_vertex;
                current_vertex = next_vertex;
            }
        }

        if chain.len() >= 2 {
            chains.push(LineString3D::from(chain));
        }
    }

    if chains.is_empty() {
        LegacyBoundary::Empty
    } else {
        LegacyBoundary::Boundary(MultiLineString3D::new(chains))
    }
}

#[cfg(all(test, feature = "new-geometry"))]
mod tests {
    use super::*;
    use crate::tests::utils::create_default_execute_context;
    use pretty_assertions::assert_eq;
    use reearth_flow_geometry::coordinate::CoordinateFrame;
    use reearth_flow_geometry::csg::Csg;
    use reearth_flow_geometry::line_string::LineString3D;
    use reearth_flow_geometry::point::Point3D;
    use reearth_flow_geometry::polygon::Polygon3D;
    use reearth_flow_geometry::polygon_mesh::PolygonMesh3DData;
    use reearth_flow_geometry::solid::Solid;
    use reearth_flow_geometry::triangular_mesh::TriangularMesh3D;
    use reearth_flow_geometry::Euclidean3DGeometry;
    use reearth_flow_runtime::forwarder::NoopChannelForwarder;
    use reearth_flow_types::{Attribute, AttributeValue, Feature};

    /// A closed 4x4 square, as an exterior ring.
    const SQUARE: [[f64; 3]; 5] = [
        [0.0, 0.0, 0.0],
        [4.0, 0.0, 0.0],
        [4.0, 4.0, 0.0],
        [0.0, 4.0, 0.0],
        [0.0, 0.0, 0.0],
    ];

    fn face() -> Polygon3D {
        Polygon3D::from_rings(
            CoordinateFrame::Euclidean,
            SQUARE,
            Vec::<Vec<[f64; 3]>>::new(),
        )
    }

    fn area() -> Geometry {
        Geometry::Euclidean3D(Euclidean3DGeometry::Polygon(Box::new(face())))
    }

    /// A closed tetrahedron: bounded by nothing.
    fn closed_shell() -> Geometry {
        Geometry::Euclidean3D(Euclidean3DGeometry::TriangularMesh(Box::new(
            TriangularMesh3D::from_parts(
                CoordinateFrame::Euclidean,
                vec![
                    [0.0, 0.0, 0.0],
                    [1.0, 0.0, 0.0],
                    [0.0, 1.0, 0.0],
                    [0.0, 0.0, 1.0],
                ],
                [0u32, 2, 1, 0, 1, 3, 0, 3, 2, 1, 2, 3],
            )
            .unwrap(),
        )))
    }

    fn boolean_tree() -> Geometry {
        let solid = || {
            Solid::from_exterior(
                CoordinateFrame::Euclidean,
                PolygonMesh3DData::from_polygons([&face()]),
            )
        };
        Geometry::Euclidean3D(Euclidean3DGeometry::Csg(Csg::union(solid(), solid())))
    }

    /// A feature carrying `geometry` and one attribute to trace through.
    fn feature(geometry: Geometry) -> Feature {
        let mut feature = Feature::from(geometry);
        feature.insert("surfaceId", AttributeValue::Number(7.into()));
        feature
    }

    /// Run the processor over `feature`, returning what it sent, port by port.
    fn extract(feature: &Feature) -> Vec<(Port, Feature)> {
        let fw = ProcessorChannelForwarder::Noop(NoopChannelForwarder::default());
        BoundaryExtractor
            .process(create_default_execute_context(feature), &fw)
            .unwrap();
        let ProcessorChannelForwarder::Noop(noop) = fw else {
            unreachable!("built as a noop forwarder");
        };
        let ports = noop.send_ports.lock().unwrap().clone();
        let features = noop.send_features.lock().unwrap().clone();
        ports.into_iter().zip(features).collect()
    }

    fn ports(sent: &[(Port, Feature)]) -> Vec<String> {
        sent.iter().map(|(port, _)| port.to_string()).collect()
    }

    #[test]
    fn an_area_leaves_with_the_curve_that_bounds_it() {
        let input = feature(area());
        let sent = extract(&input);

        assert_eq!(ports(&sent), ["features"]);
        let Geometry::Euclidean3D(Euclidean3DGeometry::LineString(ring)) = &*sent[0].1.geometry
        else {
            panic!("expected one ring, got {:?}", sent[0].1.geometry);
        };
        assert_eq!(ring.coords(), SQUARE);
    }

    // One feature in, one feature out, so nothing has to be traced back: the id
    // and the attributes are the ones it arrived with.
    #[test]
    fn the_boundary_leaves_on_the_feature_it_came_from() {
        let input = feature(area());
        let sent = extract(&input);

        assert_eq!(sent[0].1.id, input.id);
        assert_eq!(
            sent[0].1.attributes.get(&Attribute::new("surfaceId")),
            Some(&AttributeValue::Number(7.into()))
        );
    }

    // Closed geometry keeps what it came in with, so a workflow can go on using
    // it after learning that it closes.
    #[test]
    fn geometry_bounded_by_nothing_leaves_intact() {
        for geometry in [
            closed_shell(),
            Geometry::Euclidean3D(Euclidean3DGeometry::LineString(LineString3D::from_coords(
                CoordinateFrame::Euclidean,
                SQUARE,
            ))),
            Geometry::Euclidean3D(Euclidean3DGeometry::Point(Point3D::new(
                CoordinateFrame::Euclidean,
                [1.0, 2.0, 3.0],
            ))),
        ] {
            let input = feature(geometry);
            let sent = extract(&input);
            assert_eq!(ports(&sent), ["no-boundary"]);
            assert_eq!(sent[0].1.id, input.id);
            assert_eq!(&*sent[0].1.geometry, &*input.geometry);
        }
    }

    // An unevaluated tree has no boundary to give, and a feature with no geometry
    // has nothing to bound. Neither fails the run.
    #[test]
    fn geometry_with_no_boundary_to_give_is_rejected_intact() {
        for geometry in [boolean_tree(), Geometry::None] {
            let input = feature(geometry);
            let sent = extract(&input);
            assert_eq!(ports(&sent), ["rejected"]);
            assert_eq!(sent[0].1.id, input.id);
            assert_eq!(&*sent[0].1.geometry, &*input.geometry);
        }
    }
}
