use core::f64;
use std::{collections::HashMap, sync::Arc};

use once_cell::sync::Lazy;
use reearth_flow_geometry::types::{
    coordinate::Coordinate3D, line_string::LineString3D, triangular_mesh::TriangularMesh,
};
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, Expr, Feature, GeometryValue};
use rhai::AST;
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
        Some(schemars::schema_for!(SolidBoundaryValidatorParam))
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
        ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let params: SolidBoundaryValidatorParam = if let Some(with_val) = with.clone() {
            let value: Value = serde_json::to_value(with_val).map_err(|e| {
                GeometryProcessorError::SolidBoundaryValidatorFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                GeometryProcessorError::SolidBoundaryValidatorFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(GeometryProcessorError::SolidBoundaryValidatorFactory(
                "Missing required parameter `with` containing `tolerance`".to_string(),
            )
            .into());
        };

        let expr_engine = Arc::clone(&ctx.expr_engine);
        let tolerance_ast = expr_engine
            .compile(params.tolerance.as_ref())
            .map_err(|e| {
                GeometryProcessorError::SolidBoundaryValidatorFactory(format!(
                    "Failed to compile tolerance expression: {e:?}"
                ))
            })?;

        let processor = SolidBoundaryValidator {
            global_params: with,
            tolerance_ast,
        };
        Ok(Box::new(processor))
    }
}

/// # Solid Boundary Validator Parameters
/// Configure validation parameters for solid boundary geometry
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SolidBoundaryValidatorParam {
    /// # Tolerance
    /// Tolerance value for geometry operations (as an expression evaluating to f64).
    /// Used for vertex merging and face triangulation.
    pub tolerance: Expr,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ValidationResult {
    issue_count: usize,
    issue: IssueType,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
enum IssueType {
    NotA2Manifold,
    NonOrientable,
    WrongFaceOrientation,
    NotConnected,
    SelfIntersection,
    SurfaceIssue,
}

/// # Solid Boundary Validator
/// Configure which validation checks to perform on feature geometries
#[derive(Debug, Clone)]
pub struct SolidBoundaryValidator {
    global_params: Option<HashMap<String, Value>>,
    tolerance_ast: AST,
}

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

        // Evaluate tolerance expression at runtime with feature context
        let tolerance = self.evaluate_tolerance(feature, &ctx)?;

        if geometry.is_empty() {
            fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
            return Ok(());
        }

        // Extract solid faces from feature
        let faces: Vec<reearth_flow_geometry::types::line_string::LineString> = match &geometry
            .value
        {
            GeometryValue::FlowGeometry3D(geom) => {
                if let Some(solid) = geom.as_solid() {
                    solid
                        .all_faces()
                        .into_iter()
                        .map(|f| {
                            f.0.iter()
                                .map(|v| Coordinate3D::new__(v.x, v.y, v.z))
                                .collect()
                        })
                        .collect::<Vec<_>>()
                } else {
                    // Not a solid geometry.
                    fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
                    return Ok(());
                }
            }
            GeometryValue::CityGmlGeometry(gml_geom) => {
                if gml_geom.gml_geometries.len() > 1 {
                    return Err(Box::new(GeometryProcessorError::SolidBoundaryValidatorFactory(
                        "Multiple geometries detected, but only one solid can be validated at a time.".to_string(),
                    )));
                }

                let Some(geom) = gml_geom.gml_geometries.first() else {
                    // No geometry given.
                    fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
                    return Ok(());
                };

                if geom.ty.name() != "Solid" {
                    // Not a solid geometry.
                    fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
                    return Ok(());
                }

                let mut polygons = Vec::new();
                for p in &geom.polygons {
                    polygons.push(p.clone().into_merged_contour(Some(tolerance))?);
                }
                polygons
            }
            _ => {
                // Not a solid geometry, send to rejected port
                fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
                return Ok(());
            }
        };

        // Extract vertices, edges, and triangles from the solid
        // Use a much smaller tolerance for vertex merging than the geometric tolerance.
        // The geometric tolerance can cause non-adjacent but spatially close vertices
        // to merge, corrupting the topology and producing false manifold violations.
        let vertex_merge_tolerance = tolerance * 0.01;
        let mesh = match TriangularMesh::from_faces(&faces, Some(vertex_merge_tolerance)) {
            Ok(mesh) => mesh,
            Err(_) => {
                // Some faces failed triangulation (e.g., non-planar faces).
                // Retry with only faces that can be triangulated.
                // Missing faces will be caught by subsequent validation checks.
                let valid_faces: Vec<_> = faces
                    .iter()
                    .filter(|face| {
                        TriangularMesh::from_faces(&[(*face).clone()], Some(vertex_merge_tolerance))
                            .is_ok()
                    })
                    .cloned()
                    .collect();
                match TriangularMesh::from_faces(&valid_faces, Some(vertex_merge_tolerance)) {
                    Ok(mesh) => mesh,
                    Err(_) => {
                        // Even valid faces can't build a mesh
                        let mut feature = feature.clone();
                        feature.attributes_mut().insert(
                            Attribute::new("solid_boundary_issues"),
                            AttributeValue::from(
                                serde_json::to_value(&ValidationResult {
                                    issue_count: 1,
                                    issue: IssueType::SurfaceIssue,
                                })
                                .unwrap(),
                            ),
                        );
                        fw.send(
                            ctx.new_with_feature_and_port(feature.clone(), FAILED_PORT.clone()),
                        );
                        return Ok(());
                    }
                }
            }
        };
        if mesh.is_empty() {
            // If triangulation fails, send to rejected port
            fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
            return Ok(());
        }

        // Build deduplicated vertex index mapping for original polygon edges.
        // This is used for manifold and orientation checks on the original
        // (non-triangulated) boundary edges to avoid false positives from
        // internal diagonal edges introduced by ear-clipping triangulation.
        let (_vertices, indexed_faces) = Self::map_faces_to_vertex_indices(&faces);

        // Check manifold condition on original polygon edges
        let result = if let Some(result) = Self::check_manifold(&indexed_faces) {
            result
        }
        // Check connectivity
        else if !mesh.is_connected() {
            ValidationResult {
                issue_count: 1,
                issue: IssueType::NotConnected,
            }
        }
        // Check orientability
        else if !mesh.is_orientable() {
            ValidationResult {
                issue_count: 1,
                issue: IssueType::NonOrientable,
            }
        }
        // Check self-intersection
        else if let Some(result) = {
            let len = mesh.self_intersection().len();
            if len > 0 {
                Some(ValidationResult {
                    issue_count: len,
                    issue: IssueType::SelfIntersection,
                })
            } else {
                None
            }
        } {
            result
        }
        // Check face orientation
        else if let Some(result) = Self::check_orientation(&indexed_faces) {
            result
        }
        // No issues found
        else {
            fw.send(ctx.new_with_feature_and_port(feature.clone(), SUCCESS_PORT.clone()));
            return Ok(());
        };

        // Add validation results to feature attributes if there are issues
        let mut feature: reearth_flow_types::Feature = feature.clone();
        feature.attributes_mut().insert(
            Attribute::new("solid_boundary_issues"),
            AttributeValue::from(serde_json::to_value(&result).unwrap()),
        );
        // Send to failed port if there are validation issues
        fw.send(ctx.new_with_feature_and_port(feature, FAILED_PORT.clone()));

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
        "SolidBoundaryValidator"
    }
}

