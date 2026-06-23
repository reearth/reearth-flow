use std::{collections::HashMap, sync::Arc};

use once_cell::sync::Lazy;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{
    Attribute, AttributeValue, Attributes, Code, CodeType, CompiledCode, Feature,
};
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

    fn tags(&self) -> &[&'static str] {
        &["statistics", "aggregate"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone(), COMPLETE_PORT.clone()]
    }

    fn build(
        &self,
        _ctx: NodeContext,
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
        let mut calculations = Vec::<CompiledCalculation>::new();
        for calculation in &params.calculations {
            let compiled = calculation.expr.compile().map_err(|e| {
                AttributeProcessorError::StatisticsCalculatorFactory(format!("{e:?}"))
            })?;
            calculations.push(CompiledCalculation {
                expr: compiled,
                new_attribute: calculation.new_attribute.clone(),
            });
        }

        let process = StatisticsCalculator {
            group_id: params.group_id,
            group_by: params.group_by,
            calculations,
            aggregate_buffer: HashMap::new(),
        };
        Ok(Box::new(process))
    }

    fn infer_output_schema(
        &self,
        inputs: &HashMap<Port, reearth_flow_types::attr_schema::AttrSchema>,
        with: &Option<HashMap<String, Value>>,
    ) -> Option<HashMap<Port, reearth_flow_types::attr_schema::AttrSchema>> {
        use reearth_flow_types::attr_schema::{AttrField, AttrSchema, AttrType};

        let params = parse_params(with)?;

        // `default` port: a fresh, CLOSED schema with only the produced keys,
        // mirroring the `finish` insertion order: group_by, group_id, calculations.
        let mut default_schema = AttrSchema::empty();
        if let Some(group_by) = params.group_by.as_ref() {
            for attr in group_by {
                default_schema.insert(attr.clone(), AttrField::always(AttrType::String));
            }
        }
        if let Some(group_id) = params.group_id.as_ref() {
            default_schema.insert(group_id.clone(), AttrField::always(AttrType::String));
        }
        for calculation in &params.calculations {
            default_schema.insert(
                calculation.new_attribute.clone(),
                AttrField::always(AttrType::Number),
            );
        }

        // `complete` port: identity passthrough of the input feature.
        let complete_schema = inputs
            .get(&DEFAULT_PORT.clone())
            .cloned()
            .unwrap_or_else(AttrSchema::open);

        Some(HashMap::from([
            (DEFAULT_PORT.clone(), default_schema),
            (COMPLETE_PORT.clone(), complete_schema),
        ]))
    }
}

/// Deserialize the `StatisticsCalculatorParam` from the node's `with` params,
/// mirroring the deserialization done in `build`. Returns `None` when `with`
/// is absent or the params don't deserialize (inference not possible).
fn parse_params(with: &Option<HashMap<String, Value>>) -> Option<StatisticsCalculatorParam> {
    let with = with.as_ref()?;
    let value = serde_json::to_value(with).ok()?;
    serde_json::from_value::<StatisticsCalculatorParam>(value).ok()
}

#[derive(Debug, Clone)]
struct StatisticsCalculator {
    group_id: Option<Attribute>,
    group_by: Option<Vec<Attribute>>,
    calculations: Vec<CompiledCalculation>,
    aggregate_buffer: HashMap<Attribute, HashMap<String, NumericValue>>,
}

#[derive(Debug, Clone)]
struct CompiledCalculation {
    new_attribute: Attribute,
    expr: CompiledCode,
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
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct Calculation {
    /// # New attribute name
    new_attribute: Attribute,
    /// # Calculation to perform
    expr: Code<{ CodeType::FlowExpr as u32 }>,
}

impl Processor for StatisticsCalculator {
    fn is_accumulating(&self) -> bool {
        false
    }

    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let env_vars = ctx.env_vars.clone();
        let feature = &ctx.feature;
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

            let attr_val = calculation
                .expr
                .eval(feature, Arc::clone(&env_vars))
                .map_err(|e| {
                    AttributeProcessorError::StatisticsCalculator(format!(
                        "Failed to evaluate expression for attribute '{}': {e}",
                        calculation.new_attribute
                    ))
                })?;

