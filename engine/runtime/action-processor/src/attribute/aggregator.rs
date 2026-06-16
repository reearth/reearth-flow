use std::collections::HashMap;

use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{Context, ExecutorContext, NodeContext},
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

    fn tags(&self) -> &[&'static str] {
        &["aggregate"]
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
        let params: AttributeAggregatorParam = if let Some(with) = with {
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

        let mut aggregate_attributes = Vec::<CompliledAggregateAttribute>::new();
        for aggregte_attribute in params.aggregate_attributes {
            if let Some(expr) = aggregte_attribute.attribute_value {
                let compiled = expr
                    .compile()
                    .map_err(|e| AttributeProcessorError::AggregatorFactory(format!("{e:?}")))?;
                aggregate_attributes.push(CompliledAggregateAttribute {
                    attribute_value: Some(compiled),
                    new_attribute: aggregte_attribute.new_attribute,
                    attribute: None,
                });
            } else {
                aggregate_attributes.push(CompliledAggregateAttribute {
                    attribute_value: None,
                    new_attribute: aggregte_attribute.new_attribute,
                    attribute: aggregte_attribute.attribute,
                });
            }
        }

        let calculation = params
            .calculation
            .map(|c| {
                c.compile().map_err(|e| {
                    AttributeProcessorError::AggregatorFactory(format!(
                        "Failed to compile calculation: {e}"
                    ))
                })
            })
            .transpose()?;

        let process = AttributeAggregator {
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
struct AttributeAggregator {
    aggregate_attributes: Vec<CompliledAggregateAttribute>,
    calculation: Option<CompiledCode>,
    calculation_value: Option<i64>,
    calculation_attribute: Attribute,
    method: Method,
    buffer: HashMap<AttributeValue, i64>, // string is tab
}

/// # AttributeAggregator Parameters
/// Configure how features are grouped and aggregated based on attribute values
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct AttributeAggregatorParam {
    /// # List of attributes to aggregate
    aggregate_attributes: Vec<AggregateAttribute>,
    /// # Calculation to perform
    calculation: Option<Code<{ CodeType::FlowExpr as u32 }>>,
    /// # Value to use for calculation
    calculation_value: Option<i64>,
    /// # Attribute to store calculation result
    calculation_attribute: Attribute,
    /// # Method to use for aggregation
    method: Method,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct AggregateAttribute {
    /// # New attribute to create
    new_attribute: Attribute,
    /// # Existing attribute to use
    attribute: Option<Attribute>,
    /// # Value to use for attribute
    attribute_value: Option<Code<{ CodeType::FlowExpr as u32 }>>,
}

#[derive(Debug, Clone)]
struct CompliledAggregateAttribute {
    new_attribute: Attribute,
    attribute: Option<Attribute>,
    attribute_value: Option<CompiledCode>,
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
}

impl Processor for AttributeAggregator {
    fn is_accumulating(&self) -> bool {
        true
    }

    fn num_threads(&self) -> usize {
        2
    }

    fn process(
        &mut self,
        ctx: ExecutorContext,
        _fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let env_vars = ctx.expr_engine.vars().clone();

        let mut aggregates = Vec::new();
        for aggregate_attribute in &self.aggregate_attributes {
            if let Some(attribute) = &aggregate_attribute.attribute {
                let result = feature.get(attribute).ok_or_else(|| {
                    AttributeProcessorError::Aggregator(format!("Attribute not found: {attribute}"))
                })?;
                aggregates.push(result.clone());
                continue;
            }
            if let Some(code) = &aggregate_attribute.attribute_value {
                let result = code.eval(feature, env_vars.clone()).map_err(|e| {
                    AttributeProcessorError::Aggregator(format!(
                        "Failed to evaluate aggregation: {e}"
                    ))
                })?;
                aggregates.push(result);
            }
        }
        let calc = if let Some(value) = self.calculation_value {
            value
        } else if let Some(calculation) = &self.calculation {
            calculation
                .eval(feature, env_vars)
                .map_err(|e| {
                    AttributeProcessorError::Aggregator(format!(
                        "Failed to evaluate calculation: {e}"
                    ))
                })?
                .as_i64()
                .ok_or_else(|| {
                    AttributeProcessorError::Aggregator(
                        "calculation must evaluate to an integer".to_string(),
                    )
                })?
        } else {
            return Err(
                AttributeProcessorError::Aggregator("Calculation not found".to_string()).into(),
            );
        };
        let key = AttributeValue::Array(aggregates);
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
        &mut self,
        ctx: NodeContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        self.flush_buffer(ctx.as_context(), fw);
        self.buffer.clear();
        Ok(())
    }

    fn name(&self) -> &str {
        "AttributeAggregator"
    }
}

impl AttributeAggregator {
    pub(crate) fn flush_buffer(&self, ctx: Context, fw: &ProcessorChannelForwarder) {
        self.buffer.par_iter().for_each(|(key, value)| {
            let mut feature = Feature::new_with_attributes(Attributes::new());
            let AttributeValue::Array(aggregates) = key else {
                return;
            };
            for (i, aggregate_attribute) in self.aggregate_attributes.iter().enumerate() {
                feature.insert(
                    &aggregate_attribute.new_attribute,
                    aggregates.get(i).cloned().unwrap_or(AttributeValue::Null),
                );
            }
            feature.insert(
                &self.calculation_attribute,
                AttributeValue::Number(serde_json::Number::from(*value)),
            );
            fw.send(ExecutorContext::new_with_context_feature_and_port(
                &ctx,
                feature,
                DEFAULT_PORT.clone(),
            ));
        });
    }
}

#[cfg(test)]
mod tests {
    use indexmap::IndexMap;
    use reearth_flow_runtime::forwarder::NoopChannelForwarder;
    use reearth_flow_types::Feature;

    use super::*;
    use crate::tests::utils::create_default_execute_context;

    fn make_processor() -> AttributeAggregator {
        AttributeAggregator {
            aggregate_attributes: vec![CompliledAggregateAttribute {
                new_attribute: Attribute::new("file"),
                attribute: Some(Attribute::new("file")),
                attribute_value: None,
            }],
            calculation: None,
            calculation_value: Some(1),
            calculation_attribute: Attribute::new("count"),
            method: Method::Count,
            buffer: HashMap::new(),
        }
    }

    fn make_feature(file: &str) -> Feature {
        let mut attrs: IndexMap<Attribute, AttributeValue> = IndexMap::new();
        attrs.insert(
            Attribute::new("file"),
            AttributeValue::String(file.to_string()),
        );
        Feature::from(attrs)
    }

    fn collect_counts(noop: &NoopChannelForwarder) -> Vec<(String, i64)> {
        let features = noop.send_features.lock().unwrap();
        let ports = noop.send_ports.lock().unwrap();
        assert!(
            ports.iter().all(|p| *p == *DEFAULT_PORT),
            "all emissions should go to DEFAULT port"
        );
        features
            .iter()
            .map(|f| {
                let file = match f.attributes.get(&Attribute::new("file")) {
                    Some(AttributeValue::String(s)) => s.clone(),
                    other => panic!("missing/invalid 'file' attr: {other:?}"),
                };
                let count = match f.attributes.get(&Attribute::new("count")) {
                    Some(AttributeValue::Number(n)) => n.as_i64().unwrap(),
                    other => panic!("missing/invalid 'count' attr: {other:?}"),
                };
                (file, count)
            })
            .collect()
    }

    fn make_node_context() -> NodeContext {
        NodeContext::default()
    }

    #[test]
    fn count_aggregates_correctly_with_interleaved_keys() {
        let fw = ProcessorChannelForwarder::Noop(NoopChannelForwarder::default());
        let mut processor = make_processor();

        // A, B, A, B — each key 2x, fully interleaved.
        for file in ["A", "B", "A", "B"] {
            let feature = make_feature(file);
            let ctx = create_default_execute_context(&feature);
            processor.process(ctx, &fw).unwrap();
        }
        processor.finish(make_node_context(), &fw).unwrap();

        let ProcessorChannelForwarder::Noop(noop) = fw else {
            unreachable!()
        };
        let mut counts = collect_counts(&noop);
        counts.sort();
        assert_eq!(
            counts,
            vec![("A".to_string(), 2), ("B".to_string(), 2)],
            "expected one totalled feature per key; before the fix this emitted multiple partial features (e.g. A=1, B=1, A=1, B=1)"
        );
    }
}
