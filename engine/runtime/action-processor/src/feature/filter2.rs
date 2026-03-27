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
use rquickjs::{Context, Function, Runtime};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::FeatureProcessorError;
use super::quickjs_helper::{make_env_js, make_value_fn};

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

        // Validate that all expressions are valid JS by doing a trial compile
        let rt = Runtime::new().map_err(|e| {
            FeatureProcessorError::FilterV2Factory(format!("Failed to create JS runtime: {e}"))
        })?;
        let js_ctx = Context::full(&rt).map_err(|e| {
            FeatureProcessorError::FilterV2Factory(format!("Failed to create JS context: {e}"))
        })?;
        for condition in &params.conditions {
            let expr = condition.expr.as_ref().to_string();
            js_ctx.with(|ctx| {
                let wrapper = format!("(function(value, env) {{ return ({expr}); }})");
                ctx.eval::<rquickjs::Value, _>(wrapper.into_bytes())
                    .map_err(|e| {
                        FeatureProcessorError::FilterV2Factory(format!(
                            "Invalid JS expression '{expr}': {e}"
                        ))
                    })?;
                Ok::<(), FeatureProcessorError>(())
            })?;
        }

        let conditions: Vec<JsCondition> = params
            .conditions
            .into_iter()
            .map(|c| JsCondition {
                expr: c.expr.as_ref().to_string(),
                output_port: c.output_port,
            })
            .collect();

        let process = FeatureFilterV2 {
            global_params: with,
            conditions,
        };
        Ok(Box::new(process))
    }
}

#[derive(Debug, Clone)]
struct FeatureFilterV2 {
    global_params: Option<HashMap<String, serde_json::Value>>,
    conditions: Vec<JsCondition>,
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

#[derive(Debug, Clone)]
struct JsCondition {
    expr: String,
    output_port: Port,
}

impl Processor for FeatureFilterV2 {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let mut routing = false;

        let attrs = Arc::clone(&feature.attributes);

        let rt = Runtime::new().map_err(|e| {
            FeatureProcessorError::FilterV2(format!("Failed to create JS runtime: {e}"))
        })?;
        let js_ctx = Context::full(&rt).map_err(|e| {
            FeatureProcessorError::FilterV2(format!("Failed to create JS context: {e}"))
        })?;

        for condition in &self.conditions {
            let expr = &condition.expr;
            let attrs_clone = Arc::clone(&attrs);
            let eval_result: Result<bool, BoxedError> = js_ctx.with(|js| {
                let value_fn = make_value_fn(&js, attrs_clone).map_err(|e| -> BoxedError {
                    FeatureProcessorError::FilterV2(format!(
                        "Failed to create value() function: {e}"
                    ))
                    .into()
                })?;
                let js_env = make_env_js(&js, &self.global_params).map_err(|e| -> BoxedError {
                    FeatureProcessorError::FilterV2(format!("Failed to convert env: {e}")).into()
                })?;

                let code = format!("(function(value, env) {{ return ({expr}); }})");
                let func: Function = js.eval(code.into_bytes()).map_err(|e| -> BoxedError {
                    FeatureProcessorError::FilterV2(format!("Failed to compile expr: {e}")).into()
                })?;
                let result: bool = func.call((value_fn, js_env)).map_err(|e| -> BoxedError {
                    FeatureProcessorError::FilterV2(format!("Failed to eval expr: {e}")).into()
                })?;
                Ok(result)
            });

            match eval_result {
                Ok(true) => {
                    fw.send(ctx.new_with_feature_and_port(
                        feature.clone(),
                        condition.output_port.clone(),
                    ));
                    routing = true;
                }
                Ok(false) => {}
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
