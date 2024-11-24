use std::{collections::HashMap, sync::Arc};

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

use super::errors::AttributeProcessorError;

#[derive(Debug, Clone, Default)]
pub struct AttributeAggregatorFactory;

impl ProcessorFactory for AttributeAggregatorFactory {
    fn name(&self) -> &str {
        "AttributeAggregator"
    }

    fn description(&self) -> &str {
        "Aggregates features by attributes"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(AttributeAggregatorParam))
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
        ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let params: AttributeAggregatorParam = if let Some(with) = with.clone() {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                AttributeProcessorError::AggregatorFactory(format!(
                    "Failed to serialize `with` parameter: {}",
                    e
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                AttributeProcessorError::AggregatorFactory(format!(
                    "Failed to deserialize `with` parameter: {}",
                    e
                ))
            })?
        } else {
            return Err(AttributeProcessorError::AggregatorFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };

        let expr_engine = Arc::clone(&ctx.expr_engine);
        let mut aggregate_attributes = Vec::<CompliledAggregateAttribute>::new();
        for aggregte_attribute in &params.aggregate_attributes {
            if let Some(expr) = &aggregte_attribute.attribute_value {
                let template_ast = expr_engine
                    .compile(expr.as_ref())
                    .map_err(|e| AttributeProcessorError::AggregatorFactory(format!("{:?}", e)))?;
                aggregate_attributes.push(CompliledAggregateAttribute {
                    attribute_value: Some(template_ast),
                    new_attribute: aggregte_attribute.new_attribute.clone(),
                    attribute: None,
                });
            } else {
                aggregate_attributes.push(CompliledAggregateAttribute {
                    attribute_value: None,
                    new_attribute: aggregte_attribute.new_attribute.clone(),
                    attribute: aggregte_attribute.attribute.clone(),
                });
            }
        }

        let calculation = if let Some(expr) = params.calculation {
            let ast = expr_engine.compile(expr.as_ref()).map_err(|e| {
                AttributeProcessorError::AggregatorFactory(format!(
                    "Failed to compile calculation: {}",
                    e
                ))
            })?;
            Some(ast)
        } else {
            None
        };

        let process = AttributeAggregator {
            global_params: with,
            aggregate_attributes,
            calculation,
            calculation_value: params.calculation_value,
            calculation_attribute: params.calculation_attribute,
            method: params.method,
            buffer: HashMap::new(),
        };
        Ok(Box::new(process))
    }
}

#[derive(Debug, Clone)]
pub struct AttributeAggregator {
    global_params: Option<HashMap<String, serde_json::Value>>,
    aggregate_attributes: Vec<CompliledAggregateAttribute>,
    calculation: Option<rhai::AST>,
    calculation_value: Option<i64>,
    calculation_attribute: Attribute,
    method: Method,
    buffer: HashMap<String, i64>, // string is tab
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AttributeAggregatorParam {
    aggregate_attributes: Vec<AggregateAttribute>,
    calculation: Option<Expr>,
    calculation_value: Option<i64>,
    calculation_attribute: Attribute,
    method: Method,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct AggregateAttribute {
    new_attribute: Attribute,
    attribute: Option<String>,
    attribute_value: Option<Expr>,
}

#[derive(Debug, Clone)]
struct CompliledAggregateAttribute {
    new_attribute: Attribute,
    attribute: Option<String>,
    attribute_value: Option<rhai::AST>,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
pub(super) enum Method {
    #[serde(rename = "max")]
    Max,
    #[serde(rename = "min")]
    Min,
    #[serde(rename = "count")]
    Count,
}

impl Processor for AttributeAggregator {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        _fw: &mut dyn ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = ctx.feature;
        let expr_engine = Arc::clone(&ctx.expr_engine);
        let scope = feature.new_scope(expr_engine.clone(), &self.global_params);

        let mut aggregates = Vec::new();
        for aggregate_attribute in &self.aggregate_attributes {
            if let Some(attribute) = &aggregate_attribute.attribute {
                let result = feature.get(attribute).ok_or_else(|| {
                    AttributeProcessorError::Aggregator(format!(
                        "Attribute not found: {}",
                        attribute
                    ))
                })?;
                aggregates.push(result.to_string());
                continue;
            }
            if let Some(ast) = &aggregate_attribute.attribute_value {
                let result = scope.eval_ast::<String>(ast).map_err(|e| {
                    AttributeProcessorError::Aggregator(format!(
                        "Failed to evaluate aggregation: {}",
                        e
                    ))
                })?;
                aggregates.push(result);
            }
        }
        let calc = if let Some(value) = self.calculation_value {
            value
        } else if let Some(calculation) = &self.calculation {
            scope.eval_ast::<i64>(calculation).map_err(|e| {
                AttributeProcessorError::Aggregator(format!(
                    "Failed to evaluate calculation: {}",
                    e
                ))
            })?
        } else {
            return Err(
                AttributeProcessorError::Aggregator("Calculation not found".to_string()).into(),
            );
        };
        let key = generate_aggregate_key(&aggregates);
        match &self.method {
            Method::Max => {
                let value = self.buffer.entry(key).or_insert(0);
                *value = std::cmp::max(*value, calc);
            }
            Method::Min => {
                let value = self.buffer.entry(key).or_insert(i64::MAX);
                *value = std::cmp::min(*value, calc);
            }
            Method::Count => {
                let value = self.buffer.entry(key).or_insert(0);
                *value += calc;
            }
        }
        Ok(())
    }

    fn finish(
        &self,
        ctx: NodeContext,
        fw: &mut dyn ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        for (key, value) in &self.buffer {
            let mut feature = Feature::new();
            for (i, aggregate_attribute) in self.aggregate_attributes.iter().enumerate() {
                feature.attributes.insert(
                    aggregate_attribute.new_attribute.clone(),
                    AttributeValue::String(key.split('\t').nth(i).unwrap_or_default().to_string()),
                );
            }
            feature.attributes.insert(
                self.calculation_attribute.clone(),
                AttributeValue::Number(serde_json::Number::from(*value)),
            );
            fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                &ctx,
                feature,
                DEFAULT_PORT.clone(),
            ));
        }
        Ok(())
    }

    fn name(&self) -> &str {
        "AttributeAggregator"
    }
}

fn generate_aggregate_key(values: &[String]) -> String {
    values.join("\t")
}
