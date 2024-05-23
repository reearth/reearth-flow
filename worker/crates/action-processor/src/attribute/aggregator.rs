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

use super::errors::AttributeProcessorError;

#[derive(Debug, Clone, Default)]
pub struct AttributeAggregatorFactory;

#[async_trait::async_trait]
impl ProcessorFactory for AttributeAggregatorFactory {
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
        let params: AttributeAggregatorParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                AttributeProcessorError::AggregatorFactory(format!(
                    "Failed to serialize with: {}",
                    e
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                AttributeProcessorError::AggregatorFactory(format!(
                    "Failed to deserialize with: {}",
                    e
                ))
            })?
        } else {
            return Err(AttributeProcessorError::AggregatorFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };

        let process = AttributeAggregator {
            params,
            buffer: Vec::new(),
        };
        Ok(Box::new(process))
    }
}

#[derive(Debug, Clone)]
pub struct AttributeAggregator {
    params: AttributeAggregatorParam,
    buffer: Vec<Feature>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AttributeAggregatorParam {
    aggregations: Vec<Aggregation>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct Aggregation {
    attribute: Attribute,
    method: Method,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(super) enum Method {
    #[serde(rename = "max")]
    Max,
    #[serde(rename = "min")]
    Min,
    #[serde(rename = "sum")]
    Sum,
    #[serde(rename = "avg")]
    Avg,
}

impl Processor for AttributeAggregator {
    fn initialize(&mut self, _ctx: NodeContext) {}

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
        let mut feature = Feature::new();
        for aggregation in &self.params.aggregations {
            match aggregation.method {
                Method::Max => {
                    let result = self
                        .buffer
                        .iter()
                        .filter_map(|row| row.attributes.get(&aggregation.attribute))
                        .max_by(|&a, &b| a.partial_cmp(b).unwrap());
                    feature.insert(
                        format!("max_{}", &aggregation.attribute),
                        result
                            .map(|v| v.to_owned())
                            .unwrap_or(AttributeValue::Number(
                                serde_json::Number::from_f64(0.0).unwrap(),
                            )),
                    );
                }
                Method::Min => {
                    let result = self
                        .buffer
                        .iter()
                        .filter_map(|row| row.get(&aggregation.attribute))
                        .min_by(|a, b| a.partial_cmp(b).unwrap());
                    feature.insert(
                        format!("min_{}", &aggregation.attribute),
                        result
                            .map(|v| v.to_owned())
                            .unwrap_or(AttributeValue::Number(
                                serde_json::Number::from_f64(0.0).unwrap(),
                            )),
                    );
                }
                Method::Sum => {
                    let result = self
                        .buffer
                        .iter()
                        .filter_map(|row| {
                            row.get(&aggregation.attribute).and_then(|v| {
                                if let AttributeValue::Number(v) = v {
                                    v.as_f64()
                                } else {
                                    None
                                }
                            })
                        })
                        .collect::<Vec<f64>>();
                    feature.insert(
                        format!("sum_{}", &aggregation.attribute),
                        AttributeValue::Number(
                            serde_json::Number::from_f64(result.iter().sum::<f64>()).unwrap(),
                        ),
                    );
                }
                Method::Avg => {
                    let result = self
                        .buffer
                        .iter()
                        .filter_map(|row| {
                            row.get(&aggregation.attribute).and_then(|v| {
                                if let AttributeValue::Number(v) = v {
                                    v.as_f64()
                                } else {
                                    None
                                }
                            })
                        })
                        .collect::<Vec<f64>>();
                    if result.is_empty() {
                        feature.insert(
                            format!("avg_{}", &aggregation.attribute),
                            AttributeValue::Number(serde_json::Number::from_f64(0.0).unwrap()),
                        );
                        continue;
                    }
                    let result = result.iter().sum::<f64>() / result.len() as f64;
                    feature.insert(
                        format!("avg_{}", &aggregation.attribute),
                        AttributeValue::Number(serde_json::Number::from_f64(result).unwrap()),
                    );
                }
            }
        }
        fw.send(ExecutorContext::new_with_node_context_feature_and_port(
            &ctx,
            feature.clone(),
            DEFAULT_PORT.clone(),
        ));
        Ok(())
    }

    fn name(&self) -> &str {
        "AttributeAggregator"
    }
}
