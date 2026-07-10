use std::{collections::HashSet, sync::Arc};

use once_cell::sync::Lazy;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, FEATURES_PORT},
};
use reearth_flow_types::{Code, CodeType, CompiledCode};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use super::errors::FeatureProcessorError;

static UNFILTERED_PORT: Lazy<Port> = Lazy::new(|| Port::new("unfiltered"));

#[derive(Debug, Clone, Default)]
pub(super) struct FeatureFilterFactory;

impl ProcessorFactory for FeatureFilterFactory {
    fn name(&self) -> &str {
        "FeatureFilter"
    }

    fn description(&self) -> &str {
        "Filter Features Based on Custom Conditions"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(FeatureFilterParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Filter"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![FEATURES_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![UNFILTERED_PORT.clone()]
    }

    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let params: FeatureFilterParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                FeatureProcessorError::FilterFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                FeatureProcessorError::FilterFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(FeatureProcessorError::FilterFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        let mut seen_ports = HashSet::new();
        for condition in &params.conditions {
            if condition.output_port == *UNFILTERED_PORT {
                return Err(FeatureProcessorError::FilterFactory(format!(
                    "Condition output port '{}' conflicts with the reserved fallback port",
                    condition.output_port,
                ))
                .into());
            }
            if !seen_ports.insert(condition.output_port.clone()) {
                return Err(FeatureProcessorError::FilterFactory(format!(
                    "Duplicate condition output port '{}'",
                    condition.output_port,
                ))
                .into());
            }
        }
        let mut conditions = Vec::new();
        for condition in &params.conditions {
            let compiled = condition.expr.compile().map_err(|e| {
                FeatureProcessorError::FilterFactory(format!(
                    "failed to compile condition for port {:?}: {e:?}",
                    condition.output_port
                ))
            })?;
            conditions.push(CompiledCondition {
                expr: compiled,
                output_port: condition.output_port.clone(),
            });
        }
        Ok(Box::new(FeatureFilter { conditions }))
    }

    fn infer_output_schema(
        &self,
        inputs: &HashMap<Port, reearth_flow_types::attr_schema::AttrSchema>,
        with: &Option<HashMap<String, Value>>,
    ) -> Option<HashMap<Port, reearth_flow_types::attr_schema::AttrSchema>> {
        use reearth_flow_types::attr_schema::AttrSchema;

        // FeatureFilter routes whole features by expression; it never modifies
        // attributes. So every output port carries the input schema unchanged
        // (identity). Besides the static "unfiltered" fallback port, condition
        // output ports are dynamically derived from `with["conditions"]` (mirrors
        // the dynamic-port derivation in `builder_dag`).
        let input = inputs
            .get(&FEATURES_PORT.clone())
            .cloned()
            .unwrap_or_else(AttrSchema::open);

        let mut map: HashMap<Port, AttrSchema> = self
            .get_output_ports()
            .into_iter()
            .map(|port| (port, input.clone()))
            .collect();

        if let Some(with) = with {
            if let Some(Value::Array(conditions)) = with.get("conditions") {
                for condition in conditions {
                    if let Some(Value::String(port)) = condition.get("outputPort") {
                        map.entry(Port::new(port.clone()))
                            .or_insert_with(|| input.clone());
                    }
                }
            }
        }

        Some(map)
    }
}

#[derive(Debug, Clone)]
struct FeatureFilter {
    conditions: Vec<CompiledCondition>,
}

/// # Feature Filter Parameters
/// Configure the conditions and output ports for filtering features based on expressions
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct FeatureFilterParam {
    /// # Filter Conditions
    /// List of conditions and their corresponding output ports for routing filtered features
    conditions: Vec<Condition>,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct Condition {
    /// # Condition expression
    expr: Code<{ CodeType::FlowExpr as u32 }>,
    /// # Output port
    output_port: Port,
}

#[derive(Debug, Clone)]
struct CompiledCondition {
    expr: CompiledCode,
    output_port: Port,
}

impl Processor for FeatureFilter {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let env_vars = ctx.env_vars.clone();
        let feature = &ctx.feature;
        let mut routing = false;
        for condition in &self.conditions {
            match condition.expr.eval_bool(feature, Arc::clone(&env_vars)) {
                Ok(true) => {
                    fw.send(
                        ctx.new_with_feature_and_port(
                            feature.clone(),
                            condition.output_port.clone(),
                        ),
                    );
                    routing = true;
                }
                Ok(false) => {}
                Err(err) => {
                    ctx.event_hub.error_log(
                        Some(ctx.error_span()),
                        format!("filter eval error = {err:?}"),
                    );
                }
            }
        }
        if routing {
            return Ok(());
        }
        fw.send(ctx.new_with_feature_and_port(feature.clone(), UNFILTERED_PORT.clone()));
        Ok(())
    }

    fn finish(
        &mut self,
        _ctx: NodeContext,
        _fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "FeatureFilter"
    }

    fn num_threads(&self) -> usize {
        5
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reearth_flow_types::attr_schema::{AttrField, AttrSchema, AttrType};
    use reearth_flow_types::Attribute;

    #[test]
    fn infer_is_identity_on_each_output_port() {
        let mut input = AttrSchema::empty();
        input.insert(
            Attribute::new("a".to_string()),
            AttrField::always(AttrType::String),
        );
        input.insert(
            Attribute::new("b".to_string()),
            AttrField::always(AttrType::Number),
        );
        let mut inputs = HashMap::new();
        inputs.insert(FEATURES_PORT.clone(), input.clone());

        let out = FeatureFilterFactory
            .infer_output_schema(&inputs, &None)
            .expect("filter is schema-transparent and returns Some");

        let unfiltered = out
            .get(&UNFILTERED_PORT.clone())
            .expect("unfiltered port present");
        assert_eq!(*unfiltered, input);
    }

    #[test]
    fn infer_includes_condition_output_ports() {
        let mut input = AttrSchema::empty();
        input.insert(
            Attribute::new("a".to_string()),
            AttrField::always(AttrType::String),
        );
        let mut inputs = HashMap::new();
        inputs.insert(FEATURES_PORT.clone(), input.clone());

        // Condition param shape: { conditions: [ { expr, outputPort } ] }.
        let with: HashMap<String, Value> = serde_json::from_value(serde_json::json!({
            "conditions": [
                { "expr": "true", "outputPort": "matched" }
            ]
        }))
        .expect("with json");

        let out = FeatureFilterFactory
            .infer_output_schema(&inputs, &Some(with))
            .expect("filter is schema-transparent and returns Some");

        // Both the static fallback port and the dynamic condition port must be
        // present, each carrying the input schema unchanged (identity).
        let unfiltered = out
            .get(&UNFILTERED_PORT.clone())
            .expect("unfiltered port present");
        assert_eq!(*unfiltered, input);

        let matched = out
            .get(&Port::new("matched"))
            .expect("condition output port present");
        assert_eq!(*matched, input);
    }
}
