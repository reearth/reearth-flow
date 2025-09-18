use std::collections::{HashMap, HashSet};

use num_traits::FromPrimitive;
use once_cell::sync::Lazy;
use reearth_flow_geometry::{
    algorithm::GeoFloat, error, types::{
        coordinate::Coordinate, face::Face3D, geometry::Geometry, point::Point3D,
        solid::Solid3D,
    }
};
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, GeometryValue};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::GeometryProcessorError;

static SUCCESS_PORT: Lazy<Port> = Lazy::new(|| Port::new("success"));
static FAILED_PORT: Lazy<Port> = Lazy::new(|| Port::new("failed"));
static REJECTED_PORT: Lazy<Port> = Lazy::new(|| Port::new("rejected"));

#[derive(Debug, Clone, Default)]
pub struct SolidBoundaryValidatorFactory;

impl ProcessorFactory for SolidBoundaryValidatorFactory {
    fn name(&self) -> &str {
        "SolidBoundaryValidator"
    }

    fn description(&self) -> &str {
        "Validates the Solid Boundary Geometry"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(SolidBoundaryValidator))
    }

    fn categories(&self) -> &[&'static str] {
        &["Geometry"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![
            SUCCESS_PORT.clone(),
            FAILED_PORT.clone(),
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
        let processor: SolidBoundaryValidator = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                GeometryProcessorError::SoilidBoundaryValidatorFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                GeometryProcessorError::SoilidBoundaryValidatorFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(GeometryProcessorError::SoilidBoundaryValidatorFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        Ok(Box::new(processor))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ValidationResult {
    issue_count: usize,
    issue: IssueType,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
enum IssueType {
    NotA2Manifold,
    NonOrientable,
    WrongFaceOrientation,
    NotConnected,
    SelfIntersection,
}

/// # Solid Boundary Validator Parameters
/// Configure which validation checks to perform on feature geometries
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SolidBoundaryValidator {}

impl Processor for SolidBoundaryValidator {
    fn num_threads(&self) -> usize {
        2
    }

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

        // Extract solid geometry from feature
        let geom = match &geometry.value {
            GeometryValue::FlowGeometry3D(geom) => geom,
            GeometryValue::CityGmlGeometry(gml_geom) => {
                if gml_geom.gml_geometries.len() > 1 {
                    panic!();
                }
                let Some(geom) = gml_geom.gml_geometries.first() else {
                    fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
                    return Ok(());
                };


                println!("DEBUG: line_strings = {:?}", geom.line_strings);

                panic!();
            }
            _ => {
                // Not a solid geometry, send to rejected port
                fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
                return Ok(());
            }
        };

        // Extract vertices, edges, and triangles from the solid
        let (vertices, edges_with_multiplicity, triangles) = self.extract_topology(&geom.as_solid().unwrap());

        // Run validation checks and collect issues
        let mut validation_results = Vec::new();

        // Check manifold condition
        if let Some(result) = self.check_manifold_condition::<f64>(&edges_with_multiplicity) {
            validation_results.push(result);
        }

        // Check orientability
        if let Some(result) = self.check_orientability::<f64>(&triangles) {
            validation_results.push(result);
        }

        // Check connectivity
        if let Some(result) = self.check_connectivity::<f64>(&triangles) {
            validation_results.push(result);
        }

        // Check self-intersection
        if let Some(result) = self.check_self_intersection(&vertices, &triangles) {
            validation_results.push(result);
        }

        // Add validation results to feature attributes if there are issues
        let mut feature = feature.clone();
        if !validation_results.is_empty() {
            feature.attributes.insert(
                Attribute::new("boundary_validation_issues"),
                AttributeValue::from(serde_json::to_value(&validation_results).unwrap()),
            );
            // Send to failed port if there are validation issues
            fw.send(ctx.new_with_feature_and_port(feature, FAILED_PORT.clone()));
        } else {
            // No issues found, send to success port
            fw.send(ctx.new_with_feature_and_port(feature, SUCCESS_PORT.clone()));
        }

        Ok(())
    }

    fn finish(&self, _ctx: NodeContext, _fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "SolidBoundaryValidator"
    }
}

impl SolidBoundaryValidator {
    fn reject(
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) {
        fw.send(ctx.new_with_feature_and_port(
            ctx.feature.clone(),
            REJECTED_PORT.clone(),
        ));
    }

    fn extract_topology(
        &self,
        solid: &Solid3D<f64>,
    ) -> (Vec<Point3D<f64>>, Vec<([usize; 2], usize)>, Vec<[usize; 3]>) {
        let mut vertices: Vec<Point3D<f64>> = Vec::new();
        let mut vertex_map: HashMap<String, usize> = HashMap::new();
        let mut edges_with_multiplicity: Vec<([usize; 2], usize)> = Vec::new();
        let mut triangles: Vec<[usize; 3]> = Vec::new();

        // Process all faces (bottom, top, and sides)
        let all_faces: Vec<&Face3D<f64>> = solid
            .bottom
            .iter()
            .chain(solid.top.iter())
            .chain(solid.sides.iter())
            .collect();

        for face in all_faces {
            // Triangulate the face
            let face_triangles = self.triangulate_face(face);

            for triangle in face_triangles {
                let mut tri_indices = [0usize; 3];

                for (i, vertex) in triangle.iter().enumerate() {
                    // Create a key for the vertex (z is always f64 for 3D coordinates)
                    let key = format!("{:.10},{:.10},{:.10}", vertex.x, vertex.y, vertex.z);

                    // Get or insert vertex index
                    let vertex_index = match vertex_map.get(&key) {
                        Some(&idx) => idx,
                        None => {
                            let idx = vertices.len();
                            vertices.push(Point3D::new(vertex.x, vertex.y, vertex.z));
                            vertex_map.insert(key, idx);
                            idx
                        }
                    };

                    tri_indices[i] = vertex_index;
                }

                // Add triangle
                triangles.push(tri_indices);

                // Add edges. Count multiplicity for manifold check
                for i in 0..3 {
                    let j = (i + 1) % 3;
                    let edge = if tri_indices[i] < tri_indices[j] {
                        [tri_indices[i], tri_indices[j]]
                    } else {
                        [tri_indices[j], tri_indices[i]]
                    };
                    if let Ok(idx) = edges_with_multiplicity
                        .binary_search_by_key(&edge, |(e, _)| *e)
                    {
                        edges_with_multiplicity[idx].1 += 1;
                    } else {
                        edges_with_multiplicity.push((edge, 1));
                    }
                }
            }
        }

        (vertices, edges_with_multiplicity, triangles)
    }

    fn triangulate_face(
        &self,
        face: &Face3D<f64>,
    ) -> Vec<Vec<Coordinate<f64, f64>>> {
        let coords = &face.0;
        if coords.len() < 3 {
            return vec![];
        }

        // Simple ear-clipping triangulation for convex polygons
        // For more complex polygons, a more sophisticated algorithm would be needed
        let mut result = Vec::new();

        if coords.len() == 3 {
            // Already a triangle
            result.push(coords.clone());
        } else {
            // Triangulate using fan method from first vertex (assumes convex polygon)
            // For non-convex polygons, this would need to be more sophisticated
            let first = coords[0];
            for i in 2..coords.len() {
                let triangle = vec![first, coords[i - 1], coords[i]];
                result.push(triangle);
            }
        }

        result
    }

    fn check_manifold_condition<T: GeoFloat + FromPrimitive>(
        &self,
        edges_with_multiplicity: &Vec<([usize; 2], usize)>,
    ) -> Option<ValidationResult> {
        let mut non_manifold_edges = 0;

        for &(_, count) in edges_with_multiplicity {
            if count != 2 {
                non_manifold_edges += 1;
            }
        }

        if non_manifold_edges > 0 {
            Some(ValidationResult {
                issue_count: non_manifold_edges,
                issue: IssueType::NotA2Manifold,
            })
        } else {
            None
        }
    }

    fn check_orientability<T: GeoFloat + FromPrimitive>(
        &self,
        triangles: &[[usize; 3]],
    ) -> Option<ValidationResult> {
        // Check if all triangles are consistently oriented
        // Build adjacency information
        let mut edge_to_triangles: HashMap<(usize, usize), Vec<usize>> = HashMap::new();
        let mut inconsistent_orientations = 0;

        for (tri_idx, triangle) in triangles.iter().enumerate() {
            for i in 0..3 {
                let j = (i + 1) % 3;
                let edge = if triangle[i] < triangle[j] {
                    (triangle[i], triangle[j])
                } else {
                    (triangle[j], triangle[i])
                };

                edge_to_triangles
                    .entry(edge)
                    .or_insert_with(Vec::new)
                    .push(tri_idx);
            }
        }

        // Check that adjacent triangles have opposite edge orientations
        for (edge, tri_indices) in edge_to_triangles.iter() {
            if tri_indices.len() != 2 {
                continue; // Skip boundary edges
            }

            let tri1 = &triangles[tri_indices[0]];
            let tri2 = &triangles[tri_indices[1]];

            // Find edge orientation in each triangle
            let mut ori1 = None;
            let mut ori2 = None;

            for i in 0..3 {
                let j = (i + 1) % 3;
                if tri1[i] == edge.0 && tri1[j] == edge.1 {
                    ori1 = Some(true);
                } else if tri1[i] == edge.1 && tri1[j] == edge.0 {
                    ori1 = Some(false);
                }

                if tri2[i] == edge.0 && tri2[j] == edge.1 {
                    ori2 = Some(true);
                } else if tri2[i] == edge.1 && tri2[j] == edge.0 {
                    ori2 = Some(false);
                }
            }

            // Check that orientations are opposite
            if let (Some(o1), Some(o2)) = (ori1, ori2) {
                if o1 == o2 {
                    inconsistent_orientations += 1;
                }
            }
        }

        if inconsistent_orientations > 0 {
            Some(ValidationResult {
                issue_count: inconsistent_orientations,
                issue: IssueType::NonOrientable,
            })
        } else {
            None
        }
    }

    fn check_connectivity<T: GeoFloat + FromPrimitive>(
        &self,
        triangles: &[[usize; 3]],
    ) -> Option<ValidationResult> {
        if triangles.is_empty() {
            return None;
        }

        // Build adjacency list
        let mut adjacency: HashMap<usize, HashSet<usize>> = HashMap::new();
        let mut all_vertices = HashSet::new();

        for triangle in triangles {
            for i in 0..3 {
                all_vertices.insert(triangle[i]);
                for j in 0..3 {
                    if i != j {
                        adjacency
                            .entry(triangle[i])
                            .or_insert_with(HashSet::new)
                            .insert(triangle[j]);
                    }
                }
            }
        }

        // BFS to check if all vertices are reachable from the first vertex
        let mut visited = HashSet::new();
        let mut queue = vec![triangles[0][0]];
        visited.insert(triangles[0][0]);

        while let Some(vertex) = queue.pop() {
            if let Some(neighbors) = adjacency.get(&vertex) {
                for &neighbor in neighbors {
                    if !visited.contains(&neighbor) {
                        visited.insert(neighbor);
                        queue.push(neighbor);
                    }
                }
            }
        }

        // Count disconnected components
        let disconnected_vertices = all_vertices.len() - visited.len();

        if disconnected_vertices > 0 {
            Some(ValidationResult {
                issue_count: disconnected_vertices,
                issue: IssueType::NotConnected,
            })
        } else {
            None
        }
    }

    fn check_self_intersection<T: GeoFloat + FromPrimitive>(
        &self,
        vertices: &[Point3D<T>],
        triangles: &[[usize; 3]],
    ) -> Option<ValidationResult> {
        // Simple check: ensure no edges cross through triangles they're not part of
        // This is a simplified version - a complete implementation would need
        // more sophisticated triangle-triangle intersection tests

        let mut issues_found = 0;

        // Check for degenerate triangles and overlapping triangles
        for (i, tri1) in triangles.iter().enumerate() {
            // Check for degenerate triangle (collinear vertices)
            let p0 = &vertices[tri1[0]];
            let p1 = &vertices[tri1[1]];
            let p2 = &vertices[tri1[2]];

            // Calculate cross product to check if points are collinear
            let v1 = Point3D::new(p1.x() - p0.x(), p1.y() - p0.y(), p1.z() - p0.z());
            let v2 = Point3D::new(p2.x() - p0.x(), p2.y() - p0.y(), p2.z() - p0.z());

            let cross_x = v1.y() * v2.z() - v1.z() * v2.y();
            let cross_y = v1.z() * v2.x() - v1.x() * v2.z();
            let cross_z = v1.x() * v2.y() - v1.y() * v2.x();

            let cross_magnitude =
                (cross_x * cross_x + cross_y * cross_y + cross_z * cross_z).sqrt();

            // If cross product is near zero, vertices are collinear
            if cross_magnitude < T::from(1e-10).unwrap_or_else(T::zero) {
                issues_found += 1;
            }

            // Check if triangles share more than an edge (would indicate overlap)
            for (j, tri2) in triangles.iter().enumerate() {
                if i >= j {
                    continue;
                }

                let mut shared_vertices = 0;
                for &v1 in tri1 {
                    for &v2 in tri2 {
                        if v1 == v2 {
                            shared_vertices += 1;
                        }
                    }
                }

                // Triangles can share at most 2 vertices (an edge)
                // If they share 3 vertices, they're the same triangle (shouldn't happen)
                if shared_vertices > 2 {
                    issues_found += 1;
                }
            }
        }

        if issues_found > 0 {
            Some(ValidationResult {
                issue_count: issues_found,
                issue: IssueType::SelfIntersection,
            })
        } else {
            None
        }
    }
}
