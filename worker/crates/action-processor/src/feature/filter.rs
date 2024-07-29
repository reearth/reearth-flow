use std::{collections::HashMap, sync::Arc};

use reearth_flow_action_log::action_error_log;
use reearth_flow_runtime::{
    channels::ProcessorChannelForwarder,
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT, REJECTED_PORT},
};
use reearth_flow_types::Expr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::FeatureProcessorError;

#[derive(Debug, Clone, Default)]
pub struct FeatureFilterFactory;

impl ProcessorFactory for FeatureFilterFactory {
    fn name(&self) -> &str {
        "FeatureFilter"
    }

    fn description(&self) -> &str {
        "Filters features based on conditions"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(FeatureFilterParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Feature"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![REJECTED_PORT.clone()]
    }

    fn build(
        &self,
        ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let params: FeatureFilterParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                FeatureProcessorError::FilterFactory(format!("Failed to serialize with: {}", e))
            })?;
            serde_json::from_value(value).map_err(|e| {
                FeatureProcessorError::FilterFactory(format!("Failed to deserialize with: {}", e))
            })?
        } else {
            return Err(FeatureProcessorError::FilterFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        let expr_engine = Arc::clone(&ctx.expr_engine);
        let mut conditions = Vec::new();
        for condition in &params.conditions {
            let expr = &condition.expr;
            let template_ast = expr_engine
                .compile(expr.as_ref())
                .map_err(|e| FeatureProcessorError::FilterFactory(format!("{:?}", e)))?;
            let output_port = &condition.output_port;
            conditions.push(CompiledCondition {
                expr: template_ast,
                output_port: output_port.clone(),
            });
        }
        let process = FeatureFilter { conditions };
        Ok(Box::new(process))
    }
}

#[derive(Debug, Clone)]
pub struct FeatureFilter {
    conditions: Vec<CompiledCondition>,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct FeatureFilterParam {
    conditions: Vec<Condition>,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct Condition {
    expr: Expr,
    output_port: Port,
}

#[derive(Debug, Clone)]
struct CompiledCondition {
    expr: rhai::AST,
    output_port: Port,
}

impl Processor for FeatureFilter {
    fn initialize(&mut self, _ctx: NodeContext) {}

    fn num_threads(&self) -> usize {
        5
    }

    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &mut dyn ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let expr_engine = Arc::clone(&ctx.expr_engine);
        let feature = &ctx.feature;
        let mut routing = false;
        let scope = feature.new_scope(expr_engine.clone());
        for condition in &self.conditions {
            let eval = scope.eval_ast::<bool>(&condition.expr);
            match eval {
                Ok(eval) if eval => {
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
                    action_error_log!(
                        parent: ctx.error_span(), ctx.logger.action_logger("FeatureFilter"), "filter eval error = {:?}", err,
                    );
                    continue;
                }
            }
        }
        if routing {
            return Ok(());
        }
        fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
        Ok(())
    }

    fn finish(
        &self,
        _ctx: NodeContext,
        _fw: &mut dyn ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "FeatureFilter"
    }
}