impl SolidBoundaryValidator {
    /// Evaluate the tolerance expression at runtime with feature context
    fn evaluate_tolerance(
        &self,
        feature: &Feature,
        ctx: &ExecutorContext,
    ) -> Result<f64, BoxedError> {
        let expr_engine = Arc::clone(&ctx.expr_engine);
        let scope = feature.new_scope(expr_engine.clone(), &self.global_params);
        scope.eval_ast::<f64>(&self.tolerance_ast).map_err(|e| {
            GeometryProcessorError::SolidBoundaryValidatorFactory(format!(
                "Failed to evaluate tolerance expression: {e:?}"
            ))
            .into()
        })
    }

    /// Deduplicate vertices across all faces and map each face to vertex indices.
    fn map_faces_to_vertex_indices(
        faces: &[LineString3D<f64>],
    ) -> (Vec<Coordinate3D<f64>>, Vec<Vec<usize>>) {
        let epsilon = 1e-8;
        let mut vertices: Vec<Coordinate3D<f64>> = Vec::new();
        let mut indexed_faces: Vec<Vec<usize>> = Vec::new();

        for face in faces {
            let mut face_indices = Vec::with_capacity(face.0.len());
            for v in face.iter() {
                let idx = vertices
                    .iter()
                    .position(|&vv| (vv - *v).norm() < epsilon)
                    .unwrap_or_else(|| {
                        vertices.push(*v);
                        vertices.len() - 1
                    });
                face_indices.push(idx);
            }
            indexed_faces.push(face_indices);
        }

        (vertices, indexed_faces)
    }

    /// Check manifold condition on original polygon boundary edges.
    ///
    /// Every undirected edge in a closed (watertight) solid must be shared by
    /// exactly 2 faces. This operates on the original polygon edges rather than
    /// triangulated edges, avoiding false positives from internal diagonal edges
    /// introduced by ear-clipping triangulation.
    fn check_manifold(indexed_faces: &[Vec<usize>]) -> Option<ValidationResult> {
        let mut edge_counts: HashMap<[usize; 2], usize> = HashMap::new();

        for face in indexed_faces {
            for edge in face.windows(2) {
                // Skip degenerate edges where both endpoints map to the same vertex
                if edge[0] == edge[1] {
                    continue;
                }
                let key = if edge[0] < edge[1] {
                    [edge[0], edge[1]]
                } else {
                    [edge[1], edge[0]]
                };
                *edge_counts.entry(key).or_insert(0) += 1;
            }
        }

        let violation_count = edge_counts.values().filter(|&&c| c != 2).count();
        if violation_count == 0 {
            None
        } else {
            Some(ValidationResult {
                issue_count: violation_count,
                issue: IssueType::NotA2Manifold,
            })
        }
    }

