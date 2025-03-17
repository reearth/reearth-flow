use std::collections::HashMap;

use reearth_flow_common::compress::decode;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, Geometry};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::GeometryProcessorError;

#[derive(Debug, Clone, Default)]
pub struct GeometryReplacerFactory;

impl ProcessorFactory for GeometryReplacerFactory {
    fn name(&self) -> &str {
        "GeometryReplacer"
    }

    fn description(&self) -> &str {
        "Replaces the geometry of a feature with a new geometry."
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(GeometryReplacer))
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
        let processor: GeometryReplacer = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                GeometryProcessorError::GeometryReplacerFactory(format!(
                    "Failed to serialize `with` parameter: {}",
                    e
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                GeometryProcessorError::GeometryReplacerFactory(format!(
                    "Failed to deserialize `with` parameter: {}",
                    e
                ))
            })?
        } else {
            return Err(GeometryProcessorError::GeometryReplacerFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        Ok(Box::new(processor))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct GeometryReplacer {
    source_attribute: Attribute,
}

impl Processor for GeometryReplacer {
    fn num_threads(&self) -> usize {
        2
    }

    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let mut feature = feature.clone();
        let Some(source) = feature.attributes.get(&self.source_attribute) else {
            fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
            return Ok(());
        };
        let AttributeValue::String(dump) = source else {
            fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
            return Ok(());
        };
        let dump = decode(dump)?;
        let geometry: Geometry = serde_json::from_str(&dump)?;
        feature.geometry = geometry;
        feature.remove(&self.source_attribute);
        fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
        Ok(())
    }

    fn finish(&self, _ctx: NodeContext, _fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "GeometryReplacer"
    }
}
