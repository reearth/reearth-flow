use std::{collections::HashMap, sync::Arc};

use rhai::Dynamic;
use serde::{Deserialize, Serialize};

use reearth_flow_action::{
    error::Error, ActionContext, ActionDataframe, ActionResult, AsyncAction, Attribute,
    AttributeValue, Dataframe, Feature, Port,
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
impl AsyncAction for DataframeTransformer {
    async fn run(&self, ctx: ActionContext, inputs: ActionDataframe) -> ActionResult {
        let expr_engine = Arc::clone(&ctx.expr_engine);

        let mut output = ActionDataframe::new();
        let ports = inputs.keys().cloned().collect::<Vec<_>>();
        for (port, data) in inputs {
            let operation = self
                .operations
                .iter()
                .find(|operation| operation.target_port == port);
            if operation.is_none() {
                output.insert(port.clone(), data);
                continue;
            }
            let operation = operation.unwrap();
            let scope = expr_engine.new_scope();
            scope.set(port.as_ref(), AttributeValue::Array(data.into()).into());
            let new_value = scope
                .eval::<Dynamic>(&operation.transform_expr)
                .map_err(Error::internal_runtime)?;
            let new_value: AttributeValue = new_value.try_into()?;
            let AttributeValue::Array(new_value) = new_value else {
                return Err(Error::internal_runtime("Invalid output type"));
            };
            let new_value = new_value
                .iter()
                .flat_map(|v| match v {
                    AttributeValue::Map(v) => Some(v.clone().into()),
                    _ => None,
                })
                .collect::<Vec<_>>();
            output.insert(port.clone(), Dataframe::new(new_value));
        }
        for operation in &self.operations {
            if ports.contains(&operation.target_port) {
                continue;
            }
            let scope = expr_engine.new_scope();
            let new_value = scope
                .eval::<Dynamic>(operation.transform_expr.as_str())
                .map_err(Error::internal_runtime)?;
            let new_value: AttributeValue = new_value.try_into()?;
            let AttributeValue::Array(new_value) = new_value else {
                return Err(Error::internal_runtime("Invalid output type"));
            };
            let new_value = new_value
                .iter()
                .flat_map(|v| match v {
                    AttributeValue::Map(v) => Some(Feature::new_with_attributes(
                        v.iter()
                            .map(|(k, v)| (Attribute::new(k), v.clone()))
                            .collect::<HashMap<_, _>>(),
                    )),
                    _ => None,
                })
                .collect::<Vec<_>>();
            output.insert(operation.target_port.clone(), Dataframe::new(new_value));
        }
        Ok(output)
    }
}
