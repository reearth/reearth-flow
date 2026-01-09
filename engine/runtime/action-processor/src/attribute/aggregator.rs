use std::{collections::HashMap, sync::Arc};

use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use reearth_flow_eval_expr::utils::dynamic_to_value;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{Context, ExecutorContext, NodeContext},
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
    fn max(self, other: NumericValue) -> NumericValue {
        match (self, other) {
            (NumericValue::Integer(a), NumericValue::Integer(b)) => NumericValue::Integer(a.max(b)),
            (NumericValue::Float(a), NumericValue::Float(b)) => NumericValue::Float(a.max(b)),
            (NumericValue::Integer(a), NumericValue::Float(b)) => {
                NumericValue::Float((a as f64).max(b))
            }
            (NumericValue::Float(a), NumericValue::Integer(b)) => {
                NumericValue::Float(a.max(b as f64))
            }
        }
    }

    fn min(self, other: NumericValue) -> NumericValue {
        match (self, other) {
            (NumericValue::Integer(a), NumericValue::Integer(b)) => NumericValue::Integer(a.min(b)),
            (NumericValue::Float(a), NumericValue::Float(b)) => NumericValue::Float(a.min(b)),
            (NumericValue::Integer(a), NumericValue::Float(b)) => {
                NumericValue::Float((a as f64).min(b))
            }
            (NumericValue::Float(a), NumericValue::Integer(b)) => {
                NumericValue::Float(a.min(b as f64))
            }
        }
    }

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

#[derive(Debug, Clone, Default)]
pub(super) struct AttributeAggregatorFactory;

impl ProcessorFactory for AttributeAggregatorFactory {
    fn name(&self) -> &str {
        "AttributeAggregator"
    }

    fn description(&self) -> &str {
        "Group and Aggregate Features by Attributes"
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
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                AttributeProcessorError::AggregatorFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
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
                    .map_err(|e| AttributeProcessorError::AggregatorFactory(format!("{e:?}")))?;
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

        // Handle both old single-calculation format and new multiple-calculations format
        let mut compiled_calculations = Vec::new();

        // If the new multiple calculations format is provided, use it
        if let Some(multiple_calculations) = &params.calculations {
            for calculation_param in multiple_calculations {
                let calculation_ast = if let Some(expr) = &calculation_param.calculation {
                    let ast = expr_engine.compile(expr.as_ref()).map_err(|e| {
                        AttributeProcessorError::AggregatorFactory(format!(
                            "Failed to compile calculation: {e}"
                        ))
                    })?;
                    Some(ast)
                } else {
                    None
                };

                compiled_calculations.push(CompiledCalculation {
                    calculation: calculation_ast,
                    calculation_value: calculation_param.calculation_value,
                    calculation_attribute: calculation_param.calculation_attribute.clone(),
                    method: calculation_param.method.clone(),
                });
            }
        } else {
            // Use the old single calculation format for backward compatibility
            let calculation_ast = if let Some(expr) = &params.calculation {
                let ast = expr_engine.compile(expr.as_ref()).map_err(|e| {
                    AttributeProcessorError::AggregatorFactory(format!(
                        "Failed to compile calculation: {e}"
                    ))
                })?;
                Some(ast)
            } else {
                None
            };

            // Check if required fields for single calculation exist
            let calculation_attribute = params.calculation_attribute.ok_or_else(|| {
                AttributeProcessorError::AggregatorFactory(
                    "Missing required field `calculationAttribute` for single calculation format"
                        .to_string(),
                )
            })?;

            let method = params.method.ok_or_else(|| {
                AttributeProcessorError::AggregatorFactory(
                    "Missing required field `method` for single calculation format".to_string(),
                )
            })?;

            compiled_calculations.push(CompiledCalculation {
                calculation: calculation_ast,
                calculation_value: params.calculation_value,
                calculation_attribute,
                method,
            });
        }

        let process = AttributeAggregator {
            global_params: with,
            aggregate_attributes,
            calculations: compiled_calculations,
            buffer: HashMap::new(),
        };
        Ok(Box::new(process))
    }
}

