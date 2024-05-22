use std::collections::HashMap;

use reearth_flow_runtime::{
    channels::ProcessorChannelForwarder,
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, Feature};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::errors::ProcessorError;

#[derive(Debug, Clone, Default)]
pub struct AttributeDuplicateFilterFactory;

#[async_trait::async_trait]
impl ProcessorFactory for AttributeDuplicateFilterFactory {
    fn get_input_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    async fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let params: AttributeDuplicateFilterParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                ProcessorError::AttributeDuplicateFilterFactory(format!(
                    "Failed to serialize with: {}",
                    e
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                ProcessorError::AttributeDuplicateFilterFactory(format!(
                    "Failed to deserialize with: {}",
                    e
                ))
            })?
        } else {
            return Err(ProcessorError::AttributeDuplicateFilterFactory(
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
pub struct AttributeDuplicateFilter {
    params: AttributeDuplicateFilterParam,
    buffer: HashMap<AttributeValue, Feature>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AttributeDuplicateFilterParam {
    filter_by: Vec<Attribute>,
}

impl Processor for AttributeDuplicateFilter {
    fn initialize(&mut self, _ctx: NodeContext) {}

    fn process(
        &mut self,
        ctx: ExecutorContext,
        _fw: &mut dyn ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let key_values = self
            .params
            .filter_by
            .iter()
            .flat_map(|attribute| feature.get(attribute))
            .collect::<Vec<_>>();
        let key_values = key_values.iter().map(|&v| v.clone()).collect::<Vec<_>>();
        self.buffer
            .insert(AttributeValue::Array(key_values), feature.clone());
        Ok(())
    }

    fn finish(
        &self,
        ctx: NodeContext,
        fw: &mut dyn ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
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
