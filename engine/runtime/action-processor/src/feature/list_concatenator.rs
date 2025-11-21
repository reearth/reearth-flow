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
pub(super) struct ListConcatenatorFactory;

impl ProcessorFactory for ListConcatenatorFactory {
    fn name(&self) -> &str {
        "ListConcatenator"
    }

    fn description(&self) -> &str {
        "Extracts a specific attribute from each element in a list and concatenates them into a single string"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(ListConcatenator))
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
        let process: ListConcatenator = if let Some(with) = with {
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

/// # ListConcatenator Parameters
///
/// Configuration for concatenating a specific attribute from list elements.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct ListConcatenator {
    /// List attribute to read from
    list: Attribute,
    /// Attribute name to extract from each list element
    attribute: Attribute,
    /// Character(s) to use as separator between concatenated values
    separate_character: String,
    /// Name of the attribute to store the concatenated result
    output_attribute_name: Attribute,
}

impl Processor for ListConcatenator {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let mut feature = ctx.feature.clone();

        // Get the list attribute
        let Some(AttributeValue::Array(list)) = feature.attributes.get(&self.list) else {
            // If list attribute doesn't exist or isn't an array, pass through unchanged
            fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
            return Ok(());
        };

        // Collect values from each list element
        let mut values = Vec::new();
        let attribute_key = self.attribute.inner();
        for element in list {
            // Each element should be a Map containing attributes
            if let AttributeValue::Map(element_attributes) = element {
                // Try to get the specified attribute from this element
                if let Some(value) = element_attributes.get(&attribute_key) {
                    // Convert the value to string
                    values.push(value.to_string());
                }
            }
        }

        // Concatenate all values with the separator
        let concatenated = values.join(&self.separate_character);

        // Add the result as a new attribute
        feature.attributes.insert(
            self.output_attribute_name.clone(),
            AttributeValue::String(concatenated),
        );

        fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
        Ok(())
    }

    fn finish(&self, _ctx: NodeContext, _fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "ListConcatenator"
    }
}