            let numeric_value = match attr_val {
                AttributeValue::Number(n) => {
                    if let Some(i) = n.as_i64() {
                        NumericValue::Integer(i)
                    } else if let Some(f) = n.as_f64() {
                        NumericValue::Float(f)
                    } else {
                        return Err(Box::new(AttributeProcessorError::StatisticsCalculator(
                            format!("unrepresentable number for '{}'", calculation.new_attribute),
                        )));
                    }
                }
                _ => {
                    return Err(Box::new(AttributeProcessorError::StatisticsCalculator(
                        format!(
                            "expression for '{}' did not return a number",
                            calculation.new_attribute
                        ),
                    )))
                }
            };
            *content = content.add(numeric_value);
        }
        fw.send(ctx.new_with_feature_and_port(feature.clone(), COMPLETE_PORT.clone()));
        Ok(())
    }

    fn finish(
        &mut self,
        ctx: NodeContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let mut features = HashMap::<String, HashMap<Attribute, NumericValue>>::new();
        for (new_attribute, value) in &self.aggregate_buffer {
            for (aggregate_key, count) in value {
                let current = features
                    .entry(aggregate_key.to_string())
                    .or_default()
                    .entry(new_attribute.clone())
                    .or_default();
                *current = current.add(*count);
            }
        }
        for (aggregate_key, value) in features {
            let mut feature = Feature::new_with_attributes(Attributes::new());

            if let Some(group_by_attrs) = self.group_by.as_ref() {
                let group_values: Vec<&str> = aggregate_key.split('|').collect();
                for (attr, attr_value) in group_by_attrs.iter().zip(group_values.iter()) {
                    feature.insert(attr, AttributeValue::String(attr_value.to_string()));
                }
            }

            if let Some(group_id) = self.group_id.as_ref() {
                feature.insert(group_id, AttributeValue::String(aggregate_key.clone()));
            }

            for (new_attribute, count) in &value {
                feature.insert(new_attribute.clone(), count.to_attribute_value());
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

#[cfg(test)]
mod tests {
    use super::*;
    use reearth_flow_types::attr_schema::{AttrField, AttrSchema, AttrType, Presence};
    use reearth_flow_types::Attribute;
    use serde_json::json;

    fn with_from(value: Value) -> Option<HashMap<String, Value>> {
        Some(serde_json::from_value(value).unwrap())
    }

    fn attr(name: &str) -> Attribute {
        Attribute::new(name.to_string())
    }

    #[test]
    fn infer_default_port_is_closed_typed_schema() {
        let with = with_from(json!({
            "groupBy": ["region"],
            "groupId": "gid",
            "calculations": [{ "newAttribute": "total", "expr": {"type": "flowExpr", "value": "1.0"} }]
        }));

        let mut input = AttrSchema::empty();
        input.insert(attr("junk"), AttrField::always(AttrType::String));
        let mut inputs = HashMap::new();
        inputs.insert(DEFAULT_PORT.clone(), input);

        let out = StatisticsCalculatorFactory
            .infer_output_schema(&inputs, &with)
            .expect("inference should succeed");
        let schema = out
            .get(&DEFAULT_PORT.clone())
            .expect("default port present");

        assert!(!schema.open, "default schema must be closed");
        assert_eq!(schema.fields.len(), 3, "exactly 3 produced attrs");
        assert_eq!(
            schema.fields.get(&attr("region")),
            Some(&AttrField {
                ty: AttrType::String,
                presence: Presence::Always
            })
        );
        assert_eq!(
            schema.fields.get(&attr("gid")),
            Some(&AttrField {
                ty: AttrType::String,
                presence: Presence::Always
            })
        );
        assert_eq!(
            schema.fields.get(&attr("total")),
            Some(&AttrField {
                ty: AttrType::Number,
                presence: Presence::Always
            })
        );
        assert!(
            !schema.fields.contains_key(&attr("junk")),
            "input attrs must be dropped"
        );
    }

    #[test]
    fn infer_complete_port_is_identity() {
        let with = with_from(json!({
            "groupBy": ["region"],
            "groupId": "gid",
            "calculations": [{ "newAttribute": "total", "expr": {"type": "flowExpr", "value": "1.0"} }]
        }));

        let mut input = AttrSchema::empty();
        input.insert(attr("a"), AttrField::always(AttrType::String));
        input.insert(attr("b"), AttrField::always(AttrType::Number));
        let mut inputs = HashMap::new();
        inputs.insert(DEFAULT_PORT.clone(), input.clone());

        let out = StatisticsCalculatorFactory
            .infer_output_schema(&inputs, &with)
            .expect("inference should succeed");
        let complete = out
            .get(&COMPLETE_PORT.clone())
            .expect("complete port present");

        assert_eq!(
            complete, &input,
            "complete port must be identity passthrough"
        );
    }

    #[test]
    fn infer_no_group_by_only_calculations() {
        let with = with_from(json!({
            "calculations": [{ "newAttribute": "cnt", "expr": {"type": "flowExpr", "value": "1"} }]
        }));

        let inputs = HashMap::new();

        let out = StatisticsCalculatorFactory
            .infer_output_schema(&inputs, &with)
            .expect("inference should succeed");
        let schema = out
            .get(&DEFAULT_PORT.clone())
            .expect("default port present");

        assert!(!schema.open);
        assert_eq!(schema.fields.len(), 1);
        assert_eq!(
            schema.fields.get(&attr("cnt")),
            Some(&AttrField {
                ty: AttrType::Number,
                presence: Presence::Always
            })
        );
    }
}
