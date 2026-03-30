use std::{collections::HashMap, sync::Arc};

use once_cell::sync::Lazy;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::Expr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::FeatureProcessorError;
use super::quickjs_helper::JsEngine;

static UNFILTERED_PORT: Lazy<Port> = Lazy::new(|| Port::new("unfiltered"));

#[derive(Debug, Clone, Default)]
pub(super) struct FeatureFilterV2Factory;

impl ProcessorFactory for FeatureFilterV2Factory {
    fn name(&self) -> &str {
        "FeatureFilterV2"
    }

    fn description(&self) -> &str {
        "Filter Features Based on Custom Conditions (QuickJS)"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(FeatureFilterV2Param))
    }

    fn categories(&self) -> &[&'static str] {
        &["Feature"]
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
        let params: FeatureFilterV2Param = if let Some(with) = with.clone() {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                FeatureProcessorError::FilterV2Factory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                FeatureProcessorError::FilterV2Factory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(FeatureProcessorError::FilterV2Factory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };

        let mut engine = JsEngine::new().map_err(|e| {
            FeatureProcessorError::FilterV2Factory(e)
        })?;

        let mut conditions = Vec::with_capacity(params.conditions.len());
        for condition in &params.conditions {
            let name = engine.compile_expr(condition.expr.as_ref()).map_err(|e| {
                FeatureProcessorError::FilterV2Factory(format!(
                    "Invalid expression '{}': {e}",
                    condition.expr.as_ref()
                ))
            })?;
            conditions.push(CompiledCondition {
                fn_name: name,
                output_port: condition.output_port.clone(),
            });
        }

        Ok(Box::new(FeatureFilterV2 {
            engine,
            global_params: with,
            conditions,
        }))
    }
}

#[derive(Debug, Clone)]
struct FeatureFilterV2 {
    engine: JsEngine,
    global_params: Option<HashMap<String, serde_json::Value>>,
    conditions: Vec<CompiledCondition>,
}

#[derive(Debug, Clone)]
struct CompiledCondition {
    fn_name: String,
    output_port: Port,
}

/// # Feature Filter V2 Parameters
/// Configure the conditions and output ports for filtering features based on JavaScript expressions
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct FeatureFilterV2Param {
    /// # Filter Conditions
    /// List of conditions and their corresponding output ports for routing filtered features
    conditions: Vec<ConditionV2>,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct ConditionV2 {
    /// # Condition expression (JavaScript)
    expr: Expr,
    /// # Output port
    output_port: Port,
}

impl Processor for FeatureFilterV2 {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let attrs = Arc::clone(&feature.attributes);
        let mut routing = false;

        for condition in &self.conditions {
            let result = self.engine.call(&condition.fn_name, &attrs, &self.global_params);

            match result {
                Ok(serde_json::Value::Bool(true)) => {
                    fw.send(ctx.new_with_feature_and_port(
                        feature.clone(),
                        condition.output_port.clone(),
                    ));
                    routing = true;
                }
                Ok(_) => {}
                Err(err) => {
                    ctx.event_hub.error_log(
                        Some(ctx.error_span()),
                        format!("filter v2 eval error = {err:?}"),
                    );
                    continue;
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
        "FeatureFilterV2"
    }

    fn num_threads(&self) -> usize {
        5
    }
}
