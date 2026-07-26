use std::{collections::HashMap, sync::Arc};

use once_cell::sync::Lazy;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, FEATURES_PORT},
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

    fn as_f64(self) -> f64 {
        match self {
            NumericValue::Integer(i) => i as f64,
            NumericValue::Float(f) => f,
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

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, JsonSchema, Default)]
#[serde(rename_all = "camelCase")]
enum AggregationMethod {
    /// # Count
    /// Counts the number of features in each group. The value expression is not required and is ignored.
    Count,
    /// # Sum
    /// Adds the value expression across all features in each group.
    #[default]
    Sum,
    /// # Minimum
    /// Keeps the smallest value of the expression in each group.
    Min,
    /// # Maximum
    /// Keeps the largest value of the expression in each group.
    Max,
    /// # Mean
    /// Averages the value of the expression across all features in each group.
    Mean,
}

impl std::fmt::Display for AggregationMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            AggregationMethod::Count => "count",
            AggregationMethod::Sum => "sum",
            AggregationMethod::Min => "min",
            AggregationMethod::Max => "max",
            AggregationMethod::Mean => "mean",
        };
        f.write_str(s)
    }
}

impl AggregationMethod {
    /// Count is the only method that does not consume a value expression.
    fn requires_expr(self) -> bool {
        self != AggregationMethod::Count
    }
}

/// Per-(group, output attribute) running state. Every field is O(1) to
/// maintain, so no per-feature values are retained.
#[derive(Debug, Clone, Default)]
struct StatAccumulator {
    count: u64,
    sum: NumericValue,
    min: Option<NumericValue>,
    max: Option<NumericValue>,
}

impl StatAccumulator {
    /// Record a feature for a `Count` calculation (no value involved).
    fn ingest_count(&mut self) {
        self.count += 1;
    }

    /// Record an expression value for a numeric calculation.
    fn ingest_value(&mut self, value: NumericValue) {
        self.count += 1;
        self.sum = self.sum.add(value);
        self.min = Some(match self.min {
            Some(current) if current.as_f64() <= value.as_f64() => current,
            _ => value,
        });
        self.max = Some(match self.max {
            Some(current) if current.as_f64() >= value.as_f64() => current,
            _ => value,
        });
    }