#[derive(Debug, Clone)]
struct AttributeAggregator {
    global_params: Option<HashMap<String, serde_json::Value>>,
    aggregate_attributes: Vec<CompliledAggregateAttribute>,
    calculations: Vec<CompiledCalculation>,
    buffer: HashMap<AttributeValue, Vec<AggregationValue>>, // string is tab
}

/// Represents the aggregation value for each group
#[derive(Debug, Clone)]
enum AggregationValue {
    Single(NumericValue),           // For max, min, count methods
    SumAndCount(NumericValue, u64), // For average method (sum as NumericValue, count as u64)
}

impl Default for AggregationValue {
    fn default() -> Self {
        AggregationValue::Single(NumericValue::default())
    }
}

/// # AttributeAggregator Parameters
/// Configure how features are grouped and aggregated based on attribute values
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct AttributeAggregatorParam {
    /// # List of attributes to aggregate (grouping attributes)
    aggregate_attributes: Vec<AggregateAttribute>,
    /// # Calculations to perform (for backward compatibility)
    calculation: Option<Expr>,
    /// # Value to use for calculation (for backward compatibility)
    calculation_value: Option<i64>,
    /// # Attribute to store calculation result (for backward compatibility)
    calculation_attribute: Option<Attribute>,
    /// # Method to use for aggregation (for backward compatibility)
    method: Option<Method>,
    /// # Multiple calculations to perform (new feature)
    calculations: Option<Vec<Calculation>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct AggregateAttribute {
    /// # New attribute to create
    new_attribute: Attribute,
    /// # Existing attribute to use
    attribute: Option<Attribute>,
    /// # Value to use for attribute
    attribute_value: Option<Expr>,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct Calculation {
    /// # Calculation to perform
    calculation: Option<Expr>,
    /// # Value to use for calculation
    calculation_value: Option<i64>,
    /// # Attribute to store calculation result
    calculation_attribute: Attribute,
    /// # Method to use for aggregation
    method: Method,
}

#[derive(Debug, Clone)]
struct CompliledAggregateAttribute {
    new_attribute: Attribute,
    attribute: Option<Attribute>,
    attribute_value: Option<rhai::AST>,
}

#[derive(Debug, Clone)]
struct CompiledCalculation {
    calculation: Option<rhai::AST>,
    calculation_value: Option<i64>,
    calculation_attribute: Attribute,
    method: Method,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
enum Method {
    /// # Maximum Value
    /// Find the maximum value in the group
    #[serde(rename = "max")]
    Max,
    /// # Minimum Value
    /// Find the minimum value in the group
    #[serde(rename = "min")]
    Min,
    /// # Count Items
    /// Count the number of features in the group
    #[serde(rename = "count")]
    Count,
    /// # Average Value
    /// Calculate the average value in the group
    #[serde(rename = "avg")]
    Average,
}

impl Processor for AttributeAggregator {
    fn num_threads(&self) -> usize {
        2
    }

    fn process(
        &mut self,
        ctx: ExecutorContext,
        _fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let expr_engine = Arc::clone(&ctx.expr_engine);
        let scope = feature.new_scope(expr_engine.clone(), &self.global_params);

        let mut aggregates = Vec::new();
        for aggregate_attribute in &self.aggregate_attributes {
            if let Some(attribute) = &aggregate_attribute.attribute {
                // Handle missing attributes gracefully by using a null value
                let result = feature.get(attribute).unwrap_or(&AttributeValue::Null);
                aggregates.push(result.clone());
                continue;
            }
            if let Some(ast) = &aggregate_attribute.attribute_value {
                let result = scope.eval_ast::<rhai::Dynamic>(ast).map_err(|e| {
                    AttributeProcessorError::Aggregator(format!(
                        "Failed to evaluate aggregation: {e}"
                    ))
                })?;
                aggregates.push(dynamic_to_value(&result).into());
            }
        }
        let key = AttributeValue::Array(aggregates);

        // Initialize the aggregation vector if this key doesn't exist
        if !self.buffer.contains_key(&key) {
            let mut agg_values = Vec::new();
            for calculation in &self.calculations {
                match &calculation.method {
                    Method::Max => {
                        agg_values.push(AggregationValue::Single(NumericValue::Integer(i64::MIN)))
                    }
                    Method::Min => {
                        agg_values.push(AggregationValue::Single(NumericValue::Integer(i64::MAX)))
                    }
                    Method::Count => {
                        agg_values.push(AggregationValue::Single(NumericValue::Integer(0)))
                    }
                    Method::Average => {
                        agg_values.push(AggregationValue::SumAndCount(NumericValue::Integer(0), 0))
                    }
                }
            }
            self.buffer.insert(key.clone(), agg_values);
        }

        let agg_values = self.buffer.get_mut(&key).unwrap();

        // Process each calculation
        for (i, calculation) in self.calculations.iter().enumerate() {
            let calc_value = if let Some(value) = calculation.calculation_value {
                NumericValue::Integer(value)
            } else if let Some(calculation_ast) = &calculation.calculation {
                // Try to evaluate as f64 first, then fall back to i64
                let eval_result = scope.eval_ast::<f64>(calculation_ast);
                let numeric_value = match eval_result {
                    Ok(eval) => NumericValue::Float(eval),
                    Err(_) => {
                        // If f64 evaluation fails, try i64
                        let eval_result = scope.eval_ast::<i64>(calculation_ast);
                        match eval_result {
                            Ok(eval) => NumericValue::Integer(eval),
                            Err(_) => {
                                // If both evaluations fail, treat as 0.0 to handle missing attributes gracefully
                                NumericValue::Float(0.0)
                            }
                        }
                    }
                };
                numeric_value
            } else {
                return Err(AttributeProcessorError::Aggregator(
                    "Calculation not found".to_string(),
                )
                .into());
            };

            match &calculation.method {
                Method::Max => {
                    if let AggregationValue::Single(ref mut value) = agg_values[i] {
                        *value = value.max(calc_value);
                    }
                }
                Method::Min => {
                    if let AggregationValue::Single(ref mut value) = agg_values[i] {
                        *value = value.min(calc_value);
                    }
                }
                Method::Count => {
                    if let AggregationValue::Single(ref mut value) = agg_values[i] {
                        *value = value.add(calc_value);
                    }
                }
                Method::Average => {
                    if let AggregationValue::SumAndCount(ref mut sum, ref mut count) = agg_values[i]
                    {
                        *sum = sum.add(calc_value);
                        *count += 1;
                    }
                }
            }
        }

        Ok(())
    }

    fn finish(&self, ctx: NodeContext, fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        self.flush_buffer(ctx.as_context(), fw);
        Ok(())
    }

    fn name(&self) -> &str {
        "AttributeAggregator"
    }
}

impl AttributeAggregator {
    pub(crate) fn flush_buffer(&self, ctx: Context, fw: &ProcessorChannelForwarder) {
        self.buffer.par_iter().for_each(|(key, values)| {
            let mut feature = Feature::new();
            let AttributeValue::Array(aggregates) = key else {
                return;
            };
            for (i, aggregate_attribute) in self.aggregate_attributes.iter().enumerate() {
                feature.attributes.insert(
                    aggregate_attribute.new_attribute.clone(),
                    aggregates.get(i).cloned().unwrap_or(AttributeValue::Null),
                );
            }

            // Add all calculated values to the feature
            for (i, calculation) in self.calculations.iter().enumerate() {
                let final_value = match &values[i] {
                    AggregationValue::Single(v) => v.to_attribute_value(),
                    AggregationValue::SumAndCount(sum, count) => {
                        if *count > 0 {
                            match sum {
                                NumericValue::Integer(int_sum) => {
                                    let avg = *int_sum as f64 / *count as f64;
                                    AttributeValue::Number(
                                        serde_json::Number::from_f64(avg)
                                            .unwrap_or_else(|| serde_json::Number::from(0)),
                                    )
                                }
                                NumericValue::Float(float_sum) => {
                                    let avg = *float_sum / *count as f64;
                                    AttributeValue::Number(
                                        serde_json::Number::from_f64(avg)
                                            .unwrap_or_else(|| serde_json::Number::from(0)),
                                    )
                                }
                            }
                        } else {
                            AttributeValue::Number(serde_json::Number::from(0))
                        }
                    }
                };

                feature
                    .attributes
                    .insert(calculation.calculation_attribute.clone(), final_value);
            }

            fw.send(ExecutorContext::new_with_context_feature_and_port(
                &ctx,
                feature,
                DEFAULT_PORT.clone(),
            ));
        });
    }
}
