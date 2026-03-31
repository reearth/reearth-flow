use std::{collections::HashMap, sync::Arc};

use once_cell::sync::Lazy;
use reearth_flow_expr::eval::{Context, NativeFn};
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::Feature;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::FeatureProcessorError;

static UNFILTERED_PORT: Lazy<Port> = Lazy::new(|| Port::new("unfiltered"));

#[derive(Debug, Clone, Default)]
pub(super) struct FeatureFilter2Factory;

impl ProcessorFactory for FeatureFilter2Factory {
    fn name(&self) -> &str {
        "FeatureFilter2"
    }

    fn description(&self) -> &str {
        "Filter Features Based on Custom Conditions (new expr engine)"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(FeatureFilter2Param))
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
        let params: FeatureFilter2Param = if let Some(with) = with {
            let value = serde_json::to_value(with).map_err(|e| {
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

        let mut conditions = Vec::new();
        for condition in params.conditions {
            let ast = reearth_flow_expr::parse(&condition.expr)
                .map_err(|e| FeatureProcessorError::FilterFactory(format!("{e}")))?;
            conditions.push(CompiledCondition {
                ast,
                output_port: condition.output_port,
            });
        }
        Ok(Box::new(FeatureFilter2 { conditions }))
    }
}

#[derive(Debug, Clone)]
struct FeatureFilter2 {
    conditions: Vec<CompiledCondition>,
}

#[derive(Debug, Clone)]
struct CompiledCondition {
    ast: reearth_flow_expr::ast::Expr,
    output_port: Port,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct FeatureFilter2Param {
    conditions: Vec<Condition>,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct Condition {
    expr: String,
    output_port: Port,
}

/// Build an eval `Context` from a feature, registering each attribute as a
/// native `var::<name>` function. Conversion from `AttributeValue` to
/// `serde_json::Value` happens only when the attribute is actually accessed.
fn context_from_feature(feature: &Feature) -> Context {
    let mut ctx = Context::new();

    for (attr, value) in feature.attributes.iter() {
        let name = attr.clone().into_inner();
        let value: Value = value.clone().into();
        let f: NativeFn = Box::new(move |_| Ok(value.clone()));
        ctx.register("var", name, f);
    }

    ctx
}

impl Processor for FeatureFilter2 {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let eval_ctx = context_from_feature(feature);
        let mut routed = false;

        for condition in &self.conditions {
            match reearth_flow_expr::eval(&condition.ast, &eval_ctx) {
                Ok(Value::Bool(true)) => {
                    fw.send(ctx.new_with_feature_and_port(
                        feature.clone(),
                        condition.output_port.clone(),
                    ));
                    routed = true;
                }
                Ok(_) => {}
                Err(e) => {
                    ctx.event_hub.error_log(
                        Some(ctx.error_span()),
                        format!("filter2 eval error: {e}"),
                    );
                }
            }
        }

        if !routed {
            fw.send(ctx.new_with_feature_and_port(feature.clone(), UNFILTERED_PORT.clone()));
        }
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
        "FeatureFilter2"
    }

    fn num_threads(&self) -> usize {
        5
    }
}
