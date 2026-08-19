use std::collections::HashMap;
#[cfg(not(feature = "new-geometry"))]
use std::sync::Arc;

#[cfg(feature = "new-geometry")]
use once_cell::sync::Lazy;
#[cfg(feature = "new-geometry")]
use reearth_flow_geometry::ops::ExtractBoundary;
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
use reearth_flow_geometry::Geometry as NextGeometry;
#[cfg(feature = "new-geometry")]
use reearth_flow_runtime::node::REJECTED_PORT;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, FEATURES_PORT},
};
#[cfg(not(feature = "new-geometry"))]
use reearth_flow_types::{Geometry, GeometryValue};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::GeometryProcessorError;

/// Geometry that closes on itself, or carries no extent to bound, leaves here
/// with the geometry it arrived with.
#[cfg(feature = "new-geometry")]
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

    // The ports carry what `keepEmptyBoundaries` used to, and a boundary includes
    // the interior rings `exteriorOnly` dropped, so the new world takes no
    // parameters. The old world keeps its own, unchanged.
    #[cfg(feature = "new-geometry")]
    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        None
    }

    #[cfg(not(feature = "new-geometry"))]
    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(BoundaryExtractorParams))
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

    #[cfg(feature = "new-geometry")]
    fn get_output_ports(&self) -> Vec<Port> {
        vec![
            FEATURES_PORT.clone(),
            NO_BOUNDARY_PORT.clone(),
            REJECTED_PORT.clone(),
        ]
    }

    #[cfg(not(feature = "new-geometry"))]
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
        let params: BoundaryExtractorParams = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                GeometryProcessorError::BoundaryExtractorFactory(format!(
                    "Failed to serialize 'with' parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                GeometryProcessorError::BoundaryExtractorFactory(format!(
                    "Failed to deserialize 'with' parameter: {e}"
                ))
            })?
        } else {
            BoundaryExtractorParams::default()
        };
        Ok(Box::new(BoundaryExtractor { params }))
    }
}

// AUDIT NOTE (left by the Geometry A batch, 2026-07-30). This action has not been
// audited yet. The observations below came from reading this file while deciding
// something else, so treat them as leads to CHECK, not conclusions to apply —
// verify each against the code and the standard before acting, and disagree freely
// if the reading is wrong.
//
// 1. Suspected silent data loss. When `keepEmptyBoundaries` is false — the default —
//    a feature whose boundary cannot be extracted appears to be dropped entirely:
//    no port receives it and there is no `rejected` port. CityGML geometry looks
//    worst affected, since the match arm for it extracts nothing at all, so every
//    CityGML feature may vanish by default. Confirm by tracing each `None` branch
//    in `process`. If it holds, §4.3 wants a `rejected` port.
// 2. If `rejected` is added, re-examine whether `keepEmptyBoundaries` should exist
//    at all. It reads as a routing decision expressed as a parameter, which ports
//    already express; §3.5 would call that implementation leakage. Check whether any
//    workflow relies on it before removing.
// 3. `exteriorOnly` looks like a genuine semantic choice worth keeping, but it is
//    negatively framed. Consider inverting it to `includeHoles` (default true).
// 4. The description is three sentences, has no terminating period, and leaks
//    implementation detail — see §2.
//
// Cross-check before consolidating this with any other action: its shape is one
// feature in, one feature out with the geometry replaced. Geometry Part Extractor
// and Hole Extractor instead emit one feature per part. Ports are declared
// statically by the factory and cannot vary by parameter, so merging actions of
// different shapes forces dead ports onto the node.

/// # Boundary Extractor Parameters
///
/// Configuration for extracting boundaries from geometries.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema, Default)]
#[serde(rename_all = "camelCase")]
pub struct BoundaryExtractorParams {
    /// Whether to keep features with empty boundaries (default: false)
    #[serde(default)]
    keep_empty_boundaries: bool,

    /// Whether to extract only exterior boundaries (ignoring holes) for polygons (default: false)
    #[serde(default)]
    exterior_only: bool,
}

