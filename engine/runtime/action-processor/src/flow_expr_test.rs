use std::collections::HashMap;

use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, StringOrExpr, StringOrExprType};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Default)]
pub(crate) struct FlowExprTestFactory;

impl ProcessorFactory for FlowExprTestFactory {
    fn name(&self) -> &str {
        "FlowExprTest"
    }

    fn description(&self) -> &str {
        "Experimental testbed for the Flow expression engine"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(FlowExprTestParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Attribute"]
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
        let params: FlowExprTestParam = if let Some(with) = with {
            let value = serde_json::to_value(with)
                .map_err(|e| format!("Failed to serialize `with` parameter: {e}"))?;
            serde_json::from_value(value)
                .map_err(|e| format!("Failed to deserialize `with` parameter: {e}"))?
        } else {
            return Err("Missing required parameter `with`".into());
        };

        let mut mappings = Vec::new();
        for m in params.mappings {
            let compiled = match m.value.kind {
                StringOrExprType::Expr => {
                    let ast = reearth_flow_expr::compile(&m.value.value)
                        .map_err(|e| format!("Failed to parse expression: {e}"))?;
                    CompiledMapping {
                        attribute: m.attribute,
                        kind: CompiledValue::Expr(ast),
                    }
                }
                StringOrExprType::String => CompiledMapping {
                    attribute: m.attribute,
                    kind: CompiledValue::Literal(m.value.value),
                },
            };
            mappings.push(compiled);
        }

        Ok(Box::new(FlowExprTest { mappings }))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct FlowExprTestParam {
    mappings: Vec<Mapping>,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct Mapping {
    attribute: String,
    value: StringOrExpr,
}

#[derive(Debug, Clone)]
enum CompiledValue {
    Expr(reearth_flow_expr::CompiledExpr),
    Literal(String),
}

#[derive(Debug, Clone)]
struct CompiledMapping {
    attribute: String,
    kind: CompiledValue,
}

#[derive(Debug, Clone)]
struct FlowExprTest {
    mappings: Vec<CompiledMapping>,
}

impl Processor for FlowExprTest {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let eval_ctx = reearth_flow_expr::flow::context_from_feature(
            feature,
            std::sync::Arc::new(ctx.expr_engine.vars()),
        );
        let mut feature = feature.clone();

        for mapping in &self.mappings {
            let value = match &mapping.kind {
                CompiledValue::Expr(ast) => match reearth_flow_expr::eval(ast, &eval_ctx) {
                    Ok(v) => reearth_flow_expr::flow::attribute_value_from_eval(v),
                    Err(e) => {
                        ctx.event_hub.error_log(
                            Some(ctx.error_span()),
                            format!("FlowExprTest eval error for '{}': {e}", mapping.attribute),
                        );
                        AttributeValue::Null
                    }
                },
                CompiledValue::Literal(s) => AttributeValue::String(s.clone()),
            };
            feature.insert(Attribute::new(mapping.attribute.clone()), value);
        }

        fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
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
        "FlowExprTest"
    }
}
