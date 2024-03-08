use std::sync::Arc;

use rhai::Dynamic;
use serde::{Deserialize, Serialize};

use reearth_flow_action::utils::convert_dataframe_to_scope_params;
use reearth_flow_action::{Action, ActionContext, ActionDataframe, ActionResult, ActionValue};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RhaiCaller {
    callers: Vec<Caller>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct Caller {
    script: String,
    output_port: String,
}

#[async_trait::async_trait]
#[typetag::serde(name = "RhaiCaller")]
impl Action for RhaiCaller {
    async fn run(&self, ctx: ActionContext, inputs: Option<ActionDataframe>) -> ActionResult {
        let inputs = inputs.unwrap_or_default();
        let expr_engine = Arc::clone(&ctx.expr_engine);
        let params = convert_dataframe_to_scope_params(&inputs);

        let mut output = ActionDataframe::new();
        for caller in &self.callers {
            let ast = expr_engine.compile(&caller.script)?;
            let scope = expr_engine.new_scope();
            for (k, v) in &params {
                scope.set(k, v.clone().into());
            }
            let new_value = scope.eval_ast::<Dynamic>(&ast)?;
            let new_value: ActionValue = new_value.try_into()?;
            output.insert(caller.output_port.to_owned(), Some(new_value));
        }
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use reearth_flow_action::{Action, ActionContext, ActionDataframe, ActionValue};

    #[tokio::test]
    async fn test_rhai_caller_run() {
        // Create a sample ActionContext and inputs
        let ctx = ActionContext::default();
        let inputs = Some(ActionDataframe::new());

        let caller = RhaiCaller {
            callers: vec![
                Caller {
                    script: r#"
                    fn add(x, y) {
                        let a = [x, y];
                        a.for_each(|| this *= 2);
                        a.reduce(|sum| sum + this, 0)
                    }
                    add(1, 2)
                    "#
                    .to_owned(),
                    output_port: "output1".to_owned(),
                },
                Caller {
                    script: r#"[
                        #{ data: "hoge" },
                        #{ data: "fuga" }
                    ]"#
                    .to_owned(),
                    output_port: "output2".to_owned(),
                },
            ],
        };

        // Call the run method and assert the result
        let result = caller.run(ctx, inputs).await;
        assert!(result.is_ok());

        // Assert the output dataframe
        let output = result.unwrap();
        assert_eq!(output.len(), 2);
        assert_eq!(
            output.get("output1"),
            Some(&Some(ActionValue::Number(serde_json::Number::from(6))))
        );
        assert_eq!(
            output.get("output2"),
            Some(&Some(ActionValue::Array(vec![
                ActionValue::Map(
                    vec![("data".to_owned(), ActionValue::String("hoge".to_owned())),]
                        .into_iter()
                        .collect::<HashMap<_, _>>()
                ),
                ActionValue::Map(
                    vec![("data".to_owned(), ActionValue::String("fuga".to_owned())),]
                        .into_iter()
                        .collect::<HashMap<_, _>>()
                ),
            ])))
        );
    }

    #[test]
    fn test_convert_dataframe_to_scope_params() {
        // Create a sample ActionDataframe
        let dataframe = ActionDataframe::new(); // Replace with your actual dataframe

        // Call the convert_dataframe_to_scope_params function and assert the result
        let params = convert_dataframe_to_scope_params(&dataframe);
        assert!(params.is_empty()); // Replace with your actual assertions
    }
}
