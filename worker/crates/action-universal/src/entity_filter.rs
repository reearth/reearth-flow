use std::{collections::HashMap, sync::Arc};

use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use reearth_flow_action::error::Error;
use reearth_flow_action::utils::convert_dataframe_to_scope_params;
use reearth_flow_action::{
    Action, ActionContext, ActionDataframe, ActionResult, ActionValue, Port, DEFAULT_PORT,
};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EntityFilter {
    conditions: Vec<Condition>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct Condition {
    expr: String,
    output_port: String,
}

#[async_trait::async_trait]
#[typetag::serde(name = "entityFilter")]
impl Action for EntityFilter {
    async fn run(&self, ctx: ActionContext, inputs: Option<ActionDataframe>) -> ActionResult {
        let inputs = inputs.ok_or(Error::input("No Input"))?;
        let input = inputs
            .get(DEFAULT_PORT)
            .ok_or(Error::input("No Default Port"))?;
        let input = input.as_ref().ok_or(Error::input("No Value"))?;
        let expr_engine = Arc::clone(&ctx.expr_engine);
        let params = convert_dataframe_to_scope_params(&inputs);

        let mut result = HashMap::<Port, Vec<ActionValue>>::new();
        for condition in &self.conditions {
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
}
