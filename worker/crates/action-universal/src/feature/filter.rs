use std::sync::Arc;

use serde::{Deserialize, Serialize};

use reearth_flow_action::error::Error;
use reearth_flow_action::{
    ActionContext, ActionDataframe, ActionResult, AsyncAction, Dataframe, Feature, Port,
    DEFAULT_PORT, REJECTED_PORT,
};
use reearth_flow_common::collection;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FeatureFilter {
    conditions: Vec<Condition>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct Condition {
    expr: String,
    output_port: Port,
}

#[async_trait::async_trait]
#[typetag::serde(name = "FeatureFilter")]
impl AsyncAction for FeatureFilter {
    async fn run(&self, ctx: ActionContext, inputs: ActionDataframe) -> ActionResult {
        let input = inputs
            .get(&DEFAULT_PORT)
            .ok_or(Error::input("No Default Port"))?;
        let expr_engine = Arc::clone(&ctx.expr_engine);

        let mut result = ActionDataframe::new();
        for condition in &self.conditions {
            let expr = &condition.expr;
            let template_ast = expr_engine.compile(expr).map_err(Error::internal_runtime)?;
            let output_port = &condition.output_port;
            let filter = |row: &Feature| {
                let scope = expr_engine.new_scope();
                for (k, v) in &row.attributes {
                    scope.set(k.inner().as_str(), v.clone().into());
                }
                let eval = scope.eval_ast::<bool>(&template_ast);
                if let Ok(eval) = eval {
                    eval
                } else {
                    false
                }
            };
            let success = collection::filter(&input.features, filter);
            result.insert(output_port.clone(), Dataframe::new(success));
        }
        let failed = {
            let mut target = collection::vec_to_map(&input.features, |v| (v.to_string(), false));
            result.iter().for_each(|(_, v)| {
                let success = collection::vec_to_map(&v.features, |row| (row.to_string(), true));
                target.extend(success);
            });
            target
        };

        let failed = {
            collection::filter(&input.features, |v| {
                if let Some(failed) = failed.get(&v.to_string()) {
                    !*failed
                } else {
                    false
                }
            })
        };
        result.insert(REJECTED_PORT.to_owned(), Dataframe::new(failed));
        Ok(result)
    }
}