    fn check_orientation(indexed_faces: &[Vec<usize>]) -> Option<ValidationResult> {
        if indexed_faces.is_empty() {
            return None;
        }
        let mut directed_edges = Vec::new();
        for face in indexed_faces {
            for edge in face.windows(2) {
                if edge[0] < edge[1] {
                    let edge = [edge[0], edge[1]];
                    directed_edges.push((edge, true));
                } else {
                    let edge = [edge[1], edge[0]];
                    directed_edges.push((edge, false));
                }
            }
        }
        directed_edges.sort_by_key(|(e, _)| *e);
        let issue_count = directed_edges
            .chunks_exact(2)
            .filter(|chunk| chunk[0].0 != chunk[1].0 || chunk[0].1 == chunk[1].1)
            .count();
        if issue_count == 0 {
            None
        } else {
            Some(ValidationResult {
                issue_count,
                issue: IssueType::WrongFaceOrientation,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: create a LineString3D from a slice of (x, y, z) tuples.
    /// The ring is automatically closed (first vertex appended at end).
    fn ring(pts: &[(f64, f64, f64)]) -> LineString3D<f64> {
        let mut coords: Vec<Coordinate3D<f64>> = pts
            .iter()
            .map(|&(x, y, z)| Coordinate3D::new__(x, y, z))
            .collect();
        if let Some(&first) = coords.first() {
            coords.push(first);
        }
        LineString3D::new(coords)
    }

    /// Unit cube faces (6 quads). Each face is a closed ring (5 vertices).
    fn cube_faces() -> Vec<LineString3D<f64>> {
        vec![
            // bottom (z=0), CCW viewed from outside (CW from above)
            ring(&[
                (0.0, 0.0, 0.0),
                (1.0, 0.0, 0.0),
                (1.0, 1.0, 0.0),
                (0.0, 1.0, 0.0),
            ]),
            // top (z=1), CCW viewed from outside
            ring(&[
                (0.0, 0.0, 1.0),
                (0.0, 1.0, 1.0),
                (1.0, 1.0, 1.0),
                (1.0, 0.0, 1.0),
            ]),
            // front (y=0)
            ring(&[
                (0.0, 0.0, 0.0),
                (0.0, 0.0, 1.0),
                (1.0, 0.0, 1.0),
                (1.0, 0.0, 0.0),
            ]),
            // back (y=1)
            ring(&[
                (0.0, 1.0, 0.0),
                (1.0, 1.0, 0.0),
                (1.0, 1.0, 1.0),
                (0.0, 1.0, 1.0),
            ]),
            // left (x=0)
            ring(&[
                (0.0, 0.0, 0.0),
                (0.0, 1.0, 0.0),
                (0.0, 1.0, 1.0),
                (0.0, 0.0, 1.0),
            ]),
            // right (x=1)
            ring(&[
                (1.0, 0.0, 0.0),
                (1.0, 0.0, 1.0),
                (1.0, 1.0, 1.0),
                (1.0, 1.0, 0.0),
            ]),
        ]
    }

    /// Tetrahedron (4 triangular faces).
    fn tetrahedron_faces() -> Vec<LineString3D<f64>> {
        let a = (0.0, 0.0, 0.0);
        let b = (1.0, 0.0, 0.0);
        let c = (0.5, 1.0, 0.0);
        let d = (0.5, 0.5, 1.0);
        vec![
            ring(&[a, c, b]), // bottom
            ring(&[a, b, d]), // front
            ring(&[b, c, d]), // right
            ring(&[c, a, d]), // left
        ]
    }

    #[test]
    fn cube_manifold_passes() {
        let faces = cube_faces();
        let (_verts, indexed) = SolidBoundaryValidator::map_faces_to_vertex_indices(&faces);
        assert!(
            SolidBoundaryValidator::check_manifold(&indexed).is_none(),
            "Closed cube should pass manifold check"
        );
    }

    #[test]
    fn tetrahedron_manifold_passes() {
        let faces = tetrahedron_faces();
        let (_verts, indexed) = SolidBoundaryValidator::map_faces_to_vertex_indices(&faces);
        assert!(
            SolidBoundaryValidator::check_manifold(&indexed).is_none(),
            "Closed tetrahedron should pass manifold check"
        );
    }

    #[test]
    fn open_box_manifold_fails() {
        // Remove top face from cube → open box with boundary edges shared by only 1 face
        let mut faces = cube_faces();
        faces.remove(1); // remove top
        let (_verts, indexed) = SolidBoundaryValidator::map_faces_to_vertex_indices(&faces);
        let result = SolidBoundaryValidator::check_manifold(&indexed);
        assert!(
            result.is_some(),
            "Open box (missing top face) should fail manifold check"
        );
        let r = result.unwrap();
        assert_eq!(r.issue, IssueType::NotA2Manifold);
        assert!(r.issue_count > 0);
    }

    #[test]
    fn cube_orientation_passes() {
        let faces = cube_faces();
        let (_verts, indexed) = SolidBoundaryValidator::map_faces_to_vertex_indices(&faces);
        assert!(
            SolidBoundaryValidator::check_orientation(&indexed).is_none(),
            "Consistently oriented cube should pass orientation check"
        );
    }
}
