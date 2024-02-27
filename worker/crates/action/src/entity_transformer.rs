use core::result::Result;
use std::{collections::HashMap, sync::Arc};

use anyhow::anyhow;
use rayon::prelude::*;
use rhai::Dynamic;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::debug;

use reearth_flow_eval_expr::engine::Engine;
use reearth_flow_workflow::graph::NodeProperty;

use crate::action::{ActionContext, ActionDataframe, ActionValue};
use crate::error::Error;
use crate::utils::convert_dataframe_to_scope_params;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct PropertySchema {
    transform_expr: String,
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
    let inputs = inputs.ok_or(Error::input("No Input"))?;
    let expr_engine = Arc::clone(&ctx.expr_engine);
    let ast = expr_engine.compile(props.transform_expr.as_str())?;
    let params = convert_dataframe_to_scope_params(&inputs);

    let mut output = ActionDataframe::new();
    for (port, data) in inputs {
        let data = match data {
            Some(data) => data,
            None => continue,
        };
        let processed_data = match data {
            ActionValue::Array(rows) => {
                // NOTE: Parallelization with a small number of cases will conversely slow down the process.
                match rows.len() {
                    0..=1000 => rows
                        .iter()
                        .map(|row| mapper(row, &ast, &params, Arc::clone(&expr_engine)))
                        .collect::<Vec<_>>(),
                    _ => rows
                        .par_iter()
                        .map(|row| mapper(row, &ast, &params, Arc::clone(&expr_engine)))
                        .collect::<Vec<_>>(),
                }
            }
            _ => continue,
        };
        output.insert(port, Some(ActionValue::Array(processed_data)));
    }
    Ok(output)
}

fn mapper(
    row: &ActionValue,
    expr: &rhai::AST,
    params: &HashMap<String, ActionValue>,
    expr_engine: Arc<Engine>,
) -> ActionValue {
    match row {
        ActionValue::Map(row) => {
            let scope = expr_engine.new_scope();
            for (k, v) in params {
                scope.set(k, v.clone().into());
            }
            for (k, v) in row {
                scope.set(k, v.clone().into());
            }
            let new_value = scope.eval_ast::<Dynamic>(expr);
            if let Ok(new_value) = new_value {
                if let Ok(ActionValue::Map(new_value)) = new_value.try_into() {
                    return ActionValue::Map(new_value);
                }
            }
            ActionValue::Map(row.clone())
        }
        _ => row.clone(),
    }
}
