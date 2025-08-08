use std::collections::HashMap;

use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{Attribute, Feature};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::AttributeProcessorError;

#[derive(Debug, Clone, Default)]
pub(super) struct AttributeDuplicateFilterFactory;

impl ProcessorFactory for AttributeDuplicateFilterFactory {
    fn name(&self) -> &str {
        "AttributeDuplicateFilter"
    }

    fn description(&self) -> &str {
        "Remove Duplicate Features Based on Attribute Values"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(AttributeDuplicateFilterParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Attribute"]
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
        let params: AttributeDuplicateFilterParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                AttributeProcessorError::DuplicateFilterFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                AttributeProcessorError::DuplicateFilterFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(AttributeProcessorError::DuplicateFilterFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };

        let process = AttributeDuplicateFilter {
            params,
            buffer: HashMap::new(),
        };
        Ok(Box::new(process))
    }
}

#[derive(Debug, Clone)]
struct AttributeDuplicateFilter {
    params: AttributeDuplicateFilterParam,
    buffer: HashMap<String, Feature>,
}

/// # AttributeDuplicateFilter Parameters
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct AttributeDuplicateFilterParam {
    /// # Filter Attributes
    /// Attributes used to identify duplicate features - features with identical values for these attributes will be deduplicated
    filter_by: Vec<Attribute>,
}

impl Processor for AttributeDuplicateFilter {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        _fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let key_values = self
            .params
            .filter_by
            .iter()
            .flat_map(|attribute| feature.get(attribute))
            .collect::<Vec<_>>();
        let key_values = key_values
            .iter()
            .map(|&v| v.clone().to_string())
            .collect::<Vec<_>>();
        self.buffer.insert(key_values.join(","), feature.clone());
        Ok(())
    }

    fn finish(&self, ctx: NodeContext, fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        for feature in self.buffer.values() {
            fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                &ctx,
                feature.clone(),
                DEFAULT_PORT.clone(),
            ));
        }
        Ok(())
    }

    fn name(&self) -> &str {
        "AttributeDuplicateFilter"
    }
}
