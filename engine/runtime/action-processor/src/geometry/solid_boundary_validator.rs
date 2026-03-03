use core::f64;
use std::{collections::HashMap, sync::Arc};

use once_cell::sync::Lazy;
use reearth_flow_geometry::types::{
    coordinate::Coordinate3D, line_string::LineString3D, polygon::Polygon3D,
    triangular_mesh::TriangularMesh,
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

        // Extract polygons and build triangular mesh from feature geometry
        let (mesh, polygons) = match &geometry.value {
            GeometryValue::FlowGeometry3D(geom) => {
                if let Some(solid) = geom.as_solid() {
                    let polygons: Vec<Polygon3D<f64>> = solid
                        .all_faces()
                        .into_iter()
                        .map(|f| {
                            let ring: LineString3D<f64> =
                                f.0.iter()
                                    .map(|v| Coordinate3D::new__(v.x, v.y, v.z))
                                    .collect();
                            Polygon3D::new(ring, vec![])
                        })
                        .collect();
                    let faces: Vec<LineString3D<f64>> =
                        polygons.iter().map(|p| p.exterior().clone()).collect();
                    let mesh = TriangularMesh::from_faces(&faces, Some(tolerance));
                    (mesh, polygons)
                } else {
                    fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
                    return Ok(());
                }
            }
            GeometryValue::CityGmlGeometry(gml_geom) => {
                if gml_geom.gml_geometries.len() > 1 {
                    return Err(Box::new(
                        GeometryProcessorError::SolidBoundaryValidatorFactory(
                            "Multiple geometries detected, but only one solid can be validated at a time.".to_string(),
                        ),
                    ));
                }

                let Some(geom) = gml_geom.gml_geometries.first() else {
                    fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
                    return Ok(());
                };

                if geom.ty.name() != "Solid" {
                    fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
                    return Ok(());
                }

                let polygons: Vec<Polygon3D<f64>> = geom.polygons.clone();
                let mesh = TriangularMesh::try_from_polygons(polygons.clone(), Some(tolerance));
                (mesh, polygons)
            }
            _ => {
                fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
                return Ok(());
            }
        };

        let mesh = match mesh {
            Ok(mesh) => mesh,
            Err(_) => {
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
                fw.send(ctx.new_with_feature_and_port(feature.clone(), FAILED_PORT.clone()));
                return Ok(());
            }
        };
        if mesh.is_empty() {
            fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
            return Ok(());
        }

        // Check manifold condition
        let result = if let Some(result) = {
            let violating = mesh.edges_violating_manifold_condition();
            let len = violating.len();
            if len > 0 {
                Some(ValidationResult {
                    issue_count: len,
                    issue: IssueType::NotA2Manifold,
                })
            } else {
                None
            }
        } {
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
        else if let Some(result) = Self::check_orientation(&polygons, mesh.get_vertices()) {
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

    fn check_orientation(
        polygons: &[Polygon3D<f64>],
        vertices: &[Coordinate3D<f64>],
    ) -> Option<ValidationResult> {
        let epsilon = 1e-8;
        if polygons.is_empty() {
            return None;
        }

        // Map a 3D coordinate to its index in the shared vertex list.
        let vertex_index = |v: &Coordinate3D<f64>| {
            vertices
                .iter()
                .position(|&vv| (vv - *v).norm() < epsilon)
                .unwrap()
        };

        // Build directed edges from every ring of every polygon.
        // Interior hole rings are included so that edges shared between a hole
        // boundary and an adjacent wall polygon have matching directed-edge partners.
        let mut directed_edges: Vec<([usize; 2], bool)> = Vec::new();
        for polygon in polygons {
            let rings = std::iter::once(polygon.exterior()).chain(polygon.interiors().iter());
            for ring in rings {
                let indices: Vec<usize> = ring.iter().map(&vertex_index).collect();
                for w in indices.windows(2) {
                    if w[0] < w[1] {
                        directed_edges.push(([w[0], w[1]], true));
                    } else {
                        directed_edges.push(([w[1], w[0]], false));
                    }
                }
            }
        }
        directed_edges.sort_by_key(|(e, _)| *e);

        let mut issue_count = 0;
        for chunk in directed_edges.chunks_exact(2) {
            if chunk[0].0 != chunk[1].0 || chunk[0].1 == chunk[1].1 {
                issue_count += 1;
            }
        }

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