    /// Produce the final attribute value for the given method.
    fn finalize(&self, method: AggregationMethod) -> AttributeValue {
        match method {
            AggregationMethod::Count => {
                AttributeValue::Number(serde_json::Number::from(self.count))
            }
            AggregationMethod::Sum => self.sum.to_attribute_value(),
            // A group only exists once a value has been ingested, so min/max
            // are always populated for numeric methods; fall back to 0 defensively.
            AggregationMethod::Min => self
                .min
                .map(NumericValue::to_attribute_value)
                .unwrap_or_else(|| AttributeValue::Number(serde_json::Number::from(0))),
            AggregationMethod::Max => self
                .max
                .map(NumericValue::to_attribute_value)
                .unwrap_or_else(|| AttributeValue::Number(serde_json::Number::from(0))),
            AggregationMethod::Mean => {
                if self.count == 0 {
                    AttributeValue::Number(serde_json::Number::from(0))
                } else {
                    NumericValue::Float(self.sum.as_f64() / self.count as f64).to_attribute_value()
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
        "Statistics Calculator"
    }

    fn description(&self) -> &str {
        "Groups features by one or more attributes and computes an aggregate statistic (count, sum, minimum, maximum, or mean) for each group, emitting one feature per group."
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(StatisticsCalculatorParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Attribute"]
    }

    fn tags(&self) -> &[&'static str] {
        &["aggregation", "statistics"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![FEATURES_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![FEATURES_PORT.clone(), COMPLETE_PORT.clone()]
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
            // Count ignores any value expression; every other method requires one.
            let expr = if calculation.aggregation.requires_expr() {
                match &calculation.expr {
                    Some(expr) => Some(expr.compile().map_err(|e| {
                        AttributeProcessorError::StatisticsCalculatorFactory(format!("{e:?}"))
                    })?),
                    None => {
                        return Err(
                            AttributeProcessorError::StatisticsCalculatorFactory(format!(
                                "Aggregation '{}' for attribute '{}' requires an expression",
                                calculation.aggregation, calculation.new_attribute
                            ))
                            .into(),
                        );
                    }
                }
            } else {
                None
            };
            calculations.push(CompiledCalculation {
                new_attribute: calculation.new_attribute.clone(),
                aggregation: calculation.aggregation,
                expr,
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
            .get(&FEATURES_PORT.clone())
            .cloned()
            .unwrap_or_else(AttrSchema::open);

        Some(HashMap::from([
            (FEATURES_PORT.clone(), default_schema),
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
    /// group key -> output attribute -> running accumulator.
    aggregate_buffer: HashMap<String, HashMap<Attribute, StatAccumulator>>,
}

#[derive(Debug, Clone)]
struct CompiledCalculation {
    new_attribute: Attribute,
    aggregation: AggregationMethod,
    expr: Option<CompiledCode>,
}

/// # Statistics Calculator Parameters
/// Defines the grouping attributes and the statistics computed for each group.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct StatisticsCalculatorParam {
    /// # Calculations
    /// Statistics to compute for each group. Each entry names an output attribute, the aggregation method, and (except for count) the expression whose values are aggregated.
    calculations: Vec<Calculation>,
    /// # Group By
    /// Attributes to group features by before aggregating. When omitted, all input features form a single group.
    group_by: Option<Vec<Attribute>>,
    /// # Group ID
    /// Optional attribute in which to store the group identifier, formed by joining the Group By values with '|'.
    group_id: Option<Attribute>,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct Calculation {
    /// # New Attribute Name
    /// Name of the output attribute that stores the computed statistic.
    new_attribute: Attribute,
    /// # Aggregation Method
    /// Statistic to compute across each group. Defaults to sum.
    #[serde(default)]
    aggregation: AggregationMethod,
    /// # Value Expression
    /// Expression evaluated per feature to produce the value being aggregated. Required for every method except count, which counts features regardless of value.
    expr: Option<Code<{ CodeType::FlowExpr as u32 }>>,
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
            let acc = self
                .aggregate_buffer
                .entry(aggregate_key.clone())
                .or_default()
                .entry(calculation.new_attribute.clone())
                .or_default();

            match &calculation.expr {
                // Count is the only method without a value expression.
                None => acc.ingest_count(),
                Some(expr) => {
                    let attr_val = expr.eval(feature, Arc::clone(&env_vars)).map_err(|e| {
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
                                return Err(Box::new(
                                    AttributeProcessorError::StatisticsCalculator(format!(
                                        "unrepresentable number for '{}'",
                                        calculation.new_attribute
                                    )),
                                ));
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
                    acc.ingest_value(numeric_value);
                }
            }
        }
        fw.send(ctx.new_with_feature_and_port(feature.clone(), COMPLETE_PORT.clone()));
        Ok(())
    }

    fn finish(
        &mut self,
        ctx: NodeContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        for (aggregate_key, accumulators) in &self.aggregate_buffer {
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

            // Emit one attribute per calculation, in declaration order.
            for calculation in &self.calculations {
                let value = accumulators
                    .get(&calculation.new_attribute)
                    .map(|acc| acc.finalize(calculation.aggregation))
                    .unwrap_or(AttributeValue::Null);
                feature.insert(calculation.new_attribute.clone(), value);
            }
            fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                &ctx,
                feature,
                FEATURES_PORT.clone(),
            ));
        }
        Ok(())
    }

    fn name(&self) -> &str {
        "Statistics Calculator"
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
        inputs.insert(FEATURES_PORT.clone(), input);

        let out = StatisticsCalculatorFactory
            .infer_output_schema(&inputs, &with)
            .expect("inference should succeed");
        let schema = out
            .get(&FEATURES_PORT.clone())
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
        inputs.insert(FEATURES_PORT.clone(), input.clone());

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
            .get(&FEATURES_PORT.clone())
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

    // --- aggregation-method behavior ---

    use crate::tests::utils::create_default_execute_context;
    use reearth_flow_runtime::forwarder::NoopChannelForwarder;

    fn num(n: i64) -> AttributeValue {
        AttributeValue::Number(serde_json::Number::from(n))
    }

    #[test]
    fn stat_accumulator_finalizes_each_method() {
        let mut acc = StatAccumulator::default();
        for v in [10i64, 20, 30] {
            acc.ingest_value(NumericValue::Integer(v));
        }
        assert_eq!(acc.finalize(AggregationMethod::Count), num(3));
        assert_eq!(acc.finalize(AggregationMethod::Sum), num(60));
        assert_eq!(acc.finalize(AggregationMethod::Min), num(10));
        assert_eq!(acc.finalize(AggregationMethod::Max), num(30));
        assert_eq!(acc.finalize(AggregationMethod::Mean), num(20));
    }

    #[test]
    fn stat_accumulator_mean_keeps_fraction() {
        let mut acc = StatAccumulator::default();
        acc.ingest_value(NumericValue::Integer(1));
        acc.ingest_value(NumericValue::Integer(2));
        assert_eq!(
            acc.finalize(AggregationMethod::Mean),
            AttributeValue::Number(serde_json::Number::from_f64(1.5).unwrap())
        );
    }

    #[test]
    fn stat_accumulator_min_max_across_mixed_types() {
        let mut acc = StatAccumulator::default();
        acc.ingest_value(NumericValue::Float(2.5));
        acc.ingest_value(NumericValue::Integer(1));
        acc.ingest_value(NumericValue::Integer(5));
        assert_eq!(acc.finalize(AggregationMethod::Min), num(1));
        assert_eq!(acc.finalize(AggregationMethod::Max), num(5));
    }

    fn build_processor(with: Value) -> Box<dyn Processor> {
        let map: HashMap<String, Value> = serde_json::from_value(with).unwrap();
        StatisticsCalculatorFactory
            .build(
                NodeContext::default(),
                EventHub::new(30),
                "Statistics Calculator".to_string(),
                Some(map),
            )
            .expect("build should succeed")
    }

    fn feature_with_v(v: i64) -> Feature {
        let mut attrs: indexmap::IndexMap<Attribute, AttributeValue> = indexmap::IndexMap::new();
        attrs.insert(attr("v"), num(v));
        Feature::from(attrs)
    }

    /// Run features through `process` then `finish`, returning the features
    /// emitted to the default (aggregate) port.
    fn run(processor: &mut Box<dyn Processor>, features: Vec<Feature>) -> Vec<Feature> {
        let noop = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop.clone());
        for f in &features {
            let ctx = create_default_execute_context(f);
            processor.process(ctx, &fw).unwrap();
        }
        processor.finish(NodeContext::default(), &fw).unwrap();
        let feats = noop.send_features.lock().unwrap();
        let ports = noop.send_ports.lock().unwrap();
        feats
            .iter()
            .zip(ports.iter())
            .filter(|(_, p)| **p == *FEATURES_PORT)
            .map(|(f, _)| f.clone())
            .collect()
    }

    #[test]
    fn all_methods_over_one_group() {
        let expr = json!({"type": "flowExpr", "value": "attributes.get(\"v\")"});
        let with = json!({
            "calculations": [
                { "newAttribute": "n",   "aggregation": "count" },
                { "newAttribute": "tot", "aggregation": "sum",  "expr": expr },
                { "newAttribute": "lo",  "aggregation": "min",  "expr": expr },
                { "newAttribute": "hi",  "aggregation": "max",  "expr": expr },
                { "newAttribute": "avg", "aggregation": "mean", "expr": expr }
            ]
        });
        let mut p = build_processor(with);
        let out = run(
            &mut p,
            vec![feature_with_v(10), feature_with_v(20), feature_with_v(30)],
        );
        assert_eq!(out.len(), 1);
        let f = &out[0];
        assert_eq!(f.get("n"), Some(&num(3)));
        assert_eq!(f.get("tot"), Some(&num(60)));
        assert_eq!(f.get("lo"), Some(&num(10)));
        assert_eq!(f.get("hi"), Some(&num(30)));
        assert_eq!(f.get("avg"), Some(&num(20)));
    }

    #[test]
    fn sum_is_the_default_method() {
        // No `aggregation` field -> behaves like the pre-change action (sum),
        // so existing workflows are unaffected.
        let with = json!({
            "calculations": [
                { "newAttribute": "tot", "expr": {"type": "flowExpr", "value": "attributes.get(\"v\")"} }
            ]
        });
        let mut p = build_processor(with);
        let out = run(&mut p, vec![feature_with_v(2), feature_with_v(3)]);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].get("tot"), Some(&num(5)));
    }

    #[test]
    fn non_count_method_requires_expression() {
        let with = json!({ "calculations": [ { "newAttribute": "avg", "aggregation": "mean" } ] });
        let map: HashMap<String, Value> = serde_json::from_value(with).unwrap();
        let result = StatisticsCalculatorFactory.build(
            NodeContext::default(),
            EventHub::new(30),
            "Statistics Calculator".to_string(),
            Some(map),
        );
        assert!(result.is_err());
        let msg = format!("{:?}", result.unwrap_err());
        assert!(msg.contains("requires an expression"), "got: {msg}");
    }
}
