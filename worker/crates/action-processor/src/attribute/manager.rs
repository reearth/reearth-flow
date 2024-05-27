use std::{collections::HashMap, sync::Arc};

use reearth_flow_runtime::{
    channels::ProcessorChannelForwarder,
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use rhai::Dynamic;

use reearth_flow_eval_expr::engine::Engine;
use reearth_flow_types::{Expr, Feature};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::AttributeProcessorError;

#[derive(Debug, Clone, Default)]
pub struct AttributeManagerFactory;

impl ProcessorFactory for AttributeManagerFactory {
    fn name(&self) -> &str {
        "AttributeManager"
    }

    fn description(&self) -> &str {
        "Manages attributes"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(AttributeManagerParam))
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
        let params: AttributeManagerParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                AttributeProcessorError::ManagerFactory(format!("Failed to serialize with: {}", e))
            })?;
            serde_json::from_value(value).map_err(|e| {
                AttributeProcessorError::ManagerFactory(format!(
                    "Failed to deserialize with: {}",
                    e
                ))
            })?
        } else {
            return Err(AttributeProcessorError::ManagerFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };

        let expr_engine = Arc::clone(&ctx.expr_engine);
        let operations = convert_single_operation(&params.operations, Arc::clone(&expr_engine));

        let process = AttributeManager { operations };
        Ok(Box::new(process))
    }
}

#[derive(Debug, Clone)]
pub struct AttributeManager {
    operations: Vec<Operate>,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AttributeManagerParam {
    operations: Vec<Operation>,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(super) struct Operation {
    pub(super) attribute: String,
    pub(super) method: Method,
    pub(super) value: Option<Expr>,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(super) enum Method {
    Convert,
    Create,
    Rename,
    Remove,
}

#[derive(Debug, Clone)]
pub(super) enum Operate {
    Convert {
        expr: Option<rhai::AST>,
        attribute: String,
    },
    Create {
        expr: Option<rhai::AST>,
        attribute: String,
    },
    Rename {
        new_key: String,
        attribute: String,
    },
    Remove {
        attribute: String,
    },
}

impl Processor for AttributeManager {
    fn initialize(&mut self, _ctx: NodeContext) {}

    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &mut dyn ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = process_feature(&ctx.feature, &self.operations, Arc::clone(&ctx.expr_engine));
        fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
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
        "AttributeManager"
    }
}

fn process_feature(row: &Feature, operations: &[Operate], expr_engine: Arc<Engine>) -> Feature {
    let mut result = row.clone();
    for operation in operations {
        match operation {
            Operate::Convert { expr, attribute } => {
                let value = row.get(attribute);
                if value.is_none() {
                    continue;
                }
                let scope = expr_engine.new_scope();
                for (k, v) in row.iter() {
                    scope.set(k.inner().as_str(), v.clone().into());
                }
                if let Some(expr) = expr {
                    let new_value = scope.eval_ast::<Dynamic>(expr);
                    if let Ok(new_value) = new_value {
                        if let Ok(new_value) = new_value.try_into() {
                            result.insert(attribute.clone(), new_value);
                        }
                    }
                }
            }
            Operate::Create { expr, attribute } => {
                let scope = expr_engine.new_scope();
                for (k, v) in row.iter() {
                    scope.set(k.inner().as_str(), v.clone().into());
                }
                if let Some(expr) = expr {
                    let new_value = scope.eval_ast::<Dynamic>(expr);
                    if let Ok(new_value) = new_value {
                        if let Ok(new_value) = new_value.try_into() {
                            result.insert(attribute.clone(), new_value);
                        }
                    }
                }
            }
            Operate::Rename { new_key, attribute } => {
                let value = row.get(attribute);
                if value.is_none() {
                    continue;
                }
                result.remove(attribute);
                result.insert(new_key.clone(), value.unwrap().clone());
            }
            Operate::Remove { attribute } => {
                let value = row.get(attribute);
                if value.is_none() {
                    continue;
                }
                result.remove(attribute);
            }
        };
    }
    result
}

fn convert_single_operation(operations: &[Operation], expr_engine: Arc<Engine>) -> Vec<Operate> {
    operations
        .iter()
        .map(|operation| {
            let method = &operation.method;
            let attribute = &operation.attribute;
            let value = operation.value.clone().unwrap_or(Expr::new(""));
            match method {
                Method::Convert => Operate::Convert {
                    expr: expr_engine.compile(value.as_ref()).ok(),
                    attribute: attribute.clone(),
                },
                Method::Create => Operate::Create {
                    expr: expr_engine.compile(value.as_ref()).ok(),
                    attribute: attribute.clone(),
                },
                Method::Rename => Operate::Rename {
                    new_key: value.to_string(),
                    attribute: attribute.clone(),
                },
                Method::Remove => Operate::Remove {
                    attribute: attribute.clone(),
                },
            }
        })
        .collect::<Vec<_>>()
}
