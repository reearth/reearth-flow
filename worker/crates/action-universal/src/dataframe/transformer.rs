use std::sync::Arc;

use rhai::Dynamic;
use serde::{Deserialize, Serialize};

use reearth_flow_action::utils::convert_dataframe_to_scope_params;
use reearth_flow_action::{
    error::Error, Action, ActionContext, ActionDataframe, ActionResult, ActionValue, Port,
};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DataframeTransformer {
    operations: Vec<Operation>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Operation {
    transform_expr: String,
    target_port: Port,
}

#[async_trait::async_trait]
#[typetag::serde(name = "DataframeTransformer")]
impl Action for DataframeTransformer {
    async fn run(&self, ctx: ActionContext, inputs: Option<ActionDataframe>) -> ActionResult {
        let inputs = inputs.ok_or(Error::input("No Input"))?;
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
                output.insert(port.clone(), Some(data));
                continue;
            }
            let operation = operation.unwrap();
            let ast = expr_engine
                .compile(&operation.transform_expr)
                .map_err(Error::internal_runtime)?;
            let scope = expr_engine.new_scope();
            for (k, v) in &params {
                scope.set(k, v.clone().into());
            }
            scope.set(port.as_ref(), data.into());
            let new_value = scope
                .eval_ast::<Dynamic>(&ast)
                .map_err(Error::internal_runtime)?;
            let new_value: ActionValue = new_value.try_into()?;
            output.insert(port.clone(), Some(new_value));
        }
        Ok(output)
    }
}
