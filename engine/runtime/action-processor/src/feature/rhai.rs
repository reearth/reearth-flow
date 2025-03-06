use std::{collections::HashMap, sync::Arc};

use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, Expr};
use rhai::Dynamic;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::FeatureProcessorError;

#[derive(Debug, Clone, Default)]
pub(super) struct RhaiCallerFactory;

impl ProcessorFactory for RhaiCallerFactory {
    fn name(&self) -> &str {
        "RhaiCaller"
    }

    fn description(&self) -> &str {
        "Calls Rhai script"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(RhaiCallerParam))
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
        ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let params: RhaiCallerParam = if let Some(with) = with.clone() {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                FeatureProcessorError::RhaiCallerFactory(format!(
                    "Failed to serialize `with` parameter: {}",
                    e
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                FeatureProcessorError::RhaiCallerFactory(format!(
                    "Failed to deserialize `with` parameter: {}",
                    e
                ))
            })?
        } else {
            return Err(FeatureProcessorError::RhaiCallerFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };

        let expr_engine = Arc::clone(&ctx.expr_engine);
        let is_target_ast = expr_engine
            .compile(params.is_target.into_inner().as_str())
            .map_err(|e| FeatureProcessorError::RhaiCallerFactory(format!("{:?}", e)))?;
        let process_ast = expr_engine
            .compile(params.process.into_inner().as_str())
            .map_err(|e| FeatureProcessorError::RhaiCallerFactory(format!("{:?}", e)))?;
        let process = RhaiCaller {
            global_params: with,
            is_target: is_target_ast,
            process: process_ast,
        };
        Ok(Box::new(process))
    }
}

#[derive(Debug, Clone)]
struct RhaiCaller {
    global_params: Option<HashMap<String, serde_json::Value>>,
    is_target: rhai::AST,
    process: rhai::AST,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct RhaiCallerParam {
    /// # Rhai script to determine if the feature is the target
    is_target: Expr,
    /// # Rhai script to process the feature
    process: Expr,
}

impl Processor for RhaiCaller {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let expr_engine = Arc::clone(&ctx.expr_engine);
        let feature = &ctx.feature;
        let scope = feature.new_scope(expr_engine.clone(), &self.global_params);
        let is_target = scope.eval_ast::<bool>(&self.is_target);
        if let Err(e) = is_target {
            return Err(FeatureProcessorError::RhaiCaller(format!("{:?}", e)).into());
        }
        if !is_target.unwrap() {
            fw.send(ctx.new_with_feature_and_port(feature.clone(), DEFAULT_PORT.clone()));
            return Ok(());
        }
        let new_value = scope.eval_ast::<Dynamic>(&self.process);
        if let Ok(new_value) = new_value {
            if new_value.is::<rhai::Map>() {
                if let Ok(AttributeValue::Map(new_value)) = new_value.try_into() {
                    let mut feature = feature.clone();
                    feature.attributes = new_value
                        .iter()
                        .map(|(k, v)| (Attribute::new(k.clone()), v.clone()))
                        .collect();
                    fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
                    return Ok(());
                }
            } else if new_value.is::<rhai::Array>() {
                let array_values = new_value.clone().into_array().unwrap();
                for new_value in array_values {
                    if let Ok(AttributeValue::Map(new_value)) = new_value.try_into() {
                        let mut feature = feature.clone();
                        feature.refresh_id();
                        feature.attributes = new_value
                            .iter()
                            .map(|(k, v)| (Attribute::new(k.clone()), v.clone()))
                            .collect();
                        fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
                    }
                }
                return Ok(());
            }
        }
        fw.send(ctx.new_with_feature_and_port(feature.clone(), DEFAULT_PORT.clone()));
        Ok(())
    }

    fn finish(&self, _ctx: NodeContext, _fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "RhaiCaller"
    }
}
