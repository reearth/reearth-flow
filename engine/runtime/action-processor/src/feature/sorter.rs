use std::{cmp::Ordering, collections::HashMap, sync::Arc};

use reearth_flow_runtime::{
    channels::ProcessorChannelForwarder,
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, Expr, Feature};
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

    fn parameter_schema(&self) -> Option<schemars::Schema> {
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
        ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let params: FeatureSorterParam = if let Some(with) = with.clone() {
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

        let expr_engine = Arc::clone(&ctx.expr_engine);
        let mut sort_by = Vec::new();
        for sort in &params.sort_by {
            let attribute_value = if let Some(attribute_value) = &sort.attribute_value {
                Some(
                    expr_engine
                        .compile(attribute_value.as_ref())
                        .map_err(|e| FeatureProcessorError::FilterFactory(format!("{:?}", e)))?,
                )
            } else {
                if sort.attribute.is_none() {
                    return Err(FeatureProcessorError::FilterFactory(
                        "Either `attribute` or `attributeValue` is required".to_string(),
                    )
                    .into());
                }
                None
            };
            sort_by.push(CompiledSortBy {
                attribute: sort.attribute.clone(),
                attribute_value,
                order: sort.order.clone(),
            });
        }

        let process = FeatureSorter {
            global_params: with,
            params: FeatureSorterCompiledParam { sort_by },
            buffer: vec![],
        };
        Ok(Box::new(process))
    }
}

#[derive(Debug, Clone)]
pub struct FeatureSorter {
    global_params: Option<HashMap<String, serde_json::Value>>,
    params: FeatureSorterCompiledParam,
    buffer: Vec<Feature>,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct FeatureSorterParam {
    sort_by: Vec<SortBy>,
}

#[derive(Debug, Clone)]
pub struct FeatureSorterCompiledParam {
    sort_by: Vec<CompiledSortBy>,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct SortBy {
    attribute: Option<Attribute>,
    attribute_value: Option<Expr>,
    order: Order,
}

#[derive(Debug, Clone)]
struct CompiledSortBy {
    attribute: Option<Attribute>,
    attribute_value: Option<rhai::AST>,
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
                    let (a, b) = if let Some(attribute) = &sort_by.attribute {
                        (
                            a.attributes.get(attribute).cloned(),
                            b.attributes.get(attribute).cloned(),
                        )
                    } else if let Some(attribute_value) = &sort_by.attribute_value {
                        let a_scope = a.new_scope(ctx.expr_engine.clone(), &self.global_params);
                        let b_scope = b.new_scope(ctx.expr_engine.clone(), &self.global_params);
                        (
                            a_scope
                                .eval_ast::<String>(attribute_value)
                                .map(AttributeValue::String)
                                .ok(),
                            b_scope
                                .eval_ast::<String>(attribute_value)
                                .map(AttributeValue::String)
                                .ok(),
                        )
                    } else {
                        (None, None)
                    };
                    let order = &sort_by.order;
                    match (a, b) {
                        (Some(a), Some(b)) => {
                            if *order == Order::Asc {
                                a.partial_cmp(&b)
                            } else {
                                b.partial_cmp(&a)
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
