use std::collections::HashMap;

use reearth_flow_runtime::{
    channels::ProcessorChannelForwarder,
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::Attribute;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::AttributeProcessorError;

#[derive(Debug, Clone, Default)]
pub struct AttributeKeeperFactory;

impl ProcessorFactory for AttributeKeeperFactory {
    fn name(&self) -> &str {
        "AttributeKeeper"
    }

    fn description(&self) -> &str {
        "Keeps only specified attributes"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(AttributeKeeper))
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
        let processor: AttributeKeeper = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                AttributeProcessorError::Keeper(format!(
                    "Failed to serialize `with` parameter: {}",
                    e
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                AttributeProcessorError::Keeper(format!(
                    "Failed to deserialize `with` parameter: {}",
                    e
                ))
            })?
        } else {
            return Err(AttributeProcessorError::Keeper(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        Ok(Box::new(processor))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AttributeKeeper {
    keep_attributes: Vec<Attribute>,
}

impl Processor for AttributeKeeper {
    fn initialize(&mut self, _ctx: NodeContext) {}
    fn num_threads(&self) -> usize {
        5
    }
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &mut dyn ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let processed_data = feature
            .iter()
            .filter(|(key, _)| self.keep_attributes.contains(key))
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect::<HashMap<_, _>>();
        fw.send(ctx.new_with_feature_and_port(
            feature.with_attributes(processed_data),
            DEFAULT_PORT.clone(),
        ));
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
        "AttributeKeeper"
    }
}

#[cfg(test)]
mod tests {
    use reearth_flow_types::{AttributeValue, Feature};

    use crate::tests::utils::{create_default_execute_context, MockProcessorChannelForwarder};

    use super::*;

    #[test]
    fn test_attribute_keeper_process() {
        let mut processor = AttributeKeeper {
            keep_attributes: vec![Attribute::new("name"), Attribute::new("age")],
        };

        let mut attributes = HashMap::new();
        attributes.insert(
            Attribute::new("name"),
            AttributeValue::String("John".to_string()),
        );
        attributes.insert(Attribute::new("age"), AttributeValue::Number(25.into()));
        attributes.insert(
            Attribute::new("gender"),
            AttributeValue::String("Male".to_string()),
        );

        let feature = Feature::new_with_attributes(attributes);

        let ctx = create_default_execute_context(&feature);
        let mut fw = MockProcessorChannelForwarder::default();

        processor.process(ctx, &mut fw).unwrap();

        let processed_attributes = fw.send_feature.attributes;

        assert_eq!(processed_attributes.len(), 2);
        assert!(processed_attributes.contains_key(&Attribute::new("name")),);
        assert!(processed_attributes.contains_key(&Attribute::new("age")),);
        assert!(!processed_attributes.contains_key(&Attribute::new("gender")),);
    }
}
