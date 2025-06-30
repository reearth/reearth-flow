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
use reearth_flow_types::{Expr, GeometryValue};
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
        "Replaces a three Dimension box with a polygon."
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
        ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let params: ThreeDimensionRotatorParam = if let Some(with) = with.clone() {
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
        let expr_engine = Arc::clone(&ctx.expr_engine);
        let angle_degree = expr_engine
            .compile(params.angle_degree.as_ref())
            .map_err(|e| GeometryProcessorError::ThreeDimensionRotatorFactory(format!("{e:?}")))?;
        let origin_x = expr_engine
            .compile(params.origin_x.as_ref())
            .map_err(|e| GeometryProcessorError::ThreeDimensionRotatorFactory(format!("{e:?}")))?;
        let origin_y = expr_engine
            .compile(params.origin_y.as_ref())
            .map_err(|e| GeometryProcessorError::ThreeDimensionRotatorFactory(format!("{e:?}")))?;
        let origin_z = expr_engine
            .compile(params.origin_z.as_ref())
            .map_err(|e| GeometryProcessorError::ThreeDimensionRotatorFactory(format!("{e:?}")))?;
        let direction_x = expr_engine
            .compile(params.direction_x.as_ref())
            .map_err(|e| GeometryProcessorError::ThreeDimensionRotatorFactory(format!("{e:?}")))?;
        let direction_y = expr_engine
            .compile(params.direction_y.as_ref())
            .map_err(|e| GeometryProcessorError::ThreeDimensionRotatorFactory(format!("{e:?}")))?;
        let direction_z = expr_engine
            .compile(params.direction_z.as_ref())
            .map_err(|e| GeometryProcessorError::ThreeDimensionRotatorFactory(format!("{e:?}")))?;
        Ok(Box::new(ThreeDimensionRotator {
            global_params: with,
            angle_degree,
            origin_x,
            origin_y,
            origin_z,
            direction_x,
            direction_y,
            direction_z,
        }))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ThreeDimensionRotatorParam {
    angle_degree: Expr,
    origin_x: Expr,
    origin_y: Expr,
    origin_z: Expr,
    direction_x: Expr,
    direction_y: Expr,
    direction_z: Expr,
}

#[derive(Debug, Clone)]
pub struct ThreeDimensionRotator {
    global_params: Option<HashMap<String, serde_json::Value>>,
    angle_degree: rhai::AST,
    origin_x: rhai::AST,
    origin_y: rhai::AST,
    origin_z: rhai::AST,
    direction_x: rhai::AST,
    direction_y: rhai::AST,
    direction_z: rhai::AST,
}

impl Processor for ThreeDimensionRotator {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let scope = feature.new_scope(ctx.expr_engine.clone(), &self.global_params);
        let angle_degree = scope.eval_ast::<f64>(&self.angle_degree)?;
        let origin_x = scope.eval_ast::<f64>(&self.origin_x)?;
        let origin_y = scope.eval_ast::<f64>(&self.origin_y)?;
        let origin_z = scope.eval_ast::<f64>(&self.origin_z)?;
        let direction_x = scope.eval_ast::<f64>(&self.direction_x)?;
        let direction_y = scope.eval_ast::<f64>(&self.direction_y)?;
        let direction_z = scope.eval_ast::<f64>(&self.direction_z)?;
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
                    let mut geometry = geometry.clone();
                    geometry.value = GeometryValue::FlowGeometry3D(rotate);
                    Some(geometry)
                } else {
                    None
                }
            }
            _ => None,
        };

        if let Some(geometry) = geometry {
            let mut feature = ctx.feature.clone();
            feature.geometry = geometry;
            fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
        } else {
            fw.send(ctx.new_with_feature_and_port(ctx.feature.clone(), REJECTED_PORT.clone()));
        }
        Ok(())
    }

    fn finish(&self, _ctx: NodeContext, _fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "ThreeDimensionRotator"
    }
}
