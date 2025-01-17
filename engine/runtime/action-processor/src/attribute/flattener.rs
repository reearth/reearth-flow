use std::collections::HashMap;

use reearth_flow_runtime::{
    channels::ProcessorChannelForwarder,
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::AttributeProcessorError;

#[derive(Debug, Clone, Default)]
pub struct AttributeFlattenerFactory;

impl ProcessorFactory for AttributeFlattenerFactory {
    fn name(&self) -> &str {
        "AttributeFlattener"
    }

    fn description(&self) -> &str {
        "Flattens features by attributes"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(AttributeFlattenerParam))
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
        let params: AttributeFlattenerParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                AttributeProcessorError::FlattenerFactory(format!(
                    "Failed to serialize `with` parameter: {}",
                    e
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                AttributeProcessorError::FlattenerFactory(format!(
                    "Failed to deserialize `with` parameter: {}",
                    e
                ))
            })?
        } else {
            return Err(AttributeProcessorError::FlattenerFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };

        let process = AttributeFlattener { params };
        Ok(Box::new(process))
    }
}

#[derive(Debug, Clone)]
pub struct AttributeFlattener {
    params: AttributeFlattenerParam,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AttributeFlattenerParam {
    attributes: Vec<Attribute>,
}

impl Processor for AttributeFlattener {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &mut dyn ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let mut feature = ctx.feature.clone();
        for attribute in &self.params.attributes {
            if feature.attributes.contains_key(attribute) {
                if let Some(AttributeValue::Map(value)) = feature.attributes.get(attribute) {
                    let new_attributes = value
                        .iter()
                        .map(|(k, v)| (Attribute::new(k.clone()), v.clone()))
                        .collect::<HashMap<_, _>>();
                    feature.attributes.extend(new_attributes);
                    feature.attributes.remove(attribute);
                } else {
                    continue;
                }
            }
        }
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
        "AttributeFlattener"
    }
}

#[cfg(test)]
// Gnerate test code
mod test {
    use crate::tests::utils::{create_default_execute_context, MockProcessorChannelForwarder};
    use reearth_flow_types::Feature;

    use super::*;
    #[test]
    fn test_attribute_flattener() {
        let mut fw = MockProcessorChannelForwarder::default();
        let flattener: HashMap<String, AttributeValue> = vec![(
            "hoge".to_string(),
            AttributeValue::String("hogehoge".to_string()),
        )]
        .into_iter()
        .collect();
        let attributes: HashMap<Attribute, AttributeValue> =
            vec![(Attribute::new("test"), AttributeValue::Map(flattener))]
                .into_iter()
                .collect();
        let feature: Feature = Feature::from(attributes);
        let ctx = create_default_execute_context(&feature);
        let params = AttributeFlattenerParam {
            attributes: vec![Attribute::new("test".to_string())],
        };
        let mut processor = AttributeFlattener { params };
        processor.process(ctx, &mut fw).unwrap();
        assert_eq!(fw.send_ports.len(), 1);
        assert_eq!(fw.send_ports[0], DEFAULT_PORT.clone());
        assert_eq!(fw.send_features.len(), 1);
        let feature = fw.send_features[0].clone();
        assert_eq!(feature.attributes.len(), 1);
        assert!(feature
            .attributes
            .contains_key(&Attribute::new("hoge".to_string())),);
    }
}
