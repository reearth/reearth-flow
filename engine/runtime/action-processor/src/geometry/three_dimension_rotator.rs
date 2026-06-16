use std::{collections::HashMap, sync::Arc};

use reearth_flow_geometry::{
    algorithm::rotate::{query::RotateQuery3D, rotate_3d::Rotate3D},
    types::point::Point3D,
};
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT, REJECTED_PORT},
};
use reearth_flow_types::{Code, CodeType, CompiledCode, GeometryValue};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::GeometryProcessorError;

#[derive(Debug, Clone, Default)]
pub struct ThreeDimensionRotatorFactory;

impl ProcessorFactory for ThreeDimensionRotatorFactory {
    fn name(&self) -> &str {
        "ThreeDimensionRotator"
    }

    fn description(&self) -> &str {
        "Rotate 3D Geometry Around Arbitrary Axis"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(ThreeDimensionRotatorParam))
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
        let params: ThreeDimensionRotatorParam = if let Some(with) = with {
            let value = serde_json::to_value(with).map_err(|e| {
                GeometryProcessorError::ThreeDimensionRotatorFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                GeometryProcessorError::ThreeDimensionRotatorFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(GeometryProcessorError::ThreeDimensionRotatorFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        let compile = |expr: Code<{ CodeType::FlowExpr as u32 }>| {
            expr.compile()
                .map_err(|e| GeometryProcessorError::ThreeDimensionRotatorFactory(format!("{e:?}")))
        };
        Ok(Box::new(ThreeDimensionRotator {
            angle_degree: compile(params.angle_degree)?,
            origin_x: compile(params.origin_x)?,
            origin_y: compile(params.origin_y)?,
            origin_z: compile(params.origin_z)?,
            direction_x: compile(params.direction_x)?,
            direction_y: compile(params.direction_y)?,
            direction_z: compile(params.direction_z)?,
        }))
    }
}

/// # 3D Rotator Parameters
/// Configure the 3D rotation parameters including axis, origin point, and angle
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ThreeDimensionRotatorParam {
    /// # Angle in Degrees
    /// Rotation angle in degrees around the specified axis
    angle_degree: Code<{ CodeType::FlowExpr as u32 }>,
    /// # Origin X
    /// X coordinate of the rotation origin point
    origin_x: Code<{ CodeType::FlowExpr as u32 }>,
    /// # Origin Y
    /// Y coordinate of the rotation origin point
    origin_y: Code<{ CodeType::FlowExpr as u32 }>,
    /// # Origin Z
    /// Z coordinate of the rotation origin point
    origin_z: Code<{ CodeType::FlowExpr as u32 }>,
    /// # Direction X
    /// X component of the rotation axis direction vector
    direction_x: Code<{ CodeType::FlowExpr as u32 }>,
    /// # Direction Y
    /// Y component of the rotation axis direction vector
    direction_y: Code<{ CodeType::FlowExpr as u32 }>,
    /// # Direction Z
    /// Z component of the rotation axis direction vector
    direction_z: Code<{ CodeType::FlowExpr as u32 }>,
}

#[derive(Debug, Clone)]
pub struct ThreeDimensionRotator {
    angle_degree: CompiledCode,
    origin_x: CompiledCode,
    origin_y: CompiledCode,
    origin_z: CompiledCode,
    direction_x: CompiledCode,
    direction_y: CompiledCode,
    direction_z: CompiledCode,
}

impl Processor for ThreeDimensionRotator {
    #[cfg(not(feature = "new-geometry"))]
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let env_vars = ctx.expr_engine.vars().clone();
        let eval_f64 = |code: &CompiledCode| -> Result<f64, BoxedError> {
            code.eval(feature, env_vars.clone())
                .map_err(|e| GeometryProcessorError::ThreeDimensionRotator(format!("{e:?}")).into())
                .and_then(|av| {
                    av.as_f64().ok_or_else(|| {
                        GeometryProcessorError::ThreeDimensionRotator(
                            "expression must evaluate to a number".to_string(),
                        )
                        .into()
                    })
                })
        };
        let angle_degree = eval_f64(&self.angle_degree)?;
        let origin_x = eval_f64(&self.origin_x)?;
        let origin_y = eval_f64(&self.origin_y)?;
        let origin_z = eval_f64(&self.origin_z)?;
        let direction_x = eval_f64(&self.direction_x)?;
        let direction_y = eval_f64(&self.direction_y)?;
        let direction_z = eval_f64(&self.direction_z)?;
        let geometry = &feature.geometry;
        let geometry = match &geometry.value {
            GeometryValue::FlowGeometry3D(geos) => {
                if let Some(rotate_query) = RotateQuery3D::from_angle_and_direction(
                    angle_degree,
                    Point3D::new_(direction_x, direction_y, direction_z),
                ) {
                    let rotate = geos.rotate_3d(
                        rotate_query,
                        Some(Point3D::new_(origin_x, origin_y, origin_z)),
                    );
                    let mut geom = (**geometry).clone();
                    geom.value = GeometryValue::FlowGeometry3D(rotate);
                    Some(geom)
                } else {
                    None
                }
            }
            _ => None,
        };

        if let Some(geometry) = geometry {
            let mut feature = ctx.feature.clone();
            feature.geometry = Arc::new(geometry);
            fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
        } else {
            fw.send(ctx.new_with_feature_and_port(ctx.feature.clone(), REJECTED_PORT.clone()));
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
        "ThreeDimensionRotator"
    }
}
