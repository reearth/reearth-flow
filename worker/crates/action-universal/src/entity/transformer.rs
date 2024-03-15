use std::{collections::HashMap, sync::Arc};

use rhai::Dynamic;
use serde::{Deserialize, Serialize};

use reearth_flow_action::error::Error;
use reearth_flow_action::utils::convert_dataframe_to_scope_params;
use reearth_flow_action::{Action, ActionContext, ActionDataframe, ActionResult, ActionValue};
use reearth_flow_common::collection;
use reearth_flow_eval_expr::engine::Engine;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EntityTransformer {
    transform_expr: String,
}

#[async_trait::async_trait]
#[typetag::serde(name = "EntityTransformer")]
impl Action for EntityTransformer {
    async fn run(&self, ctx: ActionContext, inputs: Option<ActionDataframe>) -> ActionResult {
        let inputs = inputs.ok_or(Error::input("No Input"))?;
        let expr_engine = Arc::clone(&ctx.expr_engine);
        let ast = expr_engine
            .compile(self.transform_expr.as_str())
            .map_err(Error::internal_runtime)?;
        let params = convert_dataframe_to_scope_params(&inputs);

        let mut output = ActionDataframe::new();
        for (port, data) in inputs {
            let data = match data {
                Some(data) => data,
                None => continue,
            };
            let processed_data = match data {
                ActionValue::Array(rows) => collection::map(&rows, |row| {
                    mapper(row, &ast, &params, Arc::clone(&expr_engine))
                }),
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
            scope.set("__ALL", serde_json::to_value(row.clone()).unwrap());
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
