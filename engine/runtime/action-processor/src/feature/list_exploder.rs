use std::collections::HashMap;

use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, Feature};
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
        "Explodes array attributes into separate features, creating one feature per array element"
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

/// # ListExploder Parameters
///
/// Configuration for exploding array attributes into individual features.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct ListExploder {
    /// Attribute containing the array to explode (each element becomes a separate feature)
    source_attribute: Attribute,
    /// Aatch all attribute by name
    match_all: Option<bool>,
    /// Show element index from where the attribute is exploded.
    show_attribute_index: Option<bool>,
}

impl Processor for ListExploder {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let orignal_feature = &ctx.feature;

        if self.match_all.unwrap_or(false) {
            let source_attribute_name = self.source_attribute.to_string();
            let (
                attributes_key_contain_source_attribute,
                attributes_key_not_contain_source_attribute,
            ): (Vec<String>, Vec<String>) = orignal_feature
                .all_attribute_keys()
                .into_iter()
                .partition(|name| name.contains(&source_attribute_name));

            // begin list exploder
            let keys = orignal_feature.all_attribute_keys();

            for each_attribute_key_contains_source_attribute in
                attributes_key_contain_source_attribute
            {
                let mut new_feature = Feature::new();

                // 1. perseve all other attributes
                for each_attribute_key_need_perserved in
                    &attributes_key_not_contain_source_attribute
                {
                    let key = each_attribute_key_need_perserved.clone();
                    let value = orignal_feature.get(&key).unwrap().clone();

                    new_feature.insert(key, value);
                }

                // 2. new attribute to show where is index of the exploded attribute comes from in orginal attribute
                if self.show_attribute_index.unwrap_or(true) {
                    let index = keys
                        .iter()
                        .position(|k| *k == each_attribute_key_contains_source_attribute)
                        .unwrap();
                    new_feature
                        .insert("_element_index", AttributeValue::String(format!("{index}")));
                }

                // 3. insert one matched attribute
                let value = orignal_feature
                    .get(&each_attribute_key_contains_source_attribute)
                    .unwrap()
                    .clone();

                // 4. instead of matched attribute, use source_attribute
                new_feature.insert(self.source_attribute.clone(), value.clone());

                fw.send(ctx.new_with_feature_and_port(new_feature, DEFAULT_PORT.clone()));
            }
        } else {
            let Some(AttributeValue::Array(value)) =
                orignal_feature.attributes.get(&self.source_attribute)
            else {
                fw.send(
                    ctx.new_with_feature_and_port(orignal_feature.clone(), DEFAULT_PORT.clone()),
                );
                return Ok(());
            };
            if value.is_empty() {
                fw.send(
                    ctx.new_with_feature_and_port(orignal_feature.clone(), DEFAULT_PORT.clone()),
                );
                return Ok(());
            }
            for v in value {
                let AttributeValue::Map(attributes) = v else {
                    fw.send(
                        ctx.new_with_feature_and_port(
                            orignal_feature.clone(),
                            DEFAULT_PORT.clone(),
                        ),
                    );
                    return Ok(());
                };
                let mut feature = orignal_feature.clone();
                feature.refresh_id();
                feature.remove(&self.source_attribute);
                feature.extend_attributes(attributes.clone());
                fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
            }
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
