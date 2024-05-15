use std::{collections::HashMap, sync::Arc};

use reearth_flow_eval_expr::engine::Engine;
use reearth_flow_runtime::{
    channels::ProcessorChannelForwarder,
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT, REJECTED_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, Expr, Feature};
use rhai::Dynamic;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::errors::ProcessorError;

#[derive(Debug, Clone, Default)]
pub struct FeatureTransformerFactory;

#[async_trait::async_trait]
impl ProcessorFactory for FeatureTransformerFactory {
    fn get_input_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    async fn build(
        &self,
        ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let params: FeatureTransformerParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                ProcessorError::FeatureTransformerFactory(format!(
                    "Failed to serialize with: {}",
                    e
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                ProcessorError::FeatureTransformerFactory(format!(
                    "Failed to deserialize with: {}",
                    e
                ))
            })?
        } else {
            return Err(ProcessorError::FeatureTransformerFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };

        let expr_engine = Arc::clone(&ctx.expr_engine);
        let mut transformers = Vec::new();
        for condition in &params.transformers {
            let expr = &condition.expr;
            let template_ast = expr_engine
                .compile(expr.as_ref())
                .map_err(|e| ProcessorError::FeatureTransformerFactory(format!("{:?}", e)))?;
            transformers.push(CompiledTransform { expr: template_ast });
        }
        let process = FeatureTransformer { transformers };
        Ok(Box::new(process))
    }
}

#[derive(Debug, Clone)]
pub struct FeatureTransformer {
    transformers: Vec<CompiledTransform>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FeatureTransformerParam {
    transformers: Vec<Transform>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct Transform {
    expr: Expr,
}

#[derive(Debug, Clone)]
struct CompiledTransform {
    expr: rhai::AST,
}

impl Processor for FeatureTransformer {
    fn initialize(&mut self, _ctx: NodeContext) {}

    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &mut dyn ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let expr_engine = Arc::clone(&ctx.expr_engine);
        let feature = &ctx.feature;
        let mut new_feature = feature.clone();
        for transformer in &self.transformers {
            new_feature = mapper(&new_feature, &transformer.expr, expr_engine.clone());
        }
        fw.send(ctx.new_with_feature_and_port(new_feature, REJECTED_PORT.clone()));
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
        "FeatureTransformer"
    }
}

fn mapper(row: &Feature, expr: &rhai::AST, expr_engine: Arc<Engine>) -> Feature {
    let scope = expr_engine.new_scope();
    for (k, v) in row.attributes.iter() {
        scope.set(k.inner().as_str(), v.clone().into());
    }
    scope.set("__all", serde_json::to_value(&row.attributes).unwrap());
    let new_value = scope.eval_ast::<Dynamic>(expr);
    if let Ok(new_value) = new_value {
        if let Ok(AttributeValue::Map(new_value)) = new_value.try_into() {
            return Feature::new_with_attributes(
                new_value
                    .iter()
                    .map(|(k, v)| (Attribute::new(k.clone()), v.clone()))
                    .collect(),
            );
        }
    }
    row.clone()
}
