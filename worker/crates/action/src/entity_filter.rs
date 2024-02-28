use core::result::Result;
use std::{collections::HashMap, sync::Arc};

use anyhow::anyhow;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::debug;

use crate::action::{ActionContext, ActionDataframe, ActionValue, Port, DEFAULT_PORT};
use crate::error::Error;
use crate::utils::convert_dataframe_to_scope_params;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct PropertySchema {
    conditions: Vec<Condition>,
}

property_schema!(PropertySchema);

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct Condition {
    expr: String,
    output_port: String,
}

pub(crate) async fn run(
    ctx: ActionContext,
    inputs: Option<ActionDataframe>,
) -> anyhow::Result<ActionDataframe> {
    let props = PropertySchema::try_from(ctx.node_property)?;
    debug!(?props, "read");
    let inputs = inputs.ok_or(Error::input("No Input"))?;
    let input = inputs
        .get(DEFAULT_PORT)
        .ok_or(Error::input("No Default Port"))?;
    let input = input.as_ref().ok_or(Error::input("No Value"))?;
    let expr_engine = Arc::clone(&ctx.expr_engine);
    let params = convert_dataframe_to_scope_params(&inputs);

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
            _ => return Err(Error::input("Invalid Input. supported only Array").into()),
        };
        result.insert(output_port.clone(), output);
    }
    Ok(result
        .iter()
        .map(|(k, v)| (k.clone(), Some(ActionValue::Array(v.clone()))))
        .collect::<ActionDataframe>())
}
