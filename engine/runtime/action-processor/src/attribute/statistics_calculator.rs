use std::{collections::HashMap, sync::Arc};

use once_cell::sync::Lazy;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, Expr, Feature};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::AttributeProcessorError;

pub static COMPLETE_PORT: Lazy<Port> = Lazy::new(|| Port::new("complete"));

#[derive(Debug, Clone, Default)]
pub(super) struct StatisticsCalculatorFactory;

impl ProcessorFactory for StatisticsCalculatorFactory {
    fn name(&self) -> &str {
        "StatisticsCalculator"
    }

    fn description(&self) -> &str {
        "Calculates statistical aggregations on feature attributes with customizable expressions"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(StatisticsCalculatorParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Attribute"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone(), COMPLETE_PORT.clone()]
    }

    fn build(
        &self,
        ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let params: StatisticsCalculatorParam = if let Some(with) = with.clone() {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                AttributeProcessorError::StatisticsCalculatorFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                AttributeProcessorError::StatisticsCalculatorFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(AttributeProcessorError::StatisticsCalculatorFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        let expr_engine = Arc::clone(&ctx.expr_engine);
        let mut calculations = Vec::<CompiledCalculation>::new();
        for calculation in &params.calculations {
            let expr = &calculation.expr;
            let template_ast = expr_engine.compile(expr.as_ref()).map_err(|e| {
                AttributeProcessorError::StatisticsCalculatorFactory(format!("{e:?}"))
            })?;
            calculations.push(CompiledCalculation {
                expr: template_ast,
                new_attribute: calculation.new_attribute.clone(),
            });
        }

        let process = StatisticsCalculator {
            aggregate_name: params.aggregate_name,
            group_by: params.group_by,
            calculations,
            aggregate_buffer: HashMap::new(),
            global_params: with,
        };
        Ok(Box::new(process))
    }
}

#[derive(Debug, Clone)]
struct StatisticsCalculator {
    aggregate_name: Option<Attribute>,
    group_by: Option<Vec<Attribute>>,
    calculations: Vec<CompiledCalculation>,
    aggregate_buffer: HashMap<Attribute, HashMap<String, i64>>,
    global_params: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone)]
struct CompiledCalculation {
    new_attribute: Attribute,
    expr: rhai::AST,
}

/// # StatisticsCalculator Parameters
///
/// Configuration for calculating statistical aggregations on feature attributes.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct StatisticsCalculatorParam {
    /// Name of the attribute containing the aggregate group name
    aggregate_name: Option<Attribute>,
    /// Attributes to group features by for aggregation
    group_by: Option<Vec<Attribute>>,
    /// List of statistical calculations to perform on grouped features
    calculations: Vec<Calculation>,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct Calculation {
    /// # New attribute name
    new_attribute: Attribute,
    /// # Calculation to perform
    expr: Expr,
}

impl Processor for StatisticsCalculator {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let expr_engine = Arc::clone(&ctx.expr_engine);
        let feature = &ctx.feature;
        let scope = feature.new_scope(expr_engine.clone(), &self.global_params);
        let aggregate_key = self
            .group_by
            .as_ref()
            .unwrap_or(&Vec::new())
            .iter()
            .map(|attr| {
                let Some(value) = feature.attributes.get(attr) else {
                    return "".to_string();
                };
                value.to_string()
            })
            .collect::<Vec<_>>()
            .join("|");

        for calculation in &self.calculations {
            let aggregate_buffer = self
                .aggregate_buffer
                .entry(calculation.new_attribute.clone())
                .or_default();
            let content = aggregate_buffer.entry(aggregate_key.clone()).or_default();
            let eval = scope.eval_ast::<i64>(&calculation.expr);
            match eval {
                Ok(eval) => {
                    *content += eval;
                }
                _ => {
                    continue;
                }
            }
        }
        fw.send(ctx.new_with_feature_and_port(feature.clone(), COMPLETE_PORT.clone()));
        Ok(())
    }

    fn finish(&self, ctx: NodeContext, fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        let mut features = HashMap::<String, HashMap<Attribute, i64>>::new();
        for (new_attribute, value) in &self.aggregate_buffer {
            for (aggregate_key, count) in value {
                let current = features
                    .entry(aggregate_key.to_string())
                    .or_default()
                    .entry(new_attribute.clone())
                    .or_default();
                *current += count;
            }
        }
        for (aggregate_key, value) in features {
            let mut feature = Feature::new();

            // Add group_by attributes to the output feature
            if let Some(group_by_attrs) = self.group_by.as_ref() {
                let group_values: Vec<&str> = aggregate_key.split('|').collect();
                for (attr, attr_value) in group_by_attrs.iter().zip(group_values.iter()) {
                    feature.insert(attr, AttributeValue::String(attr_value.to_string()));
                }
            }

            // Add aggregate_name if specified
            if let Some(aggregate_name) = self.aggregate_name.as_ref() {
                feature.insert(
                    aggregate_name,
                    AttributeValue::String(aggregate_key.clone()),
                );
            }

            // Add calculated statistics
            for (new_attribute, count) in &value {
                feature.insert(
                    new_attribute.clone(),
                    AttributeValue::Number(serde_json::Number::from(*count)),
                );
            }
            fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                &ctx,
                feature,
                DEFAULT_PORT.clone(),
            ));
        }
        Ok(())
    }

    fn name(&self) -> &str {
        "StatisticsCalculator"
    }
}
