use core::f64;
use std::collections::HashMap;

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
use reearth_flow_types::{Attribute, AttributeValue, GeometryValue};
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
        None
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
        _with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let processor = SolidBoundaryValidator {};
        Ok(Box::new(processor))
    }
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
}

/// # Solid Boundary Validator
/// Configure which validation checks to perform on feature geometries
#[derive(Debug, Clone)]
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

        // Extract solid faces from feature
        let faces = match &geometry.value {
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
                    return Err(Box::new(GeometryProcessorError::SoilidBoundaryValidatorFactory(
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

                geom.polygons
                    .iter()
                    .map(|p| p.clone().into_merged_contour())
                    .collect::<Vec<_>>()
            }
            _ => {
                // Not a solid geometry, send to rejected port
                fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
                return Ok(());
            }
        };

        // Extract vertices, edges, and triangles from the solid
        let mesh = TriangularMesh::from_faces(&faces);
        if mesh.is_empty() {
            // If triangulation fails, send to rejected port
            fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
            return Ok(());
        }

        // Check manifold condition
        let result = if let Some(result) = {
            let len = mesh.edges_violating_manifold_condition().len();
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
        else if let Some(result) = Self::check_orientation(&faces, mesh.get_vertices()) {
            result
        }
        // No issues found
        else {
            fw.send(ctx.new_with_feature_and_port(feature.clone(), SUCCESS_PORT.clone()));
            return Ok(());
        };

        // Add validation results to feature attributes if there are issues
        let mut feature: reearth_flow_types::Feature = feature.clone();
        feature.attributes.insert(
            Attribute::new("solid_boundary_issues"),
            AttributeValue::from(serde_json::to_value(&result).unwrap()),
        );
        // Send to failed port if there are validation issues
        fw.send(ctx.new_with_feature_and_port(feature, FAILED_PORT.clone()));

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
    fn check_orientation(
        faces: &[LineString3D<f64>],
        vertices: &[Coordinate3D<f64>],
    ) -> Option<ValidationResult> {
        if faces.is_empty() {
            return None;
        }
        let mut directed_edges = Vec::new();
        let faces = &faces
            .iter()
            .map(|f| {
                f.iter()
                    .map(|v| vertices.iter().position(|&vv| vv == *v).unwrap())
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        for face in faces {
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
