use std::collections::HashMap;

use indexmap::IndexMap;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use reearth_flow_diagnostics::{DiagnosticDraft, ErrorCode};
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{Context, ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, FEATURES_PORT},
};
#[cfg(feature = "new-geometry")]
use reearth_flow_geometry::{Geometry, GeometryCollection};
use reearth_flow_types::{
    Attribute, AttributeValue, Attributes, Code, CodeType, CompiledCode, Feature,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::AttributeProcessorError;
use crate::geometry::dissolver::AttributeAccumulationStrategy;

#[derive(Debug, Clone, Default)]
pub(super) struct AttributeAggregatorFactory;

impl ProcessorFactory for AttributeAggregatorFactory {
    fn name(&self) -> &str {
        "Attribute Aggregator"
    }

    fn description(&self) -> &str {
        "Groups features by attribute values and emits one feature per group, combining the group's geometries into a collection. Optionally aggregates a computed value within each group."
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(AttributeAggregatorParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Attribute"]
    }

    fn tags(&self) -> &[&'static str] {
        &["aggregation"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![FEATURES_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![FEATURES_PORT.clone()]
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
                    .map_err(|e| AttributeProcessorError::AggregatorFactory(format!("{e}")))?;
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

        let aggregation = match (params.calculation_attribute, params.method) {
            (Some(attribute), Some(method)) => {
                if params.calculation_value.is_none() && calculation.is_none() {
                    return Err(AttributeProcessorError::AggregatorFactory(
                        "`calculationAttribute` requires `calculationValue` or `calculation`"
                            .to_string(),
                    )
                    .into());
                }
                Some(Aggregation {
                    attribute,
                    method,
                    value: params.calculation_value,
                    expr: calculation,
                })
            }
            (None, None) => {
                if params.calculation_value.is_some() || calculation.is_some() {
                    return Err(AttributeProcessorError::AggregatorFactory(
                        "`calculationValue` and `calculation` require `calculationAttribute` and `method`"
                            .to_string(),
                    )
                    .into());
                }
                None
            }
            _ => {
                return Err(AttributeProcessorError::AggregatorFactory(
                    "`calculationAttribute` and `method` must be set together".to_string(),
                )
                .into());
            }
        };

        let process = AttributeAggregator {
            aggregate_attributes,
            aggregation,
            attribute_accumulation: params.attribute_accumulation,
            buffer: HashMap::new(),
        };
        Ok(Box::new(process))
    }
}

#[derive(Debug, Clone)]
struct AttributeAggregator {
    aggregate_attributes: Vec<CompliledAggregateAttribute>,
    aggregation: Option<Aggregation>,
    attribute_accumulation: AttributeAccumulationStrategy,
    buffer: HashMap<AttributeValue, Group>,
}

/// The optional per-group value: written to `attribute`, computed by `method` over a
/// per-feature value taken from `value` or, when unset, `expr`.
#[derive(Debug, Clone)]
struct Aggregation {
    attribute: Attribute,
    method: Method,
    value: Option<i64>,
    expr: Option<CompiledCode>,
}

/// Everything kept for one group until finish.
#[derive(Debug, Clone)]
struct Group {
    /// Each feature's geometry, in arrival order; a collection is flattened one
    /// level so its members keep their own attributes. Geometry is only combined
    /// under `new-geometry`.
    #[cfg(feature = "new-geometry")]
    members: Vec<Geometry>,
    #[cfg(feature = "new-geometry")]
    member_attrs: Vec<Attributes>,
    /// The incoming attributes kept under the accumulation strategy: none, the first
    /// feature's, or every distinct value per key in arrival order.
    kept: IndexMap<Attribute, Vec<AttributeValue>>,
    value: Option<i64>,
}

/// # Attribute Aggregator Parameters
/// Configures how features are grouped and, optionally, which value is aggregated within each group.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct AttributeAggregatorParam {
    /// # Group-By Attributes
    /// Attributes that define each group. Each entry reads a value from an existing attribute or an expression and writes it to a new attribute on the aggregated output feature.
    aggregate_attributes: Vec<AggregateAttribute>,
    /// # Result Attribute
    /// Attribute on the aggregated feature that stores the computed result. Leave unset, along with the aggregation method, to only combine features.
    calculation_attribute: Option<Attribute>,
    /// # Aggregation Method
    /// How to aggregate the per-feature calculation value within each group: maximum, minimum, or count. Required when a result attribute is set.
    method: Option<Method>,
    /// # Calculation Value
    /// Constant integer used as the per-feature value. Takes precedence over the calculation expression when set.
    calculation_value: Option<i64>,
    /// # Calculation Expression
    /// Expression evaluated to an integer per feature, used as the per-feature value when no calculation value is set.
    calculation: Option<Code<{ CodeType::FlowExpr as u32 }>>,
    /// # Attribute Accumulation
    /// Which incoming attributes the aggregated feature keeps besides the group-by attributes and the result. Defaults to dropping them.
    #[serde(default = "default_attribute_accumulation")]
    attribute_accumulation: AttributeAccumulationStrategy,
}

fn default_attribute_accumulation() -> AttributeAccumulationStrategy {
    AttributeAccumulationStrategy::DropAttributes
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct AggregateAttribute {
    /// # Output Attribute
    /// Name of the attribute written to the aggregated feature that holds this group value.
    new_attribute: Attribute,
    /// # Source Attribute
    /// Existing attribute to read the group value from. Ignored when a group value expression is provided.
    attribute: Option<Attribute>,
    /// # Group Value Expression
    /// Expression that computes the group value. Takes precedence over the source attribute when both are set.
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
    /// # Maximum
    /// Keeps the largest per-feature calculation value in the group.
    #[serde(rename = "max")]
    Max,
    /// # Minimum
    /// Keeps the smallest per-feature calculation value in the group.
    #[serde(rename = "min")]
    Min,
    /// # Count
    /// Sums the per-feature calculation value across the group; counts features when the value is 1.
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
        let variables = ctx.variables.clone();

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
                match code.eval(feature, variables.clone()) {
                    Ok(result) => aggregates.push(result),
                    Err(e) => {
                        ctx.report(
                            DiagnosticDraft::new(ErrorCode::ExprEvaluationFailed).with_message(
                                format!("Failed to evaluate aggregation expression: {e}"),
                            ),
                        )?;
                        // Resolved below Fatal: this feature contributes no value for this
                        // aggregate rather than failing the node.
                        continue;
                    }
                }
            }
        }
        let calc = match &self.aggregation {
            None => None,
            Some(Aggregation {
                value: Some(value), ..
            }) => Some(*value),
            Some(Aggregation {
                expr: Some(expr), ..
            }) => match expr.eval_int(feature, variables) {
                Ok(value) => Some(value),
                Err(e) => {
                    ctx.report(
                        DiagnosticDraft::new(ErrorCode::ExprEvaluationFailed).with_message(
                            format!("Failed to evaluate the calculation expression: {e}"),
                        ),
                    )?;
                    // Resolved below Fatal: this feature contributes nothing to the group
                    // rather than failing the node. Returning before touching `self.buffer` keeps
                    // an all-failed group absent instead of emitting a zero for it.
                    return Ok(());
                }
            },
            Some(_) => unreachable!("build() requires a value or an expression"),
        };
        let key = AttributeValue::Array(aggregates);
        let group = self.buffer.entry(key).or_insert_with(|| Group {
            #[cfg(feature = "new-geometry")]
            members: Vec::new(),
            #[cfg(feature = "new-geometry")]
            member_attrs: Vec::new(),
            kept: IndexMap::new(),
            value: None,
        });
        match self.attribute_accumulation {
            AttributeAccumulationStrategy::DropAttributes => {}
            AttributeAccumulationStrategy::UseOneFeature => {
                if group.kept.is_empty() {
                    group.kept = feature
                        .attributes
                        .iter()
                        .map(|(k, v)| (k.clone(), vec![v.clone()]))
                        .collect();
                }
            }
            AttributeAccumulationStrategy::MergeAttributes => {
                for (k, v) in feature.attributes.iter() {
                    let values = group.kept.entry(k.clone()).or_default();
                    if !values.contains(v) {
                        values.push(v.clone());
                    }
                }
            }
        }
        if let (Some(aggregation), Some(calc)) = (&self.aggregation, calc) {
            group.value = Some(match aggregation.method {
                Method::Max => group.value.unwrap_or(0).max(calc),
                Method::Min => group.value.unwrap_or(i64::MAX).min(calc),
                Method::Count => group.value.unwrap_or(0) + calc,
            });
        }
        #[cfg(feature = "new-geometry")]
        push_geometry(group, &feature.geometry);
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
        "Attribute Aggregator"
    }
}

impl AttributeAggregator {
    pub(crate) fn flush_buffer(&self, ctx: Context, fw: &ProcessorChannelForwarder) {
        self.buffer.par_iter().for_each(|(key, group)| {
            // A key with one distinct value keeps it as is; several become an array.
            let kept: Attributes = group
                .kept
                .iter()
                .map(|(k, values)| {
                    let value = match values.as_slice() {
                        [only] => only.clone(),
                        _ => AttributeValue::Array(values.clone()),
                    };
                    (k.clone(), value)
                })
                .collect();
            #[cfg(feature = "new-geometry")]
            let mut feature = Feature::new_with_attributes_and_geometry(kept, combine_geometry(group));
            #[cfg(not(feature = "new-geometry"))]
            let mut feature = Feature::new_with_attributes(kept);
            let AttributeValue::Array(aggregates) = key else {
                return;
            };
            for (i, aggregate_attribute) in self.aggregate_attributes.iter().enumerate() {
                feature.insert(
                    &aggregate_attribute.new_attribute,
                    aggregates.get(i).cloned().unwrap_or(AttributeValue::Null),
                );
            }
            if let (Some(aggregation), Some(value)) = (&self.aggregation, group.value) {
                feature.insert(
                    &aggregation.attribute,
                    AttributeValue::Number(serde_json::Number::from(value)),
                );
            }
            fw.send(ExecutorContext::new_with_context_feature_and_port(
                &ctx,
                feature,
                FEATURES_PORT.clone(),
            ));
        });
    }
}

/// Add `geometry` to the group's members. A collection contributes its members (each
/// with its own attributes) rather than nesting, and an absent geometry contributes
/// nothing.
#[cfg(feature = "new-geometry")]
fn push_geometry(group: &mut Group, geometry: &Geometry) {
    match geometry {
        Geometry::None => {}
        Geometry::GeometryCollection(collection) => {
            let attrs = collection.member_attributes();
            for (i, member) in collection.members().iter().enumerate() {
                group.members.push(member.clone());
                group
                    .member_attrs
                    .push(attrs.get(i).cloned().unwrap_or_default());
            }
        }
        other => {
            group.members.push(other.clone());
            group.member_attrs.push(Attributes::new());
        }
    }
}

/// The group's geometry: absent when no feature had one, the geometry itself when
/// exactly one plain geometry was collected, and a collection otherwise.
#[cfg(feature = "new-geometry")]
fn combine_geometry(group: &Group) -> Geometry {
    match group.members.as_slice() {
        [] => Geometry::None,
        [only] if group.member_attrs[0].is_empty() => only.clone(),
        members => {
            let attrs = if group.member_attrs.iter().all(|a| a.is_empty()) {
                Vec::new()
            } else {
                group.member_attrs.clone()
            };
            Geometry::GeometryCollection(
                GeometryCollection::with_attributes(members.to_vec(), attrs)
                    .expect("member_attrs is kept parallel to members"),
            )
        }
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
            aggregation: Some(Aggregation {
                attribute: Attribute::new("count"),
                method: Method::Count,
                value: Some(1),
                expr: None,
            }),
            attribute_accumulation: AttributeAccumulationStrategy::DropAttributes,
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
            ports.iter().all(|p| *p == *FEATURES_PORT),
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
