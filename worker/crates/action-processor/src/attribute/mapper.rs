use std::{collections::HashMap, sync::Arc};

use reearth_flow_runtime::{
    channels::ProcessorChannelForwarder,
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, Expr};
use rhai::Dynamic;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::AttributeProcessorError;

#[derive(Debug, Clone, Default)]
pub struct AttributeMapperFactory;

impl ProcessorFactory for AttributeMapperFactory {
    fn name(&self) -> &str {
        "AttributeMapper"
    }

    fn description(&self) -> &str {
        "Maps attributes"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(AttributeMapperParam))
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
        ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let params: AttributeMapperParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                AttributeProcessorError::MapperFactory(format!(
                    "Failed to serialize `with` parameter: {}",
                    e
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                AttributeProcessorError::MapperFactory(format!(
                    "Failed to deserialize `with` parameter: {}",
                    e
                ))
            })?
        } else {
            return Err(AttributeProcessorError::MapperFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        let expr_engine = Arc::clone(&ctx.expr_engine);
        let mut mappers = Vec::<CompiledMapper>::new();
        for mapper in &params.mappers {
            let expr = &mapper.expr;
            let template_ast = expr_engine
                .compile(expr.as_ref())
                .map_err(|e| AttributeProcessorError::MapperFactory(format!("{:?}", e)))?;
            mappers.push(CompiledMapper {
                expr: template_ast,
                attribute: mapper.attribute.clone(),
            });
        }

        let processor = AttributeMapper {
            mapper: CompiledAttributeMapperParam { mappers },
        };
        Ok(Box::new(processor))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AttributeMapperParam {
    mappers: Vec<Mapper>,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct Mapper {
    attribute: String,
    expr: Expr,
}

#[derive(Debug, Clone)]
pub struct CompiledAttributeMapperParam {
    mappers: Vec<CompiledMapper>,
}

#[derive(Debug, Clone)]
struct CompiledMapper {
    attribute: String,
    expr: rhai::AST,
}

#[derive(Debug, Clone)]
pub struct AttributeMapper {
    mapper: CompiledAttributeMapperParam,
}

impl Processor for AttributeMapper {
    fn initialize(&mut self, _ctx: NodeContext) {}
    fn num_threads(&self) -> usize {
        5
    }
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &mut dyn ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let expr_engine = Arc::clone(&ctx.expr_engine);
        let mut attributes = HashMap::<Attribute, AttributeValue>::new();
        let scope = feature.new_scope(expr_engine.clone());
        for mapper in &self.mapper.mappers {
            let new_value = scope.eval_ast::<Dynamic>(&mapper.expr);
            if let Ok(new_value) = new_value {
                if let Ok(new_value) = new_value.try_into() {
                    attributes.insert(Attribute::new(mapper.attribute.clone()), new_value);
                }
            }
        }
        fw.send(
            ctx.new_with_feature_and_port(
                feature.with_attributes(attributes),
                DEFAULT_PORT.clone(),
            ),
        );
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
        "AttributeMapper"
    }
}
