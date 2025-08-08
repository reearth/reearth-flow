use std::{collections::HashMap, sync::Arc};

use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{Context, ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
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
pub(super) struct AttributeManagerFactory;

impl ProcessorFactory for AttributeManagerFactory {
    fn name(&self) -> &str {
        "AttributeManager"
    }

    fn description(&self) -> &str {
        "Create, Convert, Rename, and Remove Feature Attributes"
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
        let params: AttributeManagerParam = if let Some(with) = with.clone() {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                AttributeProcessorError::ManagerFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                AttributeProcessorError::ManagerFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(AttributeProcessorError::ManagerFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };

        let expr_engine = Arc::clone(&ctx.expr_engine);
        let operations = convert_single_operation(&params.operations, Arc::clone(&expr_engine))?;

        let process = AttributeManager {
            global_params: with,
            operations,
        };
        Ok(Box::new(process))
    }
}

#[derive(Debug, Clone)]
struct AttributeManager {
    global_params: Option<HashMap<String, serde_json::Value>>,
    operations: Vec<Operate>,
}

/// # AttributeManager Parameters
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct AttributeManagerParam {
    /// # Attribute Operations
    /// List of operations to perform on feature attributes (create, convert, rename, remove)
    operations: Vec<Operation>,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct Operation {
    /// # Attribute name
    attribute: String,
    /// # Operation to perform
    method: Method,
    /// # Value to use for the operation
    value: Option<Expr>,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
enum Method {
    Convert,
    Create,
    Rename,
    Remove,
}

#[derive(Debug, Clone)]
enum Operate {
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
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = process_feature(
            ctx.as_context(),
            &ctx.feature,
            &self.operations,
            Arc::clone(&ctx.expr_engine),
            &self.global_params,
        );
        fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
        Ok(())
    }

    fn finish(&self, _ctx: NodeContext, _fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "AttributeManager"
    }
}

fn process_feature(
    ctx: Context,
    feature: &Feature,
    operations: &[Operate],
    expr_engine: Arc<Engine>,
    global_params: &Option<HashMap<String, serde_json::Value>>,
) -> Feature {
    let mut result = feature.clone();
    for operation in operations {
        match operation {
            Operate::Convert { expr, attribute } => {
                let value = feature.get(attribute);
                if value.is_none() {
                    continue;
                }

                let scope = feature.new_scope(expr_engine.clone(), global_params);
                if let Some(expr) = expr {
                    let new_value = scope.eval_ast::<Dynamic>(expr);
                    if let Ok(new_value) = new_value {
                        if let Ok(new_value) = new_value.try_into() {
                            result.insert(attribute.clone(), new_value);
                        }
                    } else if let Err(e) = new_value {
                        ctx.event_hub
                            .warn_log(None, format!("convert error with: {e:?}"));
                    }
                }
            }
            Operate::Create { expr, attribute } => {
                let scope = feature.new_scope(expr_engine.clone(), global_params);
                if let Some(expr) = expr {
                    let new_value = scope.eval_ast::<Dynamic>(expr);
                    if let Ok(new_value) = new_value {
                        if let Ok(new_value) = new_value.try_into() {
                            result.insert(attribute.clone(), new_value);
                        }
                    } else if let Err(e) = new_value {
                        ctx.event_hub
                            .warn_log(None, format!("create error with: {e:?}"));
                    }
                }
            }
            Operate::Rename { new_key, attribute } => {
                if !feature.contains_key(attribute) || feature.contains_key(new_key) {
                    continue;
                }
                let value = feature.get(attribute);
                result.remove(attribute);
                result.insert(new_key.clone(), value.cloned().unwrap_or_default());
            }
            Operate::Remove { attribute } => {
                if !feature.contains_key(attribute) {
                    continue;
                }
                result.remove(attribute);
            }
        };
    }
    result
}

fn convert_single_operation(
    operations: &[Operation],
    expr_engine: Arc<Engine>,
) -> super::errors::Result<Vec<Operate>> {
    let mut result = Vec::new();
    for operation in operations.iter() {
        let method = &operation.method;
        let attribute = &operation.attribute;
        let expr = if let Some(expr) = operation
            .value
            .clone()
            .take_if(|_| matches!(method, Method::Convert | Method::Create))
        {
            Some(
                expr_engine
                    .compile(expr.as_ref())
                    .map_err(|e| AttributeProcessorError::ManagerFactory(format!("{e:?}")))?,
            )
        } else {
            None
        };
        let value = match method {
            Method::Convert => Operate::Convert {
                expr,
                attribute: attribute.clone(),
            },
            Method::Create => Operate::Create {
                expr,
                attribute: attribute.clone(),
            },
            Method::Rename => Operate::Rename {
                new_key: operation.value.clone().unwrap_or(Expr::new("")).to_string(),
                attribute: attribute.clone(),
            },
            Method::Remove => Operate::Remove {
                attribute: attribute.clone(),
            },
        };
        result.push(value);
    }
    Ok(result)
}
