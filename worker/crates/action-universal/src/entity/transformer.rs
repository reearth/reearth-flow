use std::{collections::HashMap, sync::Arc};

use rhai::Dynamic;
use serde::{Deserialize, Serialize};

use reearth_flow_action::error::Error;
use reearth_flow_action::utils::convert_dataframe_to_scope_params;
use reearth_flow_action::{
    Action, ActionContext, ActionDataframe, ActionResult, ActionValue, Port,
};
use reearth_flow_common::collection;
use reearth_flow_eval_expr::engine::Engine;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EntityTransformer {
    transforms: Vec<Transform>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct Transform {
    expr: String,
    target_port: Port,
}

#[async_trait::async_trait]
#[typetag::serde(name = "EntityTransformer")]
impl Action for EntityTransformer {
    async fn run(&self, ctx: ActionContext, inputs: Option<ActionDataframe>) -> ActionResult {
        let inputs = inputs.ok_or(Error::input("No Input"))?;
        let expr_engine = Arc::clone(&ctx.expr_engine);
        let params = convert_dataframe_to_scope_params(&inputs);
        let transforms = self
            .transforms
            .iter()
            .map(|transform| (transform.target_port.clone(), transform.expr.clone()))
            .collect::<HashMap<_, _>>();

        let mut output = ActionDataframe::new();
        for (port, data) in inputs {
            let data = match data {
                Some(data) => data,
                None => continue,
            };
            let expr = match transforms.get(&port) {
                Some(expr) => expr,
                None => continue,
            };
            let ast = expr_engine.compile(expr).map_err(Error::internal_runtime)?;
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
            scope.set("__all", serde_json::to_value(row.clone()).unwrap());
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

#[cfg(test)]
mod tests {
    use super::*;
    use reearth_flow_action::ActionContext;

    #[tokio::test]
    async fn test_entity_transformer() {
        // Create a sample input dataframe
        let mut input_dataframe = ActionDataframe::new();
        let mut input_data = HashMap::new();
        input_data.insert(
            Port::new("port1"),
            Some(ActionValue::Array(vec![ActionValue::Map(HashMap::new())])),
        );

        input_dataframe.insert(
            Port::new("port1"),
            Some(ActionValue::Array(vec![ActionValue::Map(HashMap::new())])),
        );

        // Create a sample action context
        let action_context = ActionContext::default();

        // Create a sample EntityTransformer
        let entity_transformer = EntityTransformer {
            transforms: vec![Transform {
                expr: "expr1".to_string(),
                target_port: Port::new("port1"),
            }],
        };

        // Run the action
        let result = entity_transformer
            .run(action_context, Some(input_dataframe))
            .await;

        // Check the result
        assert!(result.is_ok());
        let output_dataframe = result.unwrap();
        assert_eq!(output_dataframe.len(), 1);
        assert!(output_dataframe.contains_key(&Port::new("port1")));
        let output_data = output_dataframe.get(&Port::new("port1")).unwrap();
        assert!(output_data.is_some());
        let output_data = output_data.clone().unwrap();
        assert_eq!(
            output_data,
            ActionValue::Array(vec![ActionValue::Map(HashMap::new())])
        );
    }
}