#[derive(Debug, Clone)]
struct BoundaryExtractor {
    #[cfg_attr(feature = "new-geometry", allow(dead_code))]
    params: BoundaryExtractorParams,
}

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
            // An answer, not a failure, so the feature goes on with the geometry
            // it came in with.
            Ok(NextGeometry::None) => {
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
            if self.params.keep_empty_boundaries {
                fw.send(ctx.new_with_feature_and_port(ctx.feature.clone(), FEATURES_PORT.clone()));
            }
            return Ok(());
        }

        let new_geometry = match &geometry.value {
            GeometryValue::None => {
                if self.params.keep_empty_boundaries {
                    Some(geometry.clone())
                } else {
                    None
                }
            }
            GeometryValue::FlowGeometry2D(geo) => self.extract_2d_boundary(geo).map(|g| {
                let mut new_geo = (**geometry).clone();
                new_geo.value = GeometryValue::FlowGeometry2D(g);
                Arc::new(new_geo)
            }),
            GeometryValue::FlowGeometry3D(geo) => self.extract_3d_boundary(geo).map(|g| {
                let mut new_geo = (**geometry).clone();
                new_geo.value = GeometryValue::FlowGeometry3D(g);
                Arc::new(new_geo)
            }),
            GeometryValue::CityGmlGeometry(_) => {
                // For CityGML geometries, we don't extract boundaries directly
                // They should be converted to regular geometries first
                if self.params.keep_empty_boundaries {
                    Some(geometry.clone())
                } else {
                    None
                }
            }
        };

        if let Some(new_geo) = new_geometry {
            let mut new_feature = feature.clone();
            new_feature.geometry = new_geo;
            fw.send(ctx.new_with_feature_and_port(new_feature, FEATURES_PORT.clone()));
        } else if self.params.keep_empty_boundaries {
            let mut new_feature = feature.clone();
            new_feature.geometry = Arc::new(Geometry::default());
            fw.send(ctx.new_with_feature_and_port(new_feature, FEATURES_PORT.clone()));
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

#[cfg(not(feature = "new-geometry"))]
impl BoundaryExtractor {
    fn extract_2d_boundary(&self, geo: &Geometry2D) -> Option<Geometry2D> {
        match geo {
            // Point has no boundary
            Geometry2D::Point(_) => None,

            // Line boundary is its endpoints
            Geometry2D::Line(line) => {
                let points = vec![
                    Geometry2D::Point(line.start_point()),
                    Geometry2D::Point(line.end_point()),
                ];
                Some(Geometry2D::GeometryCollection(points))
            }

            // LineString boundary is its endpoints if not closed
            Geometry2D::LineString(ls) => {
                if ls.is_closed() {
                    None // Closed curve has no boundary
                } else {
                    let coords: Vec<_> = ls.coords().cloned().collect();
                    if coords.len() >= 2 {
                        let points = vec![
                            Geometry2D::Point(coords[0].into()),
                            Geometry2D::Point(coords[coords.len() - 1].into()),
                        ];
                        Some(Geometry2D::GeometryCollection(points))
                    } else {
                        None
                    }
                }
            }

            // Polygon boundary is its rings (exterior + holes)
            Geometry2D::Polygon(polygon) => {
                let rings = if self.params.exterior_only {
                    vec![polygon.exterior().clone()]
                } else {
                    polygon.rings().to_vec()
                };

                if rings.is_empty() {
                    None
                } else if rings.len() == 1 {
                    Some(Geometry2D::LineString(rings[0].clone()))
                } else {
                    Some(Geometry2D::MultiLineString(MultiLineString2D::new(rings)))
                }
            }

            // MultiPoint has no boundary
            Geometry2D::MultiPoint(_) => None,

            // MultiLineString boundary is the set of endpoints of non-closed linestrings
            Geometry2D::MultiLineString(mls) => {
                let mut endpoints = Vec::new();
                for ls in mls.iter() {
                    if !ls.is_closed() {
                        let coords: Vec<_> = ls.coords().cloned().collect();
                        if coords.len() >= 2 {
                            endpoints.push(Geometry2D::Point(coords[0].into()));
                            endpoints.push(Geometry2D::Point(coords[coords.len() - 1].into()));
                        }
                    }
                }

                if endpoints.is_empty() {
                    None
                } else {
                    Some(Geometry2D::GeometryCollection(endpoints))
                }
            }

            // MultiPolygon boundary is the union of all polygon boundaries
            Geometry2D::MultiPolygon(mp) => {
                let mut all_rings = Vec::new();
                for polygon in mp.iter() {
                    if self.params.exterior_only {
                        all_rings.push(polygon.exterior().clone());
                    } else {
                        all_rings.extend_from_slice(&polygon.rings());
                    }
                }

                if all_rings.is_empty() {
                    None
                } else if all_rings.len() == 1 {
                    Some(Geometry2D::LineString(all_rings[0].clone()))
                } else {
                    Some(Geometry2D::MultiLineString(MultiLineString2D::new(
                        all_rings,
                    )))
                }
            }

            // Rectangle boundary is its perimeter
            Geometry2D::Rect(rect) => {
                let polygon = rect.to_polygon();
                Some(Geometry2D::LineString(polygon.exterior().clone()))
            }

            // Triangle boundary is its perimeter
            Geometry2D::Triangle(triangle) => {
                let coords = triangle.to_array();
                let ls = LineString2D::from(vec![
                    coords[0], coords[1], coords[2], coords[0], // Close the triangle
                ]);
                Some(Geometry2D::LineString(ls))
            }

            // For other geometry types, return None
            _ => None,
        }
    }

    fn extract_3d_boundary(&self, geo: &Geometry3D) -> Option<Geometry3D> {
        match geo {
            // Point has no boundary
            Geometry3D::Point(_) => None,

            // Line boundary is its endpoints
            Geometry3D::Line(line) => {
                let points = vec![
                    Geometry3D::Point(line.start_point()),
                    Geometry3D::Point(line.end_point()),
                ];
                Some(Geometry3D::GeometryCollection(points))
            }

            // LineString boundary is its endpoints if not closed
            Geometry3D::LineString(ls) => {
                if ls.is_closed() {
                    None // Closed curve has no boundary
                } else {
                    let coords: Vec<_> = ls.coords().cloned().collect();
                    if coords.len() >= 2 {
                        let points = vec![
                            Geometry3D::Point(coords[0].into()),
                            Geometry3D::Point(coords[coords.len() - 1].into()),
                        ];
                        Some(Geometry3D::GeometryCollection(points))
                    } else {
                        None
                    }
                }
            }

            // Polygon boundary is its rings (exterior + holes)
            Geometry3D::Polygon(polygon) => {
                let rings = if self.params.exterior_only {
                    vec![polygon.exterior().clone()]
                } else {
                    polygon.rings().to_vec()
                };

                if rings.is_empty() {
                    None
                } else if rings.len() == 1 {
                    Some(Geometry3D::LineString(rings[0].clone()))
                } else {
                    Some(Geometry3D::MultiLineString(MultiLineString3D::new(rings)))
                }
            }

            // MultiPoint has no boundary
            Geometry3D::MultiPoint(_) => None,

            // MultiLineString boundary is the set of endpoints of non-closed linestrings
            Geometry3D::MultiLineString(mls) => {
                let mut endpoints = Vec::new();
                for ls in mls.iter() {
                    if !ls.is_closed() {
                        let coords: Vec<_> = ls.coords().cloned().collect();
                        if coords.len() >= 2 {
                            endpoints.push(Geometry3D::Point(coords[0].into()));
                            endpoints.push(Geometry3D::Point(coords[coords.len() - 1].into()));
                        }
                    }
                }

                if endpoints.is_empty() {
                    None
                } else {
                    Some(Geometry3D::GeometryCollection(endpoints))
                }
            }

            // MultiPolygon boundary is the union of all polygon boundaries
            Geometry3D::MultiPolygon(mp) => {
                let mut all_rings = Vec::new();
                for polygon in mp.iter() {
                    if self.params.exterior_only {
                        all_rings.push(polygon.exterior().clone());
                    } else {
                        all_rings.extend_from_slice(&polygon.rings());
                    }
                }

                if all_rings.is_empty() {
                    None
                } else if all_rings.len() == 1 {
                    Some(Geometry3D::LineString(all_rings[0].clone()))
                } else {
                    Some(Geometry3D::MultiLineString(MultiLineString3D::new(
                        all_rings,
                    )))
                }
            }

            // Rectangle boundary is its perimeter
            Geometry3D::Rect(rect) => Some(Geometry3D::MultiPolygon(rect.to_multi_polygon())),

            // Triangle boundary is its perimeter
            Geometry3D::Triangle(triangle) => {
                let coords = triangle.to_array();
                let ls = LineString3D::from(vec![
                    coords[0], coords[1], coords[2], coords[0], // Close the triangle
                ]);
                Some(Geometry3D::LineString(ls))
            }

            // TriangularMesh boundary is the set of boundary edges
            Geometry3D::TriangularMesh(mesh) => self
                .extract_mesh_boundary(mesh)
                .map(Geometry3D::MultiLineString),

            // Solid boundary is the triangular mesh representing its surface
            Geometry3D::Solid(solid) => {
                // A solid's boundary is its surface mesh
                // Try to convert to triangular mesh with default tolerance
                match solid.clone().as_triangle_mesh(None) {
                    Ok(mesh) => Some(Geometry3D::TriangularMesh(mesh)),
                    Err(_) => {
                        // If conversion fails, the solid might be represented as faces
                        // In this case, we cannot easily extract the boundary
                        None
                    }
                }
            }

            // GeometryCollection: extract boundaries of each geometry
            Geometry3D::GeometryCollection(collection) => {
                let mut boundaries = Vec::new();
                for geom in collection {
                    if let Some(boundary) = self.extract_3d_boundary(geom) {
                        boundaries.push(boundary);
                    }
                }

                if boundaries.is_empty() {
                    None
                } else if boundaries.len() == 1 {
                    Some(boundaries.into_iter().next().unwrap())
                } else {
                    Some(Geometry3D::GeometryCollection(boundaries))
                }
            }

            // For other geometry types like CSG, return None
            _ => None,
        }
    }

    fn extract_mesh_boundary(
        &self,
        mesh: &TriangularMesh<f64, f64>,
    ) -> Option<MultiLineString3D<f64>> {
        // Extract boundary edges from the triangular mesh
        // Boundary edges are those that belong to only one triangle
        let mut edge_count: HashMap<(usize, usize), usize> = HashMap::new();
        let triangles = mesh.get_triangles();

        // Count how many triangles each edge belongs to
        for triangle in triangles {
            let edges = [
                (triangle[0].min(triangle[1]), triangle[0].max(triangle[1])),
                (triangle[1].min(triangle[2]), triangle[1].max(triangle[2])),
                (triangle[0].min(triangle[2]), triangle[0].max(triangle[2])),
            ];

            for edge in &edges {
                *edge_count.entry(*edge).or_insert(0) += 1;
            }
        }

        let mut edges = edge_count
            .into_iter()
            .filter_map(|(edge, count)| if count == 1 { Some(edge) } else { None })
            .collect::<Vec<_>>();

        if edges.is_empty() {
            return None; // Closed surface has no boundary
        }

        edges.sort_unstable();
        let edges = edges;

        let edge_idx: HashMap<_, _> = edges
            .iter()
            .enumerate()
            .map(|(idx, &edge)| (edge, idx))
            .collect();

        // Build adjacency map: vertex -> list of connected vertices
        let mut adjacency: Vec<Vec<usize>> = vec![Vec::new(); mesh.get_vertices().len()];
        for (v1, v2) in &edges {
            adjacency[*v1].push(*v2);
            adjacency[*v2].push(*v1);
        }

        // Chain boundary edges into connected linestrings
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

            // Traverse forward from start_v1
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

                // Check if we've closed the loop
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
                    // Already visited this edge
                    break;
                }
                used_edges[*idx] = true;

                chain.push(mesh.get_vertices()[next_vertex]);
                prev_vertex = current_vertex;
                current_vertex = next_vertex;
            }

            // Traverse backward from start_v0 only if we didn't close a loop
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

                    // Prepend to chain
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
            None
        } else {
            Some(MultiLineString3D::new(chains))
        }
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

    fn area() -> NextGeometry {
        NextGeometry::Euclidean3D(Euclidean3DGeometry::Polygon(Box::new(face())))
    }

    /// A closed tetrahedron: bounded by nothing.
    fn closed_shell() -> NextGeometry {
        NextGeometry::Euclidean3D(Euclidean3DGeometry::TriangularMesh(Box::new(
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

    fn boolean_tree() -> NextGeometry {
        let solid = || {
            Solid::from_exterior(
                CoordinateFrame::Euclidean,
                PolygonMesh3DData::from_polygons([&face()]),
            )
        };
        NextGeometry::Euclidean3D(Euclidean3DGeometry::Csg(Csg::union(solid(), solid())))
    }

    /// A feature carrying `geometry` and one attribute to trace through.
    fn feature(geometry: NextGeometry) -> Feature {
        let mut feature = Feature::from(geometry);
        feature.insert("surfaceId", AttributeValue::Number(7.into()));
        feature
    }

    /// Run the processor over `feature`, returning what it sent, port by port.
    fn extract(feature: &Feature) -> Vec<(Port, Feature)> {
        let fw = ProcessorChannelForwarder::Noop(NoopChannelForwarder::default());
        BoundaryExtractor {
            params: Default::default(),
        }
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
        let NextGeometry::Euclidean3D(Euclidean3DGeometry::LineString(ring)) = &*sent[0].1.geometry
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
            NextGeometry::Euclidean3D(Euclidean3DGeometry::LineString(LineString3D::from_coords(
                CoordinateFrame::Euclidean,
                SQUARE,
            ))),
            NextGeometry::Euclidean3D(Euclidean3DGeometry::Point(Point3D::new(
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
        for geometry in [boolean_tree(), NextGeometry::None] {
            let input = feature(geometry);
            let sent = extract(&input);
            assert_eq!(ports(&sent), ["rejected"]);
            assert_eq!(sent[0].1.id, input.id);
            assert_eq!(&*sent[0].1.geometry, &*input.geometry);
        }
    }
}
