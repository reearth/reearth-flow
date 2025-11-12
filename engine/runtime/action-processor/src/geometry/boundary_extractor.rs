use std::collections::HashMap;

use reearth_flow_geometry::types::geometry::Geometry2D;
use reearth_flow_geometry::types::geometry::Geometry3D;
use reearth_flow_geometry::types::line_string::{LineString2D, LineString3D};
use reearth_flow_geometry::types::multi_line_string::{MultiLineString2D, MultiLineString3D};
use reearth_flow_geometry::types::triangular_mesh::TriangularMesh;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{Geometry, GeometryValue};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::GeometryProcessorError;

#[derive(Debug, Clone, Default)]
pub(super) struct BoundaryExtractorFactory;

impl ProcessorFactory for BoundaryExtractorFactory {
    fn name(&self) -> &str {
        "BoundaryExtractor"
    }

    fn description(&self) -> &str {
        "Extracts the boundary of geometries. For solids/meshes returns bounding surfaces, for surfaces returns boundary edges, for closed surfaces returns empty geometry"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(BoundaryExtractorParams))
    }

    fn categories(&self) -> &[&'static str] {
        &["Geometry"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
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

/// # BoundaryExtractor Parameters
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
    params: BoundaryExtractorParams,
}

impl Processor for BoundaryExtractor {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let geometry = &feature.geometry;

        if geometry.is_empty() {
            if self.params.keep_empty_boundaries {
                fw.send(ctx.new_with_feature_and_port(ctx.feature.clone(), DEFAULT_PORT.clone()));
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
                let mut new_geo = geometry.clone();
                new_geo.value = GeometryValue::FlowGeometry2D(g);
                new_geo
            }),
            GeometryValue::FlowGeometry3D(geo) => self.extract_3d_boundary(geo).map(|g| {
                let mut new_geo = geometry.clone();
                new_geo.value = GeometryValue::FlowGeometry3D(g);
                new_geo
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
            fw.send(ctx.new_with_feature_and_port(new_feature, DEFAULT_PORT.clone()));
        } else if self.params.keep_empty_boundaries {
            let mut new_feature = feature.clone();
            new_feature.geometry = Geometry::default();
            fw.send(ctx.new_with_feature_and_port(new_feature, DEFAULT_PORT.clone()));
        }

        Ok(())
    }

    fn finish(&self, _ctx: NodeContext, _fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "BoundaryExtractor"
    }
}

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
            Geometry3D::Rect(rect) => {
                let polygon = rect.to_polygon();
                Some(Geometry3D::LineString(polygon.exterior().clone()))
            }

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
                // Try to convert to triangular mesh
                match solid.clone().as_triangle_mesh() {
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

            let mut chain = Vec::new();
            let mut prev_vertex = edges[start_idx].0;
            let mut current_vertex = edges[start_idx].1;

            // Start the chain
            chain.push(mesh.get_vertices()[current_vertex]);

            loop {
                if adjacency[current_vertex].len() != 2 {
                    // Reached an endpoint
                    break;
                }
                let next_vertex = *adjacency[current_vertex]
                    .iter()
                    .find(|&&v| v != prev_vertex)
                    .unwrap(); // This is safe due to len() == 2
                chain.push(mesh.get_vertices()[next_vertex]);
                prev_vertex = current_vertex;
                current_vertex = next_vertex;
                let edge_idx = edge_idx
                    .get(&(
                        prev_vertex.min(current_vertex),
                        prev_vertex.max(current_vertex),
                    ))
                    .unwrap();
                used_edges[*edge_idx] = true;
            }

            // We need to loop in the opposite direction as well
            prev_vertex = edges[start_idx].1;
            current_vertex = edges[start_idx].0;
            loop {
                if adjacency[current_vertex].len() != 2 {
                    // Reached an endpoint
                    break;
                }
                let next_vertex = *adjacency[current_vertex]
                    .iter()
                    .find(|&&v| v != prev_vertex)
                    .unwrap(); // This is safe due to len() == 2
                chain.push(mesh.get_vertices()[next_vertex]);
                prev_vertex = current_vertex;
                current_vertex = next_vertex;
                let edge_idx = edge_idx
                    .get(&(
                        prev_vertex.min(current_vertex),
                        prev_vertex.max(current_vertex),
                    ))
                    .unwrap();
                used_edges[*edge_idx] = true;
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
