use std::{collections::HashMap, str::FromStr};

use super::errors::PlateauProcessorError;
use reearth_flow_common::uri::Uri;
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

#[derive(Debug, Clone, Default)]
pub struct ObjectListExtractorFactory;

impl ProcessorFactory for ObjectListExtractorFactory {
    fn name(&self) -> &str {
        "PLATEAU4.ObjectListExtractor"
    }

    fn description(&self) -> &str {
        "Extract object list"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(ObjectListExtractorParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["PLATEAU"]
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
        let params: ObjectListExtractorParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                PlateauProcessorError::ObjectListExtractorFactory(format!(
                    "Failed to serialize `with` parameter: {}",
                    e
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                PlateauProcessorError::ObjectListExtractorFactory(format!(
                    "Failed to deserialize `with` parameter: {}",
                    e
                ))
            })?
        } else {
            return Err(PlateauProcessorError::ObjectListExtractorFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        let process = ObjectListExtractor {
            object_list_path_attribute: params.object_list_path_attribute,
        };
        Ok(Box::new(process))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ObjectListExtractorParam {
    object_list_path_attribute: Attribute,
}

#[derive(Debug, Clone)]
pub(crate) struct ObjectListExtractor {
    object_list_path_attribute: Attribute,
}

impl Processor for ObjectListExtractor {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let mut feature = ctx.feature.clone();
        let object_list_path = feature
            .attributes
            .get(&self.object_list_path_attribute)
            .and_then(|v| v.as_string())
            .ok_or(PlateauProcessorError::ObjectListExtractor(
                "objectListPath attribute empty".to_string(),
            ))?;
        let object_list_path = Uri::from_str(object_list_path.as_str())
            .map_err(|e| PlateauProcessorError::ObjectListExtractor(format!("{}", e)))?;
        let storage_resolver = ctx.storage_resolver.clone();
        let storage = storage_resolver.resolve(&object_list_path).map_err(|e| {
            PlateauProcessorError::ObjectListExtractor(format!(
                "Failed to resolve objectList path: {}",
                e
            ))
        })?;
        let bytes = storage
            .get_sync(object_list_path.path().as_path())
            .map_err(|e| {
                PlateauProcessorError::ObjectListExtractor(format!(
                    "Failed to get objectList file: {}",
                    e
                ))
            })?;
        let (feature_types, object_list) = crate::object_list::parse(bytes).map_err(|e| {
            PlateauProcessorError::ObjectListExtractor(format!(
                "Failed to parse objectList file: {}",
                e
            ))
        })?;
        feature.insert(
            "featureTypes",
            AttributeValue::Map(
                feature_types
                    .into_iter()
                    .map(|(prefix, feature_types)| {
                        (
                            prefix.clone(),
                            AttributeValue::Array(
                                feature_types
                                    .iter()
                                    .cloned()
                                    .map(AttributeValue::String)
                                    .collect(),
                            ),
                        )
                    })
                    .collect::<HashMap<String, AttributeValue>>(),
            ),
        );
        feature.insert(
            "objectList",
            AttributeValue::Map(
                object_list
                    .into_iter()
                    .map(|(prefix, object_list)| (prefix.clone(), object_list.into()))
                    .collect::<HashMap<String, AttributeValue>>(),
            ),
        );
        fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
        Ok(())
    }

    fn finish(&self, _ctx: NodeContext, _fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "ObjectListExtractor"
    }
}
