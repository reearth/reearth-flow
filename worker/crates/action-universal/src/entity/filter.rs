use std::{collections::HashMap, sync::Arc};

use serde::{Deserialize, Serialize};

use reearth_flow_action::error::Error;
use reearth_flow_action::utils::convert_dataframe_to_scope_params;
use reearth_flow_action::{
    Action, ActionContext, ActionDataframe, ActionResult, ActionValue, Port, DEFAULT_PORT,
    REJECTED_PORT,
};
use reearth_flow_action_log::action_log;
use reearth_flow_action_log::span;
use reearth_flow_common::collection;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EntityFilter {
    conditions: Vec<Condition>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct Condition {
    expr: String,
    output_port: Port,
}

#[async_trait::async_trait]
#[typetag::serde(name = "EntityFilter")]
impl Action for EntityFilter {
    async fn run(&self, ctx: ActionContext, inputs: Option<ActionDataframe>) -> ActionResult {
        let inputs = inputs.ok_or(Error::input("No Input"))?;
        let input = inputs
            .get(&DEFAULT_PORT)
            .ok_or(Error::input("No Default Port"))?;
        let input = input.as_ref().ok_or(Error::input("No Value"))?;
        let expr_engine = Arc::clone(&ctx.expr_engine);
        let params = convert_dataframe_to_scope_params(&inputs);
        let span = span(
            ctx.root_span.clone(),
            "EntityFilter".to_string(),
            ctx.node_id.to_string(),
            ctx.node_name,
        );
        let logger = Arc::clone(&ctx.logger);

        let mut result = HashMap::<Port, Vec<ActionValue>>::new();
        for condition in &self.conditions {
            let expr = &condition.expr;
            let template_ast = expr_engine.compile(expr).map_err(Error::internal_runtime)?;
            let output_port = &condition.output_port;
            let success = match input {
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
                            action_log!(
                                parent: span,
                                logger,
                                "Invalid Input. supported only Map",
                            );
                            false
                        }
                    };
                    collection::filter(rows, filter)
                }
                _ => return Err(Error::input("Invalid Input. supported only Array")),
            };
            result.insert(output_port.clone(), success);
        }
        let failed = if let ActionValue::Array(failed) = &input {
            let mut target = collection::vec_to_map(failed, |v| (v.to_string(), false));
            result.iter().for_each(|(_, v)| {
                let success = collection::vec_to_map(v, |row| (row.to_string(), true));
                target.extend(success);
            });
            target
        } else {
            HashMap::new()
        };

        let failed = if let ActionValue::Array(all) = &input {
            collection::filter(all, |v| {
                if let Some(failed) = failed.get(&v.to_string()) {
                    !*failed
                } else {
                    false
                }
            })
        } else {
            vec![]
        };
        result.insert(REJECTED_PORT.to_owned(), failed);
        Ok(result
            .iter()
            .map(|(k, v)| (k.clone(), Some(ActionValue::Array(v.clone()))))
            .collect::<ActionDataframe>())
    }
}
