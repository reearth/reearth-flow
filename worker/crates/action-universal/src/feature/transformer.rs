use std::{collections::HashMap, sync::Arc};

use rhai::Dynamic;
use serde::{Deserialize, Serialize};

use reearth_flow_action::error::Error;
use reearth_flow_action::{
    ActionContext, ActionDataframe, ActionResult, AsyncAction, Attribute, AttributeValue,
    Dataframe, Feature, Port,
};
use reearth_flow_common::collection;
use reearth_flow_eval_expr::engine::Engine;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FeatureTransformer {
    transforms: Vec<Transform>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct Transform {
    expr: String,
    target_port: Port,
}

#[async_trait::async_trait]
#[typetag::serde(name = "FeatureTransformer")]
impl AsyncAction for FeatureTransformer {
    async fn run(&self, ctx: ActionContext, inputs: ActionDataframe) -> ActionResult {
        let expr_engine = Arc::clone(&ctx.expr_engine);
        let transforms = self
            .transforms
            .iter()
            .map(|transform| (transform.target_port.clone(), transform.expr.clone()))
            .collect::<HashMap<_, _>>();

        let mut output = ActionDataframe::new();
        for (port, data) in inputs {
            let expr = match transforms.get(&port) {
                Some(expr) => expr,
                None => continue,
            };
            let ast = expr_engine.compile(expr).map_err(Error::internal_runtime)?;
            let processed_data = collection::map(&data.features, |row| {
                mapper(row, &ast, Arc::clone(&expr_engine))
            });
            output.insert(port, Dataframe::new(processed_data));
        }
        Ok(output)
    }
}

fn mapper(row: &Feature, expr: &rhai::AST, expr_engine: Arc<Engine>) -> Feature {
    let scope = expr_engine.new_scope();
    for (k, v) in row.attributes.iter() {
        scope.set(k.inner().as_str(), v.clone().into());
    }
    scope.set("__all", serde_json::to_value(row.clone()).unwrap());
    let new_value = scope.eval_ast::<Dynamic>(expr);
    if let Ok(new_value) = new_value {
        if let Ok(AttributeValue::Map(new_value)) = new_value.try_into() {
            return Feature::new_with_attributes(
                new_value
                    .iter()
                    .map(|(k, v)| (Attribute::new(k.clone()), v.clone()))
                    .collect(),
            );
        }
    }
    row.clone()
}
