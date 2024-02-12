use core::result::Result;
use std::{collections::HashMap, sync::Arc};

use anyhow::anyhow;
use rhai::Dynamic;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::debug;

use reearth_flow_workflow::graph::NodeProperty;

use crate::action::{ActionContext, ActionDataframe, ActionValue};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct PropertySchema {
    operations: Vec<Operation>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Operation {
    pub(crate) attribute: String,
    pub(crate) method: Method,
    pub(crate) value: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) enum Method {
    #[serde(rename = "convert")]
    Convert,
    #[serde(rename = "rename")]
    Rename,
}

impl TryFrom<NodeProperty> for PropertySchema {
    type Error = anyhow::Error;

    fn try_from(node_property: NodeProperty) -> Result<Self, anyhow::Error> {
        serde_json::from_value(Value::Object(node_property)).map_err(|e| {
            anyhow!(
                "Failed to convert NodeProperty to PropertySchema with {}",
                e
            )
        })
    }
}

pub(crate) async fn run(
    ctx: ActionContext,
    inputs: Option<ActionDataframe>,
) -> anyhow::Result<ActionDataframe> {
    let props = PropertySchema::try_from(ctx.node_property)?;
    debug!(?props, "read");
    let inputs = inputs.ok_or(anyhow!("No Input"))?;
    let expr_engine = Arc::clone(&ctx.expr_engine);
    let params = inputs
        .keys()
        .filter(|&key| inputs.get(key).unwrap().is_some())
        .filter(|&key| {
            matches!(
                inputs.get(key).unwrap().clone().unwrap(),
                ActionValue::Bool(_)
                    | ActionValue::Number(_)
                    | ActionValue::String(_)
                    | ActionValue::Map(_)
            )
        })
        .map(|key| (key.to_owned(), inputs.get(key).unwrap().clone().unwrap()))
        .collect::<HashMap<_, _>>();

    let mut output = ActionDataframe::new();
    for (port, data) in inputs {
        let data = match data {
            Some(data) => data,
            None => continue,
        };
        let processed_data = match data {
            ActionValue::Array(rows) => {
                let mut processed_items = Vec::new();
                for row in rows {
                    match row {
                        ActionValue::Map(row) => {
                            let mut result = row.clone();
                            for operation in &props.operations {
                                let method = &operation.method;
                                let attribute = &operation.attribute;
                                let value = row.get(attribute).ok_or_else(|| {
                                    anyhow!("Attribute {} not found in the input", attribute)
                                })?;
                                match method {
                                    Method::Convert => {
                                        let expr = &operation.value;
                                        let scope = expr_engine.new_scope();
                                        for (k, v) in &row {
                                            scope.set(k, v.clone().into());
                                        }
                                        for (k, v) in &params {
                                            scope.set(k, v.clone().into());
                                        }
                                        let new_value = scope.eval::<Dynamic>(expr)?;
                                        result.insert(attribute.clone(), new_value.try_into()?);
                                    }
                                    Method::Rename => {
                                        let new_key = operation.value.clone();
                                        result.insert(new_key, value.clone());
                                    }
                                }
                            }
                            processed_items.push(ActionValue::Map(result));
                        }
                        _ => return Err(anyhow!("Invalid Input. supported only Map")),
                    }
                }
                ActionValue::Array(processed_items)
            }
            _ => continue,
        };
        output.insert(port, Some(processed_data));
    }
    Ok(output)
}
