use std::{collections::HashMap, sync::Arc};

use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, Expr};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::FeatureProcessorError;
use super::quickjs_helper::{js_to_json, JsEngine};

#[derive(Debug, Clone, Default)]
pub(super) struct JavaScriptCallerFactory;

impl ProcessorFactory for JavaScriptCallerFactory {
    fn name(&self) -> &str {
        "JavaScriptCaller"
    }

    fn description(&self) -> &str {
        "Executes JavaScript to process and transform features"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(JavaScriptCallerParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Feature"]
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
        let params: JavaScriptCallerParam = if let Some(with) = with.clone() {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                FeatureProcessorError::JavaScriptCallerFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                FeatureProcessorError::JavaScriptCallerFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(FeatureProcessorError::JavaScriptCallerFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };

        let mut engine = JsEngine::new().map_err(|e| {
            FeatureProcessorError::JavaScriptCallerFactory(e)
        })?;

        let fn_name = engine.compile_body(params.process.as_ref()).map_err(|e| {
            FeatureProcessorError::JavaScriptCallerFactory(format!(
                "Invalid process expression: {e}"
            ))
        })?;

        Ok(Box::new(JavaScriptCaller {
            engine,
            fn_name,
            global_params: with,
        }))
    }
}

#[derive(Debug, Clone)]
struct JavaScriptCaller {
    engine: JsEngine,
    fn_name: String,
    global_params: Option<HashMap<String, serde_json::Value>>,
}

/// # JavaScriptCaller Parameters
///
/// Execute a JavaScript program to transform features.
/// Use `value("key")` to lazily access feature attributes.
/// Use `env.key` to access global parameters.
/// Return an object to replace the feature attributes, or an array of objects to emit multiple features.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct JavaScriptCallerParam {
    /// JavaScript program body. Has access to `value(key)` and `env`.
    /// Must `return` an object or array of objects.
    process: Expr,
}

fn js_object_to_attributes(obj: &rquickjs::Object<'_>) -> Option<reearth_flow_types::Attributes> {
    let keys: Vec<String> = obj.keys::<String>().collect::<Result<Vec<_>, _>>().ok()?;
    let mut attrs = reearth_flow_types::Attributes::new();
    for key in keys {
        if let Ok(val) = obj.get::<_, rquickjs::Value>(&key) {
            let json_val = js_to_json(&val);
            let attr_val: AttributeValue = json_val.into();
            attrs.insert(Attribute::new(key), attr_val);
        }
    }
    Some(attrs)
}

impl Processor for JavaScriptCaller {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let attrs = Arc::clone(&feature.attributes);

        let result = self.engine.call_raw(
            &self.fn_name,
            &attrs,
            &self.global_params,
            |_js_ctx, result| {
                if let Some(arr) = result.as_array() {
                    for item in arr.iter::<rquickjs::Value>() {
                        let item = item.map_err(|e| format!("{e}"))?;
                        if let Some(obj) = item.as_object() {
                            if let Some(new_attrs) = js_object_to_attributes(obj) {
                                let mut new_feature = feature.clone();
                                new_feature.refresh_id();
                                new_feature.attributes = Arc::new(new_attrs);
                                fw.send(ctx.new_with_feature_and_port(
                                    new_feature,
                                    DEFAULT_PORT.clone(),
                                ));
                            }
                        }
                    }
                } else if let Some(obj) = result.as_object() {
                    if let Some(new_attrs) = js_object_to_attributes(obj) {
                        let new_feature = feature.clone().into_with_attributes(new_attrs);
                        fw.send(ctx.new_with_feature_and_port(new_feature, DEFAULT_PORT.clone()));
                    }
                } else {
                    fw.send(ctx.new_with_feature_and_port(feature.clone(), DEFAULT_PORT.clone()));
                }
                Ok(())
            },
        );

        result.map_err(|e| {
            FeatureProcessorError::JavaScriptCaller(format!("process eval error: {e}"))
        })?;

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
        "JavaScriptCaller"
    }
}
