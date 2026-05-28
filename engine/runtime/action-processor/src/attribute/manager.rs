use std::{collections::HashMap, sync::Arc};

use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{Context, ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};

use reearth_flow_types::{Code, CompiledCode, Feature};
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
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let params: AttributeManagerParam = if let Some(with) = with {
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

        let operations = convert_single_operation(&params.operations)?;
        let process = AttributeManager { operations };
        Ok(Box::new(process))
    }
}

#[derive(Debug, Clone)]
struct AttributeManager {
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
    /// # Value
    /// Value to use for the operation
    value: Option<Code>,
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
        code: Option<CompiledCode>,
        attribute: String,
    },
    Create {
        code: Option<CompiledCode>,
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
        let env_vars = ctx.expr_engine.vars();
        let feature = process_feature(ctx.as_context(), &ctx.feature, &self.operations, env_vars);
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
        "AttributeManager"
    }
}

fn process_feature(
    ctx: Context,
    feature: &Feature,
    operations: &[Operate],
    env_vars: Arc<serde_json::Map<String, serde_json::Value>>,
) -> Feature {
    let mut result = feature.clone();
    for operation in operations {
        match operation {
            Operate::Convert { code, attribute } => {
                if feature.get(attribute).is_none() {
                    continue;
                }
                if let Some(code) = code {
                    match code.eval(feature, Arc::clone(&env_vars)) {
                        Ok(new_value) => {
                            result.insert(attribute.clone(), new_value);
                        }
                        Err(e) => {
                            ctx.event_hub
                                .warn_log(None, format!("convert error with: {e:?}"));
                        }
                    }
                }
            }
            Operate::Create { code, attribute } => {
                if let Some(code) = code {
                    match code.eval(feature, Arc::clone(&env_vars)) {
                        Ok(new_value) => {
                            result.insert(attribute.clone(), new_value);
                        }
                        Err(e) => {
                            ctx.event_hub
                                .warn_log(None, format!("create error with: {e:?}"));
                        }
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

fn convert_single_operation(operations: &[Operation]) -> super::errors::Result<Vec<Operate>> {
    let mut result = Vec::new();
    for operation in operations.iter() {
        let method = &operation.method;
        let attribute = &operation.attribute;
        let code = if let Some(code) = operation
            .value
            .clone()
            .take_if(|_| matches!(method, Method::Convert | Method::Create))
        {
            Some(
                code.compile()
                    .map_err(|e| AttributeProcessorError::ManagerFactory(format!("{e:?}")))?,
            )
        } else {
            None
        };
        let value = match method {
            Method::Convert => Operate::Convert {
                code,
                attribute: attribute.clone(),
            },
            Method::Create => Operate::Create {
                code,
                attribute: attribute.clone(),
            },
            Method::Rename => Operate::Rename {
                new_key: operation
                    .value
                    .as_ref()
                    .map(|c| c.value.clone())
                    .unwrap_or_default(),
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
