use std::sync::Arc;

use anyhow::anyhow;
use rhai::Dynamic;
use serde::{Deserialize, Serialize};

use crate::action::{Action, ActionContext, ActionDataframe, ActionResult, ActionValue};
use crate::utils::convert_dataframe_to_scope_params;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DataframeTransformer {
    operations: Vec<Operation>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Operation {
    transform_expr: String,
    target_port: String,
}

#[async_trait::async_trait]
#[typetag::serde(name = "dataframeTransformer")]
impl Action for DataframeTransformer {
    async fn run(&self, ctx: ActionContext, inputs: Option<ActionDataframe>) -> ActionResult {
        let inputs = inputs.ok_or(anyhow!("No Input"))?;
        let expr_engine = Arc::clone(&ctx.expr_engine);
        let params = convert_dataframe_to_scope_params(&inputs);

        let mut output = ActionDataframe::new();
        for (port, data) in inputs {
            let data = match data {
                Some(data) => data,
                None => continue,
            };
            let operation = self
                .operations
                .iter()
                .find(|operation| operation.target_port == port);
            if operation.is_none() {
                output.insert(port, Some(data));
                continue;
            }
            let operation = operation.unwrap();
            let ast = expr_engine.compile(&operation.transform_expr)?;
            let scope = expr_engine.new_scope();
            for (k, v) in &params {
                scope.set(k, v.clone().into());
            }
            scope.set(&port, data.into());
            let new_value = scope.eval_ast::<Dynamic>(&ast)?;
            let new_value: ActionValue = new_value.try_into()?;
            output.insert(port, Some(new_value));
        }
        Ok(output)
    }
}
