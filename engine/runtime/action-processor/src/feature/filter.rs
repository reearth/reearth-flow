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
        ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let params: FeatureFilterParam = if let Some(with) = with.clone() {
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
        let expr_engine = Arc::clone(&ctx.expr_engine);
        let mut conditions = Vec::new();
        for condition in &params.conditions {
            let expr = &condition.expr;
            let template_ast = expr_engine
                .compile(expr.as_ref())
                .map_err(|e| FeatureProcessorError::FilterFactory(format!("{e:?}")))?;
            let output_port = &condition.output_port;
            conditions.push(CompiledCondition {
                expr: template_ast,
                output_port: output_port.clone(),
            });
        }
        let process = FeatureFilter {
            global_params: with,
            conditions,
        };
        Ok(Box::new(process))
    }
}

#[derive(Debug, Clone)]
struct FeatureFilter {
    global_params: Option<HashMap<String, serde_json::Value>>,
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
    expr: Expr,
    /// # Output port
    output_port: Port,
}

#[derive(Debug, Clone)]
struct CompiledCondition {
    expr: rhai::AST,
    output_port: Port,
}

impl Processor for FeatureFilter {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let expr_engine = Arc::clone(&ctx.expr_engine);
        let feature = &ctx.feature;
        let mut routing = false;
        let scope = feature.new_scope(expr_engine.clone(), &self.global_params);
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
                    ctx.event_hub.error_log(
                        Some(ctx.error_span()),
                        format!("filter eval error = {err:?}"),
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

    fn finish(&self, _ctx: NodeContext, _fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "FeatureFilter"
    }
}
