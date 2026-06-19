use std::collections::HashMap;

use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, Code, CodeType, CompiledCode, Feature};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::FeatureProcessorError;

#[derive(Debug, Clone, Default)]
pub(super) struct FeatureTransformerFactory;

impl ProcessorFactory for FeatureTransformerFactory {
    fn name(&self) -> &str {
        "FeatureTransformer"
    }

    fn description(&self) -> &str {
        "Applies transformation expressions to modify feature attributes and properties"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(FeatureTransformerParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Transform"]
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
        let params: FeatureTransformerParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                FeatureProcessorError::TransformerFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                FeatureProcessorError::TransformerFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(FeatureProcessorError::TransformerFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };

        let mut transformers = Vec::new();
        for condition in params.transformers {
            let expr = condition
                .expr
                .compile()
                .map_err(|e| FeatureProcessorError::TransformerFactory(format!("{e:?}")))?;
            transformers.push(CompiledTransform { expr });
        }
        Ok(Box::new(FeatureTransformer { transformers }))
    }
}

#[derive(Debug, Clone)]
struct FeatureTransformer {
    transformers: Vec<CompiledTransform>,
}

/// # FeatureTransformer Parameters
///
/// Configuration for applying transformation expressions to features.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct FeatureTransformerParam {
    /// List of transformation expressions to apply to each feature
    transformers: Vec<Transform>,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct Transform {
    /// Expression that modifies the feature (can access and modify attributes, geometry, etc.)
    expr: Code<{ CodeType::FlowExpr as u32 }>,
}

#[derive(Debug, Clone)]
struct CompiledTransform {
    expr: CompiledCode,
}

impl Processor for FeatureTransformer {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let env_vars = ctx.expr_engine.vars().clone();
        let mut new_feature = feature.clone();
        for transformer in &self.transformers {
            new_feature = mapper(&new_feature, &transformer.expr, env_vars.clone());
        }
        fw.send(ctx.new_with_feature_and_port(new_feature, DEFAULT_PORT.clone()));
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
        "FeatureTransformer"
    }
}

fn mapper(
    feature: &Feature,
    code: &CompiledCode,
    env_vars: std::sync::Arc<serde_json::Map<String, serde_json::Value>>,
) -> Feature {
    let Ok(new_value) = code.eval(feature, env_vars) else {
        return feature.clone();
    };
    if let AttributeValue::Map(new_value) = new_value {
        return Feature::new_with_attributes(
            new_value
                .iter()
                .map(|(k, v)| (Attribute::new(k.clone()), v.clone()))
                .collect(),
        );
    }
    feature.clone()
}
