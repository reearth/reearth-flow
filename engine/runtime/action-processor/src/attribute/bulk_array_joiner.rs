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

use super::errors::AttributeProcessorError;

#[derive(Debug, Clone, Default)]
pub(super) struct AttributeBulkArrayJoinerFactory;

impl ProcessorFactory for AttributeBulkArrayJoinerFactory {
    fn name(&self) -> &str {
        "AttributeBulkArrayJoiner"
    }

    fn description(&self) -> &str {
        "Flattens features by attributes"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(AttributeBulkArrayJoinerParam))
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
        let params: AttributeBulkArrayJoinerParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                AttributeProcessorError::FlattenerFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                AttributeProcessorError::FlattenerFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(AttributeProcessorError::FlattenerFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };

        let process = AttributeBulkArrayJoiner {
            ignore_attributes: params.ignore_attributes.clone().unwrap_or_default(),
        };
        Ok(Box::new(process))
    }
}

#[derive(Debug, Clone)]
struct AttributeBulkArrayJoiner {
    ignore_attributes: Vec<Attribute>,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct AttributeBulkArrayJoinerParam {
    /// # Attributes to ignore
    ignore_attributes: Option<Vec<Attribute>>,
}

impl Processor for AttributeBulkArrayJoiner {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let mut feature = ctx.feature.clone();
        let mut new_attributes = HashMap::<Attribute, AttributeValue>::new();
        for (key, value) in feature
            .attributes
            .iter()
            .filter(|(key, _)| !self.ignore_attributes.contains(key))
            .filter(|(key, _)| matches!(feature.get(key), Some(AttributeValue::Array(_))))
        {
            let AttributeValue::Array(value) = value else {
                continue;
            };
            if value.len() == 1 {
                if let Some(AttributeValue::Map(v)) = value.first() {
                    new_attributes.insert(key.clone(), AttributeValue::Map(v.clone()));
                    continue;
                }
            }
            let mut new_value = Vec::<String>::new();
            for detail in value {
                match detail {
                    AttributeValue::String(v) => {
                        new_value.push(v.clone());
                    }
                    AttributeValue::Number(v) => {
                        new_value.push(v.to_string());
                    }
                    AttributeValue::Bool(v) => {
                        new_value.push(v.to_string());
                    }
                    _ => {}
                }
            }
            new_attributes.insert(key.clone(), AttributeValue::String(new_value.join(",")));
        }
        feature.attributes.extend(new_attributes);
        fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
        Ok(())
    }

    fn finish(&self, _ctx: NodeContext, _fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "AttributeBulkArrayJoiner"
    }
}

#[cfg(test)]
// Gnerate test code
mod test {
    use crate::tests::utils::create_default_execute_context;
    use indexmap::IndexMap;
    use reearth_flow_runtime::forwarder::NoopChannelForwarder;
    use reearth_flow_types::Feature;

    use super::*;
    #[test]
    fn test_attribute_map_array_joiner() {
        let noop = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop);
        let flattener: HashMap<String, AttributeValue> = vec![(
            "hoge".to_string(),
            AttributeValue::String("hogehoge".to_string()),
        )]
        .into_iter()
        .collect();
        let attributes: IndexMap<Attribute, AttributeValue> = vec![(
            Attribute::new("test"),
            AttributeValue::Array(vec![AttributeValue::Map(flattener)]),
        )]
        .into_iter()
        .collect();
        let feature: Feature = Feature::from(attributes);
        let ctx = create_default_execute_context(&feature);
        let mut processor = AttributeBulkArrayJoiner {
            ignore_attributes: vec![],
        };
        processor.process(ctx, &fw).unwrap();
        if let ProcessorChannelForwarder::Noop(noop) = fw {
            assert_eq!(noop.send_ports.lock().unwrap().len(), 1);
            assert_eq!(
                noop.send_ports.lock().unwrap().first().cloned(),
                Some(DEFAULT_PORT.clone())
            );
            assert_eq!(noop.send_features.lock().unwrap().len(), 1);
            let feature = noop.send_features.lock().unwrap().first().unwrap().clone();
            assert_eq!(feature.attributes.len(), 1);
            assert!(feature
                .attributes
                .contains_key(&Attribute::new("test".to_string())),);
        }
    }

    #[test]
    fn test_attribute_single_array_joiner() {
        let noop = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop);
        let attributes: IndexMap<Attribute, AttributeValue> = vec![(
            Attribute::new("test"),
            AttributeValue::Array(vec![AttributeValue::String("fugafuga".to_string())]),
        )]
        .into_iter()
        .collect();
        let feature: Feature = Feature::from(attributes);
        let ctx = create_default_execute_context(&feature);
        let mut processor = AttributeBulkArrayJoiner {
            ignore_attributes: vec![],
        };
        processor.process(ctx, &fw).unwrap();
        if let ProcessorChannelForwarder::Noop(noop) = fw {
            assert_eq!(noop.send_ports.lock().unwrap().len(), 1);
            assert_eq!(
                noop.send_ports.lock().unwrap().first().cloned(),
                Some(DEFAULT_PORT.clone())
            );
            assert_eq!(noop.send_features.lock().unwrap().len(), 1);
            let feature = noop.send_features.lock().unwrap().first().unwrap().clone();
            assert_eq!(feature.attributes.len(), 1);
            let Some(AttributeValue::String(v)) = feature.get(&"test".to_string()) else {
                panic!();
            };
            assert_eq!(v, "fugafuga");
        }
    }

    #[test]
    fn test_attribute_multi_array_joiner() {
        let noop = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop);
        let attributes: IndexMap<Attribute, AttributeValue> = vec![(
            Attribute::new("test"),
            AttributeValue::Array(vec![
                AttributeValue::String("hogehoge".to_string()),
                AttributeValue::String("fugafuga".to_string()),
            ]),
        )]
        .into_iter()
        .collect();
        let feature: Feature = Feature::from(attributes);
        let ctx = create_default_execute_context(&feature);
        let mut processor = AttributeBulkArrayJoiner {
            ignore_attributes: vec![],
        };
        processor.process(ctx, &fw).unwrap();
        if let ProcessorChannelForwarder::Noop(noop) = fw {
            assert_eq!(noop.send_ports.lock().unwrap().len(), 1);
            assert_eq!(
                noop.send_ports.lock().unwrap().first().unwrap().clone(),
                DEFAULT_PORT.clone()
            );
            assert_eq!(noop.send_features.lock().unwrap().len(), 1);
            let feature = noop.send_features.lock().unwrap().first().unwrap().clone();
            assert_eq!(feature.attributes.len(), 1);
            let Some(AttributeValue::String(v)) = feature.get(&"test".to_string()) else {
                panic!();
            };
            assert_eq!(v, "hogehoge,fugafuga");
        }
    }
}
