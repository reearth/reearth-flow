use core::result::Result;
use std::{collections::HashMap, sync::Arc};

use anyhow::anyhow;
use rayon::prelude::*;
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
    let input = inputs.get(DEFAULT_PORT).ok_or(anyhow!("No Default Port"))?;
    let input = input.as_ref().ok_or(anyhow!("No Value"))?;
    let expr_engine = Arc::clone(&ctx.expr_engine);
    let params = inputs
        .keys()
        .filter(|&key| *key != DEFAULT_PORT && inputs.get(key).unwrap().is_some())
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

    let mut result = HashMap::<Port, Vec<ActionValue>>::new();
    for condition in &props.conditions {
        let expr = &condition.expr;
        let template_ast = expr_engine.compile(expr)?;
        let output_port = &condition.output_port;
        let output = match input {
            ActionValue::Array(rows) => {
                let filter = |row: &ActionValue| {
                    if let ActionValue::Map(row) = row {
                        let scope = expr_engine.new_scope();
                        for (k, v) in &params {
                            scope.set(k, v.clone().into());
                        }
                        for (k, v) in row {
                            scope.set(k, v.clone().into());
                        }
                        let eval = scope.eval_ast::<bool>(&template_ast);
                        if let Ok(eval) = eval {
                            eval
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                };
                // NOTE: Parallelization with a small number of cases will conversely slow down the process.
                match rows.len() {
                    0..=1000 => rows
                        .iter()
                        .filter(|&row| filter(row))
                        .cloned()
                        .collect::<Vec<_>>(),
                    _ => rows
                        .par_iter()
                        .filter(|&row| filter(row))
                        .cloned()
                        .collect::<Vec<_>>(),
                }
            }
            _ => return Err(anyhow!("Invalid Input. supported only Array")),
        };
        result.insert(output_port.clone(), output);
    }
    Ok(result
        .iter()
        .map(|(k, v)| (k.clone(), Some(ActionValue::Array(v.clone()))))
        .collect::<ActionDataframe>())
}
