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
pub(super) struct Operation {
    transform_expr: String,
    target_port: Port,
}

#[async_trait::async_trait]
#[typetag::serde(name = "DataframeTransformer")]
impl Action for DataframeTransformer {
    async fn run(&self, ctx: ActionContext, inputs: Option<ActionDataframe>) -> ActionResult {
        let inputs = inputs.unwrap_or_default();
        let expr_engine = Arc::clone(&ctx.expr_engine);
        let params = convert_dataframe_to_scope_params(&inputs);

        let mut output = ActionDataframe::new();
        let ports = inputs.keys().cloned().collect::<Vec<_>>();
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
            let scope = expr_engine.new_scope();
            for (k, v) in &params {
                scope.set(k, v.clone().into());
            }
            scope.set(port.as_ref(), data.into());
            let new_value = scope
                .eval::<Dynamic>(&operation.transform_expr)
                .map_err(Error::internal_runtime)?;
            let new_value: ActionValue = new_value.try_into()?;
            output.insert(port.clone(), Some(new_value));
        }
        for operation in &self.operations {
            if ports.contains(&operation.target_port) {
                continue;
            }
            let scope = expr_engine.new_scope();
            for (k, v) in &params {
                scope.set(k, v.clone().into());
            }
            let new_value = scope
                .eval::<Dynamic>(operation.transform_expr.as_str())
                .map_err(Error::internal_runtime)?;
            let new_value: ActionValue = new_value.try_into()?;
            output.insert(operation.target_port.clone(), Some(new_value));
        }
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_dataframe_transformer() {
        // Create a sample input dataframe
        let mut input_data = ActionDataframe::new();
        input_data.insert(
            Port::new("port1"),
            Some(ActionValue::String("value1".to_owned())),
        );
        input_data.insert(
            Port::new("port2"),
            Some(ActionValue::Number(serde_json::Number::from(42))),
        );

        // Create a sample context
        let ctx = ActionContext::default();

        // Create a sample dataframe transformer
        let transformer = DataframeTransformer {
            operations: vec![
                Operation {
                    transform_expr: r#"`${env.get("port1")}hogehoge`"#.to_owned(),
                    target_port: Port::new("port1"),
                },
                Operation {
                    transform_expr: r#"env.get("port2") * 2"#.to_owned(),
                    target_port: Port::new("port2"),
                },
            ],
        };

        // Run the transformer
        let result = transformer.run(ctx, Some(input_data)).await;

        // Check the result
        assert!(result.is_ok());
        let output_data = result.unwrap();
        assert_eq!(
            output_data.get(&Port::new("port1")),
            Some(&Some(ActionValue::String("value1hogehoge".to_owned())))
        );
        assert_eq!(
            output_data.get(&Port::new("port2")),
            Some(&Some(ActionValue::Number(serde_json::Number::from(84))))
        );
    }

    #[tokio::test]
    async fn test_dataframe_transformer_no_input() {
        // Create a sample context
        let ctx = ActionContext::default();

        // Create a sample dataframe transformer
        let transformer = DataframeTransformer {
            operations: vec![
                Operation {
                    transform_expr: r#""hogehoge""#.to_owned(),
                    target_port: Port::new("port1"),
                },
                Operation {
                    transform_expr: "15".to_owned(),
                    target_port: Port::new("port2"),
                },
            ],
        };

        // Run the transformer without input
        let result = transformer.run(ctx, None).await;

        // Check the result
        assert!(result.is_ok());
        let output_data = result.unwrap();
        assert_eq!(
            output_data.get(&Port::new("port1")),
            Some(&Some(ActionValue::String("hogehoge".to_owned())))
        );
        assert_eq!(
            output_data.get(&Port::new("port2")),
            Some(&Some(ActionValue::Number(serde_json::Number::from(15))))
        );
    }
}
