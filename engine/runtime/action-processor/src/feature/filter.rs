use std::{collections::HashSet, sync::Arc};

use once_cell::sync::Lazy;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{AttributeValue, Code, CompiledCode};
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
        vec![DEFAULT_PORT.clone()]
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
            let compiled = condition
                .expr
                .compile()
                .map_err(|e| FeatureProcessorError::FilterFactory(format!("{e:?}")))?;
            conditions.push(CompiledCondition {
                expr: compiled,
                output_port: condition.output_port.clone(),
            });
        }
        Ok(Box::new(FeatureFilter { conditions }))
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
    expr: Code,
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
        let env_vars = ctx.expr_engine.vars();
        let feature = &ctx.feature;
        let mut routing = false;
        for condition in &self.conditions {
            match condition.expr.eval(feature, Arc::clone(&env_vars)) {
                Ok(AttributeValue::Bool(true)) => {
                    fw.send(
                        ctx.new_with_feature_and_port(
                            feature.clone(),
                            condition.output_port.clone(),
                        ),
                    );
                    routing = true;
                }
                Ok(_) => {}
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
