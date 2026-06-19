use std::collections::HashMap;

use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, Code, CompiledCode};
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

        let mappings = params
            .mappings
            .into_iter()
            .map(|m| -> Result<(String, CompiledCode), BoxedError> {
                let compiled = m.value.compile().map_err(|e| {
                    format!("Failed to compile expression for '{}': {e}", m.attribute)
                })?;
                Ok((m.attribute, compiled))
            })
            .collect::<Result<Vec<_>, _>>()?;

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
    value: Code,
}

#[derive(Debug, Clone)]
struct FlowExprTest {
    mappings: Vec<(String, CompiledCode)>,
}

impl Processor for FlowExprTest {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let env_vars = ctx.env_vars.clone();
        let mut feature = ctx.feature.clone();

        for (attr, code) in &self.mappings {
            let value = match code.eval(&ctx.feature, env_vars.clone()) {
                Ok(v) => v,
                Err(e) => {
                    tracing::error!(attr, error = %e, "FlowExprTest eval error");
                    AttributeValue::Null
                }
            };
            feature.insert(Attribute::new(attr.clone()), value);
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
