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
pub(super) struct ListIndexerFactory;

impl ProcessorFactory for ListIndexerFactory {
    fn name(&self) -> &str {
        "ListIndexer"
    }

    fn description(&self) -> &str {
        "Copies attributes from a specific list element to become the main attributes of a feature"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(ListIndexer))
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
        let process: ListIndexer = if let Some(with) = with {
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

/// # ListIndexer Parameters
///
/// Configuration for copying attributes from a specific list element to main feature attributes.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct ListIndexer {
    /// List attribute to read from
    list_attribute: Attribute,
    /// Index of the list element to copy (0-based)
    list_index_to_copy: usize,
    /// Optional prefix to add to copied attribute names
    #[serde(default)]
    copied_attribute_prefix: Option<String>,
    /// Optional suffix to add to copied attribute names
    #[serde(default)]
    copied_attribute_suffix: Option<String>,
}

impl Processor for ListIndexer {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let mut feature = ctx.feature.clone();

        // Get the list attribute and extract element attributes if valid
        let element_attributes = {
            let Some(AttributeValue::Array(list)) = feature.attributes.get(&self.list_attribute)
            else {
                // If list attribute doesn't exist or isn't an array, pass through unchanged
                fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
                return Ok(());
            };

            // Check if the specified index exists
            if self.list_index_to_copy >= list.len() {
                // If index is out of bounds, pass through unchanged
                fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
                return Ok(());
            }

            // Get the element at the specified index
            let element = &list[self.list_index_to_copy];

            // Only process if the element is a Map (object with attributes)
            if let AttributeValue::Map(element_attributes) = element {
                Some(element_attributes.clone())
            } else {
                None
            }
        };

        // If we have valid element attributes, copy them to the feature
        if let Some(element_attributes) = element_attributes {
            for (key, value) in element_attributes {
                let mut new_key = key;

                // Apply prefix if specified
                if let Some(ref prefix) = self.copied_attribute_prefix {
                    if !prefix.is_empty() {
                        new_key = format!("{prefix}{new_key}");
                    }
                }

                // Apply suffix if specified
                if let Some(ref suffix) = self.copied_attribute_suffix {
                    if !suffix.is_empty() {
                        new_key = format!("{new_key}{suffix}");
                    }
                }

                // Add the attribute to the feature
                feature.attributes.insert(Attribute::new(new_key), value);
            }
        }

        fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
        Ok(())
    }

    fn finish(&self, _ctx: NodeContext, _fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "ListIndexer"
    }
}
