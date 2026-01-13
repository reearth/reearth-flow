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

#[derive(Debug, Clone, Copy)]
enum NumericValue {
    Integer(i64),
    Float(f64),
}

impl Default for NumericValue {
    fn default() -> Self {
        NumericValue::Integer(0)
    }
}

impl NumericValue {
    fn add(self, other: NumericValue) -> NumericValue {
        match (self, other) {
            (NumericValue::Integer(a), NumericValue::Integer(b)) => NumericValue::Integer(a + b),
            (NumericValue::Float(a), NumericValue::Float(b)) => NumericValue::Float(a + b),
            (NumericValue::Integer(a), NumericValue::Float(b)) => NumericValue::Float(a as f64 + b),
            (NumericValue::Float(a), NumericValue::Integer(b)) => NumericValue::Float(a + b as f64),
        }
    }

    fn to_attribute_value(self) -> AttributeValue {
        match self {
            NumericValue::Integer(i) => AttributeValue::Number(serde_json::Number::from(i)),
            NumericValue::Float(f) => {
                if f.fract() == 0.0 {
                    // If it's a whole number, try to convert to integer
                    if f >= i64::MIN as f64 && f <= i64::MAX as f64 && f == f as i64 as f64 {
                        AttributeValue::Number(serde_json::Number::from(f as i64))
                    } else {
                        AttributeValue::Number(
                            serde_json::Number::from_f64(f)
                                .unwrap_or_else(|| serde_json::Number::from(0)),
                        )
                    }
                } else {
                    AttributeValue::Number(
                        serde_json::Number::from_f64(f)
                            .unwrap_or_else(|| serde_json::Number::from(0)),
                    )
                }
            }
        }
    }
}

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
            group_id: params.group_id,
            group_by: params.group_by,
            calculations,
            aggregate_buffer: HashMap::new(),
            global_params: with,
            accumulated_features: Vec::new(),
            accumulation_mode: params.accumulation_mode.unwrap_or(false),
        };
        Ok(Box::new(process))
    }
}

#[derive(Debug, Clone)]
struct StatisticsCalculator {
    group_id: Option<Attribute>,
    group_by: Option<Vec<Attribute>>,
    calculations: Vec<CompiledCalculation>,
    aggregate_buffer: HashMap<Attribute, HashMap<String, NumericValue>>,
    global_params: Option<HashMap<String, serde_json::Value>>,
    accumulated_features: Vec<Feature>,
    accumulation_mode: bool,
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
    /// # Group id
    /// Optional attribute to store the group identifier. The ID will be formed by concatenating the values of the group_by attributes separated by '|'.
    group_id: Option<Attribute>,
    /// # Group by
    /// Attributes to group features by for aggregation. All of the inputs will be grouped if not specified.
    group_by: Option<Vec<Attribute>>,
    /// # Calculations
    /// List of statistical calculations to perform on grouped features
    calculations: Vec<Calculation>,

    /// Accumulate all incoming feature and merge static computed with them
    accumulation_mode: Option<bool>,
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

            // Try to evaluate as f64 first, then fall back to i64
            let eval_result = scope.eval_ast::<f64>(&calculation.expr);
            match eval_result {
                Ok(eval) => {
                    let numeric_value = NumericValue::Float(eval);
                    *content = match *content {
                        NumericValue::Integer(i) => numeric_value.add(NumericValue::Integer(i)),
                        NumericValue::Float(f) => numeric_value.add(NumericValue::Float(f)),
                    };
                }
                Err(_) => {
                    // If f64 evaluation fails, try i64
                    let eval_result = scope.eval_ast::<i64>(&calculation.expr);
                    match eval_result {
                        Ok(eval) => {
                            let numeric_value = NumericValue::Integer(eval);
                            *content = match *content {
                                NumericValue::Integer(i) => {
                                    numeric_value.add(NumericValue::Integer(i))
                                }
                                NumericValue::Float(f) => numeric_value.add(NumericValue::Float(f)),
                            };
                        }
                        Err(e) => {
                            return Err(Box::new(
                                AttributeProcessorError::StatisticsCalculatorFactory(format!(
                                    "Failed to evaluate expression for attribute '{}', error: {:?}",
                                    calculation.new_attribute, e
                                )),
                            ));
                        }
                    }
                }
            }
        }

        // Store the feature if accumulation_mode is enabled
        if self.accumulation_mode {
            self.accumulated_features.push(feature.clone());
        } else {
            // If not in accumulation mode, send the original feature through COMPLETE_PORT
            fw.send(ctx.new_with_feature_and_port(feature.clone(), COMPLETE_PORT.clone()));
        }

        Ok(())
    }

    fn finish(&self, ctx: NodeContext, fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        // Compute the final aggregated statistics
        let mut aggregated_results = HashMap::<String, HashMap<Attribute, NumericValue>>::new();
        for (new_attribute, value) in &self.aggregate_buffer {
            for (aggregate_key, count) in value {
                let current = aggregated_results
                    .entry(aggregate_key.to_string())
                    .or_default()
                    .entry(new_attribute.clone())
                    .or_default();
                *current = current.add(*count);
            }
        }

        if self.accumulation_mode {
            // In accumulation mode, send each stored feature with the statistical summaries attached
            for feature in &self.accumulated_features {
                // Create a new feature based on the original but with statistical summaries
                let mut feature_with_stats = feature.clone();

                // Calculate the correct aggregate key for this feature (same as in process method)
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

                // Add group_by attributes to the output feature if they exist
                if let Some(group_by_attrs) = self.group_by.as_ref() {
                    let group_values: Vec<&str> = aggregate_key.split('|').collect();
                    for (attr, attr_value) in group_by_attrs.iter().zip(group_values.iter()) {
                        feature_with_stats.insert(attr.clone(), AttributeValue::String(attr_value.to_string()));
                    }
                }

                // Add the calculated statistics for this group
                if let Some(stats) = aggregated_results.get(&aggregate_key) {
                    for (new_attribute, count) in stats {
                        feature_with_stats
                            .insert(new_attribute.clone(), count.to_attribute_value());
                    }
                }

                fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                    &ctx,
                    feature_with_stats,
                    DEFAULT_PORT.clone(),
                ));
            }
        } else {
            // In non-accumulation mode, send the aggregated results as separate features
            for (aggregate_key, value) in aggregated_results {
                let mut feature = Feature::new();

                // Add group_by attributes to the output feature
                if let Some(group_by_attrs) = self.group_by.as_ref() {
                    let group_values: Vec<&str> = aggregate_key.split('|').collect();
                    for (attr, attr_value) in group_by_attrs.iter().zip(group_values.iter()) {
                        feature
                            .insert(attr.clone(), AttributeValue::String(attr_value.to_string()));
                    }
                }

                // Add group_id if specified
                if let Some(group_id) = self.group_id.as_ref() {
                    feature.insert(
                        group_id.clone(),
                        AttributeValue::String(aggregate_key.clone()),
                    );
                }

                // Add calculated statistics
                for (new_attribute, count) in &value {
                    feature.insert(new_attribute.clone(), count.to_attribute_value());
                }
                fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                    &ctx,
                    feature,
                    DEFAULT_PORT.clone(),
                ));
            }
        }
        Ok(())
    }

    fn name(&self) -> &str {
        "StatisticsCalculator"
    }
}
