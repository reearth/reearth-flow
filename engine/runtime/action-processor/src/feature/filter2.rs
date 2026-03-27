use std::{collections::HashMap, sync::Arc};

use once_cell::sync::Lazy;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{Attribute, Expr};
use rquickjs::{Context, Function, Runtime};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::FeatureProcessorError;

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
                // Wrap as function taking value() callback + env object
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
    /// Use `value("key")` to access feature attributes lazily.
    /// Use `env.key` to access global parameters.
    /// Example: `value("area") > 100 && env.threshold < 50`
    expr: Expr,
    /// # Output port
    output_port: Port,
}

#[derive(Debug, Clone)]
struct JsCondition {
    expr: String,
    output_port: Port,
}

/// Convert a serde_json::Value into a QuickJS value (used only for env/small objects)
fn json_to_js<'js>(
    ctx: &rquickjs::Ctx<'js>,
    value: &serde_json::Value,
) -> rquickjs::Result<rquickjs::Value<'js>> {
    match value {
        serde_json::Value::Null => Ok(rquickjs::Value::new_null(ctx.clone())),
        serde_json::Value::Bool(b) => Ok(rquickjs::Value::new_bool(ctx.clone(), *b)),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(rquickjs::Value::new_int(ctx.clone(), i as i32))
            } else if let Some(f) = n.as_f64() {
                Ok(rquickjs::Value::new_float(ctx.clone(), f))
            } else {
                Ok(rquickjs::Value::new_null(ctx.clone()))
            }
        }
        serde_json::Value::String(s) => {
            let js_str = rquickjs::String::from_str(ctx.clone(), s)?;
            Ok(js_str.into_value())
        }
        serde_json::Value::Array(arr) => {
            let js_arr = rquickjs::Array::new(ctx.clone())?;
            for (i, v) in arr.iter().enumerate() {
                let js_val = json_to_js(ctx, v)?;
                js_arr.set(i, js_val)?;
            }
            Ok(js_arr.into_value())
        }
        serde_json::Value::Object(map) => {
            let js_obj = rquickjs::Object::new(ctx.clone())?;
            for (k, v) in map {
                let js_val = json_to_js(ctx, v)?;
                js_obj.set(k.as_str(), js_val)?;
            }
            Ok(js_obj.into_value())
        }
    }
}

impl Processor for FeatureFilterV2 {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let mut routing = false;

        // Keep attributes behind Arc — no conversion happens here.
        // Only the single attribute requested by value("key") is converted.
        let attrs = Arc::clone(&feature.attributes);

        // Build env object from global_params (typically small)
        let env_value: serde_json::Value = if let Some(ref params) = self.global_params {
            serde_json::Value::Object(
                params
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect(),
            )
        } else {
            serde_json::Value::Object(serde_json::Map::new())
        };

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
                // Create a `value(key)` function that lazily fetches a single
                // attribute from Rust, converting only the requested value to JS.
                // No bulk conversion of the feature attributes map happens.
                let js2 = js.clone();
                let value_fn = Function::new(js.clone(), move |key: String| {
                    match attrs_clone.get(&Attribute::new(key)) {
                        Some(v) => {
                            let json_val: serde_json::Value = v.clone().into();
                            json_to_js(&js2, &json_val)
                        }
                        None => Ok(rquickjs::Value::new_undefined(js2.clone())),
                    }
                })
                .map_err(|e| -> BoxedError {
                    FeatureProcessorError::FilterV2(format!(
                        "Failed to create value() function: {e}"
                    ))
                    .into()
                })?;

                // env is typically small (a few params), so convert it fully
                let js_env = json_to_js(&js, &env_value).map_err(|e| -> BoxedError {
                    FeatureProcessorError::FilterV2(format!("Failed to convert env: {e}")).into()
                })?;

                // Expression is called as: (function(value, env) { return (<expr>); })(value_fn, env)
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
