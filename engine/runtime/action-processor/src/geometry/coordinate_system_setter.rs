use std::collections::HashMap;

use nusamai_projection::crs::*;
use reearth_flow_runtime::{
    channels::ProcessorChannelForwarder,
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{errors::GeometryProcessorError, types::SUPPORT_EPSG_CODE};

#[derive(Debug, Clone, Default)]
pub struct CoordinateSystemSetterFactory;

impl ProcessorFactory for CoordinateSystemSetterFactory {
    fn name(&self) -> &str {
        "CoordinateSystemSetter"
    }

    fn description(&self) -> &str {
        "Sets the coordinate system of a feature"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(CoordinateSystemSetter))
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
        let processor: CoordinateSystemSetter = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                GeometryProcessorError::CoordinateSystemSetterFactory(format!(
                    "Failed to serialize `with` parameter: {}",
                    e
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                GeometryProcessorError::CoordinateSystemSetterFactory(format!(
                    "Failed to deserialize `with` parameter: {}",
                    e
                ))
            })?
        } else {
            return Err(GeometryProcessorError::CoordinateSystemSetterFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        if !SUPPORT_EPSG_CODE.contains(&processor.epsg_code) {
            return Err(GeometryProcessorError::CoordinateSystemSetterFactory(
                "Unsupported EPSG code".to_string(),
            )
            .into());
        }
        Ok(Box::new(processor))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CoordinateSystemSetter {
    epsg_code: EpsgCode,
}

impl Processor for CoordinateSystemSetter {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &mut dyn ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let mut feature = feature.clone();
        let mut geometry = feature.geometry;
        geometry.epsg = Some(self.epsg_code);
        feature.geometry = geometry;
        fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
        Ok(())
    }

    fn finish(
        &self,
        _ctx: NodeContext,
        _fw: &mut dyn ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "CoordinateSystemSetter"
    }
}
