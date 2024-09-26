use std::{cmp::Ordering, collections::HashMap};

use reearth_flow_runtime::{
    channels::ProcessorChannelForwarder,
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{Attribute, Feature};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::FeatureProcessorError;

#[derive(Debug, Clone, Default)]
pub struct FeatureSorterFactory;

impl ProcessorFactory for FeatureSorterFactory {
    fn name(&self) -> &str {
        "FeatureSorter"
    }

    fn description(&self) -> &str {
        "Sorts features by attributes"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(FeatureSorterParam))
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
        let params: FeatureSorterParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                FeatureProcessorError::SorterFactory(format!(
                    "Failed to serialize `with` parameter: {}",
                    e
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                FeatureProcessorError::SorterFactory(format!(
                    "Failed to deserialize `with` parameter: {}",
                    e
                ))
            })?
        } else {
            return Err(FeatureProcessorError::SorterFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        let process = FeatureSorter {
            params,
            buffer: vec![],
        };
        Ok(Box::new(process))
    }
}

#[derive(Debug, Clone)]
pub struct FeatureSorter {
    params: FeatureSorterParam,
    buffer: Vec<Feature>,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct FeatureSorterParam {
    sort_by: Vec<SortBy>,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct SortBy {
    attribute: Attribute,
    order: Order,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, JsonSchema)]
enum Order {
    #[serde(rename = "ascending")]
    Asc,
    #[serde(rename = "descending")]
    Desc,
}

impl Processor for FeatureSorter {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        _fw: &mut dyn ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = ctx.feature;
        self.buffer.push(feature);
        Ok(())
    }

    fn finish(
        &self,
        ctx: NodeContext,
        fw: &mut dyn ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let mut features = self.buffer.clone();
        features.sort_by(|a, b| {
            let cmp = self
                .params
                .sort_by
                .iter()
                .map(|sort_by| {
                    let attribute = &sort_by.attribute;
                    let order = &sort_by.order;
                    let a = a.attributes.get(attribute);
                    let b = b.attributes.get(attribute);
                    match (a, b) {
                        (Some(a), Some(b)) => {
                            if *order == Order::Asc {
                                a.partial_cmp(b)
                            } else {
                                b.partial_cmp(a)
                            }
                        }
                        _ => None,
                    }
                })
                .collect::<Vec<_>>();
            cmp.iter().fold(Ordering::Equal, |acc, item| match acc {
                Ordering::Equal if item.is_some() => item.unwrap(),
                _ => acc,
            })
        });
        for feature in features {
            fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                &ctx,
                feature,
                DEFAULT_PORT.clone(),
            ));
        }
        Ok(())
    }

    fn name(&self) -> &str {
        "FeatureSorter"
    }
}
