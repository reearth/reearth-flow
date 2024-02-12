use core::result::Result;
use std::{collections::HashMap, sync::Arc};

use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::debug;

use reearth_flow_workflow::graph::NodeProperty;

use crate::action::{ActionContext, ActionDataframe, ActionValue, Port, DEFAULT_PORT};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct PropertySchema {
    conditions: Vec<Condition>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct Condition {
    expr: String,
    output_port: String,
}

impl TryFrom<NodeProperty> for PropertySchema {
    type Error = anyhow::Error;

    fn try_from(node_property: NodeProperty) -> Result<Self, anyhow::Error> {
        let value = Value::Object(node_property);
        serde_json::from_value(value).map_err(|e| {
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
    let input = inputs.get(DEFAULT_PORT).ok_or(anyhow!("No Default Port"))?;
    let input = input.as_ref().ok_or(anyhow!("No Value"))?;
    let expr_engine = Arc::clone(&ctx.expr_engine);

    let output = match input {
        ActionValue::Array(rows) => {
            let mut result = HashMap::<Port, Vec<ActionValue>>::new();
            for row in rows {
                match row {
                    ActionValue::Map(row) => {
                        for condition in &props.conditions {
                            let expr = &condition.expr;
                            let output_port = &condition.output_port;
                            let entry = result.entry(output_port.to_owned()).or_default();
                            let scope = expr_engine.new_scope();
                            for (k, v) in row {
                                scope.set(k, v.clone().into());
                            }
                            let eval = scope.eval::<bool>(expr)?;
                            if eval {
                                entry.push(ActionValue::Map(row.clone()));
                            }
                        }
                    }
                    _ => return Err(anyhow!("Invalid Input. supported only Map")),
                }
            }
            result
        }
        _ => return Err(anyhow!("Invalid Input. supported only Array")),
    };
    Ok(output
        .iter()
        .map(|(k, v)| (k.clone(), Some(ActionValue::Array(v.clone()))))
        .collect::<ActionDataframe>())
}
