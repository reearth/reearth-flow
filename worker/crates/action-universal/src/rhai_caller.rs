use std::sync::Arc;

use rhai::Dynamic;
use serde::{Deserialize, Serialize};

use reearth_flow_action::{
    error::Error, ActionContext, ActionDataframe, ActionResult, AsyncAction, AttributeValue,
    Dataframe, Feature, Port,
};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RhaiCaller {
    callers: Vec<Caller>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct Caller {
    script: String,
    output_port: Port,
}

#[async_trait::async_trait]
#[typetag::serde(name = "RhaiCaller")]
impl AsyncAction for RhaiCaller {
    async fn run(&self, ctx: ActionContext, _inputs: ActionDataframe) -> ActionResult {
        let expr_engine = Arc::clone(&ctx.expr_engine);
        let mut output = ActionDataframe::new();

        for caller in &self.callers {
            let ast = expr_engine
                .compile(&caller.script)
                .map_err(Error::internal_runtime)?;
            let scope = expr_engine.new_scope();
            let new_value = scope
                .eval_ast::<Dynamic>(&ast)
                .map_err(Error::internal_runtime)?;
            let new_value: AttributeValue = new_value.try_into()?;
            let feature: Feature = new_value.into();
            output.insert(caller.output_port.to_owned(), Dataframe::new(vec![feature]));
        }
        Ok(output)
    }
}
