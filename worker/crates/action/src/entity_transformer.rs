use std::{collections::HashMap, sync::Arc};

use rayon::prelude::*;
use rhai::Dynamic;
use serde::{Deserialize, Serialize};

use reearth_flow_eval_expr::engine::Engine;

use crate::action::{Action, ActionContext, ActionDataframe, ActionResult, ActionValue};
use crate::error::Error;
use crate::utils::convert_dataframe_to_scope_params;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EntityTransformer {
    transform_expr: String,
}

#[async_trait::async_trait]
#[typetag::serde(name = "entityTransformer")]
impl Action for EntityTransformer {
    async fn run(&self, ctx: ActionContext, inputs: Option<ActionDataframe>) -> ActionResult {
        let inputs = inputs.ok_or(Error::input("No Input"))?;
        let expr_engine = Arc::clone(&ctx.expr_engine);
        let ast = expr_engine.compile(self.transform_expr.as_str())?;
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
                _ => {
                    output.insert(port, Some(data));
                    continue;
                }
            };
            output.insert(port, Some(ActionValue::Array(processed_data)));
        }
        Ok(output)
    }
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
