use std::{collections::HashMap, sync::Arc};

use reearth_flow_geometry::types::geometry::Geometry3D as FlowGeometry3D;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, FEATURES_PORT, REJECTED_PORT},
};
use reearth_flow_types::{Code, CodeType, CompiledCode, Geometry, GeometryValue};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::GeometryProcessorError;

#[derive(Debug, Clone, Default)]
pub struct ExtruderFactory;

impl ProcessorFactory for ExtruderFactory {
    fn name(&self) -> &str {
        "Extruder"
    }

    fn description(&self) -> &str {
        "Extrudes a polygon geometry vertically by a given distance to produce a solid geometry."
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(ExtruderParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Geometry"]
    }

    fn tags(&self) -> &[&'static str] {
        &["3d"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![FEATURES_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![FEATURES_PORT.clone(), REJECTED_PORT.clone()]
    }

    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let params: ExtruderParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                GeometryProcessorError::ExtruderFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                GeometryProcessorError::ExtruderFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(GeometryProcessorError::ExtruderFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };

        let distance = params
            .distance
            .compile()
            .map_err(|e| GeometryProcessorError::ExtruderFactory(format!("{e:?}")))?;
        Ok(Box::new(Extruder { distance }))
    }
}

#[derive(Debug, Clone)]
pub struct Extruder {
    distance: CompiledCode,
}

/// # Extruder Parameters
/// Configure how far each polygon is extruded.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ExtruderParam {
    /// # Distance
    /// Height to extrude the polygon by, as a constant or an expression
    /// evaluated per feature.
    distance: Code<{ CodeType::FlowExpr as u32 }>,
}

impl Processor for Extruder {
    fn num_threads(&self) -> usize {
        2
    }

    #[cfg(not(feature = "new-geometry"))]
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let height = self
            .distance
            .eval_float(feature, ctx.env_vars.clone())
            .map_err(|e| {
                GeometryProcessorError::Extruder(format!("Failed to evaluate distance: {e:?}"))
            })?;

        // A feature this action cannot extrude is routed to `rejected` rather
        // than failing the job: one stray geometry should not stop the run.
        // A polygon with no elevation is rejected too — extruding it would have
        // to invent a base elevation. Use a Three Dimension Forcer upstream to
        // set that base deliberately.
        let reject = |reason: &str| {
            ctx.event_hub.debug_log(
                Some(ctx.error_span()),
                format!("extrude rejected: {reason}"),
            );
            fw.send(ctx.new_with_feature_and_port(ctx.feature.clone(), REJECTED_PORT.clone()));
        };

        let geom_inner = (*feature.geometry).clone();
        let GeometryValue::FlowGeometry3D(flow_geometry) = &geom_inner.value else {
            reject("geometry is absent or carries no elevation");
            return Ok(());
        };
        let FlowGeometry3D::Polygon(polygon) = flow_geometry else {
            reject("geometry is not a polygon");
            return Ok(());
        };
        let solid = polygon.extrude(height);
        let geometry = Geometry {
            value: GeometryValue::FlowGeometry3D(FlowGeometry3D::Solid(solid)),
            ..geom_inner
        };
        let mut feature = feature.clone();
        feature.geometry = Arc::new(geometry);
        fw.send(ctx.new_with_feature_and_port(feature, FEATURES_PORT.clone()));
        Ok(())
    }

    #[cfg(not(feature = "new-geometry"))]
    fn finish(
        &mut self,
        _ctx: NodeContext,
        _fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "Extruder"
    }
}
