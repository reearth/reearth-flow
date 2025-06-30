use std::collections::HashMap;

use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::FeatureProcessorError;

#[derive(Debug, Clone, Default)]
pub(super) struct ListExploderFactory;

impl ProcessorFactory for ListExploderFactory {
    fn name(&self) -> &str {
        "ListExploder"
    }

    fn description(&self) -> &str {
        "Explodes list attributes"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(ListExploder))
    }

    fn categories(&self) -> &[&'static str] {
        &["Feature"]
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
        let process: ListExploder = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                FeatureProcessorError::TransformerFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                FeatureProcessorError::TransformerFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(FeatureProcessorError::TransformerFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        Ok(Box::new(process))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct ListExploder {
    /// The attribute to explode
    source_attribute: Attribute,
}

impl Processor for ListExploder {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let Some(AttributeValue::Array(value)) = feature.attributes.get(&self.source_attribute)
        else {
            fw.send(ctx.new_with_feature_and_port(feature.clone(), DEFAULT_PORT.clone()));
            return Ok(());
        };
        if value.is_empty() {
            fw.send(ctx.new_with_feature_and_port(feature.clone(), DEFAULT_PORT.clone()));
            return Ok(());
        }
        for v in value {
            let AttributeValue::Map(attributes) = v else {
                fw.send(ctx.new_with_feature_and_port(feature.clone(), DEFAULT_PORT.clone()));
                return Ok(());
            };
            let mut feature = feature.clone();
            feature.refresh_id();
            feature.remove(&self.source_attribute);
            feature.extend_attributes(attributes.clone());
            fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
        }
        Ok(())
    }

    fn finish(&self, _ctx: NodeContext, _fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "ListExploder"
    }
}
