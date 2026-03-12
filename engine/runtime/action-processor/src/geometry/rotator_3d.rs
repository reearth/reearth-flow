use std::{collections::HashMap, sync::Arc};

use nalgebra::Vector3;
use reearth_flow_geometry::{
    algorithm::centroid::Centroid,
    types::{
        coordinate::Coordinate3D, line_string::LineString3D, point::Point3D, polygon::Polygon3D,
    },
};
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT, REJECTED_PORT},
};
use reearth_flow_types::{Expr, GeometryValue};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::GeometryProcessorError;

#[derive(Debug, Clone, Default)]
pub struct Rotator3DFactory;

impl ProcessorFactory for Rotator3DFactory {
    fn name(&self) -> &str {
        "Rotator3D"
    }

    fn description(&self) -> &str {
        "Rotate a 3D polygon using from/to vectors or axis-angle specification"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(Rotator3DParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Geometry"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone(), REJECTED_PORT.clone()]
    }

    fn build(
        &self,
        ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let params: Rotator3DParam = if let Some(with) = with.clone() {
            let value = serde_json::to_value(with).map_err(|e| {
                GeometryProcessorError::Rotator3DFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                GeometryProcessorError::Rotator3DFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(GeometryProcessorError::Rotator3DFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };

        let expr_engine = Arc::clone(&ctx.expr_engine);
        let compile = |expr: &Expr| {
            expr_engine
                .compile(expr.as_ref())
                .map_err(|e| GeometryProcessorError::Rotator3DFactory(format!("{e:?}")))
        };

        let rotation = match &params.rotation {
            RotationParam::FromToVectors(p) => RotationAST::FromToVectors {
                from_x: compile(&p.from_x)?,
                from_y: compile(&p.from_y)?,
                from_z: compile(&p.from_z)?,
                to_x: compile(&p.to_x)?,
                to_y: compile(&p.to_y)?,
                to_z: compile(&p.to_z)?,
            },
            RotationParam::AxisAngle(p) => RotationAST::AxisAngle {
                axis_x: compile(&p.axis_x)?,
                axis_y: compile(&p.axis_y)?,
                axis_z: compile(&p.axis_z)?,
                angle: compile(&p.angle)?,
            },
        };

        Ok(Box::new(Rotator3D {
            global_params: with,
            rotation,
        }))
    }
}

/// # Rotator3D Parameters
/// Configure the rotation for a 3D polygon
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Rotator3DParam {
    /// # Rotation
    /// The rotation specification: either from/to vectors or axis-angle
    rotation: RotationParam,
}

/// Rotation specification
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum RotationParam {
    /// Rotation defined by two vectors (from and to)
    FromToVectors(FromToVectorsParam),
    /// Rotation defined by an axis and angle
    AxisAngle(AxisAngleParam),
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct FromToVectorsParam {
    /// X component of the source direction vector
    pub from_x: Expr,
    /// Y component of the source direction vector
    pub from_y: Expr,
    /// Z component of the source direction vector
    pub from_z: Expr,
    /// X component of the target direction vector
    pub to_x: Expr,
    /// Y component of the target direction vector
    pub to_y: Expr,
    /// Z component of the target direction vector
    pub to_z: Expr,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AxisAngleParam {
    /// X component of the rotation axis
    pub axis_x: Expr,
    /// Y component of the rotation axis
    pub axis_y: Expr,
    /// Z component of the rotation axis
    pub axis_z: Expr,
    /// Rotation angle in degrees
    pub angle: Expr,
}

#[derive(Debug, Clone)]
enum RotationAST {
    FromToVectors {
        from_x: rhai::AST,
        from_y: rhai::AST,
        from_z: rhai::AST,
        to_x: rhai::AST,
        to_y: rhai::AST,
        to_z: rhai::AST,
    },
    AxisAngle {
        axis_x: rhai::AST,
        axis_y: rhai::AST,
        axis_z: rhai::AST,
        angle: rhai::AST,
    },
}

#[derive(Debug, Clone)]
pub struct Rotator3D {
    global_params: Option<HashMap<String, serde_json::Value>>,
    rotation: RotationAST,
}

impl Processor for Rotator3D {
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

        let scope = feature.new_scope(ctx.expr_engine.clone(), &self.global_params);
        let rotation_matrix = match &self.rotation {
            RotationAST::FromToVectors {
                from_x,
                from_y,
                from_z,
                to_x,
                to_y,
                to_z,
            } => {
                let from = Vector3::new(
                    scope.eval_ast::<f64>(from_x)?,
                    scope.eval_ast::<f64>(from_y)?,
                    scope.eval_ast::<f64>(from_z)?,
                );
                let to = Vector3::new(
                    scope.eval_ast::<f64>(to_x)?,
                    scope.eval_ast::<f64>(to_y)?,
                    scope.eval_ast::<f64>(to_z)?,
                );
                rotation_from_vectors(from, to)
            }
            RotationAST::AxisAngle {
                axis_x,
                axis_y,
                axis_z,
                angle,
            } => {
                let axis = Vector3::new(
                    scope.eval_ast::<f64>(axis_x)?,
                    scope.eval_ast::<f64>(axis_y)?,
                    scope.eval_ast::<f64>(axis_z)?,
                );
                let angle_deg = scope.eval_ast::<f64>(angle)?;
                rotation_from_axis_angle(axis, angle_deg)
            }
        };

        let Some(rotation_matrix) = rotation_matrix else {
            fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
            return Ok(());
        };

        match &geometry.value {
            GeometryValue::FlowGeometry3D(
                reearth_flow_geometry::types::geometry::Geometry3D::Polygon(polygon),
            ) => {
                let centroid = polygon.centroid();
                let rotated = rotate_polygon(polygon, &rotation_matrix, centroid);
                let mut feature = feature.clone();
                feature.geometry_mut().value = GeometryValue::FlowGeometry3D(
                    reearth_flow_geometry::types::geometry::Geometry3D::Polygon(rotated),
                );
                fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
            }
            GeometryValue::CityGmlGeometry(city_gml) => {
                if city_gml.gml_geometries.len() != 1
                    || city_gml.gml_geometries[0].polygons.len() != 1
                {
                    fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
                    return Ok(());
                }
                let polygon = &city_gml.gml_geometries[0].polygons[0];
                let centroid = polygon.centroid();
                let rotated = rotate_polygon(polygon, &rotation_matrix, centroid);
                let mut new_city_gml = city_gml.clone();
                new_city_gml.gml_geometries[0].polygons[0] = rotated;
                let mut feature = feature.clone();
                feature.geometry_mut().value = GeometryValue::CityGmlGeometry(new_city_gml);
                fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
            }
            _ => {
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
        "Rotator3D"
    }
}

/// Build a rotation matrix that maps vector `from` to vector `to`.
/// Returns None if either vector is zero.
fn rotation_from_vectors(from: Vector3<f64>, to: Vector3<f64>) -> Option<nalgebra::Rotation3<f64>> {
    let a = from.try_normalize(1e-10)?;
    let b = to.try_normalize(1e-10)?;

    let cross = a.cross(&b);
    let cross_norm = cross.norm();
    if cross_norm < 1e-10 {
        // Vectors are parallel (same or opposite direction).
        // For same direction: identity. For opposite: 180-degree rotation around any perpendicular axis.
        if a.dot(&b) > 0.0 {
            return Some(nalgebra::Rotation3::identity());
        }
        // Opposite vectors: pick a perpendicular axis
        let perp = if a.x.abs() < 0.9 {
            Vector3::x()
        } else {
            Vector3::y()
        };
        let axis = a.cross(&perp).normalize();
        return Some(nalgebra::Rotation3::from_axis_angle(
            &nalgebra::Unit::new_normalize(axis),
            std::f64::consts::PI,
        ));
    }

    let angle = a.dot(&b).clamp(-1.0, 1.0).acos();
    Some(nalgebra::Rotation3::from_axis_angle(
        &nalgebra::Unit::new_normalize(cross),
        angle,
    ))
}

/// Build a rotation matrix from axis-angle (angle in degrees).
fn rotation_from_axis_angle(
    axis: Vector3<f64>,
    angle_degrees: f64,
) -> Option<nalgebra::Rotation3<f64>> {
    let _ = axis.try_normalize(1e-10)?;
    Some(nalgebra::Rotation3::from_axis_angle(
        &nalgebra::Unit::new_normalize(axis),
        angle_degrees.to_radians(),
    ))
}

/// Rotate a coordinate around an origin using pure Euclidean rotation (no geographic conversion).
fn rotate_coordinate(
    coord: &Coordinate3D<f64>,
    rotation: &nalgebra::Rotation3<f64>,
    origin: &nalgebra::Point3<f64>,
) -> Coordinate3D<f64> {
    let p = nalgebra::Point3::new(coord.x, coord.y, coord.z);
    let translated = p - origin;
    let rotated = rotation * translated + origin.coords;
    Coordinate3D::new__(rotated.x, rotated.y, rotated.z)
}

/// Rotate a line string around an origin.
fn rotate_line_string(
    ls: &LineString3D<f64>,
    rotation: &nalgebra::Rotation3<f64>,
    origin: &nalgebra::Point3<f64>,
) -> LineString3D<f64> {
    LineString3D::new(
        ls.coords()
            .map(|c| rotate_coordinate(c, rotation, origin))
            .collect(),
    )
}

/// Rotate a polygon around its centroid (or given origin) using pure Euclidean rotation.
fn rotate_polygon(
    polygon: &Polygon3D<f64>,
    rotation: &nalgebra::Rotation3<f64>,
    centroid: Option<Point3D<f64>>,
) -> Polygon3D<f64> {
    let origin = centroid
        .map(|c| nalgebra::Point3::new(c.x(), c.y(), c.z()))
        .unwrap_or(nalgebra::Point3::origin());

    let rotated_exterior = rotate_line_string(polygon.exterior(), rotation, &origin);
    let rotated_interiors = polygon
        .interiors()
        .iter()
        .map(|ls| rotate_line_string(ls, rotation, &origin))
        .collect();

    Polygon3D::new(rotated_exterior, rotated_interiors)
}
